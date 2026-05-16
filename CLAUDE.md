# CLAUDE.md — claude-core Project Guide

## Project Overview

This is the claude-core project — native delivery rails for Claude Code. It provides:
- 13 specialist Claude Code **skills** for software delivery (`<name>/SKILL.md`)
- 13 matching Claude Code **subagents** for token-efficient delegation (`.claude/agents/<name>.md`)
- 13 internal **managed profiles** consumed by the CLI (`<name>/agents/claude.yaml`)
- Workflow routing and escalation rules
- Review gates (pre-commit, pre-PR)
- Professional text templates
- Hooks wired into Claude Code's `settings.json` for transparent command rewriting and auto-routing
- A Rust CLI (`claude-skills`) for workflow, memory, command compaction, and hook installation

### Terminology

These three terms are **not** interchangeable:

| Term | What it is | Where it lives | Schema |
|---|---|---|---|
| **Skill** | Claude Code knowledge unit loaded into the main conversation when it matches a request | Source: `<name>/SKILL.md`. Installed: `~/.claude/skills/<name>/SKILL.md` | YAML frontmatter with `name`, `description`, `when_to_use`, `allowed-tools` |
| **Subagent** | Claude Code delegation target that runs in an isolated context window | `.claude/agents/<name>.md` (project) or `~/.claude/agents/<name>.md` (user) | YAML frontmatter with `name`, `description`, `tools`, `model` |
| **Managed profile** | Internal CLI configuration that wires reasoning effort, default prompts, and policy for the `claude-skills` runtime — **not** seen by Claude Code | `<name>/agents/claude.yaml` | Custom YAML consumed by the Rust CLI |

A "skill" runs in the main thread (instructions inline, costs ongoing tokens). A "subagent" runs in its own context window (saves main-thread tokens but adds delegation overhead). The "managed profile" is invisible to Claude Code itself — it only configures how `claude-skills` orchestrates work.

## Key Files

- `00-skill-routing-and-escalation.md` — Read this first. Defines skill routing and escalation.
- `AGENTS.md` — Agent operating doctrine (uses "agent" in the broad sense — covers skills, subagents, and managed profiles).
- `WORKFLOW.md` — Branch and completion rules.
- `templates/` — Professional text templates (commit, PR, final response, review).
- `.claude/review.json` — Review policy configuration.
- `.claude/hooks.json` — Claude Code hook wiring rendered by `claude-skills hook install`.
- `.claude-plugin/plugin.json` — Plugin manifest for Claude Code's plugin system.

## Specialist Layout

Each specialist contains three artifacts:
- `<name>/SKILL.md` — Skill definition (loaded by Claude Code when relevant)
- `.claude/agents/<name>.md` — Subagent definition (delegation target with isolated context)
- `<name>/agents/claude.yaml` — Managed profile (CLI runtime configuration)
- `<name>/references/` — Deep knowledge files referenced by SKILL.md

13 specialists: `software-development-life-cycle`, `web-development-life-cycle`, `mobile-development-life-cycle`, `backend-and-data-architecture`, `cloud-and-devops-expert`, `qa-and-automation-engineer`, `security-and-compliance-auditor`, `git-expert`, `preserve-existing-flow`, `reviewer`, `ui-design-systems-and-responsive-interfaces`, `ux-research-and-experience-strategy`, `memory-status-reporter`.

## Schema Compliance Notes

**SKILL.md frontmatter** follows the official Claude Code skill spec. The fields used in this project are all documented Claude Code fields: `name` and `description` are required; `when_to_use`, `allowed-tools`, `effort`, and `paths` are official optional fields. Reference: https://code.claude.com/docs/en/skills.

Other official optional fields not currently used here include `disable-model-invocation`, `user-invocable`, `argument-hint`, `arguments`, `model`, `context`, `agent`, `hooks`, and `shell`. Add them deliberately when a skill needs that capability.

