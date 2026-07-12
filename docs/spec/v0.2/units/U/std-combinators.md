# U-STD — Option + List Combinator Layer (as-built)

- **Status:** ✅ Landed — `176d454` (Option combinators), `5e2b395` (List combinators), `454f2b8` (discharge DEFERRED #25 + STATE). In-tree on `main`, no worktree.
- **Realizes:** [ADR-0019](../../../../adr/0019-freeze-vm-blessed-primitive-floor.md) (frozen floor — this unit adds **zero** primitives), [ADR-0020](../../../../adr/0020-kernel-list-native-array-protocol.md) (List floor), [ADR-0007](../../../../adr/0007-option-as-abstract-with-some-none.md) (Option as abstract). Spec [values-and-absence.md](../../values-and-absence.md) §3.3, [catalog-delta.md](../../core/catalog-delta.md) §2.2 / §2.4.
- **Reviewer gate:** OFF per policy — self-verified on the green gate (`../../../../forge/STATE.md` §"U-STD — LANDED"; reviewer roster line: "Reviewer OFF … U-STD").

## Mission

Ship the genuinely-remaining, unblocked, additive standard-library residual as **Option (B)** (per the user's ratification, U-STD-implementation-spec §0.1): the **Option + List combinator layer**, all pure `.ph` layered over the frozen floor. The plan's literal Object/Number/String/Symbol/System surface was ~90% already-landed or re-carved to future U-CORE-N units, so U-STD built only the combinator protocol. **Zero new native primitives; no `primitive/*.rs`, `universe.rs`, `vm.rs`, `bytecode.rs`, or `phalcom-ast/*` touched** — the sole edited runtime artifact is `phalcom-core/core/core.ph`.

## Surface / behavior

```phalcom
// Option — over the native match(some:, none:) eliminator
Some.new(3).map { v => v + 1 }          // Some(4)
Some.new(3).flatMap { v => Some.new(v) } // Some(3)  (no re-wrap)
Some.new(3).filter { v => v > 5 }        // None
Some.new(3).ifSome { v => log(v) }       // side effect, returns self (chains)
None.unwrapOr(0)                         // 0  (eager extract)

// List — over the frozen each/add/rawSet/List.new floor
list.map { x => x * 2 }
list.filter { x => x > 0 }
list.reduce(0) { acc, x => acc + x }     // reduce(_,_); trailing block = 2nd arg
list.includes(x)                         // true / false
list.isEmpty                             // size == 0
list.at(i, put: v)                       // wraps rawSet, returns self (chains)
```

**Option combinators** (`map`, `flatMap`, `filter`, `ifSome`, `unwrapOr`) each dispatch through the native `match` eliminator so `Some`/`None` branching stays dispatch, not a variant check. `map` lifts and re-wraps; `flatMap` is the monadic bind (no re-wrap); `filter` keeps `Some(v)` iff `pred(v)`, else the shared `None` singleton; `ifSome` is the `Some`-side mirror of `ifNone` (effect + `self` passthrough); `unwrapOr` is the eager sibling of `orElse`.

**List combinators** (`map`, `filter`, `reduce`, `includes`, `isEmpty`, `at(_,put:)`) build over `each`/`add`/`rawSet`/`List.new`. **Selector spellings (comma form, ADR-0012):** `reduce(_,_)` — 2 positional args, the trailing block desugars to the 2nd (`reduce(init) { acc, x => … }`); `at(_,put:)` — matches `rawSet`'s arity, labeled param named `put` (label == name), returns `self` for chaining. None of the combinators stringify an element, avoiding the `toString`-message class-name trap (DEFERRED #19).

## Implementation

Single file — `phalcom-core/core/core.ph`:

- **`Option` block** (over native `match(some:, none:)`): `map(f)` → `self.match(some: { v => Some.new(f.call(v)) }, none: { self })`; `flatMap(f)` → `some` arm calls `f.call(v)` directly; `filter(pred)` → `some` arm is a value-yielding `if (pred.call(v)) { self } else { None }`; `ifSome(f)` → `some` arm runs `f.call(v); self`; `unwrapOr(default)` → `some: { v => v }, none: { default }`. `pred` must return a real `Bool` (ADR-0021).
- **`List` block** (over `each`/`add`/`rawSet`/`List.new`): `map`/`filter` allocate a fresh `List.new()` and `each`-append; `filter` uses `pred.call(x).ifTrue { … }` (result discarded); `reduce(init, f)` folds an accumulator via `each`; `includes(x)` uses `(e == x).ifTrue { … }` (`==` is an ordinary send); `isEmpty => self.size == 0`; `at(i, put:)` wraps `rawSet(i, put)` and returns `self`.
- **Discharged the `List`-block header comment**: the "do not add `map`/`reduce`/`filter`" deferral is now false; reworded to note the bodies live below and only **list-literal syntax** `[a, b, c]` remains deferred (DEFERRED #6/#28). List-literal syntax was **not** added.

## Invariants & tests

- **Zero new floor primitives** — census unchanged; Option B added no Rust, so `cargo doc --workspace --no-deps` produced no new warnings.
- **New active `option` lang label:** `option_map_both_arms`, `option_filter`, `option_flatmap`, `option_ifsome_effect_and_passthrough` (each covers both arms). **New `list/` PASS cases:** `list_map_and_filter`, `list_reduce_sum`, `list_includes_and_isempty`, `list_at_put`.
- **Goldens byte-identical:** `examples/core_new.ph`, `person2.ph`, `calculator.ph`, `tests/fixtures/golden/*` unchanged (methods only added).
- **Discharged DEFERRED #25:** `blocks/pending/blocks_argument_to_method.ph` was rewritten off the real `List.reduce` (list built with `List.new()`/`add(_)`, no literal) and **promoted** to the active `blocks/blocks_argument_to_method.ph`; the empty `blocks/pending/` dir was kept so the ignored `blocks_pending` probe still finds a directory.
- Full `phalcom-core` suite + goldens green.

## Deviations & deferrals

- **Option (B) scope, not the literal plan:** the plan's broad Object/Number/String/Symbol/System surface was already-landed or re-carved to the `docs/spec/core/` U-CORE-N track — the forge-index vs. `docs/spec/core` scope-taxonomy divergence is [DEFERRED #29](../../../../forge/DEFERRED.md) (resolved for this unit via Option B).
- **No element stringification** in any combinator — a general user-callable content `toString` does not exist yet (blocked on U-CORE-4); see [DEFERRED #19](../../../../forge/DEFERRED.md).
- **List-literal syntax `[a, b, c]` still deferred** (needs a new ADR + parser work) — [DEFERRED #6/#28](../../../../forge/DEFERRED.md).
- The `absence` label was **not** un-ignored (its `#[ignore]` reason is unrelated drift); `system()`/`system_pending()` untouched.
- See also [deferred-work.md](../../deferred-work.md).

## Sources

- ADRs: [0019-freeze-vm-blessed-primitive-floor.md](../../../../adr/0019-freeze-vm-blessed-primitive-floor.md), [0020-kernel-list-native-array-protocol.md](../../../../adr/0020-kernel-list-native-array-protocol.md), [0007-option-as-abstract-with-some-none.md](../../../../adr/0007-option-as-abstract-with-some-none.md).
- Spec: [values-and-absence.md](../../values-and-absence.md) §3.3; [catalog-delta.md](../../core/catalog-delta.md) §2.2 / §2.4.
- Code: `phalcom-core/core/core.ph` (`Option` block L70–124, `List` block L142–212); tests `phalcom-core/tests/lang/option/*`, `phalcom-core/tests/lang/list/*`, `phalcom-core/tests/lang/blocks/blocks_argument_to_method.ph`.
- Forge: [STATE.md](../../../../forge/STATE.md) §"U-STD — LANDED"; [U-STD-implementation-spec.md](../../../../forge/U-STD-implementation-spec.md); [U-STD-plan.md](../../../../forge/U-STD-plan.md).
- Deferred: [deferred-work.md](../../deferred-work.md); [DEFERRED.md](../../../../forge/DEFERRED.md) #6/#18/#19/#25/#28/#29.
