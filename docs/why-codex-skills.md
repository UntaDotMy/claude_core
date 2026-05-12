# Why `claude_skills`

This page is the narrow public comparison surface for `claude_skills`.

It is intentionally evidence-based instead of trying to claim broad overall leadership. The point is to show where this repository is stronger today, where it is still catching up, and which operator problems it is designed to solve.

Comparison references used for this page:

- Native Codex CLI primitives and current repo behavior in this repository
- Anonymized runtime-shell comparator class
- Anonymized workflow-teaching comparator class

Those public surfaces were re-checked on 2026-04-06 before this page was added.

## Choose `claude_skills` when the team needs

- one explicit path from intake to proof: `workflow start` -> `workflow cockpit` -> `workflow finish`
- deterministic local review gates before a PR is treated as real
- explicit hosted-check discipline after the PR is open
- durable working-brief, requirement, lane, and closure artifacts
- native install, update, status, verify, and uninstall flows for the managed pack
- a branch-closeout path that refuses to call work done while proof is still incomplete

## Stronger than raw Codex when

Raw Codex gives powerful primitives. This repository adds a stricter operator layer on top of them.

Current strengths over raw Codex:

- operator-first workflow routing, cockpit, worktree, and finish commands
- tracked completion-gate artifacts instead of chat-only closure claims
- native pre-commit and pre-PR review surfaces
- hosted-check fix-loop guidance and branch-closeout discipline
- repo-managed install and status surfaces instead of ad hoc local setup

Choose raw Codex instead when the team wants the thinnest possible layer and does not need the extra workflow or proof posture.

## Stronger than runtime-shell comparator when

The current public `runtime-shell comparator` surface emphasizes a richer runtime experience around Codex: setup, role keywords, team runtime helpers, HUDs, and durable repo-local runtime state.

Current `claude_skills` advantages are narrower:

- stronger deterministic closeout posture around review, verification, and hosted green checks
- a more explicit branch-to-proof path under one workflow surface
- native repo-managed install, update, status, verify, and uninstall commands
- clearer distinction between local proof, hosted proof, and final closure

Current `runtime-shell comparator` advantages still matter:

- more polished runtime-first onboarding
- stronger keyword-driven day-to-day interaction surfaces
- more visible team runtime and HUD presentation

Choose `claude_skills` over `runtime-shell comparator` when the priority is stricter completion proof and operator-controlled closure. Choose `runtime-shell comparator` when the priority is a richer conversational runtime layer first.

## Stronger than workflow-teaching comparator when

The current public `workflow-teaching comparator` surface emphasizes skill-driven workflow phases, explicit design gates, TDD discipline, and composable process skills.

Current `claude_skills` advantages are:

- native manager packaging and release-managed install path
- native review gates and host-neutral review artifact generation
- integrated hosted-check watching and PR-fix workflow surfaces
- workflow worktree and branch-finish paths that are already productized under one CLI

Current `workflow-teaching comparator` advantages still matter:

- very clear phase-based software workflow education
- stronger skill-first planning and implementation framing
- a simpler install story for some Codex users through `npx`

Choose `claude_skills` over `workflow-teaching comparator` when the repo needs a stricter closure system with built-in manager and hosted-check discipline. Choose `workflow-teaching comparator` when the team wants a skill-first process overlay with lighter packaging expectations.

## Where `claude_skills` is still catching up

This repository should not claim to be ahead everywhere.

Current known gaps:

- benchmark evidence is now shipped across 8 tracked scenarios, and the shared harness contract now exists, but peer repos still need more source-backed scenario entries before universal claims are justified
- demo flows now cover greenfield delivery, stateful fixes, hosted rescue, branch closeout, closure proof, Windows validation, docs governance, and regression hardening, but only for the tracked scenarios published so far
- comparison docs now include a narrower winner-by-dimension scorecard, but the suite still should not make broad market-leadership claims
- runtime polish is improving, but the operator surface is still more rigorous than friendly

## Practical summary

Use `claude_skills` when the team wants a Codex-native workflow layer that is harder to fake as finished.

Use another layer when the priority is a friendlier runtime shell, a more guided skill-first teaching surface, or a lighter install footprint with less closure discipline.
