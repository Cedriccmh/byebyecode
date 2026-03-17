# ByeByeCode 项目开发指南

> ⚠️ **第一优先级规则**：**未经 JohnYe 本人明确同意，严禁向上游仓库（upstream）提交 PR！** 所有代码变更默认只推送到 fork 仓库（origin），需要提交 PR 时必须先询问并获得许可。

## 项目概述

ByeByeCode 是一个 Rust 编写的 Claude Code 状态栏增强工具，用于显示 88code/Packy 中转站的套餐用量和订阅信息。

- **仓库**: https://github.com/byebye-code/byebyecode
- **语言**: Rust
- **用途**: Claude Code 状态栏插件

## Git 分支说明

### 远程仓库

| 远程名 | 地址 | 说明 |
|--------|------|------|
| `origin` | https://github.com/johnnyee/byebyecode | Fork 仓库（个人开发用） |
| `upstream` | https://github.com/byebye-code/byebyecode | 上游官方仓库 |

### 分支列表

| 分支名 | 用途 | 对应 PR | 状态 |
|--------|------|---------|------|
| `main` | 主分支 | - | ✅ 活跃 |
| `fix/issue-9-subscription-usage-display` | Issue #9 修复 | PR #10 | ✅ 已合并 |
| `feature/progress-bar-usage-display` | 进度条功能 | PR #11 | ✅ 已合并 |
| `fix/skip-free-subscription` | 跳过 FREE 套餐 | PR #12 | ✅ 已合并 |
| `feature/simplify-subscription-display` | 精简订阅显示格式 | PR #15 | ✅ 已合并 |
| `feature/support-new-88code-domains` | 支持新域名 88code.ai | PR #16 | ✅ 已合并 |
| `feature/sort-subscriptions-by-remaining-days` | 按剩余天数排序 | PR #18 | ✅ 已合并 |
| `fix/issue-26-usage-api-fallback` | Usage API 返回 null 时 fallback | PR #27 | ✅ 已合并 |

### 分支工作流

### 两种运行模式

1. **管道模式**（Claude Code 调用）：stdin 接收 `InputData` JSON → 输出状态栏字符串
2. **交互模式**（终端直接运行）：显示 TUI 主菜单/配置器（需 `tui` feature）

### 核心模块

- **`src/core/segments/`** — 每个段（segment）是独立的数据收集器，实现 `Segment` trait（`collect` + `id`）。段不做渲染，只返回 `SegmentData { primary, secondary, metadata }`
- **`src/core/statusline.rs`** — 渲染引擎，负责 ANSI 颜色、Powerline 箭头分隔符、智能换行
- **`src/api/`** — 88code/Packy API 客户端（`client.rs`）、数据结构与转换（`mod.rs`）、文件缓存（`cache.rs`）
- **`src/config/`** — 配置加载（`loader.rs`）、类型定义（`types.rs`）、模型上下文限制（`models.rs`）

### 段的配置传递

段的 `options` 配置不通过 `Segment::collect(input)` 传递（trait 签名只有 `InputData`），而是在 `statusline.rs:collect_all_segments()` 中从 `SegmentConfig.options: HashMap<String, serde_json::Value>` 读取，通过 builder 模式注入段实例（参考 `GitSegment::with_sha()`、`ContextWindowSegment::with_show_tokens()`）。

`byebyecode_usage` 和 `byebyecode_subscription` 不走 Segment trait，直接接收 `Config` 参数。

### 配置文件

| 文件 | 路径 | 格式 | 说明 |
|------|------|------|------|
| byebyecode 配置 | `~/.claude/byebyecode/config.toml` | TOML | 段启用/颜色/options |
| Claude Code 设置 | `~/.claude/settings.json` | JSON | statusLine command 指向 |
| 模型配置 | `~/.claude/byebyecode/models.toml` | TOML | 模型上下文窗口大小 |

### 88code API 关键行为

- `/api/usage` — 用量查询，**必须传 `model` 参数**否则返回 FREE 套餐数据
- `/api/subscription` — 订阅查询，返回所有套餐（包括 PAYGO）
- Usage API 的 `subscriptionEntityList` **不返回 PAYGO**，需从 Subscription API fallback
- Usage API 可能返回全 null 字段，代码有 fallback 到 Subscription API 的逻辑
- 套餐扣费优先级：FREE → PLUS/PRO/MAX → PAYGO（Claude Code 跳过 FREE）

