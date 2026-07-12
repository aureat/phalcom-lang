# U-CORE-5 — Collection Protocol **CONTRACT** — Implementation Spec

> **Status:** Normative implementation spec. U-CORE-5 delivers the **shared
> collection-protocol contract** (selectors + laws) that the kernel `List`
> already satisfies as the reference implementation and that every *future*
> collection (`Map`/`Set`/`Tuple`/`Range`) must satisfy — **not** new collection
> classes (those are U-STD / deferred, per [ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md)).
> Its concrete artifacts are: (a) a **reusable conformance harness keyed by "the
> collection under test,"** (b) the minimal `.ph` needed to make `List` a *fully*
> conformant reference implementation, and (c) the golden corpus that pins the
> laws. It adds **zero** floor primitives — no [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md)
> amendment.

> **Baseline:** HEAD `8733afe`. Recommended-order position (per
> `implementation-status.md`'s spine, unchanged by this refresh): **U-CORE-1,
> U-CORE-2, U-CORE-3 are landed**; **U-CORE-4 is in flight right now** (a
> concurrent session is editing `value.rs`/`universe.rs`/`primitive/number.rs`/
> `core.ph` to add the `Object#toString` class-receiver fix + `Number#toString`);
> **U-CORE-5 runs after U-CORE-4 lands.** **Floor is 85 installed bindings / 69
> distinct fns today** (post-U-CORE-3 — `floor-census.md` §1.1); U-CORE-4 is
> expected to land it at **86 / 70** (+1 `Number#toString`, its own ADR-0019
> amendment, visible in `universe.rs`'s currently-dirty tree). **This unit's own
> floor delta is +0**, expressed as math on whichever of those two numbers is
> current at dispatch — do not hardcode a literal (see §3.1).
> Encodes [`decisions.md`](../../../spec/v0.2/core/decisions.md) **Q5**
> (mutability + equality + hashability). Depends on **U-CORE-1** (`isA(_)` and
> `hash` — **confirmed landed**, commit `03764e3`,
> [`decisions.md`](../../../spec/v0.2/core/decisions.md) Q1). See
> [`README.md`](../../../spec/v0.2/core/README.md) for the baseline-pin policy.

---

## §0. Scope gate — what this unit is and is **not**

### 0.1 In scope

1. The **normative contract**: the sequence-protocol selectors (`size`, `at(_)`,
   `add(_)`, `each(_)`) and the **laws** governing them (totality, deterministic
   iteration, structural equality, hashability-iff-immutable) — the
   [`decisions.md`](../../../spec/v0.2/core/decisions.md) Q5 ruling made executable.
2. A **reusable conformance harness** parameterized by *the collection under
   test* (a build-closure + a `ContractSpec`), so that when `Map`/`Set`/`Tuple`
   land (U-STD) they are certified against **this** corpus rather than ad-hoc
   tests ([R-INV-5.4](../../../spec/v0.2/core/invariant-requirements.md)).
3. The **minimal `.ph`** that makes `List` satisfy the *whole* contract today —
   specifically a **structural `==`** (and its paired `!=`). `List` today has only
   **identity** equality (§1.3); without this it is not a conformant reference
   implementation, and the contract would have no green reference.
4. The **golden corpus** (`tests/lang/collections/`) that pins the user-observable
   laws against `List` in already-supported syntax.

### 0.2 Explicitly OUT of scope (do not implement here)

| Deferred item | Owner | Why not here |
|---|---|---|
| `Tuple` / `Map` / `Set` / `Range` **classes** | U-STD / collections | ADR-0020: each collection is its own later unit. U-CORE-5 is the *contract they conform to*. |
| `List#map`/`reduce`/`filter`/`includes`/`isEmpty`/`at(_,put)` | **U-STD — already landed** (`core.ph` L162–211, commit `5e2b395`, predates this unit's own dispatch) | catalog-delta §2.4 (now reads "✅/✅ … U-STD — combinators landed; literal syntax still open"). This unit does **not** touch them — it only certifies the `each`/`size`/`at` substrate they are built over stays contract-conformant. Do **not** re-derive or duplicate them. |
| Collection **literal syntax** `[…]`/`{…}`/`(…)`/`#{…}` | U-LEX | pending-retirement §3 cat C. |
| Making `List#hash` **raise** (mutable-key enforcement) | U-CORE-6 + U-STD | needs the error mechanism (ADR-0008) and a `Map`/`Set` key-boundary; neither exists yet (§2.4). The *law* is specified here; *enforcement* is a consumer obligation. |
| Any new native primitive | — (frozen) | ADR-0019 floor stays wherever U-CORE-4 lands it (**85** today → **86** expected; §2.1). Re-read `floor-census.md` §1.1 live at dispatch — do not hardcode a number. |

> **Staleness note (re-grounded at HEAD `8733afe`).** At the time this unit was
> first drafted, the derived combinators above were still hypothetical future
> work. They have since landed on `main` (`5e2b395` "U-STD: List combinators",
> `176d454` "U-STD: Option combinators") — **ahead of, and independent of**, this
> unit's own sequencing. This does **not** shrink U-CORE-5's scope: the one gap
> this unit still closes — structural `List#==`/`!=` — is **re-confirmed still
> absent** from `core.ph` (grep-verified: no `==`/`!=` override anywhere in the
> `List` reopen at HEAD `8733afe`). Golden fixtures for the already-landed
> combinators already exist at `phalcom-core/tests/lang/list/list_map_and_filter.ph`,
> `list_reduce_sum.ph`, `list_includes_and_isempty.ph`, `list_at_put.ph` — they
> guard `List`'s *own* unit, same rule as the pre-existing
> `list_to_string_renders_brackets.ph`/`list_each_sums_elements.ph` callout in
> §3.3(b): do **not** duplicate them under `collections/`.

### 0.3 Prerequisites (must be landed before this unit runs)

- **U-CORE-1 — ✅ landed** (`03764e3`) — provides `Object#isA(_)` (the type guard
  used by `List#==`, catalog-delta §4.5, invariant 1.2) **and** `Object#hash` (Q1,
  the hashability law's precondition, ADR-0019 amendment). `isA(_)` is confirmed
  present at `core.ph` L9 (derived over `class`/`==`/`superclass`, no native
  primitive — `floor-census.md` L48–49). This was a **hard, ordered dependency**
  (the `.ph` `==` body dispatches `other.isA(List)`); it is now **satisfied**.
- **U-LIST** (ADR-0020) — the native-array `List` and its `size`/`at`/`add`/`each`
  protocol. Landed; re-verified current at `core.ph` L143–160 (⚠ *`core.ph` is one
  of U-CORE-4's concurrently-edited files — re-confirm this range at dispatch*).
- **U10** — non-local `return` from blocks (used only incidentally; the `.ph`
  bodies below avoid block-return by using structured `if/else`). Landed.
- **U-CORE-3 — ✅ landed** (`10ebd06`) — *newly surfaced dependency on
  re-grounding*: the combinators U-STD has since layered onto the `List` reopen
  (`map`/`filter`/`reduce`/`includes`, §0.2) call `f.call(...)` throughout, i.e.
  they depend on U-CORE-3's `Block`/`Function` `call` reflection surface. That
  surface is landed, so this is satisfied — but it was **not previously stated**
  in this doc and is worth recording precisely: U-CORE-5's *own* deliverable
  (structural `==`/`!=`) needs no `Block#call` at all — only `isA`/`size`/`at`/
  `while`/`and`/`not` — so U-CORE-5 itself has **no** hard dependency on
  U-CORE-3; only the pre-existing combinators sharing its `List` reopen do.
- **U-CORE-4 (in flight)** — **no dependency, confirmed**. U-CORE-4's
  `Object#toString` fix changes class-receiver rendering and adds
  `Number#toString`; `List` already has its own native `toString`
  (`list_to_string`, `primitive/list.rs`) that overrides the `Object` default, so
  U-CORE-4 landing changes nothing this unit relies on.

---

## §1. What exists vs what is missing (grounded)

### 1.1 The reference implementation exists — `List` sequence surface

`List` is a native `Vec<Value>`-backed heap object
([`phalcom-core/src/list.rs`](../../../../phalcom-core/src/list.rs) L21–76,
`ListObject` — **re-verified current at HEAD `8733afe`, unchanged**) with five
raw floor primitives + native `toString`
([`phalcom-core/src/primitive/list.rs`](../../../../phalcom-core/src/primitive/list.rs) —
**re-verified current**; `list_raw_at` still at L72–79), and a `.ph` public
protocol ([`phalcom-core/core/core.ph`](../../../../phalcom-core/core/core.ph)
**L142–212** ⚠ *`core.ph` is one of U-CORE-4's concurrently-edited files —
re-confirm this range at dispatch*). The sequence-protocol core
(`size`/`at`/`add`/`each`) sits at **L143–160**; U-STD has since landed
`map`/`filter`/`reduce`/`includes`/`isEmpty`/`at(_,put:)` in the same reopen
(**L162–211**, commit `5e2b395`) — **pre-existing, not built by this unit**
(§0.2):

```
size    => self.rawLength
at(i)   { return self.rawAt(i) }
add(v)  { self.rawPush(v); return self }
each(f) { var i = 0; while (i < self.size) { f.call(self.at(i)); i = i + 1 } }
```

These already satisfy the **structural laws** of the contract (R-INV-5.1/5.2):
`size ≥ 0`, `at(i)` total for `0 ≤ i < size` and surfacing `None` out of range
(`list_raw_at`, primitive/list.rs L72–79), `add` growing `size` by 1, and `each`
visiting each element once in insertion order via a **local** cursor `i` (no
collection-global iteration state — see §5.2).

### 1.2 What Q5 mandates that is **already** true of `List`

- **Mutable by default.** `List` mutates through `rawPush`/`rawSet` over the
  native `Vec` (ADR-0020); no `freeze`/immutable view exists (correctly — that is
  opt-in *later*). ✅
- **Deterministic iteration.** Insertion order. ✅

### 1.3 What Q5 mandates that is **NOT yet true** of `List` — the gap this unit closes

- **Structural `==` for sequences.** Q5: *"`==` is structural for sequences —
  element-wise, order-sensitive, comparing with each element's own `==`."* Today
  `List` inherits `Object#==` (`object_eq`,
  [`primitive/object.rs`](../../../../phalcom-core/src/primitive/object.rs)
  **L119–121** ⚠ *re-grounded at HEAD `8733afe`; `primitive/object.rs` is
  currently dirty under the concurrent U-CORE-4 session (adding
  `object_to_string`) — re-confirm at dispatch*), which delegates to
  `Value::value_eq` ([`value.rs`](../../../../phalcom-core/src/value.rs)
  **L245–272** ⚠ *`value.rs` is one of U-CORE-4's concurrently-edited files —
  re-confirm at dispatch*). For two `List` handles that arm falls through to
  `a == b` **handle identity** (value.rs **L266**, inside the
  `(Value::Obj(a), Value::Obj(b))` match arm — ⚠ same caveat). So `List`
  equality is **identity, not structural** — `[1,2] == [1,2]` on two distinct
  lists is **`false`** today (re-confirmed: no `==`/`!=` override exists
  anywhere in the `List` reopen at HEAD `8733afe`). This unit adds a structural
  `List#==`.
- **The `!=` decoupling hazard.** `object_neq` (primitive/object.rs **L125–127**
  ⚠ same dirty-tree caveat) negates `value_eq` **directly** — it does *not*
  dispatch `self.==`. So overriding only `==` would leave `list != other` still
  identity-based and **inconsistent** with the new `==`. `List#!=` must be
  overridden in lockstep (§3.2).
- **Hashability.** `hash` now **exists** (U-CORE-1 landed, `03764e3`,
  `Object#hash` = an identity digest) — which is precisely the danger Q5 warns
  about: `List`'s inherited identity `hash` is **inconsistent** with the
  structural `==` this unit adds (two structurally-equal lists on different
  handles would compare equal by `==` but unequal by `hash`). Q5: *mutable
  collections are not hashable; immutable ones are.* `List` (mutable) must
  **not** be a valid `Map`/`Set` key. There is still nothing to *enforce*
  against today (no `Map`/`Set`, no error mechanism — U-CORE-6) — so this
  remains **specified now, enforced by consumers** (§2.4), now with a
  concretely-landed `hash` to specify *against*.

### 1.4 Substrate the `.ph` `==` needs (confirmed present / absent)

| Need | Present? | Source |
|---|:--:|---|
| `if/else`, `while` control flow | ✅ | `control_flow_if_else` graduated U5; `core.ph` `each` uses `while` |
| `Bool#not()`, `Bool#and(_)` | ✅ | floor-census §2.6 (sacred) |
| `Number#<(_)`, element `==` (via `Object#==`) | ✅ | floor-census §2.1/§2.4 |
| `Object#isA(_)` (the type guard) | ✅ **landed** (U-CORE-1, `03764e3`) | catalog-delta §4.5; confirmed present at `core.ph` L9 (`isA(cls) { ... }`), derived over `class`/`==`/`superclass`, no native primitive (`floor-census.md` L48–49) |
| `===` identity operator (for an identity fast-path) | ❌ (re-confirmed) | grep confirms no `===` in `phalcom-ast`; see SD-2 |

---

## §2. The contract (normative)

The contract is **keyed by the collection under test**: every law is a predicate
over a collection value `C` built from a known element sequence `e₀…eₙ₋₁`, plus
two static parameters that distinguish collection kinds:

| Parameter | `List` | `Tuple` | `Map`/`Set` | Meaning |
|---|:--:|:--:|:--:|---|
| `mutable` | ✅ | ❌ | ✅ | supports in-place `add`/growth |
| `hashable` | ❌ | ✅ | ❌ | is a valid `Map`/`Set` key |
| `ordered` | ✅ | ✅ | ❌ (`Set`/`Map`) | iteration order is a function of construction |

### 2.1 The sequence laws (R-INV-5.1 / 5.2)

For an *ordered sequence* collection `C` with `n` elements:

| Law | Statement |
|---|---|
| **L1 size total** | `C.size` is a `Number` and `C.size ≥ 0`. |
| **L2 size = count** | `C.size == n`. |
| **L3 at total in range** | for `0 ≤ i < n`, `C.at(i)` returns `eᵢ` (by the element's own `==`). |
| **L4 at absence out of range** | `C.at(n)` (and any `i ≥ size`) returns the **`None`** singleton, never a panic, never the `nil` sentinel (Invariant 4). |
| **L5 add grows** *(mutable only)* | after `C.add(x)`, `C.size` is `n+1` and `C.at(n) == x`; `add` returns `C` (chainable). |
| **L6 iteration order** | `each` visits exactly `[C.at(0), C.at(1), …, C.at(size-1)]`, in that order, **each element exactly once** — `size` visits total. |
| **L7 stateless iteration** | iteration holds its cursor **locally**, never on `C`; `each` is reentrant (nested/self `each` compose) and carries no global-stack assumption (§5.2). |

### 2.2 The equality laws (R-INV-5.3) — **structural, Q5**

For collections `A`, `B` of the same kind:

| Law | Statement |
|---|---|
| **E1 structural** | `A == B` iff `A.size == B.size` **and** `A.at(i) == B.at(i)` for all `i` (order-sensitive, via each element's own `==`). For `Set`/`Map` (unordered), "same elements/entries" replaces index-wise. |
| **E2 cross-kind** | `A == B` is `false` when `B` is not the same collection kind (guard: `B.isA(<kind>)`). Follows Python (`[1,2] != (1,2)`). |
| **E3 reflexive** | `A == A` is `true`. *(Caveat: holds for well-behaved elements; a `NaN` element breaks it — see SD-2 and §Risk.)* |
| **E4 symmetric** | `(A == B) == (B == A)`. |
| **E5 transitive** | `A == B ∧ B == C ⇒ A == C`. |
| **E6 `!=` is `¬==`** | `A != B` is `(A == B).not()` — routed through `==`, **not** through floor identity (§1.3, §3.2). |

### 2.3 The hashability law (R-INV-5.3, forward — feeds R-INV-1.3)

| Law | Statement |
|---|---|
| **H1 hashable ⟺ immutable** | `mutable ⇒ ¬hashable`; `immutable ⇒ hashable`. `List` is **not** a valid `Map`/`Set` key; `Tuple` is. |
| **H2 hash ⇔ `==`** *(hashable kinds only)* | for a hashable collection, `A == B ⇒ A.hash == B.hash`, derived from element hashes (order-sensitive combine), **never** from element representation (§5.1). This is the `Map`/`Set` key precondition and directly feeds R-INV-1.3. |
| **H3 mutable key rejected** *(enforcement, consumer)* | a `mutable` collection used as a `Map`/`Set` key is **rejected** (raises, once errors + `Map`/`Set` land). U-CORE-5 **specifies** this; it does not enforce it (§2.4). |

### 2.4 Why hashability is specified-not-enforced here (the honest framing)

`List` **inherits** `Object#hash` (an *identity* digest, U-CORE-1, landed). That
identity hash is **inconsistent** with `List`'s new *structural* `==` (two equal
lists, different handles ⇒ equal by `==` but unequal by identity `hash`) — which is
*precisely why* Q5 forbids mutable collections as keys. Making that safe requires
one of:

- `List#hash` **raises** "mutable collections are not hashable" — needs the error
  mechanism (**U-CORE-6**, ADR-0008), which lands *after* U-CORE-5; or
- `Map`/`Set` key-insertion **rejects** `mutable` keys — needs `Map`/`Set`
  (**U-STD**).

Neither consumer exists at U-CORE-5 time. Therefore U-CORE-5 **states H1–H3 as
normative law** and encodes `hashable = false` for `List` in the harness (so the
H2 consistency assertion is *skipped* for `List` — correct, `List` is a non-key),
leaving **enforcement** as a `ContractSpec`-driven obligation the consumers
inherit via R-INV-5.4. This is the task's "flips nothing directly; enables what
U-STD flips" reality, stated plainly.

---

## §3. Native-vs-`.ph` split + exact changes

### 3.1 Native (Rust floor) — **NONE**

U-CORE-5 adds **no** native primitive. **Re-grounded at HEAD `8733afe`: the
"73/57" figure this section originally cited is stale** — two amendments have
landed since this doc was drafted (U-CORE-1 / ADR-0023: 73→80 bindings, 57→64
fns; U-CORE-3 / ADR-0028: 80→85, 64→69), so the floor is **85 installed bindings
/ 69 distinct fns today**, with U-CORE-4 (in flight) expected to land it at
**86/70** (+1 `Number#toString`, its own ADR-0019 amendment — see
`universe.rs`'s currently-dirty tree). **No ADR-0019 amendment from this unit.**
The census audit (R-INV-0.1) must continue to read **whatever count U-CORE-4
leaves it at** — read `floor-census.md` §1.1 live at dispatch time rather than
hardcoding 73, 57, 85, 69, 86, or 70. Structural `==` is *derivable* over the
existing floor (`size`, `at`, `isA`, element `==`, `Number#<`, `Bool#not`/`and`,
`while`), so it belongs in `.ph` per the census §3 boundary rule.

> If a future implementer is tempted to add a native `list_eq` primitive "for
> speed," that is an **ADR-0019 amendment**, and it is a **speed** item — it goes to
> the deferred register, not this unit. The `.ph` derivation is the correct default.

### 3.2 `.ph` — the reference-implementation conformance patch (`core.ph`, `List` reopen)

**Write-set:** `phalcom-core/core/core.ph`, the `List { … }` reopen (currently
L142–212 ⚠ *re-confirm at dispatch — `core.ph` is one of U-CORE-4's
concurrently-edited files*). Add two methods. This is the **only**
production-code change in U-CORE-5.

```phalcom
class List {
  // ... existing size / at(i) / add(v) / each(f) / map / filter / reduce /
  // includes / isEmpty / at(i, put:) unchanged ...

  // Structural equality (decisions.md Q5, R-INV-5.3 E1–E5): element-wise,
  // order-sensitive, via each element's own `==`. Guarded by `isA(List)` so a
  // non-List `other` is simply unequal (E2), never a dNU. Derived entirely over
  // the floor — no new primitive (ADR-0019 unchanged). Requires U-CORE-1's
  // `isA(_)`.
  ==(other) {
    if (other.isA(List)) {
      var same = (self.size == other.size)
      var i = 0
      // `and(_)` is eager; when `same` is already false we still evaluate the
      // cheap bound and exit — `at(i)` runs only while both hold, so `i < size`
      // is guaranteed in the body.
      while (same.and(i < self.size)) {
        same = (self.at(i) == other.at(i))
        i = i + 1
      }
      return same
    } else {
      return false
    }
  }

  // `!=` MUST route through `==` (R-INV-5.3 E6). The floor `Object#!=`
  // (`object_neq`) negates identity `value_eq`, NOT `self.==`, so without this
  // override `list != other` would stay identity-based and contradict the
  // structural `==` above. This is the `==`⊗`!=` decoupling hazard (§1.3).
  !=(other) {
    return (self == other).not()
  }
}
```

Notes for the implementer:

- `==(other)` interns as `==(_:)` / `!=(_:)` (`SignatureKind::Method(1)`, matching
  how `object_eq`/`object_neq` install, universe.rs **L257–258** ⚠ *`universe.rs`
  is one of U-CORE-4's concurrently-edited files — re-confirm at dispatch*), so
  these shadow the floor `Object#==`/`Object#!=` via ordinary method lookup (the
  `object_eq` docstring explicitly anticipates subclass shadowing). `==` is
  **not** a sacred selector (only `Bool`/`Block` selectors are — floor-census
  §5), so there is **no inliner deopt to budget**.
- **Termination / correctness walk:** equal-size all-equal lists → `same` stays
  `true`, loop exits at `i == size`, returns `true`; first mismatch sets `same =
  false`, next condition check is `false.and(…) == false`, loop exits, returns
  `false`; unequal sizes → `same` starts `false`, loop body never runs, returns
  `false`; two empties → `same = (0 == 0) = true`, `true.and(0 < 0) = false`, returns
  `true`. Nested lists recurse into `List#==` (deep structural equality).
- **Single-branch `if` avoidance:** the body uses only `if/else` + `while` +
  `.and`/`.not` (all confirmed working). If the implementer prefers guard clauses
  (`if (cond) { return false }`), confirm single-branch `if` parses (it is the
  `Bool#ifTrue(_)` desugar and should) — otherwise keep the `if/else` form above.

### 3.3 The reusable conformance harness (corpus — R-INV-5.4)

Two surfaces, mirroring the repo's existing two invariant surfaces (in-process
`tests/invariants.rs` + shell-out `tests/lang/` goldens).

#### (a) In-process Rust harness — the **gate** — new file `phalcom-core/tests/collections_contract.rs`

Confirmed **not yet created** at HEAD `8733afe`. The literal realization of
"keyed by the collection under test": a `ContractSpec` plus a build-closure.
**U-STD certifies `Tuple`/`Map`/`Set` by calling the same function with a
different closure** — the contract is the gate, not each class's ad-hoc tests.

```rust
//! Reusable conformance harness for the collection-protocol contract
//! (docs/spec/v0.2/units/U-CORE/5-collection-contract.md). Keyed by "the collection
//! under test": a `ContractSpec` + a build-closure. New collections (U-STD)
//! are certified by adding a `build_*` closure and one `#[test]` — R-INV-5.4.

use phalcom_core::method::{make_signature, SignatureKind};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

/// The static parameters that distinguish one collection kind from another (§2).
struct ContractSpec {
    class_name: &'static str, // for diagnostics + the `isA` witness
    mutable: bool,            // List = true, Tuple = false
    hashable: bool,           // List = false, Tuple = true (gates the H2 assertion)
    ordered: bool,            // List/Tuple = true, Set/Map = false
}

/// Builds a `List` from element values through the *surface* protocol
/// (`List.new()` + `.add(_)`), so the harness exercises the same path user code
/// does. (Resolve the `List` class as the invariants corpus resolves core
/// classes — `vm.universe.classes.list_class` (confirmed present,
/// `universe.rs` L706) — or via the `List` global.)
fn build_list(vm: &mut VM, elems: &[Value]) -> Value { /* new(), then add() each */ }

/// Runs the full R-INV-5.x contract against whatever `build` produces. Every
/// send goes through `vm.send_dynamic(recv, sel, &args)` with selectors interned
/// via `make_signature` (cf. tests/invariants.rs).
fn assert_sequence_contract(vm: &mut VM, spec: &ContractSpec, build: impl Fn(&mut VM, &[Value]) -> Value) {
    // L1/L2: size >= 0 and == element count, for n = 0,1,2,3.
    // L3:    at(i) value_eq elem[i] for 0 <= i < n.
    // L4:    at(n) is the None singleton (not nil, not a panic).
    // L5:    if spec.mutable: add(x) grows size by 1 and at(oldSize) == x; add returns self.
    // E1/E3/E4: build A,B from equal elems -> (A==B) true, (A==A) true, symmetric.
    // E2:    (A == someNumber) is false (cross-kind).
    // E1/E5: build C differing at one index -> (A==C) false; A==B, B==A2 -> A==A2.
    // E6:    (A != C) == (A == C).not(); (A != B) == false.
    // H2:    if spec.hashable: A==B => hash(A)==hash(B). if !hashable: SKIP
    //        (List is a non-key; enforcement is a consumer obligation, §2.4).
}

#[test]
fn list_satisfies_sequence_contract() {
    let mut vm = VM::new();
    let spec = ContractSpec { class_name: "List", mutable: true, hashable: false, ordered: true };
    assert_sequence_contract(&mut vm, &spec, build_list);
}
```

Selector interning pattern (from `tests/invariants.rs`): `size` =
`make_signature("size", SignatureKind::Getter)`; `at`/`add`/`==` =
`make_signature(name, SignatureKind::Method(1))`; then `vm.get_or_intern(&sig)` →
`vm.send_dynamic(recv, sym, &args)`. Out-of-range absence is the `None` singleton
`vm.universe.classes.none_singleton` (invariants.rs precedent).

#### (b) `.ph` golden corpus — user-observable laws + the reuse **template**

New label directory `phalcom-core/tests/lang/collections/` (confirmed **not yet
created** at HEAD `8733afe`), wired by a `collections()` →
`support::check_pass("collections")` fn in
[`tests/lang.rs`](../../../../phalcom-core/tests/lang.rs) (mirroring `list()`,
re-verified now at **L194**, was L169 at the doc's original drafting). All
fixtures in **already-supported** `List.new()`/`.add(_)` syntax (no lexer
dependency). They double as the **template** U-STD copies per collection,
swapping only the constructor prologue.

| Fixture (`collections/`) | Proves | Law |
|---|---|---|
| `sequence_size_at_add.ph` | `size`/`at`/`add` round-trip + chaining | L1,L2,L3,L5 |
| `sequence_each_visits_in_order.ph` | `each` prints `at(0..size-1)` once, in order | L6 |
| `sequence_each_is_reentrant.ph` | nested `outer.each { inner.each { … } }` → no shared cursor | L7, forward-compat §1 |
| `sequence_structural_equality.ph` | `a == b` (equal), `a == c` (differ), `a != c`, `a == a` | E1,E3,E6 |
| `sequence_equality_is_deep.ph` | nested-list `==`, order-sensitivity, empty-list equality | E1,E2 |

Example — `sequence_structural_equality.ph`:

```phalcom
let a = List.new()
a.add(1)
a.add(2)
a.add(3)
let b = List.new()
b.add(1)
b.add(2)
b.add(3)
let c = List.new()
c.add(1)
c.add(9)
c.add(3)
System.print(a == b)   // true  — equal elements, in order (E1)
System.print(a == c)   // false — differs at index 1
System.print(a != c)   // true  — routed through == (E6)
System.print(a == a)   // true  — reflexive (E3)
```
`.expected`:
```
true
false
true
true
```

Example — `sequence_each_is_reentrant.ph` (fiber-safe / no shared cursor):

```phalcom
let outer = List.new()
outer.add(1)
outer.add(2)
let inner = List.new()
inner.add(10)
inner.add(20)
var total = 0
outer.each({ x =>
  inner.each({ y =>
    total = total + (x * y)
  })
})
System.print(total)    // 1*10 + 1*20 + 2*10 + 2*20 = 90
```
`.expected`: `90`

> **Do not duplicate** the existing `list/list_to_string_renders_brackets.ph`,
> `list/list_each_sums_elements.ph`, or — landed since this doc was originally
> drafted (U-STD, `5e2b395`) — `list/list_map_and_filter.ph`,
> `list/list_reduce_sum.ph`, `list/list_includes_and_isempty.ph`,
> `list/list_at_put.ph`. Those guard `List`'s *own* unit; the `collections/`
> corpus guards the *shared contract*.

---

## §4. Test strategy

### 4.1 Acceptance bar (green **today**, against `List`)

- `cargo test --test collections_contract` — `list_satisfies_sequence_contract`
  passes (the in-process gate). Confirmed **not yet created** at HEAD `8733afe`.
- `cargo test --test lang collections` — the `collections/` golden corpus
  passes. Confirmed **not yet created** at HEAD `8733afe`.
- `cargo test --test invariants` — unchanged; **R-INV-0.1 still reads whatever
  floor-binding count U-CORE-4 landed it at** (85 today, 86 expected
  post-U-CORE-4 — read it live rather than hardcoding "73"; proves no native
  primitive slipped in).

### 4.2 Invariants this unit adds

All of R-INV-5.1…5.4 are **corpus** ("C") assertions — **none** touch
`verify_invariants`/`universe.rs` (nothing here is a boot-soundness invariant), per
[`invariant-requirements.md`](../../../spec/v0.2/core/invariant-requirements.md) §4 U-CORE-5.

| R-INV | Contract law(s) | Where it lands |
|---|---|---|
| **5.1** size≥0, at total, add grows | L1,L2,L3,L4,L5 | `collections_contract.rs` (gate) + `sequence_size_at_add.ph` |
| **5.2** deterministic iteration, each visits `size` once | L6,L7 | `sequence_each_visits_in_order.ph`, `sequence_each_is_reentrant.ph` |
| **5.3** equality reflexive/symmetric/transitive + consistent-with-hash | E1–E6, H1,H2 | `collections_contract.rs` (==/!= algebra) + `sequence_structural_equality.ph`/`_equality_is_deep.ph`; **H2 forward-activated** when a `hashable` collection + `hash` are present |
| **5.4** future collections checked against this corpus | (all) | *is* the harness — U-STD adds a `build_*` closure + `#[test]`, reusing `assert_sequence_contract` verbatim |

### 4.3 `_pending` tests this unit flips — **NONE directly** (honest statement)

Per pending-retirement.md §4, U-CORE-5 (a contract, not classes — ADR-0020) **flips
no pending fixture on its own.** It **enables** fixtures that **U-STD / U-LEX** then
flip:

| Fixture (`…/pending/`) | Real blocker | Flips when |
|---|---|---|
| `blocks/blocks_argument_to_method` | ~~`[…]` literal + `List#reduce`~~ **already resolved** — `reduce` landed (`core.ph` L185–189, `5e2b395`); `pending-retirement.md` §2 confirms this fixture has already been `git mv`'d out of `pending/` (re-confirmed on disk: it now lives at `tests/lang/blocks/blocks_argument_to_method.ph`, no `pending/` in its path) | **already green — not a U-CORE-5-gated item** |
| `dispatch/dispatch_rest_param` | ~~`numbers.reduce(…)` → dnu~~ **blocker resolved** — `List#reduce` landed (same commit); `pending-retirement.md` L95 marks it "flips … move to `dispatch/`," but re-confirmed on disk the fixture **still physically sits** at `tests/lang/dispatch/pending/dispatch_rest_param.ph` — an un-executed tree-move, not a functional blocker | **blocker already gone; only a standing retirement-cleanup TODO remains, owned by whoever runs that sweep — not U-CORE-5** |
| `lexical/lexical_map_literal` / `_set_literal` / `_tuple_literal` | literal syntax + the class (`Map`/`Set`/`Tuple` are still ❌/❌ in catalog-delta §2.4 — confirmed "entire class" not yet started) | U-LEX + collections (each certified against **this** corpus) |

**Acceptance is therefore the new unit-local `collections/` corpus + the
`collections_contract.rs` gate passing against `List` today**, not any lexer- or
U-STD-gated fixture (the pending-retirement.md §4 rule for units whose capability
lands ahead of the surface syntax).

> **Re-grounding note.** The first two rows above turned out to be **already
> resolved by U-STD's `reduce`**, independent of this unit — they were never
> really "enabled by U-CORE-5," they were enabled by the pre-existing
> `each`/`reduce` combinators (§0.2). This doc's original framing ("U-CORE-5
> enables what U-STD then flips") slightly overstated the causal link for those
> two rows. The `lexical_*_literal` trio is the only row still genuinely gated
> on collections (`Map`/`Set`/`Tuple`) landing.

---

## §5. Must-not-preclude check (forward-compat.md)

U-CORE-5's applicable sections (forward-compat §5): **§4 (int/float split)** and
**§1 (concurrency / no global-stack assumption)**.

### 5.1 §4 — Integer/Float split (open-Q2, ADR-0005)

- **Equality delegates, never inspects representation.** `List#==` compares
  elements via `self.at(i) == other.at(i)` — an ordinary send to the *element's*
  `==`. It bakes in **no** `f64` assumption. When `Number` splits into
  `Integer`/`Float`, whatever `Number#==` decides for `2 == 2.0` (forward-compat §4
  passing constraint) is inherited transparently by every collection. ✅
- **Collection hash (H2) derives from element hashes.** The law mandates a
  hashable collection's hash be an *order-sensitive combine of `element.hash`* —
  **never** of element bits. So a future `Integer`/`Float` that hash `2` and `2.0`
  equal (forward-compat §4) keeps `Tuple(2)` and `Tuple(2.0)` hashing equal, with
  **no** change to the collection contract. ✅ (Specified here; realized when the
  hashable `Tuple` lands in U-STD.)

### 5.2 §1 — Concurrency / fiber-safe iteration (concurrency.md, ADR-0013)

- **L7 (stateless iteration)** is the explicit must-not-preclude guard. `each`
  holds its cursor in a **local** `var i` and calls `f.call(...)` — it stores **no**
  iteration position on the collection and assumes **no single global VM stack**.
  It is therefore reentrant (proved by `sequence_each_is_reentrant.ph`) and stays
  **fiber-local** when `Fiber` relocates the stack behind `current`
  (concurrency.md §1). The contract **forbids** any conforming collection from
  implementing iteration via a collection-global "current position" cursor — that
  would break both reentrancy and fiber-safety. ✅
- The harness and `.ph` bodies touch **no** `Value` `match` that would need a
  `Fiber` arm; `Value` stays open for extension (forward-compat §1 passing
  constraint). ✅

### 5.3 §2/§3 (errors, modules) — not tripped

- **Errors (§2):** U-CORE-5 raises nothing and forks no error channel. The H3
  enforcement it *defers* is explicitly routed to the **unified unwind** when
  U-CORE-6 lands (§2.4) — no second, parallel channel is introduced. ✅
- **Modules (§3):** no new global is added by name to a flat table; the only
  surface change (`List#==`/`!=`) reopens the existing `List` core-module row via
  `core.ph`, re-scopable by a future `import`. ✅

---

## §6. Open sub-decisions + traceability

### 6.1 Sub-decisions (recommendations given; confirm before implementing)

**SD-1 — Does U-CORE-5 add `List#==`/`!=`, or defer to U-STD?**
The task frames U-CORE-5 as "a contract, not new methods," yet Q5 *mandates*
structural `==` and the acceptance bar is "corpus green against `List` today"
(§1.3: `List` is identity-equal today).
- *Recommendation: **ADD them** (§3.2).* They are the **minimal** `.ph` that makes
  the reference implementation actually satisfy the contract this unit defines; a
  contract with no conformant reference is self-defeating. They are `.ph`-derivable
  (no floor amendment) and touch only the `List` reopen. Deferring would make the
  E-laws untestable against `List` today, gutting the central Q5 decision.

**SD-2 — Identity fast-path / `===` for reflexivity-under-`NaN` + cycle
termination?** `===` does not exist (§1.4, re-confirmed at HEAD `8733afe`).
Without it, `List#==` cannot short-circuit on identity, so (a) a `List`
containing `NaN` is non-reflexive (`[nan] == [nan]` element-compare is `false`),
and (b) a self-referential list (`l.add(l)`) recurses unboundedly.
- *Recommendation: **DEFER** — no floor amendment.* The corpus builds only
  well-behaved (finite, acyclic) element lists, so "green today" holds and E3 is
  satisfied for the tested domain. Document the `NaN`/cycle caveat (below).
  Revisit if/when `===` (an `Object` identity primitive — an ADR-0019 amendment,
  *not* a collection concern) or a `Number#==` `NaN`-normalization lands; that
  would add the identity fast-path `if (other === self) { return true }` and close
  both edges. This is a **correctness-edge**, not a speed item, but it is
  out-of-scope for a frozen-floor contract unit.

**SD-3 — Cross-kind equality guard: `isA(List)` vs exact class?** §2 E2 uses
`other.isA(List)`.
- *Recommendation: **`isA(List)`*** (Liskov-friendly: a `List` subclass compares
  structurally). Exact-class (`other.class == self.class`) is the stricter
  alternative; pick it only if a future subclass must be equality-incomparable with
  its base. Minor; `isA` is the sensible default.

### 6.2 Risks

- **Borrow model:** none new. The `.ph` `==`/`!=` run entirely as dispatched sends
  over the existing `Vec`-backed `List` (ADR-0009/0020, no `Rc`/`RefCell`); there
  is no new native code and thus no borrow-panic surface.
- **`NaN`/cycles (SD-2):** documented caveat; corpus avoids the domain. A `.ph`
  `==` on a cyclic list will stack-overflow rather than return — acceptable for now,
  fixed by SD-2's future identity fast-path.
- **`==`⊗`!=` decoupling (§1.3):** the load-bearing subtlety — forgetting `List#!=`
  leaves inequality identity-based and silently inconsistent. The
  `sequence_structural_equality.ph` golden asserts `a != c` explicitly to catch a
  missing/incorrect `!=` override.
- **Ordering dependency — SATISFIED.** U-CORE-1 has landed (`03764e3`); `isA` is
  confirmed present (`core.ph` L9). The historical risk ("implementing before
  U-CORE-1 lands `isA` makes the `==` body fail at dispatch") no longer applies;
  §0.3 still records it as the reason for the ordering.
- **Hashability is unenforced today (§2.4):** a reader may assume `List` is safely
  a `Map` key because it "has a hash" (inherited identity, now concretely landed
  via U-CORE-1). H1–H3 + R-INV-5.4 make the rejection a consumer obligation; the
  risk is a `Map`/`Set` author *forgetting* to reject mutable keys. Mitigation:
  R-INV-5.4 makes the `ContractSpec.hashable` flag the gate every collection is
  certified through.

### 6.3 Traceability

| Claim | Source |
|---|---|
| Contract = selectors + laws, not new classes | [ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md); [`decisions.md`](../../../spec/v0.2/core/decisions.md) Q5; catalog-delta §2.4 |
| Mutable by default; `==` structural; mutable⇒not-hashable | [`decisions.md`](../../../spec/v0.2/core/decisions.md) Q5 |
| `List` reference protocol (`size`/`at`/`add`/`each`) | `core.ph` **L143–160** (⚠ shifts with U-CORE-4, re-confirm at dispatch; was L75–94 pre-U-STD); floor-census §2.13/§3; `src/list.rs`; `src/primitive/list.rs` |
| `List` combinators (`map`/`filter`/`reduce`/`includes`/`isEmpty`/`at(_,put)`) already landed, pre-dating this unit | commits `5e2b395`/`176d454`; `core.ph` L162–211; `catalog-delta.md` §2.4 ("combinators landed"); `pending-retirement.md` §2/L95 |
| `List` equality is identity today (the gap) | `value.rs` **L245–272** ⚠; `primitive/object.rs` **L119–127** ⚠ (`object_eq`/`object_neq`) — both files concurrently edited by U-CORE-4, re-confirm at dispatch |
| `==`/`!=` install as `Method(1)`, not sacred | `universe.rs` **L257–258** ⚠ (concurrently edited by U-CORE-4, re-confirm at dispatch); floor-census §2.1/§5 |
| `at` out-of-range → `None` singleton | `primitive/list.rs` L72-79 (re-verified unchanged); invariants.rs (absence precedent) |
| `isA(_)` / `hash` are U-CORE-1 — **landed** (`03764e3`) | catalog-delta §4.5; [`decisions.md`](../../../spec/v0.2/core/decisions.md) Q1; invariant-requirements 1.2/1.3 |
| R-INV-5.1…5.4 all corpus | [`invariant-requirements.md`](../../../spec/v0.2/core/invariant-requirements.md) §4 U-CORE-5 |
| Flips nothing directly; enables U-STD/U-LEX fixtures (2 of 3 rows turned out already resolved, see §4.3 re-grounding note) | [`pending-retirement.md`](../../../spec/v0.2/core/pending-retirement.md) §4 |
| No floor amendment; census stays wherever U-CORE-4 lands it (85 today → 86 expected) | [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md); floor-census §1.1/§7 |
| int/float-safe by element delegation; fiber-safe iteration | [`forward-compat.md`](../../../spec/v0.2/core/forward-compat.md) §4, §1 |
| Harness surfaces mirror in-process + golden precedent | `tests/invariants.rs`; `tests/support/mod.rs`; `tests/lang.rs` |
