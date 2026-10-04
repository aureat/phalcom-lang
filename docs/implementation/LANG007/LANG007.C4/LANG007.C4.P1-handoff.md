---
id: LANG007.C4.P1-handoff
kind: handoff
plan: LANG007.C4.P1
checkpoint: LANG007.C4
created: 2026-10-04
for: next implementer (G2 failure classification and T11 closure)
---

# LANG007.C4.P1 — Handoff

This handoff records the current **partial** P1 state after focused T7/T8/T10
work and the G2 affected-crate run. P1 remains active and P2 must not begin.

---

## 1. Current revision and state

The working branch is `main`. No feature branch was created.

```text
T0  COMPLETE — C4 takeover record established
T1  COMPLETE — BindingKind::LetMut AST surface added
T2  COMPLETE — 719 corpus files migrated
T3  COMPLETE — semantic binding/module/source semantics flipped
T4  COMPLETE — 3-state initialization lattice + BindingReadSafety
T5  COMPLETE — ValueTag::UninitializedBinding, GetLocalChecked, 11/11 tests passed
T6  COMPLETE — static const phase barrier, 5/5 tests passed
T7  COMPLETE — explicit-field behavior, layout parity oracle, and focused checks pass
T8a compiler consumer and semantic product/incremental checks pass; classes corpus lanes pass
T8 immutable lifecycle semantic/runtime checks pass; positive and negative classes corpus lanes pass
T9 tooling implementation and focused LSP/REPL checks passed
G1  BASELINE_BLOCKED / INCOMPLETE (semantic selectors and fields/classes corpus pass; bindings corpus D failure)
T10 FOCUSED_TESTED (incremental invalidation, checked-read lowering, branch-free GetLocal, zero heap allocation)
G2  BASELINE_BLOCKED / INCOMPLETE (AST and LSP pass; modules, semantic, core, and REPL have failures)
T11 PARTIAL (checkpoint and handoff updated; final plan closure records pending)
```

Focused T8a test that passed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_installs_and_executes_canonical_constructor_projections -- --nocapture
```

The approved superclass rule is explicit: subclass constructors call the
direct parent's canonical initializer with `super.new(...)`. The compiler
consumes the semantic resolved link; it does not synthesize a missing call.

The current continuation adds the T7 parity oracle, registers the G1 `fields`
corpus lane, adds one explicit-field/inherited-constructor corpus fixture, and
updates stale modifier fixtures found during G2. Both positive and negative
`classes` corpus lanes pass. The positive lane required updating
`ctor_inherited_bare_new.ph` to call `super.new()` under the approved
explicit-parent-initializer rule.

---

## 2. Completed prerequisites

The following are established and must not be redesigned:

- **`BindingKind::LetMut`** in `phalcom-ast/src/ast.rs` — canonical AST surface for mutable runtime bindings
- **`ValueTag::UninitializedBinding = 11`** in `phalcom-core/src/value/repr.rs` — private VM sentinel
- **`GetLocalChecked(slot, name_idx)`**, **`GetUpvalueChecked(idx, name_idx)`**, **`GetGlobalChecked(idx)`** opcodes in `phalcom-core/src/bytecode.rs`
- **`Value::uninitialized_binding()`**, **`Value::is_uninitialized_binding()`**, **`Value::wrap_some()` guard** in `phalcom-core/src/value/mod.rs`
- **`RuntimeError::UninitializedBinding { name }`** — runtime trap on checked read
- **`ModuleBindingSemantics::Runtime { mutable: bool }` / `StaticConstant`** in `phalcom-modules/src/interface.rs`
- **`global_static_constants: HashSet<Symbol>`** and **`static_const_scopes: Vec<HashSet<Symbol>>`** on `Compiler` struct
- **`CompilerError::StaticConstUnmaterialized(String)`** and **`CompilerError::ConstWithoutInitializer(String)`** in `phalcom-core/src/compiler/lib/error.rs`
- **`TopLevelBindingKind::StaticConstant`** in `phalcom-core/src/modules/linkage.rs`
- **Corpus**: 719 files migrated to `let mut` / explicit field syntax

---

## 3. Stable P1 interface map

### AST

```rust
// phalcom-ast/src/ast.rs
pub enum BindingKind { Let, LetMut, Const }
impl BindingKind {
    pub fn is_mutable(&self) -> bool        // true only for LetMut
    pub fn requires_initializer(&self) -> bool  // true for Let and Const
}
```

### Module semantics

```rust
// phalcom-modules/src/interface.rs
pub enum ModuleBindingSemantics {
    Runtime { mutable: bool },
    StaticConstant,
}
```

### Initialization lattice (semantic layer)

```rust
// phalcom-semantic/src/checker/flow/mod.rs or state.rs
pub enum InitializationState { Uninitialized, MaybeInitialized, DefinitelyInitialized }
impl InitializationState {
    pub fn join(self, other: Self) -> Self  // lattice join
}