## 自定义功能（custom-patches/）

本 fork 相对 upstream 的自定义修改，通过 patch 文件管理。

### 功能清单

1. **Context Window 进度条** — 彩色进度条显示上下文使用率（`show_tokens`, `color_low/mid/high`）
2. **Model 显示修复** — Opus 4.6 1M 等具体模型的精确匹配（`src/config/models.rs`）
3. **Effort 显示** — 比较 transcript `/effort` 时间戳 vs `settings.json` mtime 取更新者，fallback env `CLAUDE_CODE_EFFORT_LEVEL` → 默认 "auto"（`show_effort` 配置项）

### Patch 管理

- Patch 基于 `upstream/main` 生成：`git diff upstream/main..HEAD -- src/ > custom-patches/all-custom.patch`
- 自定义修改已作为 commit 存在于 fork 中，patch 文件仅作为紧急回退用
- 详见 `custom-patches/README.md`

### ESET 注意事项

ESET 杀毒软件可能阻止 `byebyecode.exe` 运行（误报为不可信程序）。需将 `~/.claude/byebyecode/byebyecode.exe` 添加到 ESET 排除列表。

### Cargo 编译环境

Git Bash 中 `cargo` 不在 PATH，需手动添加：
```bash
# 同步上游代码
git fetch upstream
git checkout main
git merge upstream/main

# 创建功能分支
git checkout -b feature/xxx

# 提交 PR 后合并，删除本地分支
git branch -d feature/xxx
```

## 项目结构

```
byebyecode/
├── src/
│   ├── api/
│   │   ├── mod.rs          # API 数据结构定义
│   │   ├── client.rs       # API 客户端实现
│   │   └── cache.rs        # 缓存管理
│   ├── core/
│   │   └── segments/
│   │       ├── byebyecode_usage.rs        # 用量显示段
│   │       └── byebyecode_subscription.rs # 订阅显示段
│   ├── config/             # 配置管理
│   └── main.rs             # 入口
├── Cargo.toml              # 项目依赖
└── npm/                    # npm 发布相关
```

## 构建命令

### Windows 环境构建

Windows 需要 Visual Studio Build Tools：

```powershell
# 安装 MSVC Build Tools
choco install visualstudio2022buildtools visualstudio2022-workload-vctools -y

# 构建
cargo build --release
```

**注意**: Git 的 `link.exe` 可能与 MSVC 的 `link.exe` 冲突，需要配置 `.cargo/config.toml`：

```toml
[target.x86_64-pc-windows-msvc]
linker = "D:\\Program Files\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\\link.exe"
```

### Linux/macOS

```bash
cargo build --release
```

## 本地测试

1. 编译项目：`cargo build --release`
2. 修改 `~/.claude/settings.json`：
```json
{
  "statusLine": {
    "command": "D:/Dev/OpenSource/byebyecode/target/release/byebyecode.exe",
    "type": "command"
  }
}
```
3. 重启 Claude Code

## 状态栏段（Segments）配置

byebyecode 支持多个状态栏段，可以根据需要启用或禁用。

### 可用段列表

| 段 ID | 名称 | 说明 |
|-------|------|------|
| `model` | 模型 | 显示当前使用的 AI 模型 |
| `directory` | 目录 | 显示当前工作目录 |
| `git` | Git | 显示 Git 分支和状态 |
| `context_window` | 上下文窗口 | 显示上下文窗口使用情况 |
| `usage` | 用量 | 显示 API 用量（原生） |
| `cost` | 费用 | 显示会话费用 |
| `session` | 会话 | 显示会话信息 |
| `output_style` | 输出样式 | 显示当前输出样式 |
| `update` | 更新 | 显示更新提示 |
| `byebyecode_usage` | 88code 用量 | 显示 88code/Packy 套餐用量（带进度条） |
| `byebyecode_subscription` | 88code 订阅 | 显示所有订阅套餐详情（含重置次数） |
| `byebyecode_status` | 88code 状态 | 显示 88code 服务状态 |

### 88code 专用段详解

#### `byebyecode_usage` - 用量段

显示当前正在扣费的套餐用量，带进度条可视化：

```
88code $34.53/$50 ▓▓▓▓▓▓▓░░░
```

