# U-ITERABLE — Work order: bare-cursor Route B + kernel `Iterable` root

_Self-contained implementation plan for **one** implementer. Post-U-ITER / post-U-STD unit.
**Reviewer ON** (touches `compiler/lib.rs`, a spine file, plus `bytecode.rs`/`vm.rs`) — hand the
diff to `phalcom-reviewer`; do not self-approve. Green gate: `./scripts/verify.sh` exits 0 +
`cargo doc --workspace --no-deps` clean. Grounded in **[ADR-0048](../../../adr/0048-amend-iteration-bare-cursor-sentinel-and-iterable-root.md)**
(amends ADR-0035 §1/§4) and normative **[iteration.md](../../../spec/v0.2/iteration.md) §1–§6**
(already rewritten for this unit — read it first, it is more current than ADR-0035 alone).

> **Grounding correction vs. any stale framing.** ADR-0048 is **already Accepted** (dated
> 2026-07-13) and `iteration.md` is **already rewritten** to its post-amendment shape (§5
> `Countdown extends Iterable`, §2 bare-cursor desugar). There is **no new ADR to write** —
> this plan realizes an already-ratified amendment; cite ADR-0048 directly, do not treat it as
> pending. `docs/spec/v0.2/deferred-work.md` already carries a **U-ITERABLE** row ("ratified
> (ADR-0048); code unbuilt") plus forward rows for **U-SEQ** (`all`/`any`/`count`/`find`/`join`
> + lazy views, depends on this unit) and **U-STRING** — both out of scope here, cited only for
> the must-not-preclude check.

## 1. Mission (one sentence)
Kill the per-step `Some` allocation in the cursor protocol (`iterate(_)` returns the **bare next
cursor** or the `None` singleton — never `Some(cursor)`), retarget `for`'s loop-scaffold to test
the returned cursor against `None` **by identity** via one new opcode, and introduce a kernel
`Iterable` root that `List`/`Map`/`Set`/`Tuple`/`Range` superclass to, hoisting the one true
`iterate`/`each`/`map`/`filter`/`reduce`/`includes`/`isEmpty` definition onto it.

## 2. Preconditions (verify on actual HEAD — do not assume)
- **U-ITER landed** — `for`/`break`/`continue`, the `LoopContext` stack, `Jump`/`JumpIfFalse`/
  `Loop`, and `compile_for` (`compiler/lib.rs::compile_for`, **verified at L1421–L1514** on
  HEAD) all exist and are green. This unit edits `compile_for` **in place** — it is the same
  function, not a new one.
- **U-STD landed** — `List#map`/`filter`/`reduce`/`includes`/`isEmpty` exist **only on `List`**
  today (`core.ph` **L332–L369**), built over `List#each` (**L299–L303**, itself `for (x in
  self) { f.call(x) }`). **Grounding correction:** `Map`/`Set`/`Tuple`/`Range` do **not**
  currently define these five combinators at all — only `iterate` (**verified identical
  one-armed shape** at `core.ph` L318–L321 List, L482–L485 Map, L545–L548 Set, L595–L598 Tuple,
  L695–L698 Range) and `each` (**verified NOT uniform**: List's `each` already routes through
  `for`; `Map`/`Set`/`Tuple`/`Range`'s `each` are hand-rolled `while (i < self.size) { … }`
  index loops that never call `iterate`/`iteratorValue`, and **`Map`'s `each` is 2-arity**
  `(k, v)` — semantically different, not a shape duplicate of the other four). Hoisting
  `map`/`filter`/`reduce`/`includes`/`isEmpty` onto `Iterable` is therefore **new capability**
  for `Map`/`Set`/`Tuple`/`Range`, not just dedup of `List`'s copies — treat it as such when
  writing goldens (new-capability tests, not just no-regression tests).
