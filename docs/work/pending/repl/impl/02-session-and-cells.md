# §02 — Stage 1: session module, cell loop, echo mode

Implements plan.md **§D1** (one session module), **§D10** (cell boundaries unwind), and
**§D3** (echo mode). First point at which the REPL evaluates anything.

**Phase A for the `phalcom-core` half (§2.2), Phase B for the `phalcom-repl` half.** See
[§00](00-branch-protocol.md); the split is why this stage is written with its core-side
change isolated into one subsection.

## 1. §D1 — one session module

### 1.1 Shape

The session owns a `VM` and a single `ModuleObject` for its whole lifetime. Per cell:

```
compile_closure(module, src, UnitKind::Repl)  →  run  →  unwind_cell()  →  snapshot  →  print
```

Globals accumulate in the shared table. Precondition 1 (`patterns.rs:44` emits
`DefineGlobal` for top-level binds) and precondition 2 (`ModuleObject.declare` is
idempotent, `heap/module.rs:156`) mean **persistence needs no new mechanism**. Do not
add one.

### 1.2 `ReplSession` becomes real

`phalcom-repl/src/repl.rs` is 40 lines and entirely a stub. Replace `ReplSession`'s body:

```rust
pub struct ReplSession {
    vm: phalcom_core::vm::VM,
    module: phalcom_core::heap::ObjRef,
    cwd: std::path::PathBuf,
    next_cell: usize,
    /// Every cell's source, in submission order — `:reload`'s input (§S9).
    history: Vec<String>,
}
```

`cwd` loses its `#[allow(dead_code)]`: it becomes the module's path argument, which is
what the annotation's "will be read once the VM is wired up" anticipated.

`eval` returns a result the loop can render, not a counter:

