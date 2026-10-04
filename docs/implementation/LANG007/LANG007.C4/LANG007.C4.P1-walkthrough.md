---
id: LANG007.C4.P1-walkthrough
kind: walkthrough
plan: LANG007.C4.P1
checkpoint: LANG007.C4
status: PARTIAL
completion: PARTIAL
verification: FOCUSED_TESTED
created: 2026-10-04
---

# LANG007.C4.P1 — Walkthrough

## 1. Plan identity and final state

**Plan**: LANG007.C4.P1 — Binding, Mutability, Field Lifecycle, and Corpus Cutover  
**Status**: PARTIAL (resumed after the earlier T6 pause; latest pause is after the T8a compiler-consumer slice)  
**Completion**: T0–T6 COMPLETE; T7 implementation present; T8a compiler consumer implemented; T8 lifecycle open; T9 tooling implemented; closure gates pending  
**Verification**: FOCUSED_TESTED for T0–T6, the T8a consumer test, and listed T9 checks; T7 gates, T8 lifecycle, G1, T10, and G2 remain UNVERIFIED

---

## 2. Objective and outcome

P1 was the foundational semantic cutover for `LANG007.C4`. The plan established:

1. Runtime immutable `let` and runtime mutable `let mut` as distinct syntactic forms
2. `const` as a static-phase declaration (not a runtime immutable binding)
3. True uninitialized mutable binding state via `ValueTag::UninitializedBinding`
4. Path-sensitive definite-initialization proof with runtime fallback via `GetLocalChecked`
5. Corpus-wide migration (719 files) from legacy `let` → `let mut` / field `const _field` → `mut _field`
6. A closed `const` phase boundary that fails loudly if a `const` name reaches runtime lowering

**Still open for P1**: T7 focused gates and failure classification; immutable-field construction lifecycle plus broader T8a semantic/incremental evidence; G1; T10; G2; and final T11 records. T9's focused LSP/REPL checks and grammar validation have passed. This walkthrough was updated at the T8a pause and does not close P1.

---

## 3. Architecture implemented

### Binding phase and mutability (T1–T3)

- Added `BindingKind::LetMut` variant to `phalcom-ast/src/ast.rs`
- Parser: `let mut` → `BindingKind::LetMut`, bare `let` → `BindingKind::Let`, `const` → `BindingKind::Const`
- `BindingKind::is_mutable()` and `BindingKind::requires_initializer()` methods
- `phalcom-modules/src/interface.rs`: `ModuleBindingSemantics::Runtime { mutable: bool }` replaces old `bool` mutability flag; `StaticConstant` variant added
- `phalcom-semantic` checker: `let` maps to `mutable: false`, `let mut` to `mutable: true`, `const` to static phase distinction

### Initialization lattice (T4)

- Three-state `InitializationState`: `Uninitialized`, `MaybeInitialized`, `DefinitelyInitialized`
- `BindingReadSafety` three-state: `ProvenInitialized`, `DefinitelyUninitialized`, `RuntimeCheckRequired`
- Semantic checker emits `BindingReadSafety` per-expression; compiler consumes it
- Closure/cross-boundary join: outer bindings written in closures transition `Uninitialized → MaybeInitialized` via `InitializationState::join`

### Runtime uninitialized binding (T5)

- `ValueTag::UninitializedBinding = 11` — distinct from `Nil (0)` and guest `None`
- `Value::uninitialized_binding()` and `is_uninitialized_binding()`
- `Value::wrap_some()` errors on `UninitializedBinding`
- `RuntimeError::UninitializedBinding { name }` for runtime trap
- New bytecodes: `UninitializedBinding`, `GetLocalChecked(slot, name_idx)`, `GetUpvalueChecked(idx, name_idx)`, `GetGlobalChecked(idx)`
- `GetLocal` remains completely branch-free (P1-INV-11)
- `GetLocalChecked` traps on `UninitializedBinding` and surfaces `RuntimeError`

### Static const barrier (T6)

- Compiler `static_const_scopes: Vec<HashSet<Symbol>>` and `global_static_constants: HashSet<Symbol>`
- `is_static_constant(name)` helper checks local const scopes and linked bindings
- `Statement::Let` with `BindingKind::Const`: registers name in static const registry, emits no bytecode, no `DefineGlobal`, no slot allocation
- `Expr::Var` with a static const name: fails with `CompilerError::StaticConstUnmaterialized`
- `Expr::Assignment` to a static const: fails with `CompilerError::AssignToImmutable`
- `CompilerError::StaticConstUnmaterialized(String)` and `ConstWithoutInitializer(String)` added
- `TopLevelBindingKind::StaticConstant` added to `phalcom-core/src/modules/linkage.rs`
- Removed invalid loop over `module.interface.declarations` that accessed non-existent fields
- Module interface: `ModuleBindingSemantics::StaticConstant` (pre-existing in phalcom-modules) confirmed as authority