**特性**：
- 自动跳过 FREE 套餐（FREE 不支持 Claude Code）
- 额度用完时显示重置提示
- 支持 88code 和 Packy 两种服务

#### `byebyecode_subscription` - 订阅段

显示所有活跃订阅的详细信息：

```
订阅 PLUS ¥198/月付 (可重置2次, 剩余53天) | PAYGO ¥66/年付 (剩余989天)
```

**特性**：
- 显示所有活跃套餐
- 包含重置次数、剩余天数
- 每个套餐有独特的颜色标识
- 仅支持 88code（Packy 不显示）

### 配置示例

完整配置示例（`~/.claude/settings.json`）：

```json
{
  "statusLine": {
    "command": "byebyecode",
    "type": "command",
    "config": {
      "segments": [
        { "id": "model", "enabled": true },
        { "id": "directory", "enabled": true },
        { "id": "git", "enabled": true },
        { "id": "context_window", "enabled": true },
        { "id": "byebyecode_usage", "enabled": true },
        { "id": "byebyecode_subscription", "enabled": true }
      ]
    }
  }
}
```

### 自定义 API 配置

如果需要自定义 API 地址或密钥，可以在段的 `options` 中配置：

```json
{
  "id": "byebyecode_usage",
  "enabled": true,
  "options": {
    "api_key": "your-api-key",
    "usage_url": "https://www.88code.org/api/usage"
  }
}
```

**说明**：
- 如果不配置 `api_key`，会自动从 `~/.claude/settings.json` 的 `ANTHROPIC_AUTH_TOKEN` 读取
- 如果不配置 `usage_url`，会根据 `ANTHROPIC_BASE_URL` 自动判断使用 88code 或 Packy

## 代码规范

### 格式检查

提交前必须运行 `cargo fmt`，CI 会检查格式：

```bash
cargo fmt           # 自动格式化
cargo fmt -- --check  # 检查格式（CI 使用）
```

### 函数签名格式

多参数函数需要换行：

```rust
// 正确
pub fn get_subscriptions(
    &self,
    model: Option<&str>,
) -> Result<Vec<SubscriptionData>, Box<dyn std::error::Error>> {

// 错误（CI 会失败）
pub fn get_subscriptions(&self, model: Option<&str>) -> Result<Vec<SubscriptionData>, Box<dyn std::error::Error>> {
```

## 88code API 说明

### 套餐扣费逻辑

| 套餐 | 支持 Claude Code | 支持 Codex | 扣费顺序 |
|------|------------------|------------|----------|
| FREE | ❌ 不支持 | ✅ 支持 | 1️⃣ 最先 |
| PLUS/PRO/MAX | ✅ 支持 | ✅ 支持 | 2️⃣ 其次 |
| PAYGO | ✅ 支持 | ✅ 支持 | 3️⃣ 最后 |

### API 接口

- `/api/usage` - 获取用量数据
- `/api/subscription` - 获取订阅信息

**重要**: 需要传入 `model` 参数才能获取正确套餐的用量，否则 API 默认返回 FREE 套餐数据。

### API 返回结构

```json
{
  "creditLimit": 20.0,        // 顶层数据（可能是 FREE）
  "currentCredits": 20.0,
  "subscriptionEntityList": [  // 实际套餐数据在这里
    {
      "subscriptionName": "FREE",
      "creditLimit": 20,
      "currentCredits": 20
    },
    {
      "subscriptionName": "PLUS",
      "creditLimit": 50,
      "currentCredits": 45.47   // 正在使用的套餐
    }
  ]
}
```

---

## ✅ 已解决：Usage API 不返回 PAYGO 套餐（2025-12-11 修复）

### 问题描述

当用户同时拥有 PLUS 和 PAYGO 套餐，且 PLUS 额度用完后，状态栏应该显示 PAYGO 的额度，但实际显示的是 FREE 套餐的额度（$0/$20）。

### 根本原因

**Usage API 的 `subscriptionEntityList` 不返回 PAYGO 套餐数据！**

#### Usage API（用量查询）

**请求**：
```bash
curl -s "https://www.88code.ai/api/usage" -X POST \
  -H "Authorization: Bearer 88_xxx" \
  -H "Content-Type: application/json" \
  -d '{"model":"claude-opus-4-5-20250514"}'
```

