---
name: nudging
description: Make one small, incremental code improvement to reduce codebase entropy. Use this skill whenever the user wants to nudge the codebase toward better shape, asks for a tiny refactor, says "nudge", or wants a low-risk cleanup pass — even if they don't describe a specific change.
---

## What this does

Find a single, small improvement in the codebase and plan it. Think of it as paying down technical debt one coin at a time — each nudge makes the code slightly cleaner without any risk to behavior.

## Workflow

1. Scan the codebase for a concrete improvement opportunity. Pick one.
2. Create a task: `fa task create <slug>`
3. Write `spec.md` in the task directory describing the change.
4. Write `plan.md` in the same directory. The plan must be detailed enough that someone unfamiliar with the codebase could execute it without making a single design decision.

Output JSON when done:

```json
{
  "task_id": <task-id>,
  "task_path": "<absolute-path-to-task-directory>"
}
```

## What counts as a nudge

- Extract a code fragment into a pure function, add type hints, write unit tests
- Fix naming inconsistencies that hurt readability
- Simplify equivalent code — remove redundancy or unnecessary complexity
- Any small, self-contained improvement that leaves the codebase cleaner

This list is not exhaustive — use your judgment.

## Rules

- Each function should do one thing. If you spot one that does two, that is a valid nudge.
- Never change business logic. The behavior before and after must be identical.
- Keep the scope small. One nudge per invocation — resist the urge to chain improvements.
