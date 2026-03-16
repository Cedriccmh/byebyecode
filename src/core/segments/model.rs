use super::{Segment, SegmentData};
use crate::config::{InputData, ModelConfig, SegmentId};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::SystemTime;

pub struct ModelSegment {
    show_effort: bool,
}

impl Default for ModelSegment {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelSegment {
    pub fn new() -> Self {
        Self { show_effort: true }
    }

    pub fn with_show_effort(mut self, show: bool) -> Self {
        self.show_effort = show;
        self
    }
}

impl Segment for ModelSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let mut metadata = HashMap::new();
        metadata.insert("model_id".to_string(), input.model.id.clone());
        metadata.insert("display_name".to_string(), input.model.display_name.clone());

        let model_name = self.format_model_name(&input.model.id, &input.model.display_name);

        let primary = if self.show_effort {
            let effort = resolve_effort_level(&input.transcript_path);
            metadata.insert("effort".to_string(), effort.clone());
            format!("{} · {}", model_name, effort)
        } else {
            model_name
        };

        Some(SegmentData {
            primary,
            secondary: String::new(),
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::Model
    }
}

impl ModelSegment {
    fn format_model_name(&self, id: &str, display_name: &str) -> String {
        let model_config = ModelConfig::load();

        // Try to get display name from external config first
        if let Some(config_name) = model_config.get_display_name(id) {
            config_name
        } else {
            // Fallback to Claude Code's official display_name for unrecognized models
            display_name.to_string()
        }
    }
}

/// Valid effort levels recognized by Claude Code.
const VALID_EFFORTS: &[&str] = &["low", "medium", "high", "max", "auto"];

/// Parse the last /effort command from the transcript JSONL file.
/// Returns (effort_level, iso8601_timestamp).
fn parse_effort_from_transcript<P: AsRef<Path>>(transcript_path: P) -> Option<(String, String)> {
    let path = transcript_path.as_ref();
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_default();

    let mut last_effort: Option<(String, String)> = None;

    for line in &lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Check for /effort command — must be a real user command entry, not tool results
        if line.contains("<command-name>/effort</command-name>") {
            if let Some((effort, ts)) = extract_effort_from_user_command(line) {
                last_effort = Some((effort, ts));
            }
        }

        // Check for queue-operation with /effort
        if line.contains("\"queue-operation\"") && line.contains("/effort ") {
            if let Some((effort, ts)) = extract_queue_effort(line) {
                last_effort = Some((effort, ts));
            }
        }
    }

    last_effort
}

/// Resolve effort level by comparing transcript and settings.json timestamps.
/// Whichever was modified more recently wins.
/// Fallback: env var → default "auto".
fn resolve_effort_level(transcript_path: &str) -> String {
    let transcript_result = parse_effort_from_transcript(transcript_path);
    let settings_result = read_effort_from_settings();

    match (&transcript_result, &settings_result) {
        (Some((t_effort, t_ts)), Some((s_effort, s_mtime))) => {
            // Both exist — compare timestamps
            if iso_ts_to_epoch(t_ts) > s_mtime.duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0) {
                t_effort.clone()
            } else {
                s_effort.clone()
            }
        }
        (Some((t_effort, _)), None) => t_effort.clone(),
        (None, Some((s_effort, _))) => s_effort.clone(),
        (None, None) => {
            // Env var fallback
            if let Ok(val) = env::var("CLAUDE_CODE_EFFORT_LEVEL") {
                let effort = val.trim().to_lowercase();
                if VALID_EFFORTS.contains(&effort.as_str()) {
                    return effort;
                }
            }
            "auto".to_string()
        }
    }
}

