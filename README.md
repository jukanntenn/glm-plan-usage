# glm-plan-usage

简体中文 | [English](README_en.md)

一个用于 Claude Code 的插件，在状态栏显示 GLM（智谱/ZAI）算力套餐的使用量统计。

> ⚠️ **征集测试**: 如果您有**周限额**的 API Key，欢迎联系我们进行测试，帮助完善周限额显示功能。

![demo](screenshots/demo.png)

## 功能特性

- 📊 **实时使用量追踪**: 显示 Token 和 MCP 使用百分比
- 🗓️ **周限额支持**: 显示每周 Token 使用量（仅限新版套餐）
- 🎨 **颜色警告提示**: 绿色 (0-50%)、黄色 (51-80%)、红色 (81-100%)
- ⚡ **智能缓存**: 5 分钟缓存减少 API 调用
- 🔍 **自动平台检测**: 支持 ZAI 和智谱平台
- 🌍 **跨平台支持**: 支持 Windows、macOS 和 Linux

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

## Claude Code 配置

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
    "command": "$HOME/.claude/glm-plan-usage/glm-plan-usage.exe",
    "padding": 0
  }
}
```

> **注意:** 老版本的 Claude Code 可能需要使用 Windows 风格路径，如 `%USERPROFILE%\.claude\glm-plan-usage\glm-plan-usage.exe`。

重启 Claude Code，状态栏将显示：

```text
🪙 32% · ⏱ 14:30 | 🗓️ 24% | 🌐 20/100
   │  │    │        │       │     └─ MCP 使用量（已用/总计）
   │  │    │        │       └─ Segment 分隔符
   │  │    │        └─ 周限额使用百分比（新版套餐）
   │  │    └─ Token 重置时间（时钟模式）
   │  └─ 内部分隔符
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

`$HOME/.claude/status-line-combined.ps1` 脚本示例：

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
    "command": "powershell.exe -File $HOME/.claude/status-line-combined.ps1",
    "padding": 0
  }
}
```

> **注意:** 老版本的 Claude Code 可能需要使用 Windows 风格路径。

## 环境变量

**注意**：这些变量通常已在 Claude Code 的 `settings.json` 中配置。如果没有，可以手动设置：

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

## 使用

### 配置管理

插件提供以下命令管理配置：

```bash
glm-plan-usage init       # 初始化配置文件 (~/.claude/glm-plan-usage/config.toml)
glm-plan-usage print      # 打印当前配置内容
glm-plan-usage check      # 验证配置文件是否有效
```

全局选项：
- `--verbose`: 显示详细输出（调试用）
- `--no-cache`: 禁用缓存（本次运行）

### 自定义显示

配置文件支持自定义 Segment 显示：

- **显示模式**: 可选择 `auto`（自动检测）、`emoji` 或 `ascii` 三种模式
- **自定义图标**: 可为 `emoji` 和 `ascii` 模式分别设置图标
- **自定义分隔符**: 修改 `style.separator` 改变 Segment 间的分隔符
- **定时器模式**: 设置 `timer_mode` 为 `clock`（时钟）或 `countdown`（倒计时）
- **启用/禁用**: 通过 `enabled` 字段控制各 Segment 的显示

配置文件位于 `~/.claude/glm-plan-usage/config.toml`，运行 `glm-plan-usage init` 生成默认配置，内含详细注释说明。

## 许可证

MIT
