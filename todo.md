<!--
Purpose: Track the current repo initiative for global startup context and system-map discipline.
Caller: Contributors and AI agents using todo.md as the mutable roadmap source of truth.
Dependencies: Native memory scope, managed prompt surfaces, doc headers, and Rust tests.
Main Functions: Define the active project-scoped SYSTEM_MAP rollout and its closure criteria.
Side Effects: Drives current roadmap expectations for docs, prompts, and validation.
-->
# Current Initiative

This roadmap resets the repo around global startup context and navigation quality instead of the previous benchmark comparison push.

## Native Startup Context

- [x] Expose a canonical project-scoped global `SYSTEM_MAP.md` target through the native memory scope.
- [x] Keep `SYSTEM_MAP.md` under Codex-global project-scoped storage, not inside the user workspace.
- [x] Make the default startup flow read scoped memory plus `SYSTEM_MAP.md` before broad repository exploration.
- [x] Add a native `claude-skills memory system-map refresh` command so the scoped map can be refreshed without ad hoc repo scanning.

## Map Authoring Standard

- [x] Build or refresh the map with `trace-by-function / trace-by-flow` from the most relevant entrypoint instead of blind full-file scans.
- [x] Keep the global map optimized for fast AI follow-up context and default it to English unless the user explicitly asks for another language.
- [x] Group monorepo or multi-app systems by app inside one scoped project map.
- [x] Mark unknown facts as `Not found` instead of guessing.

## Navigation Discipline

- [x] Use the map and doc headers to go directly to the target; only widen search when the map is insufficient.
- [x] Ignore dependency, build, IDE, cache, and generated artifact trees during map creation unless the user explicitly asks for them.
- [x] Before editing, note the target file and traced function flow that will be touched.
- [x] Update the scoped global `SYSTEM_MAP.md` in the same session when the main flow or important file inventory changes.
- [x] On each prompt or resumed turn, read scoped memory plus `SYSTEM_MAP.md` before deciding whether broader repo exploration is needed.
- [x] Before each patch batch, re-read the exact target and, after each patch batch, re-read the edited target plus direct callers or callees before widening or finalizing.
- [x] Reuse the existing owner path instead of introducing duplicate helpers, duplicate functions, or parallel ownership for the same behavior.

## Documentation And Query Discipline

- [x] Every created or modified file must start with a short doc header that lists purpose, caller, dependencies, main functions, and side effects in the file's native comment style.
- [x] Keep logic and query changes general, configurable, and free from project-specific hardcoding.
- [x] For DB-heavy changes, explain the efficiency rationale, trade-offs, and waste avoided before closure.

## Delivery Guardrails

- [x] Keep one cohesive feature per branch and pair the change with the narrowest proving Rust tests.
- [x] Do not present startup-context work as done until the global path, managed prompts, and validating tests all agree.
- [x] Reviewer lanes must read the working brief, scoped memory, `SYSTEM_MAP.md`, changed-surface map, and proving validation before findings or approval.

---

# Issue #105: Native Hook-Compatible Command Output Compaction

Goal: Implement external-reducer-like command-output compaction natively in claude-skills (no external reducer dependency).

## Completed

- [x] Analyze current command_compaction.go implementation gaps vs external reducer tools
- [x] Expand command filters to cover 50+ commands (broad command coverage)
- [x] Implement token savings analytics (savings-dashboard gain dashboard)
- [x] Implement native hook installation for Codex CLI
- [x] Update AGENTS.md and docs with external benchmark guidance
- [x] Fix FilterRegistry disconnection - make all filters functional
- [x] Add missing git commands (push/pull/fetch)
- [x] Add JSON/structured data filter

## Pending

- [x] Implement transparent command rewriting (`claude-skills rewrite` and the managed hook provide native command-family rewrites through `claude-skills run --`)
- [ ] Implement transparent hook rewrite (Codex `PreToolUse` currently supports deny-and-rerun instead of transparent substitution)
- [x] Clean up dead code and consolidate
- [x] Add exit code propagation to filters
- [x] Add native hook integration for noisy command rerun guidance
- [x] Improve token estimation accuracy

---

# PR #134 Gap Closure: Token-Saving Core

Goal: Make the open token-proxy PR satisfy the capture-first model: command enters `claude-skills run`, raw stdout/stderr are saved before the agent sees them, semantic adapters compact the output, token savings are measured, and raw recovery remains available.

## Verified In PR #134

- [x] Add a separate `proxy/` layer with command AST, adapter trait, registry, raw store, renderer, token meter, and run orchestration.
- [x] Route `claude-skills run` through the proxy instead of the older generic runner path.
- [x] Save raw stdout/stderr to disk before compact output is returned.
- [x] Preserve `--full` and `--no-compact` passthrough behavior for exact raw output.
- [x] Add a first built-in Git adapter and generic fallback adapter.

## Fixed In Follow-Up

- [x] Register adapters by priority (`tests`, `git`, then `generic`) and expose them through `--list-adapters`.
- [x] Invoke `CommandAdapter::rewrite_args` in the proxy pipeline before execution.
- [x] Add a Tier 1 test-runner adapter for cargo/nextest/pytest/go/npm/pnpm/yarn/jest/vitest/playwright/mvn/gradle/dotnet test commands.
- [x] Replace Git adapter head/tail-only behavior with semantic status and diff summaries, while keeping raw recovery.
- [x] Persist compact output and updated metadata beside raw stdout/stderr.
- [x] Record token-based compaction events with `tokensBefore`, `tokensAfter`, `tokensSaved`, `adapterName`, `rawPath`, `compactPath`, `agent`, and `workspace`.
- [x] Update `gain` to report token totals and top token savers instead of presenting byte counters as tokens.
- [x] Expand `doctor` checks for Codex binary presence, `codex_hooks = true`, hooks file presence, PreToolUse Bash matcher, rerun wrapper installation, and unified-exec warning.
- [x] Add doc headers to new proxy and adapter source files.

## Still Pending

- [ ] Finish the full adapter matrix: search/read/list (`rg`, `grep`, `find`, `tree`, `ls -R`, large `cat`, `sed`, `head`, `tail`, `jq`), build/lint/log/infra (`tsc`, `eslint`, `ruff`, `mypy`, `cargo clippy`, `cargo build`, Docker, Kubernetes, Terraform, AWS logs).
- [ ] Add `claude-skills discover` to scan Codex transcripts/logs for large raw command output that bypassed the proxy and emit missed-savings fixes.
- [ ] Add optional YAML/TOML project filters layered after built-in Rust adapters, without replacing structured parsers for tests, diffs, JSON, and compiler errors.
- [ ] Split hook integration files into dedicated host modules (`hooks/codex.rs`, `hooks/claude.rs`, `hooks/cursor.rs`, `hooks/windsurf.rs`) once host-specific behavior exists.
- [ ] Add focused regression tests and fixtures for passing and failing outputs across the Tier 1 test runners and Git diff/status reducers.
