---
id: LANG007.C4.P1-walkthrough
kind: walkthrough
plan: LANG007.C4.P1
checkpoint: LANG007.C4
status: PARTIAL
completion: PARTIAL
verification: BASELINE_BLOCKED
created: 2026-10-04
---

# LANG007.C4.P1 — Walkthrough

## 1. Plan identity and final state

**Plan**: LANG007.C4.P1 — Binding, Mutability, Field Lifecycle, and Corpus Cutover  
**Status**: PARTIAL (T7–T10 implementation and focused checks are recorded; G1/G2 prevent plan closure)

**Completion**: T0–T6 COMPLETE; T7 explicit-field behavior and parity oracle complete; T8a products/compiler consumer implemented; T8 lifecycle implemented; T9 tooling implemented; T10 focused evidence complete; closure gates pending

**Verification**: focused T7/T8/T10 checks, the fields/classes corpus lanes, and full AST/LSP crate gates pass. G1 remains BASELINE_BLOCKED by the classified bindings corpus D failure. G2 is BASELINE_BLOCKED: semantic, modules, core, and REPL packages retain failures, and semantic failure classification is incomplete.

---

## 2. Objective and outcome

P1 was the foundational semantic cutover for `LANG007.C4`. The plan established:

1. Runtime immutable `let` and runtime mutable `let mut` as distinct syntactic forms
2. `const` as a static-phase declaration (not a runtime immutable binding)
3. True uninitialized mutable binding state via `ValueTag::UninitializedBinding`
4. Path-sensitive definite-initialization proof with runtime fallback via `GetLocalChecked`
5. Corpus-wide migration (719 files) from legacy `let` → `let mut` / field `const _field` → `mut _field`
6. A closed `const` phase boundary that fails loudly if a `const` name reaches runtime lowering

**Still open for P1**: finish G2 failure classification and T11. The core bindings corpus has a classified D failure. P1 remains partial and this walkthrough does not close it.

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

The T7 layout builder uses declared fields only. Focused semantic/core tests
reject undeclared writes, preserve declaration order and immutability, and the
post-inference-deletion parity oracle verifies legacy assignment order, both
field sides, own slot offsets, counts, superclass prefixes, and immutable
declaration migration.

The T8a compiler consumer projects constructor factory/initializer members
from canonical semantic constructor products and signatures. Its focused
end-to-end test passed. Immutable-field write-once and constructor-exit
lifecycle tests, plus semantic/incremental product checks, passed. The
positive `classes` corpus passed after migrating one stale fixture to the
approved explicit-parent-initializer rule.

The accepted superclass rule is explicit: a subclass constructor must call the direct parent's canonical initializer with `super.new(...)`; the compiler consumes the semantic resolved link and does not synthesize an omitted parent call.

---

## 9. Corpus migration details

- Tool: `phalcom-ast/src/bin/c4_migrate.rs` (AST-driven, idempotent)
- Files changed: 719
- Strategy: Parse each `.ph` file; for any `Statement::Let` with `BindingKind::Let` (no initializer would be a new error, so assignments also caught), rewrite surface text; for class fields, scan `ClassMember::Field` nodes for mutability
- Idempotence: second run changed 0 files
- Layout oracle: pre-migration slot order snapshot compared before inference deletion; the post-deletion oracle also passes against explicit-field layout

---

## 10. Deviations from plan

1. **T7 focused behavior passes**: explicit order, undeclared-write rejection, migration unit checks, and the post-deletion layout parity oracle pass.
2. **T8 focused lifecycle passes**: semantic constructor-path tests, core default/read runtime tests, semantic product/incremental checks, and the positive/negative class corpus lanes pass.
3. **T9 tooling checks**: the full LSP crate passes. The full REPL crate has one remaining failure in reflection-child import; its P1 field-mutation persistence fixture was updated to `mut _v` and passes. TextMate JSON validation passed; the extension-host suite was not run.
4. **T10 focused evidence passes**: binding-flow and field-mutability incremental edits, checked-read lowering, branch-free `GetLocal`, and zero-allocation sentinel checks pass.
5. **P1 closure remains pending**: the fields and classes corpus lanes pass; G1 remains baseline-blocked by the bindings corpus D failure. G2 ran: AST and LSP pass; modules, semantic, core, and REPL have failures. T11 remains partial while the remaining failures are classified.
6. **Partial T6**: the static const barrier was implemented at compiler level. Module-level `StaticConstant` in `linkage.rs` removed the stale `declarations` loop since `LinkedModuleInterface` does not have `.declarations` (only `UnlinkedModuleInterface` does). The `const` barrier therefore relies on compile-time scope tracking rather than linked module inspection for within-module constants.

---

## 11. Consultations

The T8a constructor-consumer architecture was consulted and recorded in the checkpoint. The user selected the rule that subclass constructors require an explicit `super.new(...)` call; no synthesized parent call is permitted. This walkthrough records that decision, not a new consultation.

---

## 12. Tests added

