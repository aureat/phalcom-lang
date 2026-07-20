# The IO protocol axes want mixins; single inheritance gives them one slot

- Deferred: 2026-07-20
- Raised by: [decision 0069](../decisions/0069-resources-are-disposable-handles-not-finalized.md) §3
- Blocks: nothing today. Becomes load-bearing if generic IO algorithms are ever wanted.
- Related: [ADR-0048](../adr/accepted/0048-amend-iteration-bare-cursor-sentinel-and-iterable-root.md)
  (`Iterable` as a kernel root — the precedent 0069 §3 followed for one axis),
  U-INH (single inheritance), [decision 0068](../decisions/0068-io-is-future-shaped-reactor-owned.md)
  (the surface that surfaces this)

## The problem

`File` is readable, writable, seekable, and closeable. `TcpStream` is readable, writable, and
closeable but **not** seekable. `BytesReader` is readable and seekable but neither writable nor
closeable. These are four independent axes, and Phalcom gives a class exactly one `extends` slot.

Decision 0069 §3 resolved this for now by reifying **only** closeability (`Resource` as a kernel
root class) and leaving Reader / Writer / Seekable as informal respond-to protocols. That is the
right call for the current surface: the only mechanisms that need to *ask the type* are leak
reporting and generic cleanup, and both ask about closeability. Nothing needs to ask "is this a
Reader?" — it just sends `read(_)`.

## When this stops being adequate

The informal-protocol answer holds until someone wants a function that is generic over readers and
must *verify* rather than assume. Concretely:

- `copy(from:, to:)` — wants to accept any Reader and any Writer, and give a decent error when
  handed something else. Duck-typing gives a `doesNotUnderstand` deep inside the loop, after a
  partial write.
- `BufferedReader.new(_)` — wrapping an arbitrary Reader. Same failure shape.
- `on(Reader) { … }` — typed catch or dispatch on the axis. Impossible without a class.
- Anything that wants `isA(Writer)` for a capability check before starting work.

The tell is a partial-effect failure: duck-typing reports the type error *after* side effects have
begun, and for IO that means a half-written file.

## The options, unranked

1. **Mixins / traits.** A composable unit of behavior a class can include several of. Reifies all
   four axes properly. The largest change: it touches method lookup (ADR-0012's selector dispatch),
   the metaclass tower, and the layout rules (ADR-0011's frozen slot offsets), since a mixin
   contributing state changes what a class's layout is.
2. **Interface-style declarations with no state.** Weaker than mixins — a name plus a required
   selector set, checkable by `isA`, contributing no fields. Sidesteps the layout question
   entirely, which is most of the cost of (1).
3. **Structural checks.** `respondsTo(_)` at the boundary of generic functions, no new types.
   Cheapest; makes the error early and decent without a language feature. Possibly enough.
4. **Do nothing.** The current state. Fine until a generic IO algorithm exists.

Option 2 is worth examining before 1 — most of what the IO surface wants from mixins is *checkable
identity*, not *shared implementation*, and those separate cleanly.

## What must not happen

Do not grow this by accident. Adding a second kernel root class per axis (`Reader`, `Writer`,
`Seekable` alongside `Resource`) does not work under single inheritance — `File` can extend exactly
one of them — and any attempt will produce a linearized chain like `Resource < Reader < Writer`
that encodes an ordering nobody meant and that `TcpStream` (not seekable) and `BytesReader` (not
closeable) both falsify. If the axes get reified, they get reified by a ruled language feature, not
by hierarchy tricks.
