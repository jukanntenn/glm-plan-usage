[English](README.md) | [中文](README_zh.md)

# GLM 使用量状态插件

一个用于 Claude Code 的插件，在状态栏显示 GLM（智谱/ZAI）算力套餐的使用量统计。

## 功能特性

- **实时使用量追踪**: 显示 Token 和 MCP 使用百分比
- **颜色警告提示**: 绿色 (0-79%)、黄色 (80-94%)、红色 (95-100%)
- **智能缓存**: 5 分钟缓存减少 API 调用
- **自动平台检测**: 支持 ZAI 和智谱平台
- **优雅降级**: API 错误时静默失败

## 安装

### 通过 npm 安装（推荐）

```bash
npm install -g glm-plan-usage
glm-plan-usage --init
```

### 手动编译

```bash
git clone https://github.com/your-repo/glm-plan-usage.git
cd glm-plan-usage
cargo build --release
mkdir -p ~/.claude/glm-plan-usage
cp target/release/glm-plan-usage ~/.claude/glm-plan-usage/
~/.claude/glm-plan-usage/glm-plan-usage --init
```

## 配置

### 1. 设置环境变量

添加到 `~/.bashrc` 或 `~/.zshrc`：

**智谱平台：**
```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

**ZAI 平台：**
```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://api.z.ai/api/paas/v4/"
```

### 2. 配置 Claude Code

编辑 `~/.config/claude-code/settings.json`：

```json
{
  "statusLine": {
    "type": "command",
    "command": "glm-plan-usage",
    "padding": 0
  }
}
```

手动安装使用：`"command": "~/.claude/glm-plan-usage/glm-plan-usage"`

### 3. 重启 Claude Code

状态栏将显示：`T:42% M:15%`

## 输出格式

```
T:42% M:15%
│   │  └─ MCP 使用量（30 天窗口）
│   └─ Token 使用量（5 小时窗口）
└─ 百分比
```

## 配置选项

编辑 `~/.claude/glm-plan-usage/config.toml`：

```toml
[style]
separator = " | "

[[segments]]
id = "glm_usage"
enabled = true

[segments.colors]
text = { c256 = 109 }  # 绿色（256 色调色板）

[segments.styles]
text_bold = true

[api]
timeout_ms = 5000
retry_attempts = 2

[cache]
enabled = true
ttl_seconds = 300  # 5 分钟
```

## 命令行选项

```bash
--init        初始化配置文件
--verbose     启用详细输出
--no-cache    禁用缓存
--help        显示帮助
```

## 故障排查

**没有输出？**
```bash
# 检查环境变量
echo $ANTHROPIC_AUTH_TOKEN

# 使用详细模式测试
glm-plan-usage --verbose < input.json
```

**颜色不显示？**
```bash
# 测试终端颜色
echo -e "\x1b[38;5;109m绿色\x1b[0m"
```

## 更新

**通过 npm：**
```bash
npm update -g glm-plan-usage
```

**手动更新：**
```bash
cargo build --release
cp target/release/glm-plan-usage ~/.claude/glm-plan-usage/
```

## 文档

- [快速入门指南](guides/QUICK_START.md)
- [实现说明](guides/IMPLEMENTATION_NOTES.md)
- [NPM 发布指南](guides/NPM_PUBLISHING_GUIDE.md)

## 许可证

MIT
