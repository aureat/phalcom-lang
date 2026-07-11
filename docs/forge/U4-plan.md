# U4 — Work order: blocks / closures (Lua-style open/closed upvalues + frame-token infra)

_Self-contained implementation plan for **one** `phalcom-implementer` agent. Load-bearing unit →
independent `phalcom-reviewer` gate afterward. Grounded in **ADR-0013** (open/closed upvalues +
frame-token non-local return), **ADR-0006** (`Function` abstract callable root), and **ADR-0009/0010**
(heap handles + tagged `Value`). Spec: [`blocks.md`](../spec/blocks.md) §1–7 and
[`functions.md`](../spec/functions.md) §1–4 — the latter is authoritative on the `Value`/opcode shape._

---

## 0. Mission (one sentence)
Make blocks first-class: compile block literals to heap `ClosureObject`s that capture free variables
via **Lua-style open/closed upvalues** (open = alias the live stack slot, closed = copy on scope exit
so escaping blocks + shared mutation both work), dispatch `call`/`call(_:…)`/`arity` on them via U3
selectors, and **stand up the frame-token infrastructure** (frame pointer + generation counter) that
U10 will consume — **without** implementing non-local-return semantics.

## 1. Hard guardrails (read before writing any code)
- **This unit builds on the post-U1 substrate.** Closures, upvalue cells, block objects, and call
  frames live in the **`Heap`**; you hold them by `Copy` `ObjRef` handles and reach them through
  `heap.get(id)` / `heap.get_mut(id)`. There is **no** `Rc<RefCell>`/`PhRef` and no per-object
  `RefCell`. Methods take `&Heap`/`&mut Heap`. If U1 has not landed on your base, **STOP and report** —
  do not build on the old substrate.
- **This unit builds on the post-U2 tower.** `Function` (abstract) and `Block` get installed under
  `Object` as siblings of the already-existing `Method` (functions.md §1, ADR-0006). The metaclass
  parallel rule + `verify_invariants()` are U2's; adding two kernel classes must keep both green — do
  **not** touch the parallel-superclass wiring, only hang new classes off the corrected tower.
- **Do NOT implement non-local return.** `^`/`return`-unwinds-to-home-method is **U10**. U4 lays the
  *infrastructure only*: the generation counter on `CallFrame`, the `home_frame_token` field on
  `BlockObject`, and the token-construction helper. A `return` inside a braced block is **out of scope
  for U4** — U4 does not emit a non-local-return opcode and ships no test that exercises one. (See §4
  "U4/U10 boundary".) Keep unbraced arrows expression-only so they *cannot* carry `return` (blocks §2).
- **Realize, don't re-litigate.** The capture model (open/closed upvalues), the token shape (ptr +
  generation), and the `Value::Block(BlockObject{closure, token})` representation are **decided**
  (ADR-0013, functions.md §2). Implement them; do not survey alternatives.
- **Block invocation selector is `call`, not `value:`.** functions.md §1–2 is source of truth: the
  apply protocol is `call` / `call(_:)` / `call(_:_:)` / `callWith(_:)` / `arity` / `name`, and
  `f(a, b)` desugars to `f.call(a, b)`. (An upstream brief said Smalltalk `value:`; the spec overrides
  it. Do not introduce `value:`.)
- Stay inside the write-set (§3). If forced outside it, **STOP and report a conflict**; append
  out-of-scope ideas to [`DEFERRED.md`](DEFERRED.md). **Do not self-approve.**

## 2. Preconditions (verify first; do not assume)
- Runs **in its own worktree off `main`** (`feat/u4-blocks`), branched from the committed U1(+U2) green
  base. Confirm `./scripts/verify.sh` is green before the first edit (baseline).
- **Scope correction — verify on HEAD.** Block literals are **not yet in the AST or parser.** `Expr`
  (phalcom-ast/src/ast.rs) has no `Block`/arrow arm; `parse_*` (phalcom-ast/src/parser.rs) consumes
  `FatArrow` only in *method/getter body* position (~L648), not as an expression literal. So U4 must
  **add** block parsing, not merely lower an existing node. Confirm this still holds on your base
  before planning the parser edit.
- Confirm the closure substrate: `compile_block` (compiler/lib.rs) already emits a `ClosureObject` for
  method/getter bodies but hardcodes `num_upvalues: 0` / `upvalues: Vec::new()` (a `// TODO`). U4
  replaces that stub with real upvalue resolution. `bytecode.rs` has **no** `Closure`/`GetUpvalue`/
  `SetUpvalue` opcode yet — confirm.
- graphify-first: `graphify explain "ClosureObject"`, `graphify affected "CallFrame"`,
  `graphify affected "compile_block"` on the actual HEAD before reading source.

