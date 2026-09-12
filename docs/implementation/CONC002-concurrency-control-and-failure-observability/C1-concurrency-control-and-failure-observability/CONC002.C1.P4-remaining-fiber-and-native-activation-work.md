# CONC002.C1.P4 — Remaining Fiber and native activation work

Start from the uncommitted P3 changes; preserve unrelated user work. P3 implements
invariant Future<T>, typed async/map/then/catch/recoverWith/flatten, Unit identity,
scheduler entry validation, scheduled-yield refusal and root-result retention.
Do not redo those mechanisms. No commits or broad cleanups are authorized here.

## 1. Fiber generics and explicit terminal access

Owner: `primitive/fiber.rs`, `concurrency/fiber.ph`, canonical callable signatures.

Use `Fiber<I, R>` for resume input and terminal success, without claiming a typed
yield channel until the VM can enforce one. `call`/`try` still mix yielded values,
terminal values and captured failure; keeping their result Dynamic is more honest
than declaring R. Add a separate `result -> Option<R>` returning Some only for
Done, plus the existing `error -> Option<Error>` for Failed.

A zero-argument constructor can infer `new<R>(() -> R) -> Fiber<Unit, R>`.
A separately named one-argument constructor can infer
`withInput<I,R>((I) -> R) -> Fiber<I,R>`, sharing the existing allocation primitive.
Do not annotate arbitrary Function as returning R: that does not establish the
entry's result type. Preserve rest-argument support through a separately specified
pack-input constructor or defer that surface explicitly. Add native registrations
and source declarations together. Keep `current` and heterogeneous internal queue
handles erased until an existential capability can express them soundly.

Test inferred terminal R, incompatible resume inputs, first-entry arity rejection,
yielded Error versus failed Fiber, and None/Unit results. Changing Fiber to generic
also requires auditing every bare Fiber annotation in the canonical source and
native signature products; never add unsaturated types to the bootstrap baseline.

## 2. Remove native callback re-entry with owned activations

Owners: `primitive/block.rs`, `vm/send.rs`, `vm/dispatch.rs`, `frame.rs`,
`heap/fiber.rs`, `heap/trace.rs`, `vm/gc.rs`.

Migrate on/ensure together. Introduce Fiber-owned resumable handler/cleanup records
with saved frame/stack depths, callable handles, phase and a pending outcome
(normal value, Raise, or non-local return with target frame token). Trace every
held Value, Error and callable. Avoid putting suspended state in Rust locals or
VM-global maps. Keep leaf primitives synchronous.

The call gateway must push the protected activation and return to the ordinary
dispatch loop. On completion/unwind, run cleanup as an ordinary activation; it may
park. Cleanup failure or non-local return supersedes the pending outcome. Close
open upvalues before each truncation. Refactor ReturnNonLocal's eager bulk unwind
so it visits cleanup records instead of deleting their target frames first.
Preserve frame generations and native visibility authority across these steps.

Only after these records work should the Fiber Call failure cascade become an
exception injected at the caller's call site. A child failure must then traverse
parent catch/ensure instead of terminally discarding parents. Do not remove
`native_reentry_depth` guards before this migration is complete.

Acceptance: success/error/non-local return through nested ensure; cleanup itself
awaiting; cleanup overriding an earlier error; call-linked child failure caught
by parent; escaped upvalues and pending errors surviving forced GC; exact logical
traceback frames; no host recursion proportional to nested user callbacks.

## 3. Generator await and cancellation ownership

The current manual-await rejection is intentional protection. Supporting it needs
a durable consumer relation independent of the scheduler resumer. Park the
consumer's request for the next user stop; readiness resumes the generator, and
its next user yield wakes/delivers to that consumer, never to the executor driver.
Retain active-ancestor exclusion. Do not implement this by treating park as None
or recursively pumping the scheduler inside every generator.

Before cancellation can revoke queued work, replace raw ready handles with an
activation generation; otherwise a stale entry can claim a later queue episode.
Cancellation consumes the current wait capability and runs normal cleanup before
publishing a terminal cancellation outcome. Do not reuse abort or Error-as-data.

## 4. Additional concrete findings and verification boundaries

- A raw generic `Future` used in a new `result.is(Future)` guard caused a semantic
  panic: `types/store.rs` union member must be a proper type. The guard was removed;
  chaining now awaits the callback result *inside the observed action*, so invalid
  Dynamic results reject normally. Reproduce the generic type-form kind bug in an
  isolated semantic test before fixing it; do not weaken the proper-type assertion.
- `compiler/lib/expr.rs` still lowers non-reference Expr::TypeForm to Nil. Applied
  class expressions such as `Box<Int>.new()` need canonical executable type-form
  lowering. Do not erase generic reflection identity as a shortcut. Typed variable
  contexts and inferred constructors used in P3 do not require that expression.
- General receiver-dependent Self proof remains incomplete. P3 settlement methods
  accurately return Future<T>; it did not repair the general Self checker.
- Constructor and method type inference must remain in phalcom-semantic. The P3
  source-backed tests check Future.value, contextual pending construction, async,
  map, then, await, nested Future payloads, Unit and incompatible operations.
- The full semantic suite passed before the generic Future extension: 1141 passed,
  42 existing ignores. Final focused results belong in P3; this is not workspace
  or release certification. Run Cargo checks serially and only broaden on evidence.
