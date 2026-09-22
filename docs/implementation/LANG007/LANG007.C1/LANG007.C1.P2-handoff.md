---
id: LANG007.C1.P2
kind: implementation-plan-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: 33ee08f28988e8483259a0870f1bd247c0022547
implementation_revision: 33ee08f28988e8483259a0870f1bd247c0022547
working_tree: scoped P2 decision, production cleanup, specification convergence, and lifecycle records
---

# LANG007.C1.P2 — Handoff

## 1. Current state

`LANG007.C1.P2` is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

The decision registry, compiler source/presentation layer, and normative language specifications have converged on the ratified `intrinsic` keyword model with `@intrinsic` retired from source authority.

## 2. Canonical decision and specification chain

- **TDR-0081:** Tuple and Record are transparent structural value products.
- **TDR-0082:** Declaration modifiers and identity-based access domains.
- **TDR-0083:** Type-level structure has distinct domains; compile-time values are typed values.
- **TDR-0084:** Canonical type-level terms are stable kinded identities.
- **TDR-0085:** Type members are parameterized, domain-aware projections with symbolic residuals.
- **TDR-0086:** Tuple type subscript and `At<I>` are one compile-time member identity.
- **TDR-0087:** Structural `Map<F>` preserves shape and transforms payload types.
- **TDR-0088:** Native/intrinsic type operations use closed compiler normalization, not VM execution.
- **TDR-0089:** `intrinsic` is a first-class implementation-kind keyword and `@intrinsic` is retired.

## 3. Core architectural invariants for takeover

1. **`intrinsic` keyword:** `intrinsic` is a first-class declaration modifier mapped to `ImplementationKind::Intrinsic`.
2. **`@intrinsic` retired:** `@intrinsic` is no longer a builtin attribute and is never emitted in virtual source.
3. **`NativeIntrinsicId` preserved:** Internal native metadata (e.g. `BoolNot`) is distinct from source `ImplementationKind`.
4. **Subscript / `At` identity:** `T[I] ≡ T::At<I>` denote one Subscript type-member identity with `At` as the named presentation alias.
5. **Structural `Map<F>`:** Requires `F : Type -> Type`, uses ordinary type application `F<T>`, maps open-row tails symbolically, and preserves label order and keys.
6. **Zero product:** `Tuple([], []) ≡ Unit ≡ ()`; `Record({})` is a distinct static Record type.
7. **No VM execution:** Intrinsic type members normalize purely in the semantic/type-system layer with zero VM bytecode or dispatch.
8. **Bounded scope:** No general nominal/trait const generics, no GATs, no open type families, and no runtime pack/row objects.

## 4. Stable source and test takeover map

- **AST Token & Modifier:** `phalcom-ast/src/token.rs`, `phalcom-ast/src/parser.rs`, `phalcom-common/src/modifier.rs`.
- **Type Member Identity & Registry:** `phalcom-semantic/src/types/type_member.rs`.
- **Projection Normalizer:** `phalcom-semantic/src/types/projection.rs`.
- **Type Annotations & Validation:** `phalcom-semantic/src/types/annotation.rs`.
- **Presentation IR:** `phalcom-semantic/src/core_surface/presentation.rs`.
- **Universe Collections:** `phalcom-core/core/universe/src/collections/{tuple,record}.ph`.
- **Primary Specifications:** `docs/specs/{source-declarations,type-members,type-level-domains,structural-products,universe}.md`.
- **Focused Test Targets:**
  - `cargo test -p phalcom-ast --test integration intrinsic`
  - `cargo test -p phalcom-semantic --test semantic integration::native_conformance`
  - `cargo test -p phalcom-semantic --test semantic foundations::type_members`

## 5. Known residuals

- `BASELINE-001` and `BASELINE-002`: Two pre-existing trait dependency failures reproduced at predecessor baseline and deferred in the checkpoint.

## 6. Next plan objective (P3)

Execute `LANG007.C1.P3`:
- Validate complete architecture baseline.
- Establish comprehensive acceptance matrix across all C1 vertical layers.
- Verify checkpoint closure criteria.

## 7. First recommended commands for P3

```sh
git status --short
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test integration intrinsic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::native_conformance
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_members
```

## 8. Do-not-redesign list

Subsequent agents must not:
- Resurrect `@intrinsic` as a source attribute or alias.
- Collapse `Native` and `Intrinsic` implementation kinds.
- Delete or rename internal `NativeIntrinsicId` metadata.
- Separate `T[I]` and `T::At<I>` into two semantic members.
- Re-open anonymous structural types as inherent `impl` targets.
- Introduce VM bytecode or runtime dispatch for type-member evaluation.
- Promote general const generics or GATs without explicit new programs.
