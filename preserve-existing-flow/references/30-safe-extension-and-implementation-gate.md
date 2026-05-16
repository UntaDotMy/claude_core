# Safe Extension Pattern and Implementation Gate

This reference describes the implementation shape that preserves existing flows, the questions that must be answered before editing, and the discipline that keeps patches small and reviewable.

## Safe Extension Pattern

Prefer this shape every time:

1. **Keep the original path intact.** The working flow continues to exist and pass its existing tests.
2. **Add a new typed intent, packet, command, event, or adapter beside the original path.** New behavior gets a new shape, not a smuggled override of the old shape.
3. **Route new behavior through the same owner layer that already performs side effects.** If a transport drain owns sending today, the new behavior also sends through that drain.
4. **Keep producers side-effect-light.** Producers create intent. Owners perform effects. New producers should not grow ad-hoc side effects.
5. **Pop, acknowledge, clear, or finalize only after the owner confirms success.** Premature cleanup hides retry needs and races.
6. **Validate startup, runtime, failure, retry, and recovery paths.** Five passes — boot the system, run the happy path, force a failure, force a retry, force a recovery — before claiming the change is safe.

This shape is the default. Departures need a written rationale in the working brief.

## Implementation Gate

Before editing, answer these questions. If any answer is "I don't know", stop and continue reading; do not edit.

- What currently works?
- Which function or module owns the current behavior?
- Which function only produces data?
- Which function performs the side effect?
- Which queue, state, storage format, or transport contract is being reused?
- Will the new behavior change the meaning of existing data?
- What other consumers read this data?
- What breaks if this structure changes?
- Is this a layered extension or an overwrite?
- Is user approval needed because ownership is changing?

Record the answers in the working brief. The reviewer will check for them.

## Reporting Format Before Code

When the user asks to understand first, report in this structure:

- **Current flow** — concise step-by-step flow with file and function anchors.
- **Preserved owner** — the original function or module that should remain authoritative.
- **Drift or risk** — where current or planned code bypasses, overwrites, duplicates, or mixes ownership.
- **Recommended shape** — how to layer the change without replacing the original flow.
- **Implementation boundary** — files and functions that would need changes later, if approved.
- **Blockers or unknowns** — facts not proven yet.

This is the pre-edit deliverable. It is also the structure the reviewer will read against the diff.

## Code Change Rules (When Implementation Is Approved)

- **Make small patches.** A change that touches three files is reviewable; one that touches thirty hides regressions.
- **Touch the owner layer, not only the symptom branch.** Fix the source of truth; let consumers stay simple.
- **Re-read the exact function before patching.** Memory of the file is stale the moment another edit lands. Re-read every batch.
- **Re-read direct callers and callees before finalizing.** The patch is correct only if its neighbors still hold the contract.
- **Keep old working behavior unless the user explicitly asked to replace it.** Layered, not overwritten.
- **Delete dead duplicate logic only when the replacement owner is proven and validated.** If the new owner has not run startup, runtime, failure, retry, and recovery, do not delete the old one.
- **Add comments only for non-obvious ownership or protocol rules.** Code should read as itself; comments earn their place when the why is hidden.
- **Run the narrowest useful validation after each meaningful patch batch.** A failing test caught after one batch costs minutes; one caught after ten batches costs hours.

## Patch Batch Discipline

Group edits into batches that can be validated together. A useful batch:

- touches one owner layer or one consumer at a time
- has a single, statable intent ("route alarm reports through the typed drain")
- can be validated with a narrow command (a test, a build, a flow run)
- leaves the tree green at the end

Avoid batches that mix unrelated changes ("typed drain + rename helper + bump dep"). Each unrelated cleanup belongs in its own batch with its own validation.

## Validation Coverage

After each batch, ask whether the change has been exercised in:

- **startup** — the path the system runs once, on boot
- **runtime** — the steady-state path
- **failure** — the path triggered by an error or unexpected input
- **retry** — the path triggered by transient failure
- **recovery** — the path triggered after a fault to restore correct state

A bug fix that only proves the runtime path is incomplete. The reviewer will mark it as such.
