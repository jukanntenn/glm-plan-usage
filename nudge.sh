#!/usr/bin/env bash
set -euo pipefail

prompt="${1:-/nudging}"

if [[ "$prompt" != *"/nudging"* ]]; then
    prompt="/nudging $prompt"
fi

output=$(claude --dangerously-skip-permissions -p "$prompt")

json=$(echo "$output" | awk '
    BEGIN { in_json=0; buf="" }
    /```json/ { in_json=1; next }
    /```/ && in_json { in_json=0; print buf; exit }
    in_json { buf = buf $0 "\n" }
    !in_json && /^\s*\{/ { buf=$0; in_json=2; next }
    !in_json && /^\s*\[/ { buf=$0; in_json=2; next }
    in_json==2 {
        buf = buf $0 "\n"
        if (/^\s*\}\s*$/ || /^\s*\]\s*$/) { print buf; exit }
    }
')

if [[ -z "$json" ]]; then
    echo "$output" | sed -n '/```json/,/```/p' | sed '1d;$d' > /dev/null 2>&1 || true
    json=$(echo "$output" | sed -n '/```json/,/```/p' | sed '1d;$d')
fi

if [[ -z "$json" ]]; then
    json=$(echo "$output" | grep -oP '(?s)\{.*"task_id".*\}' | head -n 1 || true)
fi

if [[ -z "$json" ]]; then
    echo "Error: failed to extract JSON from claude output" >&2
    echo "--- raw output ---" >&2
    echo "$output" >&2
    exit 1
fi

task_id=$(echo "$json" | python3 -c '
import sys, json, re
raw = sys.stdin.read()

# Try direct JSON parse
try:
    data = json.loads(raw)
    if isinstance(data, dict) and "task_id" in data:
        print(data["task_id"])
        sys.exit(0)
except Exception:
    pass

# Try to find JSON object with task_id anywhere in the text
matches = re.findall(r"\{[^{}]*\"task_id\"[^{}]*\}", raw)
for m in matches:
    try:
        data = json.loads(m)
        if "task_id" in data:
            print(data["task_id"])
            sys.exit(0)
    except Exception:
        pass

# Try nested braces with balanced matching
def find_json_objects(text):
    results = []
    i = 0
    while i < len(text):
        if text[i] == "{":
            depth = 1
            j = i + 1
            while j < len(text) and depth > 0:
                if text[j] == "{":
                    depth += 1
                elif text[j] == "}":
                    depth -= 1
                j += 1
            if depth == 0:
                results.append(text[i:j])
            i = j
        else:
            i += 1
    return results

for obj in find_json_objects(raw):
    try:
        data = json.loads(obj)
        if isinstance(data, dict) and "task_id" in data:
            print(data["task_id"])
            sys.exit(0)
    except Exception:
        pass

sys.exit(1)
' 2>/dev/null) || {
    echo "Error: failed to parse task_id from JSON" >&2
    echo "--- extracted JSON ---" >&2
    echo "$json" >&2
    exit 1
}

if [[ -z "$task_id" ]]; then
    echo "Error: task_id not found in JSON" >&2
    echo "--- extracted JSON ---" >&2
    echo "$json" >&2
    exit 1
fi

fa gestate "$task_id" --max-rounds 1 --run-rounds 1
