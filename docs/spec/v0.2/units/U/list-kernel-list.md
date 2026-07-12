# U-LIST — Kernel List (as-built)

- **Status:** ✅ Landed — `c7c63fb` (native variant + primitives), `6fdf0c7` (core.ph execution + `.ph` protocol), `b2f7aec` (acceptance corpus), `333823a` (forge docs)
- **Realizes:** [ADR-0019](../../../../adr/0019-freeze-vm-blessed-primitive-floor.md) (frozen VM-blessed primitive floor), [ADR-0020](../../../../adr/0020-kernel-list-native-array-protocol.md) (native `Vec<Value>` List behind the handle heap); spec [core/catalog-delta.md §2.4](../../core/catalog-delta.md), [core/floor-census.md](../../core/floor-census.md)
- **Reviewer gate:** OFF per the load-bearing-only review policy (STATE.md) — self-verified on the green gate (`cargo build`/`test`/`doc`/`clippy` all clean).

## Mission
Ship the kernel `List` as a native `Vec<Value>` heap object (not an `InstanceObject`, so no
U7 dependency), expose exactly the frozen-floor primitives ADR-0019 authorizes, and layer a
thin `.ph` protocol (`at`/`size`/`add`/`each`) over them. Along the way, land the fix that
makes `core.ph` actually execute at boot — previously it was registered but never run, so
every core-class reopen was inert.

## Surface / behavior
```phalcom
var xs = List.new()
xs.add(1).add(2).add(3)   // add returns self → chainable
xs.size                    // 3
xs.at(0)                   // 1
xs.at(99)                  // None  (out-of-range → absence, never a panic)
xs.each { x => System.print(x) }
xs.toString                // "[1, 2, 3]"
```
- `at(_:)` on an out-of-range index yields `None`; a non-Number / negative / fractional /
  infinite index is a hard `RuntimeError::Type`.

## Implementation
- **`list.rs` (new)** — `ListObject { elements: Vec<Value> }`, mirroring `StringObject`.
- **`heap.rs`** — `Object::List` variant + `alloc_list`/`list`/`list_mut`/`as_list`.
- **`value.rs`** — `class()`/`to_debug()`/`to_context()` gain `List` arms.
- **`primitive/list.rs` (new)** — **five** floor primitives (not six): `list_class_new`
  (public, backs `List.new()`) and the internal `rawLength`/`rawAt`/`rawSet`/`rawPush`
  (`rawXxx`-named so the `.ph` wrappers don't recurse on the public selector), plus a native
  `toString`. No separate "grow" primitive — `rawPush` relies on `Vec::push`'s own amortized
  doubling. `rawAt` returns `vm.none_value()` for an out-of-range index (never `Value::Nil`).
- **`universe.rs`** — `List` created in `create_core_classes` right after `Option`/`Some`/`None`
  (ADR-0020 load order), the same way `Option`/`Bool`/`String` are — **not** via an
  `InstanceObject`.
- **`primitive/mod.rs` / `vm.rs`** — `ClassName::List` + `List` registered as a global.
- **`core.ph`** — thin protocol: `size => self.rawLength`; `at(_:)` and `add(_:)` wrap the raw
  primitives (`add` returns `self`); `each(_:)` is a `.ph` while-loop calling
  `f.call(self.at(i))` (proves block-calling into `List` iteration). `rawSet` is wired but
  **not** yet surfaced (`at(_:put:)` deferred).

## The core.ph-inert bug (found and fixed here)
- **`core.ph` was registered as source (`VM::install_core`) but never compiled or executed** —
  not by the CLI, not by the test harness — so every `.ph` class-reopen skeleton
  (`Option`/`Some`/`String`/…) was silently inert. `VM::new` now calls a new
  **`VM::run_core_module()`** right after `Universe::install_primitives`, making `List`'s `.ph`
  protocol — and every other core-class reopen — take effect.
- Running `core.ph` for the first time surfaced a second latent bug: `Statement::Class`
  unconditionally emits `DefineGlobal` at the end of every class body, reopen or not. For most
  core classes this is a no-op, but `None`'s global is deliberately bound to the shared
  singleton *instance*, so the empty `class None {}` reopen clobbered that binding back to the
  class the instant `core.ph` ran. Fixed by dropping the purposeless empty reopen (nothing
  lost) and documenting the trap; the compiler special-case itself is deferred for whoever
  next needs real `None` members.

## Invariants & tests
- `list` PASS golden label (4): construction + add + size + at round-trip;
  absence at the `at(_:)` boundary (`None`, not a panic/sentinel); `each(_:)` block-calling
  sum; `toString` bracket rendering.
- 1 NEGATIVE in the shared `runtime-errors` label: non-Number index → type error.
- `MANIFEST.md` counts + label matrix updated.

## Deviations & deferrals
- **`List.toString` is a native primitive, not `.ph` over `each` + concat** (the plan's
  sketch) — no kernel value type has a general user-callable `.toString` yet, so building it in
  `.ph` would render every non-`String` element as `"<ClassName>"`. Move to `.ph` once value
  types get real `toString` → [forge/DEFERRED.md](../../../../forge/DEFERRED.md) #19.
- **`rawSet` not surfaced** — no `at(_:put:)` selector this unit →
  [forge/DEFERRED.md](../../../../forge/DEFERRED.md) #18 (later delivered by U-STD).
- **No combinators or list-literal syntax** (`map`/`reduce`/`filter`, `[a, b, c]`) — U-STD's
  job, layered additively over the floor → [forge/DEFERRED.md](../../../../forge/DEFERRED.md) #20;
  see also [deferred-work.md](../../deferred-work.md) (collection-literal lowering).
- **`None`-reopen compiler trap** (`Statement::Class` unconditional `DefineGlobal`) →
  [forge/DEFERRED.md](../../../../forge/DEFERRED.md) #17 (high priority for the next `None` work).

## Sources
- [forge/STATE.md](../../../../forge/STATE.md) "U-LIST — LANDED"; [forge/PHASE2-INDEX.md](../../../../forge/PHASE2-INDEX.md).
  Per-unit planning record (`U-LIST-plan.md`, `U-LIST-U8-implement-handoff.md`) folded into this spec; see git history.
- Commits `c7c63fb`, `6fdf0c7`, `b2f7aec`, `333823a`.
- Code: `phalcom-core/src/{list.rs,heap.rs,value.rs,universe.rs,vm.rs}`, `primitive/{list.rs,mod.rs}`, `core/core.ph`.
