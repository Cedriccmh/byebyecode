# Custom Patches

byebyecode upstream 升级后重新应用自定义功能的 patch 文件。

## Patch 文件

| 文件 | 说明 |
|------|------|
| `context-window-progressbar.patch` | context window 进度条 + statusline 配置读取 |
| `all-custom.patch` | 全部自定义修改（进度条 + model 修复 + effort 显示） |

**重要**：patch 基于 `upstream/main` 生成，不是 `HEAD`。

## 升级流程

```bash
# 0. 确保 cargo 在 PATH 中
export PATH="/c/Users/Administrator/.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin:$PATH"

# 1. 合并 upstream（当前自定义修改已在 commit 中，不需要先保存 patch）
git fetch upstream && git merge upstream/main

# 2. 如果有冲突，解决冲突后继续
# 如果冲突无法解决，可以 reset 后用 patch 重新应用：
#   git reset --hard upstream/main
#   git apply custom-patches/all-custom.patch

# 3. 编译部署
cargo build --release
cp target/release/byebyecode.exe ~/.claude/byebyecode/byebyecode.exe

# 4. 重新生成 patch 快照（基于 upstream）
git diff upstream/main..HEAD -- src/ > custom-patches/all-custom.patch
```

## 自定义功能清单

### 1. Context Window 进度条
- 彩色进度条显示（低/中/高分别对应绿/黄/红）
- 可配置 `show_tokens`、`color_low`/`color_mid`/`color_high`

### 2. Model 显示修复
- 添加 `opus-4-6[1m]` → "Opus 4.6 1M" 等具体模式
- 保留 `[1m]` → "1M Context" 通用兜底
- 可在 `~/.claude/byebyecode/models.toml` 自定义新模式

### 3. Effort 显示
- Fallback 链：transcript `/effort` → `CLAUDE_CODE_EFFORT_LEVEL` env → `settings.json effortLevel` → 默认 "auto"
- 在 model segment 显示：`Opus 4.6 1M · max`
- 可配置 `show_effort = true/false`

**Claude Code effort 存储行为**（v2.1.76 实测）：
- `auto`：删除 settings.json 的 `effortLevel` 字段
- `max`：不写 settings.json（session-only, Opus 4.6 only），仅记录在 transcript
- `low/medium/high`：写入 settings.json `effortLevel`
- Alt+P 切换 max 时不写 settings.json 也不写 transcript，是唯一无法检测的边界情况

## Claude Code statusline JSON 结构（v2.1.76）

Claude Code 发给 statusline command 的 stdin JSON **不包含 effort**，完整字段：

```
session_id, transcript_path, cwd, session_name, model{id, display_name},
workspace{current_dir, project_dir, added_dirs}, version, output_style{name},
cost{total_cost_usd, ...}, context_window{context_window_size, used_percentage,
remaining_percentage, current_usage{...}}, exceeds_200k_tokens
```