### Corpus migration (T2)

- Migration binary `phalcom-ast/src/bin/c4_migrate.rs`
- Migrated 719 `.ph` files across all fixture directories
- `let x = expr` → `let mut x = expr` (runtime-visible bindings that may be assigned later)
- `const _field` class member syntax → bare `_field` (immutable)
- `let _field` and mutable inferred fields → `mut _field`
- Idempotence verified: second pass made zero changes
- Layout parity tool verified slot order preservation before inference deletion

---

## 4. Important code changes

| File | Change |
|---|---|
| `phalcom-ast/src/ast.rs` | Added `BindingKind::LetMut`; `is_mutable()`, `requires_initializer()` |
| `phalcom-ast/src/parser.rs` | Parse `let mut`, emit `LetMut`; field `mut _name` form |
| `phalcom-ast/src/bin/c4_migrate.rs` | Migration binary (new file) |
| `phalcom-modules/src/interface.rs` | `ModuleBindingSemantics::Runtime { mutable }`, `StaticConstant` |
| `phalcom-modules/src/fingerprint.rs` | Fingerprint updated for new semantics |
| `phalcom-semantic/src/checker/analysis.rs` | Updated for LetMut |
| `phalcom-semantic/src/checker/binding.rs` | Updated mutability logic |
| `phalcom-semantic/src/checker/context.rs` | Updated for 3-state init |
| `phalcom-semantic/src/checker/statement.rs` | `let` requires initializer; `let mut` allows omitting |
| `phalcom-semantic/src/checker/expression.rs` | `BindingReadSafety` lowering; closure join |
| `phalcom-semantic/src/checker/flow/mod.rs` | `InitializationState` 3-state lattice |
| `phalcom-semantic/src/checker/flow/state.rs` | `join()` operator |
| `phalcom-semantic/src/diagnostic.rs` | New diagnostic codes for init/const |
| `phalcom-semantic/src/source_index/` | Updated scope, reference, builder |
| `phalcom-core/src/value/repr.rs` | `ValueTag::UninitializedBinding = 11` |
| `phalcom-core/src/value/mod.rs` | `Value::uninitialized_binding()`, `is_uninitialized_binding()`, `wrap_some()` guard |
| `phalcom-core/src/bytecode.rs` | `UninitializedBinding`, `GetLocalChecked`, `GetUpvalueChecked`, `GetGlobalChecked` |
| `phalcom-core/src/vm/dispatch.rs` | Dispatch for new opcodes |
| `phalcom-core/src/compiler/lib/mod.rs` | `static_const_scopes`, `global_static_constants`, `is_static_constant()`, `const` handling in `Statement::Let` |
| `phalcom-core/src/compiler/lib/scope.rs` | Push/pop `static_const_scopes` in `begin_scope`/`end_scope` |
| `phalcom-core/src/compiler/lib/expr.rs` | `Expr::Var` and `Expr::Assignment` const barrier |
| `phalcom-core/src/compiler/lib/error.rs` | `StaticConstUnmaterialized`, `ConstWithoutInitializer` |
| `phalcom-core/src/modules/linkage.rs` | `TopLevelBindingKind::StaticConstant`; removed stale declarations loop |
| Corpus: 719 `.ph` files | `let` → `let mut`, field syntax migration |

---

## 5. Binding initialization design

The semantic layer owns all initialization facts. Three states:

```text
Uninitialized       → slot filled with UninitializedBinding at declaration
MaybeInitialized    → some paths wrote, some did not (runtime check required)
DefinitelyInitialized → all paths wrote (unchecked GetLocal)
```

Closure cross-boundary join: when a block closure writes to an outer binding, the outer binding's state is joined (via `InitializationState::join`) into the enclosing flow context, transitioning from `Uninitialized` → `MaybeInitialized`. This ensures closures don't incorrectly mark outer bindings as definitely uninitialized.

---

## 6. Runtime sentinel/check design

`ValueTag::UninitializedBinding = 11` (private, not user-accessible):
- Distinct from `Nil` (tag 0) and guest `None`
- Cannot be wrapped in `Some`
- Zero heap allocation
- `GetLocal`: branch-free, reads raw slot value
- `GetLocalChecked(slot, name_idx)`: checks tag, raises `RuntimeError::UninitializedBinding { name }` if uninitialized
- Upvalue and global checked variants follow the same pattern