pub enum BindingReadSafety {
    ProvenInitialized,
    DefinitelyUninitialized,
    RuntimeCheckRequired,
}
```

### Runtime value

```rust
// phalcom-core/src/value/mod.rs
impl Value {
    pub fn uninitialized_binding() -> Value
    pub fn is_uninitialized_binding(&self) -> bool
    pub fn wrap_some(self) -> Value  // panics on UninitializedBinding
}
```

### Bytecodes

```rust
// phalcom-core/src/bytecode.rs
Bytecode::UninitializedBinding              // push sentinel onto stack
Bytecode::GetLocalChecked(u8, u16)         // slot, name_idx; traps on sentinel
Bytecode::GetUpvalueChecked(u8, u16)       // idx, name_idx; traps on sentinel
Bytecode::GetGlobalChecked(u32)            // idx; traps on sentinel
```

### Compiler const tracking

```rust
// phalcom-core/src/compiler/lib/mod.rs  (on Compiler struct)
fn is_static_constant(&self, name: Symbol) -> bool
// checks both global_static_constants and static_const_scopes
```

---

## 4. P1 invariants in force

The following are **FIXED** and must not be weakened:

- **P1-INV-01**: `let` = runtime immutable; `let mut` = runtime mutable; `const` = static phase
- **P1-INV-02**: `let` without initializer is a compile error
- **P1-INV-03**: `const` without initializer is a compile error
- **P1-INV-04**: `let mut` without initializer is valid
- **P1-INV-05**: `UninitializedBinding` is never `None` and never guest-visible
- **P1-INV-06**: `DefinitelyInitialized` reads use unchecked `GetLocal` (branch-free)
- **P1-INV-07**: `MaybeInitialized` reads use `GetLocalChecked` (trapped)
- **P1-INV-08**: Semantic layer owns all initialization proofs; compiler does not re-derive
- **P1-INV-09**: `const` at module/local scope registers in const tracking and emits no runtime slot
- **P1-INV-10**: Assigning to a `const` name at runtime scope is `CompilerError::AssignToImmutable`
- **P1-INV-11**: `GetLocal` has no uninitialized branch (proven reads stay branch-free)
- **P1-INV-12**: `const` is not materialized via P2 static evaluator during P1

---

## 5. P1 task evidence and remaining closure work

### T7 — Verify explicit field layout authority

The explicit-field implementation and undeclared-write diagnostic are present.
A live search confirms `class_decl.rs` no longer has the `collect_assigned_fields`
inference fallback or `has_declared_fields` branch. Focused semantic/core
layout tests, the AST implicit-field migration test, and the
`migrated_layout_matches_pre_cutover_oracle` test pass. The parity oracle parses
legacy assignment order and compares it with compiled explicit instance and
class-side layouts, including own-slot offsets, counts, superclass prefixes,
and immutable-to-bare field migration.

**Files to inspect first**:
- `phalcom-core/src/compiler/lib/class_decl.rs`: confirm physical layout comes from explicit declared fields and preserves declaration order.
- `phalcom-semantic/src/checker/expression.rs`: confirm undeclared field writes produce the planned diagnostic.
- Relevant parser/core tests and the scoped T7 diff.

**Focused evidence already run**:
- Semantic `assignment_does_not_declare_a_field` passed in the field capability suite.
- Core `undeclared_assignment_cannot_add_a_field_layout_slot` and
  `explicit_field_layout_preserves_declaration_order_and_mutability` passed.
- AST migration test `test_migrate_implicit_fields` passed.
- `cargo test -p phalcom-core field_layout -- --nocapture`: 4 passed,
  1 ignored (GATED).

**Status**: T7 is complete as `IMPLEMENTED / FOCUSED_TESTED`.

The legacy `const` field spelling is intentionally rejected by the current
parser, so the test freezes the historical immutable field names in its oracle
data and compares them against current bare immutable declarations.

**Key invariant**: `P1-INV-15` (explicit fields only). After T7, assignment is a consumer of declared storage, never a creator.

**STOP AND CONSULT if**: focused results expose a material cross-layer or semantic conflict that cannot be resolved within the accepted explicit-field design.

### T8 — Verify and close immutable-field construction lifecycle

T8a projects constructor factory/initializer members from published semantic
products. Its compiler consumer test, semantic constructor-product tests, and
incremental parent-product dependency tests pass. The instance-default
collector now uses canonical `member_side`, excluding `@class` defaults from
instance construction products. T8 immutable-lifecycle semantic tests and core
runtime default/read tests pass. The positive `classes` corpus passed after
migrating its stale subclass-constructor fixture to call the parent
initializer explicitly; `classes_negative` also passed.

**Files and existing regressions to inspect/run**:
- `phalcom-semantic/src/checker/field_lifecycle.rs`, `body.rs`, and `expression.rs`: verify lifecycle transitions, signature ownership, and every normal constructor exit.
- `phalcom-semantic/tests/semantic/capabilities/fields.rs`:
  - `immutable_constructor_write_is_exactly_once`
  - `immutable_field_write_outside_constructor_is_rejected_and_mutable_reassignment_remains_valid`
  - `immutable_constructor_exit_requires_every_normal_path_to_initialize`
- `phalcom-core/tests/core/language/field_lifecycle.rs`:
  - `early_method_read_of_immutable_field_traps_during_construction`
  - `declared_default_is_visible_to_immutable_field_reads`
- Confirm constructor writes satisfy:
  - In constructor: `U → D` is legal (first write)
  - In constructor: `M → write` or `D → write` on immutable field → diagnostic
  - Outside constructor: any write to immutable field → diagnostic
- Confirm the diagnostic codes and constructor-path cases required by plan §45. Add only missing coverage after inspecting the existing tests.

**Key invariants**: P1-INV-18 through P1-INV-23.

### T9 — Tooling alignment

The tooling changes are present. Focused checks passed, and the full LSP
package gate passed after making its field-completion fixture explicit about
the field type. The full REPL package gate now has one remaining reflection
import failure after migrating its stale mutable-field fixture:

- LSP `keyword_hovers_describe_canonical_binding_forms` — 1 passed.
- LSP filter `mut` — 2 passed, covering identifier/contextual modifier.
- LSP `binding_hover_ranges_are_exact_and_scope_aware` — 1 passed.
- REPL filter `let_mut` — 2 passed.
- REPL `contextual_mut_highlighting_respects_declarations_and_identifiers` — 1 passed.
- VS Code TextMate grammar JSON parsed successfully. The extension-host suite
  was not run.

- `phalcom-lsp/src/hover.rs`: update hover to report `let`/`let mut`/`const` canonically
- `phalcom-lsp/src/semantic_tokens.rs`: contextual `mut` for `let mut` and `mut _field`
- `tools/vsphalcom/syntaxes/phalcom.tmLanguage.json`: update TextMate grammar
- `phalcom-repl/src/highlighter.rs`: update for `let mut` and `mut _field`

### G1 — Semantic/runtime behavior gate

G1 is **BASELINE_BLOCKED / INCOMPLETE**:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::bindings # 12 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::fields   # 27 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus bindings          # D failure below
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus fields            # 1 passed
```

