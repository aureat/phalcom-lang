# U-HOTPATH — Implementation spec (dispatch-loop hot path, behavior-invariant)

> Companion to [`plan.md`](plan.md) — that file has rationale + the Wren precedent. **This file
> supersedes plan.md's file/line refs** (written pre-`vm.rs` split) and **adds Change 4**, which on
> HEAD is bigger than any of plan.md's three. Written against HEAD 2026-07-14 (`2b75429`).
>
> **The whole unit is behavior-invariant.** Any golden that changes output = a bug you introduced.
> Never rebaseline a golden in this unit. There is no ADR to amend.

## 0. Path corrections vs plan.md

| plan.md ref | HEAD |
|---|---|
| `vm.rs:1182` `run_until_inner` | `phalcom-core/src/vm/dispatch.rs:389` |
| `vm.rs:1208` `match opcode` | `phalcom-core/src/vm/dispatch.rs:431` |
| `vm.rs:1566` Invoke chunk chase | `phalcom-core/src/vm/dispatch.rs:830` |
| `vm.rs:1589-1592` variadic probe | `phalcom-core/src/vm/dispatch.rs:853-856` |
| `value.rs:95` `Value::class` | `phalcom-core/src/value/mod.rs:121` |
| `value.rs:146` init-fallback `format!` | `phalcom-core/src/value/mod.rs:172` |
| `class.rs` | `phalcom-core/src/heap/class.rs` |

## 1. Change 4 (NEW, do this FIRST — biggest win, lowest risk)

**Share `Callable` instead of deep-cloning it per block literal.**

`vm/dispatch.rs:449` — every execution of `Bytecode::Closure` (i.e. **every time a block literal is
evaluated**, including inside loops) does:

```rust
let callable = self.heap.closure(template_id).callable.clone();
```

`Callable` (`phalcom-core/src/callable.rs:22-33`) owns a `Chunk`, and `Chunk`
(`phalcom-core/src/chunk.rs:7-11`) owns `Vec<Bytecode>` + `Vec<Value>` + `Vec<SourceRange>`. So this
is **three heap allocations plus a full bytecode copy per block creation**, in a loop.

**Do:** change `ClosureObject.callable` (`phalcom-core/src/heap/closure.rs:24`) to
`Rc<Callable>`, and make `dispatch.rs:449` `Rc::clone` the template's:

```rust
let callable = Rc::clone(&self.heap.closure(template_id).callable);
```

Notes for the implementer:
- `Rc`, not `Arc`: the VM is single-threaded. If the tree already forbids `Rc` somewhere, STOP and
  report rather than reaching for `Arc`.
- The compiler builds a `Callable` then allocates the template closure — wrap it in `Rc::new` at that
  one construction site (`grep -rn 'ClosureObject {' phalcom-core/src`). Fix the compile errors that
  fall out; `Rc<Callable>` derefs to `Callable`, so `closure.callable.chunk.code[ip]` still compiles
  unchanged at read sites.
- **Nothing mutates a `Callable` after construction.** Verify (`grep -rn 'callable\.' | grep -v
  '\.callable\.\(chunk\|arity\|name_sym\|max_slots\|num_upvalues\|upvalues\)'`). If something does,
  STOP and report — that would need `Rc<RefCell<>>` and is a different unit.
- `heap/trace.rs` traces closures: confirm the tracer reaches the callable's constants through the
  same path (an `Rc` deref is transparent). If it matched on the owned struct, adjust the one line.
- This also makes U-IC's per-site cache survive block re-materialization (U-IC §2.1). If U-IC has
  already landed, this change is what turns its block-body caches on — mention that in the return.

Bench with `scripts/vm_bench.rs` before/after; expect the biggest single number in this unit.

## 2. Change 3 (trivial, do second) — `Value::class` arm order

`value/mod.rs:121-158` matches `Nil, Bool, Number, Symbol, Obj`. `Obj` and `Number` are the hot
arms. Reorder the arms so `Obj` and `Number` come first, keep the fn `#[inline]`. A `match` is
exhaustive regardless of arm order — zero semantic change.

**Measure it.** LLVM very likely already does this. If the bench shows a wash, **drop the change**
and say so in the return shape. Do not keep a churn commit that buys nothing.

## 3. Change 2 (do third) — kill `String` allocs on derived-selector paths

Two sites build a `String` at runtime just to re-intern it:

- `value/mod.rs:171-173` — `format!("init {}", selector_str)` + `interner.find`, on **every**
  primary-lookup miss with a class receiver.
- `dispatch.rs:853-856` — `decode_selector(...)` (allocates `String` + `Vec`) then
  `format!("{name}(*)")` + `intern`, on **every** `Invoke` exact-probe miss.

**Do:** memoize `Symbol -> Symbol` maps on the `VM` (next to `interner`), populated lazily:

```rust
/// `sel` -> the interned `init <sel>` selector; memoizes the class-side
/// constructor-fallback probe (`Value::lookup_method`) so a miss costs a
/// hash lookup, not a `format!` + re-intern.
init_selector_cache: HashMap<Symbol, Symbol>,
/// `sel` -> the interned `<name>(*)` variadic selector, or `None` when `sel`
/// is not variadic-probe-eligible (labelled/getter/setter/subscript).
variadic_selector_cache: HashMap<Symbol, Option<Symbol>>,
```

Rules:
- The **derived symbol is a pure function of the source symbol**, so memoizing is sound forever —
  interned symbols are never reused for different text (`interner.rs:67-83`).
- The `variadic_selector_cache` must memoize the **eligibility decision too** (`dispatch.rs:854`:
  `matches!(kind, SignatureKind::Method(_)) && labels.iter().all(Option::is_none)`), i.e. cache
  `None` for ineligible selectors. That is where the `decode_selector` alloc actually goes.
