<!--
Purpose: Index the AGENTS.md doctrine references so contributors load only the section they need.
Caller: AGENTS.md and contributors deciding which reference to open.
Dependencies: The other files in AGENTS/references/.
Main Functions: Map AGENTS.md headings to reference files and capture loading discipline.
Side Effects: None — this file is informational.
-->
# AGENTS Knowledge Map

`AGENTS.md` is the entry point. The detailed doctrine lives here so the entry point stays small and the rules stay searchable.

## Section-to-File Map

| Topic | Reference file |
|---|---|
| Native command routing, hook rewrite, token compaction | [10-native-command-routing.md](10-native-command-routing.md) |
| Skill routing, specialist skills, skill-focused execution, agent profiles | [20-skill-routing.md](20-skill-routing.md) |
| Execution strategy, iterative development loop, flow control, loop limits, general approach | [30-execution-strategy.md](30-execution-strategy.md) |
| Code quality standards, testing requirements, feature flags | [40-code-quality-and-testing.md](40-code-quality-and-testing.md) |
| Feature delivery rules, best practices, prohibited shortcuts | [50-delivery-and-prohibited-shortcuts.md](50-delivery-and-prohibited-shortcuts.md) |
| Windows environment, cross-platform script portability | [60-environment-and-portability.md](60-environment-and-portability.md) |
| Code review requirements, automated quality checks, quality gates, final output, reasoning effort, model policy, git identity | [70-review-quality-gates-and-policies.md](70-review-quality-gates-and-policies.md) |
| Source anchors, related skills, tooling anchors | [99-source-anchors.md](99-source-anchors.md) |

## Loading Discipline

- Open `AGENTS.md` first to confirm scope. It is the index and the canonical pointer surface.
- Open one reference file at a time, scoped to the section you actually need. Do not load every reference up front.
- When a reference and `AGENTS.md` disagree, `AGENTS.md` wins. Open a follow-up to reconcile the reference.
- When a reference and a specialist `SKILL.md` disagree on the specialist's own surface, the specialist `SKILL.md` wins for that surface.