---

## 7. Static const phase boundary

`const x = expr` in P1:
- Registers `x` in `global_static_constants` or `static_const_scopes`
- Emits **no** bytecode and allocates **no** runtime slot
- Any runtime reference to the name via `Expr::Var` → `StaticConstUnmaterialized` error
- Any assignment → `AssignToImmutable` error
- `ConstWithoutInitializer` if no initializer supplied

P2 must extend this to attach `StaticValue` and the static evaluator to the `const` declaration identity. P1 does not implement any static evaluator.

---

## 8. Field explicit-layout/lifecycle design

The T7 explicit-field implementation is present in the shared tree, but its focused gates have not yet been run and classified. Do not treat T7 as accepted until those gates pass.

The T8a compiler consumer now projects constructor factory/initializer members from canonical semantic constructor products and signatures. Its focused end-to-end test passed. Immutable-field write-once and constructor-exit lifecycle acceptance, plus broader semantic/incremental evidence for T8a, remain open.

The accepted superclass rule is explicit: a subclass constructor must call the direct parent's canonical initializer with `super.new(...)`; the compiler consumes the semantic resolved link and does not synthesize an omitted parent call.

---

## 9. Corpus migration details

- Tool: `phalcom-ast/src/bin/c4_migrate.rs` (AST-driven, idempotent)
- Files changed: 719
- Strategy: Parse each `.ph` file; for any `Statement::Let` with `BindingKind::Let` (no initializer would be a new error, so assignments also caught), rewrite surface text; for class fields, scan `ClassMember::Field` nodes for mutability
- Idempotence: second run changed 0 files
- Layout oracle: pre-migration slot order snapshot compared against post-migration explicit-field layout; parity confirmed before moving on

---

## 10. Deviations from plan

1. **T7 implementation is present, acceptance is pending**: focused explicit-field gates and failure classification remain outstanding.
2. **T8 is partial**: the T8a compiler consumer slice is implemented and focused-tested; immutable-field lifecycle and broader semantic/incremental checks remain open.
3. **T9 focused tooling checks passed**: LSP/REPL checks and TextMate JSON validation are recorded in the handoff; the extension-host suite was not run.
4. **P1 closure remains pending**: G1, T10, G2, and final T11 records have not been completed.
5. **Partial T6**: the static const barrier was implemented at compiler level. Module-level `StaticConstant` in `linkage.rs` removed the stale `declarations` loop since `LinkedModuleInterface` does not have `.declarations` (only `UnlinkedModuleInterface` does). The `const` barrier therefore relies on compile-time scope tracking rather than linked module inspection for within-module constants.

---

## 11. Consultations

The T8a constructor-consumer architecture was consulted and recorded in the checkpoint. The user selected the rule that subclass constructors require an explicit `super.new(...)` call; no synthesized parent call is permitted. This walkthrough records that decision, not a new consultation.

---

## 12. Tests added

### New test files
- `phalcom-core/tests/core/execution/binding_init.rs` — 11 T5 tests
- `phalcom-core/tests/core/compiler/static_const.rs` — 5 T6 tests
- `phalcom-semantic/tests/semantic/foundations/binding_init.rs` — semantic init tests

### T5 tests (all PASSED)
- `uninitialized_binding_is_not_none`
- `uninitialized_binding_is_not_nil_if_separate_tag`
- `uninitialized_binding_cannot_wrap_some`
- `checked_local_traps`
- `checked_local_after_assignment_succeeds`
- `checked_upvalue_traps_then_succeeds`
- `checked_global_traps`
- `explicit_none_checked_read_succeeds`
- `proven_local_uses_unchecked_getlocal`
- `maybe_local_uses_checked_getlocal`
- `checked_getlocal_is_not_fused_into_unchecked_invoke_local`

### T6 tests (all PASSED)
- `const_is_static_module_surface`
- `const_initializer_not_emitted_as_runtime_define`
- `const_does_not_allocate_runtime_local`
- `const_without_initializer_rejected`
- `runtime_use_of_unmaterialized_static_const_fails_closed`

### T8a focused consumer test (PASSED)
- `compiler_installs_and_executes_canonical_constructor_projections` — exercises `@construct`, `@data`, default-only parent construction, source constructors, exact factory/initializer selectors, and parent-before-child defaults.
- Added in `phalcom-core/tests/core/language/compiler/lowering_scenarios.rs`.

---

## 13. Tests executed

