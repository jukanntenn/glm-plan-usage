#!/bin/bash
# Installation script for GLM Usage Status Plugin

set -e

echo "=== GLM Usage Status Plugin Installation ==="
echo

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "Step 1: Building release binary..."
cargo build --release

echo "Step 2: Installing to ~/.claude/glm-plan-usage/"
mkdir -p ~/.claude/glm-plan-usage
cp target/release/glm-plan-usage ~/.claude/glm-plan-usage/
chmod +x ~/.claude/glm-plan-usage/glm-plan-usage

echo "Step 3: Creating default configuration..."
~/.claude/glm-plan-usage/glm-plan-usage --init

echo
echo "=== Installation Complete ==="
echo
echo "Binary installed at: ~/.claude/glm-plan-usage/glm-plan-usage"
echo "Config created at: ~/.claude/glm-plan-usage/config.toml"
echo
echo "Next steps:"
echo
echo "1. Add environment variables to your ~/.bashrc or ~/.zshrc:"
echo "   export ANTHROPIC_AUTH_TOKEN=\"your-token-here\""
echo "   export ANTHROPIC_BASE_URL=\"https://open.bigmodel.cn/api/anthropic\""
echo
echo "2. Configure Claude Code by editing ~/.config/claude-code/settings.json:"
echo
echo '   {'
echo '     "statusLine": {'
echo '       "type": "command",'
echo '       "command": "~/.claude/glm-plan-usage/glm-plan-usage",'
echo '       "padding": 0'
echo '     }'
echo '   }'
echo
echo "3. Restart Claude Code"
echo
echo "For testing, run:"
echo "  export ANTHROPIC_AUTH_TOKEN=\"your-token\""
echo "  export ANTHROPIC_BASE_URL=\"https://open.bigmodel.cn/api/anthropic\""
echo "  echo '{\"model\":{\"id\":\"claude-sonnet-4\"}}' | ~/.claude/glm-plan-usage/glm-plan-usage"
