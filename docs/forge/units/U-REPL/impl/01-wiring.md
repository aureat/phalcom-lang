# §01 — Stage 0: wire `phalcom-repl` to the language

**Phase A. Lands on `main`.** Prerequisite for every later stage.

Neither plan.md nor surface.md states this stage, because both assume the REPL crate can
already reach the compiler. It cannot.

## 1. The precondition, verified

```
phalcom-repl/Cargo.toml [dependencies]:
    unicode-segmentation, once_cell, regex, reedline, nu-ansi-term
```

No `phalcom-core`. No `phalcom-ast`. No `phalcom-lsp`.

```sh
grep -rn "phalcom_core\|phalcom_ast\|phalcom_lsp" phalcom-repl/src/   # returns nothing
```

`ReplSession::eval` (`phalcom-repl/src/repl.rs:35`) increments a counter and returns it;
its own doc says "Currently a stub — compilation and VM execution are not yet wired up."
`ReplSession.cwd` carries `#[allow(dead_code)] // will be read once the VM is wired up`.
This stage is the wiring that comment anticipates.

## 2. Dependencies to add

```toml
# phalcom-repl/Cargo.toml
phalcom-core = { path = "../phalcom-core" }
phalcom-ast  = { path = "../phalcom-ast" }
phalcom-lsp  = { path = "../phalcom-lsp" }
```

Add all three now, in one commit, even though `phalcom-ast` is not consumed until §S5-L1
and `phalcom-lsp` not until §S2/§S5-L2. Splitting them across stages buys nothing and
costs three `Cargo.lock` churns on a branch that must stay rebase-clean.

**Direction check (§D8).** `phalcom-repl` → {`phalcom-core`, `phalcom-lsp`}. The arrow
never reverses: `phalcom-lsp` gains no dependency on `phalcom-core`, so ADR-0056 §2's
VM-free constraint on the LSP is untouched. Adding `phalcom-core` to `phalcom-lsp` is
out of the write-set for the entire unit.

`phalcom-lsp` exposes what §S2/§S5-L2 need as a library — `src/lib.rs` declares
`pub mod completion; pub mod semantic_tokens; pub mod selectors; pub mod line_index;`
among others. It is consumed, never modified.

## 3. Public surface `phalcom-core` must expose

Three items the REPL needs and cannot currently reach. All three are visibility changes
or thin wrappers — **no logic moves**.

### 3.1 `unwind_to` — required by §D10

`vm/dispatch.rs:110` is `pub(crate) fn unwind_to(&mut self, stack_len: usize, frames_len: usize)`.

**Ruled: expose a named method, not `pub fn unwind_to`.** Add to `phalcom-core`'s public
VM API (`vm/api.rs` is the established home):

```rust
/// Discards all execution state at a REPL cell boundary, closing any open
/// upvalues first (U-REPL §D10).
///
/// `run_in_module`'s raw `frames.clear(); stack.clear()` is **not** equivalent:
/// `open_upvalues` is keyed by absolute value-stack index, so clearing the stack
/// beneath it aliases the previous cell's captured slots onto the next cell's
/// values — silent corruption, not a crash.
pub fn unwind_cell(&mut self) {
    self.unwind_to(0, 0);
}
```

Rationale for the wrapper over widening `unwind_to`: `unwind_to(stack_len, frames_len)`
takes two raw indices whose only correct REPL argument is `(0, 0)`, and a public
two-index unwinder invites a caller to pass something else. `unwind_cell()` has one
meaning. It also gives the doc comment a home where it will be read.

### 3.2 Chunk source registration — already public

§D2 landed `ModuleObject::push_source` (`heap/module.rs:102`) and `source_at`
(`:112`), both `pub`, and `VM::compile_closure` (`interpret.rs:148`) already appends
the cell's text and stamps `Chunk.source_id`. **Nothing to do.** Listed so an
implementer does not re-derive it.

### 3.3 Module and value access

`VM::create_module` (`vm/api.rs:83`) is `pub`. `Heap::module` / `Heap::closure`
(`heap/accessors.rs:111`, `:135`) are `pub`. `ModuleObject::declare` (`:156`) and
`define` (`:182`) are `pub`. The §S1 snapshot builder needs `globals`,
`name_to_slot`, and the class of a `Value`; verify each is reachable from outside the
crate **before** writing §05, and if one is not, add the accessor in that stage rather
than pre-emptively here.

## 4. What this stage does *not* do

- Does not call the VM from `repl.rs`. `eval` stays a stub until [§02](02-session-and-cells.md).
- Does not add the unit-kind type (§D3). That is [§02 §2.2](02-session-and-cells.md).
- Does not touch `completer.rs` or `highlighter.rs`.

Keeping this stage to "deps + three visibility items" is what makes it landable on
`main` between class units without a semantic review.

## 5. Write-set

| Path | Change |
|---|---|
| `phalcom-repl/Cargo.toml` | add three path dependencies |
| `phalcom-core/src/vm/api.rs` | add `unwind_cell` |
| `Cargo.lock` | regenerated |

**Conflict risk vs class work: none.** `vm/api.rs` is in U-CLASSNS's write-set
(`create_class`/`create_single_class` module param, inserts at `:76-77`), but the
addition here is a new method at the end of the impl block and touches no line CLASSNS
edits. If CLASSNS has already landed, rebase and re-check.

## 6. Gate

`cargo build --workspace && cargo test --workspace && cargo clippy --workspace` green;
28 suites, 0 failures. No new warnings.

Adding an unused dependency is a `cargo` no-op, not a warning — but `unwind_cell` will
be dead until §02 lands. If `dead_code` fires on it, that is expected; annotate with a
`#[allow(dead_code)]` carrying the same "will be read once the cell loop is wired up"
note `repl.rs:18` uses, and **delete the annotation in §02**. Do not leave it.
