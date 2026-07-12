# U4 — Blocks & Closures (as-built)

- **Status:** ✅ Landed — `707dc17` (`feat(u4): first-class blocks + open/closed upvalues + frame-token infra`), plus in-flight fixes `71df836` (blocks without a trailing `return`) and the review-driven runtime wiring closed in the same session. In-tree on `main`, no worktree.
- **Realizes:** [ADR-0013](../../../../adr/0013-closure-upvalues-and-frame-token-return.md) (open/closed upvalues + frame-token non-local return infrastructure); [ADR-0006](../../../../adr/0006-function-as-abstract-callable-root.md) (`Function` abstract callable root); builds on [ADR-0009/0010](../../../../adr/0009-handle-arena-heap.md) (heap handles + tagged `Value`). Spec: [blocks.md](../../blocks.md) §1–7, [functions.md](../../functions.md) §1–4.
- **Reviewer gate:** ON (load-bearing — closures can corrupt the object model). The independent `phalcom-reviewer` pass returned `request-changes` on the first cut: the front end + type scaffolding were correct but the runtime was stubbed (`block.call` returned "not wired yet", `GetUpvalue`/`SetUpvalue` raised `RuntimeError::Internal`, `Bytecode::Closure` never allocated a `BlockObject`) and it silently regressed the `example_calculator` golden. The gaps were closed in a follow-up pass and re-verified green.

## Mission
Make blocks first-class values: compile block literals to heap closures that capture free variables via **Lua-style open/closed upvalues** (open = alias the live stack slot; closed = copy the value out on scope exit, so escaping blocks and shared mutation both work), dispatch the `call`/`arity`/`name` protocol on them through the U3 selector path, and **stand up the frame-token infrastructure** (per-frame generation counter + `FrameToken`) that U10 later consumes — **without** implementing any non-local-return semantics.

## Surface / behavior
Blocks are written as braced multi-parameter literals, unbraced single-parameter expression forms, or trailing-block sugar; postfix `(…)` on any expression lowers to a `call(_:…)` send. A block value's `class` is `Block`, and `Block isA Function` is true. Blocks close over enclosing locals and `self`, and captured variables are **shared** while the scope is live.

```phalcom
let makeCounter = {
  var n = 0
  { n = n + 1; n }        // inner block captures `n` by open upvalue
}
let next = makeCounter.call()
next.call()               // 1
next.call()               // 2  (the counter escaped its home frame; upvalue promoted to Closed)

let add = { a, b => a + b }
add.call(2, 3)            // 5
add.arity                 // 2
```

## Implementation
Front end (`phalcom-ast`): new `Expr::Block`/`BlockExpr` AST node (`ast.rs`) plus parsing in `parser.rs` for the braced multi-param form, the unbraced single-param expression-only form (statement/`return` bodies rejected), and trailing-block sugar; postfix `(…)` desugars to `call(_:…)`.

Object model (`phalcom-core/src`):
- `block.rs` (new) — `BlockObject { closure: ObjRef, home_frame_token: FrameToken }`, a `Copy` struct pairing the closure handle with the token of its creating activation.
- `upvalue.rs` (new) — `Upvalue` cell, `Open(usize)` (index of a live VM stack slot) or `Closed(Value)` (value copied out after the frame exited); heap-owned so a closed upvalue outlives its frame ([ADR-0009](../../../../adr/0009-handle-arena-heap.md)).
- `closure.rs` — `ClosureObject` carries heap-owned upvalue handles (replacing the old `upvalues: Vec<Value>` / `num_upvalues: 0` stub); `callable.rs` — `Callable` carries capture **descriptors** (`is_local` + `index`) telling the VM how to materialize each cell at closure creation.
- `frame.rs` — `CallFrame` gains a monotonic `generation: u64` (minted by `VM::new_call_frame` via `next_frame_generation`) plus `CallFrame::token(frame_index) -> FrameToken`; `FrameToken { frame_index, generation }`. The `home_frame_token` field is present on `CallFrame` but only populated by U10. `CallFrame` stays `Copy`.
- `value.rs` — blocks are carried as a heap `Obj(ObjRef)` to a `BlockObject`.

