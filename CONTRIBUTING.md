# Contributing

## Purpose

This repository is a managed Codex skill pack, not a loose prompt collection. Contributions should preserve the same production-readiness standard across code, docs, tests, generated home wiring, and validation behavior.

## Contribution Workflow

1. Start from a concrete working brief.
2. Preserve one top-level plan item per explicit user task.
3. Keep the first implementation pass anchored to the named scope.
4. Prefer small validated batches over large rewrites.
5. Patch the source doctrine, validator, and contract coverage together when a rule is meant to fail closed.
6. Update README or supporting docs when user-facing behavior changes.
7. Run the proving loop before asking for review.

## Feature Delivery Rules

- One feature = one branch = one merge request.
- Do not mix unrelated features or fixes in the same branch.
- Use `git add -p` when selective staging is required.
- Review `git diff --cached` before each commit.
- Run `claude-skills git-workflow preflight --repo-root . --base-ref origin/main` before push or merge-request creation.
- When opening a PR or MR from the CLI, use a real multiline body or `--body-file` instead of embedding escaped newline sequences such as `\\n` in the published text.
- Follow [WORKFLOW.md](WORKFLOW.md) when the change touches branching, merge-request shape, or reviewer expectations.

## Required Validation

Run this default native loop from the repository root against a temporary Codex home target:

```bash
temporary_claude_home="$(mktemp -d)"
CODEX_TARGET_OVERRIDE="$temporary_claude_home" cargo run --bin claude-skills -- validate --profile smoke
CODEX_TARGET_OVERRIDE="$temporary_claude_home" cargo run --bin claude-skills -- install --repo-root "$PWD"
"$temporary_claude_home/claude-skills" verify --repo-root "$PWD"
"$temporary_claude_home/claude-skills" status --repo-root "$PWD"
```

Windows contributors should run the same Rust CLI shape from PowerShell:

```powershell
$temporaryCodexHome = Join-Path $env:TEMP "claude-skills-test-home"
New-Item -ItemType Directory -Force -Path $temporaryCodexHome | Out-Null
$env:CODEX_TARGET_OVERRIDE = $temporaryCodexHome
cargo run --bin claude-skills -- validate --profile smoke
cargo run --bin claude-skills -- install --repo-root .
& (Join-Path $temporaryCodexHome "claude-skills.exe") verify --repo-root .
& (Join-Path $temporaryCodexHome "claude-skills.exe") status --repo-root .
```

Use the live `~/.claude` target only as an intentional final check when the change specifically needs that real-home proof.

Full validate now proves the Rust-native CLI foundation. Install the stable Rust toolchain before running the complete repository loop locally; CI enforces the same Rust workspace proof.

When the change touches a narrower surface, also run the smallest direct proof that covers the edited area, such as `cargo test --workspace`, `cargo test -p <crate>`, or `cargo build --release --bin claude-skills`.

## Scope Rules

- Do not add parallel install or update entrypoints when the managed ones can absorb the change.
- Do not add new helper functions when existing code already owns the behavior cleanly.
- Do not present partial implementation as complete.
- Do not weaken runtime-safe clarification, live-research-first behavior, completion discipline, or memory-safety rules.

## Documentation Rules

- Keep committed comments and docs professional, concise, and neutral.
- Use README for end-user setup, architecture, and operational workflow.
- Use AGENTS.md and skill docs for agent doctrine, not marketing copy.
- Keep SECURITY.md current when the reporting path or validated security posture changes.

## Review Expectations

- Findings-first review for bugs, regressions, missing validation, or misleading docs
- Honest status labels for what is verified, inferred, skipped, or blocked
- Root-cause fixes over workaround-only patches

## Cross-Platform Expectations

- macOS, Linux, and Windows behavior should stay aligned
- the Rust-native CLI is the only supported install/update entrypoint
- Shell and PowerShell launcher scripts must not be reintroduced as runtime paths
