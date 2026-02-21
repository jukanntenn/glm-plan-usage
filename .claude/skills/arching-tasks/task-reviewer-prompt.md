# Task Document Reviewer Prompt Template

Use this template when dispatching a task document reviewer subagent.

**Purpose:** Verify the task.md is complete, clear, and ready for implementation.

**Dispatch after:** task.md is written to `.fa/tasks/{id}-{mm-dd}-{slug}/task.md`

---

## Prompt Template

```
Task tool (general-purpose):
  description: "Review task.md"
  prompt: |
    You are a task document reviewer. Verify this task.md is complete and ready for implementation.

    **Task file to review:** [TASK_FILE_PATH]
    **Is parent task:** [true/false]

    ## What to Check

    ### All Tasks

    | Category | What to Look For |
    |----------|------------------|
    | Completeness | TODOs, placeholders, "TBD", incomplete sections, missing headers |
    | Clarity | Requirements ambiguous enough to cause someone to build the wrong thing |
    | Implementability | Could an engineer follow the implementation steps without getting stuck? |
    | File paths | All file paths are explicit and correct (no vague references like "update the config") |
    | Steps | Each step is actionable (not "add validation" but "add X validation to Y function") |

    ### Parent Tasks (is_parent=true)

    | Category | What to Look For |
    |----------|------------------|
    | Decomposition quality | Each subtask has clear boundaries and could be worked on independently |
    | Dependency order | Subtasks are sequenced logically, dependencies are explicit |
    | Scope coverage | The decomposition covers all aspects of the parent goal |
    | Subtask descriptions | Each subtask's purpose is clear from the parent context |

    ### Subtasks (is_parent=false, has parent)

    | Category | What to Look For |
    |----------|------------------|
    | Parent alignment | Design and implementation align with parent's overall architecture |
    | Scope | Subtask stays within its boundaries, doesn't overlap with siblings |
    | Context | References parent where appropriate, doesn't repeat parent-level context unnecessarily |

    ## Calibration

    **Only flag issues that would cause real problems during implementation.**

    Issues worth flagging:
    - Missing or ambiguous requirements that could lead to wrong implementation
    - Implementation steps that are too vague to act on
    - Placeholder content that needs to be filled in
    - Missing file paths or incorrect paths
    - For parent tasks: subtasks that are too coupled or missing coverage
    - For subtasks: design that contradicts parent architecture

    Not worth flagging:
    - Minor wording improvements
    - Stylistic preferences
    - "Could be more detailed" without specific gaps

    Approve unless there are serious gaps that would lead to failed implementation.

    ## Output Format

    ## Task Review

    **Status:** Approved | Issues Found

    **Issues (if any):**
    - [Section/Location]: [specific issue] - [why it matters for implementation]

    **Recommendations (advisory, do not block approval):**
    - [suggestions for improvement that don't require changes]
```

---

## Example Usage

### For a Simple Task

```
Task tool (general-purpose):
  description: "Review task.md"
  prompt: |
    You are a task document reviewer. Verify this task.md is complete and ready for implementation.

    **Task file to review:** .fa/tasks/5-03-25-add-rate-limiting/task.md
    **Is parent task:** false

    ## What to Check

    ### All Tasks

    | Category | What to Look For |
    |----------|------------------|
    | Completeness | TODOs, placeholders, "TBD", incomplete sections, missing headers |
    | Clarity | Requirements ambiguous enough to cause someone to build the wrong thing |
    | Implementability | Could an engineer follow the implementation steps without getting stuck? |
    | File paths | All file paths are explicit and correct (no vague references like "update the config") |
    | Steps | Each step is actionable (not "add validation" but "add X validation to Y function") |

    ## Calibration

    **Only flag issues that would cause real problems during implementation.**

    Issues worth flagging:
    - Missing or ambiguous requirements that could lead to wrong implementation
    - Implementation steps that are too vague to act on
    - Placeholder content that needs to be filled in
    - Missing file paths or incorrect paths

    Not worth flagging:
    - Minor wording improvements
    - Stylistic preferences

    Approve unless there are serious gaps that would lead to failed implementation.

    ## Output Format

    ## Task Review

    **Status:** Approved | Issues Found

    **Issues (if any):**
    - [Section/Location]: [specific issue] - [why it matters for implementation]

    **Recommendations (advisory, do not block approval):**
    - [suggestions for improvement that don't require changes]
```

