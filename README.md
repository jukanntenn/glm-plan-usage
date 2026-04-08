# glm-plan-usage

简体中文 | [English](README_en.md)

一个用于 Claude Code 的插件，在状态栏显示 GLM（智谱/ZAI）算力套餐的使用量统计。

![demo](screenshots/demo.png)

## 功能特性

- 📊 **实时使用量追踪**: 显示 Token 和 MCP 使用百分比
- 🎨 **颜色警告提示**: 绿色 (0-79%)、黄色 (80-94%)、红色 (95-100%)
- ⚡ **智能缓存**: 5 分钟缓存减少 API 调用
- 🔍 **自动平台检测**: 经过验证支持 **智谱 CN** 和海外 **z.ai** 双平台
- 🌍 **跨平台支持**: 支持 Windows、macOS 和 Linux

> 验证说明：本项目对 z.ai 平台的支持已通过代码分析、单元测试和公开文档验证。详细验证结论请参考 [z.ai 平台验证报告](.planning/phases/01-verify-and-implement-z-ai-support/01-z-ai-verification.md)。

## 安装

### 通过 npm 安装（推荐）

```bash
npm install -g @jukanntenn/glm-plan-usage
```

如遇网络问题，可使用 npm 镜像加速安装：

```bash
npm install -g @jukanntenn/glm-plan-usage --registry https://registry.npmmirror.com
```

更新：

```bash
npm update -g @jukanntenn/glm-plan-usage
```

<details>
<summary>手动安装（点击展开）</summary>

