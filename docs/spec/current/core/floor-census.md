# Primitive Floor Census (U-CORE-0)

> **Status:** Normative. This document is the authoritative enumeration of the
> **VM-blessed primitive floor** frozen by
> [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md). It is an
> *audit* of what `install_primitives` actually binds, not a wishlist. Any
> change to the set below is an ADR-0019 amendment (see §7), not an ordinary
> commit.

## 1. What the floor is

A **floor primitive** is a method implemented in native Rust and bound onto a
kernel class at bootstrap, because it cannot be expressed in Phalcom over
lower-level Phalcom (it touches the heap representation, an immediate value's
bits, control flow, or I/O). ADR-0019's rule: the floor is *closed*. Every new
core capability must be either

1. **derivable** — written in `core.ph` in terms of the selectors below, or
2. **an amendment** — a deliberate, ADR-recorded widening of the floor.

The default answer to "add a primitive" is **no**. The floor exists so the
language is *self-hosting above a small, fixed native boundary*
([`../experimental/bootstrapping-and-self-hosting.md`](../experimental/bootstrapping-and-self-hosting.md)
§D1).

### 1.1 Two counts that differ

- **Installed bindings** — one per `(class, selector)` pair added by
  `install_primitives`. The shared `call(***)` gateway is bound once on
  `Function` and inherited by concrete callable classes.
- **Distinct native functions** — the Rust `fn` behind the install-time
  bindings. This is not a stable census metric: several selectors share one
  function, and bootstrap may replace an installed native entry with a
  Phalcom implementation before the live floor is measured.

| Metric | Count |
|---|---|
| Installed `(class, selector)` bindings — **all audited** (§1.3) | **150** |
| Distinct native Rust functions | not separately maintained |
| Classes carrying floor primitives | **23** (of 29 audited kernel classes) |
| Sacred selectors (§5) | **7** |

**Installed = audited, as of 2026-08-11.** Every native binding `VM::new()` installs is
enumerated in §2 and guarded by R-INV-0.1. That is a new property, not a standing one —
until CB-5 closed, `Fiber`'s 11 were installed but audited by nothing (§1.4).

### 1.3 The source of record is the test, not this file

**Never quote a floor count from prose — including this file's.** The authority is
[`phalcom-core/tests/invariants.rs`](../../../../phalcom-core/tests/invariants.rs)
`floor_census_matches_installed_bindings` (R-INV-0.1): it reconstructs the installed
`(class, selector)` set from a live `VM::new()`, asserts it equals the enumeration in §2,
and asserts the total equals the sum of its own per-amendment constants. It is
machine-checked and green; the numbers above are a *rendering* of it, and drift the moment
an amendment lands without editing this file. If the two disagree, **the test is right**.

An amendment must bump the test's constant in the same change that edits §2 — that is what
turns floor drift into a red test rather than silent prose rot.

> **Reconciled 2026-07-15 (CB-2).** §1.1 read **113** and §7 read **117** while the test
> asserted **125** — three numbers, no two alike, in the one document every ADR is told to
> cite instead of quoting its own figure. The 12-binding gap was exactly the five
> amendments that landed without updating §1.1 (`NEW_SCHED` 2, `NEW_INVARIANT_GUARD` 2,
> `NEW_ATTR_ROOT` 3, `NEW_GC` 1, `NEW_STRING` 4); §7 had been carried forward two of those
> five and then abandoned. "Distinct native Rust functions" was likewise stale at **98**
> (actual **110**). See [`docs/forge/DEFERRED.md`](../../../forge/DEFERRED.md) CB-2.

### 1.4 `Fiber` was outside this census until 2026-07-15 — how, and why it matters

**Closed (DEFERRED CB-5): `Fiber` is now audited (§2.17).** Kept as a record because the
*shape* of this hole is the one to watch for, and the fix is only one class deep.

For as long as fibers had shipped, `VM::new()` installed **136** native bindings while the
census enumerated **125** and R-INV-0.1 audited those same 125. The missing 11 were
`Fiber`'s — a real kernel class (`universe/core_classes.rs:152`) carrying real primitives
(`universe/primitives.rs` L362-374), **absent from the test's `core_class_rows` and from
this document entirely**. So ADR-0019's freeze did not bind `Fiber`: a primitive added to,
or dropped from, it changed the floor and **no test went red**.

**Why nothing caught it.** The census and the test agreed with each other perfectly
(125 = 125, green), so every consistency check *between those two* passed. Neither was ever
compared against the install site — and a class that appears in neither cannot be missed by
either. The gap was found by reconciling `install_primitives` against the audit set by hand
while fixing CB-2, and it surfaced as an unexplained 11-binding discrepancy, not as any
failure.

**The generalisable lesson.** Two artifacts agreeing is not evidence that either is right;
it is evidence that they are *coupled*. R-INV-0.1's authority comes from comparing prose to
a **live `VM::new()`** — but only over the classes it is told to look at. `core_class_rows`
is the audit's true boundary, and nothing audits *it*. If a future kernel class is created
without a row there, this hole reopens silently and identically. **When adding a kernel
class that carries primitives, add its `core_class_rows` row in the same change** — that
row, not the count, is what makes the freeze real.

> **U-CORE-1 amendment ([ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)).**
> Kernel reflection admits **+7** bindings (73 → 80) and **+7** distinct fns
> (57 → 64): `Object#hash` (`object_hash`), per-immediate `hash` overrides on
> `Number`/`String`/`Bool`/`Symbol` (`{number,string,bool,symbol}_hash`), and
> `Behavior#name`/`Behavior#methods` (`behavior_name`/`behavior_methods`).
> Floor-carrying classes stay at **16** — `Behavior` already carried
> `superclass`. `Object#isA(_)` is **not** on this list: it is derived in
> `core.ph` over `class`/`==`/`superclass` (ADR-0019 §1), not a native
> primitive. R-INV-0.1 (`tests/invariants.rs`) now audits this set from a live
> `VM::new()` and fails on drift.

> **U-CORE-3 amendment ([ADR-0028](../../../adr/0028-amend-floor-admit-method-reflection.md)).**
> The `Method` reflection surface admits **+5** bindings (80 → 85) and **+5**
> distinct fns (64 → 69): `Object#methodFor(_)` (`object_method_for`),
> `Method#invokeOn(_,***)` (`method_invoke_on_shape`), `Method#bind(_)`
> (`method_bind`), `Method#selector` (`method_selector`), `Method#holder`
> (`method_holder`). Floor-carrying classes stay at **16** — `Object` and
> `Method` already carried primitives. Also adds one **heap representation**,
> `Object::BoundMethod` (surface class `Block`), the value `bind(_)` returns —
> not a new `Value` arm, so it changes no count in this table.
> `block_arity`/`block_name`/`resolve_callable`/`block_call` learn the
> `Object::Method` and `Object::BoundMethod` receivers as **behavior
> completions** (zero new bindings). R-INV-0.1 (`tests/invariants.rs`) audits
> this set from a live `VM::new()` and fails on drift.

