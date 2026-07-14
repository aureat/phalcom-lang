# U-IC — Implementation spec (monomorphic inline cache)

> Companion to [`plan.md`](plan.md) — that file has rationale, ADR anchors, hazards. **This file
> supersedes plan.md's file/line references and two of its design choices** (see §0). Follow this
> file literally; it is written against HEAD (2026-07-14, `2b75429`) after the `vm.rs`/`class.rs`/
> `value.rs` module split.

## 0. Scope corrections vs plan.md — READ FIRST

plan.md was written before the module split and before `Bytecode` was confirmed to be a Rust enum.
Three corrections, all **scope reductions**. Do not implement the dropped items.

| plan.md said | HEAD reality | This spec does |
|---|---|---|
| Change 1: selector-only `SelectorId` interner (prerequisite) | Only needed if you index a per-class array *by selector*. We do not. | **DROPPED from v1.** `Symbol` is the cache key comparand; no interner change. |
| Change 3: operand-free superinstructions `LOAD_LOCAL_0..15`, opcode-budget check | `Bytecode` is `#[derive(Copy)] enum` with inline operands (`bytecode.rs:2-3`, `GetLocal(u16)` at `bytecode.rs:35`). There is **no separate operand fetch** to fold away, and no 256-opcode budget. The Wren technique targets a `u8` bytestream; it is a no-op here. | **DROPPED.** Record in the return shape that it was dropped and why. |
| Change 2 storage: "per-class own-method arrays (design B)" | The superclass walk already does `IndexMap::get` per level (`heap/class.rs:74-85`). Replacing the map is a second, independent optimization. | **DROPPED from v1.** v1 is the cache *only*; the walk stays exactly as-is and is what a cache miss falls back to. |
| DEC-IC-A: recommend per-class epoch | `superclass=` is **sealed** — `class_set_superclass` always returns `InvalidSetSuper` (`primitive/class.rs:43-45`). Methods are only ever *added/replaced*, never removed. Mutation happens almost exclusively during class-body execution (startup). | **Resolved: global `world_version: u64`.** Simplest sound thing. Per-class epochs buy nothing measurable when mutation is startup-clustered. |
| DEC-IC-B SuperSend | unchanged | **Uncached in v1.** `SuperSend` (`dispatch.rs:674`) is untouched. |
| DEC-IC-D mono vs PIC | unchanged | **Monomorphic.** Slot layout leaves room to grow (§2). |

Net v1 = **one** change: a per-call-site monomorphic cache on `Bytecode::Invoke`, invalidated by a
global version counter. Nothing else.

## 1. Preconditions — verify on HEAD before writing code

Run these and confirm each. If any is false, **STOP and report**; the design below assumes them.

1. `lookup_method_in_hierarchy(heap, class, selector)` — `phalcom-core/src/heap/class.rs:74-85` —
   walks `superclass` doing `current.methods.get(&selector)` per level. No cache anywhere.
2. `Bytecode::Invoke(arity, selector_idx)` handler — `phalcom-core/src/vm/dispatch.rs:829-868`.
   The hit path is `receiver.lookup_method(self, selector_sym)` at `dispatch.rs:837`.
3. `Value::lookup_method` — `phalcom-core/src/value/mod.rs:164-181` — calls
   `lookup_method_in_hierarchy`, then for **class receivers only** does an `init <sel>` fallback
   probe (`value/mod.rs:171-177`). **The cache must reproduce this fallback's result**, not just the
   primary walk (§2.3).
4. The **complete** set of sites that mutate a `ClassObject.methods` table (`grep 'add_method'`):
   - `phalcom-core/src/vm/dispatch.rs:733` — `Bytecode::Method`, static (metaclass row)
   - `phalcom-core/src/vm/dispatch.rs:735` — `Bytecode::Method`, instance-side
   - `phalcom-core/src/primitive/mod.rs:116` and `:131` — the `primitive!` registration macro
   - `phalcom-core/src/universe/primitives.rs:156` and `:188` — Bool/Option per-symbol installs

   These 6 are **all of them**. Missing one is the unsoundness bug this unit exists to avoid.
5. The only `set_superclass` caller is `phalcom-core/src/vm/api.rs:41` (bootstrap wiring, before user
   code runs); the surface setter is sealed at `primitive/class.rs:43`. Confirm both.
6. `ClosureObject.callable` is a **plain `Callable`, deep-cloned** on every block materialization
   (`vm/dispatch.rs:449`, `let callable = self.heap.closure(template_id).callable.clone();`). This
   decides cache storage — see §2.1.

