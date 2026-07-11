# U-CORE-2 — Absence + Boolean (implementation spec)

> **Status:** Normative work order. **Verification / hardening + residue**, *not*
> new protocol. The behavioural core of U-CORE-2 already shipped in commit
> [`0da64d6`](#a-the-0da64d6-baseline); this spec closes the remaining
> **invariant-corpus** gap that hardens it and pins the boundaries U-CORE-2 must
> not cross.
>
> **Baseline:** HEAD `4e2ec73` (U10 landed); last U-CORE-2 code commit
> `0da64d6`. **Floor delta: none** (no new native primitive — see §2).
>
> **Governing anchors:** [`values-and-absence.md`](../values-and-absence.md) §3
> (absence is `Option`), §3.3 (combinator groups); [ADR-0007](../../adr/0007-option-as-abstract-with-some-none.md);
> [ADR-0021](../../adr/0021-no-truthiness-enforcement.md);
> [ADR-0018](../../adr/0018-sacred-selector-inliner-and-override-guard.md) + its
> **U-CORE-2 amendment**; [`catalog-delta.md`](./catalog-delta.md) §2.2 / §4.2;
> [`decisions.md`](./decisions.md) Q2 / §4.4; [`floor-census.md`](./floor-census.md)
> §2.6 / §2.8; [`invariant-requirements.md`](./invariant-requirements.md) R-INV-2.1–2.4,
> R-INV-0.3; [`forward-compat.md`](./forward-compat.md) §2.

---

## §0. Prerequisites and scope gate

### 0.1 What is already DONE (do not re-implement)

The bulk of U-CORE-2 is **landed and green**. An implementer must treat the
following as *fixed inputs* and only **verify** them, never rewrite them.

#### A. The `0da64d6` baseline

`0da64d6` ("U-CORE-2: Some-lift one-armed ifTrue/ifFalse, close the half-Option
divergence") delivered, and this spec confirms as-built:

1. **`Bool#ifTrue(_)` / `ifFalse(_)` now return a well-formed `Option`.** The
   *taken* arm is `Some`-lifted; the *untaken* arm is the `None` singleton — so
   the result is `Some(A) ∪ None`, not the pre-U-CORE-2 half-`Option`
   (`A ∪ None`). This closes [`catalog-delta.md`](./catalog-delta.md) §4.2.
   - `primitive/boolean.rs` `bool_if_true` (L115–122): `true` → `wrap_some(vm, block_call(…))`; `false` → `vm.none_value()`.
   - `primitive/boolean.rs` `bool_if_false` (L133–140): mirror.
   - `primitive/nil.rs` `wrap_some` (L47–58): the shared allocator factored out of `some_new`; **asserts `!matches!(value, Value::Nil)`** (Invariant 4).

2. **The sacred inliner `Some`-lifts in lockstep** via a new `Bytecode::WrapSome`
   opcode (ADR-0018 amendment), so the guarded fast path is observationally
   identical to the primitive deopt path.
   - `bytecode.rs` `WrapSome` (L194); VM exec `vm.rs` (L982–986): pop, `wrap_some`, push.
   - `compiler/inliner.rs` `compile_if_true` emits `WrapSome` at L296 (only when `want_value`); `compile_if_false` at L324.
   - **Pop-context elision:** `want_value` is threaded `compile_statement_with_pop_control` → `compile_expr_want` → `compile_sacred_call_want` → `compile_if_true`/`compile_if_false` (`compiler/lib.rs` L502/L509/L939; `inliner.rs` L148). When the result is discarded (`emit_pop`), `WrapSome` is skipped.

3. **`core.ph`'s `Option` reopen gained four combinators**, each derived purely
   over the `match(some, none)` eliminator (`core.ph` L42–60):
   `ifNone(_)` (L46–48), `orElse(_)` (L53–55), `isSome` (L57), `isNone` (L59).

4. **Golden fixtures already promoted** (in `tests/lang/`, active PASS lane):
   `absence/absence_iftrue_empty_body_is_some_none`,
   `absence/absence_iftrue_issome_isnone`, `absence/absence_iftrue_orelse`,
   `control-flow/control_flow_iftrue_ifnone_desugar` (plus the pre-existing U6
   `absence/absence_iftrue_false_branch_is_none`).

#### B. The load-bearing safety fact (verify, keep)

`Bytecode::Nil` pushes the **`None` singleton**, not the raw `Value::Nil`
sentinel (`vm.rs` L804 — `self.stack.push(self.none_value())`). Therefore an
**empty-bodied** `true.ifTrue { }` inlines to `Nil; WrapSome`, and `WrapSome`
wraps the *`None` singleton* → `Some(None)` — a legal `Some` (Invariant 4 forbids
only the raw sentinel, and `Some(None)` is explicitly allowed, see
`tests/invariants.rs::some_can_wrap_the_none_singleton`). The `wrap_some`
Invariant-4 assert is thus never tripped on this path. **Any change that made
`Bytecode::Nil` push the raw sentinel would panic `WrapSome`** — this is the
single subtlest correctness coupling in the unit and R-INV-2.1/2.2 must fence it.

### 0.2 What REMAINS (this unit's deliverable)

**One thing:** the U-CORE-2 **invariant corpus** — the behavioural assertions
`R-INV-2.1…2.4` (and coordination on `R-INV-0.3`) that harden the landed code
against regression. See §4. That is the whole work order. No runtime code
changes are expected; if the implementer finds one is *needed* to make an
invariant pass, that is a latent bug the invariant just caught — fix it minimally
and note it.

### 0.3 Explicitly OUT of scope (do not add here)

| Item | Owner | Anchor |
|---|---|---|
| `None#toString` / `Some#toString` surface (the `toString` **message**) | **U-CORE-4** | [`decisions.md`](./decisions.md) §4.4 |
| `Option#ifSome(_)`, `map(_)`, `flatMap(_)`, `filter(_)`, `unwrapOr(_)`, `unwrapOrElse(_)`, `unwrap()`, `zip(_)`, `contains(_)` | **U-STD** | [`catalog-delta.md`](./catalog-delta.md) §2.2 |
| `??` / `?.` surface tokens + desugar | **U-LEX** | [`values-and-absence.md`](../values-and-absence.md) §3.4 |
| `Some(x)` bare-call sugar (only `Some.new(x)` exists today) | **U-LEX** | [ADR-0021](../../adr/0021-no-truthiness-enforcement.md) |
| Abstract `Bool` + `True`/`False` singleton *representation* | **U11** (separate forge unit) | [ADR-0004](../../adr/0004-boolean-as-abstract-bool-with-true-false.md); see §5.3 |
| `Result`/`Ok`/`Err` (the `Option` sibling) | **U-CORE-6** (reserve) / later | [`forward-compat.md`](./forward-compat.md) §2; §5.2 below |
| Removing the `bool_class_new` debug `println!`s (`boolean.rs` L33/L35) | **DEFERRED** (pre-existing noise) | `docs/forge/DEFERRED.md` |

> **Note on the README parenthetical.** [`README.md`](./README.md) L47–48 lists
> the U-CORE-2 residue as "(absence invariants, `None`/`Some` surface
> `toString`)". That parenthetical is loose: the authoritative owner ruling is
> [`decisions.md`](./decisions.md) §4.4, which assigns per-type `toString`
> **to U-CORE-4**. This spec follows the ruling — `toString` is **not** U-CORE-2.

---

## §1. What exists vs what is missing (grounded)

### 1.1 The absence + Boolean floor (present, frozen)

Per [`floor-census.md`](./floor-census.md) §2.6 / §2.8, the relevant floor is
complete and is **not** touched by this unit:

| Selector (human `_` form) | Class | Native fn | Sacred? |
|---|---|---|---|
| `and(_)` · `or(_)` · `not()` | `Bool` | `bool_and`/`bool_or`/`bool_not` | ★ |
| `ifTrue(_)` · `ifFalse(_)` | `Bool` | `bool_if_true`/`bool_if_false` | ★ |
| `ifTrue(_, ifFalse)` | `Bool` | `bool_if_true_if_false` (interns `ifTrue(_:ifFalse:)`) | ★ |
| `new()` / `new(_)` | `Bool.class` | `bool_class_new` | |
| `new(_)` | `Some.class` | `some_new` | |
| `match(some, none)` | `Option` (inherited by `Some`/`None`) | `option_match` (interns `match(some:none:)`) | |

`None` carries **no** floor primitives — it is a shared singleton *value*, not a
constructed instance. The four `Option` combinators live in `core.ph`, not the
floor (§2). *(Selector notation is the human `_` form per floor-census §1.2; the
interned heap form is noted parenthetically where it differs.)*

### 1.2 The invariant surface (the gap)

`tests/invariants.rs` today covers the absence *non-surfacing* set
(`expression_result_absence_surfaces_to_none` L65–115 — which already exercises
`bool_if_true`/`bool_if_false` on the **untaken** arm returning `None`;
`some_construction_never_wraps_the_sentinel` L117–125;
`some_can_wrap_the_none_singleton` L127–137). What is **absent** is any assertion
that:

- the **taken** arm's `Some`-lift is present and **fast-path ≡ deopt-path** (R-INV-2.1);
- pop-context elision is **observationally invisible** (R-INV-2.2);
- the paired conditional and `and`/`or` are **still raw**, un-lifted (R-INV-2.3);
- the combinators **route through `match`** and respect an override (R-INV-2.4).

These four are this unit's deliverable.

---

## §2. Native-vs-`.ph` split — no floor change

**This unit adds ZERO native primitives.** ADR-0019's frozen floor is untouched;
**no ADR-0019 amendment is required.** For the record, the pieces that landed in
`0da64d6` are *not* floor additions:

- `wrap_some` (`nil.rs` L47–58) is a **private `pub(crate)` helper** factored out
  of the existing `some_new` primitive — it registers no new `(class, selector)`
  binding, so the floor census count is unchanged (still 73; R-INV-0.1 must stay
  green, unmoved).
- `Bytecode::WrapSome` is a **compiler opcode**, not a message-send primitive —
  it is invisible to dispatch and to the floor census.
- `ifNone` / `orElse` / `isSome` / `isNone` are **`.ph`** methods over `match`
  (the floor ↔ `core.ph` boundary template of floor-census §3) — the correct
  side of the boundary.

The verification job for §2 is therefore purely: **confirm the census still
counts 73 floor bindings** (the R-INV-0.1 audit, owned by U-CORE-1) and that
nothing in this unit's test additions smuggles a `primitive!` call. It does not.

---

## §3. Concrete change set

No production `.rs` / `.ph` bodies are added or edited. The entire change set is
**test files** (§4) plus this spec. If the implementer's invariant run stays
green, `phalcom-core/src/**` and `core.ph` are **untouched**.

The one contingency: should R-INV-2.1 or R-INV-2.2 go red, the implicated code is
narrowly one of `bool_if_true`/`bool_if_false` (boolean.rs L115–140),
`wrap_some` (nil.rs L47–58), the `WrapSome` emit sites (inliner.rs L296/L324), the
`want_value` thread (lib.rs L509 / inliner.rs L148), or the `WrapSome` VM exec
(vm.rs L982–986). Fix minimally, re-run, and record the defect — do not refactor.

---

## §4. Test strategy — the invariant corpus (the deliverable)

**Acceptance bar:** the four new invariant fixtures below go green. Per
[`pending-retirement.md`](./pending-retirement.md) §4, U-CORE-2 flips **no
pending fixture** ("its combinators already landed `0da64d6`; no pending fixture
is gated on the residue"). In particular `control-flow/control_flow_iftrue_iffalse`
needs `Option#unwrapOr`, which is **U-STD**, not this unit — leave it in
`pending/`. This unit's acceptance is the invariant corpus, not a pending flip.

**Home = the golden corpus** (`tests/lang/…`, exact-stdout snapshot), matching the
`0da64d6` precedent; this is the "C — corpus" surface of
[`invariant-requirements.md`](./invariant-requirements.md) §1 (user-observable
`.ph` output). All are **corpus**, none is **boot** (`verify_invariants`). Each
`.ph` carries the standard header (`// area:`, `// spec:`, `// status: PASS`).

> **Design constraint on the fixtures (avoid cross-unit coupling).** Prefer
> printing `Bool` results (`isSome`/`isNone` → `true`/`false`) or `match`-extracted
> values, **not** a bare `Some`/`None`, so the `.expected` does not depend on the
> `Value::to_string` rendering that **U-CORE-4 will change**. The one exception is
> the empty-body case, whose `<Some instance>` rendering is already pinned by the
> *existing* `absence_iftrue_empty_body_is_some_none` fixture (that one file is
> re-blessed by U-CORE-4, not here).

### R-INV-2.1 — Fast-path ≡ deopt-path Some-lift (the load-bearing WrapSome check)

**Claim:** an inlined `ifTrue { A }` and the *same* site after the sacred epoch
has flipped both yield `Some(A)` on the taken arm and `None` on the untaken arm.

**Mechanism to exploit:** `GuardBool` takes the fast path iff the receiver is a
`Bool` **and** `universe.bool_sacred_pristine` (`vm.rs` L1135). Reopening `Bool`
to (re)install **any** sacred selector flips that flag via `note_method_installed`
(`universe.rs` L214–220, called from `vm.rs` L907), deopting every inlined `Bool`
site to a real send — which still resolves to the kernel `bool_if_true`/`bool_if_false`
(the reopen redefined a *different* sacred selector), so the `Some`-lift is now
exercised through the **primitive** path.

**Two fixtures, identical `.expected`** (that identity *is* the assertion):

- `absence/absence_iftrue_some_lift_fast_path.ph` — pristine (inlined):
  ```phalcom
  System.print(true.ifTrue { 42 }.isSome)     // true
  System.print(false.ifTrue { 42 }.isSome)    // false  (untaken → None)
  System.print(false.ifTrue { 42 }.isNone)    // true
  ```
- `absence/absence_iftrue_some_lift_deopt_path.ph` — flip the epoch first, then
  the *same* three lines:
  ```phalcom
  class Bool { not() { return false } }        // reopen a SACRED selector ≠ ifTrue → flips bool_sacred_pristine
  System.print(true.ifTrue { 42 }.isSome)     // true  (now via bool_if_true deopt)
  System.print(false.ifTrue { 42 }.isSome)    // false
  System.print(false.ifTrue { 42 }.isNone)    // true
  ```
  Both `.expected`:
  ```
  true
  false
  true
  ```

Also fold in the **empty-body** taken-arm case on both paths (guards §0.1-B):
`System.print(true.ifTrue { }.isSome)` → `true` on fast *and* deopt path (proves
`Nil; WrapSome` → `Some(None)`, and the primitive `wrap_some(None-singleton)`,
both without tripping the Invariant-4 assert).

### R-INV-2.2 — Pop-context elision is observationally invisible

**Claim:** discarding an `ifTrue`/`ifFalse` result (statement position, where
`WrapSome` is elided) produces identical program output to a run that uses the
result. The elided `Some` allocation never changes semantics or the block's side
effects, and never errors.

**Fixture** `absence/absence_iftrue_pop_elision_invisible.ph`:
```phalcom
// statement position → WrapSome elided; body must still run exactly once
true.ifTrue  { System.print("taken") }        // -> taken
false.ifTrue { System.print("skip")  }        // (untaken: no output, no error)
true.ifFalse { System.print("skip")  }        // (untaken: no output, no error)
false.ifFalse{ System.print("takenF") }       // -> takenF
// value position (WrapSome present) over the SAME bodies: side effect fires once, Some still observable
System.print(true.ifTrue { System.print("effect"); 1 }.isSome)   // -> effect \n true
```
`.expected`:
```
taken
takenF
effect
true
```
This pins: (a) elided statement-position `ifTrue`/`ifFalse` runs the taken body
**exactly once** and the untaken arm prints nothing and does not error; (b) the
value-position twin fires the side effect once *and* still yields a `Some`
(`isSome == true`) — so eliding the allocation is invisible.

### R-INV-2.3 — Paired conditional and `and`/`or` are still RAW (not Some-lifted)

**Claim:** the `Some`-lift is **one-armed only**. `ifTrue(_, ifFalse)`, `and(_)`,
`or(_)` return their block result **raw**, never wrapped
([`catalog-delta.md`](./catalog-delta.md) §4.2; ADR-0018 amendment).

**Fixture** `control-flow/control_flow_paired_and_or_raw.ph` — send a `Number`
message to the result; if it were `Some(_)`-lifted, `Some` would `dnu '+(_)'`:
```phalcom
System.print((3 > 2).ifTrue(10, ifFalse: 20) + 1)   // 11  (raw 10, not Some(10))
System.print((2 > 3).ifTrue(10, ifFalse: 20) + 1)   // 21
System.print((true.and  { 5 }) + 1)                 // 6   (raw 5)
System.print((false.or  { 7 }) + 1)                 // 8   (raw 7)
System.print(true.and  { 5 })                       // 5
System.print(false.or  { 7 })                       // 7
```
`.expected`:
```
11
21
6
8
5
7
```
*(Also confirms the **inliner** paths: `compile_if_true_if_false` (inliner.rs
L339–357) emits **no** `WrapSome`, and `compile_and`/`compile_or` (L363–399)
leave the value raw — matching the `bool_if_true_if_false`/`bool_and`/`bool_or`
primitives.)*

### R-INV-2.4 — Combinators route through `match` (respect an override)

**Claim:** every `Option` combinator is defined over `match` (no combinator peeks
at a variant tag), so overriding `match` reroutes `isSome`/`isNone`/`ifNone`/`orElse`
([`values-and-absence.md`](../values-and-absence.md) §3.3;
[ADR-0007](../../adr/0007-option-as-abstract-with-some-none.md)'s "dispatch
replaces branching").

**Today-executable form (reopen `Option`, override `match`).** A genuine
*subclass* (`class MyOpt : Option { … }`) needs `:` inheritance syntax, which is
**U-LEX** — so the acceptance fixture reopens `Option` and overrides `match` to
prove the routing property with syntax that exists now. A pathological override
that always drives the `none` arm must flip every combinator's answer:

**Fixture** `absence/absence_combinators_route_through_match.ph`:
```phalcom
// Baseline: real match — Some drives the some: arm, None drives none:
System.print(Some.new(1).isSome)                       // true
System.print(None.isNone)                              // true
// Override match on Option to ALWAYS take the none: arm.
class Option { match(some, none) { return none.call() } }
// Every combinator now reflects the override (proving they route through match,
// not a variant tag): a Some reports itself absent.
System.print(Some.new(1).isSome)                       // false
System.print(Some.new(1).isNone)                       // true
System.print(Some.new(1).orElse { Some.new(9) }.match(some: { v => v }, none: { -1 }))  // 9
```
`.expected`:
```
true
true
false
true
9
```
> **Residual risk to verify:** this reopen *replaces a floor primitive*
> (`option_match`) with a `.ph` body on a kernel class. Confirm kernel-class
> primitive→`.ph` replacement via reopen works for `match` exactly as ADR-0018
> deviation #3 (class reopening) provides for `Block`. If it does not, downgrade
> this fixture to **U-LEX-gated** (the genuine `class MyOpt : Option` subclass
> form) and land a Rust-level `tests/invariants.rs` twin that calls the combinator
> closures against a stub whose `match` is overridden. Prefer the corpus form if
> the reopen works.

### R-INV-0.3 — Absence non-surfacing at boot (COORDINATE, do not duplicate)

- **Boot half** (`verify_invariants`): the `None` singleton is a live value
  distinct from `Value::Nil`, and the `None` **global** resolves to that
  singleton *value*, not the `None` **class**
  ([`invariant-requirements.md`](./invariant-requirements.md) §3, R-INV-0.3).
  This is **owned by U-CORE-1** (R-INV-1.1 "closes R-INV-0.1…0.4"). **U-CORE-2
  must NOT add it** — doing so would edit `universe.rs::verify_invariants`, a
  file U-CORE-1 owns, risking write-contention/merge churn.
- **Corpus half** already exists and is green: `tests/invariants.rs`
  `sentinel_surfaces_to_none_and_never_survives_as_nil` and
  `expression_result_absence_surfaces_to_none` (L48–115). U-CORE-2 *depends on*
  and *reaffirms* it — the new R-INV-2.1 empty-body case (§0.1-B) is an
  additional behavioural witness that the sentinel never surfaces raw through the
  `WrapSome` path.
- **Coordination rule:** if U-CORE-1 lands first, U-CORE-2 need do nothing here.
  If U-CORE-2 lands first, it still adds nothing to `verify_invariants`; it relies
  on the existing corpus guard and flags R-INV-0.3's boot half as a U-CORE-1
  obligation. **Say boot vs corpus:** boot → U-CORE-1; corpus → already green.

### 4.x Wiring

All four new `.ph` fixtures live under existing labels already registered in
`tests/lang.rs` (`fn absence()` → `check_pass("absence")` L97–100; `fn
control_flow()` → `check_pass("control-flow")` L76–79). No new test-fn or harness
change is needed — dropping the `.ph`/`.expected` pairs into `tests/lang/absence/`
and `tests/lang/control-flow/` is sufficient (`support::collect_cases` globs the
directory).

---

## §5. Must-not-preclude

### 5.1 `forward-compat.md` §2 — `Option`↔`Result` shape parity (the section this unit must clear)

Per [`forward-compat.md`](./forward-compat.md) §5, U-CORE-2's applicable section
is **§2 (error mechanism), specifically the `Option`↔`Result` shape-parity
constraint**: `Result`/`Ok`/`Err` must be able to mirror `Option`/`Some`/`None`
(abstract root + two concrete subclasses, one field each), and the `Some`/`None`
machinery + `WrapSome`-style helpers must be shareable.

**Walk-through — nothing here trips it:**

1. **No shape change to `Option`.** This unit adds *zero* fields, *zero*
   representation changes to `Option`/`Some`/`None`. `Some` remains one field
   (`_value` at slot 0); `None` remains the shared singleton. A future
   `Result`/`Ok`/`Err` can be stamped with the identical abstract-root +
   two-subclass layout (`Ok._value`, `Err._error`) with no conflict.
2. **Combinators are `match`-derived, so the pattern ports.** `isSome`/`isNone`/
   `ifNone`/`orElse` are pure `.ph`-over-`match` (core.ph L42–60). `Result`'s
   `isOk`/`isErr`/`orElse`/`mapErr` will be the same shape over `match(ok, err)`.
   R-INV-2.4 *locks in* this "route through the eliminator" discipline, which is
   exactly what makes the sibling layerable.
3. **`WrapSome` / `wrap_some` are `Some`-specific — and that is fine.** The
   `Some`-lift exists because a `Bool` maps to an `Option` (`ifTrue`), a mapping
   with **no `Result` analogue** — booleans do not lift to `Ok`/`Err`. So there
   is no missing symmetric "`WrapOk`" this unit was obliged to leave room for; the
   opcode's `Some`-specificity does **not** give `Ok`/`Err` an ad-hoc
   representation the shared machinery can't reach (`forward-compat.md` §2(b)).
   If a later unit ever wants a `WrapOk`, it is an *additive* opcode, not a
   reshape.
4. **No forked unwind, no second error channel.** This unit touches neither the
   unwind path nor any error channel; the U10 non-local-return unwind that
   `throw` will reuse is untouched (`forward-compat.md` §2(c)).

**Verdict: clears §2.** The `Option`↔`Result` bridge (`okOr`, `ok`) remains
layerable later over `match` without re-shaping `Option`.

### 5.2 Reserve, don't build, `Result`

Consistent with [`decisions.md`](./decisions.md) Q2 (confirm ADR-0008; U-CORE-6
reifies `Error` and **reserves** `Result`): U-CORE-2 must not sneak in
`Result`/`Ok`/`Err`. It does not.

### 5.3 U11 relationship — surface vs representation (do NOT absorb)

U11 (abstract `Bool` + `True`/`False` singletons, [ADR-0004](../../adr/0004-boolean-as-abstract-bool-with-true-false.md))
is a **separate** forge unit about Bool *representation* (today `Bool` is the
immediate `Value::Bool(bool)`, and `GuardBool` matches `Value::Bool(_)`, `vm.rs`
L1135). U-CORE-2 is about the `Option`-returning *surface* of `ifTrue`/`ifFalse`
— **orthogonal**. Complementarity, with a must-not-preclude edge:

- The R-INV-2.x fixtures are **behavioural** (`.ifTrue { A }.isSome`,
  `and`/`or` truth), never keyed on `Value::Bool` layout, so they survive U11's
  representation swap unchanged. Do not write a fixture that pins Bool's *identity*
  or `Value` arm.
- U11 will re-parent the sacred `Bool` selectors onto `True`/`False`; the sacred
  set and epoch (`BOOL_SACRED_SELECTORS`, `bool_sacred_pristine`) must stay the
  compiler-coupled interface (floor-census §5). Nothing in U-CORE-2 changes that
  set, so U11 is not boxed in.

---

## §6. Open sub-decisions and traceability

### 6.1 Open sub-decisions

**None blocking.** One implementer-facing contingency, resolvable during
implementation (not a user decision):

- **BD-U-CORE-2-A (implementation contingency, not a user gate):** whether R-INV-2.4
  ships as a **reopen-`Option`** corpus fixture (preferred, executable today) or a
  **U-LEX-gated subclass** fixture + Rust twin — decided by whether kernel-class
  primitive→`.ph` `match` replacement via reopen works (see §4 R-INV-2.4 residual
  risk). Default: reopen form. No user input required.

### 6.2 Traceability

| Requirement | Spec / ADR anchor | As-built (verify) | Acceptance |
|---|---|---|---|
| `ifTrue`/`ifFalse` return well-formed `Option` | values-and-absence §3; catalog-delta §4.2; ADR-0018 amendment | `boolean.rs` L115–140; `nil.rs` L47–58 | R-INV-2.1 fixtures |
| Fast path ≡ deopt path (Some-lift) | ADR-0018 amendment; invariant-req R-INV-2.1 | `inliner.rs` L296/L324; `vm.rs` L982–986, L1135; `universe.rs` L214–220 | `absence_iftrue_some_lift_{fast,deopt}_path` |
| Pop-context elision invisible | ADR-0018 amendment ("Allocation elision") | `lib.rs` L502/L509; `inliner.rs` L148/L295/L323 | `absence_iftrue_pop_elision_invisible` |
| Paired / `and` / `or` still raw | catalog-delta §4.2; invariant-req R-INV-2.3 | `boolean.rs` L152–155; `inliner.rs` L339–399 | `control_flow_paired_and_or_raw` |
| Combinators route through `match` | values-and-absence §3.3; ADR-0007; invariant-req R-INV-2.4 | `core.ph` L42–60; `nil.rs` L92–116 | `absence_combinators_route_through_match` |
| Absence non-surfacing (boot) | invariant-req R-INV-0.3 | `universe.rs::verify_invariants` | **U-CORE-1 owns**; corpus half green in `invariants.rs` L48–115 |
| Empty-body `Some(None)` safe vs Invariant 4 | values-and-absence Inv 4; ADR-0010 | `vm.rs` L804; `nil.rs` L48–51 | R-INV-2.1 empty-body line; existing `absence_iftrue_empty_body_is_some_none` |
| No floor delta | ADR-0019; floor-census §2.6/§2.8 | census unchanged = 73 | R-INV-0.1 audit (U-CORE-1), stays unmoved |
| `Option`↔`Result` parity preserved | forward-compat §2 | no `Option` shape change | §5.1 walk-through |
| Flips no pending fixture | pending-retirement §4 | `control_flow_iftrue_iffalse` stays in `pending/` (needs U-STD `unwrapOr`) | n/a |
