# 14. Variable bindings are `let` (immutable) and `var` (mutable)

- Status: **Superseded by [ADR-0064](0064-let-const-bindings-and-field-mutability.md)** (2026-07-15)

> **Superseded on spelling, not semantics.** ADR-0064 renames `var` → `let` and
> `let` → `const`; every rule below carries over unchanged, including "an uninitialized
> mutable binding reads `None`" and "an immutable binding requires an initializer".
> If you are here chasing a citation about *behavior*, this document is still correct —
> substitute the new keywords. ADR-0064 additionally rules field mutability, which this
> ADR never covered (and which was silently unenforced: a `let` field was writable from
> any method).
- Date: 2026-07-11
- Related: [open question Q1](../../spec/v0.2/open-questions.md); `docs/spec/v0.2/values-and-absence.md` §3; [ADR-0007](0007-option-as-abstract-with-some-none.md)

## Context

[Open question Q1](../../spec/v0.2/open-questions.md) left the binding forms unresolved: the
lexer today has only `let`, with no way to declare a rebindable variable and no
stated rule for a binding introduced without an initializer. The absence-as-`Option`
model ([Values & Absence](../../spec/v0.2/values-and-absence.md), [ADR-0007](0007-option-as-abstract-with-some-none.md))
already fixes what "no value yet" must mean — there is no surface `nil` to fall back
on — so the initializer-less case needs an answer consistent with it.

## Decision

Two binding forms, resolving Q1:

- **`let`** introduces an **immutable** binding — it cannot be reassigned after
  initialization.
- **`var`** introduces a **mutable** binding — it can be reassigned.
- **`var x` without an initializer reads as absence: `None`** ([ADR-0007](0007-option-as-abstract-with-some-none.md)),
  consistent with a declared-but-unassigned field ([Classes §2](../../spec/v0.2/classes.md)).
  The private `Nil` sentinel ([ADR-0010](0010-tagged-value-enum.md)) backs the
  uninitialized slot internally and is surfaced as `None`, never leaked to user code
  (Invariant 4). `let x` with no initializer is not meaningful (an immutable binding
  that can never be given a value) and is rejected.

## Consequences

- Mutability is explicit at the declaration site: `let` communicates "this will not
  change," `var` communicates "this may," matching the JavaScript/Swift intuition
  most Phalcom users arrive with.
- The initializer-less `var x` has one coherent meaning — `None` — with no surface
  `nil` and no separate "uninitialized" state, keeping the no-`nil` invariant intact.
- The lexer/grammar gains the `var` keyword alongside the existing `let`.
- Uninitialized `var` reading `None` unifies with unassigned fields reading `None`,
  so one absence rule covers both bindings and fields.

## Alternatives considered

- **`let`-only** (the current state). No way to express mutation without a
  workaround, and no answer for the initializer-less case. Rejected — the language
  needs a mutable binding form.
- **`var x` without initializer as a compile error.** Defensible, but it denies the
  common "declare now, assign later" pattern that the `None` reading supports
  cleanly; the absence model already gives an unambiguous value for the empty slot.
  Rejected in favor of reading `None`.
- **A surface uninitialized/`nil` value for the empty case.** Reintroduces the null
  coercion the object model exists to remove ([Values & Absence §2](../../spec/v0.2/values-and-absence.md)).
  Rejected.