## 2. Design

### 2.1 Where the cache lives (resolves DEC-IC-C)

Per call site = per `(chunk, ip)`. Store it **in the `Chunk`**, in a `Vec` parallel to `code`:

```rust
// phalcom-core/src/chunk.rs
use std::cell::Cell;

/// One monomorphic inline-cache slot, owned by a single `Bytecode::Invoke` site.
#[derive(Debug, Clone, Copy)]
pub struct InlineCache {
    /// Receiver class the cached resolution was recorded for.
    pub class: ClassId,
    /// The resolved `MethodObject` handle.
    pub method: ObjRef,
    /// `VM.world_version` at record time; a mismatch means a method was
    /// (re)defined somewhere since, and the entry must be discarded.
    pub version: u64,
}

pub struct Chunk {
    pub code: Vec<Bytecode>,
    pub constants: Vec<Value>,
    pub spans: Vec<SourceRange>,
    /// Parallel to `code`; only `Bytecode::Invoke` indices are ever non-`None`.
    pub caches: Vec<Cell<Option<InlineCache>>>,
}
```

`Cell<Option<InlineCache>>` is `Copy`-payloaded, so `Chunk` stays `Clone`, and — critically — the
cache is writable through a **shared** `&Chunk` borrow. That is what makes §2.2 pass the borrow
checker without `unsafe` and without a `closure_mut` accessor.

Keep `caches.len() == code.len()`: push a `Cell::new(None)` inside `Chunk::add_instruction`
(`chunk.rs:22-25`). That is the only place instructions are appended — verify with
`grep -rn 'code.push' phalcom-core/src`; if there is another, extend it too.

**Known limitation, document it, do not fix it here:** a block literal deep-clones its `Callable`
each time `Bytecode::Closure` runs (`dispatch.rs:449`), so a block body's caches reset per
materialization. Method bodies (the common case) hold one closure for the class's lifetime and cache
normally. Making `ClosureObject.callable` an `Rc<Callable>` fixes both this and a chunk deep-clone —
it is specced as U-HOTPATH Change 4. If U-HOTPATH lands first, this unit gets the fix for free and
needs no change (`Cell` works identically behind an `Rc`).

### 2.2 The Invoke fast path

Replace `dispatch.rs:829-868`. The current body reads the selector constant, then calls
`receiver.lookup_method`. New shape:

```rust
Bytecode::Invoke(arity, selector_idx) => {
    let arity = arity as usize;
    let receiver_idx = self.stack.len() - 1 - arity;
    let receiver = self.stack[receiver_idx];
    let receiver_class = receiver.class(self);

    // Cache probe. `chunk` is a shared borrow; the `Cell` is what lets us
    // write back through it (§2.1).
    let cached = {
        let chunk = &self.heap.closure(closure_id).callable.chunk;
        chunk.caches[ip].get().filter(|slot| {
            slot.class == receiver_class && slot.version == self.world_version
        }).map(|slot| slot.method)
    };

    if let Some(method) = cached {
        self.call_method(&receiver, method, arity, source_range)?;
    } else {
        let selector_val = self.heap.closure(closure_id).callable.chunk.constants[selector_idx as usize];
        let selector_sym = selector_val.as_symbol().unwrap();

        if let Some(method) = receiver.lookup_method(self, selector_sym) {
            // Refill. Both `receiver_class` and `world_version` are read
            // AFTER the lookup on purpose — see §2.3 hazard 2.
            let entry = InlineCache { class: receiver_class, method, version: self.world_version };
            self.heap.closure(closure_id).callable.chunk.caches[ip].set(Some(entry));
            self.call_method(&receiver, method, arity, source_range)?;
        } else {
            // ... existing variadic-probe + dNU fallback, dispatch.rs:853-866, UNCHANGED.
            // Do NOT cache a variadic hit or a dNU forward in v1.
        }
    }
}
```

`ip` is already in scope (`dispatch.rs:409`). `world_version` is a new `pub u64` field on `VM`
(`vm/mod.rs`, next to `universe`/`heap`), initialized to `0`.

### 2.3 Soundness — three hazards, each with its rule

1. **`init` fallback parity.** `Value::lookup_method` (`value/mod.rs:164`) resolves *more* than the
   plain hierarchy walk: for a class receiver it also probes `init <sel>`. The refill in §2.2 caches
   whatever `lookup_method` returned, so parity is automatic — **that is exactly why the refill calls
   `lookup_method` and not `lookup_method_in_hierarchy`.** Do not "optimize" it to the latter.