The bindings corpus failure is `bindings/destructure_list_no_rest.ph`: pattern
lowering calls `unwrapOr(_)` on integer `1`. It is classified D and remains
outside P1. The new `fields` lane executes an explicit-field class with a
defaulted immutable field, a mutable field, and a subclass that explicitly
calls `super.new(...)`; its corpus test passed. Both classes corpus tests
passed after the stale explicit-super fixture was migrated.

### T10 — Incremental and performance stabilization

T10 is focused-tested:

- Binding-flow edit changed the semantic read product and re-lowered the local
  read from `GetLocal` to `GetLocalChecked`.
- Bare-field ↔ `mut` edit invalidated field signature and affected write
  products while reusing the unrelated callable product.
- `binding_init` passed 13 tests, including the structural `GetLocal` path and
  zero heap allocations for 128 `UninitializedBinding` values.

### G2 — Affected-crate stabilization gate

```sh
cargo test -p phalcom-ast
cargo test -p phalcom-modules
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
cargo test -p phalcom-core
cargo test -p phalcom-lsp
cargo test -p phalcom-repl
```

Results from the serial G2 run:

| Crate | Result | Details / next action |
|---|---|---|
| `phalcom-ast` | PASS | 63 parser/library, 5 migration-tool, 282 integration (1 ignored), and 12 trait-syntax tests passed. The migration utility now accepts only its exact legacy-`const` recovery diagnostic and determines field conversion from source modifiers. |
| `phalcom-modules` | FAIL | `bcat_08_root_alias_does_not_create_synthetic_declaration` cannot link `universe.collections.iterator_laws`; the test's partial interface set omits the dependency. Confirm against a clean predecessor before calling it D. |
| `phalcom-semantic` | FAIL | 1,738 passed, 64 failed, 42 ignored. Three stale P1 fixtures (runtime-mutability, initializerless `let`, and loop-binder mutability) were updated; each exact rerun passes. The full package was not rerun after those fixture edits. Triage the remaining failures; do not assume the whole set is baseline. |
| `phalcom-core` | FAIL | 151 passed, 6 failed in the library target. The P1 stale `let i` fixture is now `let mut i`, and its exact VM test passes. Remaining library failures are inline-cache warming, Range lowering, Option match override routing, captured setter arity, and traceback selector handling. The command stopped before other core targets. |
| `phalcom-lsp` | PASS | Full package passed after the completion fixture declared `mut _client: Client`. |
| `phalcom-repl` | FAIL | 28 passed; `repl_mi_06_all_reflection_children_importable` remains failing on `modifier.illegal_target` while importing reflection child `module`. The P1 `mut _v` persistence fixture passes in isolation. |

