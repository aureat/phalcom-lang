# Architecture Decision Records

This directory holds **Architecture Decision Records (ADRs)** — short documents
that capture a single significant design decision, the context that forced it,
and its consequences. They are the durable "why" behind the code: the parts you
can't recover by reading `class.rs` or querying the graphify graph.

## Format

We use [Michael Nygard's format](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions.html):
**Context → Decision → Consequences**, plus a **Status**. One decision per file.

Statuses: `Proposed`, `Accepted`, `Deprecated`, `Superseded by ADR-NNNN`.

## Conventions

- Files are named `NNNN-kebab-case-title.md`, numbered sequentially.
- ADRs are immutable once `Accepted`. To change a decision, write a **new** ADR
  that supersedes the old one, and update the old one's status to point at it.
- Keep them short (a page or two). Link to `docs/object-model.md` for full detail.

## Index

| ADR | Title | Status |
| --- | ----- | ------ |
| [0001](0001-record-architecture-decisions.md) | Record architecture decisions | Accepted |
| [0002](0002-metaclass-tower-parallel-rule.md) | Metaclass tower follows the parallel rule | Accepted |
| [0003](0003-introduce-behavior-kernel-class.md) | Introduce `Behavior` as a shared kernel class | Accepted |
| [0004](0004-boolean-as-abstract-bool-with-true-false.md) | Represent booleans as abstract `Bool` + `True`/`False` | Accepted |
| [0005](0005-number-as-flat-f64.md) | Keep a single flat `Number` type backed by `f64` | Accepted |
| [0006](0006-function-as-abstract-callable-root.md) | `Function` as the abstract root of the callable tower | Accepted |
| [0007](0007-option-as-abstract-with-some-none.md) | Represent absence as abstract `Option` + `Some`/`None` | Accepted |

> ADRs 0003–0007 were ratified in the object-model design session and are now
> **Accepted** — the design baseline for implementation. Per the immutability
> convention, changing any of them requires a new superseding ADR.
