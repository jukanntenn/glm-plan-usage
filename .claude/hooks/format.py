#!/usr/bin/env python3
"""
PostToolUse hook for formatting Rust files after editing.
This hook runs cargo fmt after Edit or Write operations on .rs files.
"""

import json
import subprocess
import sys


def main():
    """Main function to run cargo fmt hook."""
    try:
        hook_data = json.load(sys.stdin)
        tool_input = hook_data.get("tool_input", {})

        file_path = tool_input.get("file_path", "")

        if not file_path or not file_path.endswith(".rs"):
            sys.exit(0)

        try:
            result = subprocess.run(
                ["cargo", "fmt"],
                capture_output=True,
                text=True,
            )
        except FileNotFoundError:
            print("cargo not found. Please install Rust toolchain.", file=sys.stderr)
            sys.exit(0)

        if result.stdout:
            print(result.stdout, end="")
        if result.stderr:
            print(result.stderr, file=sys.stderr, end="")

        sys.exit(0)

    except Exception as e:
        print(f"Hook error: {e}", file=sys.stderr)
        sys.exit(0)


if __name__ == "__main__":
    main()
