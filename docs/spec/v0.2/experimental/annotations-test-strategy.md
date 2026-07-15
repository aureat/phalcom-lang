# Annotations — test strategy and diagnostics catalog

- Status: **Proposed** (experimental; not ratified)
- Date: 2026-07-11
- Depends on: [annotations-core.md](annotations-core.md), [annotations-legality-grammar.md](annotations-legality-grammar.md), [decorators.md](../decorators/README.md) (five-tier model — adds Install/Dispatch/Runtime failure modes below), [attribute-classes.md](../decorators/on.md) (attribute-as-class — adds tier-inference and `@AttributeUsage` failure modes below)
- Resolves: process gaps — no test plan, no diagnostics catalog (repo conventions)
- Related: docs/forge/test-corpus-plan.md, `Universe::verify_invariants`, ADR-0016 (multi-error diagnostics), [ADR-0052](../../../adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md) (invariant re-entrancy fix — adds the `contracts_invariant_cross_receiver.ph` case below)

> **Catalog currency note.** This doc predates `decorators.md` and
> `attribute-classes.md` by one day; the diagnostics catalog below has been
> extended to cover their tier-inference and `Attribute`-class surface, but
> any future addition to the five-tier model must update this table in the
> same change — it is the single diagnostics source of truth for `@`, not
> just the Draft-0.1 core mechanism.

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
| `contracts_unchecked_metadata_stripped.ph` | in `unchecked` mode, reflectable predicate metadata is absent (not just guards) |
| `contracts_invariant_cross_receiver.ph` | object `A`'s public method calling object `B`'s public method still checks `B`'s own `@invariant` (ADR-0052 regression case) |
| `contracts_invariant_survives_throw.ph` | a thrown error inside an `@invariant`-checked call does not leave the re-entrancy guard permanently inflated (ADR-0052) |
| `construct_subclass_super.ph` | `Dog.new(name:, breed:)` sets both slots |
| `construct_subclass_hand_written_parent.ph` | `@construct` subclass of a hand-written single-constructor parent infers the super-signature correctly (F fix: no longer requires the parent to also use `@construct`) |
| `construct_subclass_ambiguous_super.ph` | `@construct` subclass of a superclass with two overloaded `new` selectors is a `construct.super_ambiguous` compile error |
| `annotation_unknown_error.ph` | `@typo` is a compile error, not ignored |
| `decorator_computed_no_leak.ph` | a receiver decorated with `@computed` is collectible once otherwise unreferenced (ADR-0052 — Layout, not Install, storage) |
| `decorator_memoize_per_receiver.ph` | `@memoize` on a stateful method returns distinct results for two different receivers called with equal args |
| `attribute_tier_ambiguous_error.ph` | a user `Attribute` subclass implementing two hook selectors without `@tier` is a compile error |
| `attribute_usage_violation_error.ph` | an attribute applied outside its `@AttributeUsage` target set is a compile error |
| `attribute_compile_tier_forbidden.ph` | a user `Attribute` subclass whose only hook is `expand(_)`/`finalizeLayout(_)` is a compile error (Compile/Layout stay builtin-owned) |

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
| `construct.super_ambiguous` | `@construct` subclass whose superclass has more than one constructor selector (no single signature to infer) | the class name |
| `attr.tier_ambiguous` | `Attribute` subclass implements more than one hook selector (`wrap`/`onMiss`/`aroundSend`/…) without an explicit `@tier` | the class name |
| `attr.compile_tier_forbidden` | user `Attribute` subclass's only hook is `expand(_)`/`finalizeLayout(_)` — Compile/Layout are builtin-owned ([attribute-classes.md](../decorators/on.md) "What this precludes") | the class name |
| `attr.usage_violation` | attribute applied to a target outside its declared `@AttributeUsage(...)` set | the `@name` at the use site |
| `attr.receiver_keyed_install_state` | (lint, not a hard error — golden-test only, per [ADR-0052](../../../adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)) an Install-tier `wrap(_)` closes over a collection keyed by the receiver it decorates | the `wrap(_)` body |

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
