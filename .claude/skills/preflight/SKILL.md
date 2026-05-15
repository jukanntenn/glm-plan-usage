---
name: preflight
description: >
  Generate an end-to-end acceptance checklist by diffing the current codebase against
  the last released version. Compiles every user-facing and behavioral change into a
  structured, step-by-step verification manual. Use this skill whenever the user
  mentions "preflight", "acceptance testing", "acceptance manual", "release checklist",
  "verification checklist", "what changed since last release", or wants to validate
  readiness before shipping a new version. Also trigger when the user asks to review
  changes between versions or prepare QA steps.
---

Generate an end-to-end acceptance manual by diffing the current working tree against the last released version.

## Process

1. **Identify versions** — find the latest tag (`git describe --tags --abbrev=0`) and confirm the target scope (default: diff from that tag to HEAD).
2. **Gather the full diff** — `git diff <tag>..HEAD` plus `git log --oneline <tag>..HEAD`. Read changed source files to understand intent behind each change — not just what lines moved, but what behavior changed for the user.
3. **Classify every change** — for each diff, determine: new features, modified behavior, fixed bugs, removed functionality, config changes, and breaking changes.
4. **Produce the acceptance manual** — output a structured checklist organized by functional area (not by file or commit) to a file using output format bellow. For each item include:
   - What changed (one-line summary)
   - How to verify it (concrete steps: which command to run, what output to expect, which config field to check)
   - Edge cases or risks to watch for

## Output format

```markdown
# Preflight Acceptance Manual — <tag> → HEAD

## Summary

<one-paragraph overview of all changes>

## 1. <Functional Area>

- [ ] **<change summary>** — <verification steps>
- [ ] **<change summary>** — <verification steps>

## 2. <Functional Area>

...
```

Keep the manual concise, actionable, and free of implementation details. Every checklist item must be verifiable by running a command or observing a behavior — no vague items like "verify it works."