G2 remains `BASELINE_BLOCKED / INCOMPLETE`. The module/core/REPL failures are
outside the P1 source changes and are D candidates, but predecessor evidence
has not been collected. The semantic remainder is unclassified. Exact focused
P1 tests for binding mutability, missing initializers, loop-binder
immutability, LSP field completion, and REPL import persistence now pass.
`cargo fmt --all -- --check` fails on formatting drift across many existing
files, including untouched regions of files in this worktree. No broad
formatting was applied; `git diff --check` passes.

### T11 — Completion records

- Update `LANG007.C4-CHECKPOINT.md` with P1 final state
- Close this handoff/walkthrough with final verification classification

These records now reflect the available G1/G2 evidence, but P1 plan completion
remains open until the remaining G2 failures are classified and any P1-owned
failures are repaired or verified. Keep T11 partial and P2 blocked.

---

## 6. Known baseline/deferred failures

- `adt_lower_08_imported_field_layout_uses_declared_physical_slot` — **IGNORED (GATED)**.
- `language-corpus bindings` fails on `bindings/destructure_list_no_rest.ph`
  because pattern lowering applies `unwrapOr(_)` to integer `1`. Classification:
  D, unrelated baseline; do not repair in P1.
- `language-corpus fields` now passes one fixture covering explicit fields and
  the approved explicit-super constructor rule.