或者从 [Releases](https://github.com/jukanntenn/glm-plan-usage/releases) 手动下载：

#### Linux

#### 选项 1: 动态链接版本（推荐）
```bash
mkdir -p ~/.claude/glm-plan-usage
wget https://github.com/jukanntenn/glm-plan-usage/releases/latest/download/glm-plan-usage-linux-x64.tar.gz
tar -xzf glm-plan-usage-linux-x64.tar.gz
cp glm-plan-usage ~/.claude/glm-plan-usage/
chmod +x ~/.claude/glm-plan-usage/glm-plan-usage
```
*系统要求: Ubuntu 22.04+, CentOS 9+, Debian 11+, RHEL 9+ (glibc 2.35+)*

#### 选项 2: 静态链接版本（通用兼容）
```bash
mkdir -p ~/.claude/glm-plan-usage
wget https://github.com/jukanntenn/glm-plan-usage/releases/latest/download/glm-plan-usage-linux-x64-musl.tar.gz
tar -xzf glm-plan-usage-linux-x64-musl.tar.gz
cp glm-plan-usage ~/.claude/glm-plan-usage/
chmod +x ~/.claude/glm-plan-usage/glm-plan-usage
```
*适用于任何 Linux 发行版（静态链接，无依赖）*

#### macOS (Intel)

```bash
mkdir -p ~/.claude/glm-plan-usage
wget https://github.com/jukanntenn/glm-plan-usage/releases/latest/download/glm-plan-usage-macos-x64.tar.gz
tar -xzf glm-plan-usage-macos-x64.tar.gz
cp glm-plan-usage ~/.claude/glm-plan-usage/
chmod +x ~/.claude/glm-plan-usage/glm-plan-usage
```

#### macOS (Apple Silicon)

```bash
mkdir -p ~/.claude/glm-plan-usage
wget https://github.com/jukanntenn/glm-plan-usage/releases/latest/download/glm-plan-usage-macos-arm64.tar.gz
tar -xzf glm-plan-usage-macos-arm64.tar.gz
cp glm-plan-usage ~/.claude/glm-plan-usage/
chmod +x ~/.claude/glm-plan-usage/glm-plan-usage
```

#### Windows

```powershell
# 创建目录并下载
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.claude\glm-plan-usage"
Invoke-WebRequest -Uri "https://github.com/jukanntenn/glm-plan-usage/releases/latest/download/glm-plan-usage-windows-x64.zip" -OutFile "glm-plan-usage-windows-x64.zip"
Expand-Archive -Path "glm-plan-usage-windows-x64.zip" -DestinationPath "."
Move-Item "glm-plan-usage.exe" "$env:USERPROFILE\.claude\glm-plan-usage\"
```

</details>

### 从源码构建

```bash
git clone https://github.com/jukanntenn/glm-plan-usage.git
cd glm-plan-usage
cargo build --release
cp target/release/glm-plan-usage ~/.claude/glm-plan-usage/
```

## 配置

在 Claude Code 的 `settings.json` 中添加：

**Linux/macOS:**

```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/glm-plan-usage/glm-plan-usage",
    "padding": 0
  }
}
```

**Windows:**

```json
{
  "statusLine": {
    "type": "command",
    "command": "%USERPROFILE%\\.claude\\glm-plan-usage\\glm-plan-usage.exe",
    "padding": 0
  }
}
```

重启 Claude Code，状态栏将显示：

```text
🪙 32% (⌛️ 1:44) · 🌐 20/100
   │  │           │     └─ MCP 使用量（已用/总计）
   │  │           └─ 分隔符
   │  └─ Token 倒计时（小时:分钟）
   └─ Token 使用百分比

```

如果已在使用 [CCometixLine](https://github.com/Haleclipse/CCometixLine) 或其它类似插件，可创建脚本组合使用：

**Linux/macOS:**

`~/.claude/status-line-combined.sh` 脚本示例：

```bash
#!/bin/bash

# 从 stdin 读取 JSON 输入
INPUT=$(cat)

# 使用相同输入运行两个命令
CCLINE_OUTPUT=$(echo "$INPUT" | ~/.claude/ccline/ccline 2>/dev/null)
GLM_OUTPUT=$(echo "$INPUT" | ~/.claude/glm-plan-usage/glm-plan-usage 2>/dev/null)

# 构建组合输出
OUTPUT=""

# 如果 ccline 有输出，添加到输出
if [ -n "$CCLINE_OUTPUT" ]; then
    OUTPUT="$CCLINE_OUTPUT"
fi

# 如果 glm-plan-usage 有输出，添加到输出
if [ -n "$GLM_OUTPUT" ]; then
    if [ -n "$OUTPUT" ]; then
        OUTPUT="$OUTPUT | $GLM_OUTPUT"
    else
        OUTPUT="$GLM_OUTPUT"
    fi
fi

# 打印组合输出
if [ -n "$OUTPUT" ]; then
    printf "%s" "$OUTPUT"
fi
```

赋予脚本执行权限：`chmod +x ~/.claude/status-line-combined.sh`

在 Claude Code 的 `settings.json` 中配置：

```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/status-line-combined.sh",
    "padding": 0
  }
}
```

**Windows (PowerShell):**

`%USERPROFILE%\.claude\status-line-combined.ps1` 脚本示例：

```powershell
# 从 stdin 读取 JSON 输入
$InputString = [Console]::In.ReadToEnd()

# 使用相同输入运行两个命令
$CclineOutput = $InputString | & "$env:USERPROFILE\.claude\ccline\ccline.exe" 2>$null
$GlmOutput = $InputString | & "$env:USERPROFILE\.claude\glm-plan-usage\glm-plan-usage.exe" 2>$null

# 构建组合输出
$Output = ""

# 如果 ccline 有输出，添加到输出
if (-not [string]::IsNullOrEmpty($CclineOutput)) {
    $Output = $CclineOutput
}

# 如果 glm-plan-usage 有输出，添加到输出
if (-not [string]::IsNullOrEmpty($GlmOutput)) {
    if (-not [string]::IsNullOrEmpty($Output)) {
        $Output = "$Output | $GlmOutput"
    } else {
        $Output = $GlmOutput
    }
}

# 打印组合输出
if (-not [string]::IsNullOrEmpty($Output)) {
    Write-Host -NoNewline $Output
}
```

PowerShell 中赋予脚本执行权限：`Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`

在 Claude Code 的 `settings.json` 中配置：

```json
{
  "statusLine": {
    "type": "command",
    "command": "powershell.exe -File %USERPROFILE%\\.claude\\status-line-combined.ps1",
    "padding": 0
  }
}
```

## 环境变量

**注意**：这些变量通常已在 Claude Code 的 `settings.json` 中配置。如果没有，可以手动设置。插件会自动从 `ANTHROPIC_BASE_URL` 检测平台，不支持的主机会直接报错，不会自动猜测。

### 智谱 CN (ZHIPU)

**Linux/macOS:**

```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

**Windows (Command Prompt):**

```cmd
set ANTHROPIC_AUTH_TOKEN=your-token-here
set ANTHROPIC_BASE_URL=https://open.bigmodel.cn/api/anthropic
```

**Windows (PowerShell):**

```powershell
$env:ANTHROPIC_AUTH_TOKEN="your-token-here"
$env:ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

### 海外 z.ai (ZAI)

根据官方文档，海外用户请使用 `https://api.z.ai/api/anthropic` 作为基础 URL：

**Linux/macOS:**

```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://api.z.ai/api/anthropic"
```

**Windows (Command Prompt):**

```cmd
set ANTHROPIC_AUTH_TOKEN=your-token-here
set ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic
```

**Windows (PowerShell):**

```powershell
$env:ANTHROPIC_AUTH_TOKEN="your-token-here"
$env:ANTHROPIC_BASE_URL="https://api.z.ai/api/anthropic"
```

验证说明：`Platform::detect_from_url` 会自动检测：
- 包含 `api.z.ai` → `Platform::Zai`
- 包含 `bigmodel.cn` 或 `zhipu` → `Platform::Zhipu`

## 常见问题排查

### 平台检测失败

如果遇到 "Platform detection failed" 错误，请检查 `ANTHROPIC_BASE_URL` 配置：

```bash
# Linux/macOS: 检查当前配置
echo $ANTHROPIC_BASE_URL

# 智谱 CN 用户应使用：
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"

# 海外 z.ai 用户应使用：
export ANTHROPIC_BASE_URL="https://api.z.ai/api/anthropic"
```

插件不会对未知主机自动猜测，必须匹配上述两种模式之一。

### API 请求失败

验证配置后可以手动测试 API 连通性（**注意：不要在输出中暴露真实 Token**）：

```bash
# Linux/macOS: 使用 curl 测试（不打印敏感信息）
curl -H "Authorization: Bearer $ANTHROPIC_AUTH_TOKEN" \
  "$ANTHROPIC_BASE_URL/monitor/usage/quota/limit" \
  -o /dev/null -w "%{http_code}\n"
```

应该返回 `200` 状态码。如果返回 401 或 403，请检查 Token 是否正确。

### 额外说明

本项目对 z.ai 平台的支持基于公开文档和代码测试验证。由于没有可用的 live token，无法进行端到端集成验证。[验证报告](.planning/phases/01-verify-and-implement-z-ai-support/01-z-ai-verification.md)中记录了期望的响应格式和更多技术细节。

## 许可证

MIT
