# Gating Decisions (U-CORE-0)

> **Status:** Normative rulings. Closes the questions that block the U-CORE
> implementation units — the requirements-analysis **Q1/Q2/Q4/Q5** and the two
> catalog↔code divergences **§4.1** (Method superclass) and **§4.4** (per-type
> `toString`). Each ruling names the owning unit and whether it needs an ADR.

> **Baseline:** HEAD `0f84232`; last code-affecting commit `0da64d6`. (Repinned
> 2026-07-12 to fold in U10/U-LEX/U-STD/U11 — none of these landings affect the
> Q1–Q5/§4.1/§4.4 rulings below; they were docs/`.ph`/compiler-only, no floor
> primitive added. See [`floor-census.md`](./floor-census.md) for the itemized
> re-baseline.)
>
> **Numbering caveat.** These **Q**s are the U-CORE *requirements-analysis*
> numbers (Q1 hash, Q2 errors, Q4 prelude, Q5 collections). They are **not**
> [`open-questions.md`](../open-questions.md)'s own Q1–Q14. Cross-references to
> that file use its numbers explicitly (e.g. "open-Q2 Int/Float", now closed by
> [ADR-0024](../../../adr/0024-numeric-surface-split-int-float-and-division.md);
> "open-Q8 imports", now closed by
> [ADR-0027](../../../adr/0027-modules-as-files-with-public-by-default-imports.md)).

## Summary

| Q / § | Question | Ruling | ADR action | Owner |
|---|---|---|---|:--:|
| **Q1** | Is `Object#hash` a floor primitive? | **Yes** — hash needs native access to representation/identity; underivable in `.ph`. | **ADR-0019 amendment — ratified as [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md) (Accepted)** | U-CORE-1 |
| **Q2** | Error mechanism | **Confirm [ADR-0008](../../../adr/0008-layered-exceptions-and-result.md)** — layered exceptions + `Result`, terminating, one unwind. Do not redesign. | none (note only) | U-CORE-6 |
| **Q4** | Prelude / global model | Kernel names are **the core module's exports, auto-imported** into every unit; user `import` deferred to the module unit. | none (ruling) | (module unit) |
| **Q5** | Collection mutability + equality | **Mutable by default**; sequence `==` is **structural**; **mutable collections are not hashable**, immutable ones are (Python-precedented). | none (contract in U-CORE-5) | U-CORE-5 |
| **§4.1** | `Method` superclass: catalog `<Function`, code `<Object` | **Re-parent code to `Method < Function`** (ADR-0006 is explicit). | none (fixes an ADR-0006 violation) | U-CORE-1/3 |
| **§4.4** | Per-type `toString` | **U-CORE-4** adds `toString` message overrides; keep the native `Value::to_string` print-path separate. | none | U-CORE-4 |

---

## Q1 — `Object#hash` is a floor primitive (ADR-0019 amendment)

**Question.** `Map`/`Set` need `key.hash` consistent with `==` (object-model §4).
Can `hash` be written in `.ph` over the existing floor, or must it be native?

**Finding — it must be native.** Every hashable kind needs data the floor does
**not** currently expose to `.ph`:

| Kind | What `hash` must read | Exposed to `.ph` today? |
|---|---|---|
| identity object | the `ObjRef`/handle integer | **No** — no primitive returns it |
| `Number` (`f64`) | an integer digest of the bits/value | **No** — no int type, no bit access |
| `String` | its bytes/codepoints | **No** — `String` floor is only `+`/`new` (no length/index) |
| `Symbol` | its interned id | **No** — only `toString`/`new` |

So `hash` fails the [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md)
§1 derivability test: it touches representation/identity below the `.ph`
boundary. Per ADR-0019, adding it is an **amendment via a new superseding ADR**,
not an ordinary commit.

**Ruling.** `Object#hash` (identity hash from the handle) joins the floor as a
universal `Object` primitive (object-model §8 already lists `hash` there), with
value-based overrides on the immediates (`Number`, `String`, `Symbol`, `Bool`)
also native. It lands with **U-CORE-1** (the reflection unit that owns the
universal `Object` protocol) and **blocks `Map`/`Set`** (U-STD), not U-CORE-1
itself.

**Ratified as [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)
(Accepted, 2026-07-12)** — the omnibus amendment covering this ruling plus the
U-CORE-3/4/6 amendments (README.md §"Cross-spec integration notes" note 2). Text:

