# PDR-0012 — The numeric tower lands: `Int`/`Float` implementation rulings and the floor amendment (137 → 153)

- Status: **Accepted** (ratified 2026-07-20, same day as proposed; ratified in the same
  pass as PDR-0011/PDR-0013, so ruling 21's rebase applies to all three off the 137 base)
- Date: 2026-07-20
- Amends: [ADR-0019](../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md) (the floor
  freeze — `docs/adr/` is frozen, so the amendment is carried here, the way ADR-0039/ADR-0049
  carried theirs when that folder was live, and the way [PDR-0011](0011-admit-bytes-native-octet-buffer.md)
  ruling 3 carries its own)
- Related: [ADR-0024](../adr/accepted/0024-numeric-surface-split-int-float-and-division.md)
  (Accepted 2026-07-12, **verified unbuilt 2026-07-20** — this record does **not** supersede or
  reopen it; §A restates its rulings as settled input and §B rules only what it left open),
  [ADR-0005](../adr/retired/0005-number-as-flat-f64.md) (its `f64` survives as `Float`'s
  representation), [ADR-0023](../adr/accepted/0023-amend-floor-admit-hash-and-kernel-reflection.md)
  (value-based `hash`; R-INV-1.3 is what ruling 10 protects),
  [ADR-0018](../adr/accepted/0018-sacred-selector-inliner-and-override-guard.md) (the inliner
  whose hardening ruling 17 orders against),
  [ADR-0009](../adr/accepted/0009-handle-arena-heap.md) / [ADR-0010](../adr/accepted/0010-tagged-value-enum.md)
  (the heap and `Value` this widens),
  [PDR-0001](0001-classes-are-closed.md) (reserved kernel names — ruling 15),
  [PDR-0011](0011-admit-bytes-native-octet-buffer.md) (**also amends ADR-0019**; the two
  compose — ruling 14)
- Spec: [`docs/spec/v0.2/core/numeric-tower.md`](../spec/v0.2/core/numeric-tower.md) holds the
  full implementation detail, phase order, write set, and test matrix. Normative upon
  ratification.

## Context

ADR-0024 ruled the numeric split on 2026-07-12: abstract `Number`, exact auto-promoting `Int`,
IEEE `Float`, value `==`, canonicalizing `hash`, true `/`, floor `~/`. It is unusually
complete on **semantics** and silent on **realization** — and it has sat at zero
implementation with no owning unit for eight days.

Writing the implementation spec surfaced work ADR-0024 does not cover, and three findings that
change what the unit costs:

1. **The lexer destroys the discriminant.** `Token::Number(f64)` (`lexer.rs:217`) — `1` and
   `1.0` are the same token before the parser ever runs. The old U12 plan
   (`docs/forge/units/U12/plan.md:38-44`) posed exactly this precondition, could not answer it,
   and scheduled around both branches hoping to stay compiler-side. It cannot. `phalcom-ast`
   is in the write set.
2. **The audited floor is 137**, verified by running `floor_census_matches_installed_bindings`
   green in a clean detached worktree at `8b4465c` — not read off a document. Two records
   disagree and both are stale: `floor-census.md:36,843` says **136** (missing `Fiber#isRoot`),
   and the brief this work was commissioned from said **125** (missing the whole `NEW_FIBER`
   amendment). `floor-census.md:843` states the rule that settles it: *"The test is the source
   of record for the count; do not restate the number here."*
3. **The per-class split doubles the numeric floor** (14 → 30), which under ADR-0019's one-way
   ratchet (`0019…md:26`) is an amendment, not a census bump. `floor-census.md` §7.1 fixes the
   protocol; `docs/adr/` is frozen, so it is carried here.

The unit is cheap **right now** for one reason that will not last: no arithmetic opcode exists
and the sacred-selector inliner is control-flow only (`inliner.rs:5-8`), so nothing in the VM
layer assumes `f64`. Ruling 17 is about keeping it that way until this lands.

## §A — Carried from ADR-0024 (settled; restated, not reopened)

Recorded here so this document is readable standalone. **These are not up for discussion in
this record.** Argue with ADR-0024 if you want them changed.

- **A1.** `Number` is abstract with `Int` and `Float` as its only concrete subclasses; it has
  no instances. Literals decide: `1` is `Int`, `1.0` is `Float`. (§1)
- **A2.** `Int` is exact and unbounded: a tagged `i64` small path, a heap `LargeInt` large
  path, one surface class over two hidden representations. No trap, no wraparound. (§2)
- **A3.** `==` compares by mathematical value (`1 == 1.0` is `true`); `hash` canonicalizes so
  an integral `Float` hashes as the equal `Int`. (§3)
- **A4.** `/` is always true division; `Int / Int` promotes to `Float`. `6 / 2` is `3.0`. (§4)
- **A5.** `~/` is integer division, spelled `~/` because `//` is the line comment, with **floor**
  semantics so its sign agrees with `%`. `-7 ~/ 2 == -4`. (§5)
- **A6.** `Int ⊕ Int → Int`; any `Float` operand contaminates to `Float`. (§6)

## Rulings

### Representation

1. **`Value::Number(f64)` is removed, not aliased.** `Value` gains `Int(i64)` and `Float(f64)`;
   the old arm ceases to exist. **No `_ =>` wildcard may be added to silence the fallout.**
   The exhaustiveness break is the migration's primary instrument, and it is load-bearing
   rather than merely convenient: ruling 11's `send_hash` defect is a `match` on a `Value`
   *return value*, invisible to review and to every test that does not exercise a `Map`
   insertion. Removing the arm is what surfaces it at compile time. Keeping `Number` as an
   alias would ship it.

2. **`Object::LargeInt(BigInt)` is stored inline, not `Box`ed.** `BigInt` is a sign plus a
   `Vec<u64>`; it fits under the `Object` slot ceiling that boxes `ClassObject` (280 B),
   `MethodObject`, `ModuleObject`, and `ClosureObject` (`heap/object.rs:29-45`). Same test
   PDR-0011 ruling 1 applied to `Bytes`. Its `trace_object` arm traces **nothing** — a `BigInt`
   holds no `ObjRef` — and must be written as an explicit commented no-op, never a fallthrough.
   *Caveat: the 32-byte figure is from general knowledge, not measured. Confirm against the
   slot rule before relying on it.*

3. **Demotion is mandatory and `Value::Int` is the canonical form.** Invariant:
   `∀ v : Value. v = Obj(LargeInt(b)) ⇒ b ∉ [i64::MIN, i64::MAX]`. Enforced in a single
   `normalize(BigInt) -> Value` — the only constructor of a `LargeInt` value in the tree —
   with a debug assertion and a Rust-level test. Without this, two equal `Int`s can hold
   different representations and every `==`/`hash`/`match` path needs a cross-representation
   case for a state that should be unreachable.

4. **Bind `num-bigint`; pin it in the root `[workspace.dependencies]`.** Do not hand-roll.
   Two corrections to how this was framed when commissioned:
   - It is **not** phalcom-core's first runtime dependency — the crate already carries
     `indexmap`, `slotmap`, `rand`, `lazy_static`, `clap`, `anyhow`, `color-print`, `thiserror`,
     `tracing`. The true and narrower claim: it is the first dependency that participates in
     **user-visible value semantics**, so it is the first whose behaviour is part of the
     language contract rather than an implementation convenience.
   - There is **no uniform pinning convention to match**. `indexmap`/`tracing` use
     `{ workspace = true }`; `thiserror`/`anyhow`/`lazy_static`/`rand`/`clap`/`color-print`/
     `slotmap` are pinned literally in the crate, and `thiserror` is pinned in *both* places.
     The workspace ruling stands on its own merit; do not justify it as consistency.

   **Do not take `num-traits` on speculation** — only if `ToPrimitive`/`FromPrimitive` are
   genuinely used. An unused dependency in the first value-semantics dependency is a bad
   precedent.

### Literals

5. **Split the token; do not tag it.** `Token::Int(i64)` and `Token::Float(f64)`, not
   `Token::Number(f64, is_float: bool)`. The payload types genuinely differ: an integer literal
   must not round-trip through `f64`, which is the exactness ADR-0024 exists to deliver.
   `phalcom-ast` enters the write set as a consequence — the branch U12's plan hoped to avoid.

6. **An integer literal too large for `i64` lexes as `Token::BigInt(String)`** and is parsed to
   a `BigInt` constant in the compiler. `Int` is unbounded, so `99999999999999999999` is a
   legal literal; it cannot be an `i64` and must not silently become an `f64`. Rejected:
   `Token::Int(BigInt)` (drags `num-bigint` into a dependency-free front-end crate for a rare
   case) and lexing it as an error (contradicts "`Int` is unbounded" in the single place a user
   most expects it to hold). ADR-0024 does not cover this case at all.

7. **No hex and no exponent literals in this unit.** The grammar has neither today
   (`lexer.rs:208-218` is exactly `scan_digits [ '.' scan_digits ]`) and ADR-0024 does not ask
   for them. Adding `1e5` means deciding whether it is `Int` or `Float` — a real question
   (Python/Ruby/Dart all say float) with nothing to do with the tower. Bundling a new syntax
   decision inside a representation change is how both get decided badly. It is *cheaper* after
   this lands: the `is_float` predicate will already exist and `1e5 → Float` becomes one arm.

### `~/`

8. **`~/` takes precedence 6, left-associative — the same as `/` and `%`**
   (`parser.rs:2841-2842`). Any other choice makes `a ~/ b * c` group differently from
   `a / b * c`, for two operators users will reach for interchangeably.

9. **No `~/=` compound assignment.** `SlashEqual`/`PercentEqual` exist; `~/=` is unrequested by
   ADR-0024 and costs a token, a lexer path, and a desugar for no stated need.

   Both selector-name sites must be updated in lockstep — `parse_method_name`
   (`parser.rs:1441`) and the `super.<operator>` arm (`parser.rs:2210`). The parser's own doc
   comment at `parser.rs:2171-2175` records that these diverged once already (U-ERR-FIX
   SUPER-OP-SYNTAX), which is why this is a ruling and not an implementation note.

### Arithmetic

10. **`Int#%` is floored, not truncating — and `Float#%` is unchanged.** This is *forced*, not
    chosen: ADR-0024 §5 rules `~/` as floor, and the identity
    `a == (a ~/ b) * b + (a % b)` then requires `-7 % 2 == 1`. Rust's `i64 %` truncates and
    gives `-1`. The current `number_mod` doc (`primitive/number.rs:142-147`) says "sign follows
    the dividend" — correct for `Float`, wrong for `Int` the day this lands.

    **This is the change most likely to be mistaken for a regression**, because it is
    user-visible, affects only negative operands, and appears nowhere in ADR-0024's text. It
    needs its own golden over a negative-operand table, and the divergence from `Float#%` must
    be pinned too.

11. **`~/` and `Int#%` raise on a zero divisor.** `/` keeps IEEE semantics (`1 / 0` is `inf`)
    because it promotes to `Float` first — existing arithmetic goldens are unaffected
    (`primitive/number.rs:106-109` records that they pin this). `~/` has no such escape: there
    is no integer infinity. *Unverified: episodic memory reports a defined-but-unused
    `RuntimeError::ZeroDivision`; confirm before adding a variant.*

12. **The small-integer path must not route through `BigInt`.** One promotion lattice in one
    helper (U12's guardrail, `U12/plan.md:30-33` — no scattered `match (Int, Float)`), but
    `(Int, Int)` is a `checked_*` fast path ahead of it, promoting only on overflow. A `BigInt`
    allocation per `i64 + i64` would make the common case slower than the `f64` it replaces —
    a self-inflicted regression on a codebase that measures its arithmetic.

### Equality and hash

13. **`Float#hash` canonicalizes at every magnitude, not just below 2^53.** The existing guard
    (`primitive/number.rs:65`) hashes an integral `f64` as an integer only when
    `n.abs() < 9_007_199_254_740_992.0`, falling back to raw bits above it.

    **The split turns that bound into a live defect.** `2.0f64.powi(100)` is finite, integral,
    and exactly equal to the `Int` `2^100` — so under ADR-0024 §3 they are `==`, but they land
    in different branches and hash differently. That breaks `a == b ⇒ a.hash == b.hash`
    (R-INV-1.3, ADR-0023) and silently desyncs `Map`/`Set` keys. It is harmless today only
    because there is nothing for a large integral float to *be* equal to.

    This correction is derived from ADR-0024 §3's own rule; the ADR does not state it.

14. **`send_hash` must accept `Value::Int`.** `primitive/mod.rs:338-348` matches
    `Value::Number` and errors on anything else, and `Map`/`Set` hash their keys by **sending
    the Phalcom `hash` selector** through it (`primitive/map.rs:55`; the module doc at
    `map.rs:12-15` calls it "the re-entrant key-hash crux"). The moment `hash` returns an `Int`,
    every `Map` and `Set` insertion fails. A user-defined `hash` returning an integral `Float`
    is accepted for compatibility; non-integral is an error.

15. **`hash_code`'s 53-bit mask stays.** `primitive/mod.rs:150-155` masks digests so the
    `as f64` cast round-trips — a constraint this record removes. Widening is now *available*
    and buys nothing: digest stability is only required within a run (R-INV-1.4), and changing
    every hash value inside a change already touching `Map`/`Set` key paths is unforced risk.
    The doc comment's stated *reason* must still be corrected, or it becomes a lie about why
    the mask exists.

16. **`impl Hash for Value` must stay coherent even though no consumer was found.** A search
    for `HashMap<Value`, `HashSet<Value`, and `ConstKey` across `phalcom-core/src` returned
    **nothing**. That is reported as a failed search, **not** as a finding that the impl is
    dead. This repo has already shipped a use-after-free because an audit concluded "zero sites
    need this" from a predicate too narrow to see them. Keep it coherent; if an implementer
    establishes it really is unreachable, that is a separate result worth writing down.

### The tower and the floor

17. **The `toString` pristine flag splits into two.** `note_method_installed`
    (`universe/mod.rs:196-198`) sets `number_tostring_pristine` by comparing `ClassId` against
    `number_class`. After the split, an override of `Int#toString` would **never match** that
    comparison, so the flag would stay `true` and the `value/render.rs:110` fast path would keep
    skipping dispatch — **the user's override would be silently ignored**. That is a
    correctness bug, not a missed optimization. Two flags: `int_tostring_pristine` and
    `float_tostring_pristine`, both snapshotted in the post-`core.ph` reset.

18. **`Number.new` is re-homed, not deleted.** `Number` becomes abstract, so
    `Number.class::new()`/`new(_)` cannot survive; they become `Int.new()`/`Int.new(_)` and
    `Float.new()`/`Float.new(_)` (+4 on the floor). Rejected: deleting them and pushing string
    parsing to `String#toInt`/`String#toFloat`, which is *also* +2 floor, lands in U-STRING's
    territory, and changes a public spelling for no gain. Fix U12's adopted debt in the same
    pass — `primitive/number.rs:34` still hardcodes `found: "value"` instead of
    `arg.type_name()`, `TODO` intact.

19. **`Number` keeps zero floor bindings but stays in the census and in `core_class_rows`.**
    Not vestigial: it is a tripwire. A future primitive accidentally installed on `Number`
    re-flattens the tower, and the census row is what makes that a red test instead of a silent
    regression.

    `core_class_rows` goes **29 → 31**. `floor-census.md` §7.2's coverage caveat exists because
    `Fiber` slipped exactly this gap and sat unaudited for the whole life of the fiber work:
    *"a kernel class missing from it is unfrozen in fact, whatever ADR-0019 says."* The array's
    length is in its type signature, so forgetting it is a compile error — the one piece of
    luck in this design.

20. **The floor amendment: 137 → 153.** Per-class split, `Int` and `Float` each taking the full
    arithmetic/comparison surface plus `~/`, `hash`, `toString`, and both `new` arities:

    ```
      137   audited today (invariants.rs:642-723; test run green at 8b4465c)
     − 14   Number emptied (12 instance + 2 static)
     + 26   Int (13 instance) + Float (13 instance)
     +  4   Int.new()/new(_) + Float.new()/new(_)
     ────
      153
    ```

    **152** if Q-1 below resolves `~/` as `Int`-only. The numeric floor goes **14 → 30**.

    **The justification, stated so it can be argued with:** the split adds **no new
    capability** — all 30 bindings are existing blessed capabilities, re-homed. The count
    doubles for a *representation* reason: under the per-class ruling there is no shared
    implementation to inherit, so one binding becomes two. A reviewer may reasonably answer
    that this is the ratchet working as designed and the doubling is the price of the class
    structure. That is the trade; this record does not pretend otherwise.

    **Implementation note.** Every constant in `floor_census_matches_installed_bindings` is a
    `usize` *addition*. This is the first unit to **remove** bindings. Do not fold the removal
    into a smaller positive constant — that erases the fact that `Number` was emptied. Add a
    named `NUMERIC_SPLIT_REMOVED` alongside `NUMERIC_SPLIT_ADDED`, keeping the addition ahead
    of the subtraction so the expression never goes negative mid-evaluation.

21. **This amendment and PDR-0011's are independent and additive.** PDR-0011 also amends
    ADR-0019 for `Bytes`, taking the floor `137 → 147`. Both are Proposed against the same
    measured baseline of **137**, and they touch disjoint classes, so whichever ratifies second
    rebases onto the other: `Bytes` first gives `147 → 163`; this one first gives `153 → 163`.
    **Neither may quietly restate 137 as its base after the other lands** — that is how a
    census silently loses bindings. The test's per-amendment constants exist to make the
    composition explicit; that is what they are for.

    *This hazard is not hypothetical: PDR-0011 moved from six primitives (`137 → 143`) to ten
    (`137 → 147`) during the hours this record was being written. Only the shared 137 baseline
    and the named constants make that safe to absorb. **Recompute the composed figure against
    both records at ratification time — do not trust the number in this sentence.***

22. **Kernel-name reservation is inherited, not built here.** [PDR-0001](0001-classes-are-closed.md)
    ruling 3 reserves "the name set enumerated in `add_class!`". `Int` and `Float` get
    `add_class!` rows, so they become reserved automatically once PDR-0001 ships; this record's
    only obligation is the rows. PDR-0001 ruling 4 (stub completion gated on the core module) is
    the *sanctioned* path for layering `.ph` protocol onto Rust-installed kernel classes, which
    is exactly what `Int`/`Float` do. **Ordering is free in both directions** and neither
    blocks the other.

### Scope and ordering

23. **Call-site tightening is out of scope.** ADR-0024's Consequences promise that a `Float`
    index is a type error at the boundary. That touches every collection primitive and a large
    fixture corpus, and it is separable — the tower is correct without it. The line:
    `size` **returning** `Int` is in scope (choosing the right constructor); `list.at(2.0)`
    **being rejected** is not (tightening an accept-predicate). `expect_index`
    (`primitive/list.rs:29`) therefore stays permissive, gaining an `Int` arm while keeping its
    `Float` one, with a doc comment naming the follow-on unit. The follow-on is cheap *because*
    of this ordering: `size` already returns `Int` and indices already are `Int`, so it only
    removes an arm and fixes fixtures.

24. **This must land before any arithmetic fast path is burned into bytecode**, per ADR-0024
    §Context. The window is open — and the reason matters, because it is not the reason
    recorded anywhere:

    - The sacred-selector inliner is **control-flow only** (`inliner.rs:5-8`).
    - **DEC-PRIM-B** (the guarded arithmetic fast path) was deferred from U-PRIM-ABI to U-IC —
      the on-stack arg buffer alone won ~41% on `arith_send` (`docs/forge/UNITS-TRACKER.md:129`).
    - **U-IC then dropped it.** `U-IC/implementation-spec.md:16` records Change 3 as DROPPED:
      the Wren technique targets a `u8` bytestream and Phalcom's `Bytecode` is a `Copy` enum
      with inline operands, so "it is a no-op here".

    So the arithmetic fast path is not merely unbuilt — **it has no owning unit at all**, while
    `SCOREBOARD.md:438` still lists `vm::send::call_method` (4.8%) as `open — DEC-PRIM-B`. The
    window is open because the work fell between two units, not because anyone is holding it
    open deliberately.

    **An unowned constraint is an invisible one.** `docs/forge/units/U-IC/plan.md` must record
    that Change 3 was dropped and that any successor arithmetic fast path is gated on this
    record — written **now**, not when the fast path is finally scheduled, because by then
    there is nobody left who knows.

## Consequences

- **Integers are never silently wrong.** The point of the split (ADR-0024).
- **Negative-operand `%` changes on the `Int` path** (ruling 10) — the least obvious
  user-visible consequence, forced by ADR-0024 §5 rather than chosen here.
- **`6 / 2` is `3.0`.** ADR-0024's deliberate cost; `~/` is the tool when an `Int` is wanted.
- **The numeric floor doubles** (ruling 20). Requires ratification; not assumed.
- **`phalcom-ast` enters the write set** (ruling 5) — this is no longer the compiler-side unit
  U12's plan hoped for.
- **Arithmetic gets slower before it gets faster.** Two arms and a promotion lattice replace
  one `f64` op on a hot path with no fast path yet (ruling 24). Expect a measurable
  `arith_send` regression and **measure it deliberately** — a predicted regression recorded now
  costs less than an unexplained one found later. Numbers come from
  `docs/forge/perf-log/SCOREBOARD.md` only.
- **`Bytes` gets its element contract for free.** PDR-0011 ruling 2 states its element type
  representation-independently ("an integer in 0–255") precisely so this record can land
  without a surface change there. It holds: those values become small `Int`s.

## Alternatives rejected

ADR-0024's own alternatives (f64-backed tag, trap-on-overflow `i64`, wraparound, surface
`SmallInteger`/`LargeInteger`, flooring `/`, truncating `~/`) are settled there and not
relitigated. New to this record:

- **A shared `Number` implementation dispatching on the arm internally.** Would hold the floor
  near 15 and avoid this amendment entirely. **Rejected by user ruling**, and the ruling is
  defensible on merit: a shared `Number#+` misdescribes the tower — `Int` and `Float` have
  genuinely different arithmetic (exact/promoting vs IEEE-754), and one binding means the class
  a user overrides is not the class that implements the behaviour. Recorded so ruling 20 can be
  argued on the real trade rather than on the count alone.
- **Keeping `Value::Number` as a third arm through the migration.** Rejected: ruling 1's
  exhaustiveness break is the migration's best instrument, and ruling 14 is a live defect that
  *only* surfaces because the arm disappears.
- **Deleting `Number.new` outright** — rejected, ruling 18.
- **Widening `hash_code`'s mask in this unit** — rejected as scope, ruling 15.
- **Adding hex/exponent literals here** — rejected, ruling 7.

## Open questions — must be settled before or during implementation

| # | Question | Recommendation |
|---|---|---|
| **Q-1** | **Is `~/` defined on `Float`?** ADR-0024 §5 says it "returns an exact `Int`" without restricting the receiver; §6 says any `Float` operand contaminates to `Float`. Both cannot hold for `7.5 ~/ 2`. Genuinely unruled — this record does not guess. | Define it on `Float`, returning `Int` (Dart precedent; §5's wording is unconditional). Floor semantics; raise on non-finite operands. Floor +1 → 153. |
| **Q-2** | **`Int.new(2.7)`** — truncate, round, or raise? ADR-0024 has no ruling, and it is the one place a user can request a lossy narrowing. | Lean **raise** — it matches "integers are never silently wrong". Truncation is defensible if an explicit narrowing door is wanted, but then it belongs as `Float#truncated`, not hidden in a constructor. |
| **Q-3** | **Ratify the amendment in ruling 20?** This is what the record's Proposed status turns on. | Ratify — the cost follows from a class-structure ruling already made. The user's call; not assumed. |
| **Q-4** | Does the constant pool's GC rooting cover a compile-time-minted `LargeInt` `ObjRef` (ruling 6)? **Not verified.** | Verify before the primitives phase; root it if not. |
| **Q-5** | Should `expect_index`'s transitional `Float` arm carry a machine-checkable tripwire rather than only a doc comment (ruling 23)? | A doc comment naming the follow-on is probably enough; a tripwire is cheap insurance against the arm becoming permanent. |
| **Q-6** | `phalcom-core`'s dependency pinning is split between workspace and crate-literal, with `thiserror` in both (ruling 4). Normalize? | Separately — unrelated to the tower, and bundling it hides a real cleanup inside a semantics change. |

> **Q-1 and Q-2 are discharged by [PDR-0025](0025-numeric-tower-residue-rulings.md)** (Proposed
> 2026-07-20, same day): `~/` is total over the tower and returns `Int` (the stated exception to
> §A A6); `Int.new` never narrows (any `Float` argument raises); and the re-homing of ruling 18
> drops `number_class_new`'s undocumented `Bool` arm. Q-3 was discharged by this record's own
> ratification. Q-4/Q-5 remain implementation-time checks; Q-6 remains parked.

## Verified vs assumed

**Verified at `8b4465c`, with `file:line`:** the single `Value::Number` arm; `core.ph:82`'s flat
`class Number {}`; the total absence of `~/` from the token set; the number-literal grammar
(no hex, no exponent) and the **loss of the int/float discriminant at lex time**; `Number`'s
12 instance + 2 static bindings; the floor count **137** (test run green in a clean detached
worktree, not read off a doc) and both stale records; the inliner's control-flow-only sacred
set; every collateral site in ruling 13/14/15/17; `Map`/`Set` hashing via the `hash` **send**;
`patterns.rs`'s four minted constants; U12's adopted-debt `TODO` still present at
`number.rs:34`; PDR-0001's rulings 3 and 4 and its unimplemented status; `core_class_rows`'s 29
rows; phalcom-core's existing dependency list; DEC-PRIM-B's deferral to U-IC **and U-IC's
subsequent drop of it**; PDR-0011's 137 baseline and its `137 → 143`.

**Assumed, not verified:** that `//` is genuinely unavailable as the line-comment token (taken
from ADR-0024 §5); that `RuntimeError::ZeroDivision` exists and is unused (episodic memory
only); that nothing else claims `~` in the lexer; `BigInt`'s in-memory size (ruling 2); the
current keyword table after U-BINDINGS; whether `impl Hash for Value` has a live consumer
(ruling 16 — a failed search, deliberately not a conclusion). Every performance statement in
Consequences is a prediction.
