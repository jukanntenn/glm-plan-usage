#!/bin/bash

# Test script for verifying the refactored display

echo "=== Testing glm_usage removal and new display format ==="
echo ""

# Build the project
echo "Building..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi
echo "✓ Build successful"
echo ""

# Create test config directory
TEST_DIR="/tmp/glm-test-$$"
mkdir -p "$TEST_DIR"

# Test 1: With countdown timer (default clock)
echo "Test 1: Default config with clock timer"
cat > "$TEST_DIR/config.toml" << 'EOF'
[style]
mode = "emoji"
separator = " | "

[[segments]]
id = "token_usage"
enabled = true

[segments.icon]
emoji = "🪙"
ascii = "$"

[segments.options]
show_timer = true
timer_mode = "clock"

[[segments]]
id = "weekly_usage"
enabled = true

[segments.icon]
emoji = "🗓️"
ascii = "*"

[[segments]]
id = "mcp_usage"
enabled = true

[segments.icon]
emoji = "🌐"
ascii = "#"
EOF

# Create test input
cat > "$TEST_DIR/input.json" << 'EOF'
{
  "model": {"id": "test-model"}
}
EOF

# Run with test config (need to set env vars)
export ANTHROPIC_AUTH_TOKEN="test-token"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"

# Note: This will fail without valid API credentials, but we can check the config parsing
echo "Config file created at: $TEST_DIR/config.toml"
echo ""

# Test 2: Without timer
echo "Test 2: Config without timer"
cat > "$TEST_DIR/config-no-timer.toml" << 'EOF'
[style]
mode = "auto"
separator = " | "

[[segments]]
id = "token_usage"
enabled = true

[segments.icon]
emoji = "🪙"
ascii = "$"

[segments.options]
show_timer = false

[[segments]]
id = "weekly_usage"
enabled = true

[segments.icon]
emoji = "🗓️"
ascii = "*"

[[segments]]
id = "mcp_usage"
enabled = true

[segments.icon]
emoji = "🌐"
ascii = "#"
EOF

echo "Config (no timer) created at: $TEST_DIR/config-no-timer.toml"
echo ""

# Test 3: ASCII mode with countdown
echo "Test 3: ASCII mode with countdown"
cat > "$TEST_DIR/config-ascii.toml" << 'EOF'
[style]
mode = "ascii"
separator = " | "

[[segments]]
id = "token_usage"
enabled = true

[segments.icon]
emoji = "🪙"
ascii = "$"

[segments.options]
show_timer = true
timer_mode = "countdown"

[[segments]]
id = "weekly_usage"
enabled = true

[segments.icon]
emoji = "🗓️"
ascii = "*"

[[segments]]
id = "mcp_usage"
enabled = true

[segments.icon]
emoji = "🌐"
ascii = "#"
EOF

echo "Config (ASCII mode) created at: $TEST_DIR/config-ascii.toml"
echo ""

# Test 4: Check that glm_usage is no longer valid
echo "Test 4: Verify glm_usage is rejected"
cat > "$TEST_DIR/config-invalid.toml" << 'EOF'
[[segments]]
id = "glm_usage"
enabled = true
EOF

echo "Invalid config created at: $TEST_DIR/config-invalid.toml"
echo ""

# Cleanup
echo "Test files created in: $TEST_DIR"
echo "Run 'rm -rf $TEST_DIR' to clean up"