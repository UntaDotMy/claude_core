# Review Fail Conditions and Final Answer Format

This reference is the verdict half of the skill. It enumerates the conditions that must fail a brownfield change in review, and the structure of the final answer when the change is closed out.

## Review Fail Conditions

Fail the change if it:

- **Bypasses an existing owner without explaining why.** A new direct path around a working drain is an ownership split; the change must justify why the old owner cannot serve the new behavior.
- **Sends, writes, notifies, persists, or mutates external state from a producer path that should only create intent.** Producers create; owners perform. Smuggling a side effect into a producer is an ownership leak.
- **Mixes different packet or data shapes in one raw queue without updating every consumer.** Old consumers misinterpret or silently drop new shapes; the contract needs a typed packet or an updated consumer set.
- **Changes a loop, scheduler, interrupt handler, callback, or recovery path without tracing dependent behavior.** These are the highest-blast-radius layers; untraced edits there cause regressions far from the diff.
- **Copies reference code blindly instead of preserving the reference architecture pattern.** Copy the pattern (entry point → producer → owner → recovery), not the literal feature set.
- **Presents a partial slice as complete while another required owner path is still inconsistent.** Half-done flows are worse than untouched flows; the reviewer will not approve a "done" claim that leaves the recovery path silent.
- **Changes original or reference files that were marked read-only.** The user or the brief named these files off-limits; the change crossed the line.
- **Ignores a user instruction to report first or avoid edits.** "Do not change anything yet" means read-only; the reviewer will fail any change made before that instruction was lifted.

Each fail condition can be verified from the diff plus the working brief. If the brief does not exist, that itself is a fail — the skill requires a working brief before edits.

## Severity Ladder

Mirror the reviewer skill's severity vocabulary so feedback lands clearly:

- **Blocker** — bypassed owner, untraced loop edit, ownership migration without approval, partial slice presented as complete, edited read-only file.
- **Major** — producer growing side effects without rationale, queue mixing without consumer audit, missing recovery-path validation.
- **Minor** — comment-only documentation drift, batch discipline lapses (unrelated cleanup mixed in), narrow validation coverage on a non-critical path.
- **Nit** — naming or organization tweaks that do not affect ownership or correctness.

Blockers must be fixed before merge. Majors must be fixed or explicitly accepted with a mitigation plan. Minors and nits land as suggestions.

## Final Answer Requirements

When finished, state:

- **What was verified.** Concrete reads, runs, tests, or inspections — not "I checked the code".
- **What changed, if anything.** Files and functions touched, in one line each.
- **What existing behavior was preserved.** The owners, queues, state machines, and recovery paths that stayed authoritative.
- **What validation ran.** The exact commands, tests, or simulator runs, with pass / fail evidence.
- **What remains unverified or blocked.** Anything the brief flagged that did not get proven, with why.

Never claim production-ready completion if ownership, recovery, or validation is incomplete. The phrase "production-ready" is reserved for flows where startup, runtime, failure, retry, and recovery have all been exercised and proven.

## Anti-Patterns in the Final Answer

Reject these closing patterns even when the underlying change is sound:

- "I think it works" — replace with the validation that proved it.
- "Should be safe" — replace with the named owners that stayed authoritative.
- "Tested locally" — name the commands and the results.
- "Backwards compatible" — name the old consumers that still pass.
- "Will follow up later" — file the follow-up as a tracked item before closing, not as a phrase in the answer.

The final answer is the receipt for the change. If a future reviewer cannot reproduce the verification from the answer alone, the answer is incomplete.

## Handoff to the Reviewer Skill

When a brownfield change closes, the reviewer skill takes over for the production-readiness verdict. The preserve-existing-flow final answer is the input the reviewer reads against the diff. A clean handoff includes:

- the working brief (requested, preserved, owners, evidence)
- the changed-surface map (files, functions, owners touched)
- the validation evidence (commands, results, edge cases covered or marked blocked)
- the open follow-ups, if any

The reviewer can then run its own ladder (smoke → functional → integration → UI → load → stress → security) without re-deriving the ownership picture from scratch.