**返回的 `subscriptionEntityList`**：
```json
[
  {
    "subscriptionName": "FREE",
    "currentCredits": 20.0,
    "creditLimit": 20.0,
    "isActive": true
  },
  {
    "subscriptionName": "PLUS",
    "currentCredits": -0.0666407615,
    "creditLimit": 50.0,
    "isActive": true
  }
]
```

**⚠️ 没有返回 PAYGO！**

#### Subscription API（订阅查询）

**请求**：
```bash
curl -s "https://www.88code.ai/api/subscription" -X POST \
  -H "Authorization: Bearer 88_xxx" \
  -H "Content-Type: application/json" \
  -d '{"model":"claude-opus-4-5-20250514"}'
```

**返回**（摘要）：
```json
[
  {"subscriptionPlanName": "FREE", "currentCredits": 20.0, "isActive": true, "remainingDays": 28},
  {"subscriptionPlanName": "PLUS", "currentCredits": -0.07, "isActive": true, "remainingDays": 27},
  {"subscriptionPlanName": "PLUS", "currentCredits": 50.0, "isActive": true, "remainingDays": 53},
  {"subscriptionPlanName": "PAYGO", "currentCredits": 64.03, "isActive": true, "remainingDays": 988}
]
```

**✅ Subscription API 返回了 PAYGO！**

### 解决方案（已实现）

采用 **方案 A：从 Subscription API 获取 PAYGO 额度**

**实现逻辑**：
```
1. 调用 Usage API 获取 PLUS 等套餐数据
2. 判断是否所有 PLUS 用完（currentCredits <= 0）
3. 如果用完，调用 Subscription API
4. 从订阅列表中找到有余额的 PAYGO 套餐
5. 显示 "PAYGO $XX.XX"（蓝色，无进度条）
```

**关键代码** (`src/core/segments/byebyecode_usage.rs`):
```rust
if usage.is_exhausted() {
    let subscriptions = fetch_subscriptions_sync(&api_key, &subscription_url, Some(model_id));

    if let Some(subs) = subscriptions {
        // 仅 88code 服务支持 PAYGO 回退
        if service_name == "88code" {
            let paygo = subs.iter()
                .filter(|s| s.is_active)
                .filter(|s| s.plan_name.to_uppercase() == "PAYGO")
                .find(|s| s.current_credits > 0.0);

            if let Some(paygo_sub) = paygo {
                // 显示 PAYGO 剩余额度（蓝色）
                return Some(SegmentData {
                    primary: format!("PAYGO ${:.2}", paygo_sub.current_credits),
                    ...
                });
            }
        }
    }
}
```

**性能优化**：
- 订阅数据使用 5 分钟缓存，避免频繁 API 调用
- API 失败时降级到过期缓存

### 状态

✅ **已解决**（2025-12-11）

**限制**：PAYGO 无法显示进度条，因为 Subscription API 不返回 `creditLimit`（总额度）字段。

---

## ✅ 已解决：Usage API 返回 null 值导致状态栏无法显示（2026-01-05 修复）

### 问题描述（Issue #26）

88code 的 `/api/usage` 接口返回的 `creditLimit`、`currentCredits`、`subscriptionEntityList` 字段全是 `null`，导致状态栏无法正确显示用量。

**API 返回示例**：
```json
{
  "code": 0,
  "data": {
    "creditLimit": null,
    "currentCredits": null,
    "subscriptionEntityList": null,
    "totalCost": 296.295528,
    "totalTokens": 216080709
  }
}
```

### 根本原因

88code 的 Usage API 不再返回用量相关字段，只返回基础的 key 统计信息。但 `/api/subscription` 接口仍然能正常返回订阅数据。

### 解决方案（双重保障策略）

采用 **优先原接口，失败时 fallback** 的方案：

```
┌─────────────────────────────────────────────────────────────┐
│  Step 1: 调用 /api/usage                                    │
│  ↓                                                          │
│  检查关键字段是否有效 (is_valid())：                         │
│  ├─ creditLimit > 0                                         │
│  └─ subscriptionEntityList 非空                             │
│                                                             │
│  ✅ 有效 → 使用原方案逻辑（代码不变）                         │
│  ❌ 无效 → fallback 到 Step 2                               │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  Step 2: Fallback - 调用 /api/subscription                  │
│  ↓                                                          │
│  from_subscriptions() 构造等价的 UsageData                   │
│  ├─ 筛选活跃套餐 (is_active && status == "活跃中")           │
│  ├─ 按扣费优先级排序 (PLUS > PAYGO > FREE)                  │
│  └─ 跳过 FREE，找第一个有消费的套餐                          │
│  ↓                                                          │
│  正常显示进度条 ✅                                           │
└─────────────────────────────────────────────────────────────┘
```

