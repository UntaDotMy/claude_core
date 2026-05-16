# Ownership Rules and No-Overwrite Discipline

This reference codifies the boundaries that protect existing flows from accidental overwrite, and the shortcuts that must be rejected during review.

## Ownership Boundaries

Every working flow has four roles. Identify and preserve them.

### Producer

- May create intent or data.
- Should not directly perform transport or side effects unless it is already the established owner.
- Examples: a mapper that turns a UI event into a typed command, a sensor reader that builds a sample record, a parser that produces a request struct.

Producer code stays side-effect-light. If a producer also writes, sends, or persists today, that is an existing ownership decision — do not pile new side effects on top until the brief has confirmed the pattern.

### Transport / Side-Effect Owner

- Sends data, notifies hosts, writes to external systems, pops queues, acknowledges packets, performs I/O.
- Owns retry, success, and failure semantics for the side effect.
- Examples: a USB transmit drain, an HTTP client wrapper, a database writer, a message bus publisher, a render loop.

When new behavior needs a side effect, route it through the existing transport owner instead of bypassing it.

### State Owner

- Decides current state, mode, mapping, persistence, or recovery behavior.
- Owns transitions and invariants.
- Examples: a connection state machine, a feature-flag resolver, a mode register, an auth session store.

State changes must go through the state owner. A consumer that reads cached state is not the owner of that state.

### Queue Owner

- Defines packet shape, push rules, pop rules, overflow behavior, and success or failure handling.
- Owns the queue contract for every consumer.
- Examples: a typed event queue, a lock-free ring buffer, a job queue with retry policy.

If new behavior needs another report type, command type, event type, or message type, prefer a typed packet or layered path through the existing queue or handler pattern. Do not push incompatible data into an old raw queue unless every consumer is updated together.

## No-Overwrite Rules

Reject these shortcuts during review:

| Shortcut | Why it fails | Safe alternative |
|---|---|---|
| Direct send from mapping, business, or producer code when an existing transport drain owns sending | Splits ownership; future side effects hide outside the drain | Route the new send through the existing drain owner |
| New payload shape into an existing queue without updating every consumer | Old consumers misinterpret or drop new payloads silently | Add a typed packet or version tag, then update every consumer in the same change |
| Modify a main loop because a new feature needs polling | Existing scheduler, tick owner, callback, or event source is bypassed; loop ownership becomes shared | Find the existing scheduler / event source and register through it |
| Duplicate an existing function with a similar new function | Two owners drift; bug fixes land in only one | Extend the existing function, or prove the old owner wrong before forking |
| Replace original behavior when a layered extension can preserve it | Working flow regresses; reviewers cannot tell what was intentional | Layer the new path beside the original |
| Patch only one consumer branch when the source of truth is elsewhere | Same incorrect state still flows through other branches | Fix at the source of truth, then audit consumers |
| Speculative fallback to hide an untraced root cause | Hides the real bug; recovery path becomes load-bearing for normal operation | Trace the root cause; add a fallback only when the failure mode is named |
| Refactor unrelated code while adding the requested behavior | Review surface inflates; regressions slip in unrelated to the brief | Keep the change minimal; open a separate ticket for cleanup |

## Reference Comparison Rule

When a reference implementation exists, compare by role instead of copying blindly:

- entry point to entry point
- producer to producer
- queue or storage to queue or storage
- transport owner to transport owner
- recovery path to recovery path

Copy the architecture pattern, not necessarily the exact feature set. If the reference only supports one report type but the current product needs multiple report types, preserve the reference flow idea while adding typed or layered handling for the extra report types.

A reference is a teaching tool, not a target to clone. Mark which parts you adopted, which parts you adapted, and which parts you intentionally rejected.

## Ownership Migration

If the requested behavior genuinely needs ownership to move (producer becomes a side-effect owner, queue gets a new contract, state machine adds a mode), this is no longer a layered extension — it is an ownership migration.

Ownership migration requires explicit user approval. Before approving a migration:

- Name the old owner and the new owner.
- List every consumer that depends on the old contract.
- Describe the cutover plan: parallel run, feature flag, hard switch, or staged rollout.
- Identify the rollback path.

Without approval and a cutover plan, ownership migration must be rejected during review.