- **DEFERRED.md's DEC-ITER-A** (`docs/forge/DEFERRED.md` **L22**) is exactly what this unit
  discharges ("migrate the combinators off `size`/`at` onto the `iterate`/`iteratorValue`
  protocol as `.ph` defaults, owning unit U-STD" — U-STD deferred it, this unit is the actual
  owner per ADR-0048 §3/deferred-work.md's new U-ITERABLE row). Strike that DEC-ITER-A row in
  the return contract (do not edit `DEFERRED.md` yourself per the write-set below — flag it for
  the orchestrator/reviewer to close).
- **`Value: PartialEq` is derived** (`value.rs` L30, confirmed: `Value::Obj(ObjRef)` compares by
  the arena-slot `ObjRef`'s own derived `PartialEq` — index/generation identity, not structural)
  — so `cursor_value == Value::Obj(universe.classes.none_singleton)` is already a cheap,
  correct **identity** test with **zero new machinery**; only the *opcode* to run it inline is
  new.
- **Two-armed `ifTrue(_, ifFalse:)` never `Some`-lifts** — confirmed by reading
  `compiler/inliner.rs::compile_if_true_if_false` (**L352–L370**): unlike
  `compile_if_true`/`compile_if_false` (which conditionally `emit(Bytecode::WrapSome, …)` per
  the `want_value` dance), the two-armed inline path has **no `WrapSome` emission at all** —
  both arms are `compile_inline_block_body`'d directly. It is also already **inliner-eligible**
  (guarded fast path + `emit_sacred_send("ifTrue", &[None, Some("ifFalse")], …)` fallback), so
  the Route-B `.ph` bodies below get the fast path for free, no new inliner work.
- **No identity/`None`-test opcode exists today.** Full read of `bytecode.rs`'s `Bytecode` enum
  (**L3–L277**, all 26 variants) — only `Jump(i32)`/`JumpIfFalse(i32)`/`Loop(i32)` and the two
  deopt guards (`GuardBool`/`GuardBlock`) branch on anything; there is no `Equal`/`Identical`/
  `JumpIfNone`-shaped opcode anywhere. §3.2 below is therefore the **one flagged new opcode**
  ADR-0048 §2 pre-authorized ("the realizing unit resolves whether an existing opcode expresses
  the `None`-identity branch or one honest opcode must be added (flagged, not invented)") — this
  plan resolves it as: add one.
- **Superclass wiring precedent** — `universe.rs::make_core_class` (**L823**) requires its
  `superclass` argument's `.class` (metaclass) link already wired; `List`/`Map`/`Set`/`Tuple`/
  `Range` are each created via `make_core_class(heap, "<Name>", object_class, metaclass_class)`
  at **L211/L218/L219/L224/L229** respectively, all currently superclassed directly to
  `object_class`. `CoreClasses` (struct at **L843**) has no `iterable_class` field yet; the
  name→class global-binding list (`("List", c.list_class)`, … at **L717–L721**) has no
  `"Iterable"` row yet.
- **Concurrent-session caution** (standing repo hazard) — `main` had `bytecode.rs`/`vm.rs`/
  `universe.rs`/`compiler/lib.rs` mid-edit by other sessions as recently as this same day (U16
  work); by plan time they are committed and clean again (verified via `git status --porcelain`
  at plan time — only doc files dirty: `deferred-work.md`, `iteration.md`,
  `values-and-absence.md`, plus the new ADR-0048 and three `docs/spec/v0.2/next/*` files,
  untracked). **Re-run `git status`/`graphify affected` on all five write-set Rust files
  immediately before dispatch** — do not trust these line numbers blindly; they are exact as of
  this read, but `main` moves.
- Baseline `./scripts/verify.sh` green before the first edit.

## 3. Design (realise ADR-0048 / iteration.md — do not re-litigate Route A vs B or the sentinel choice)

### 3.1 `iterate(cursor)` returns the bare cursor, never `Some` (ADR-0048 §1)
Every per-collection `iterate` moves onto the new `Iterable` root (§3.3) and is rewritten from
the current one-armed shape:
```phalcom
// OLD (List/Map/Set/Tuple/Range, identical, allocates a Some+an Option#map Some every step):
iterate(cursor) {
  let next = cursor.map { c => c + 1 }.unwrapOr(0)
  return (next < self.size).ifTrue { next }
}
```
to the two-armed, zero-allocation shape (ADR-0048 §1, exact text):
```phalcom
iterate(cursor) {
  let next = (cursor == None).ifTrue({ 0 }, ifFalse: { cursor + 1 })
  return (next < self.size).ifTrue({ next }, ifFalse: { None })
}
```
Both `ifTrue(_, ifFalse:)` calls are two-armed (§2 precondition) — no `WrapSome`, no
`Option#map` allocation either (the old shape's `cursor.map { … }` allocated a **second**
`Some` via `Option#map`, `core.ph` L131, on top of the final `ifTrue`'s `WrapSome` — Route B
removes both). **Protocol constraint** (new, ADR-0048 §1): a cursor value may never itself be
`None` — vacuous for every kernel collection (integer indices) but must be stated in the
`Iterable` class doc comment for user subclasses.

### 3.2 `compile_for` retarget: one new opcode, `Bytecode::JumpIfNone` (ADR-0048 §2)
**Resolved per §2 precondition: no existing opcode expresses this; add
`Bytecode::JumpIfNone(i32)`.** Same shape as `Jump`/`JumpIfFalse`/`Loop` (a relative
instruction-count offset, `i32`, applied via the existing `apply_jump_offset` helper), so it
slots into `emit_forward_jump`/`patch_forward_jump`/`patch_forward_jump_to` with **zero**
changes to those helpers. `bytecode.rs`, insert near `Loop`/`GuardBool` (~L172):
```rust
/// Pops the top of stack; if it **is** the shared `None` singleton (tested by
/// identity — `Value`'s derived `PartialEq` on `Value::Obj(ObjRef)`, i.e. arena-slot
/// identity, never structural/`==` dispatch), adds `offset` to `ip` (see
/// `Bytecode::Jump` for the offset convention). Otherwise falls through, the popped
/// value already consumed. Realizes the cursor-protocol end-of-iteration test
/// (ADR-0048 §2, iteration.md §2) as one direct, non-overridable branch — `for`'s
/// loop condition is compiler-owned, not a `!=` send.
JumpIfNone(i32),
```
`vm.rs`, insert the handler beside `JumpIfFalse`'s (~L1716–L1726), mirroring its pop-then-branch
shape but with **no type-error path** (a cursor may be any `Value` variant; only equality to
the `None` singleton means "stop" — unlike `JumpIfFalse`, which type-errors on a non-`Bool`,
`JumpIfNone` is total over every `Value`):
```rust
Bytecode::JumpIfNone(offset) => {
    let cursor = self.pop()?;
    if cursor == Value::Obj(self.universe.classes.none_singleton) {
        self.apply_jump_offset(offset);
    }
}
```
`compiler/lib.rs::compile_for` (**L1421–L1514**), two edits, both confirmed necessary together
(see §5 build order — they cannot land independently green):
1. **Condition test** (currently L1448–L1451: `GetLocal` + `emit_getter_send("isSome", …)` +
   `emit_forward_jump(Bytecode::JumpIfFalse, …)`) becomes `GetLocal` +
   `emit_forward_jump(Bytecode::JumpIfNone, …)` — drop the `isSome` send entirely.
2. **Cursor extraction** (currently L1453–L1459: `GetLocal(coll)` + `GetLocal(cursor)` +
   `Constant(0)` + `emit_operator_send("unwrapOr", 1, …)` + `emit_operator_send("iteratorValue",
   1, …)`) becomes `GetLocal(coll)` + `GetLocal(cursor)` +
   `emit_operator_send("iteratorValue", 1, …)` — the bare cursor passes straight through, no
   `unwrapOr`/no `Constant(0)` (drop the now-dead `zero_idx` local).
The initial seed (`$coll.iterate(None)`, L1432–L1438) and the step (`$cursor =
$coll.iterate($cursor)`, L1493–L1497) are **unchanged** — both already push/consume `None`/the
raw cursor correctly under either protocol shape; only the *test* and the *unwrap* go away.
`emit_getter_send` (`compiler/lib.rs` L1367) stays alive — also used at L1674/L1751 for
destructuring-pattern `size` sends, unrelated to this edit; no dead-code fallout.

### 3.3 Kernel `Iterable` root; rehome the shared layer (ADR-0048 §3)
**`universe.rs`:**
- Add `pub iterable_class: ClassId` to `CoreClasses` (near `list_class`, **L907**), with rustdoc
  citing ADR-0048 §3.
- Insert `let iterable_class = make_core_class(heap, "Iterable", object_class, metaclass_class);`
  **before** the `list_class`/`map_class`/`set_class`/`tuple_class`/`range_class` lines
  (**L211–L229**) — `Iterable`'s own metaclass wiring only needs `object_class` already linked
  (true at that point), and it must exist before the five collections reference it as their
  superclass.
- Change each of the five `make_core_class(heap, "<Name>", object_class, metaclass_class)`
  calls' third argument from `object_class` to `iterable_class` (**L211/L218/L219/L224/L229**).
  This is a **pure Rust-side rewire** — `core.ph`'s `class List { … }` / `class Map { … }` / …
  reopen headers carry **no `extends` clause** today and none is added; the superclass is fixed
  at bootstrap, exactly as `object_class` is today.
- Add `("Iterable", c.iterable_class)` to the name→global-binding list (**L717–L721**) so `.ph`
  code (e.g. a future `class Countdown extends Iterable { … }`, iteration.md §1's own example)
  can resolve the identifier.
- `Iterable` itself carries **zero native primitive rows** — it is a bare class+metaclass shell
  in Rust; every method is attached by the `core.ph` reopen below (net floor delta unaffected by
  this class, ADR-0019 unchanged).

**`core.ph`:** insert a new `class Iterable { … }` block **before** `class List {` (**L280** —
i.e. immediately after the existing "Kernel List" doc comment block ending at L278, so
`Iterable`'s `FinalizeClass` runs — and its `base_names` index finalizes — before `List`'s
reopen finalizes and merges from it; `Statement::Class`'s `FinalizeClass` opcode, per its own
doc in `bytecode.rs` **L262–L276**, rebuilds a class's `base_names` from its own methods merged
with its **already-finalized** superclass index, so source order here is load-bearing):
```phalcom
class Iterable {
  // Generic index-cursor walk over `self.size` (ADR-0048 §1/§3). A subclass whose
  // cursor is not a 0..size index (none in-kernel today) overrides this.
  iterate(cursor) {
    let next = (cursor == None).ifTrue({ 0 }, ifFalse: { cursor + 1 })
    return (next < self.size).ifTrue({ next }, ifFalse: { None })
  }

  each(f) {
    for (x in self) {
      f.call(x)
    }
  }

  // map/filter/reduce/includes walk `iterate`/`iteratorValue` DIRECTLY, not
  // `self.each` — `Map` overrides `each` with an incompatible 2-arity (k, v)
  // selector (DEC-CT-E); routing these through `self.each` would silently call a
  // 1-arity block with 2 arguments the moment `Map` inherits them. See the rubric.
  map(f) {
    var result = List.new()
    var c = self.iterate(None)
    while (c != None) {
      result.add(f.call(self.iteratorValue(c)))
      c = self.iterate(c)
    }
    return result
  }

  filter(pred) {
    var result = List.new()
    var c = self.iterate(None)
    while (c != None) {
      var x = self.iteratorValue(c)
      pred.call(x).ifTrue({ result.add(x) }, ifFalse: { None })
      c = self.iterate(c)
    }
    return result
  }

  reduce(init, f) {
    var acc = init
    var c = self.iterate(None)
    while (c != None) {
      acc = f.call(acc, self.iteratorValue(c))
      c = self.iterate(c)
    }
    return acc
  }

  includes(x) {
    var found = false
    var c = self.iterate(None)
    while (c != None) {
      (self.iteratorValue(c) == x).ifTrue({ found = true }, ifFalse: { None })
      c = self.iterate(c)
    }
    return found
  }

  isEmpty => self.size == 0
}
```
Then, per collection (exact deletions — see §4 write-set table for the precise line ranges to
remove from each class body; nothing else in any of the five classes moves):
- **`List`**: **remove** `each` (L299–L303), `iterate` (L318–L321), `map` (L332–L336), `filter`
  (L341–L345), `reduce` (L352–L356), `includes` (L361–L365), `isEmpty` (L369) — all now
  inherited verbatim (behavior-preserving; `iterate` is the only one that also *changes*
  shape). **Keep**: `size`, `at`, `add`, `at(_,put:)`, `==`, `!=`.
- **`Map`**: **remove** `iterate` (L482–L485) only. **Keep `each`** (L470–L476, the 2-arity
  `(k, v)` override — do **not** hoist, do **not** delete). **Keep**: `size`, `at`,
  `at(_,put:)`, `includes` (own `rawHas`-backed O(1) override — Iterable's generic O(n)
  `includes` would still be *correct* here since `iteratorValue` yields keys per DEC-CT-E, but
  strictly worse; keep the override), `remove`, `keys`, `values`, `iteratorValue`, `==`, `!=`.
  **Gains** (new capability, inherited): `map`/`filter`/`reduce`/`isEmpty`, all walking keys
  (DEC-CT-E's already-established convention — `iteratorValue` yields the key).
- **`Set`**: **remove** `each` (L535–L541), `iterate` (L545–L548). **Keep**: `size`, `add`,
  `includes` (own O(1) override — keep), `remove`, `at`, `iteratorValue`, `==`, `!=`. **Gains**:
  `map`/`filter`/`reduce`/`isEmpty`.
- **`Tuple`**: **remove** `each` (L586–L592), `iterate` (L595–L598). **Keep**: `size`, `at`,
  `iteratorValue`, `==`, `!=`, `hash`. **Gains**: `map`/`filter`/`reduce`/`includes`/`isEmpty`.
- **`Range`**: **remove** `each` (L677–L683), `iterate` (L695–L698). **Keep**: `first`, `last`,
  `size`, `includes` (own O(1) bound-check override — **keep**, do **not** let the generic
  O(n) walk shadow it; subclass-defined wins by ordinary lookup so this is automatic as long as
  the line is not deleted), `at`, `toList`, `iteratorValue`, `==`, `!=`, `hash`. **Gains**:
  `map`/`filter`/`reduce`/`isEmpty`.

### 3.4 Native-vs-`.ph` split & the floor
- Protocol + combinators on `Iterable`: **pure `.ph`**, **0 new primitives** (ADR-0019
  unaffected — `Iterable` has no native rows).
- `for`'s loop-scaffold retarget: **1 new bytecode opcode** (`JumpIfNone`), 0 new primitives.
  This is a VM/bytecode-level addition, orthogonal to the ADR-0019 "floor primitive" axis (which
  tracks native *method* rows on classes, not opcodes) — same accounting precedent as U-ITER's
  own DEC-ITER-C (`Jump`, ultimately not needed there since it already existed; here the
  equivalent check comes up genuinely needed).
- **Net floor-primitive delta: 0.** One new opcode, flagged and justified above, not silently
  invented.

### Rubric — hazards & preclusion (mandatory)
- **`Map`'s `each` arity vs. inherited combinators (the load-bearing design hazard this unit
  must not get wrong).** `Map#each` is a genuine 2-arity `(k, v)` override, not a shape-variant
  of the generic 1-arity `each`. If `Iterable#map`/`filter`/`reduce`/`includes` were written
  over `self.each { x => … }` (as `List`'s *current* `map`/`filter`/`reduce`/`includes` are),
  `Map` inheriting them would dispatch to `Map`'s own 2-arity `each`, invoking a 1-arity block
  with 2 arguments — a silent arity mismatch, not a compile error. **Resolved** (§3.3): every
  `Iterable` combinator except `each` itself walks `iterate`/`iteratorValue` **directly**, never
  through `self.each` — immune to any subclass's `each` override by construction. Pin a `Map`
  golden that calls `.map`/`.filter`/`.reduce`/`.isEmpty` and asserts key-based results (not a
  crash / not silently wrong arity).
- **Per-class `includes`/`each` overrides must survive the hoist.** `Map#includes` (`rawHas`,
  O(1)), `Set#includes` (`rawHas`, O(1)), `Range#includes` (bound check, O(1)) and `Map#each`
  (2-arity) are **not** deleted even though `Iterable` now provides fallbacks/homonyms — ordinary
  method-lookup override semantics make the subclass version win automatically, but the
  *deletion* list in §3.3 is exact and excludes these four; do not "clean up" past it.
- **`for` ⊗ the Fiber generator (still the load-bearing check, iteration.md §6 / ADR-0030 §4).**
  `for`'s lowering stays an inlined `while` (no `.each`, no `block_call`) — this unit only
  changes the **condition test** (`isSome` send → `JumpIfNone` opcode) and the **extraction**
  (`unwrapOr` send dropped); it does not touch the jump-skeleton shape U-ITER built. Re-run/
  update the existing `for_disasm_no_block_call.ph` golden (`tests/lang/iteration/`) — it must
  still show **no `Closure(`** and now must show **`JumpIfNone(`** and **not** `"isSome"` /
  `"unwrapOr"` selector text in the disassembly.
- **`iterate`/`iteratorValue` stay non-inlined, ordinary sends (iteration.md §4).** Only the
  loop *scaffold*'s end-test is compiler-owned (`JumpIfNone`); a user `Iterable` subclass still
  drives `for` purely through overridable sends. Re-pin the iteration.md §1 `Countdown` example
  verbatim (now `extends Iterable`) as the non-`List` conformance golden.
- **`Iterable`-before-`List` compile order in `core.ph` is load-bearing.** `FinalizeClass`
  (`bytecode.rs` L262–L276) merges a class's own methods with its superclass's
  **already-finalized** `base_names` index; `class Iterable { … }` must textually precede
  `class List { … }` (and the other four) in `core.ph` so `List`'s `FinalizeClass` sees
  `Iterable`'s methods already indexed. Getting the order backwards would not necessarily fail
  loudly — it could leave `List`/`Map`/`Set`/`Tuple`/`Range` unable to resolve inherited
  `map`/`filter`/`reduce`/`includes`/`isEmpty`/`each` via the U16 Open-family fast path even
  though ordinary dispatch still finds them by walking `superclass` at miss time — pin a
  base_names-specific golden (a `Family`/`::` reference to `List::map`, U16) alongside the
  ordinary-call goldens so a silent base_names gap cannot hide behind ordinary dispatch working.
- **No-nil Invariant 4 preserved.** `None` is a real surface value; `JumpIfNone` tests it by
  **identity** (arena-slot equality on `Value::Obj`), never truthiness, and never touches the
  private `Value::Nil` sentinel (`sentinel_to_option`, `value.rs` L343, stays the only
  Nil→`None` boundary, untouched by this unit). State this explicitly in the return contract.
- **The cursor-may-never-be-`None` constraint is unenforced (by design, per ADR-0048).** No
  runtime check rejects a user `iterate` that returns `None` mid-sequence meaning "skip, not
  stop" — that would be a protocol violation the type system does not catch. Document it on
  `Iterable`'s doc comment; do not add an enforcement mechanism (explicitly out of scope, mirrors
  Wren).
- **Representation/dispatch impact:** one new `Bytecode` variant; no `Value` tag change, no
  selector-encoding change, no change to `SignatureKind`/method-lookup shape.
- **Precedent:** Wren's null-cursor rule (`iterate`/`iteratorValue` returning the sentinel
  directly, no wrapper) is the direct model (ADR-0048 §Alternatives already rejected the
  niche-encoding and dual-protocol alternatives — do not reopen).

## 4. Confirmed write-set (tight & disjoint; re-validate with `graphify affected` on HEAD)
| File | Why | Slice |
|---|---|---|
| `phalcom-core/src/bytecode.rs` | add `Bytecode::JumpIfNone(i32)` (§3.2) | bytecode |
| `phalcom-core/src/vm.rs` | dispatch handler for `JumpIfNone` (§3.2) | VM |
| `phalcom-core/src/compiler/lib.rs` **(SPINE — reviewer ON)** | `compile_for` retarget (§3.2), the two edits at L1448–L1451 / L1453–L1459 only | compiler |
| `phalcom-core/src/universe.rs` | `iterable_class` field + bootstrap wiring + name-binding row (§3.3) | bootstrap |
| `phalcom-core/core/core.ph` | new `class Iterable { … }` (before `List`); strip duplicated members from `List`/`Map`/`Set`/`Tuple`/`Range` per the exact per-class deletion list (§3.3) | protocol |
| `phalcom-core/tests/support/mod.rs` | extend/add a disasm-assertion helper (JumpIfNone present, `isSome`/`unwrapOr`/`WrapSome` absent) alongside the existing `check_for_no_block_call` | test harness |
| `phalcom-core/tests/lang.rs` | wire updated/new test fns | test harness |
| `phalcom-core/tests/lang/iteration/` (existing label, reused) | update `for_disasm_no_block_call.ph`'s assertions; add the Route-B zero-alloc probe, the `Countdown extends Iterable` golden, the per-class new-capability goldens (Map/Set/Tuple/Range `map`/`filter`/`reduce`/`includes`/`isEmpty`), the base_names/`Family` golden | goldens |

**Deliberately NOT in scope:** `phalcom-ast/*` (no surface syntax changes — `for`/`break`/
`continue` grammar is untouched), `primitive/*` (zero new native rows), `heap.rs`/`class.rs`
(no new `Value`/`Object` variant, no `ClassObject` shape change beyond the ordinary
`make_core_class` call already used for every kernel class), `docs/adr/*`/`docs/spec/*` (already
written — this unit is realization only), `docs/forge/DEFERRED.md` (flag DEC-ITER-A's closure
for the orchestrator/reviewer, do not edit it yourself), U-SEQ's `all`/`any`/`count`/`find`/
`join`/`toList`/lazy-view suite (iteration.md §5 lists it on `Iterable` but explicitly assigns
the lazy half to "the U-SEQ unit" — the mission for this unit pins exactly `iterate`/`each`/
`map`/`filter`/`reduce`/`includes`/`isEmpty`; treat the rest as a `DEFERRED.md`-flagged
follow-on, not scope creep here), `Option`/`Some`/`None` conforming to `Iterable`
(`values-and-absence.md`'s new §3.6 note that `Option` should conform later — not this unit),
U-NATIVE-MARKER's `raw*`→`*_` rename (separate unit, already has its own plan dir).

### 4.1 Write-set collision risk (flag, don't resolve)
- **`bytecode.rs`/`vm.rs`/`universe.rs`/`compiler/lib.rs`** were all mid-edit by a concurrent
  U16 session earlier the same day and are clean/committed again as of this plan (verified via
  `git status --porcelain`, commit `71c703d "feat(U16-Pinned): add pinned :: method references
  alongside Open"`). **Re-verify clean + re-diff line numbers immediately before dispatch** —
  this repo has continuous concurrent sessions landing on `main` (standing hazard, see
  `phalcom-concurrent-session-hazards` memory).
- **`phalcom-core/core/core.ph`** — never two editors. Confirmed clean at plan time (no U-CORE
  session currently touching it per `git status`), but **serialize** against any live U-CORE/
  U-COLL/U-STD-follow-on session before dispatch; this unit's edit spans five class bodies
  (List/Map/Set/Tuple/Range) plus a new class, the widest single-unit `core.ph` touch to date —
  do not attempt to land it concurrently with anything else touching those classes.
- **`compiler/lib.rs`** — spine file; confirm no concurrent unit holds it (U-ITER-FIX-style
  follow-ons, U9 variadics work, etc. have all touched this file historically) before dispatch.
- **`tests/support/mod.rs`/`tests/lang.rs`** — low risk but shared; keep the diff to
  additive/extension of `check_for_no_block_call`'s sibling helper, do not restructure existing
  helpers other units may depend on.

## 5. Build order (small, independently-green diffs)
1. **Opcode.** Add `Bytecode::JumpIfNone(i32)` + the `vm.rs` handler (§3.2). No caller yet — a
   dead-but-compiled variant. Green (`cargo build`/`clippy`/existing test suite unaffected).
2. **`compile_for` retarget + `Iterable` root + per-class rehome, together (see §2's
   precondition list — this seam is NOT independently splittable further).** These three edits
   are causally entangled: (a) `compile_for`'s new `JumpIfNone`-based test only works correctly
   once `iterate` returns a bare cursor, since the old `Some`-wrapped shape would make
   `JumpIfNone` never fire on a continuing step's Some-wrapped value being compared to `None`
   directly incorrectly extracted downstream — actually more precisely: (b) dropping
   `compile_for`'s `unwrapOr` step *requires* `iterate` to already return the bare cursor, or
   `iteratorValue` receives a `Some`/`None` object instead of an index and type-errors; and
   conversely (c) rewriting `core.ph`'s `iterate` to Route B *requires* `compile_for` to already
   drop `unwrapOr`/`isSome`, or the OLD desugar calls `.isSome`/`.unwrapOr` on a bare `Number`
   cursor (which has neither selector) and dNUs immediately. Land all three (opcode wiring from
   step 1 is prerequisite but already inert; this step activates it) in one commit, verified
   together: `for` over `List`/`Map`/`Set`/`Tuple`/`Range`/a `Countdown`-style user `Iterable`
   all still produce identical output to pre-unit behavior. Green.
3. **Goldens + disasm proofs.** Update `for_disasm_no_block_call.ph`'s assertions (`JumpIfNone(`
   present, `Closure(` absent, `"isSome"`/`"unwrapOr"` selector text absent); add the Route-B
   zero-`WrapSome` probe (§7); add the `Countdown extends Iterable` conformance golden (from
   iteration.md §1, verbatim); add the per-collection new-capability goldens (Map/Set/Tuple/
   Range gaining `map`/`filter`/`reduce`/`isEmpty`, Tuple gaining `includes`); add the
   base_names/`Family` inheritance golden (the rubric's `Iterable`-before-`List` ordering
   guard). Green.
4. **Cleanup.** Flag `DEFERRED.md`'s DEC-ITER-A row (`docs/forge/DEFERRED.md` L22) for the
   reviewer/orchestrator to strike or update — do not edit the file yourself (out of write-set).

Each step is a self-verifiable commit except step 2, which is atomic by construction (see its
own note) — do not attempt to sub-split it further; a half-landed state between the two halves
of step 2 is not compilable-and-correct, it is compilable-and-wrong (silent dNU or silent
wrong-value bugs, not a build failure), which is worse than a non-atomic build order.

## 6. Mandatory rules
- **Docs:** `///` on the new `Bytecode::JumpIfNone` variant and the `vm.rs` handler, citing
  ADR-0048 §2. `cargo doc --workspace --no-deps` adds no warnings.
- **Green gate:** `./scripts/verify.sh` exits 0; no new clippy; no `unsafe`.
- **Reviewer ON** (spine file `compiler/lib.rs`, plus a new opcode touching `vm.rs`'s dispatch
  loop) — `phalcom-reviewer` gates the diff; the writer never self-approves.

## 7. Test strategy (the green gate must assert) — reuse the `iteration` label
- **Route-B protocol (PASS):** direct `xs.iterate(None)` → the bare `0` (a `Number`, **not** an
  `Option`/`Some` instance — assert `xs.iterate(None).isA(Number)` or equivalent, not merely
  `== 0`, to catch an accidental `Some`-wrap regression); `xs.iterate(0)` → `1`; past-end →
  identically `None` (the singleton, `is` / identity-equal, not just `==`).
- **Zero-allocation structural proof (disasm, PASS):** a probe class/method reproducing the
  exact `Iterable#iterate` body (§3.1) disassembles with **no `WrapSome`** anywhere in its
  chunk — the direct structural proof that the two-armed `ifTrue(_, ifFalse:)` shape never
  Some-lifts, tying the `.ph` source shape to the compiled-bytecode guarantee.
- **`for` disasm (PASS — the load-bearing preclusion guard, updated from U-ITER):** the `for`
  loop's chunk contains `JumpIfNone(` + a `Loop(` back-edge + `iterate`/`iteratorValue` sends,
  and contains **neither** `Closure(` **nor** the selector text `"isSome"`/`"unwrapOr"`.
- **`for` over every kernel collection (PASS, behavior-preserving):** `List`/`Map`/`Set`/
  `Tuple`/`Range` each drive `for` correctly post-rehome (same visitation order/values as
  pre-unit); `Map`'s `for` still visits **keys** (DEC-CT-E unchanged).
- **`Countdown extends Iterable` (PASS — non-`List` conformance, iteration.md §1 verbatim):**
  drives `for` purely through the two protocol selectors, proving `Iterable` is a real
  extension point, not `List`-only plumbing.
- **New-capability goldens (PASS):** `Map`/`Set`/`Tuple`/`Range` each answer `map(f)`/
  `filter(pred)`/`reduce(init, f)`/`isEmpty` correctly (new this unit); `Tuple`/`Set` answer
  the generic `includes` correctly where they had none before; `Map`'s inherited `map`/`filter`
  operate over **keys**, not entries (must not silently break on the 2-arity `each` — the
  rubric's load-bearing hazard, pin explicitly with a block-arity assertion, not just a
  result-value assertion).
- **Override-precedence goldens (PASS — no regression):** `Map#includes`/`Set#includes`/
  `Range#includes` still take the O(1) path (assert via a large-N timing-insensitive proxy, e.g.
  a `rawHas`/bound-check call-count instrumentation if available, or at minimum a correctness
  golden that would catch an accidental deletion); `Map#each` still receives `(k, v)` 2-arity.
- **`break`/`continue` regression (PASS — U-ITER's own suite, must stay green unmodified):** no
  behavior change to loop-control jump targets; the only touch is the condition-test opcode and
  the removed `unwrapOr`, neither of which `break`/`continue`'s label machinery reads.
- **base_names / `Family` inheritance (PASS):** a `::`-reference (U16 Open/Pinned family) to an
  inherited combinator (e.g. `List::map`) resolves correctly, proving `FinalizeClass` picked up
  `Iterable`'s methods (the compile-order rubric hazard).
- **NEGATIVE:** none new — this unit changes no error surface (`iterate`/`iteratorValue` dNU
  paths, non-`Bool` `ifTrue` receiver errors, etc. are all pre-existing and unaffected).

## 8. Decisions flagged
| ID | Decision | Resolution |
|---|---|---|
| **DEC-ITERABLE-A** — resolved by grounding, per ADR-0048's own delegation ("the realizing unit resolves … flagged, not invented") | **`None`-identity test mechanism for `compile_for`.** | **Add `Bytecode::JumpIfNone(i32)`** (§3.2) — confirmed no existing opcode expresses it (full `Bytecode` enum read, 26 variants, none test identity/`None`). Reviewer should confirm this reading against HEAD at dispatch time (opcode set is a spine surface other concurrent units could in principle also be extending). |
| **DEC-ITERABLE-B** — resolved, not blocked | **Should `Iterable`'s `map`/`filter`/`reduce`/`includes` be built over `self.each` (matching `List`'s current style) or directly over `iterate`/`iteratorValue`?** | **Directly over `iterate`/`iteratorValue`** (§3.3) — building over `self.each` breaks under `Map`'s 2-arity override (the rubric's load-bearing hazard); the direct-cursor form is correct under every subclass's `each` override, including `Map`'s. |
| **DEC-ITERABLE-C** — flagged, not resolved (informational, not blocking) | **`Option`/`Some`/`None` conforming to `Iterable`** (`values-and-absence.md`'s new note: "`Option` conforms to `Iterable`, yielding zero or one element"). | **Not this unit.** `values-and-absence.md` explicitly defers it ("protocol finalized with the iteration work" — this unit *is* that work for collections, but the mission's write-set never names `Option`/`Some`/`None`). Flag as a small, obvious follow-on (`Some`/`None` each need a 2-line `iterate`/`iteratorValue`) — not BLOCKED, just out of this unit's write-set; note in the return contract so it isn't lost. |

No item here is **BLOCKED-ON-DECISION** — ADR-0048 and iteration.md already resolved every
design axis this unit touches; DEC-ITERABLE-A/B are architect-resolved-by-grounding (shown for
reviewer visibility, not user sign-off), DEC-ITERABLE-C is an explicit scope note.

## 9. Must-not-preclude check
- **U-SEQ (lazy views + `all`/`any`/`count`/`find`/`join`/`toList`, deferred-work.md's new row,
  deps U-ITERABLE):** actively *served* — `Iterable` is exactly the extension point U-SEQ needs;
  a lazy view class `extends Iterable`, implements `iterate`/`iteratorValue` over an upstream
  iterable, and inherits the whole eager combinator suite for free (Wren's
  `MapSequence`/`WhereSequence` model, cited directly in ADR-0048's consequences). This unit adds
  no eager/lazy coupling that would need undoing.
- **U-STRING (`codePoints`/`bytes` sub-iterables):** not precluded — a future `String` subclass
  or delegate iterable conforming to the same two selectors gets the same combinator suite with
  no compiler changes.
- **`Option` conforming to `Iterable` (DEC-ITERABLE-C):** not precluded — `Some`/`None` each need
  only a trivial `iterate`/`iteratorValue` pair once someone wires `Option`'s superclass (a
  `universe.rs` one-line change plus two `.ph` methods, exactly this unit's own shape in
  miniature); nothing here assumes `Iterable`'s only subclasses are the five kernel collections.
- **A future fail-fast mutation-during-iteration counter** (iteration.md §7's explicit
  non-goal): not precluded — `iterate`'s cursor is opaque to the loop scaffold; a collection is
  free to fold a modification-counter check into its own `iterate` override without any compiler
  change.
- **The deferred `Some` niche-encoding** (ADR-0044/deferred-work.md, now explicitly "no longer
  on the iteration hot path" per this unit's ADR-0048): not precluded, and this unit's own
  `deferred-work.md` diff already states the relationship — a future niche-encoding remains a
  pure `Option`-ergonomics change, orthogonal to iteration performance now.
- **U-NATIVE-MARKER's `raw*`→`*_` rename:** not precluded — this unit introduces no new `raw*`
  selectors (it only relocates existing ones' *callers*, e.g. `Iterable#map` now calls
  `self.iteratorValue`, never a raw accessor directly); the rename unit's mechanical sweep is
  unaffected by where a call site happens to live.

## 10. Return contract (report to `phalcom-reviewer`)
The `Bytecode::JumpIfNone(i32)` opcode + its `vm.rs` handler (identity test against
`universe.classes.none_singleton`, no type-error path) · the two `compile_for` edits (condition
test, cursor extraction) with the atomicity note from §5 step 2 honored · the `Iterable` class
(bootstrap wiring in `universe.rs` + the `.ph` body) and the exact per-class deletion list from
§3.3 applied to `List`/`Map`/`Set`/`Tuple`/`Range` · confirmation `Map#each`'s 2-arity override
is intact and `Iterable`'s combinators do not route through `self.each` (the rubric's load-bearing
hazard) · confirmation `Map#includes`/`Set#includes`/`Range#includes`'s O(1) overrides are intact
· the zero-`WrapSome` disasm proof and the updated `for`-loop disasm proof (`JumpIfNone(` present,
`Closure(`/`"isSome"`/`"unwrapOr"` absent) · the `Countdown extends Iterable` conformance golden ·
the per-collection new-capability goldens · the base_names/`Family` inheritance golden ·
confirmation **net floor-primitive delta = 0** (one new opcode, justified, tracked separately per
§3.4) · how DEC-ITERABLE-A/B were grounded (cite the exact `Bytecode` enum read proving no prior
opcode existed) and the DEC-ITERABLE-C scope note · a flagged pointer for the reviewer/
orchestrator to close `DEFERRED.md`'s DEC-ITER-A row (L22) and confirm the new U-ITERABLE/U-SEQ/
U-STRING rows already present in `deferred-work.md` need no further edit · `verify.sh` + `cargo
doc` tails.