### 代码修改

1. **`src/api/mod.rs`**：
   - 添加自定义反序列化函数 `deserialize_null_as_zero` 和 `deserialize_null_as_empty_vec` 处理 null 值
   - 新增 `is_valid()` 方法判断 usage 数据是否有效
   - 新增 `SubscriptionPlan` 结构体解析嵌套的 `creditLimit`
   - 新增 `from_subscriptions()` 方法从订阅数据构造等价的 UsageData

2. **`src/api/client.rs`**：
   - 在 `get_usage()` 中加入 fallback 逻辑

3. **`src/core/segments/byebyecode_usage.rs`**：
   - 修复 `fetch_usage_with_cache()` 传入正确的 `subscription_url`

### 优势

- **向后兼容**：88code 修复接口后自动恢复原方案
- **当前可用**：接口没修复时 subscription 方案兜底
- **最小改动**：原方案代码基本不动，只加判断层
- **风险隔离**：新方案只在原方案失败时才启用

### 状态

✅ **已解决**（2026-01-05，PR #27）

---

## 🚨 待解决：Privnode API 返回数据无法正确显示账户余额（2025-12-11）

### 问题描述

使用 Privnode 中转站时，状态栏显示 `relay $10.92/$1`，与实际账户数据不符：
- **实际当前余额**：$14.01
- **实际历史消耗**：$11.01
- **状态栏显示**：`$10.92/$1`（已用/总额）

### API 返回数据分析

**请求**：
```bash
curl -s "https://privnode.com/api/usage/token/" \
  -H "Authorization: Bearer sk-xxx"
```

**返回**：
```json
{
  "code": true,
  "data": {
    "expires_at": 0,
    "model_limits": {},
    "model_limits_enabled": false,
    "name": "251113",
    "object": "token_usage",
    "total_available": -5007103,
    "total_granted": 500000,
    "total_used": 5507103,
    "unlimited_quota": true
  },
  "message": "ok"
}
```

### 字段分析

| 字段 | 值 | 转换后（÷500000） | 含义 |
|------|-----|------------------|------|
| `total_used` | 5507103 | **$11.01** | 历史消耗 ✓ 正确 |
| `total_granted` | 500000 | **$1.00** | 初始赠送额度（不是账户总额） |
| `total_available` | -5007103 | **-$10.01** | 负数，计算值（granted - used） |
| `unlimited_quota` | true | - | 无限额度账户 |

### 问题根因

1. **`total_granted` 只返回初始赠送额度（$1）**，不是用户充值后的账户总额度
2. **缺少"当前账户余额"字段**：用户实际余额 $14.01 不在 API 返回中
3. **`total_available` 计算方式有问题**：`granted - used = $1 - $11.01 = -$10.01`，对于充值账户无意义
4. **`unlimited_quota: true` 时**：`total_granted` 和 `total_available` 无法反映真实账户状态

### 期望的 API 返回

为了正确显示账户余额，建议 API 返回以下字段：

```json
{
  "data": {
    "total_used": 5507103,        // 历史消耗（保持不变）
    "total_balance": 7005000,     // 当前账户余额：$14.01 × 500000
    "total_granted": 12512103,    // 账户总额度（充值+赠送）：余额+已用
    "total_available": 7005000,   // 可用额度 = 当前余额
    "unlimited_quota": true
  }
}
```

或者添加新字段：

```json
{
  "data": {
    "account_balance": 7005000,   // 新增：账户余额（$14.01 × 500000）
    "total_used": 5507103,
    "total_granted": 500000,      // 可以保持为初始赠送
    "unlimited_quota": true
  }
}
```

### 影响范围

- byebyecode 状态栏无法正确显示 Privnode 用户的账户余额
- 进度条显示异常（已用 $11 / 总额 $1 = 1100%）
- 用户无法通过状态栏了解真实的账户状态

### 临时解决方案

