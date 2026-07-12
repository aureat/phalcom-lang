# Typed initialization — fields, `var`, and definite assignment (proposed)

- Status: **Proposed** (experimental; not ratified) · **soundness teeth**
- Axis: typing ⊗ absence ⊗ bindings
- Resolves: [typing.md](typing.md) Tier-1 gap #1 (typed init vs "unassigned reads as `None`")
- Related: [ADR-0007](../../../adr/0007-option-as-abstract-with-some-none.md) (Option/None), [ADR-0014](../../../adr/0014-let-and-var-bindings.md) (`let`/`var`), [classes.md](../classes.md) (fields, read-before-write), [values-and-absence.md §3](../values-and-absence.md)

## Problem

[typing.md](typing.md) never types fields or local bindings, and that silence
collides with three committed behaviors:

1. **`var x` with no initializer reads as `None`** ([ADR-0014](../../../adr/0014-let-and-var-bindings.md)).
2. **A declared-but-unassigned field reads as `None`** ([values §3](../values-and-absence.md)).
3. **`nil` may never be user-visible; absence is always `Option`** (Invariant 4).

A typed `var x: Int` (no init) would then have static type `Int` but runtime value
`None` — the annotation *lies*, which violates the erasure invariant **E**
([typing.md §5.2](typing.md)): stripping the annotation must not change behavior, but
here the annotation is already inconsistent with behavior. The same clash hits a
non-`Option` field `_age: Int` left unassigned.

The doc also conflates two different analyses under "no flow analysis"
([typing.md §5.12](typing.md)) — see §Clarification.

## Decision

**Absence in the type system is *always* `Option`. "Unassigned ⇒ `None`" is
well-typed only when the declared type admits `None`.** Three consequences:

### Fields — lean on read-before-write

`classes.md` already makes **read-before-write a compile error**, i.e. a
definite-assignment analysis already exists. Typing reuses it as a *type*
obligation:

- A field typed `T` where `None ⋢ T` (non-`Option`) must be **definitely assigned on
  every `construct` path before its first read.** This is exactly the existing rule;
  typing adds no new machinery, only a reason.
- A field legitimately absent-able is typed `Option<T>`. Then unassigned reads as
  `None : Option<Nothing> <: Option<T>` — **consistent**, because `Nothing` is the
  bottom type and `Option` is covariant ([typing.md §5.4.1](typing.md)).

So the "unassigned field is `None`" behavior is *sound precisely for `Option`-typed
fields* and *rejected for others by a rule that already exists*.

### Locals — a typed `var` requires an initializer

`var x: T` with a type annotation **must** have an initializer. The "no init ⇒
`None`" convenience is reserved for *un-annotated* `var x` (today's behavior,
untouched). Rationale:

- Preserves **E**: no annotation ⇒ today's semantics; annotation ⇒ strictly tighter.
- Avoids the silent `T`-vs-`None` mismatch entirely rather than papering it with an
  implicit `Option<T>` widening the programmer didn't write.
- `let x` with no initializer is already rejected ([ADR-0014](../../../adr/0014-let-and-var-bindings.md)) — this is the symmetric rule for `var`.

An author who wants absence writes it: `var x: Option<Int> = None`.

### Clarification — definite assignment ≠ flow typing

[typing.md §5.12](typing.md)'s "no flow analysis" means **no type narrowing** (no
`if (x is T)` refining `x`'s type). **Definite assignment** — a boolean
assigned-before-use analysis, already required by read-before-write — is a *separate*
thing and **is** permitted. The two must not be conflated: the type system rejects
unions-without-narrowing, but still tracks whether a slot has been written.

## Edge cases

| Case | Resolution |
|------|-----------|
| `_age: Int` assigned in one `construct` but not another | Compile error on the path that omits it (existing read-before-write rule). |
| `_age: Option<Int>`, never assigned | Reads `None`; sound (§Fields). |
| `var x: Int` no init | Compile error: "a typed `var` must be initialized; use `Option<Int>` for absence." |
| `var x` no init (un-annotated) | `None`, today's behavior; unchanged. |
| `self` escaping `construct` before all fields assigned | **Known limitation** — a partially-initialized `self` can defeat field-type soundness (Swift's two-phase-init problem). Deferred; flagged in [typing.md §11](typing.md) T-follow-up. |

## Precludes

- A non-`Option` field silently defaulting to `None` — that is `nil` under a new
  name (violates Invariant 4). Rejected.
- Implicitly widening a typed `var x: Int` (no init) to `Option<Int>` — hides the
  author's intent and spreads `Option` where it wasn't written. Rejected in favor of
  an explicit-initialization error.
- Couples the type checker to the definite-assignment pass; the two ship together.
