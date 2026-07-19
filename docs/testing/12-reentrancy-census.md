# 12 — Lane C: Re-entrancy Census

> **Oracle:** invariant (guard completeness) + pinned diagnostic (guard firing).
> **Closes:** [G4, G5](02-coverage-ledger.md#2-what-that-leaves-uncovered).
> **Status:** specified, not built.

## 1. What is actually being tested

Read [02 §3](02-coverage-ledger.md#3-the-re-entrancy-guard-as-it-actually-is)
first. The guard is **one VM counter**, not a per-primitive check — which is a
good design, and it changes what needs testing.

`Fiber.yield` and `Fiber#resume` raise `CannotYieldAcrossNativeFrame` when
`VM::native_reentry_depth` indicates a native Rust frame sits between the fiber
and the dispatch loop. Every re-entrant primitive inherits that guard by routing
through one of four incrementing wrappers. So the two things that can break are
not "did primitive X remember to check" but:

> **C-INV-1 (completeness).** Every path from native Rust into `run_until`
> passes through a site that increments `native_reentry_depth`.
>
> **C-INV-2 (balance).** `native_reentry_depth` is zero whenever the dispatch
> loop is at top level — on the success path *and* the error path.

Neither is asserted today. Both are mechanically checkable.

## 2. C-INV-2 — balance

The cheaper and sharper of the two. All four sites currently spell it:

```rust
self.native_reentry_depth += 1;
let result = self.run_until(base_frames);
self.native_reentry_depth -= 1;
result
```

Correct — and *fragile*, in a specific way worth naming. A future edit to
`let result = self.run_until(base_frames)?;` would be the natural-looking
change, would compile, would pass every existing test, and would leak the
counter on the error path. Every subsequent `Fiber.yield` in the process then
raises `CannotYieldAcrossNativeFrame` spuriously, forever. The failure surfaces
far from its cause and looks like a fiber bug.

Assertions:

- **C-2a.** After any top-level program run — success or failure —
  `vm.native_reentry_depth == 0`. Cheapest possible check; wire it into the
  in-process test harness so it runs after *every* in-process case, not as its
  own test.
- **C-2b.** Explicitly for the error path: a `.ph` program that raises from
  inside `Block#call`, from inside a `doesNotUnderstand` forward, and from
  inside `Method#invokeOn` — one case each, asserting the counter is zero
  afterward and a subsequent `Fiber.yield` still works.
- **C-2c.** Debug-build `debug_assert` at the decrement site that the counter
  was nonzero before decrementing, catching underflow from an unbalanced pair.

C-2b is the one that would have caught the hypothetical `?` edit. Structure it
so the fixture *uses a fiber after the error* — asserting only that the counter
is zero tests the implementation; asserting that yielding still works tests the
behavior anyone actually cares about.

## 3. C-INV-1 — completeness

Harder: it quantifies over paths, and Phalcom has no static analysis to prove
it. Three approximations, in increasing strength.

### 3a. Grep census (build-time)

`run_until` is `pub(crate)`. Enumerate its call sites; each must either be the
top-level driver (`run`, `base_frames == 0`) or be immediately preceded by an
increment. Today that yields exactly the four sites in the ledger, plus `run`.

Ship it as a test that reads the source tree and asserts the census matches a
pinned list, with the failure message naming the new site and pointing here.
Crude, and it works: it converts "a reviewer might notice" into "CI blocks."
Precedent — the `core_class_rows` census in
[`invariants.rs`](../../phalcom-core/tests/invariants.rs) is exactly this shape
and exactly this justification.

### 3b. Behavioral census (runtime)

For every native primitive that can re-enter `.ph`, one fixture asserting a
`Fiber.yield` from inside it raises `CannotYieldAcrossNativeFrame` rather than
corrupting. Enumerate the sites from `src/primitive/`, not from memory:

| Re-entry site | Fixture | Routes via |
|---|---|---|
| `Block#call` | `reentry_block_call_yield_raises` | `block.rs:158` |
| `List#each` / `map` / `filter` / `reduce` | one each | `block_call` |
| `Map` key `hash` / `==` | `reentry_map_hash_yield_raises` | `block_call` (noted at `map.rs:53`) |
| sort comparator | `reentry_sort_comparator_yield_raises` | `block_call` |
| `toString` during `print` | `reentry_tostring_yield_raises` | `send_dynamic` |
| `doesNotUnderstand` forward | `reentry_dnu_yield_raises` | `send.rs:233` |
| `Object#perform` / `performWith` | `reentry_perform_yield_raises` | `send.rs:233` |
| `Method#invokeOn` / `bind` | `reentry_invokeon_yield_raises` | `send.rs:276` |
| `Family` call | `reentry_family_yield_raises` | `send_dynamic` |
| `Module` dNU forward | `reentry_module_dnu_yield_raises` | `send_dynamic` |
| decorator Runtime-tier interception | `reentry_decorator_yield_raises` | `send_dynamic` — **gated on the Runtime tier being built** |

Each is a NEGATIVE case in `tests/lang/concurrency/negative/` pinning the
diagnostic. Nine such fixtures already exist for the paths someone thought of;
this makes the set derived rather than remembered.

The mirror case is worth the extra fixture: for each site, a *resume* variant
too, since `fiber_resume` checks `depth != 0` while `fiber_yield` checks
`depth != floor_depth`. Different conditions, different bugs.

### 3c. What would actually prove it

A newtype wrapper making `run_until` unreachable except through a
depth-incrementing guard object (RAII: increment on construct, decrement on
`Drop`). That converts C-INV-1 into a type-system property and C-INV-2 into an
unwind-safety property for free — `Drop` runs on panic, which the current manual
decrement does not.

This is an **implementation** change, not a test, so it is out of this
directory's scope. It is recorded here because it is strictly better than 3a and
3b, and if it lands, both approximations can be retired. Flagged for the forge
track, not for a test lane.

## 4. Cost and gating

Cheap. Fixtures are small, the grep census is milliseconds, C-2a is a single
assertion in the harness. **In the default green gate.**

## 5. Preclusion

The fixtures encode "yielding across a native frame raises." That is ADR-0030
§4's restricted execution model — and it is a **snapshot of a deferred
decision**, not an invariant.

ADR-0033 (`CallBlock` trampoline) is **Deferred, past v0.2**. If it lands, it
makes `.each { Fiber.yield(x) }` yield-*transparent*, and a large share of the
§3b table flips from raise to succeed. The overlay is explicit that the residue
is that reflective `call` and native Rust callers stay non-transparent — so the
table would split rather than vanish.

Each fixture must therefore carry a header comment naming ADR-0033 and stating
that a flip to success is an **intended graduation, not a regression**. Without
that, a future implementer reads a wall of red as breakage and either reverts or
weakens the guard. The MANIFEST's PENDING mechanism is the right destination for
the flipped cases: they move to `pending/` with the intended output pinned,
which is precisely the graduation path the corpus already models.

Second preclusion, minor: pinning diagnostic text couples these fixtures to the
`CannotYieldAcrossNativeFrame` message strings in `fiber.rs`. Those are
deliberately different for yield vs resume. NEGATIVE cases match on substring,
so pin the stable prefix (`cannot switch fibers across a native call frame`)
rather than the full rendered message including the parenthetical example.
