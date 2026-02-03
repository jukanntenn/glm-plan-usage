#!/usr/bin/env node

const { spawnSync } = require('child_process');

console.log('Testing glm-plan-usage...\n');

// Test 1: Help command
console.log('Test 1: Help command');
const helpResult = spawnSync('node', ['bin/glm-plan-usage.js', '--help'], {
  stdio: 'inherit',
  cwd: __dirname
});

if (helpResult.status === 0) {
  console.log('✓ Help command works\n');
} else {
  console.error('✗ Help command failed\n');
  process.exit(1);
}

// Test 2: Version flag
console.log('Test 2: Run with sample input');
const input = JSON.stringify({
  model: { id: 'test', display_name: 'Test' },
  workspace: { current_dir: '/home/test' },
  transcript_path: '/tmp/test.json'
});

const runResult = spawnSync('node', ['bin/glm-plan-usage.js'], {
  input: input,
  stdio: ['pipe', 'inherit', 'pipe'],
  cwd: __dirname
});

if (runResult.status === 0) {
  console.log('✓ Plugin executes successfully\n');
} else {
  console.error('✗ Plugin execution failed\n');
  process.exit(1);
}

console.log('All tests passed! ✓');
