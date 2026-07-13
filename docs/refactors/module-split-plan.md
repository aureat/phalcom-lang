# Module split plan: `compiler/lib.rs`, `vm.rs`, `universe.rs`

## Why

Three files in `phalcom-core/src` dominate the crate by line count and are the
files most likely to be touched by more than one concurrent session/worktree
at once (forge units routinely add a compiler pass, an opcode, or a primitive
registration). Splitting them into smaller, concern-scoped modules:

- shrinks the diff surface any one unit's work touches, reducing merge
  conflicts across worktrees;
- keeps each file's job legible without reading 700-1600 lines to find the
  seam.

Everything else in `phalcom-core/src` is already small (most files under 200
lines, following one-concern-per-file) — this plan only touches the three
outliers. No behavior changes; this is a pure module reorganization
(`impl` blocks moved, no logic edited).

**Precedent already in the codebase:** `compiler/inliner.rs` already contains
a second `impl<'vm> Compiler<'vm> { ... }` block alongside `compiler/lib.rs`'s
own `impl<'vm> Compiler<'vm>`. Rust allows splitting a single type's inherent
`impl` across multiple files within the same module tree — this plan is that
pattern applied systematically.

## Ground rule: split without forcing it

Only pull a piece out when it is a legible, mostly-self-contained concern.
Tightly coupled machinery (e.g. the opcode dispatch loop, or local/upvalue
resolution used by nearly every compiler method) stays together even if long
— splitting mid-algorithm just adds `mod.rs` glue with no real decoupling.

## Sequencing / collision note

`compiler/lib.rs` is the one file with a **known collision risk right now**:
the in-flight `u-iter-work` worktree (background U-ITER implementer,
for/break/continue) compiles through this exact file (loop-context stack,
`Statement`/`Expr` compilation). Do **not** start the `compiler/lib.rs` split
until U-ITER lands and merges to `main`. `vm.rs` and `universe.rs` have no
known overlapping in-flight work and can be split independently, in any
order, before or after U-ITER lands.

---

## 1. `vm.rs` (1800 lines) → `vm/`

Turn `vm.rs` into `vm/mod.rs` (struct + field docs only) plus:

| New file | Contents | Why this seam |
|---|---|---|
| `vm/mod.rs` | `pub struct ClassLayout`, `pub struct VM` (fields + doc comments only), `mod` declarations | The shared type — every other file needs it in scope; keep it minimal and stable so the other files don't churn it. |
| `vm/bootstrap.rs` | `impl VM { fn new, fn run_core_module, fn install_core }` | Kernel bring-up: allocates the heap, wires the metaclass tower, stamps fixed-slot layouts (`Some`/`Message`/`Error`/`MessageNotUnderstood`), compiles `core.ph`. Runs once, at startup, and is edited only when the kernel bootstrap sequence itself changes — rarely touched alongside opcode/dispatch work. |
| `vm/api.rs` | `impl VM { get_or_intern, resolve_symbol, alloc_string_value, create_single_class, create_class, create_module, register_path, register_source, get_module, get_module_from_str, define_global }` | The runtime API surface other modules (compiler, primitives) call into. Grows independently as new host-callable helpers are added; distinct concern from the dispatch loop itself. |
| `vm/send.rs` | `impl VM { call_method, new_message, forward_does_not_understand, send_dynamic, invoke_method_object }` | Message-send machinery: method resolution → call, `doesNotUnderstand` forwarding, reflective `send_dynamic`/`invokeOn`. One coherent concern (method-lookup.md §2), separate from raw bytecode dispatch even though `run_until` calls into it. |
| `vm/dispatch.rs` | `impl VM { run, run_until, apply_jump_offset, current_frame_token, capture_upvalue, close_upvalues_from, surface_absence, none_value, new_call_frame, pop, runtime_error, compiler_error }` + the full opcode `match` | The hot loop. Kept as one file even though it's long — it's the single most tightly-coupled piece (every opcode handler shares the same frame/stack/heap state machine) and splitting it further would just scatter one algorithm across files for no decoupling benefit. This is the "don't force it" file. |

Net effect: `vm/dispatch.rs` will still be sizeable (the opcode match alone is
~500 lines), but it's now the *only* long file in this crate, and its length
is inherent to the algorithm rather than accumulated unrelated concerns.

## 2. `universe.rs` (969 lines) → `universe/`

| New file | Contents | Why this seam |
|---|---|---|
| `universe/mod.rs` | `pub struct Universe` (fields), `impl Universe { fn new, fn note_method_installed }`, `mod` declarations | Core struct + the two small methods that don't obviously belong elsewhere. |
| `universe/core_classes.rs` | `pub struct CoreClasses`, `impl Universe { fn create_core_classes }`, `fn make_core_class` (private helper) | Building the metaclass tower's class *rows* (allocate-then-patch bootstrap, ADR-0002/ADR-0009). Self-contained: reads/writes only the heap and the class handles it's constructing. |
| `universe/primitives.rs` | `impl Universe { fn install_primitives }` | Registers every native Rust method onto its core class. High-churn: adding a new primitive method is one of the most common unit-of-work edits in this codebase, so isolating it means that work never touches bootstrap or invariant-checking code. |
| `universe/invariants.rs` | `impl Universe { fn verify_invariants }` | The kernel-soundness checker (object-model.md §5-6). Read-only over `&Heap`; a fully separate concern from both construction and primitive registration, and changed only when the invariant set itself changes. |

