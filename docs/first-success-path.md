# First Success Path

This guide is the fastest honest path through the current native workflow surface.

It is for a new operator who already has `claude_skills` installed and wants one satisfying end-to-end run without memorizing every workflow command first.

If you want the named native operator path first, run:

```bash
claude-skills workflow setup
```

The older `claude-skills workflow first-run` command remains supported as the static guided reference path.

Use `claude-skills workflow setup --request "..." --workstream-key feature-branch` when you want the same operator path with your own request text and tracked workstream key. The command refreshes install, runs doctor, routes the request, starts the workstream, and returns the live cockpit shell in one pass.

## Goal

Start from one broad request, enter a tracked workstream, keep the operator view visible, prove the branch, and only then close it out.

## Five-Minute Path

1. Refresh the native shell and verify readiness.

```bash
claude-skills install
claude-skills doctor
```

What to look for:
- managed install and wrapper health
- doctor follow-up guidance if the environment is not ready yet

2. Start from one broad request.

```bash
claude-skills workflow route --request "Compare the current repo, fix the biggest gaps, and carry the branch to closure"
```

What to look for:
- the short `Start Now` command
- the recommended mode
- the plain-language mode summary

3. Enter the workstream with the recommended lane.

```bash
claude-skills workflow start --mode auto --workstream-key feature-branch --request "Compare the current repo, fix the biggest gaps, and carry the branch to closure"
```

If you already know the work is coordinated, it is also valid to start the explicit team lane:

```bash
claude-skills workflow start --mode team --workstream-key feature-branch --request "Coordinate the next multi-lane task"
```

Preset shorthand when the job shape is already obvious:

- `autopilot`
  - `claude-skills workflow start --preset autopilot --workstream-key feature-branch --request "Carry the current task to closure"`
  - Use when broad feature or maintenance work needs one owner driving to closure.
  - Proof to expect: the brief, completion gate, cockpit proof board, review pass, and native finish checks stay current.
  - If interrupted: reopen with `claude-skills workflow status --workstream-key feature-branch`, `claude-skills workflow cockpit --workstream-key feature-branch`, and `claude-skills workflow resume --workstream-key feature-branch`.
- `debug`
  - `claude-skills workflow start --preset debug --workstream-key feature-branch --request "Trace the failing behavior, fix it, and prove it"`
  - Use when the root cause is still unclear or the failure crosses runtime or recovery boundaries.
  - Proof to expect: traced behavior mismatch, narrow repro or regression proof, and hosted-check repair proof when relevant.
  - If interrupted: return through `claude-skills workflow cockpit --workstream-key feature-branch`, `claude-skills workflow resume --workstream-key feature-branch`, and `claude-skills workflow branch hosted fix-loop --workstream-key feature-branch` for hosted failures.
- `tdd`
  - `claude-skills workflow start --preset tdd --workstream-key feature-branch --request "Write the proving test first, then implement the feature"`
  - Use when failing-test-first discipline is the safest way to hold scope and prove the change.
  - Proof to expect: failing proof first, fix proof second, regression proof third, plus the normal review and finish checks.
  - If interrupted: use `claude-skills workflow cockpit --workstream-key feature-branch`, `claude-skills workflow resume --workstream-key feature-branch`, and `claude-skills memory completion-gate check --workstream-key feature-branch --require-closure-ready`.
- `review`
  - `claude-skills workflow start --preset review --workstream-key feature-branch --request "Audit the current branch and call out the real gaps"`
  - Use when verification is the primary job and implementation is secondary.
  - Proof to expect: workflow audit, reviewer proof, and closeout checks drive the decision.
  - If interrupted: recover from `claude-skills workflow audit --workstream-key feature-branch`, `claude-skills workflow cockpit --workstream-key feature-branch`, and `claude-skills workflow resume --workstream-key feature-branch`.
- `eco`
  - `claude-skills workflow start --preset eco --workstream-key feature-branch --request "Carry the small maintenance task to closure"`
  - Use when the task is smaller but still deserves tracked closure.
  - Proof to expect: the same brief, cockpit, and finish structure, with the narrowest honest proving validation for the touched scope.
  - If interrupted: reopen with `claude-skills workflow status --workstream-key feature-branch`, `claude-skills workflow cockpit --workstream-key feature-branch`, and `claude-skills workflow resume --workstream-key feature-branch`.
- `parallel`
  - `claude-skills workflow start --preset parallel --workstream-key feature-branch --request "Coordinate the next multi-lane task"`
  - Use when the work already implies specialist or parallel lanes.
  - Proof to expect: the lane board, required-lane completion, proof board, and finish blockers stay visible until every required lane is terminal.
  - If interrupted: recover from `claude-skills workflow cockpit --workstream-key feature-branch`, `claude-skills workflow team resume --workstream-key feature-branch`, and `claude-skills workflow team await --workstream-key feature-branch --timeout-seconds 300 --poll-seconds 15`.

4. Keep the live watch surface open while the work is moving.

```bash
claude-skills workflow dashboard --workstream-key feature-branch
```

Use the dashboard to watch:
- live runtime state and next action
- team-health status for active, stalled, and required lanes
- active and stalled lanes
- closure blockers
- audit pass or fail state

5. Use the proof-board console when you need route, blockers, and closeout guidance in one place.

```bash
claude-skills workflow cockpit --workstream-key feature-branch
```

Use the cockpit to watch:
- route and next command
- active phase, requirement, and blocker state
- hosted check summary when a PR exists

6. Turn local work into proof before you call it done.

```bash
cargo test --workspace
claude-skills review pre-pr --base-ref origin/main
claude-skills git-workflow preflight --repo-root . --base-ref origin/main
```

7. If the branch is on GitHub, wait for the real hosted result.

```bash
gh pr checks --watch
```

If a hosted lane fails, fix the root cause on the same branch, push again, and wait again.

8. Close the workstream only after the proof is real.

```bash
claude-skills workflow finish --workstream-key feature-branch
```

If the PR is already green and the branch is in the closeout lane, `claude-skills workflow branch finish --workstream-key feature-branch` is the shorter merge-oriented shortcut.

## Why This Is The First Success Path

- It starts from a broad request instead of forcing workflow vocabulary first.
- It now exposes one named native operator path instead of leaving install, doctor, route, start, and the live shell to be assembled manually.
- It uses the existing native workflow surface instead of a separate onboarding-only path.
- It keeps one visible route from intake to proof to closeout.
- It ends with real proof, not just a confident-looking summary.

## If You Want The Slightly Shorter Version

When the task is single-owner and you do not need to route first:

```bash
claude-skills workflow start
claude-skills workflow cockpit
claude-skills workflow finish
```

That is the default operator path, but the routed path above is the better first run when the request still feels broad.
