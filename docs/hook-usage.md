<!--
Purpose: Document the managed Codex lifecycle hook contract for agents and operators.
Caller: README, AGENTS.md, and contributors looking for the hook usage and rerun handling rules.
Dependencies: ~/.claude/hooks.json layout, claude-skills run wrapper, and claude-skills rewrite.
Main Functions: Explain what the hook does, what it does not do, and how agents interact with transparent rewrite.
Side Effects: None; documentation only.
-->
# Codex Hook Usage

The managed hook set is installed at `~/.claude/hooks.json` by the one-line installer and by `claude-skills hook install`. It manages every supported Codex lifecycle event: `PreToolUse`, `PermissionRequest`, `PostToolUse`, `PreCompact`, `PostCompact`, `SessionStart`, `UserPromptSubmit`, and `Stop`. `PreToolUse` is the only event that blocks and reroutes noisy Bash commands because token-saving has to happen before command output exists; the other lifecycle hooks are native no-op/checkpoint surfaces for memory and recovery wiring. This page is the agent-facing contract; `claude-skills hook instructions` prints the same content (`--format markdown` is the default; `--format json` returns a structured payload).

## Token-saving rule

The goal is to prevent noisy raw command output from entering Codex context. Do not run a raw noisy command first and compact afterward; route through `claude-skills run -- <command>` or rely on the hook's transparent rewrite before noisy output is produced.

## What the hook does

- Inspects supported Bash commands and transparently rewrites them via `toolInputOverride`.
- Wraps the original command in `claude-skills run --` before it executes, preventing noisy output from entering context.
- Emits command-specific semantic reducers, high-signal error/warning context, and compacted head/tail summaries for noisy or long output while recording the full raw stream under the Codex home raw-output recovery log.
- Records native savings analytics for `claude-skills gain`, including command family and reducer dimensions.

## What the hook does not do

- It does not cover `apply_patch` or other file-edit tool surfaces. Existing-source edits stay governed by Preserve Existing Flow and review gates (`~/.claude/memories/workspaces/<workspace-slug>/flow/flow-check.json`, `claude-skills review pre-pr`, `claude-skills review gates check`).
- It does not replace flow or review gates.

## Transparent Rewrite Handling

When the hook intercepts a supported Bash command, it returns `permissionDecision: "allow"` with a `toolInputOverride` that wraps the command in `claude-skills run -- ...`. The execution proceeds transparently — no manual rerun is needed.

Example: a raw `cargo test --workspace` is transparently rewritten to `claude-skills run -- cargo test --workspace` and the compacted output is returned directly.

## Compaction surface hierarchy

- **Level 1 — Direct native wrapper:** `claude-skills run -- <command>` is the most reliable transparent surface; it owns command execution, shell-aware parser/rewrite support, command-specific reducers, high-signal extraction, head/tail compaction, raw-output recovery, and native savings analytics in one step. Add `--stream` before `--` when bounded live progress is more important than keeping the terminal silent until final compaction.
- **Level 2 — Rewrite helper:** `claude-skills rewrite "<command>"` returns the resolved wrapper without executing it. It recognizes common shell wrappers, environment-prefix commands, and pipelines; shell syntax is rerouted through `bash -lc` so the wrapper executes the intended command rather than a partial token.
- **Level 3 — Hook guidance:** `claude-skills hook install` registers the managed lifecycle hooks described above. The `PreToolUse` hook may transparently rewrite tool input via `toolInputOverride` (not a block-and-rerun).
- **Level 4 — Native install/update:** Use the installed Rust binary directly for update, verify, status, hooks, and compaction. Shell and PowerShell profile wrappers are not supported runtime entrypoints.

## Related commands

```bash
claude-skills hook install        # Install managed lifecycle hooks in ~/.claude/hooks.json
claude-skills hook uninstall      # Remove managed lifecycle hooks
claude-skills hook list           # List installed hooks
claude-skills hook show           # Show hooks.json content
claude-skills hook instructions   # Print this contract (markdown by default)
claude-skills hook instructions --format json   # Same contract as a structured payload
```
