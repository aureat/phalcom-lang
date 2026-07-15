# 6. `Function` as the abstract root of the callable tower

- Status: Accepted
- Date: 2026-07-11
- Related: [ADR-0003](0003-introduce-behavior-kernel-class.md); `docs/spec/v0.2/functions.md`, `docs/spec/v0.2/blocks.md`

## Context

The [Blocks](../../spec/v0.2/blocks.md) spec (§7) stated "a method *is* a `Block`." That
conflates two things that share a representation but differ in protocol: a `Block`
is an anonymous lexical closure with a non-local-return home frame and no receiver;
a `Method` is bound to a class under a selector and receives `self`.

The implementation already reflects the split: `ClosureObject`
(`closure.rs`) is the shared closure, but a `Method` wraps it inside a
`MethodObject` with a `Signature` and a holder (`method.rs`), while a first-class
block would wrap the same closure with a home-frame token. Neither is a subtype of
the other. Making `Method` a subclass of `Block` would force methods to carry a
meaningless home frame, and blocks to carry a meaningless selector.

## Decision

Introduce `Function` as an **abstract** kernel class ([Object Model](../../spec/v0.2/object-model.md)):

- `Function` owns the universal call protocol: `call`, `call(_,…)`, `callWith(_)`,
  `arity`, `name`. Function-application sugar `f(...)` desugars to `call(_,…)`.
- `Block` and `Method` both inherit from `Function` as **siblings**.
- `Function` inherits from `Object`. No value has `Function` as its direct class.

`Method.bind(_)` closes a method over a receiver and returns a `Function` (reusing
the `Block` machinery) — this is the precise, non-hand-wavy meaning of the old
"a method bound to a class."

## Consequences

- One call protocol, one closure representation (`ClosureObject`), two owners.
  `Fiber` and `Future` ([Fibers & Futures](../../spec/v0.2/concurrency.md)) take a
  `Function` as their unit of work without caring whether it is a block or a bound
  method.
- [Blocks §7](../../spec/v0.2/blocks.md) is amended: "sibling under `Function`," not "is a
  `Block`." Recorded inline in that file.
- Requires first-class blocks to land (a `Value::Block` arm plus `Closure` /
  `GetUpvalue` / `SetUpvalue` opcodes), which the current tree lacks — tracked in
  [Implementation Status](../../spec/v0.2/implementation-status.md).
- Slightly deeper callable chain (`Method` → `Function` → `Object`); negligible
  lookup cost, and it removes the special-casing that "method is a block" would
  otherwise smuggle in.

## Status note

Open question pending a go/no-go decision. Recommended to accept: it is the
minimal abstraction that lets the concurrency layer and reflection treat "a thing
you can call" uniformly, and it matches the representation the VM already has.