```sh
RUSTFLAGS='' cargo check -p phalcom-core     # PASSED
RUSTFLAGS='' cargo test -p phalcom-core --test core compiler_static_const   # 5/5 PASSED
RUSTFLAGS='' cargo test -p phalcom-core --test core execution_binding_init  # 11/11 PASSED
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic fields           # 33/33 PASSED
RUSTFLAGS='' cargo test -p phalcom-core field_layout                        # 2 passed, 1 ignored (GATED)
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_installs_and_executes_canonical_constructor_projections -- --nocapture  # 1 passed
```

T9 focused LSP and REPL selectors passed, and `tools/vsphalcom/syntaxes/phalcom.tmLanguage.json` parsed successfully; exact checks are listed in the handoff. The extension-host suite was not run.

---

## 14. Tests deferred

- G1 semantic/runtime behavior gate (broad `phalcom-semantic` + `phalcom-core` capability suites)
- G2 affected-crate stabilization gate (workspace crate tests)
- T10 incremental invalidation and performance evidence
- T7 explicit field layout tests: `bare_field_signature_is_immutable`, `mut_field_signature_is_mutable`, `assignment_to_undeclared_field_is_diagnostic`, `assignment_does_not_change_layout`, `explicit_field_layout_preserves_declaration_order`, `migrated_layout_matches_pre_cutover_oracle`
- T8 immutable-field lifecycle regressions and broader semantic/incremental evidence for the T8a consumer slice.
- T9 extension-host suite; focused LSP/REPL and grammar checks passed.

---

## 15. Performance evidence

- `GetLocal` is branch-free: confirmed structurally in `proven_local_uses_unchecked_getlocal` test (bytecode inspection shows `GetLocal`, no `GetLocalChecked`)
- `UninitializedBinding` uses an immediate `ValueTag` representation. T10's explicit zero-allocation confirmation remains pending.
- No formal benchmark run executed; T10 is not complete.

---

## 16. Residual/baseline failures

No known P1-caused failures. Corpus migration and semantic tests all green within the focused surface.

The `adt_lower_08_imported_field_layout_uses_declared_physical_slot` test is **ignored (GATED)** — baseline/pre-existing, not caused by P1.

---

## 17. Verification classification

| Task | Status | Verification |
|---|---|---|
| T0 | COMPLETE | FOCUSED_TESTED |
| T1 | COMPLETE | FOCUSED_TESTED |
| T2 | COMPLETE | FOCUSED_TESTED (parity confirmed) |
| T3 | COMPLETE | FOCUSED_TESTED |
| T4 | COMPLETE | FOCUSED_TESTED |
| T5 | COMPLETE | FOCUSED_TESTED (11/11) |
| T6 | COMPLETE | FOCUSED_TESTED (5/5) |
| T7 | IMPLEMENTATION_PRESENT | UNVERIFIED (focused gates pending) |
| T8a compiler consumer | IMPLEMENTED | FOCUSED_TESTED (one end-to-end test) |
| T8 immutable-field lifecycle | OPEN | UNVERIFIED |
| T9 | IMPLEMENTED | FOCUSED_TESTED (LSP/REPL; grammar JSON valid) |
| G1 | NOT_RUN | UNVERIFIED |
| T10 | NOT_STARTED | UNVERIFIED |
| G2 | NOT_RUN | UNVERIFIED |
| T11 | PARTIAL | N/A |

**Overall P1 verification**: PARTIAL. T0–T6, the focused T8a consumer test, and listed T9 checks are focused-tested. T7 acceptance, T8 lifecycle, G1, T10, and G2 remain unverified; T11 is partial.

---

## 18. Follow-up for P2

P2 will build the static evaluator on top of P1's `const` phase boundary. Before P2 can begin, P1 must be completed (T7–T11). The next executor should:

1. Run T7's focused explicit-field gates and classify every failure A/B/C/D.
2. Complete T8 immutable-field lifecycle enforcement and its positive/negative constructor-path regressions; finish broader semantic/incremental evidence for T8a.
3. Confirm and record T9's focused evidence (already passed); run the extension-host suite if required by the active plan.
4. Run G1's four named semantic and language-corpus gates.
5. Complete T10 incremental invalidation, `GetLocal` branch inspection, and zero-allocation evidence.
6. Run G2 serially across the six affected crates and classify all failures A/B/C/D.
7. Update the checkpoint and close the walkthrough/handoff in T11. P2 remains blocked until then.

P2 hook: the `const` declaration identity (name in `global_static_constants`) is where P2 should attach `StaticValue`, the initializer semantic body, declared result domain, and materialization descriptor.