## 3. Confirmed write-set
| File | Why it's in scope |
|---|---|
| `phalcom-ast/src/ast.rs` | New `Expr::Block { params, body, expr_body: bool, range }` node. |
| `phalcom-ast/src/parser.rs` | Parse braced `{ p, q => … }`, unbraced single-param `n => e` (expr-only), trailing-block sugar (block after an arg list = final argument), and postfix `(…)` lowering to a `call(_:…)` send (functions.md §1). |
| `phalcom-ast/src/{token,lexer}.rs` | **Only if a token is missing.** `FatArrow` and `{`/`}` already exist — expect no change; touch only if the braced/trailing form needs a lexer tweak, and justify it. |
| `phalcom-core/src/value.rs` | `Value::Block` arm — a heap `Obj`/`ObjRef` to a `BlockObject` (functions.md §2). |
| `phalcom-core/src/block.rs` **(NEW)** | `BlockObject { closure: ObjRef, home_frame_token }` — the closure + frame-token pair (functions.md §2). |
| `phalcom-core/src/upvalue.rs` **(NEW)** | The `Upvalue` cell: `Open(stack_index)` / `Closed(Value)`, heap-owned (ADR-0013 §Decision, ADR-0009). |
| `phalcom-core/src/closure.rs` | `ClosureObject` gains the captured-upvalue handles (heap-based, not `Vec<Value>`); wraps the shared `Callable`. |
| `phalcom-core/src/callable.rs` | `Callable` gains upvalue **descriptors** (`is_local: bool` + `index`) so the runtime knows how to capture at `Closure` execution. |
| `phalcom-core/src/frame.rs` | `CallFrame` gains a **generation counter** (frame-token infra); helper to mint a `FrameToken`. **No unwind logic** (that's U10). |
| `phalcom-core/src/bytecode.rs` | New `Closure(u16)`, `GetUpvalue(u16)`, `SetUpvalue(u16)`, `CloseUpvalue(u16)` opcodes. **No return-opcode change.** |
| `phalcom-core/src/chunk.rs` | If upvalue descriptors ride on the chunk/`Callable` constant, thread them through `add_constant`/disasm. |
| `phalcom-core/src/compiler/lib.rs` | Compile `Expr::Block`; **upvalue resolution** (walk enclosing compilers, mark local captured, build descriptors); emit `Closure`/`GetUpvalue`/`SetUpvalue`/`CloseUpvalue`; capture `self` as an upvalue (functions.md §2). Replace the `num_upvalues: 0` stub. |
| `phalcom-core/src/vm.rs` | Execute `Closure` (allocate `BlockObject`, capture upvalues **open**, stamp the frame token), `GetUpvalue`/`SetUpvalue`, `CloseUpvalue` on scope/frame exit; dispatch `call`/`call(_:…)`/`arity`/`name` to blocks; stamp each pushed `CallFrame` with a fresh generation. |
| `phalcom-core/src/primitive/block.rs` **(NEW)** + `primitive/mod.rs` | Native `call`/`callWith(_:)`/`arity`/`name` for `Block`/`Function` (register in `mod.rs`). `callWith(_:)` needs `List` — if `List` isn't in the kernel yet, stub it to `ArgumentError`/defer and note in DEFERRED. |
| `phalcom-core/src/{universe,class}.rs` | Install `Function` (abstract) + `Block` kernel classes under `Object`, siblings of `Method` (functions.md §1). |
| `phalcom-core/core/core.ph` | Only if any `Function`/`Block` surface method is best expressed in Phalcom (e.g. `callWith`); otherwise leave untouched. |
| `phalcom-core/bin/phalcom/disasm.rs` | Disassemble the four new opcodes. |
| `phalcom-core/tests/lang.rs` | Un-pend the `blocks()` acceptance group (capture / escape / shared mutation / `call` / `arity`). **Do not** add non-local-return cases (U10). |
| `phalcom-core/tests/fixtures/golden/` + `golden.rs` | New block golden(s): map/reduce-style capture, an escaping counter closure. |

## 4. Design decisions (ADR-0013 / ADR-0006 / functions.md — realize, don't re-litigate)
- **`Value::Block`.** Blocks are first-class values. A `BlockObject` (heap, new `block.rs`) wraps the
  `ClosureObject` handle plus a `home_frame_token`. Under U1's tagged `Value` this is carried as
  `Obj(ObjRef)` to a heap `BlockObject` (or a dedicated `Value::Block(ObjRef)` arm if that sharpens
  dispatch — pick one, justify in the `//!` doc, cite ADR-0010). `x.class` for a block is `Block`;
  `isA(Function)` is true (functions.md §1).
- **Upvalues — open/closed (ADR-0013).** An `Upvalue` cell is `Open(stack_index)` or `Closed(Value)`,
  **owned by the heap** so a closed upvalue outlives its frame (ADR-0009). While open, the block and
  the enclosing scope share the *same* cell → mutation is shared (blocks §5). On scope/frame exit,
  `CloseUpvalue` copies the live slot value into the cell and flips it to `Closed`. The compiler emits
  capture **descriptors** (`is_local` + `index`) on the `Callable`; the VM materializes cells at
  `Closure` execution. `self` is captured as an ordinary upvalue (functions.md §2).
- **Frame token — INFRASTRUCTURE ONLY.** `CallFrame` gains a monotonically-assigned **generation
  counter**; a `FrameToken` = (frame pointer/index, generation). `BlockObject` stores the token of the
  activation it was created in. U4 mints and stores the token and provides the compare helper — it does
  **not** unwind. (ADR-0013 §Decision; the `raw pointer, no generation` alternative is *rejected* there
  — always carry the generation.)
- **Callable tower (ADR-0006, functions.md §1).** `Function` is abstract (no direct instances);
  `Block` and `Method` are siblings sharing `ClosureObject`. Install `Function` + `Block`; reuse the
  existing `Method`. Apply sugar: postfix `(…)` on any expression lowers to `call(_:…)` — the parser's
  job, not a new "call" value path.
- **U4/U10 boundary (keep crisp).** U4 = closures + open/closed upvalues + block-`call` machinery +
  frame-token *infrastructure*. U10 = the `^`/`return`-inside-block *semantics* that consume the token
  (new `ReturnNonLocal` opcode, unwind loop, `DeadFrameError`). U4 ships zero non-local-return behavior
  and zero non-local-return tests; its green gate rests only on capture/escape/shared-mutation/`call`/
  `arity`. If during implementation you find a `return` inside a braced block reaches the compiler,
  compile the block body's `return` as a normal expression-position error/deferral — **do not** invent
  the non-local path; note it for U10. **No BLOCKED-ON-DECISION** items in this unit — every choice
  above is pinned by ADR-0013 / ADR-0006 / functions.md §2.

## 5. Build order (land as one coherent, reviewable diff)
1. **Front end** — `Expr::Block` in `ast.rs`; parse braced multi-param, unbraced single-param
   expression-only (reject `n, x => …` per blocks §3 and reject statement/`return` bodies in the
   unbraced form per blocks §2), and trailing-block sugar (blocks §4). Confirm postfix `(…)` lowers to
   `call(_:…)`. Parser unit tests for each form + each rejection.
2. **Value + heap objects** — `upvalue.rs` (`Upvalue` cell), `block.rs` (`BlockObject`), `Value::Block`
   arm, `ClosureObject` upvalue handles, `Callable` upvalue descriptors. Full rustdoc, cite ADR-0013/0009.
3. **Opcodes** — `Closure`/`GetUpvalue`/`SetUpvalue`/`CloseUpvalue` in `bytecode.rs` + `disasm.rs`.
4. **Compiler** — `compile_block` upvalue resolution (walk enclosing scopes, capture `self`), emit the
   new opcodes, replace the `num_upvalues: 0` stub, close upvalues at scope exit.
5. **VM** — execute the four opcodes; frame push stamps a fresh generation; dispatch `call`/`arity`/
   `name`/`callWith` to `BlockObject` via the U3 selector path (`primitive/block.rs`).
6. **Tower** — install `Function` + `Block` in `universe.rs`/`class.rs`; keep `verify_invariants()` green.
7. **Frame-token infra** — generation counter on `CallFrame` + `FrameToken` mint/compare helper in
   `frame.rs`, stored on `BlockObject`. Leave a `// U10:` marker where the unwind will consume it.
8. **Tests** — un-pend `blocks()` in `lang.rs` (capture/escape/shared-mutation/`call`/`arity`); add
   block goldens; snapshot the closure/upvalue disassembly.

## 6. Fold-in cleanup
None assigned. If you find dead closure stubs (e.g. the old `num_upvalues: 0` TODO path) fully
superseded, remove them within the write-set and note it; anything outside → `DEFERRED.md`.

## 7. Mandatory rules
- **Docs** ([`docs/rust-documentation-guidelines.md`](../rust-documentation-guidelines.md)): `//!` on
  new `block`/`upvalue` modules + every touched module; `///` on every public item (`BlockObject`,
  `Upvalue` + both variants, `FrameToken`, each new opcode, each migrated method) with `# Panics`/
  `# Safety` where applicable, intra-doc links, and ADR-0013/0006/0009 citations. `cargo doc
  --workspace --no-deps` adds **no new warnings**.
- **Green gate:** `./scripts/verify.sh` exits 0 (build + test + clippy + golden + `lang.rs` +
  invariants). The 2 spec-target invariants stay as U2 left them. Golden output byte-identical for
  pre-existing goldens; new block goldens added deliberately. No new clippy warnings; fix pre-existing
  ones in files you rewrite.
- **Best practices:** `rust-best-practices` skill. The open→closed promotion is the standing borrow/
  lifetime hazard — the upvalue cell's owner is the **closure/heap**, never the frame; never alias a
  popped stack slot. Any `unsafe` needs a `// SAFETY:` note and `rust-sanitizers-miri`.

## 8. Return contract (to the reviewer, not self-approval)
Report: `Value::Block`/`BlockObject` layout + upvalue-cell design · the four opcodes + their VM
semantics · how open→closed promotion keeps escaping blocks + shared mutation correct (with the
escaping-counter golden) · the frame-token infra you stood up and the **explicit statement that no
non-local-return behavior was implemented** (U10 owns it) · `Function`/`Block` tower install +
`verify_invariants()` still green · files changed · `verify.sh` + `cargo doc` tails · any `DEFERRED.md`
entries. A `phalcom-reviewer` independently verifies capture correctness, borrow-hazard absence, the
U4/U10 boundary, and the green gate.