- The positive `language-corpus classes` lane passed after the inherited
  constructor fixture was updated to call `super.new()`; `classes_negative`
  passed.
- `bcat_08_root_alias_does_not_create_synthetic_declaration` fails because the
  test's partial interface graph omits imported module
  `universe.collections.iterator_laws`; retain it as a D candidate pending
  predecessor comparison.
- The broad semantic package command reported 64 failures and 42 ignored. The
  focused P1 stale-fixture failures were corrected and rerun, but the broad
  package was not rerun and the remaining set is unclassified.
- The core package command reported six library failures after the P1 loop
  fixture was corrected: two inline-cache tests, Range lowering, Option match
  override routing, captured setter arity, and selector-shaped traceback.
- The LSP package passes. The REPL package retains one failure in
  `repl_mi_06_all_reflection_children_importable`, reporting
  `modifier.illegal_target` for the `module` reflection child.

---

## 7. Must-read files

Before touching T7, read:

```text
phalcom-core/src/compiler/lib/class_decl.rs  (lines 416–554: layout pass; 1416–1540: helpers)
phalcom-semantic/src/checker/expression.rs   (line 452: Expr::Field assignment handler)
phalcom-semantic/src/checker/declaration_signature.rs  (FieldSemanticSignature)
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P1-luna-patch-grade-implementation-plan.md  (T7 §25)
```

Before touching T8, also read:

```text
phalcom-semantic/src/checker/field_lifecycle.rs   (if it exists; else create)
```

---

## 8. Current next objective: finish G2 classification and T11

P1 remains `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED`. Do not begin C4.P2.
The next implementer should:

1. Compare the module/core/REPL failures with a clean predecessor or collect
   direct causal evidence; do not label them D solely because they are outside
   the current source diff.
2. Triage the remaining semantic package failures. The full semantic run was
   not repeated after three stale P1 expectations were corrected.
3. Repair any remaining P1-owned issue, then rerun its focused gate. Re-run
   broader crate suites only where the classification requires it.
4. Update the checkpoint, walkthrough, and this handoff with the final G1/G2
   classification, then close T11 when the plan's acceptance conditions are
   met.

P2 implements the static evaluator and `StaticValue`; it must not begin until
P1 T7–T11 are complete.

P2 implements the static evaluator and `StaticValue`. P2 must not begin until P1 T7–T11 are complete.

P2 hook: the `const` declaration identity is the `HashMap` entry in `global_static_constants` / `static_const_scopes` (compiler side) and `ModuleBindingSemantics::StaticConstant` (module interface side). P2 should attach `StaticValue`, the initializer semantic body, declared result domain, dependency fingerprint, and materialization descriptor to this identity — not to a new runtime slot.

---

## 9. Recommended first commands

```sh
# Select an exact remaining semantic failure from the package test output and
# reproduce it individually before making changes.
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test builtin_catalog bcat_08_root_alias_does_not_create_synthetic_declaration
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-repl --test repl_imports repl_mi_06_all_reflection_children_importable
```

---

## 10. Do-not-redesign guidance

The following must not be changed:

```text
- Do NOT turn `const` back into a runtime immutable slot
- Do NOT replace UninitializedBinding state with None
- Do NOT bypass semantic phase ownership of initialization facts
- Do NOT restore implicit field layout inference
- Do NOT add a second VM or second pattern system for static evaluation
- Do NOT weaken `GetLocal` to include an uninitialized sentinel check
- Do NOT let MaybeInitialized reads silently succeed without a runtime check
- Do NOT make const materialization happen inside P1 (that is P2's job)
```