在 Privnode 修复 API 之前，byebyecode 可以：
1. 当 `unlimited_quota: true` 且 `total_available < 0` 时，只显示已用金额
2. 不显示误导性的总额度和进度条

### 状态

🔴 **待 Privnode 修复** - 需要 API 返回正确的账户余额字段

---

## 已完成的功能

### Issue #9 修复 (PR #10, #12)

**问题**: 状态栏始终显示 Free 套餐用量（$0/$20），即使 Plus 套餐正在被扣费。

**解决方案**:
1. 解析 `subscriptionEntityList` 获取真实套餐数据
2. Claude Code 环境下跳过 FREE 套餐
3. 选择第一个有消费的非 FREE 套餐显示

**关键代码** (`src/api/mod.rs`):
```rust
let active_subscription = self
    .subscription_entity_list
    .iter()
    .filter(|s| s.is_active)
    .filter(|s| s.subscription_name.to_uppercase() != "FREE") // 跳过 FREE
    .find(|s| s.current_credits < s.credit_limit);
```

### 进度条功能 (PR #11)

用进度条替代冗余的文字显示：

**改进前**: `$13.86/$50 剩$36.13`
**改进后**: `$13.86/$50 ▓▓▓░░░░░░░`

**关键代码** (`src/core/segments/byebyecode_usage.rs`):
```rust
let bar_length = 10;
let filled = ((percentage / 100.0) * bar_length as f64).round() as usize;
let empty = bar_length - filled;
let progress_bar = format!("{}{}", "▓".repeat(filled), "░".repeat(empty));
```

## PR 提交清单

提交 PR 前确保：

- [ ] `cargo fmt` 格式化代码
- [ ] `cargo build --release` 编译通过
- [ ] 本地测试功能正常
- [ ] 只提交必要的代码文件（不要提交 `.cargo/`、`build.ps1` 等本地配置）
- [ ] commit message 使用中文描述

## 已提交的 PR

| PR | 状态 | 内容 |
|----|------|------|
| #10 | ✅ 已合并 | 修复状态栏错误显示 Free 套餐用量的问题 |
| #11 | ✅ 已合并 | 用进度条可视化用量显示 |
| #12 | ✅ 已合并 | Claude Code 环境下跳过 FREE 套餐 |
| #27 | ✅ 已合并 | 修复 Issue #26：Usage API 返回 null 时 fallback 到 Subscription API |

## 常见问题

### Windows 编译报错 `linking with link.exe failed`

Git 的 `link.exe` 干扰了 MSVC 的 `link.exe`。解决方案：

1. 创建 `.cargo/config.toml` 指定正确的 linker 路径
2. 或设置 `LIB` 和 `PATH` 环境变量指向 MSVC 工具链

### CI 格式检查失败

运行 `cargo fmt` 后重新提交。

### 状态栏显示 FREE 套餐用量

确保代码包含跳过 FREE 的逻辑（PR #12）。

---

## 项目审查报告（2025-01-13）

本章节包含对 byebyecode 项目的全面审查，涵盖 UI/UX 优化建议、已发现的潜在 Bug、关键文件清单及实施建议。

### 📊 审查概览

- **审查日期**: 2025-01-13
- **审查范围**: UI/UX、架构设计、Bug 排查
- **发现数量**: 9 项 UI 优化建议、9 个潜在 Bug
- **关键文件**: 8 个核心文件需重点关注

---

## 🎨 UI 优化与高级感提升建议

### 🔴 P0 - 必须优化（严重影响用户体验）

#### 1. 额度用完：视觉警示强化

