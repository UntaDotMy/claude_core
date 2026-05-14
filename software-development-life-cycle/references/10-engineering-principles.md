# Engineering Principles Playbook

## Core Engineering Foundations

Apply these principles in design, implementation, and review:

- Modularity: split systems into cohesive, independent, reusable modules.
- Abstraction: expose stable interfaces and hide implementation details.
- Encapsulation: protect invariants and data through controlled access.
- DRY: remove duplicated logic and duplicated knowledge.
- KISS: prefer simpler designs with lower cognitive load.
- YAGNI: avoid speculative complexity and unused features.
- SOLID: design classes/modules for maintainability, extension, and testability.

## Clean Code and Maintainability Standards

- Use descriptive naming and clear boundaries.
- Keep functions focused and short.
- Handle failures explicitly.
- Maintain consistency with project conventions.
- Prefer composition over deep inheritance.

## Reuse-First Policy

1. Search for existing implementation before creating new code.
2. Reuse shared components/utilities where behavior matches.
3. Extract shared behavior when duplication appears across modules.
4. Record reuse decisions in ADRs or PR notes for traceability.

## Clarification-First Policy

For high-impact ambiguity, ask before implementation:

- Unclear acceptance criteria
- Conflicting requirements
- Underspecified constraints (security, compliance, performance)
- Ambiguous ownership or lifecycle responsibilities

## TDD and Testability Guidance

Use test-driven or test-first workflows where practical:

1. Define expected behavior (acceptance criteria).
2. Write failing unit test for smallest behavior increment.
3. Implement minimal code to pass.
4. Refactor while preserving test green state.
5. Extend with integration/system tests for interfaces and workflows.

## Code Quality Standards

### Readability
- Use full, descriptive names for variables and functions (no shortforms or abbreviations)
- Use verb + noun pattern: `getUserData`, `calculateTotal`, `validateEmail`
- Be specific: `fetchUserProfile` not `getData`
- Comments only for non-obvious logic or business rules

### Scope Discipline
- Only implement what was requested
- Delete unused code completely
- No backward compatibility unless explicitly requested
- No renaming unused variables with underscore
- No re-exporting old names
- No adding "// removed" or "// deprecated" comments

### DRY, Simplicity, and Architecture
- Reuse existing functions/components; extract common logic into shared utilities
- Solve the stated problem, nothing more; avoid premature optimization
- Prefer standard library over external dependencies
- Follow existing project structure; maintain clear module boundaries
- Keep coupling low, cohesion high; use appropriate design patterns without forcing them

## Architecture Patterns

### SOLID Principles
| Principle | Meaning |
|---|---|
| Single Responsibility | One reason to change |
| Open/Closed | Open for extension, closed for modification |
| Liskov Substitution | Subtypes must be substitutable |
| Interface Segregation | Many specific interfaces > one general |
| Dependency Inversion | Depend on abstractions, not concretions |

### Modularity and Abstraction
- Clear separation of concerns
- Each module has single, well-defined purpose
- Minimize dependencies between modules
- Hide implementation details; expose clean interfaces
- Make it easy to change implementations

## Anti-Patterns to Avoid

| Anti-Pattern | Problem |
|---|---|
| Over-engineering | Complexity not required by current needs |
| Premature optimization | Optimizing before measuring |
| God objects | Classes/modules that do too much |
| Tight coupling | Hard to change one thing without breaking others |
| Magic numbers | Unexplained constants in code |
| Copy-paste | Duplicating code instead of extracting shared logic |
| Ignoring errors | Swallowing exceptions without handling |
| Hardcoding | Config values embedded in code |

## Software Crisis Context

Use the historical software crisis lens to justify disciplined engineering:

- Frequent schedule overruns
- Cost blowouts
- Defect-heavy releases
- Low maintainability of growing codebases

Treat process rigor, measurement, and architecture hygiene as risk controls, not bureaucracy.