**Subagent frontmatter** (`.claude/agents/<name>.md`) follows the official spec: `name`, `description`, `tools` (comma-separated bare tool names), `model` (`opus`, `sonnet`, `haiku`, or `inherit`). The optional `color` field is supported. Note: scoped tool patterns like `Bash(git diff:*)` work in SKILL.md `allowed-tools` but not in subagent `tools` — subagents use bare tool names. Reference: https://code.claude.com/docs/en/sub-agents.

**Hook events** (`.claude/hooks.json`) are wired through `claude-skills hook <event>` for every Claude Code lifecycle event the manager observes. The event list is kept in sync with the hooks reference at https://code.claude.com/docs/en/hooks (see `rust/crates/claude-skills/src/hooks/claude.rs::EVENTS`). When Anthropic adds or renames events, update both that constant and `.claude/hooks.json`. Events the binary supports but Claude Code does not yet emit are silently ignored at runtime.

**Plugin manifest** (`.claude-plugin/plugin.json`) follows the official plugin schema with `skills`, `agents`, and `hooks` paths. Reference: https://code.claude.com/docs/en/plugins-reference.

**Token-saving proxy**: command-output compaction lives in `rust/crates/claude-skills/src/proxy/`. The native `claude-skills run -- <command>`, `claude-skills rewrite`, and `claude-skills gain` surfaces own this work. When Claude Code introduces native compaction primitives, prefer them and keep this layer thin.

## Routing Rules

1. Routing is driven by Claude Code's native skill matcher against the installed `~/.claude/skills/<name>/SKILL.md` files — each skill's frontmatter (`description`, `when_to_use`) is what triggers selection. The `UserPromptSubmit` hook is intentionally silent to preserve cache; the static operating contract is delivered once per session via `SessionStart` `additionalContext` and the project CLAUDE.md, then cached for the rest of the session.
2. Run `preserve-existing-flow` before editing any existing source file.
3. Run `reviewer` before closing any work.
4. Delegate to the matching `.claude/agents/<name>.md` subagent for heavy work that benefits from an isolated context window (saves main-thread tokens).
5. Use `templates/` for commit bodies, PR bodies, final responses, and review summaries.
6. Read `WORKFLOW.md` for branch naming, commit format, and completion rules.

## Commands

- `claude-skills workflow route --request "..."` — Route a request
- `claude-skills workflow start --preset <preset> --request "..."` — Start work
- `claude-skills workflow cockpit` — Watch state
- `claude-skills review pre-commit` — Pre-commit review
- `claude-skills review pre-pr` — Pre-PR review
- `claude-skills workflow finish` — Finish branch with proof
- `claude-skills run -- <command>` — Run with output compaction
- `claude-skills memory scope resolve --create-missing --refresh-system-map` — Refresh memory
- `claude-skills hook install` — Wire hooks into Claude Code's `settings.json`

### Declarative Filter Registry

`claude-skills run` supports project-specific TOML filter files that compact command output without writing Rust code.

Place a filter file at either:
- `.claude-skills/filters.toml`
- `claude-skills.filters.toml` (project root)

Example:
```toml
[[filter]]
name = "cargo-test"
command = "cargo test"
match_mode = "starts_with"  # starts_with | exact | contains | regex
keep = ["FAILED", "error", "test result"]
remove = ["running", "Doc-tests"]
max_lines = 50
```

| Field | Required | Default | Description |
|---|---|---|---|
| `name` | yes | — | Filter identifier |
| `command` | yes | — | Command string to match |
| `match_mode` | no | `starts_with` | How to match: `starts_with`, `exact`, `contains`, `regex` |
| `exit_code` | no | any | Only apply when exit code matches |
| `keep` | no | `[]` | Line substrings to retain (empty = keep all non-removed) |
| `remove` | no | `[]` | Line substrings to discard before keep |
| `max_lines` | no | `40` | Max lines to retain |
| `enabled` | no | `true` | Toggle filter on/off |

## Build & Test

```bash
cargo build
cargo test
cargo fmt --all -- --check
cargo clippy -- -D warnings
```