> *Amends ADR-0019.* Add to the frozen floor: `Object#hash` (identity digest of
> the handle) and per-immediate `hash` overrides on `Number`/`String`/`Symbol`/`Bool`.
> Justification: hash reads representation/identity below the `.ph` boundary
> (see the table above); it is the `Map`/`Set` key precondition and cannot be
> derived. Constraint: `a == b ⇒ a.hash == b.hash` (invariant-requirements R-INV-1.3),
> and — per forward-compat §4 — `Number#hash` must digest the *mathematical value*
> so the Int/Float split (open-Q2, since decided by ADR-0024) keeps `2` and `2.0` hashing equal.
> Floor count moves **73 → 73 + N** where N = the installed hash bindings; update
> [`floor-census.md`](./floor-census.md) in the same change.

**Note.** ADR-0023 admits these primitives to the floor *in principle*; each is
still *installed* only when its owning unit (U-CORE-1 for `hash`/`Behavior`
reflection) actually lands and bumps the census.

## Q2 — Error mechanism: confirm ADR-0008, do not redesign

**Question.** `throw`/`try`/`catch` vs `Result`? Resumable or terminating?

**Ruling.** Already decided by **[ADR-0008](../../../adr/0008-layered-exceptions-and-result.md)**
(Accepted) — this unit **confirms**, it does not re-open:

- **Layer both.** `throw`/`Error` unwind for exceptional failure; `Result`/`Ok`/`Err`
  values for expected failure; bridges (`{…}.attempt()`, `.unwrap()`, `.okOr(_)`,
  `.ok()`) connect them.
- **Terminating, not resumable.** No Smalltalk `resume:`.
- **One unwind primitive.** `return`, `throw`, fiber `abort` are three payloads of
  the same stack unwind; `ensure`/`finally` fires on any unwind.
- **Handling is a `Block` protocol** (`blk.on(E){…}`, `blk.ensure{…}`);
  `try`/`catch`/`finally` is sugar.

