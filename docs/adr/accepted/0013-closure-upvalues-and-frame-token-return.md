# 13. Open/closed upvalues and frame-token non-local return

- Status: Accepted
- Date: 2026-07-11
- Related: `docs/spec/v0.2/blocks.md` §5; `docs/spec/v0.2/object-model.md` §4; [ADR-0006](0006-function-as-abstract-callable-root.md); [ADR-0009](0009-handle-arena-heap.md)

## Context

Blocks are the keystone construct — a block, a lambda, a method body, and a getter
body share one closure representation ([Blocks](../../spec/v0.2/blocks.md), [ADR-0006](0006-function-as-abstract-callable-root.md)).
Two behaviors in the spec constrain how closures capture and return:

- **Escaping blocks.** A block may outlive the frame in which it was created
  ([Blocks §5](../../spec/v0.2/blocks.md)); captured variables must remain live and, where
  mutated, must stay **shared** between the block and its home scope.
- **Non-local return.** `return` inside a block unwinds to its *home method* frame,
  not the block, and returns from that method. A block invoked after its home frame
  is gone must fail cleanly, not corrupt memory — the spec calls for `DeadFrameError`
  ([Blocks §5](../../spec/v0.2/blocks.md), [Object Model §4](../../spec/v0.2/object-model.md)).

## Decision

Capture uses **Lua-style open/closed upvalues**; non-local return uses a **frame
token**:

- An upvalue starts **open**, referencing the variable's slot on the stack. While
  open, the block and the enclosing scope see the *same* cell, so mutation is
  shared. When the scope exits, the upvalue is **closed** — its value is copied off
  the stack into the upvalue itself — so the block keeps working after its frame
  pops. This is the mechanism that makes escaping blocks with shared mutation work.
- Each block carries a **frame token**: the home frame pointer plus a **generation
  counter**. On non-local `return`, the token is compared against the live frame; a
  generation mismatch means the home frame is gone, and the VM raises
  **`DeadFrameError`** — a cheap integer compare that turns a memory-safety hazard
  into a clean runtime error.
- One `ClosureObject` is shared by `Block` and `Method` ([ADR-0006](0006-function-as-abstract-callable-root.md));
  upvalue cells live in the heap ([ADR-0009](0009-handle-arena-heap.md)) so a closed
  upvalue outliving its frame has a well-defined owner.

## Consequences

- Escaping blocks and shared mutation of captured variables both work, matching
  [Blocks §5](../../spec/v0.2/blocks.md); an inner block that mutates a captured `var` is
  seen by the outer scope while the frame is live.
- Non-local return is safe by construction: the generation check converts "return to
  a dead frame" from undefined behavior into `DeadFrameError`. (The unbraced arrow
  form is expression-only and cannot carry `return` ([Blocks §2](../../spec/v0.2/blocks.md)),
  which keeps the safe cases the common cases.)
- One closure representation means `Fiber`/`Future` ([Concurrency](../../spec/v0.2/concurrency.md))
  take any `Function` as their unit of work without caring block-vs-method.
- Open→closed promotion must interact correctly with the heap ownership model
  ([ADR-0009](0009-handle-arena-heap.md)): the upvalue cell's lifetime is the
  closure's, not the frame's.
- The frame token also unifies with `throw` and fiber `abort` as one stack-unwinding
  primitive ([ADR-0008](0008-layered-exceptions-and-result.md)).

## Alternatives considered

- **By-value snapshot capture** (copy captured variables into the closure at
  creation). Simpler and needs no open/closed machinery, but it **breaks shared
  mutation** — the block and its home scope would see divergent copies — and it
  fights non-local return, which needs the live home frame's identity. Rejected.
- **Raw frame pointer with no generation counter.** Cheaper token, but a reused
  frame slot would alias a stale pointer to a live frame, silently returning to the
  wrong method. The generation counter is what makes the dead-frame case detectable.
  Rejected.