**当前问题**：
\`\`\`
❌ "已用完 提示：你有其他套餐可用"
\`\`\`
- 纯文字提示，无颜色/图标
- 缺少行动指引（"手动重置" vs "切换套餐"）

**优化方案**：
\`\`\`rust
// src/core/segments/byebyecode_usage.rs:107-125

// 额度用完时
primary: format!("⚠️ 已用完 ${}/${}", used, total)  // 红色背景 + 感叹号图标
secondary: match has_reset_times {
    true => format!("→ 可重置×{} 点击重置", reset_count),  // 行动指引
    false => "→ 切换至其他套餐".to_string(),
}

// 应用危险色
metadata.insert("danger_mode".to_string(), "true".to_string());
\`\`\`

---

### 🟡 P1 - 应该优化（用户体验改进）

#### 4. 错误提示改进

**当前**：API 失败显示 "未配置密钥"（可能是网络错误）

**改进**：
\`\`\`rust
match fetch_usage_sync(...) {
    Ok(usage) => usage,
    Err(e) => {
        let error_msg = if e.to_string().contains("timeout") {
            "⏱️ 网络超时"
        } else if e.to_string().contains("401") {
            "🔑 密钥无效"
        } else {
            format!("❌ API错误: {}", e)
        };
        return Some(SegmentData {
            primary: error_msg,
            secondary: String::new(),
        });
    }
}
\`\`\`

---

### 🟢 P2 - 可以优化（锦上添花）

#### 5. 响应式布局

\`\`\`rust
// 根据终端宽度自动切换精简模式
let terminal_width = terminal::size().map(|(w, _)| w).unwrap_or(80);
let compact_mode = terminal_width < 80;

if compact_mode {
    // 只显示当前扣费套餐
    // 缩短文字格式
}
\`\`\`

#### 6. 快过期警示

\`\`\`rust
// 订阅段：剩余天数 < 7 天时高亮显示
let days_color = if sub.remaining_days <= 7 {
    AnsiColor::Color16 { c16: 9 }  // 红色
} else if sub.remaining_days <= 30 {
    AnsiColor::Color16 { c16: 11 } // 黄色
} else {
    AnsiColor::Color16 { c16: 7 }  // 白色
};
\`\`\`

#### 7. 配置项扩展

\`\`\`toml
[byebyecode_usage.options]
show_progress_bar = true
show_percentage = true
compact_mode = false
warning_threshold = 80  # 百分比超过 80% 显示黄色

[byebyecode_subscription.options]
show_reset_times = true
show_days_threshold = 30  # 只在剩余天数<30天时显示
compact_mode = false
\`\`\`

---

## 🐛 已发现的潜在 Bug

### 🔴 高严重性（可能导致 panic 或崩溃）

#### Bug #1: 货币计算可能溢出

**位置**：\`src/api/mod.rs:173\`

**问题**：浮点数乘 100 后转 u64，超过 u64::MAX 会 panic。

\`\`\`rust
// 当前代码
self.used_tokens = (used_credits * 100.0).max(0.0) as u64;

// 修复方案
self.used_tokens = (used_credits * 100.0)
    .max(0.0)
    .min(u64::MAX as f64) as u64;
\`\`\`

#### Bug #2: unwrap() 导致 panic

**位置**：\`src/core/segments/byebyecode_usage.rs\` 多处

**问题**：如果 model 为 None 会 panic。

\`\`\`rust
// 当前代码
let model_id = &input.model.id;

// 修复方案
let model_id = input.model.as_ref().map(|m| m.id.as_str());
\`\`\`

#### Bug #3: API 响应状态未验证

**位置**：\`src/api/client.rs:43-44\`

**问题**：只检查 HTTP 状态码，未检查业务状态码（\`code\` 字段）。

\`\`\`rust
// 当前代码
if !response.status().is_success() {
    return Err(format!("API request failed: {}", response.status()).into());
}

// 修复方案
let resp: ResponseDTO<Code88UsageData> = serde_json::from_str(&response_text)?;
if resp.code != 0 {  // 假设 0 表示成功
    return Err(format!("API error: {}", resp.message).into());
}
\`\`\`

---

### 🟡 中等严重性（数据不一致或逻辑错误）

#### Bug #4: 浮点数精度问题

**位置**：\`src/api/mod.rs:167-168\`

**问题**：连续浮点运算可能累积误差。

\`\`\`rust
// 当前代码
self.percentage_used = (used_credits / credit_limit * 100.0).clamp(0.0, 100.0);

// 修复方案
self.percentage_used = ((used_credits / credit_limit) * 10000.0).round() / 100.0;
// 保留两位小数
\`\`\`

#### Bug #5: 订阅过滤边界错误

**位置**：\`src/core/segments/byebyecode_subscription.rs:120\`

**问题**：\`remaining_days == 0\` 当天仍然有效，不应过滤。

\`\`\`rust
// 当前代码
.filter(|sub| sub.is_active && sub.remaining_days > 0)

// 修复方案
.filter(|sub| sub.is_active && sub.remaining_days >= 0)
\`\`\`

#### Bug #6: 缓存 URL 硬编码

**位置**：\`src/api/cache.rs:120-142\`

**问题**：与实际使用的 88code API 不一致，缓存机制无法工作。

\`\`\`rust
// 当前代码（硬编码）
subscription_url: "https://api.cometix.cn/v1/billing/subscription/list"

// 修复方案：从配置或 Claude settings 读取
\`\`\`

---

### 🟢 低严重性（改进机会）

#### Bug #7: 线程安全隐患

**位置**：\`src/api/cache.rs:113-152\`

**问题**：多个并发刷新可能竞争写入缓存文件。

**建议**：使用文件锁或原子操作。

#### Bug #8: 配置错误提示不清晰

**位置**：\`src/core/segments/byebyecode_usage.rs:48-52\`

**当前**：
\`\`\`rust
primary: "未配置密钥".to_string(),
\`\`\`

**建议**：
\`\`\`rust
primary: "未配置密钥 (检查 ~/.claude/settings.json)".to_string(),
\`\`\`

#### Bug #9: URL 判断逻辑可能误判

**位置**：\`src/api/mod.rs:302-310\`

**问题**：\`rainapp.top\` 应该使用其原始域名，而非重定向到 \`88code.ai\`。

\`\`\`rust
// 当前代码
if base_url.contains("88code.ai") || base_url.contains("rainapp.top") {
    Some("https://www.88code.ai/api/usage".to_string())
}

// 修复方案
if base_url.contains("rainapp.top") {
    Some(format!("{}/api/usage", base_url))  // 保持原域名
} else if base_url.contains("88code.ai") {
    Some("https://www.88code.ai/api/usage".to_string())
}
\`\`\`

---

## 📁 关键文件清单（按优先级）

### 🎨 UI 优化相关

1. **\`src/core/segments/byebyecode_usage.rs\`** (178 行)
   - 用量段完整逻辑：进度条、百分比计算、状态色

2. **\`src/core/segments/byebyecode_subscription.rs\`** (182 行)
   - 订阅段实现：颜色生成、格式化、排序

3. **\`src/core/statusline.rs\`** (522 行)
   - 渲染引擎：ANSI 颜色、Powerline 箭头

### 🐛 Bug 修复相关

4. **\`src/api/mod.rs\`** (312 行)
   - 货币计算溢出、浮点精度问题

5. **\`src/api/client.rs\`** (121 行)
   - API 状态码验证、错误处理

6. **\`src/api/cache.rs\`** (152 行)
   - 缓存 URL 硬编码、线程安全

### ⚙️ 配置与架构

7. **\`src/config/types.rs\`** (420 行)
   - 配置结构定义、颜色类型

8. **\`src/ui/themes/theme_default.rs\`** (233 行)
   - 默认主题配置、图标语义化

---

## 💡 实施建议与优先级

### 📊 优化收益评估

| 优化项 | 实施难度 | 用户体验提升 | 建议优先级 |
|--------|---------|-------------|-----------|
| 百分比优先 + 状态色 | ⭐⭐ | ⭐⭐⭐⭐⭐ | 🔴 P0 |
| 订阅段精简格式 | ⭐⭐⭐ | ⭐⭐⭐⭐ | 🔴 P0 |
| 额度用完视觉警示 | ⭐⭐ | ⭐⭐⭐⭐ | 🔴 P0 |
| 货币计算溢出修复 | ⭐ | ⭐⭐⭐⭐⭐ | 🔴 P0 |
| 加载状态可视化 | ⭐ | ⭐⭐⭐ | 🟡 P1 |
| 错误提示改进 | ⭐ | ⭐⭐⭐ | 🟡 P1 |
| 响应式布局 | ⭐⭐⭐⭐ | ⭐⭐ | 🟢 P2 |

### 🎯 推荐实施路径

**第一阶段（优先修复）**：
1. 修复货币计算溢出 bug（5分钟）
2. 修复订阅过滤边界错误（2分钟）
3. 实现状态色系统（30分钟）
4. 百分比优先显示（15分钟）

**第二阶段（用户体验提升）**：
5. 精简订阅段格式（1小时）
6. 额度用完视觉警示（30分钟）
7. 加载状态可视化（20分钟）

**第三阶段（按需优化）**：
8. 响应式布局
9. 配置项扩展
10. 其他低优先级优化