## 3. `compiler/lib.rs` (2623 lines) → `compiler/`

**U-ITER landed on main — gate clear.**

| New file | Contents | Why this seam |
|---|---|---|
| `compiler/error.rs` | `pub enum CompilerError` (all variants + docs) | Zero coupling to the rest of the compiler beyond the type itself; already reads as a standalone diagnostic catalog. |
| `compiler/state.rs` | `struct Local`, `pub(crate) struct FunctionState` (+ `impl FunctionState::new`) | The per-function compilation state records. Used everywhere but are pure data + one constructor — no reason to keep them inline with `Compiler`'s logic. |
| `compiler/scope.rs` | `impl<'vm> Compiler<'vm> { emit, add_constant, emit_self, begin_scope, end_scope, add_local, resolve_local, resolve_local_in, resolve_upvalue, resolve_upvalue_in, add_upvalue, emit_operator_send, compile_super_send }` | The local/upvalue/scope resolution machinery — tightly coupled to `FunctionState`'s internals and to each other (resolve_upvalue_in recurses through resolve_local_in), so it stays as one unit, just pulled out of the 1644-line file. This is the "don't force it, but do carve it out" middle ground. |
| `compiler/class_decl.rs` | `impl<'vm> Compiler<'vm> { }`'s `Statement::Class` handling, extracted into `fn compile_class(&mut self, class_def: ClassDef, range) -> Result<(), CompilerError>`; plus `fn collect_assigned_fields`, `fn collect_assigned_fields_stmt` | The single largest chunk (~400 lines): class layout computation (superclass field-count resolution, own instance/static field collection passes) and member compilation (method/getter/setter/construct). This is also the piece most likely to be touched again soon (U-INH follow-ons, further class-system work per the `phalcom-two-track-roadmap` memory) — isolating it means that work has its own file. |
| `compiler/expr.rs` | `impl<'vm> Compiler<'vm> { compile_expr, compile_expr_want }` (the full `Expr` match) + `fn branch_condition_of, fn is_option_literal, fn wrap_expr_as_lazy_block, fn binary_op_selector_name` | Expression lowering — a second large, coherent concern distinct from class-declaration and scope handling. The `Expr::MethodCall`/`GetProperty` super-send interception and the sacred-inliner dispatch live here since they're expression-shape decisions. |
| `compiler/lib.rs` (remainder) | `pub(crate) struct Compiler<'vm>` (fields), `impl<'vm> Compiler<'vm> { new, compile, compile_block, compile_statement_with_pop_control }`, `mod` declarations, `#[cfg(test)] mod tests` | The struct itself, top-level `compile()`/`compile_block()` orchestration, and the statement dispatcher that delegates `Statement::Class` to `class_decl::compile_class`. Kept minimal so it doesn't itself become a collision point. |

`compiler/inliner.rs` is untouched — it already follows this exact
one-concern-per-file, `impl Compiler` split pattern and needs no rework.

## Mechanics (for whoever executes this)

For each target file:

1. `mkdir` the new subdirectory (e.g. `vm/`), move the existing file to
   `vm/mod.rs` (or, for `compiler/`, split directly since `compiler/mod.rs`
   already exists as the 2-line module declarator — check whether it needs
   merging with the new `compiler/lib.rs` remainder or can stay separate).
2. Cut each seam's methods into its own file as a fresh `impl<'vm> Compiler<'vm>`
   / `impl VM` / `impl Universe` block — copy first, verify, then delete from
   the original.
3. Add `mod bootstrap; mod api; mod send; mod dispatch;` (etc.) to the
   remaining root file. No `pub(crate) use` re-exports should be needed:
   sibling modules in the same tree see private items of a shared ancestor
   only through `pub(crate)`/`pub(super)` — check field/method visibility on
   `VM`/`Compiler`/`Universe` structs as each file is split out, since
   moving a method across a module boundary can turn a private-field access
   into a compile error that needs `pub(crate)` added.
4. After each single-seam move: `cargo build -p phalcom-core` and
   `cargo test -p phalcom-core` before moving to the next seam — land the
   split as several small green commits, not one big-bang rewrite (existing
   project convention: commit per green checkpoint).
5. `cargo doc --workspace --no-deps` clean at the end — every item split out
   keeps its existing rustdoc verbatim; this repo requires doc coverage on
   every public item (see `docs/rust-documentation-guidelines.md`), and nothing
   here is deleting or adding public API, just relocating it.
6. Re-run `graphify update . --no-cluster` once the split lands so the
   knowledge graph reflects the new file boundaries.

## Suggested order

1. `universe.rs` split (no dependents on file boundaries, lowest risk).
2. `vm.rs` split (touches `compiler/lib.rs` only via `crate::vm::ClassLayout`
   and `crate::vm::VM` — neither's path changes, so this is safe independent
   of the `compiler/lib.rs` work).
3. `compiler/lib.rs` split — once U-ITER's worktree has merged to `main`.
