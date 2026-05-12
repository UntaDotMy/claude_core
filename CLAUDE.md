# CLAUDE.md — claude-skills Project Guide

## Project Overview

This is the claude-skills project — native delivery rails for Claude Code. It provides:
- 13 managed specialist profiles for software delivery
- Workflow routing and escalation rules
- Review gates (pre-commit, pre-PR)
- Professional text templates
- A Rust CLI tool for workflow, memory, and command compaction

## Key Files

- `00-skill-routing-and-escalation.md` — Read this first. Defines skill routing and escalation.
- `AGENTS.md` — Agent operating doctrine.
- `WORKFLOW.md` — Branch and completion rules.
- `templates/` — Professional text templates (commit, PR, final response, review).
- `.claude/review.json` — Review policy configuration.

## Specialist Profiles

Each profile directory contains:
- `SKILL.md` — Main skill definition and guidance
- `agents/claude.yaml` — Agent configuration
- `references/` — Deep knowledge files

13 profiles: `software-development-life-cycle`, `web-development-life-cycle`, `mobile-development-life-cycle`, `backend-and-data-architecture`, `cloud-and-devops-expert`, `qa-and-automation-engineer`, `security-and-compliance-auditor`, `git-expert`, `preserve-existing-flow`, `reviewer`, `ui-design-systems-and-responsive-interfaces`, `ux-research-and-experience-strategy`, `memory-status-reporter`.

## Routing Rules

1. Route to the appropriate specialist when work is domain-specific.
2. Run `preserve-existing-flow` before editing any existing source file.
3. Run `reviewer` before closing any work.
4. Use `templates/` for commit bodies, PR bodies, final responses, and review summaries.
5. Read `WORKFLOW.md` for branch naming, commit format, and completion rules.

## Commands

- `claude-skills workflow route --request "..."` — Route a request
- `claude-skills workflow start --preset <preset> --request "..."` — Start work
- `claude-skills workflow cockpit` — Watch state
- `claude-skills review pre-commit` — Pre-commit review
- `claude-skills review pre-pr` — Pre-PR review
- `claude-skills workflow finish` — Finish branch with proof
- `claude-skills run -- <command>` — Run with output compaction
- `claude-skills memory scope resolve --create-missing --refresh-system-map` — Refresh memory

## Build & Test

```bash
cargo build
cargo test
cargo fmt --all -- --check
cargo clippy -- -D warnings
```