- The `init_selector_cache` must preserve `find`-vs-`intern` semantics: `value/mod.rs:173` uses
  `interner.find` (does **not** intern a missing one). Cache the `Option` result — but note
  `find`ing later can succeed if the class defines `init foo` afterwards. **So: only cache the
  `Some` outcome; a `None` must re-probe.** Get this wrong and a constructor defined after the first
  miss stops resolving. (Simpler alternative that sidesteps it: cache `Symbol -> Symbol` by always
  `intern`ing the derived name, then do the ordinary hierarchy walk with it. Interning a never-used
  selector is harmless. **Prefer this.**)
- Leave the **cold dNU path** (`vm/send.rs:138-140`, `new_message`) allocating. Not hot, not in scope.

Behavior-invariant: the exact same selectors resolve, just without the transient `String`.

## 4. Change 1 (do LAST — riskiest) — hoist chunk access out of the loop

`dispatch.rs:412-415` re-derives the chunk **every instruction**:

```rust
let (opcode, source_range) = {
    let chunk = &self.heap.closure(closure_id).callable.chunk;
    (chunk.code[ip], chunk.spans[ip])
};
```

and again per-opcode at `:433`, `:444`, `:830` etc. Wren hoists `ip`/`fn`/`stackStart` into locals
and re-syncs only at frame transitions (`wren_vm.c:832-862`).

**The borrow problem:** a `&Chunk` borrowed out of `self.heap` cannot live across `&mut self` calls
(`call_method`, `stack.push` are fine, but any `&mut self` is not). Resolutions, in preference order:

- **(a) Do Change 4 first, then hoist an `Rc<Callable>` clone into a loop local.** After §1, the
  frame's callable is an `Rc` — clone it once per frame entry into a local `current: Rc<Callable>`,
  read `current.chunk.code[ip]` with no `self.heap` borrow at all, and refresh `current` at every
  frame transition. This is cheap (a refcount bump per frame push, not per instruction) and needs
  **zero `unsafe`**. **This is the recommended path and the reason Change 4 is first.**
- (b) index-based access with a cached `ObjRef` — barely better than today.
- (c) raw `*const Chunk` snapshot — **needs reviewer sign-off; do not do this without one.**

**Re-sync points — miss one and you execute the wrong bytecode.** After hoisting, `current` must be
refreshed at *every* place `self.frames` changes shape:
- `Bytecode::Invoke` / `SuperSend` → `call_method` (`vm/send.rs:19`) pushes a frame
- `Bytecode::Return` (`dispatch.rs:910`) and `ReturnNonLocal` (`:927`) pop frames
- **fiber switch** — `run_until` (`dispatch.rs:219`) and the fiber primitives swap the frame stack
  entirely. A stale hoisted callable across a fiber switch is the classic bug this unit could ship.
- any error/unwind path that re-enters the loop

The safe implementation shape: keep the hoist **inside** the loop body but derive it from
`frames.last()` only when the frame token/`closure_id` differs from the hoisted one — i.e. a
one-compare guard per instruction instead of a heap chase per instruction. Simpler to prove correct
than tracking every re-sync point by hand, and it still removes the `heap.get` + pointer chase.

If it does not measure, **drop it**. It is the riskiest change here and only worth shipping for a
real number.

## 5. Write-set (STOP-and-report if outside)

- `phalcom-core/src/heap/closure.rs` — `callable: Rc<Callable>` (Change 4).
- `phalcom-core/src/vm/dispatch.rs` — `:449` `Rc::clone`; `:853-856` alloc removal; `:412-415` hoist.
- `phalcom-core/src/vm/mod.rs` — the two memo maps + their rustdoc (Change 2).
- `phalcom-core/src/value/mod.rs` — `:121` arm order; `:171-177` init-fallback alloc removal.
- `phalcom-core/src/compiler/lib/` — only the `Rc::new` at the template-closure construction site.
- `phalcom-core/src/heap/trace.rs` — only if the tracer needs an `Rc` deref adjustment.
- **No new goldens** (behavior-invariant). **Floor: +0.**

## 6. Build order + gates

Commit per green change ([[commit-frequently]]), in this order:
**Change 4 → Change 3 → Change 2 → Change 1.** (plan.md said 3→2→1; Change 4 goes first because
Change 1's clean, `unsafe`-free resolution depends on it.)

Each commit: `cargo build && cargo test && cargo clippy --workspace` green, `cargo doc` clean
([[rust-doc-mandatory]]), **zero golden diff**, WORKTREE-VERIFY at the SHA
([[clean-checkout-verify-each-commit]]). Bench with `scripts/vm_bench.rs` per change so each number
is attributable — a batched measurement tells you nothing about which change paid.

## 7. Reviewer (ON — writer ≠ approver)

Reviewer confirms: goldens byte-identical; nothing mutates a `Callable` post-construction (Change 4's
whole soundness argument); the hoist re-syncs across **fiber switch** specifically (Change 1); the
memoized derived selectors resolve identically to the `format!` path, including the `init`-defined-
later case (§3); no `unsafe` without explicit sign-off.

## 8. Return shape

commit SHAs · Change 4: `Rc<Callable>` landed + confirmation nothing mutates a `Callable` + the
bench delta · Change 3: kept or dropped + the number that decided it · Change 2: both alloc sites
removed + which memo strategy (always-intern vs cache-`Some`-only) · Change 1: hoist shape + the
borrow resolution (a/b/c) + every re-sync point covered incl. fiber switch, **or** dropped + why ·
**zero golden diff confirmation** · any `unsafe` (expect none) · floor delta (0) · verify +
`cargo doc` tails · write-set confirm.