### For a Parent Task

```
Task tool (general-purpose):
  description: "Review task.md"
  prompt: |
    You are a task document reviewer. Verify this task.md is complete and ready for implementation.

    **Task file to review:** .fa/tasks/2-03-25-user-auth-system/task.md
    **Is parent task:** true

    ## What to Check

    ### All Tasks

    | Category | What to Look For |
    |----------|------------------|
    | Completeness | TODOs, placeholders, "TBD", incomplete sections, missing headers |
    | Clarity | Requirements ambiguous enough to cause someone to build the wrong thing |
    | Implementability | Could an engineer follow the implementation steps without getting stuck? |
    | File paths | All file paths are explicit and correct |
    | Steps | Each step is actionable |

    ### Parent Tasks

    | Category | What to Look For |
    |----------|------------------|
    | Decomposition quality | Each subtask has clear boundaries and could be worked on independently |
    | Dependency order | Subtasks are sequenced logically, dependencies are explicit |
    | Scope coverage | The decomposition covers all aspects of the parent goal |
    | Subtask descriptions | Each subtask's purpose is clear from the parent context |

    ## Calibration

    **Only flag issues that would cause real problems during implementation.**

    Issues worth flagging:
    - Subtasks that are too coupled (need to be worked on simultaneously)
    - Missing coverage of parent goal
    - Illogical dependency order
    - Vague subtask purposes

    Approve unless there are serious gaps.

    ## Output Format

    ## Task Review

    **Status:** Approved | Issues Found

    **Issues (if any):**
    - [Section/Location]: [specific issue] - [why it matters for implementation]

    **Recommendations (advisory, do not block approval):**
    - [suggestions for improvement]
```

### For a Subtask

```
Task tool (general-purpose):
  description: "Review task.md"
  prompt: |
    You are a task document reviewer. Verify this task.md is complete and ready for implementation.

    **Task file to review:** .fa/tasks/2-03-25-user-auth-system/3-03-25-setup-db/task.md
    **Is parent task:** false
    **Parent task file:** .fa/tasks/2-03-25-user-auth-system/task.md

    ## What to Check

    ### All Tasks

    | Category | What to Look For |
    |----------|------------------|
    | Completeness | TODOs, placeholders, "TBD", incomplete sections, missing headers |
    | Clarity | Requirements ambiguous enough to cause someone to build the wrong thing |
    | Implementability | Could an engineer follow the implementation steps without getting stuck? |
    | File paths | All file paths are explicit and correct |
    | Steps | Each step is actionable |

    ### Subtasks

    | Category | What to Look For |
    |----------|------------------|
    | Parent alignment | Design and implementation align with parent's overall architecture |
    | Scope | Subtask stays within its boundaries, doesn't overlap with siblings |
    | Context | References parent where appropriate, doesn't repeat parent-level context unnecessarily |

    ## Calibration

    **Only flag issues that would cause real problems during implementation.**

    Issues worth flagging:
    - Design that contradicts parent architecture
    - Scope creep beyond subtask boundaries
    - Missing alignment with sibling subtasks
    - Implementation that won't integrate with parent

    Approve unless there are serious gaps.

    ## Output Format

    ## Task Review

    **Status:** Approved | Issues Found

    **Issues (if any):**
    - [Section/Location]: [specific issue] - [why it matters for implementation]

    **Recommendations (advisory, do not block approval):**
    - [suggestions for improvement]
```

---

## Reviewer Returns

| Field | Description |
|-------|-------------|
| **Status** | `Approved` or `Issues Found` |
| **Issues** | List of specific issues with locations and impact (only if Issues Found) |
| **Recommendations** | Advisory suggestions that don't require changes (optional) |
