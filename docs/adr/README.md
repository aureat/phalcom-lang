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
| [0005](0005-number-as-flat-f64.md) | Keep a single flat `Number` type backed by `f64` | Superseded in part by [0024](0024-numeric-surface-split-int-float-and-division.md) |
| [0006](0006-function-as-abstract-callable-root.md) | `Function` as the abstract root of the callable tower | Accepted |
| [0007](0007-option-as-abstract-with-some-none.md) | Represent absence as abstract `Option` + `Some`/`None` | Accepted |
| [0008](0008-layered-exceptions-and-result.md) | Layered exceptions + `Result`, with terminating semantics | Accepted |
| [0009](0009-handle-arena-heap.md) | Object graph lives in a handle/arena heap | Accepted |
| [0010](0010-tagged-value-enum.md) | `Value` is a tagged `enum` with a private `Nil` sentinel | Accepted |
| [0011](0011-static-instance-slot-layout.md) | Instances use a static per-class slot layout | Accepted |
| [0012](0012-selector-signature-encoding-and-dispatch.md) | Label-encoded selectors and inline-cache-ready dispatch | Accepted |
| [0013](0013-closure-upvalues-and-frame-token-return.md) | Open/closed upvalues and frame-token non-local return | Accepted |
| [0014](0014-let-and-var-bindings.md) | Variable bindings are `let` (immutable) and `var` (mutable) | Accepted |
| [0015](0015-object-default-tostring.md) | `Object` default `toString` is `"<ClassName>"` | Accepted |
| [0016](0016-hand-written-lexer-and-recursive-descent-parser.md) | Hand-written lexer and recursive-descent parser (replacing LALRPOP) | Accepted |
| [0017](0017-class-side-stored-static-fields.md) | Class-side stored static fields live on the metaclass instance | Accepted |
| [0018](0018-sacred-selector-inliner-and-override-guard.md) | Sacred-selector inliner with override-epoch deopt guard | Accepted |
| [0019](0019-freeze-vm-blessed-primitive-floor.md) | Freeze the VM-blessed primitive floor | Accepted |
| [0020](0020-kernel-list-native-array-protocol.md) | Kernel `List` is a native-array-backed protocol on the critical path | Accepted |
| [0021](0021-no-truthiness-enforcement.md) | No-truthiness enforcement: typed branch floor + literal-only compile check | Accepted |
| [0022](0022-string-interpolation-backslash-paren-sigil.md) | String interpolation uses the `\(expr)` sigil | Accepted |
| [0023](0023-amend-floor-admit-hash-and-kernel-reflection.md) | Amend the frozen floor to admit `hash` + kernel reflection | Accepted |
| [0024](0024-numeric-surface-split-int-float-and-division.md) | Split `Number` into exact `Int` (bignum) + `Float`; `/` true division, `~/` integer division | Accepted |
| [0025](0025-external-internal-parameter-names.md) | Separate external labels from internal parameter names | Accepted |
| [0026](0026-class-hierarchy-mutability.md) | Methods are open; superclass reparenting is sealed | Accepted |
| [0027](0027-modules-as-files-with-public-by-default-imports.md) | A module is a file; public-by-default exports; qualified/selective/aliased imports | Accepted |

> ADRs 0024–0027 were ratified by the user on 2026-07-12, resolving open-questions
> Q2 (numeric split + division), Q3 (parameter names), Q4 (hierarchy mutability), and
> Q8 (modules). See [`docs/spec/open-questions.md`](../spec/open-questions.md).

> ADRs 0019–0020 were ratified by the user on 2026-07-11, clearing the U-LIST-plan
> §0 gate: they derive from the experimental draft
> `docs/spec/experimental/bootstrapping-and-self-hosting.md` (D1, D2/DEC-A). 0019
> consolidates an existing boundary; 0020 resolves DEC-A.

> ADRs 0003–0008 were ratified in the object-model / language-design sessions, and
> ADRs 0009–0015 in the Phase-2 VM-architecture session (2026-07-11); all are now
> **Accepted** — the design baseline for implementation. Per the immutability
> convention, changing any of them requires a new superseding ADR.