### New test files
- `phalcom-core/tests/core/execution/binding_init.rs` — 11 T5 tests plus 2 T10 regressions
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
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::bindings # 12 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::fields   # 27 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core field_layout -- --nocapture                # 4 passed, 1 ignored (GATED)
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core field_lifecycle -- --nocapture             # 2 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core binding_init -- --nocapture    # 13 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic binding_flow_edit_rechecks_only_the_affected_callable # 1 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic field_mutability_edit_invalidates_field_and_write_products # 1 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --bin c4_migrate test_migrate_implicit_fields # 1 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_installs_and_executes_canonical_constructor_projections -- --nocapture  # 1 passed
```

T9 focused LSP and REPL selectors passed, and `tools/vsphalcom/syntaxes/phalcom.tmLanguage.json` parsed successfully; exact checks are listed in the handoff. The extension-host suite was not run.

The required G1 `language-corpus bindings` command failed in
`bindings/destructure_list_no_rest.ph`: pattern lowering attempted
`unwrapOr(_)` on integer `1`. This is classified D and is outside P1. The G1
`language-corpus fields` lane now passes one fixture exercising explicit
instance fields, an immutable default, and a subclass's explicit
`super.new(...)` call. The positive `classes` corpus also passes after the
stale subclass-constructor fixture was updated to call its parent explicitly;
`classes_negative` passed.

---

## 14. Tests deferred

- G1 closure: the bindings corpus has a D failure.
- The positive `classes` corpus passed after the explicit-super fixture correction.
- G2 affected-crate stabilization gate (six crate test suites).
- T9 extension-host suite; focused LSP/REPL and grammar checks passed.

---

## 15. Performance evidence

- `GetLocal` is branch-free: confirmed structurally in `proven_local_uses_unchecked_getlocal` test (bytecode inspection shows `GetLocal`, no `GetLocalChecked`)
- `UninitializedBinding` uses an immediate `ValueTag` representation; the new 128-value heap-count regression passed with zero additional heap objects.
- T10 binding-flow and field-mutability incremental edit tests passed. No formal benchmark run executed.

---

## 16. Residual/baseline failures

The core bindings corpus has one failure classified D (baseline/unrelated):
`bindings/destructure_list_no_rest.ph` attempts to call `unwrapOr(_)` on integer
`1` through pattern lowering. Do not repair it under P1. The new core fields
corpus lane selects and passes one fixture. The positive classes corpus result
also passes after the stale subclass constructor fixture was updated to call
its parent explicitly.

The `adt_lower_08_imported_field_layout_uses_declared_physical_slot` test is **ignored (GATED)**.

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
| T7 | IMPLEMENTED | FOCUSED_TESTED (explicit layout and post-cutover parity oracle pass) |
| T8a compiler consumer | IMPLEMENTED | FOCUSED_TESTED (consumer, semantic products, incremental dependency) |
| T8 immutable-field lifecycle | IMPLEMENTED | FOCUSED_TESTED (semantic and core runtime tests) |
| T9 | IMPLEMENTED | FOCUSED_TESTED (LSP/REPL; grammar JSON valid) |
| G1 | INCOMPLETE | BASELINE_BLOCKED (semantic and fields corpus pass; bindings corpus D failure) |
| T10 | COMPLETE | FOCUSED_TESTED |
| G2 | INCOMPLETE | BASELINE_BLOCKED (AST and LSP pass; modules, semantic, core, and REPL have failures) |
| T11 | PARTIAL | N/A |

**Overall P1 verification**: PARTIAL / BASELINE_BLOCKED. T7, T8, T9, T10, and the fields/classes corpus lanes have focused evidence. G1 retains its classified D failure. G2 remains incomplete: semantic failures are not fully classified; modules/core/REPL failures need predecessor or causal confirmation. T11 remains partial.

### G2 affected-crate results

| Crate | Result | Evidence |
|---|---|---|
| `phalcom-ast` | PASS | Full package passed: 63 library, 5 migration-tool, 282 integration (1 ignored), 12 trait-syntax tests. |
| `phalcom-modules` | FAIL | `bcat_08_root_alias_does_not_create_synthetic_declaration` has an unresolved import of `universe.collections.iterator_laws` in its partial test interface graph; predecessor comparison remains. |
| `phalcom-semantic` | FAIL | 1,738 passed, 64 failed, 42 ignored. Three stale P1 tests were updated and each focused rerun passed; the full suite was not rerun after those edits, and the remaining failures are not fully classified. |
| `phalcom-core` | FAIL | 151 passed, 6 failed in the library target. The stale P1 loop-mutation fixture was corrected and its focused rerun passed. Other failing tests: `ic_add_method_invalidates`, `ic_override_after_caching`, `canonical_range_lower_option_closes_number_at_is_none_call_edge`, `kernel_option_match_override_reroutes_every_combinator`, `captured_setter_rejects_an_ordinary_positional_rhs_lane`, and `walk_orders_oldest_first_with_selector_shaped_names`. |
| `phalcom-lsp` | PASS | Full package passed after the field completion fixture declared `mut _client: Client`. |
| `phalcom-repl` | FAIL | 28 passed, 1 failed: `repl_mi_06_all_reflection_children_importable` reports `modifier.illegal_target` while importing the `module` reflection child. The P1 persistence fixture's focused rerun passed after changing `_v` to `mut _v`. |

The G2 package gate is `BASELINE_BLOCKED / INCOMPLETE`; do not present this as a green six-crate stabilization result. The semantic full-suite failures require a bounded classification pass before P1 closure.

`cargo fmt --all -- --check` fails on formatting differences across many
existing files, including untouched regions of some changed files. No broad
formatting was applied. `git diff --check` passes.

---

## 18. Follow-up for P2

P2 will build the static evaluator on top of P1's `const` phase boundary. Before P2 can begin, P1 must be completed (T7–T11). The next executor should:

1. Classify the remaining semantic failures and confirm the module/core/REPL D candidates against a clean predecessor or direct causal evidence.
2. Fix any P1-owned failures and rerun the relevant focused gates; rerun a broad package only when classification requires it.
3. Finish T11 with the checkpoint, walkthrough, and handoff after G1/G2 reach the plan's acceptance level. P2 remains blocked until then.

P2 hook: the `const` declaration identity (name in `global_static_constants`) is where P2 should attach `StaticValue`, the initializer semantic body, declared result domain, and materialization descriptor.
