# Annotation showcase (design-only, **not** a runnable benchmark)

This directory exists to show the **full Phalcom annotation surface** in one
place, documented with [Phaldoc](../../docs/spec/experimental/doc-comments-phaldoc.md).
It is deliberately **not** under [`benchmarks/math/`](../math/), because
[`run.sh`](../math/run.sh) globs every `*.ph` in that directory and would try to
execute these files — which cannot lex today.

## Why it doesn't run

Two independent things are unbuilt on the current tree:

1. **Code-level `@`-attributes** (`@requires`, `@construct`, `@data`, …) need
   `Token::At` in the lexer and the desugar pass
   ([annotations-core.md](../../docs/spec/experimental/annotations-core.md)) — all
   still *Proposed*. A `@requires(x > 0)` line lexes as `/` `@` … garbage today.
2. **The Phaldoc doc generator** (`phalcom doc`) does not exist yet. The `///` /
   `//!` comments *are* inert (they are `//`-prefixed trivia — see
   [`lexer.rs:88`](../../phalcom-ast/src/lexer.rs)), but nothing harvests them.

So [`showcase.ph`](showcase.ph) is a **visual specimen**: it is what idiomatic,
fully-annotated Phalcom is intended to look like once both land. Do not add it to
CI or `run.sh` until the `@` lexer is in.

## The one rule it demonstrates

Phaldoc §8.1 — **prose and attributes never restate each other**:

- `///` prose says *what and why* (intent a machine can't infer).
- `@…` attributes say the *checkable facts* (which inputs are legal, what's
  guaranteed, what's generated).

`deposit`'s `///` never says "amount must be positive" — that fact lives only in
`@requires(amount > 0)`, and the doc tool harvests it (Eiffel's contract view).

## What `phalcom doc` would render for `deposit(_)`

Harvesting the two layers per Phaldoc §8.2 produces:

```
BankAccount ▸ deposit(_)
  Add funds.

  Requires   amount > 0                              [checked: debug+release]
  Ensures    _balance == old(_balance) + amount      [checked: debug]
  Invariant  _balance >= 0                            (class, all public methods)
  Raises     PreconditionError   — precondition violated   (derived)
             PostconditionError  — postcondition violated  (derived)
  Example
             let a = BankAccount.opened(100)
             a.deposit(50)          // ok
             a.deposit(0 - 1)       // raises PreconditionError
```

Every line under `Requires`/`Ensures`/`Invariant`/`Raises` is **harvested**, not
authored — the author wrote only the summary and the `@example`. The
`[checked: …]` badges come from the compile-mode table
([annotations-contract-semantics.md](../../docs/spec/experimental/annotations-contract-semantics.md)):
`@ensures` is stripped in `release`, so the doc says so; the *specification* is
shown regardless because contract metadata is retained in every mode
(D-contract-1).

## Files

| File | Shows |
|---|---|
| [`showcase.ph`](showcase.ph) | `@requires`/`@ensures`/`@invariant` (DbC), `@construct` + `@get`/`@set` (layout), `@data`/`@sealed`/`@variant` (ADTs + exhaustive `match`), `@observable`/`@computed` (reactive) — each wrapped in Phaldoc `///`/`//!` |