2. **Read `world_version` after the lookup, never before.** `lookup_method` cannot itself define a
   method today, but reading the version *after* means the recorded version can only ever be ≥ the
   truth, never <. A stale-low version would make a stale entry look fresh. Cheap, so just do it.
3. **Every mutation site bumps.** Add `self.world_version += 1;` (or
   `vm.world_version += 1;` inside the macro) at **all six** sites from §1.4. The macro sites
   (`primitive/mod.rs:116,131`) only fire at bootstrap, but bump anyway — a rule with an exception is
   a rule someone will break. Grep for `add_method` after the change: **every** call must have a bump
   in its immediate scope, or you must be able to name why not in the return shape.

Do NOT bump on `set_superclass` (`vm/api.rs:41`) — it runs during bootstrap before any Invoke has
cached anything. Note it; do not add speculative bumps elsewhere.

## 3. Write-set (STOP-and-report if you touch anything else)

- `phalcom-core/src/chunk.rs` — `InlineCache` struct, `caches` field, push in `add_instruction`.
- `phalcom-core/src/vm/mod.rs` — the `world_version: u64` field + its rustdoc (the invalidation
  contract belongs in that doc — [[rust-doc-mandatory]]).
- `phalcom-core/src/vm/dispatch.rs` — the `Invoke` arm (§2.2); `+= 1` at `:733`, `:735`.
- `phalcom-core/src/primitive/mod.rs` — `+= 1` in the `primitive!` macro (both arms).
- `phalcom-core/src/universe/primitives.rs` — `+= 1` at `:156`, `:188`.
- `phalcom-core/tests/` — the coherence goldens (§5).
- **Floor: +0.** No new primitive, no surface change, no ADR amendment.

## 4. Build order — commit each green step ([[commit-frequently]])

1. **Version counter alone.** Add `VM.world_version`, bump at all 6 sites. Nothing reads it yet.
   `cargo test` green. Commit.
2. **Cache plumbing, always-miss.** Add `InlineCache` + `Chunk.caches`, keep `caches[ip]` always
   `None` (do not write the refill yet). Proves the parallel-vec invariant holds across the whole
   corpus. `cargo test` green. Commit.
3. **Wire probe + refill** (§2.2). `cargo test` green **and** the §5 coherence tests pass. Commit.
4. Run `scripts/vm_bench.rs`; record the delta. Commit nothing but the number (perf-log).

## 5. Tests — the coherence test is the gate, not the corpus

The full golden corpus passing proves nothing about an IC (it is static). Write these under
`phalcom-core/tests/lang/` following the existing positive-lane convention
([[phalcom-golden-test-lanes]]):

- `ic_override_after_caching.ph` — define `class A { foo => 1 }`, send `a.foo` in a loop ~10 times
  (populates the slot), then reopen/redefine `foo => 2` at runtime, send again. **Expect `2`.** A
  missing version bump prints `1`. This is the load-bearing test.
- `ic_add_method_invalidates.ph` — send a selector that resolves on the *superclass*, cache it, then
  define an override on the *subclass*, send again from the same call site. Expect the subclass's.
- `ic_megamorphic_still_correct.ph` — one call site (a method taking a param, called in a loop) hit
  with receivers of 4+ different classes, each printing its own answer. Expect every answer correct;
  the cache thrashes but must never serve the wrong class's method.
- `ic_class_side_init_fallback.ph` — a call site whose resolution comes from the `init <sel>`
  fallback (`value/mod.rs:171`), sent repeatedly. Expect identical behavior to HEAD (hazard 1).

Gate: `cargo build && cargo test && cargo clippy --workspace` green, `cargo doc` clean,
WORKTREE-VERIFY each SHA on a throwaway checkout ([[clean-checkout-verify-each-commit]]).

## 6. Return shape

commit SHAs · confirmation that all 6 `add_method` sites bump `world_version` (list them) ·
confirmation that the refill routes through `Value::lookup_method` (init-fallback parity) ·
the 4 coherence tests + their results · zero golden diff · `scripts/vm_bench.rs` send/arith delta ·
explicit note that plan.md's Change 1 (selector interner), Change 3 (superinstructions) and design-B
own-method arrays were **dropped as no-ops/out-of-scope on HEAD** (§0) · any `unsafe` (expect none) ·
floor delta (0) · write-set confirm.