```rust
pub enum CellOutcome {
    /// An expression cell; render `// => {0}` (§S4 value echo).
    Value(phalcom_core::value::Value),
    /// A statement cell; render nothing.
    Unit,
    /// Compile or runtime failure; the diagnostic has already been printed.
    Failed,
}
```

### 1.3 Rejected, per plan.md §D1 — do not revisit

- **Replaying the accumulated buffer per cell.** Side effects re-fire; O(n²).
  (`:reload` in [§07](07-commands.md) replays *by explicit user request*. That is not
  the same thing and does not reopen this.)
- **Cell-as-module with a parent-lookup chain.** Adds a fallback walk to the hot
  `GetGlobal` path to serve a REPL-only feature.

### 1.4 Class redefinition — settled, write against the post-CLASSCLOSE premise

PDR-0001 ruling 6: **cells shadow; they do not reopen.** A later `class Foo` binds
a new class; instances made under the old definition keep it (they hold a `ClassId`);
the old class becomes unreachable by name. No live object is silently patched.

This needs **no machinery beyond §D1/§D2** — it is what a fresh binding in a shared
global table already does. Write the cell loop assuming reopening does not exist, which
is what U-CLASSCLOSE makes true. If U-REPL lands first, the behavior is identical
anyway; if it lands second, nothing rebases. That is the point of writing it this way.

## 2. §D3 — echo mode

### 2.1 Behavior

A Repl-mode unit suppresses the trailing `Pop` on a **final expression statement** and
leaves the value on the stack for the loop to print. Statement cells echo nothing. `_`
binds the last value as an ordinary global.

Mirrors CPython's `"single"` mode. Rejected (plan.md §D3): Lua-style `return`-prepending
with retry — double-compiles and misreports spans on the retry path.

### 2.2 The type is new and orthogonal to `CompileMode` — RULED

> **This overrides the literal wording of plan.md §D3.** The design is unchanged; the
> name was already taken.

`compiler::attributes::CompileMode` exists and means something else: ADR-0052's
contract-weaving axis, `Debug` / `Release` / `Unchecked`, deciding whether `@requires` /
`@ensures` / `@invariant` guards are woven or stripped. It is a **global** setting, held
once at `vm/mod.rs:232` and read by the attribute expanders through
`ExpandCtx.compile_mode` (`compiler/attributes.rs:46`).

Adding a `Repl` variant there is wrong twice over: it forces every contract expander to
answer "how do contracts weave in Repl mode?" (not a question), and it puts a **per-cell**
property in a **process-global** field.

Add instead:

```rust
/// Whether a compilation unit is a whole file or a single REPL cell.
///
/// Orthogonal to [`CompileMode`](crate::compiler::attributes::CompileMode), which
/// governs contract weaving and is global; this is per-unit. A `Repl` unit keeps
/// its final expression's value instead of popping it (U-REPL §D3) and relaxes
/// prior-unit binding checks (§D4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnitKind {
    /// A whole source file. Every statement's value is discarded.
    #[default]
    File,
    /// One REPL cell.
    Repl,
}
```

Home: `compiler/lib/mod.rs`, beside `Compiler`. Threaded as a `Compiler` field set at
construction — the same shape `source_id` took in §D2 (`Compiler::new(vm, module,
source_id)` at `compiler/lib/mod.rs`), so follow that precedent and extend the
constructor rather than inventing a builder.

`VM::compile_closure` (`interpret.rs:148`) gains a sibling rather than a breaking
signature change:

```rust
pub fn compile_closure_as(&mut self, module: ObjRef, source: &str, kind: UnitKind) -> PhResult<ObjRef>
```

with the existing `compile_closure` delegating as `UnitKind::File`. Every current caller
keeps compiling. **Do not change `compile_closure`'s signature** — it has callers in
`interpret.rs`, `vm/api.rs`, and two integration tests, and a churned signature is a
merge conflict against the class work for no benefit.

### 2.3 Where the `Pop` is suppressed

The trailing `Pop` on an expression statement is emitted by the statement lowering in
`compiler/lib/`. Locate the exact emit site before editing — it is the *only* site that
changes — and gate it on `self.unit_kind == UnitKind::Repl && is_final_statement`.

"Final" means the last statement of the unit's top-level block, not the last of any
block. A `Repl` unit whose last statement is a `let` or a `class` echoes nothing.

## 3. §D10 — cell boundaries unwind, they do not raw-clear

### 3.1 The defect being avoided

Precondition 4: `run_in_module` (`interpret.rs:170`) clears `frames` and `stack` but
**not** `open_upvalues`. That map is keyed by **absolute value-stack index**. Clearing
the stack beneath it is safe exactly once — at the outermost entry, when the map is
already empty. Repeated per cell, it aliases cell N's captured slots onto cell N+1's
values: **silent corruption, not a crash.** Same family as the F1 fiber-floor defect and
E001–E003.

### 3.2 The rule

The cell loop calls `VM::unwind_cell()` ([§01 §3.1](01-wiring.md)) at every cell
boundary — **after** the cell completes, whether it succeeded or failed. It must not call
`run_in_module` per cell.

**`runtime_error` prints a trace but does not unwind.** The cell loop owns unwinding
after a failed cell and must not assume the VM cleaned up. This is stated in plan.md
§D10 and is easy to get wrong, because the success path and the failure path both need
it and only the failure path looks like it might not.

### 3.3 Fibers survive

Precondition 5: `FiberObject` holds its own `stack`, `frames`, `open_upvalues`
(`heap/fiber.rs:62`), empty while running and mirrored into the VM's. At a cell boundary
only the **running** fiber's state is in `vm.*`. Suspended fibers are untouched and
remain resumable from a later cell. Do not clear them, and do not add a fiber sweep.

## 4. Tests

Rust integration tests in `phalcom-core/tests/` for anything provable without a terminal;
`phalcom-repl` gets unit tests for the loop's sequencing.

| Test | Proves | Home |
|---|---|---|
| `globals_persist_across_cells` | `let x = 1` in cell 1 readable in cell 2, one module | `phalcom-core/tests/` |
| `echo_mode_keeps_final_expression` | `Repl` unit yields a value; `File` unit does not | `phalcom-core/tests/` |
| `statement_cell_echoes_nothing` | `let x = 1` as a cell yields no value | `phalcom-core/tests/` |
| `underscore_binds_last_value` | `_` is an ordinary global holding the prior cell's value | `phalcom-core/tests/` |
| `open_upvalue_hygiene_across_cells` | **the §D10 regression** — see below | `phalcom-core/tests/` |
| `class_redefinition_shadows` | cell 2's `class Foo` is a new class; a cell-1 instance keeps the old one | `phalcom-core/tests/` |

**`open_upvalue_hygiene_across_cells` is the load-bearing one.** Shape: a cell that
errors while a captured local is live, followed by a cell that pushes new values, must
not alias. It must fail if `unwind_cell` is replaced by a raw clear — write it against
that specific substitution and verify it goes red when you make it, before landing.
A test for this defect that passes under both implementations is worthless.

## 5. Write-set

| Path | Change | Phase |
|---|---|---|
| `phalcom-core/src/compiler/lib/mod.rs` | `UnitKind`; `Compiler` field + constructor arg | **A** |
| `phalcom-core/src/compiler/lib/` (statement lowering) | gate the final-expression `Pop` | **A** |
| `phalcom-core/src/interpret.rs` | `compile_closure_as` | **A** |
| `phalcom-core/tests/` | the six tests above | A |
| `phalcom-repl/src/repl.rs` | `ReplSession` becomes real; `CellOutcome` | B |
| `phalcom-repl/src/main.rs` | loop calls `eval`, renders outcome | B |

**Conflict risk vs class work:** `compiler/lib/mod.rs` is wanted by all three units.
U-CLASSNS adds `current_class: Option<ClassKey>` and `class_key()`; U-CLASSCLOSE edits
the `global_bindings` doc at `:57-61`. This stage adds a separate field and a separate
type. Textually adjacent, semantically disjoint — a rebase resolves by keeping both.
Land Phase A before U-CLASSNS if at all possible.

## 6. Gate

Workspace green, 28 suites + the six new tests, 0 failures. `unwind_cell`'s
`#[allow(dead_code)]` from [§01](01-wiring.md) is **deleted** in this stage.

Manual check, because no automated test covers the terminal: `cargo run -p phalcom-repl`,
then `let x = 1`, `x + 1` (expect `// => 2`), `x = 5`, `_` — and confirm a cell that
raises leaves the next cell usable.
