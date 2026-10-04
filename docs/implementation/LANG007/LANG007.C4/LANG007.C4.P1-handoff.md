---
id: LANG007.C4.P1-handoff
kind: handoff
plan: LANG007.C4.P1
checkpoint: LANG007.C4
created: 2026-10-04
for: next implementer (continuing T7–T11)
---

# LANG007.C4.P1 — Handoff

This handoff records a **partial** P1 pause after the T8a compiler-consumer
slice. P1 remains active and P2 must not begin.

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
T7  explicit-field implementation is present in the shared tree; focused gates pending
T8a compiler consumer implemented; focused end-to-end test passed; broader semantic/incremental evidence pending
T8 lifecycle acceptance remains open
T9 tooling implementation and focused LSP/REPL checks passed
G1  NOT_RUN
T10 NOT_STARTED
G2  NOT_RUN
T11 PARTIAL (checkpoint/handoff updated for this pause; final closure records pending)
```

Focused T8a test that passed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_installs_and_executes_canonical_constructor_projections -- --nocapture
```

The approved superclass rule is explicit: subclass constructors call the
direct parent's canonical initializer with `super.new(...)`. The compiler
consumes the semantic resolved link; it does not synthesize a missing call.

Untracked files (do not `git clean`):
- `docs/implementation/LANG007/LANG007.C3/LANG007.C3.P5-full-implementation-audit-2026-10-01.md`
- `docs/implementation/LANG007/LANG007.C4/` (entire directory, untracked)
- `phalcom-ast/src/bin/c4_migrate.rs`
- `phalcom-core/tests/core/execution/binding_init.rs`
- `phalcom-semantic/tests/semantic/foundations/binding_init.rs`

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

## 5. Remaining tasks and required files

### T7 — Verify explicit field layout authority

The explicit-field implementation and undeclared-write diagnostic are present
in the shared worktree. A live search confirms `class_decl.rs` no longer has the
`collect_assigned_fields` inference fallback or `has_declared_fields` branch.
T7 is not accepted yet: inspect the scoped diff, run the focused
explicit-field/parser/core tests, and classify failures before changing code.

**Files to inspect first**:
- `phalcom-core/src/compiler/lib/class_decl.rs`: confirm physical layout comes from explicit declared fields and preserves declaration order.
- `phalcom-semantic/src/checker/expression.rs`: confirm undeclared field writes produce the planned diagnostic.
- Relevant parser/core tests and the scoped T7 diff.

**Expected focused cases**:
  - `assignment_does_not_create_field`
  - `undeclared_field_write_is_diagnostic`
  - `explicit_layout_preserved_after_inference_deletion`
  - Layout parity oracle check (compare pre-T7 slot order against post-T7)

**Key invariant**: `P1-INV-15` (explicit fields only). After T7, assignment is a consumer of declared storage, never a creator.

**STOP AND CONSULT if**: focused results expose a material cross-layer or semantic conflict that cannot be resolved within the accepted explicit-field design.

### T8 — Verify and close immutable-field construction lifecycle

T8a now projects constructor factory/initializer members from the published
semantic products. The focused compiler consumer test passed; T8's lifecycle
acceptance and the broader T8a semantic/incremental checks remain open. The
`field_lifecycle.rs` module and checker hooks already exist in the shared tree.
The live checker contains immutable writes restricted to the canonical
constructor, repeat-write diagnostics, constructor-exit checks, and explicit
super-call enforcement. Focused semantic/runtime tests also exist, but this
continuation did not run them. Verify their coverage and results before editing.

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

The tooling changes are present and these focused checks passed:

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

Run after T8:

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::bindings
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::fields
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus bindings
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus fields
```

### T10 — Incremental and performance stabilization

- Verify incremental: `let → let mut` edit invalidates semantic product, re-lowers to checked read
- Verify `field bare → mut` invalidates field signature product
- Verify `GetLocal` bytecode path has no uninitialized check (structural opcode inspection)
- Verify `UninitializedBinding` representation is zero heap allocation

### G2 — Affected-crate stabilization gate

```sh
cargo test -p phalcom-ast
cargo test -p phalcom-modules
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
cargo test -p phalcom-core
cargo test -p phalcom-lsp
cargo test -p phalcom-repl
```

### T11 — Completion records

- Update `LANG007.C4-CHECKPOINT.md` with P1 final state
- Close this handoff/walkthrough with final verification classification

---

## 6. Known baseline/deferred failures

- `adt_lower_08_imported_field_layout_uses_declared_physical_slot` — **IGNORED (GATED)**, pre-existing baseline. Classification: D. Not P1-caused.

No other deferred failures identified in the focused T0–T6 surface.

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

## 8. Next objective: C4.P2

P2 implements the static evaluator and `StaticValue`. P2 must not begin until P1 T7–T11 are complete.

P2 hook: the `const` declaration identity is the `HashMap` entry in `global_static_constants` / `static_const_scopes` (compiler side) and `ModuleBindingSemantics::StaticConstant` (module interface side). P2 should attach `StaticValue`, the initializer semantic body, declared result domain, dependency fingerprint, and materialization descriptor to this identity — not to a new runtime slot.

---

## 9. Recommended first commands

```sh
# Verify current clean state
RUSTFLAGS='' cargo check -p phalcom-core

# Confirm T6 tests still pass
RUSTFLAGS='' cargo test -p phalcom-core --test core compiler_static_const

# Confirm T5 tests still pass
RUSTFLAGS='' cargo test -p phalcom-core --test core execution_binding_init

# Survey field inference code before T7
rg -n 'collect_assigned_fields' phalcom-core/src/compiler/lib/class_decl.rs
rg -n 'has_declared_fields' phalcom-core/src/compiler/lib/class_decl.rs
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
