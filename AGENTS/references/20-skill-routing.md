<!--
Purpose: Capture skill routing rules, the specialist roster, skill-focused execution, and agent profiles previously inline in AGENTS.md.
Caller: AGENTS.md when picking a primary skill, deciding whether to compose, or wiring agent-profile TOMLs.
Dependencies: The 13 specialist SKILL.md files and the matching .claude/agents/<name>.md subagent files.
Main Functions: Define routing defaults, the specialist matrix, composition discipline, and agent-profile expectations.
Side Effects: None — this file is informational.
-->
# Skill Routing, Skill-Focused Execution, and Agent Profiles

## Skill Routing

### Default Behavior

When no skill is explicitly mentioned:
1. Route directly to the primary domain skill when the task clearly belongs to one surface
2. If a non-trivial task clearly belongs to one specialist surface, load that skill before absorbing the work into generic execution
3. Use `software-development-life-cycle` when the work is mainly sequencing, cross-domain planning, or architecture framing
4. Start with `reviewer` only for audits, production-readiness checks, explicit gap-finding, or final validation
5. Return to `reviewer` for the final quality check when a separate implementation skill owned the work
6. Be honest in user-facing reporting: state what is verified, what is inferred, and what remains blocked, partial, or unvalidated

### Specialist Skills

Load specialist skills when the task clearly requires domain expertise:

- **reviewer**: Code review, quality gate, production readiness (includes DRY/simplification)
- **software-development-life-cycle**: Architecture, SDLC process, cross-domain engineering
- **preserve-existing-flow**: Universal pre-edit gate for existing source files and brownfield flow preservation before changing existing functions, loops, handlers, queues, state machines, transport flows, firmware flows, protocol flows, or source-of-truth ownership
- **web-development-life-cycle**: Web performance, SEO, browser compatibility
- **mobile-development-life-cycle**: Mobile lifecycle, permissions, offline sync
- **backend-and-data-architecture**: API design, database schemas, microservices, messaging
- **cloud-and-devops-expert**: Infrastructure as Code, CI/CD pipelines, container orchestration, staged rollout doctrine, red-team and blue-team operations, and deployment evidence gates
- **qa-and-automation-engineer**: Test automation, E2E frameworks, load testing
- **security-and-compliance-auditor**: Vulnerability hunting, threat modeling, compliance
- **ui-design-systems-and-responsive-interfaces**: Design systems, responsive UI, brownfield visual fidelity, component quality, and generic-looking UI repair
- **ux-research-and-experience-strategy**: UX research, user testing, journey friction, decision architecture, and recovery-path quality
- **git-expert**: Complex git operations, issue-driven worktree flow, branching strategy, and clean push hygiene
- **memory-status-reporter**: Memory health, daily learnings, mistake ledgers, heuristic status reporting, and explicit recap reporting

### Keep It Simple

- Don't load multiple skills for simple tasks
- Use single skill when sufficient
- Don't route to `reviewer` as reflex triage when a primary domain skill or focused local path already fits
- For simple docs-only changes, use native or local validation unless risk, scope, or the user explicitly requires review
- Let Claude Code CLI's native capabilities handle basic operations

## Skill-Focused Execution

- Keep one primary skill responsible for the user-facing answer.
- Compose supporting skills only through deterministic, documented workflow steps when they add clear value.
- Keep context boundaries explicit: expose only the instructions, files, tool results, and memory artifacts needed for the current task.
- Use native `claude-skills` commands for routing, validation, review, memory, and compaction when those surfaces own the job.

### Agent Profiles

Your managed Claude Code home should expose these 13 skill-owned agent profiles under `~/.claude/agent-profiles/*.toml`:

- **backend-and-data-architecture**: Backend systems, APIs, data models, caching, and messaging
- **cloud-and-devops-expert**: Infrastructure, CI/CD, containers, and IaC
- **git-expert**: Git workflows, history surgery, branching, and release hygiene
- **memory-status-reporter**: Memory health, learning recaps, mistake ledgers, user-needs summaries, and heuristic status reporting
- **mobile-development-life-cycle**: Android and iOS lifecycle, permissions, offline sync, and release flow
- **preserve-existing-flow**: Brownfield ownership tracing, `~/.claude/memories/workspaces/<workspace-slug>/flow/flow-check.json` evidence, and behavior preservation before existing-source edits
- **qa-and-automation-engineer**: Test automation, regression coverage, E2E flow, and validation strategy
- **reviewer**: Feedback, code review, production-readiness checks, and final quality gate
- **security-and-compliance-auditor**: Vulnerability hunting, threat modeling, and compliance checks
- **software-development-life-cycle**: Sequencing, architecture framing, and cross-domain delivery coordination
- **ui-design-systems-and-responsive-interfaces**: Responsive UI, accessibility, design systems, and visual consistency
- **ux-research-and-experience-strategy**: Research planning, usability evidence, and experience strategy
- **web-development-life-cycle**: Web app architecture, browser behavior, performance, SEO, and deployment

The old generic `default`, `explorer`, `worker`, `architect`, and `awaiter` TOMLs are not the repo-managed profile surface anymore. Runtime helper roles may still exist inside Claude Code, but the managed install should mirror these 13 specialist skill profiles instead.