**U-CORE-6 scope (minimal reification slice):** reify the `Error` root +
`MessageNotUnderstood`, give them `message`/`raise`, and wire the existing native
miss path (U8's dNU/`Message` reification) to **raise `MessageNotUnderstood`**
through the unified unwind. **Reserve** — do not implement — `Result`/`Ok`/`Err`
and the full `try`/`catch`/`on`/`ensure` block protocol; those are a later unit,
shaped to mirror `Option`/`Some`/`None`. If U-CORE-6 needs a native raise
primitive, that is a separate **ADR-0019 amendment** (cross-ref Q1's mechanism).
The [ADR-0008 amendment note](../../../forge/archive/phase2/PHASE2-INDEX.md) ("`MessageNotUnderstood`
= default-dNU raise") is folded in by U-CORE-6.

## Q4 — Prelude / global model

**Question.** How do kernel names (`Object`, `None`, `System`, …) become visible,
and is there a user-facing prelude?

**As-built.** `install_core` binds the class globals + the `None` **value** global
(values-and-absence §3.1); `core.ph` emits a `DefineGlobal` per class body. There
is one flat global namespace, populated at boot.

**Ruling.** Model the current global set as **"the core module's exports,
auto-imported into every compilation unit."** Concretely:

1. Kernel names live in the **core module** and are visible by default — this *is*
   the prelude; there is no separate, customizable prelude object in core scope.
2. A U-CORE unit that adds a surface name adds it to the core module (via
   `install_core` / `core.ph`), **not** to an ad-hoc global table keyed by raw
   string — so the `import` system (open-Q8, now decided by
   [ADR-0027](../../../adr/0027-modules-as-files-with-public-by-default-imports.md):
   file-as-module, public-by-default, qualified/selective/aliased) can re-scope
   or shadow it without a breaking change (forward-compat §3).
3. User-facing `import` semantics are now **decided** by ADR-0027 (open-Q8
   closed); their **implementation** remains out of core scope — deferred to the
   module unit. Core must not preclude them.

No ADR — this is a low-ceremony ruling that fixes the forward-compat §3 constraint.

## Q5 — Collection mutability + equality

**Question.** Are core collections mutable? Is `==` structural or identity? (The
central choice the U-CORE-5 *contract* encodes.)

**Ruling.**

- **Mutable by default.** `List` is already mutable (`rawPush`/`rawSet` over a
  native `Vec`, ADR-0020); the Smalltalk lineage and the native substrate both
  favor mutability. Immutability is **opt-in later** (a `freeze`/immutable view),
  not the default. Tuple is the fixed-arity immutable exception (object-model §4).
- **`==` is structural for sequences** — element-wise, order-sensitive, comparing
  with each element's own `==`. Two `List`s with equal elements in order are `==`.
  (Identity is still available via `===`/`Object` identity where the language
  provides it.)
- **Mutable collections are *not* hashable; immutable ones are.** This is the
  Python-precedented resolution of the mutable-key footgun (a mutable key whose
  hash changes corrupts the table): `List` (mutable) is **not** a valid `Map`/`Set`
  key; `Tuple` (immutable) **is**. This keeps R-INV-5.3 (`hash`↔`==` consistency,
  invariant-requirements) satisfiable without freezing all collections.

**U-CORE-5 encodes this as the shared collection-protocol *contract*** (not new
classes — ADR-0020): the selectors + laws (`size`/`at`/`each` totality,
deterministic iteration, structural `==`, hashability iff immutable) that `List`
already satisfies and that `Map`/`Set`/`Tuple`/`Range` (U-STD) must satisfy. No
ADR; the contract lives in the U-CORE-5 spec.

## §4.1 — `Method` superclass: re-parent to `Method < Function`

**Divergence.** Catalog (object-model §4) and [ADR-0006](../../../adr/0006-function-as-abstract-callable-root.md)
say **"`Block` and `Method` both inherit from `Function` as siblings."** The code
(`universe.rs` `make_core_class(heap, "Method", object_class, …)`) makes
**`Method < Object`** — a direct **ADR-0006 violation**, not a catalog error.

**Ruling.** **Re-parent the code to `Method < Function`.** ADR-0006 is Accepted
and explicit; the callable protocol (`call`/`arity`/`name`, and `bind(_)`
returning a callable — the `functions_method_*` pending fixtures) is exactly why
`Method` belongs under `Function`. Do **not** amend the catalog.

**Implementation note (load order).** `create_core_classes` currently allocates
`Method` **before** `Function`/`Block` (bootstrap-phases §2.1 step 5:
"`… Method, Function, Block(<Function) …`"). Re-parenting `Method < Function`
requires `Function` to exist first, so the fix **also moves `Method`'s
`make_core_class` after `Function`** in the load order. This preserves the
allocate-then-patch order and the parallel rule (verify with R-INV-0.2 / R-INV-3.1).
Owned by **U-CORE-1** (tower/reflection) or **U-CORE-3** (callables) — whichever
lands first should make the change; the other asserts it.

## §4.4 — Per-type `toString`

**Divergence (confirmed, not open).** `Object#toString` aliases `object_name`
(class name, ADR-0015), and no value type overrides it, so `42.toString` (the
*message*) yields `"Number"`. The value-rendering path is a **separate** native
method, `Value::to_string(vm)` (used by `System.print`), so `System.print(42)`
still shows `42`. The gap is the `toString` **message** only.

**Ruling.** **U-CORE-4 owns the per-type `toString` overrides** — `Number`,
`String`, `Symbol`, `Bool`, `Option` (`None`/`Some`). Keep the native
`Value::to_string` print-path **separate**; the invariant (R-INV-4.1) is that the
two **agree** for every value type, not that they merge. This is the unit that
flips `absence_option_none` / `absence_var_defaults_to_none` /
`binding_var_uninitialized` (pending-retirement §4). No ADR; it is scoped work
under an existing confirmed divergence. (Relatedly, DEFERRED F4 — the
`object_name`/instance-`toString` home — resolves here.)

## Traceability

| Ruling | Source |
|---|---|
| hash reads representation below `.ph` | [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md) §1; [`floor-census.md`](./floor-census.md) §2.5/§2.7 (String/Symbol floor) |
| `hash` on `Object` protocol; `Map`/`Set` use `hash`/`==` | [`object-model.md`](../object-model.md) §4, §8 |
| Error model | [ADR-0008](../../../adr/0008-layered-exceptions-and-result.md); [`error-handling.md`](../error-handling.md) |
| dNU/`Message` reification (U8) the raise wires to | [`floor-census.md`](./floor-census.md) §2.14; [`catalog-delta.md`](./catalog-delta.md) §2.7 |
| `import` semantics decided (ADR-0027), single global today | [ADR-0027](../../../adr/0027-modules-as-files-with-public-by-default-imports.md); [`open-questions.md`](../open-questions.md) §8; `vm.rs::install_core` |
| `List` mutable native `Vec` | [ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md); [`floor-census.md`](./floor-census.md) §2.13 |
| `Method`/`Block` siblings under `Function` | [ADR-0006](../../../adr/0006-function-as-abstract-callable-root.md); [`object-model.md`](../object-model.md) §4 |
| `Method` load order | [`bootstrap-phases.md`](./bootstrap-phases.md) §2.1 step 5 |
| `toString` message vs `Value::to_string` | [`catalog-delta.md`](./catalog-delta.md) §4.4; [ADR-0015](../../../adr/0015-object-default-tostring.md) |
