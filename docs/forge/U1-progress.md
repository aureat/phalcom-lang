# U1 progress — slice handoff log

Authority: `docs/forge/U1-plan.md` (§0–§8). Behavior-preserving; do NOT fix F2; fold DEFERRED #1.
Rule: each slice = one fresh subagent, bounded scope, commit a checkpoint, update this file, STOP.
Never grind one agent to huge context.

## Slice 1 — DONE (committed `be1e183`, WIP, NOT green)
Built new types + migrated object structs. Touched: heap.rs (NEW), value.rs, class, closure,
frame, instance, method, module, nil, boolean, string, universe, primitive/{class,mod,number,object,string,symbol}, Cargo.{toml,lock}.

## Baseline after slice 1: 201 compile errors. Dominant clusters:
- 84 × `no field heap on VM` → VM must own `Heap`; `heap` module not wired into crate root.
- 39 × `heap` unresolved import / not in crate root → declare `mod heap;` + re-exports.
- 27 × `Value::Class/String/Method/Instance` gone → migrate to new tagged repr (`Value::Obj(ObjRef)`); define/fix helpers (`alloc_string_value`, `string_from`, `set_class_owned`).
- 25 × `.borrow()` on `String` → primitives still borrow migrated immediates.
- 3 × `lalrpop_util` → DEFERRED #1 not yet removed.

## Slice 2 — DONE (committed `326170a`, WIP, not yet green)
Wired `mod heap` into the crate root; gave `VM` an owned `heap: Heap` field + init via
`Universe::new(&mut heap)`; rewrote `vm.rs` end-to-end for the handle/heap world (Copy
`CallFrame`s in a plain `Vec`, heap-threaded dispatch loop, `create_single_class`/`create_class`
allocate-then-patch, `install_core` exposing classes as `Value::Obj(ClassId)`); migrated every
`Value::Class/String/Method/Instance` reference in the vm/universe/value cluster to
`Value::Obj(ObjRef)`; added the `VM::alloc_string_value` helper the migrated primitives call.
`universe.rs` bootstrap (allocate-then-patch, F2 observationally unchanged) was already done in
slice 1 — verified coherent. `==`/`!=` now use `Value::value_eq` (heap-aware string equality).
Touched: `lib.rs` (declare `mod heap`), `vm.rs` (rewrite), `docs/forge/U1-progress.md`.

## Baseline after slice 2: 19 compile errors (down from 201). Remaining clusters (all slice 3/4):
- 17 × `compiler/lib.rs` → `Value::Method`/`Value::Class`/`Value::string_from` constant pool +
  `PhRef`/`.borrow()` threading (slice 3).
- 7 × `interpret.rs` → `compile_closure`/`run_in_module` still build `PhRef` frames/closures (slice 3).
- 3 × `lalrpop_util` unresolved → DEFERRED #1 not yet folded (slice 4).
- vm.rs / universe.rs / value.rs / migrated primitives: **0 errors** (cluster coherent).

## Slice 3 — DONE (committed `605df4e`, WIP, not yet green)
Threaded the remaining pre-heap code into the handle/heap world. Workspace now compiles
**0 errors** (`cargo build --workspace 2>&1 | grep -cE '^error'` = 0), down from 19.

### Constant-pool design decision (resolved here)
The brief's *preferred* approach was heap-free descriptor constants materialized at load
time. **I deviated** and had the compiler allocate heap objects directly, storing `Value::Obj`
handles in the constant pool (`Chunk.constants` stays `Vec<Value>`, unchanged). Rationale:
the brief's precondition ("the compiler must stay heap-free") is already false in this codebase
— `Compiler` holds `&mut VM` and thus the `Heap`, interner and universe. A descriptor enum +
load-time materialization pass would be a strictly larger, riskier refactor (new `Chunk` repr,
new VM `Constant`/`Method` handling, disasm changes, a materialization pass) for zero behavioral
benefit in a compile-clean slice — "clearly worse" per the brief. So:
- string literals → `self.vm.alloc_string_value(value)` → `Value::Obj(str_id)`;
- method defs → `self.vm.heap.alloc(Object::Method(..))` → `Value::Obj(method_id)`, wrapping a
  closure allocated by `compile_block` (`self.vm.heap.alloc(Object::Closure(..))` → `ObjRef`);
- superclass → `Value::Obj(object_class)` (already a `ClassId`/`ObjRef`).
The VM's existing `Constant`/`Method`/`Class` handlers already expect `Value::Obj(..)`, so no
runtime change was needed. Decoupling the compiler from the VM (true heap-free compiler) is left
as a future unit. Cites ADR-0009 (heap) / ADR-0010 (tagged Value). Verified end-to-end: heap
string constants concat + print correctly (`"hi " + "there"` → `hi there`).

### Files touched (slice 3)
- `compiler/lib.rs`: `Compiler.module`/`compile`/`compile_block` now use `ObjRef`; constants
  materialized on the heap; **deleted dead `CompilerError::ParseError` variant + its
  `lalrpop_util` `From` impl** (this removed all 3 lalrpop errors early — they referenced an
  unlinked crate and blocked lib.rs, which I was editing; permitted by the brief). Added crate
  `//!` + item docs.