> **U-CORE-4 amendment (ADR-00NN, floor amendment; number claimed at dispatch
> time — see `docs/adr/` for the current max).** Value-class `toString`
> (catalog-delta.md §4.4) admits **+1** binding (85 → 86) and **+2** distinct
> fns (69 → 71): `Number#toString` (`number_to_string`) is the one new floor
> primitive — rendering an `f64` as decimal text is unreachable from `.ph`, the
> same derivability failure as `hash` (decisions.md Q1). `Object#toString` is
> **re-homed** off `object_name` onto a new, distinct fn `object_to_string`
> (ADR-0015's `"<ClassName>"` default + class-own-name fix, DEFERRED F4) — the
> `(Object, toString)` binding itself is unchanged, so this contributes to the
> distinct-fn count but not the binding count. `String#toString` (`=> self`),
> `Bool#toString` (over `ifTrue(_, ifFalse)`), and `Option#toString` (over
> `match`) are **derivable** and stay in `core.ph` — not floor amendments.
> Floor-carrying classes stay at **16** — `Object` and `Number` already carried
> primitives. R-INV-0.1 (`tests/invariants.rs`) audits this set from a live
> `VM::new()` and fails on drift.

> **U-CORE-6 amendment ([ADR-0037](../../../adr/0037-amend-floor-admit-error-root.md)).**
> The minimal `Error` reification (object-model.md §4 "Errors",
> [ADR-0008](../../../adr/0008-layered-exceptions-and-result.md)) admits **+2**
> bindings (86 → 88) and **+2** distinct fns (71 → 73): `Error#message`
> (`error_message`) and `Error#raise` (`error_raise`) — both new native
> functions, no rehome subtlety. Floor-carrying classes move **16 → 17**: the
> new `Error` row is the first of the two new kernel classes
> (`Error`/`MessageNotUnderstood`) to carry a primitive —
> `MessageNotUnderstood` carries none of its own (it inherits `message` from
> `Error`), so it does not bump the count further. Producing the
> `RuntimeError::Raise` payload the dNU miss now raises through is **plumbing,
> not itself a bound selector** (ADR-0023 Decision §4) — it does not count
> toward either metric. R-INV-0.1/R-INV-6.5 (`tests/invariants.rs`) audit this
> set from a live `VM::new()` and fail on drift.

> **U-COLLTYPES Phase 1 amendment ([ADR-0039](../../../adr/0039-amend-floor-admit-collection-container-primitives.md)).**
> The `Map`/`Set` hash-collection floor admits **+14** bindings (88 → 102) and
> **+14** distinct fns (73 → 87): `Map` — `new()`, `_$size`, `_$get(_)`,
> `_$put(_,_)`, `_$has(_)`, `_$remove(_)`, `_$keyAt(_)`, `_$valueAt(_)`
> (8, `primitive/map.rs`); `Set` — `new()`, `_$size`, `_$add(_)`, `_$has(_)`,
> `_$remove(_)`, `_$at(_)` (6, `primitive/set.rs`). `Set` shares `Map`'s Rust
> backing struct ([`MapObject`](../../../../phalcom-core/src/map.rs), DEC-CT-B)
> but every binding is its own distinct native fn — no rehome subtlety.
> Floor-carrying classes move **17 → 19** (`Map`/`Set` are new rows, neither
> previously carrying a primitive). `_$get`/`_$put`/`_$has`/`_$remove`
> re-enter the VM to send Phalcom `hash`/`==` on keys (not Rust `Value: Hash`)
> and `_$put`/`_$add` reject a mutable-collection key (DEC-CT-C,
> collection-protocol.md law 4). R-INV-0.1 (`tests/invariants.rs`) audits this
> set from a live `VM::new()` and fails on drift.

> **U-COLLTYPES Phase 2 amendment ([ADR-0039](../../../adr/0039-amend-floor-admit-collection-container-primitives.md)).**
> The `Tuple` floor admits **+3** bindings (102 → 105) and **+3** distinct fns
> (87 → 90): `_$fromList(_)` (class-side, `tuple_from_list_internal`), `_$size`
> (`tuple_raw_size`), `_$at(_)` (`tuple_raw_at`) — all in `primitive/tuple.rs`.
> **No mutation primitive** — immutability is a representation guarantee
> ([`TupleObject`](../../../../phalcom-core/src/tuple.rs)'s `Box<[Value]>`, no
> mutable accessor exists). Floor-carrying classes move **19 → 20**. `hash`
> stays `.ph` (DEC-CT-D: an order-sensitive fold over `_$at`+element `.hash`,
> zero new floor) — it is **not** a binding here. R-INV-0.1 audits this set.

> **U-COLLTYPES Phase 3 amendment ([ADR-0039](../../../adr/0039-amend-floor-admit-collection-container-primitives.md)).**
> **Superseded for Range by C.2.** Direct `BuildRange` bytecode constructs the
> bounds descriptor; only `_$lower`, `_$upper`, and `_$upperInclusive` remain as
> native observations. They surface omission as `Option`, preserving a present
> `None` endpoint. Progression and Range equality/hash remain deferred.

> **U-ERR amendment ([ADR-0038](../../../adr/0038-amend-floor-admit-block-on-ensure.md)).**
> The error-handling catch protocol admits **+2** bindings (109 → 111) and
> **+2** distinct fns (94 → 96): `Block#on(_,_)` (`block_on`) and
> `Block#ensure(_)` (`block_ensure`), both `primitive/block.rs`. Installed on
> `Block` only (mirroring `whileTrue(_)`, not `call`/`arity`/`name`/
> `callWith`) — every `on`/`ensure` receiver, whether at a `try` desugar site
> or inside `Function#attempt`, is always a literal `{ }` block. This is the
> **whole** floor for error *handling* (the *raising* half, `Error#message`/
> `raise()`, already landed under ADR-0037): `throw`, the `try`/`on`/`catch`/
> `ensure` statement, `Result`/`Ok`/`Err`, and `Block#attempt` are all parser
> sugar / pure `.ph` over these two plus `Error#raise` — **zero** further
> bindings. Floor-carrying classes stay at **21** (`Block` already carried
> `whileTrue`). R-INV-0.1 audits this set.
>
> **U15 amendment ([ADR-0045](../../../adr/0045-module-import-relative-path-whole-module-binding.md)).**
> The `import` member-access miss path admits **+1** binding (111 → 112) and
> **+1** distinct fn (96 → 97): `Module#doesNotUnderstand(_)`
> (`module_does_not_understand`, `primitive/module.rs`) — overrides `Object`'s
> default miss handler so a member send (`math.pi`, `math.distance(1, 2)`)
> reaches the module's own `globals`/`name_to_slot` table before falling
> through to the ordinary `MessageNotUnderstood` raise; this table has no
> other `.ph`-reachable accessor, so it fails the §1 derivability test exactly
> as ADR-0038 found for the error-handling catch protocol. Floor-carrying
> classes stay at **21** (`Module` already carried `new()`). The rest of
> `import`'s surface — path resolution, the canonical-path registry, cyclic-
> import termination, compile-once-run-once evaluation — is VM/compiler
> plumbing (`Bytecode::Import` + the pre-existing `Bytecode::DefineGlobal`),
> not a bound selector; it adds nothing to either count. R-INV-0.1 audits this
> set.
>
> **U16-Open amendment — superseded by Task Set 3.** The former Family call
> router was removed. Family calls now enter through the shared Function
> gateway, which rebuilds and dispatches the selected shape directly; no
> Family floor binding remains.
>
> ---
>
> _The five banners below were **written 2026-07-15** (DEFERRED #34), long after their
> amendments landed. The chain above stopped at U16-Open/113 while the code went to 125;
> §1.1 stayed consistent with the chain, which is exactly why it never looked wrong. All
> five are reconstructed from the test's constants and the install site — see §1.3._
>
> **U-SCHED amendment ([ADR-0030](../../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md) §Consequences).**
> The native ready-queue scheduler seam admits **+2** bindings (113 → 115) and **+2**
> distinct fns (98 → 100), both class-side on `System` and both `primitive/system.rs`:
> `System.schedule(_)` (`system_schedule`) enqueues a block on the ready queue, and
> `System.nextScheduled` (`system_next_scheduled`, a getter) pops the next one. They are
> the floor under the `.ph` scheduler: the queue itself is native because it outlives any
> one fiber and must be reachable from the collector's roots. Floor-carrying classes stay
> **22** — `System` already carried `print(_)`/`new()`.
>
> **U-ANNOT-CONTRACTS amendment ([ADR-0052](../../../adr/accepted/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md) Fix 1).**
> The `@invariant` re-entrancy guard admits **+2** bindings (115 → 117) and **+2** distinct
> fns (100 → 102), both instance-side on `Object` and both `primitive/object.rs`:
> `Object#_$invariantEnter()` (`object_invariant_enter`) and `Object#_$invariantExit()`
> (`object_invariant_exit`). The woven prologue/epilogue call them; they are never
> `.ph`-authored and are not part of any public protocol. They sit on `Object` because any
> receiver can carry an `@invariant`. Floor-carrying classes stay **22**. See §2.1.
>
> **M-ATTR-ROOT amendment** (no ADR — the unit's own amendment).
> The attribute-retention store admits **+3** bindings (117 → 120) and **+3** distinct fns
> (102 → 105), all instance-side on `Object`, all `primitive/attribute.rs`:
> `Object#_$attributes` (`attribute_attributes`), `Object#_$attach(_)`
> (`attribute_attach`), `Object#_$freezeAttributes()` (`attribute_freeze`). The compiler's
> `@Name(args)` desugar (`compiler::attributes`, `compiler::lib::class_decl`) calls them;
> never `.ph`-authored. On `Object` because every class and method row *is* an `Object`.
> Floor-carrying classes stay **22**. See §2.1.
>
> > **Dangling citation.** Both this amendment and the test's `NEW_ATTR_ROOT` comment cite
> > **`attribute-classes.md`** as their spec. **That file does not exist** anywhere under
> > `docs/`. These three bindings have no spec outside this census and the code. Not fixed
> > here — recorded so the next reader does not go looking. (DEFERRED CB-3 also lists
> > `attribute-classes.md` as a doc that omits `@sealed`/`@variant`; it cannot omit
> > anything, being absent.)
>
> **U-GC amendment ([ADR-0050](../../../adr/accepted/0050-non-moving-mark-sweep-collector.md), Step 3).**
> The collector's manual entry point admits **+1** binding (120 → 121) and **+1** distinct
> fn (105 → 106): `System.gc` (`system_gc`, `primitive/system.rs`), class-side, a getter —
> it forces a full mark-sweep cycle at a safepoint and answers the receiver. Native by
> necessity: nothing expressible in `.ph` can trace the heap. Floor-carrying classes stay
> **22**.
>
> **U-STRING amendment ([ADR-0049](../../../adr/accepted/0049-amend-floor-admit-string-byte-and-raw-write-primitives.md)).**
> Raw byte-level string access plus raw stdout write admit **+4** bindings (121 → 125) and
> **+4** distinct fns (106 → 110) — the amendment that takes the floor to its current
> figure. Instance-side on `String`: `String#_$byteCount`,
> `String#_$byteAt(_)`, and `String#_$slice(_,_)`. Class-side on `System`:
> `System._$write(_)`, stdout with no newline
> and no `toString` send. All four are floor because they touch the UTF-8 representation or
> the I/O boundary directly; the `.ph` `String` surface (`trim`, `split`, `startsWith`, …)
> is derived over them. The trailing `_` marks them native-raw and not-for-surface-use
> (U-NATIVE-MARKER renamed these from `rawByteAt`-style spellings, 2026-07-15).
> Floor-carrying classes stay **22** — `String` and `System` both already carried
> bindings. See §2.5 / §2.11.
>
> **`Fiber` admission — NOT an amendment ([ADR-0030](../../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md)).**
> **No primitive was added.** `Fiber`'s **11** bindings and **8** distinct fns have been
> installed since the fiber work landed under ADR-0030; what changed on **2026-07-15** is
> that they became *audited* (125 → **136**, fns 110 → **118**, floor-carrying classes
> 22 → **23**, audited kernel classes 28 → **29**). Listed here for chain continuity, but it
> is a **bookkeeping correction, not a floor widening** — the native boundary did not move;
> the census's account of it did. See §1.4 for how the hole survived, and §2.17 for the
> enumeration. This is the one row in the chain where the delta is the *census* catching up
> to the code rather than the code being extended.
>
> **Baseline:** post-U-STRING, + `Fiber` admitted — the figures above (**136 / 118 / 23 / 7** =
> bindings / distinct fns / floor-carrying classes / sacred selectors) are the current
> **audited** floor (was **121 / 106 / 22 / 7** post-U-GC, **120 / 105 / 22 / 7**
> post-M-ATTR-ROOT, **117 / 102 / 22 / 7** post-U-ANNOT-CONTRACTS,
> **115 / 100 / 22 / 7** post-U-SCHED, **113 / 98 / 22 / 7** post-U16-Open,
> **112 / 97 / 21 / 7** post-U15, **111 / 96 / 21 / 7** post-U-ERR,
> **109 / 94 / 21 / 7** post-U-COLLTYPES-Phase-3, **105 / 90 / 20 / 7** post-Phase-2,
> **102 / 87 / 19 / 7** post-Phase-1, **88 / 73 / 17 / 7** post-U-CORE-6; and
> **125 / 110 / 22 / 7** for the ~24h in 2026-07-15 between CB-2 reconciling the count and
> CB-5 admitting `Fiber`). **Do not quote this line** — it is a dated rendering of
> `invariants.rs` (§1.3), and it sat five amendments stale (at post-U15 / 112) until
> 2026-07-15. Installed now equals audited: all **136** (§1.1).
> This census is the ground-truth *enumeration*; the count's authority is the test. The
> landing history + drift policy live in [`README.md`](./README.md) §"Baseline & drift
> policy" (itself re-baselined 2026-07-15 — it had been frozen at U-ERR/111).
>
> One census-specific caution: of the post-U-CORE-0
> landings, **U-CORE-1 added +7 (73 → 80, ADR-0023), U-CORE-3 added +5
> (80 → 85, ADR-0028), U-CORE-4 added +1 (85 → 86, ADR-0036), U-CORE-6 added
> +2 (86 → 88, ADR-0037), U-COLLTYPES Phase 1 added +14 (88 → 102, ADR-0039),
> U-COLLTYPES Phase 2 added +3 (102 → 105, ADR-0039), U-COLLTYPES Phase 3
> added +4 (105 → 109, ADR-0039), U-ERR added +2 (109 → 111, ADR-0038),
> U15 added +1 (111 → 112, ADR-0045), U16-Open added +1 (112 → 113, ADR-0047),
> U-SCHED added +2 (113 → 115, ADR-0030), U-ANNOT-CONTRACTS added +2
> (115 → 117, ADR-0052), M-ATTR-ROOT added +3 (117 → 120, no ADR), U-GC added
> +1 (120 → 121, ADR-0050), and U-STRING added +4 (121 → 125, ADR-0049)**. **U-FIBER
> (ADR-0030) added 11 too** — but they went uncounted until 2026-07-15 (125 → **136**),
> which is why this list read as complete while it was not (§1.4). Every other unit either landed
> `.ph`/compiler surface or added zero bindings. U8's reflective surface and
> the `Message` class were already in the 73 (§2.1/§2.14); U-CORE-2/U-LEX/
> U-STD were `.ph`/compiler-only; U11 added `True`/`False` as kernel classes
> (19 → 21) with **+0** bindings — so "classes added" never implies "bindings
> added" (U11 is the counterexample; see §2.6). U-CORE-6 is the exception: its
> two new classes (`Error`/`MessageNotUnderstood`, 21 → 23) do come with
> bindings, but only on `Error` — see its amendment note above. U-COLLTYPES
> Phase 1 adds two more new classes (`Map`/`Set`, 23 → 25) that *both* carry
> bindings; Phase 2 adds one more (`Tuple`, 25 → 26); Phase 3 adds the last
> (`Range`, 26 → 27) — closing out the +21-binding, four-class amendment
> ADR-0039 enumerated in full. U-ERR adds **no** new classes (`Result`/`Ok`/
> `Err` are pure `.ph`, 27 → 27) — only two bindings on the pre-existing
> `Block` row.

### 1.2 Selector notation

Selectors are shown in **human-facing notation**: a getter is a bare name
(`size`), a setter is `name=(put)`, an arity-*n* method is `name(_, …)` with *n*
positional holes (`+(_)`, `new()`), and labeled arguments are named
(`ifTrue(_, ifFalse)`, `match(some, none)`).

> **Notation vs the interned string.** This differs from the canonical selector
> string that `make_signature`/`encode_selector`
> ([`method.rs`](../../../../phalcom-core/src/method.rs),
> [ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md))
> actually intern, which writes each positional hole as `_:` and each label as
> `label:`. So `+(_)` interns as `+(_:)`, `class=(put)` as `class=(_:)`, and
> `match(some, none)` as `match(some:none:)` — the same selector, different
> surface. The `_:` form is what you will find in `Universe::BOOL_SACRED_SELECTORS`
> and on the heap. (Heads-up: the `Sig` constants in
> [`primitive/mod.rs`](../../../../phalcom-core/src/primitive/mod.rs) are written in
> the human `_` form, so they do **not** string-match interned selectors — they
> are display aliases, not lookup keys.)
>
> **Canonical vs. current interned form.** The **comma / no-space form**
> (`+(_)`, `match(some,none)`, `move(_,to,duration)`) is the *canonical* spelling
> per [ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md) —
> use it in all normative prose. The colon `_:` form documented above is the
> *current interned/heap encoding only*; it is transitional, and migrating the
> interner to emit the comma form is owned by
> [U-CORE-4](../../../forge/units/U-CORE-4/as-built.md) (BD-CORE4-2). Colon-form
> selector spellings are **deprecated** as a canonical notation — they persist in
> as-built docs solely to describe what the binary interns today.

"Instance" primitives are installed on the class row via `primitive!`; "static"
primitives are installed on the class's **metaclass** via `primitive_static!`.

## 2. Census by class

Ordered as `install_primitives` installs them
([`universe/primitives.rs`](../../../../phalcom-core/src/universe/primitives.rs) L38–376).

### 2.1 `Object` — root protocol

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `name` | instance | `object_name` | class-name string ([ADR-0015](../../../adr/0015-object-default-tostring.md)) |
| `class` | instance | `object_class` | |
| `class=(put)` | instance | `object_set_class` | reflective class reassignment |
| `toString` | instance | `object_to_string` | default display, `"<ClassName>"` for an instance / own name for a class receiver (ADR-0015; U-CORE-4 re-home off `object_name`, fixes DEFERRED F4) |
| `==(_)` | instance | `object_eq` | ordinary send, **not** an opcode (control-flow.md §1) |
| `!=(_)` | instance | `object_neq` | ordinary send |
| `perform(_,***)` | instance | `object_perform_shape` | reflective send preserving complete argument shape (U8, messages-and-selectors.md §5) |
| `respondsTo(_)` | instance | `object_responds_to` | pure probe; never triggers dNU |
| `doesNotUnderstand(_)` | instance | `object_does_not_understand` | terminal miss handler; overridable so a proxy subclass can intercept |
| `hash` | instance | `object_hash` | identity digest of the heap handle (ADR-0023); immediates override below |
| `methodFor(_)` | instance | `object_method_for` | reifies the resolved `Method` for a selector; `None` on a miss; pure probe, never fires dNU (U-CORE-3, ADR-0028) |
| `_$invariantEnter()` | instance | `object_invariant_enter` | internal `@invariant` re-entrancy guard entry |
| `_$invariantExit()` | instance | `object_invariant_exit` | internal `@invariant` re-entrancy guard exit |
| `_$attributes` | instance | `attribute_attributes` | internal attribute-retention read |
| `_$attach(_)` | instance | `attribute_attach` | compiler-owned attribute-retention write |
| `_$freezeAttributes()` | instance | `attribute_freeze` | compiler-owned retention-store seal |

### 2.2 `Behavior` — class-side reflection

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `superclass` | instance | `class_superclass` | on `Behavior` so `Class` and `Metaclass` both inherit it ([ADR-0003](../../../adr/0003-introduce-behavior-kernel-class.md)) |
| `superclass=(put)` | instance | `class_set_superclass` | |
| `name` | instance | `behavior_name` | the receiver class's OWN name; **shadows** `Object#name` for class receivers (ADR-0023) |
| `methods` | instance | `behavior_methods` | own method-dictionary selector Symbols, as a fresh `List` (ADR-0023) |

### 2.3 `Class` — instantiation apex

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `+(_)` | instance | `class_add` | |
| `_$new()` | instance | `class_new_` | internal generic bare allocator reachable through the metaclass chain apex |

### 2.4 `Number` — flat `f64` ([ADR-0005](../../../adr/0005-number-as-flat-f64.md))

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `+(_)` `-(_)` `*(_)` `/(_)` `%(_)` | instance | `number_add` … `number_mod` | never inlined; ordinary sends (control-flow.md §1) |
| `<(_)` `<=(_)` `>(_)` `>=(_)` | instance | `number_lt` … `number_ge` | |
| `negated()` | instance | `number_negated` | |
| `hash` | instance | `number_hash` | digest of the mathematical value, class-agnostically (ADR-0023; forward-compat §4) |
| `toString` | instance | `number_to_string` | decimal-string render of the `f64` value, delegates to `Value::to_string` (U-CORE-4, ADR-00NN amendment) |
| `new()` , `new(_)` | static | `number_class_new` | coercion / zero |

### 2.5 `String`

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `+(_)` | instance | `string_add` | concatenation |
| `hash` | instance | `string_hash` | cached djb2 **content** hash — equal content ⇒ equal hash (ADR-0023) |
| `new()` , `new(_)` | static | `string_class_new` | |
| `_$byteCount` | instance | `string_raw_byte_count` | internal UTF-8 byte length |
| `_$byteAt(_)` | instance | `string_raw_byte_at` | internal raw byte read |
| `_$slice(_,_)` | instance | `string_raw_slice` | internal validated byte-range slice |

The rest of the `String` protocol (`split`, `replace`, `trim`/`trimStart`/`trimEnd`,
`*(count)`, `indexOf`, `codePointAt`, `bytes`/`codePoints`) is `.ph`-derived over these
three plus `Number` arithmetic — see `core.ph`'s `String` reopen.

### 2.6 `Bool` — abstract, `True`/`False` by dispatch ([ADR-0004](../../../adr/0004-boolean-as-abstract-bool-with-true-false.md))

| Selector | Side | Native fn | Sacred? |
|---|---|---|---|
| `new()` , `new(_)` | static | `bool_class_new` | |
| `and(_)` | instance | `bool_and` | ★ |
| `or(_)` | instance | `bool_or` | ★ |
| `not()` | instance | `bool_not` | ★ |
| `ifTrue(_)` | instance | `bool_if_true` | ★ |
| `ifFalse(_)` | instance | `bool_if_false` | ★ |
| `ifTrue(_, ifFalse)` | instance | `bool_if_true_if_false` | ★ — encoded explicitly, not via `make_signature`; interns as `ifTrue(_:ifFalse:)` |
| `hash` | instance | `bool_hash` | 1 for `true`, 0 for `false` — distinct, stable, **not** sacred (ADR-0023) |

★ = sacred selector (§5). No-truthiness ([ADR-0021](../../../adr/0021-no-truthiness-enforcement.md)):
these dispatch on real `True`/`False` receivers; there is no implicit coercion.

> **U11 landed** (`true_class`/`false_class`, [`universe/core_classes.rs`](../../../../phalcom-core/src/universe/core_classes.rs)
> L67/L68): `True`/`False` are now concrete singleton subclasses of `Bool`,
> not just a documented design intent. Neither carries any *own* floor
> primitive — both selectors and sacred inlining stay on `Bool`, reached by
> ordinary inheritance, so this unit added **0** rows to this table. It did
> add **2** to the kernel-class count (§1.1).

### 2.7 `Symbol`

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `toString` | instance | `symbol_tostring` | |
| `hash` | instance | `symbol_hash` | digest of the interned id — equal symbols agree (ADR-0023) |
| `new(_)` | static | `symbol_class_new` | interning constructor |

### 2.8 Absence — `Option` / `Some` / `None` ([ADR-0007](../../../adr/0007-option-as-abstract-with-some-none.md))

| Selector | Side | Class | Native fn | Notes |
|---|---|---|---|---|
| `call(_)` | static | `Some` | `some_call` | canonical present-value construction; immediate bounded `Some1`…`Some7`, no instance fields |
| `new(_)` | static | `Some` | `some_new` | compatibility alias for `Some.call(_)`; same immediate bounded representation |
| `match(some, none)`† | instance | `Option` | `option_match` | the eliminator, on abstract `Option` so `Some`/`None` inherit it (values-and-absence.md §3.2); encoded explicitly; interns as `match(some:none:)` |

† rendered from `encode_selector("match", [Some("some"), Some("none")], Method(2))`.
`None` carries **no** floor primitives of its own — it is a shared singleton
value, not a constructed instance. The combinator suite (`map`, `flatMap`,
`orElse`, `ifSome`, `unwrapOr`, …) is deliberately **not** on the floor; it is
`core.ph`/U-STD work layered over `match`.

### 2.9 `Method`

`Method` remains a reified dispatch object directly under `Object`; it does not
inherit the Function call protocol and does not answer raw `call` while
unbound. It carries its own `arity`/`name` reflection accessors, static
`new(_)`, and the U-CORE-3 reflection surface
([ADR-0028](../../../adr/0028-amend-floor-admit-method-reflection.md)):
applying a reified method to an explicit receiver (`invokeOn`), closing one
over a receiver (`bind`), and reading its selector/holder.

| Selector | Side | Native fn |
|---|---|---|
| `new(_)` | static | `method_class_new` |
| `invokeOn(_,***)` | instance | `method_invoke_on_shape` |
| `bind(_)` | instance | `method_bind` |
| `selector` | instance | `method_selector` |
| `holder` | instance | `method_holder` |

`Object#methodFor(_)` (`object_method_for`, §2.1) reifies the `MethodObject` a
selector resolves to on a receiver, as a bare `Method` value; the `None`
singleton on a miss. `bind(_)` returns a new heap representation,
`Object::BoundMethod` (method handle + receiver, no closure or frame token —
it must work for primitive methods too), whose surface class is `BoundMethod`;
as a `Function` descendant it answers the shared call protocol in §2.10.

### 2.10 `Function` / `Block` — callables ([ADR-0006](../../../adr/0006-function-as-abstract-callable-root.md), [ADR-0013](../../../adr/0013-closure-upvalues-and-frame-token-return.md))

`Closure` (the runtime value for a Block literal), `BoundMethod`, `Family`, and
`BoundMethodFamily` are subclasses of `Function`; the shared callable protocol
is installed on `Function`, so every concrete Function reaches one gateway.
`Closure` retains only its block-specific control-flow primitives.

| Selector | Side | Class(es) | Native fn | Sacred? |
|---|---|---|---|---|
| `arity` | instance | Function, Block | `block_arity` | |
| `name` | instance | Function, Block | `block_name` | |
| `callWith(_)` | instance | Function | `block_call_with_shape` | complete `Unit`/`Tuple` argument pack |
| `call(***)` | instance | Function | `block_call_shape` | one complete positional/labeled argument shape |
| `whileTrue(_)` | instance | Block | `block_while_true` | ★ sacred loop fallback |
| `on(_,_)` | instance | Block | `block_on` | U-ERR, ADR-0038 — typed catch (`try`/`on`/`catch` desugar target) |
| `ensure(_)` | instance | Block | `block_ensure` | U-ERR, ADR-0038 — always-runs cleanup (`try`/`ensure` desugar target) |

**U-CORE-3 behavior completions.** `block_arity`/`block_name` learn an
`Object::Method` receiver (reading
`signature.positional_arity`/`signature.selector` directly) and an
`Object::BoundMethod` receiver (delegating to the wrapped method). The common
Function gateway activates Closure, BoundMethod, Family, and BoundMethodFamily
values directly, so `bound.call(***args) ≡ method.invokeOn(recv, ***args)` holds
by construction (R-INV-3.3). MethodFamily remains the immutable reflection
snapshot that BoundMethodFamily closes over.

### 2.11 `System` — I/O floor

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `print(_)` | static | `system_class_print` | the sole I/O primitive |
| `new()` | static | `system_class_new` | |
| `_$write(_)` | class-side | `system_raw_write` | internal raw stdout write; public wrappers are `.ph`-derived |

> Also present but not yet catalogued in this table: `schedule(_)`/`system_schedule`,
> `nextScheduled`/`system_next_scheduled` (U-SCHED), `gc()`/`system_gc` (U-GC step 3).
> Pre-existing staleness, out of scope for the U-STRING doc-sync pass.

### 2.12 `Module` — namespace object (U15, [ADR-0045](../../../adr/0045-module-import-relative-path-whole-module-binding.md))

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `new()` | static | `module_class_new` | always rejects — a `Module` is only ever produced by `VM::import_module` |
| `doesNotUnderstand(_)` | instance | `module_does_not_understand` | overrides `Object`'s default miss handler; member access as an ordinary send (U15) |

### 2.13 `List` — native array-backed kernel collection ([ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md))

A dedicated `Object::List` heap variant (`crate::list::ListObject`), **not** an
`InstanceObject`. The floor is five raw primitives + native `toString`; the
public protocol (`size`/`at`/`add`/`each`) is `core.ph` over them (§3).

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `new()` | static | `list_class_new` | |
| `_$length` | instance | `list_raw_length` | internal; wrapped by `size` |
| `_$at(_)` | instance | `list_raw_at` | internal; wrapped by `at(_)` |
| `_$set(_, _)` | instance | `list_raw_set` | internal; wrapped by `at(_,put)` and `[_]=(put)` |
| `_$push(_)` | instance | `list_raw_push` | internal; wrapped by `add(_)` |
| `_$replaceSlice(_, _, _)` | instance | `list_replace_slice` | internal variable-length replacement |
| `toString` | instance | `list_to_string` | native this unit (see U-LIST return contract) |

### 2.13a `Map`/`Set` — native hash collections (U-COLLTYPES Phase 1, [ADR-0032](../../../adr/0032-collections-representation-and-literals.md) §1, [ADR-0039](../../../adr/0039-amend-floor-admit-collection-container-primitives.md))

Dedicated `Object::Map`/`Object::Set` heap variants over the shared
`crate::map::MapObject` ordered-hash backing struct (DEC-CT-B: `Set` is a
keys-only sibling, distinct heap variant, distinct bindings). Both are
**mutable** ⇒ inherit identity `Object#hash` (Q5) — neither installs its own
`hash`, so neither is a valid `Map`/`Set` key. `_$get`/`_$put`/`_$has`/
`_$remove`/`_$add` re-enter the VM to send **Phalcom** `hash`/`==` on keys
(`primitive/map.rs`'s `locate`); `_$put`/`_$add` reject a mutable-collection
key (`List`/`Map`/`Set`, DEC-CT-C) with a raised catchable `Error`. The public
protocol (`at(_)`/`at(_,put:)`/`size`/`includes(_)`/`remove(_)`/`keys`/
`values`/`each(_)` for `Map`; `add(_)`/`includes(_)`/`size`/`remove(_)`/
`each(_)`/`at(_)` for `Set`) is `core.ph` over these (§3).

| Selector | Side | Class | Native fn | Notes |
|---|---|---|---|---|
| `new()` | static | `Map` | `map_class_new` | |
| `_$size` | instance | `Map` | `map_raw_size` | wrapped by `size` |
| `_$get(_)` | instance | `Map` | `map_raw_get` | wrapped by `at(_)` |
| `_$put(_, _)` | instance | `Map` | `map_raw_put` | wrapped by `at(_,put)` |
| `_$has(_)` | instance | `Map` | `map_raw_has` | wrapped by `includes(_)` |
| `_$remove(_)` | instance | `Map` | `map_raw_remove` | wrapped by `remove(_)` |
| `_$keyAt(_)` | instance | `Map` | `map_raw_key_at` | internal iteration support |
| `_$valueAt(_)` | instance | `Map` | `map_raw_value_at` | internal iteration support |
| `new()` | static | `Set` | `set_class_new` | |
| `_$size` | instance | `Set` | `set_raw_size` | wrapped by `size` |
| `_$add(_)` | instance | `Set` | `set_raw_add` | wrapped by `add(_)` |
| `_$has(_)` | instance | `Set` | `set_raw_has` | wrapped by `includes(_)` |
| `_$remove(_)` | instance | `Set` | `set_raw_remove` | wrapped by `remove(_)` |
| `_$at(_)` | instance | `Set` | `set_raw_at` | internal indexed read |

### 2.13b `Tuple` — native fixed-arity immutable product (U-COLLTYPES Phase 2, [ADR-0032](../../../adr/0032-collections-representation-and-literals.md) §1, [ADR-0039](../../../adr/0039-amend-floor-admit-collection-container-primitives.md))

A dedicated `Object::Tuple` heap variant (`crate::tuple::TupleObject`, a fixed
`Box<[Value]>`), **not** an `InstanceObject`. The floor is three raw
primitives — **no mutation primitive**, since immutability is structural (no
`at(_, put:)`/`add(_)` accessor exists at all). Immutable ⇒ value-hashable and
a valid `Map`/`Set` key (Q5). The public protocol (`size`/`at(_)`/`each(_)`/
`==`/`!=`/`hash`) is `core.ph` over these (§3); `hash` is a `.ph` fold over
`_$at`+element `.hash` (DEC-CT-D), not a floor primitive.

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `_$fromList(_)` | class-side | `tuple_from_list_internal` | internal conversion bridge |
| `_$size` | instance | `tuple_raw_size` | wrapped by `size` |
| `_$at(_)` | instance | `tuple_raw_at` | wrapped by `at(_)`/`each(_)` |
| `_$positionalSize` | instance | `tuple_raw_positional_size` | positional lane length |
| `_$labelAt(_)` | instance | `tuple_raw_label_at` | label-lane observation |
| `_$positionals` | instance | `tuple_raw_positionals` | positional projection |
| `_$labeled` | instance | `tuple_raw_labeled` | labeled projection |
| `_$slice(_, _)` | instance | `tuple_raw_slice` | internal label-preserving slice |

### 2.13c `Range` — native lazy numeric interval (U-COLLTYPES Phase 3, [ADR-0032](../../../adr/0032-collections-representation-and-literals.md) §1, [ADR-0039](../../../adr/0039-amend-floor-admit-collection-container-primitives.md))

A dedicated `Object::Range` heap variant (`crate::range::RangeObject`) — three
fields (`start`/`end`/`inclusive`), **no element storage** (RG-2 laziness).
Range construction is direct `BuildRange` bytecode. Its three native observers
surface optional bounds; progression, equality, hashing, and traversal are not
yet part of the Range protocol.

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `_$lower` | instance | `range_raw_lower` | `Option` distinguishes omitted lower bound |
| `_$upper` | instance | `range_raw_upper` | `Option` distinguishes omitted upper bound |
| `_$upperInclusive` | instance | `range_raw_upper_inclusive` | true only for `..=` with upper bound |

### 2.14 `Message` — reified miss-send ([messages-and-selectors.md](../messages-and-selectors.md) §5, U8)

**Not** an `object-model.md` §4 catalog class — a fixed-slot `InstanceObject`
(four slots) built directly by `VM::new_message` and handed to
`doesNotUnderstand(_)`. Its field count is stamped in `VM::new` (mirroring
`Some`); it has no `.ph` surface but *is* a surface global
(`add_class!(message_class)`).

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `selector` | instance | `message_selector` | the interned selector symbol |
| `name` | instance | `message_name` | **shadows** `Object#name` — returns the *sent method* name, not the class name |
| `labels` | instance | `message_labels` | per-argument labels |
| `args` | instance | `message_args` | argument values |

### 2.15 `Error` — raisable root ([object-model.md](../../object-model.md) §4 "Errors", [ADR-0008](../../../adr/0008-layered-exceptions-and-result.md), U-CORE-6)

Root of the surface error hierarchy; `MessageNotUnderstood < Error` is the
sole subclass this unit reifies (the retired native
`RuntimeError::MessageNotUnderstood` is now this class). Like `Message`, both
are fixed-slot `InstanceObject`s stamped in `VM::new`'s Phase E — `Error` has
one field (`_message`, slot 0); `MessageNotUnderstood` inherits it and adds
`_reifiedMessage` (slot 1). Both are surface globals
(`add_class!(error_class)` / `add_class!(message_not_understood_class)`), no
`.ph` reopen. `MessageNotUnderstood` carries no primitives of its own — it
inherits `message`/`raise` from `Error`.

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `message` | instance | `error_message` | reads `_message` (slot 0); mirrors `Message`'s native accessors |
| `raise()` | instance | `error_raise` | initiates the unified unwind's `Raise` payload (`RuntimeError::Raise`); `throw expr === expr.raise()` (ADR-0031 §1); installed on `Error` only (R-INV-6.3) |

### 2.16 `Family` — `::` method-reference Function value ([selectors.md §3](../selectors.md#3-method-references-))

`Family` remains a native heap representation under `Object`. `obj::name` is
an exact getter reference, `obj::name()` is an exact nullary-method reference,
and ellipsis forms are structural-pattern references; unbound `Type::name`
forms do not exist. Calls enter through the shared Function gateway: exact
Families retain selector identity, while pattern Families match their stored
predicate against the current method table. Family installs no
`doesNotUnderstand(_)` router primitive.

### 2.17 `Fiber` — cooperative coroutine (U-FIBER / U-FIBER-REFLECT, [ADR-0030](../../../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md))

> **Audited only since 2026-07-15** (DEFERRED CB-5). These 11 bindings shipped with the
> fiber work but were enumerated nowhere and audited by nothing; the class was missing from
> the test's `core_class_rows`. **No primitive was added to admit them** — see §1.4.

A native `Object::Fiber` heap variant (no `Value::Fiber` arm — reached through `Value::Obj`,
as `Object::List` is), sitting directly under `Object`. The whole class is floor by
necessity: a fiber *is* a suspended native call stack, so nothing in `.ph` can build,
resume, or inspect one. Note the split — `call`/`try`/`isDone`/`error` are **instance**-side
(they act on a particular fiber), while `yield`/`current`/`abort` are **class**-side (they
act on whatever fiber is running *now*, which the receiver cannot name).

`call` vs `try` is the whole error contract: both resume the receiver, and they differ only
in what an uncaught failure does — `call` re-raises it into the resumer
(`FiberResumeMode::Call`); `try` captures it at the fiber floor and delivers it as an
`Error` value (`FiberResumeMode::Try`). Each is bound at arity 0 and 1 (resume with no
value, or with one), which is why 11 bindings need only 8 native fns.

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `new(_)` | static | `fiber_new` | builds a suspended fiber over a `Block`/`Closure`; `RuntimeError::Type` if the argument is neither |
| `call()` | instance | `fiber_call` | resume, no value passed; uncaught failure **re-raises into the resumer** |
| `call(_)` | instance | `fiber_call` | resume, passing one value |
| `try()` | instance | `fiber_try` | resume; uncaught failure is **captured at the fiber floor** and delivered as an `Error` |
| `try(_)` | instance | `fiber_try` | as `try()`, passing one value |
| `yield()` | static | `fiber_yield` | suspend the running fiber back to its resumer |
| `yield(_)` | static | `fiber_yield` | suspend, yielding one value. Raises `CannotYieldAcrossNativeFrame` if `native_reentry_depth` has grown past the fiber's recorded `floor_depth` since it was last resumed (ADR-0030 §4); `RuntimeError::NotAllowed` from the root fiber, which has no resumer |
| `current` | static | `fiber_current` | the running fiber (`VM::current`) |
| `abort(_)` | static | `fiber_abort` | fails the running fiber with the given value (`RuntimeError::Raise`); `RuntimeError::NotAllowed` from the root fiber — it has nowhere to propagate a floor capture to (spec §2 rule 7, §6) |
| `isDone` | instance | `fiber_is_done` | pure read over `FiberObject::status`; no scheduler dependency (U-FIBER-REFLECT) |
| `error` | instance | `fiber_error` | pure read over `FiberObject::result`; `RuntimeError::Type` if the receiver is not a `Fiber` |

The scheduler seam (`System.schedule(_)` / `System.nextScheduled`) is **not** here — it is
class-side on `System` (§2.11), and is what the `.ph` scheduler is written over.

## 3. The floor ↔ `core.ph` boundary

Two classes now carry `.ph` surface protocol self-hosted over the floor
([`core.ph`](../../../../phalcom-core/core/core.ph)):

**`List`** (ADR-0020) —

```
size       => self._$length
at(_ i)    { return self._$at(i) }
add(_ v)   { self._$push(v); return self }
each(_ f)  { let i = 0; while (i < self.size) { f.call(self.at(i)); i = i + 1 } }
```

`each` closes over three floor capabilities — `Block#call(_)`, `Number#<(_)`,
and `while` lowering (`Block#whileTrue(_)` / sacred inliner) — plus the
same-class `size`/`at` defined above it.

**`Option`** (U-CORE-2, `0da64d6`; `toString` added by U-CORE-4) — combinators
and display, each derived purely over the `match(some, none)` eliminator (the
sole floor capability they touch):

```
ifNone(f)  => self.match(some: { v => self }, none: { f.call(); self })
orElse(f)  => self.match(some: { v => self }, none: { f.call() })
isSome     => self.match(some: { v => true }, none: { false })
isNone     => self.match(some: { v => false }, none: { true })
toString   => self.match(some: { v => "Some(" + v.toString + ")" }, none: { "None" })
```

**`String`** (U-CORE-4) — a string's display *is* itself, no representation
read:

```
toString => self
```

**`Bool`** (U-CORE-4) — derived over the sacred `ifTrue(_, ifFalse)` selector
(non-sacred itself, no inliner deopt — §5):

```
toString { return self.ifTrue({ "true" }, ifFalse: { "false" }) }
```

Every other `core.ph` class today is an **empty reopen** (`Object`, `Class`,
`Metaclass`, `Symbol`, `Some`) that only makes the name surface-visible;
`System` carries an empty `static print()` shell backed by the native
primitive. (`None` deliberately has **no** reopen — see the `core.ph` comment
on the `DefineGlobal`-clobber hazard.)

> This boundary is the template for U-CORE-2…5: **push protocol into `core.ph`,
> keep the floor minimal.** A new method belongs on the floor only if it fails
> the derivability test in §1.

## 4. Dispatch subtlety — two `new`s

`new()` is bound in two places:

- `object_class_new` on **`Object class`** (metaclass) via `primitive_static!`.
- `class_new` on **`Class`** via `primitive!`.

For a user class `Foo < Object`, a `Foo.new` send searches the metaclass chain
`Foo class → Object class → Class → Behavior → Object`. `Object class` is nearer
than `Class`, so `object_class_new` is the **effective default allocator**;
`class_new` is a deeper fallback. Specialized static `new`s (`Number`, `String`,
`Bool`, `Symbol`, `Method`, `List`, `System`, `Module`) override on their own
metaclass. Any core-library change that touches instantiation must preserve this
ordering — it is load-bearing for `construct` (U7 / [ADR-0011](../../../adr/0011-static-instance-slot-layout.md)).

## 5. Sacred selectors (R-SACRED) — the compiler-coupled subset

Seven floor selectors are **sacred**: the sacred-selector inliner
([ADR-0018](../../../adr/0018-sacred-selector-inliner-and-override-guard.md))
special-cases literal-block call sites for them and emits a `GuardBool`
deopt that falls back to *exactly these* real sends on override or receiver
mismatch. The core library treats this set as a **fixed interface** — a kernel
`Bool`/`Block` reopen that changes their shape breaks the compiler.

| Receiver | Sacred selectors | Override-epoch flag |
|---|---|---|
| `Bool` | `and(_)`, `or(_)`, `not()`, `ifTrue(_)`, `ifFalse(_)`, `ifTrue(_, ifFalse)` | `Universe::bool_sacred_pristine` |
| `Block` | `whileTrue(_)` | `Universe::block_sacred_pristine` |

`Universe::note_method_installed` flips the relevant flag the first time any of
these is (re)installed on the kernel row, deopting every inlined site. Source of
truth: `Universe::BOOL_SACRED_SELECTORS` / `BLOCK_SACRED_SELECTORS` (which store
the interned `_:` form: `and(_:)`, `ifTrue(_:ifFalse:)`, `whileTrue(_:)`).

> **Requirement:** any U-CORE unit that reopens `Bool` or `Block` must (a) keep
> these exact selector shapes, and (b) budget for the deopt if it *replaces* a
> sacred method body.

## 6. Explicitly *not* on the floor (deferred / derivable)

| Item | State | Owner |
|---|---|---|
| `List#at(_,put)` (wrap `_$set`) | landed | U-STD |
| `List` `map`/`reduce`/`filter`/literal syntax | derivable over floor | U-STD |
| `Option` combinators (`map`/`flatMap`/`orElse`/`ifSome`/`unwrapOr`) | derivable over `match` | U-STD / U-CORE-2 |
| `Block#repeat(_)` | receiver/semantics unpinned | deferred (U5-plan BD-U5-2) |
| `callWith(_)` packed-arg semantics | bound; accepts complete `Unit`/`Tuple` packs | `block_call_with_shape` |
| surface `Nil` / `nil` | **forbidden** — Invariant 4 ([ADR-0010](../../../adr/0010-tagged-value-enum.md), [ADR-0021](../../../adr/0021-no-truthiness-enforcement.md)) | never |

The `Nil` class row exists in the tower (to back `Value::Nil.class`) but is
bound to **no global** and carries **no primitives** — it is unreachable from
user code by construction.

## 7. Amendment protocol & audit

Because the floor is frozen (ADR-0019), this census is a **contract**:

1. **To add/remove a primitive** — open an ADR amending 0019, justify why the
   capability fails the §1 derivability test, then update this file in the same
   change.
2. **Audit hook (R-INV-0.1, landed U-CORE-1):**
   `floor_census_matches_installed_bindings` in
   [`tests/invariants.rs`](../../../../phalcom-core/tests/invariants.rs)
   reconstructs the installed native-`(class, selector)` set from a live
   `VM::new()` (filtering out `core.ph`-defined closures) and asserts it equals
   the census here. **The test is the source of record for the count (§1.3); do
   not restate the number here.** As of 2026-07-15 it is **136**, the sum of its
   own per-amendment constants — but read the constants, not this sentence.
   This turns silent floor drift into a red test; §1.1 is no longer a manual
   checksum.

   **Coverage caveat.** The hook audits only the classes in the test's
   `core_class_rows`. That list is the audit's real boundary and **nothing audits
   it** — a kernel class missing from it is unfrozen in fact, whatever ADR-0019
   says. This is not hypothetical: `Fiber` sat outside it, unlisted and unaudited,
   for the whole life of the fiber work (§1.4, closed 2026-07-15). **A new kernel
   class carrying primitives must gain its `core_class_rows` row in the same
   change** — that row, not this protocol's prose, is what binds it.

## 8. Traceability

**Cite the symbol, not the line.** Every row below is keyed by a named symbol; the line
numbers are a 2026-07-15 convenience and rot on contact. `universe.rs` is no longer a
file — it is the `universe/` directory (`mod.rs`, `core_classes.rs`, `primitives.rs`,
`invariants.rs`), and every ref in this table pointed at the dead path until this pass.

| Section | Symbol | Line (2026-07-15) |
|---|---|---|
| §2 all | `universe/primitives.rs::install_primitives` | L38–376 |
| §2.1 Object reflective surface (U8) | `perform`/`respondsTo`/`doesNotUnderstand`/`methodFor` | L56–63 |
| §2.6 encoded `ifTrue(_:ifFalse:)` | hand-rolled `encode_selector` + `new_primitive` (not the `primitive!` macro) | L158–165 |
| §2.8 encoded `match(some:none:)` | hand-rolled, installed on `Option` so `Some`/`None` inherit | L191–198 |
| §2.8 `Some`/`None` representation | `universe/core_classes.rs` / `value/option.rs` | zero fields; native immediate variants |
| §2.10 Function gateway | `call(***)` and `callWith(_)` are shape-aware gateways; no finite arity ceiling | L215+ |
| §2.14 `Message` | `universe/primitives.rs` (`message_cls` block) | L81–85 |
| §3 `List` protocol | `core.ph::class List` | L779+ |
| §5 sacred set | `universe/mod.rs::{BOOL,BLOCK}_SACRED_SELECTORS` (6 + 1 = 7) | L100–106 |
| §2.17 `Fiber` | `universe/primitives.rs` (`fiber_cls` block) · `primitive/fiber.rs` | L362–374 |
| The audit's own boundary (§1.4) | `tests/invariants.rs::core_class_rows` — 29 rows; a kernel class absent here is unaudited, and nothing audits *this* list | L48+ |

Two bindings are **not** installed via the `primitive!`/`primitive_static!` macros —
`Bool#ifTrue(_:ifFalse:)` and `Option#match(some:none:)` are hand-rolled because both carry
labeled arguments that the label-free `make_signature` cannot encode. A grep for the macro
therefore undercounts the floor by two; the test does not.
