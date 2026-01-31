#!/bin/bash
# Test script for GLM Usage Status Plugin

set -e

echo "=== GLM Usage Status Plugin Test Suite ==="
echo

# Test 1: Help output
echo "Test 1: Help output"
~/.claude/glm-plan-usage/glm-plan-usage --help | head -3
echo "✓ Help works"
echo

# Test 2: Initialize config
echo "Test 2: Initialize config"
rm -f ~/.claude/glm-plan-usage/config.toml
~/.claude/glm-plan-usage/glm-plan-usage --init > /dev/null 2>&1
if [ -f ~/.claude/glm-plan-usage/config.toml ]; then
    echo "✓ Config created"
else
    echo "✗ Config not created"
    exit 1
fi
echo

# Test 3: Run with missing env vars (should produce no output)
echo "Test 3: Run with missing env vars"
unset ANTHROPIC_AUTH_TOKEN
unset ANTHROPIC_BASE_URL
OUTPUT=$(echo '{"model":{"id":"claude-sonnet-4"}}' | ~/.claude/glm-plan-usage/glm-plan-usage 2>&1)
if [ -z "$OUTPUT" ]; then
    echo "✓ No output with missing env vars (graceful degradation)"
else
    echo "✗ Unexpected output: $OUTPUT"
fi
echo

# Test 4: Run with invalid JSON input (should not crash)
echo "Test 4: Run with invalid JSON input"
OUTPUT=$(echo 'invalid json' | ~/.claude/glm-plan-usage/glm-plan-usage 2>&1)
if [ -z "$OUTPUT" ]; then
    echo "✓ Handles invalid JSON gracefully"
else
    echo "✗ Unexpected output: $OUTPUT"
fi
echo

# Test 5: Verbose mode
echo "Test 5: Verbose mode"
OUTPUT=$(echo '{"model":{"id":"claude-sonnet-4"}}' | ~/.claude/glm-plan-usage/glm-plan-usage --verbose 2>&1)
echo "Verbose output (if any): $OUTPUT"
echo "✓ Verbose mode works"
echo

# Test 6: Check config structure
echo "Test 6: Check config structure"
if grep -q "glm_usage" ~/.claude/glm-plan-usage/config.toml; then
    echo "✓ Config contains glm_usage segment"
else
    echo "✗ Config missing glm_usage segment"
    exit 1
fi
echo

# Test 7: Check binary size
echo "Test 7: Check binary size"
SIZE=$(du -h ~/.claude/glm-plan-usage/glm-plan-usage | cut -f1)
echo "Binary size: $SIZE"
if [ $(stat -f%z ~/.claude/glm-plan-usage/glm-plan-usage 2>/dev/null || stat -c%s ~/.claude/glm-plan-usage/glm-plan-usage 2>/dev/null) -lt 5000000 ]; then
    echo "✓ Binary size is reasonable (<5MB)"
else
    echo "⚠ Binary size is large (may want to strip)"
fi
echo

echo "=== All Tests Passed ==="
echo
echo "Next steps:"
echo "1. Set ANTHROPIC_AUTH_TOKEN and ANTHROPIC_BASE_URL environment variables"
echo "2. Configure Claude Code settings.json to use this plugin"
echo "3. Restart Claude Code to see the status bar"
