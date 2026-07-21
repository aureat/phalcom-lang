# The IO protocol axes want stateless interface declarations; single inheritance gives them one slot

- Deferred: 2026-07-20
- Raised by: [PDR-0005](../pdr/0005-resources-are-disposable-handles-not-finalized.md) §3
- Blocks: nothing today. Becomes load-bearing the first time a generic IO algorithm exists.
- Related: [ADR-0048](../adr/accepted/0048-amend-iteration-bare-cursor-sentinel-and-iterable-root.md)
  (`Iterable` as a kernel root — the precedent 0069 §3 followed for one axis),
  [ADR-0011](../adr/accepted/0011-static-instance-slot-layout.md) (frozen slot offsets — the reason
  the stateless/stateful distinction below is the whole ballgame), U-INH (single inheritance),
  [PDR-0004](../pdr/0004-io-is-future-shaped-reactor-owned.md)

> **Scope note.** An earlier draft of this file was titled "…need mixins" and framed the deferral as
> a traits/mixins feature. That was a mis-transcription of the ruling. What is deferred is
> **stateless interface-style declarations** — checkable identity, no shared implementation, no
> contributed fields. Mixins appear below only as the heavier alternative that was *not* asked for.

## The problem

`File` is readable, writable, seekable, and closeable. `TcpStream` is readable, writable, and
closeable but **not** seekable. `BytesReader` is readable and seekable but neither writable nor
closeable. Four independent axes; a class has one `extends` slot.

PDR-0005 §3 resolved this for now by reifying **only** closeability (`Resource` as a kernel
root class) and leaving Reader / Writer / Seekable as informal respond-to protocols. That is right
for the current surface: the only mechanisms that need to *ask the type* are leak reporting and
generic cleanup, and both ask about closeability. Nothing needs to ask "is this a Reader?" — it
sends `read(_)`.

## When this stops being adequate

The informal answer holds until a function is generic over readers and must *verify* rather than
assume:

- `copy(from:, to:)` — accepts any Reader and any Writer, and should give a decent error when handed
  something else. Duck-typing gives `doesNotUnderstand` deep inside the loop, after a partial write.
- `BufferedWriter.new(_)` — wraps an arbitrary Writer. Same failure shape.
- `on(Reader) { … }` — dispatch or typed catch on the axis. Impossible without a reified type.
- Any capability check that must happen *before* side effects begin.

The tell is a **partial-effect failure**: duck-typing reports the type error after side effects have
started, and for IO that means a half-written file.

## The deferred design: stateless interface declarations

A named, checkable set of required selectors. Contributes **no fields and no implementation**. A
class declares it satisfies one or more; `isA(_)` answers true; method lookup is untouched.

Why this shape specifically, and why it is much cheaper than mixins:

- **It does not touch layout.** [ADR-0011](../adr/accepted/0011-static-instance-slot-layout.md)
  freezes instance slot offsets. A mixin contributing *state* changes what a class's layout is and
  forces that machinery open — the single most expensive part of a traits feature. A stateless
  declaration adds nothing to any instance.
- **It does not touch dispatch.** No new method sources, so
  [ADR-0012](../adr/accepted/0012-selector-signature-encoding-and-dispatch.md)'s lookup and the
  inline caches are unaffected. A real mixin introduces a second contributor to the method table and
  a linearization question with it.
- **It is mostly a compile-time check plus an `isA` bit.** Runtime cost is a set-membership test;
  compile-time cost is verifying the declared selectors exist.

Open sub-questions, none answered here: whether satisfaction is *declared* or *inferred
structurally*; whether an unsatisfied declaration is a compile error or a runtime one; whether a
declaration can require a full selector *signature* (arity + labels, ADR-0012's encoded form) or
only a bare name; and how this interacts with [PDR-0001](../pdr/0001-classes-are-closed.md)'s
closed classes, since a declaration is a second thing that can be said about a class after it is
defined.

## Alternatives, recorded not chosen

1. **Full mixins / traits.** Composable units carrying implementation and possibly state. Strictly
   more powerful and strictly more expensive — touches ADR-0011 layout and ADR-0012 dispatch as
   above. **Not what was asked for**; recorded so the scope difference stays visible.
2. **Structural checks only.** `respondsTo(_)` at the boundary of each generic function. No language
   change; makes the error early and decent. Possibly sufficient, and it is the honest baseline any
   interface feature must beat.
3. **Do nothing.** Current state. Fine until a generic IO algorithm exists.

Option 2 is the baseline to measure against: if `respondsTo(_)` guards at function entry give
adequate errors, the language feature may never be worth its weight.

## What must not happen

Do not grow this by hierarchy. Adding a kernel root class per axis (`Reader`, `Writer`, `Seekable`
alongside `Resource`) cannot work under single inheritance — `File` extends exactly one — and any
attempt produces a linearized chain like `Resource < Reader < Writer` that encodes an ordering
nobody meant and that `TcpStream` (not seekable) and `BytesReader` (not closeable) both falsify.
That is the shape someone reaches for under deadline. If the axes get reified, it is by a ruled
language feature.
