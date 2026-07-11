# Annotations — test strategy and diagnostics catalog

- Status: **Proposed** (experimental; not ratified)
- Date: 2026-07-11
- Depends on: [annotations-core.md](annotations-core.md), [annotations-legality-grammar.md](annotations-legality-grammar.md)
- Resolves: process gaps — no test plan, no diagnostics catalog (repo conventions)
- Related: docs/forge/test-corpus-plan.md, `Universe::verify_invariants`, ADR-0016 (multi-error diagnostics)

## Context

Every unit in this repo is gated by a golden `.ph` corpus + snapshot tests +
`verify_invariants()`. The annotation ADRs describe behavior but name no tests
and no diagnostics — they are not yet implementable to the project's standard.

## Decision

### Snapshot the desugared AST

The expander pass (annotations-core.md) is a `ClassDef -> ClassDef` transform, so
its primary test surface is **AST snapshots** (insta), mirroring
`phalcom-ast/tests/snapshots/`. For each attribute, a fixture pairs the annotated
source with the expected desugared members:

| Snapshot | Asserts |
|----------|---------|
| `expand__requires_prologue` | `@requires` prepends the `ifFalse`-guard statement |
| `expand__ensures_old_hoist` | `old(_x)` lifted to `__old_0` before body; occurrence rewritten |
| `expand__ensures_result_bind` | last expr bound to `__result`; postcondition appended; `__result` returned |
| `expand__invariant_wrap` | every public method wrapped entry+exit; statics untouched |
| `expand__construct_params` | fields → labeled params → assignments; super-chaining |
| `expand__get_set_pair` | `@get`/`@set` derive the accessor selectors |

### Golden `.ph` behavior corpus

Runtime `.ph` cases under the forge corpus, one green + one red per feature:

| Case | Expect |
|------|--------|
| `contracts_precondition_pass.ph` | body runs, returns normally |
| `contracts_precondition_fail.ph` | raises `PreconditionError`, message names the selector |
| `contracts_postcondition_old.ph` | `old(_balance)` reflects pre-state |
| `contracts_invariant_reentrancy.ph` | nested public send does **not** re-trip invariant (contract-semantics) |
| `contracts_release_stripped.ph` | in `release` mode, `@ensures` does not fire |
| `construct_subclass_super.ph` | `Dog.new(name:, breed:)` sets both slots |
| `annotation_unknown_error.ph` | `@typo` is a compile error, not ignored |

### Diagnostics catalog (miette)

Each failure is a named diagnostic carrying the offending span (D3):

| Code | Trigger | Span |
|------|---------|------|
| `attr.unknown` | unregistered attribute name | the `@name` |
| `attr.illegal_target` | attribute on an illegal member kind | the `@name` |
| `attr.dangling` | attribute with no following member | the `@name` |
| `attr.accessor_collision` | `@get`/`@set`/`@construct` selector clashes with hand-written | the member |
| `contract.impure_predicate` | assignment / known mutator in a predicate | the sub-expr |
| `contract.old_on_mutable` | `old(...)` on a mutable, non-`@data` operand | the `old(...)` |
| `construct.super_uninferrable` | `@construct` subclass of hand-written-constructor parent | the class name |

### `verify_invariants()` extension

After bootstrap, if any core class ships contracts, `verify_invariants()` gains a
cheap assertion that woven `__check_invariant` methods exist on exactly the
classes declaring `@invariant` and nowhere else — the same self-checking-tower
discipline (ADR-0002) applied to the annotation layer.

## Consequences

- The AST-snapshot surface means most annotation logic is testable **without
  running the VM**, keeping the feedback loop fast and matching how
  `phalcom-ast` is already tested.
- The diagnostics catalog is the contract between the parser/expander and the
  multi-error recovery machinery (ADR-0016) — every listed code must recover, not
  panic.

## What this precludes

Nothing — this draft is additive process scaffolding. It is the gate the other
annotation drafts must pass before promotion to `docs/adr/` + `docs/spec/`.
