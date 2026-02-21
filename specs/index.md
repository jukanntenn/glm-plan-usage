# Specifications

Design specifications for the `glm-plan-usage` plugin. These documents capture golden principles, patterns, high-level design, and domain knowledge — not implementation details. For argument flags, code-level APIs, and full templates, read the source code.

## Specification Files

| File                                             | Domain              | When to Refer                                                             |
| ------------------------------------------------ | ------------------- | ------------------------------------------------------------------------- |
| [directory-structure.md](directory-structure.md) | Module organization | When creating or modifying source paths, adding new modules or segments   |
| [configuration.md](configuration.md)             | Config system       | When changing config schema, segment options, or style modes              |
| [api.md](api.md)                                 | API integration     | When changing API endpoints, response parsing, or adding new data sources |
| [error-handling.md](error-handling.md)           | Error handling      | When adding new error paths or changing error handling behavior           |
| [logging.md](logging.md)                         | Output conventions  | When adding output channels or changing verbose mode behavior             |
| [quality.md](quality.md)                         | Code quality        | When writing new code, reviewing changes, or adding tests                 |

## Principles

- Specifications describe **WHAT** and **WHY**, not **HOW**
- Each file must remain under 200 lines
- When the mechanism changes, update the specification to reflect the new design
- For implementation details, read the source code under `src/`