/// Read effortLevel from ~/.claude/settings.json.
/// Returns (effort_level, file_mtime).
/// If effortLevel key is absent (meaning "auto"), returns ("auto", mtime).
fn read_effort_from_settings() -> Option<(String, SystemTime)> {
    let home = dirs::home_dir()?;
    let settings_path = home.join(".claude").join("settings.json");
    let mtime = fs::metadata(&settings_path).ok()?.modified().ok()?;
    let content = fs::read_to_string(&settings_path).ok()?;
    let settings: serde_json::Value = serde_json::from_str(&content).ok()?;

    match settings.get("effortLevel").and_then(|v| v.as_str()) {
        Some(effort) => {
            let effort = effort.trim().to_lowercase();
            if VALID_EFFORTS.contains(&effort.as_str()) {
                Some((effort, mtime))
            } else {
                Some(("auto".to_string(), mtime))
            }
        }
        // No effortLevel key means "auto", but still return mtime
        // so we can compare with transcript
        None => Some(("auto".to_string(), mtime)),
    }
}

/// Convert ISO 8601 timestamp (e.g. "2026-03-16T04:52:11.745Z") to epoch milliseconds.
fn iso_ts_to_epoch(ts: &str) -> u128 {
    // Parse manually: YYYY-MM-DDThh:mm:ss.mmmZ
    // Use chrono-free approach for simplicity
    let ts = ts.trim().trim_end_matches('Z');
    let parts: Vec<&str> = ts.splitn(2, 'T').collect();
    if parts.len() != 2 {
        return 0;
    }
    let date_parts: Vec<u64> = parts[0].split('-').filter_map(|s| s.parse().ok()).collect();
    let time_str = parts[1];
    let time_parts: Vec<&str> = time_str.splitn(2, '.').collect();
    let hms: Vec<u64> = time_parts[0].split(':').filter_map(|s| s.parse().ok()).collect();
    let millis: u64 = if time_parts.len() > 1 {
        time_parts[1].parse().unwrap_or(0)
    } else {
        0
    };

    if date_parts.len() != 3 || hms.len() != 3 {
        return 0;
    }

    let (year, month, day) = (date_parts[0], date_parts[1], date_parts[2]);
    let (hour, min, sec) = (hms[0], hms[1], hms[2]);

    // Days from epoch (1970-01-01) — simplified calculation
    let mut days: u64 = 0;
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }
    let month_days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        days += month_days[m as usize];
        if m == 2 && is_leap_year(year) {
            days += 1;
        }
    }
    days += day - 1;

    let epoch_ms = ((days * 86400 + hour * 3600 + min * 60 + sec) * 1000 + millis) as u128;
    epoch_ms
}

fn is_leap_year(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Extract effort level and timestamp from a user command entry.
/// Returns (effort, timestamp).
fn extract_effort_from_user_command(line: &str) -> Option<(String, String)> {
    let entry: serde_json::Value = serde_json::from_str(line).ok()?;

    // Must be a "user" type entry
    if entry.get("type")?.as_str()? != "user" {
        return None;
    }

    // message.content must be a string (direct user command), not an array (tool_result)
    let content = entry.get("message")?.get("content")?.as_str()?;

    // Must contain the /effort command tag
    if !content.contains("<command-name>/effort</command-name>") {
        return None;
    }

    // Extract args
    let start_tag = "<command-args>";
    let end_tag = "</command-args>";
    let start = content.find(start_tag)? + start_tag.len();
    let end = content[start..].find(end_tag)? + start;
    let args = content[start..end].trim().to_lowercase();

    if args.is_empty() {
        return None;
    }

    // Validate it's a known effort level
    if VALID_EFFORTS.contains(&args.as_str()) {
        let ts = entry.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string();
        Some((args, ts))
    } else {
        None
    }
}

/// Extract effort level and timestamp from queue-operation content like "/effort auto".
fn extract_queue_effort(line: &str) -> Option<(String, String)> {
    let entry: serde_json::Value = serde_json::from_str(line).ok()?;
    let content = entry.get("content")?.as_str()?;
    let effort = content.strip_prefix("/effort ")?.trim().to_lowercase();
    if effort.is_empty() {
        return None;
    }
    if VALID_EFFORTS.contains(&effort.as_str()) {
        let ts = entry.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string();
        Some((effort, ts))
    } else {
        None
    }
}
