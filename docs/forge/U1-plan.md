# U1 — Work order: handle/arena heap + tagged `Value` (dispatch-ready)

_Self-contained implementation plan for **one** `phalcom-implementer` agent. Supersedes the staging
notes in [`U1-heap-brief.md`](U1-heap-brief.md) with a confirmed write-set + a concrete build order.
Load-bearing unit → independent `phalcom-reviewer` gate afterward. Grounded in **ADR-0009** (heap)
and **ADR-0010** (Value repr); STATE.md ADR mapping is authoritative (PLAN.md's 0008–0012 are stale)._

---

## 0. Mission (one sentence)
Migrate the Phalcom object graph from `Rc<RefCell<T>>` handles (`PhRef`) to a central **handle/arena
heap** with `Copy` handles, and make `Value` a tagged `enum` — **behavior-preserving**: every golden
program and every currently-passing invariant test stays green with byte-identical output.

## 1. Hard guardrails (read before writing any code)
- **This is a representation migration, not a feature or a bug-fix.** Same language semantics, new substrate.
- **Do NOT fix the metaclass parallel-superclass bug (F2/F5/F6) — that is U2.** Preserve today's
  *observable* tower wiring exactly. ADR-0009 replaces ADR-0002's `Rc::new_cyclic` mechanism with
  "allocate handles, then patch their fields" — implement the bootstrap that way, but the resulting
  wiring must be observationally identical to today.
- **Do NOT touch other live bugs** (F1 swallowed `Result` in `vm.rs:506`, F4 `object_name`). They
  belong to later units. Migrating their code to handles is fine; changing their behavior is not.
- The two `#[ignore]`d spec-target invariants in `tests/invariants.rs` **stay ignored** (U2 owns them).
- Stay inside the write-set (§3). If forced outside it, **STOP and report a conflict**; append
  out-of-scope ideas to [`DEFERRED.md`](DEFERRED.md). **Do not self-approve.**

## 2. Preconditions (verify first; do not assume)
- Runs **in-tree on `feat/classes`**, on the committed green base — *not* in a worktree. (U1's
  write-set is nearly all of `phalcom-core/src`, so it runs alone; worktree isolation buys nothing
  and the uncommitted-base seeding hazard makes it costly. Per STATE.md "Worktree seeding hazard".)
- Confirm `./scripts/verify.sh` is green before the first edit (baseline). It was green at plan time.
- Re-run `graphify affected "PhRef"` and `graphify affected "Value"` on the actual HEAD to confirm
  nothing new references them beyond §3. (graphify-first: `graphify explain "<type>"` before reading source.)

## 3. Confirmed write-set (from `graphify affected` on HEAD — includes 3 files the brief missed)
| File | Why it's in scope |
|---|---|
| `phalcom-common/src/refs.rs` (+ `lib.rs` re-exports) | Home of `PhRef`/`phref_new`/`phref_weak` — replace or repurpose. |
| `phalcom-core/src/heap.rs` **(NEW)** | The `Heap` + `Copy` handle types. |
| `phalcom-core/src/value.rs` | The tagged `Value` enum. |
| `phalcom-core/src/{class,instance,closure,frame,method,module,universe}.rs` | Object structs → handle-based; interior mutability moves into the heap. |
| `phalcom-core/src/{nil,boolean,string}.rs` | Immediate/primitive value types touched via `Value`. |
| `phalcom-core/src/chunk.rs` | Constant pool holds `Value` (`add_constant`). ⟵ *brief missed this* |
| `phalcom-core/src/interpret.rs` | `compile_closure`/`run_in_module`/`interpret_source` thread `PhRef`. ⟵ *brief missed this* |
| `phalcom-core/src/vm.rs` | VM **owns** the `Heap`; dispatch + bootstrap use handles. |
| `phalcom-core/src/compiler/lib.rs` | Constant/handle references. |
| `phalcom-core/src/primitive/*.rs` | Native methods take/return `Value`/handles + `&Heap`/`&mut Heap`. |
| `phalcom-core/bin/phalcom/disasm.rs` | Handle-aware disassembly. |
| `phalcom-core/tests/invariants.rs` | `imports_from PhRef` — update to the new API (keep the 2 `#[ignore]`s). ⟵ *brief missed this* |
| `phalcom-core/Cargo.toml` | Arena-crate dep (if used) + **fold DEFERRED #1** (§6). |

## 4. Design decisions (ADR-0009 / ADR-0010 — realize, don't re-litigate)
- **Heap:** a `Heap` owning all object storage; objects referenced by a `Copy` handle (`ObjRef`, plus
  typed aliases like `ClassId` where they sharpen intent). **Recommend `slotmap`** (Copy keys,
  generational, zero `unsafe`, use-after-free → clean `None`, not UB) over a hand-rolled `Vec`-arena;
  `generational-arena` is the fallback. **Justify the final choice in the module `//!` doc** and cite
  ADR-0009. Interior mutability lives in the heap (`heap.get(id)` / `heap.get_mut(id)`), **not** in
  per-object `RefCell`.
- **`Value`:** tagged `enum` — `Number(f64)`, `Bool(bool)`, `Obj(ObjRef)`, a **private** `Nil`
  sentinel (never surface-visible, never inside a `Some`), interned `Symbol`. NaN-boxing stays
  deferred behind the same API (note it in the doc + [`DEFERRED.md`](DEFERRED.md)).
- **Ownership:** the `VM` owns the `Heap`. Methods that today call `self.borrow()`/`borrow_mut()`
  take `&Heap`/`&mut Heap` instead.
- **Bootstrap (supersedes `Rc::new_cyclic`):** allocate all kernel class/metaclass objects first to
  get their handles, then patch `superclass`/`class` fields (`Metaclass.class` self-cycle becomes a
  handle to itself — trivially fine in an arena; no `MaybeWeak`, no weak path). **Observable wiring
  identical to today** — preserve F2 as-is.

## 5. Build order (keeps the change reviewable; land as one coherent diff)
1. **`heap.rs`** — introduce `Heap` + `ObjRef` (+ typed aliases). Full rustdoc, cite ADR-0009.
2. **`value.rs`** — tagged `Value` enum. Full rustdoc + per-variant `///`, cite ADR-0010.
3. **`phalcom-common/refs.rs`** — retire `PhRef`/`phref_new`/`phref_weak` (or repurpose). Decide
   where `ObjRef` lives (recommend `phalcom-core/heap.rs`; gut `refs.rs` accordingly) and record why.
4. **Object structs** (`class`, `instance`, `closure`, `frame`, `method`, `module`) — store handles;
   move interior mutability into `Heap`; methods take `&Heap`/`&mut Heap`.
5. **`vm.rs` + `universe.rs`** — VM owns `Heap`; bootstrap via allocate-then-patch (§4).
6. **Threading** — `chunk`, `compiler/lib`, `interpret`, `primitive/*`, `disasm`, immediate types
   (`nil`/`boolean`/`string`) take `Value`/handles + heap access.
7. **`tests/invariants.rs`** — migrate to the new API; the 2 spec-target invariants **stay `#[ignore]`d**.
8. **Cleanup** — fold DEFERRED #1 (§6) and drop any borrow-hazard remnants.

## 6. Fold-in cleanup (DEFERRED #1 — U1 owns `phalcom-core/src` + `Cargo.toml`)
Remove the residual LALRPOP debt now that `phalcom-ast` no longer uses it:
- `lalrpop-util` dep in `phalcom-core/Cargo.toml`.
- dead `CompilerError::ParseError` variant + its `From<lalrpop_util::ParseError>` impl in `phalcom-core/src/error.rs`.

Confirm removal doesn't touch anything outside the write-set (`graphify affected "CompilerError"` first).

## 7. Mandatory rules
- **Docs** ([`docs/rust-documentation-guidelines.md`](../rust-documentation-guidelines.md)): `//!` on
  new `heap` module + every touched module; `///` on every public item (`Heap`, `ObjRef`, `Value` +
  all variants, every migrated method) with `# Panics`/`# Safety` where applicable, intra-doc links,
  and ADR-0009/0010 citations. Prefer safe generational indices; any `unsafe` needs a `// SAFETY:` note.
  `cargo doc --workspace --no-deps` adds **no new warnings**.
- **Green gate:** `./scripts/verify.sh` exits 0 (build + test + clippy + golden + invariants). Golden
  output byte-identical. Don't add clippy warnings; fix the pre-existing ones in files you rewrite.
- **Best practices:** `rust-best-practices` skill; `rust-sanitizers-miri` if any `unsafe` lands. The
  whole point is to *remove* borrow hazards — do not reintroduce any.

## 8. Return contract (to the reviewer, not self-approval)
Report: heap/handle choice + rationale · `Value` layout · files changed · how goldens + invariants
stayed green (with `verify.sh` tail) · `cargo doc` tail · **explicit confirmation F2 was NOT touched**
and the tower is observationally unchanged · DEFERRED #1 removed · any new `DEFERRED.md` entries.
A `phalcom-reviewer` independently verifies representation correctness + borrow-hazard removal + green gate.