- `interpret.rs`: `compile_closure`/`run_in_module`/`interpret_source` take/return `ObjRef`;
  `CallFrame` pushed by value (no `phref_new`).
- `bin/phalcom/disasm.rs`: reads the chunk via `vm.heap.closure(closure)`.
- `phalcom-common/src/lib.rs`: **retired** `PhRef`/`PhWeakRef`/`MaybeWeak`/`phref_new`/
  `phref_weak`; updated the "What is NOT here" doc to point at the handle world.
  `ObjRef` lives in `phalcom-core/src/heap.rs` (via `slotmap` `new_key_type!`).
- Deleted orphaned `phalcom-common/src/refs.rs` (was never `mod`-declared).
- `chunk.rs`: **unchanged** — constant pool stays `Vec<Value>` per the decision above.
- Primitives (`module/method/nil/boolean/system`): **no change needed** — already migrated in
  slices 1–2 (0 errors, no stale `.borrow()`/`Value::*`).

### Left for slice 4
- `tests/invariants.rs` still imports the now-retired `PhRef` → migrate it to the `ObjRef`/heap
  API (it does not break `cargo build --workspace`; integration tests aren't built there, but it
  breaks `cargo test`). Keep the 2 planned `#[ignore]`s.
- DEFERRED #1: the dead `CompilerError::ParseError`/`lalrpop_util` is already gone; confirm no
  `lalrpop_util` remains in `Cargo.toml`/`Cargo.lock` and close the item.
- Remaining warnings to clear for the green gate: unused `mut last_statement` + `mut self`
  hints in `compiler/lib.rs`, dead `format_stack_trace` in `vm.rs`, an unused `PhError` in a bin.
- Full rustdoc pass, `./scripts/verify.sh` green + byte-identical goldens, `cargo doc` clean.

## Slice 4 — DONE (committed `f9a4508`, GREEN)
`./scripts/verify.sh` = **exit 0, "all lanes green"** (build + full test + clippy +
golden byte-identical + object-model invariants); `cargo doc --workspace --no-deps` clean.

- **`tests/invariants.rs`** migrated off the retired `PhRef<ClassObject>` to the `ObjRef`/heap
  API: class identity is `==` on `Copy` handles (was `Rc::ptr_eq`), metaclass/superclass links
  read through `Heap::class(..).class` / `.superclass`. The 2 spec-target invariants stay
  `#[ignore]`d (parallel-superclass rule + `Behavior` class — U2's, ADR 0002). 7 active pass.
- **DEFERRED #1 CLOSED:** `lalrpop-util` confirmed absent from `phalcom-core/Cargo.toml` and
  `Cargo.lock`; the dead `CompilerError::ParseError` variant + `From<lalrpop_util::ParseError>`
  impl (deleted in slice 3) confirmed gone workspace-wide. Register row removed (moved to
  `_Closed:_`), new low-rank entry filed for a pre-existing `error.rs:30` unused-lifetime
  clippy warning that is on `main` and outside U1's write-set (untouched).
- **Warnings cleared** in U1-rewritten files: dead `last_statement` (compiler/lib.rs), dead
  `format_stack_trace` + now-unused `CallContext` import (vm.rs), interpret.rs clippy nits
  (needless borrows, redundant closure, `map_err`→`inspect_err`, `Default` impl),
  `should_implement_trait` allow on `StringObject::from_str`. Remaining clippy warning:
  the one pre-existing `error.rs:30` item above (verify.sh does not `-D warnings`; deferred).
- **Docs:** fixed 3 redundant explicit intra-doc link targets (vm.rs ×2, universe.rs ×1),
  added the `interpret` module `//!` and docs on the migrated public methods
  (`compile_closure` / `run_in_module` / `interpret_source`). `cargo doc` warning-free.

Behavior-preserving: goldens byte-identical to `main`; F2 metaclass tower untouched
(observationally unchanged).

## Remaining slices (planned)
- **Slice 3:** ~~thread the rest — chunk.rs, interpret.rs, compiler/lib.rs, disasm.rs, remaining primitives (module/method/nil/boolean/system), retire `phalcom-common/refs.rs` PhRef. Get workspace compiling clean.~~ **DONE (see above).**
- **Slice 4:** ~~migrate `tests/invariants.rs` to new API (keep 2 `#[ignore]`s); fold DEFERRED #1 (lalrpop dep + `CompilerError::ParseError`); full rustdoc; `./scripts/verify.sh` green + byte-identical goldens; `cargo doc` clean. Commit green.~~ **DONE (see above).**

## Reviewer must-fix (resolved)
- Reviewer BLOCK: derived `PartialEq` fallback in `Value::value_eq` changed `==`/`!=` for `Symbol` and `Module` pairs vs `main` (`Symbol.new("foo") == Symbol.new("foo")` printed `true`; `main` prints `false`). Fixed by making `value_eq` match all variant pairs explicitly (Symbol pair and Module-Obj pair → `false`), no longer delegating to the structural derive. Repro now matches `main`; `verify.sh` all lanes green. See SHA below.