Opcodes (`bytecode.rs`, executed in `vm.rs`, disassembled via `Debug`):
- `Closure(u16)` — allocate a `ClosureObject`, capture upvalues **open**, wrap in a `BlockObject` stamped with the creating frame's token.
- `GetUpvalue(u16)` / `SetUpvalue(u16)` — read/write through an upvalue cell (open → live stack slot; closed → the cell).
- `CloseUpvalue(u16)` — promote open cells to `Closed` on scope/frame exit.

Compiler (`compiler/lib.rs`): `compile_block` performs upvalue resolution (walk enclosing compiler scopes, mark captured locals, build descriptors, capture `self` as an upvalue), emits the four opcodes, and closes upvalues at scope exit.

Tower (`universe.rs`): `Function` (abstract) is installed under `Object`; `Block` is installed under `Function`, and `Method` re-parents from `Object` to `Function` so all three share the `ClosureObject` call protocol ([ADR-0006](../../../../adr/0006-function-as-abstract-callable-root.md)). `Function` must be allocated before `Block`/`Method` because `make_core_class` reads `Function`'s metaclass to wire the parallel rule.

Apply protocol (`primitive/block.rs`, wired in `universe.rs`): `arity` and `name` are getters; `call` is registered **per arity** `Method(0)..=Method(4)` (`MAX_CALL_ARITY = 4`) on both `Function` and `Block`, because dispatch keys on the arity-encoded selector rather than a single variadic entry. `block_call` resolves the closure handle via `resolve_callable`, checks arity, pushes a fresh `CallFrame`, and re-enters `VM::run_until` with the current frame count as the floor so the call returns synchronously. `callWith(_:)` (one packed argument) is a forward stub pending kernel `List`.

## Invariants & tests
- `verify_invariants()` stays green with `Function`/`Block` added — the metaclass parallel rule wires their metaclasses by the same ADR-0002 rule.
- `tests/lang.rs` `blocks()` group un-pended with 6 cases: capture, escape, shared mutation, `call`, `arity`.
- New goldens: `tests/fixtures/golden/blocks_map_reduce.ph`, `blocks_escaping_counter.ph`. Pre-existing goldens byte-identical after the runtime was correctly wired (the first-cut `example_calculator` regression was fixed before landing).
- Closure/upvalue disassembly snapshotted.

## Deviations & deferrals
- **No non-local return.** U4 ships the frame-token *infrastructure only* — no `ReturnNonLocal` opcode, no unwind logic, no non-local-return test. A `return` in a block compiled to an ordinary `Bytecode::Return` (a latent single-frame-only semantic, corrected by U10). That work is [U10 — Non-local return](10-non-local-return.md).
- **`call` protocol, not Smalltalk `value:`.** The apply selectors are `call`/`call(_:)`/`call(_:_:)`/`callWith(_:)`/`arity`/`name` per functions.md §1–2, overriding an upstream brief that assumed `value:`.
- **`callWith(_:)` stubbed** pending kernel `List` (landed later in U-LIST); tracked in [`docs/forge/DEFERRED.md`](../../../../forge/DEFERRED.md).
- **`call` arity capped at 4** (`MAX_CALL_ARITY`); higher-arity blocks are out of the pre-registered set.
- Unbraced arrow blocks are expression-only by construction, so they cannot carry `return` (blocks.md §2) — a property U10 relies on.
- See [deferred-work.md](../../deferred-work.md) for the running deferral ledger.

## Sources
- Forge work order: [`docs/forge/U4-plan.md`](../../../../forge/U4-plan.md); handoff: [`docs/forge/U4-handoff.md`](../../../../forge/U4-handoff.md); landing record: [`docs/forge/STATE.md`](../../../../forge/STATE.md) "U4 — LANDED".
- Code: `phalcom-core/src/{block,upvalue,closure,callable,frame,value,bytecode}.rs`, `phalcom-core/src/compiler/lib.rs`, `phalcom-core/src/vm.rs`, `phalcom-core/src/universe.rs`, `phalcom-core/src/primitive/block.rs`; `phalcom-ast/src/{ast,parser}.rs`.
