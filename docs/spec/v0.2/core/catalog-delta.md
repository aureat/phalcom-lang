# Core Class Catalog Delta (U-CORE-0)

> **Status:** Normative reconciliation. Maps every row of the aspirational
> catalog in [`../object-model.md`](../object-model.md) §4 against the
> **as-built** tree — the tower rows in `create_core_classes`, the globals in
> `install_core`, the floor primitives in [`floor-census.md`](./floor-census.md),
> and the `.ph` protocol in [`core.ph`](../../../../phalcom-core/core/core.ph).
> Its job is to say, per class, *what exists, what is missing, and who owns the
> gap.*

> **Baseline:** post-U-CORE-1. The authoritative pin + full landing history live
> in [`README.md`](./README.md) §"Baseline & drift policy"; this doc inherits
> them. Catalog-specific effects of those landings: **U-CORE-2** (Bool
> `Some`-lift + core `Option` combinators) is why §2.2/§4.2 below read as
> resolved; **U-STD** landed the remaining `Option`/`List` combinators
> (`map`/`flatMap`/`filter`/`ifSome`/`unwrapOr` on `Option`,
> `map`/`filter`/`reduce`/`includes`/`isEmpty` on `List`) in
> [`core.ph`](../../../../phalcom-core/core/core.ph) L77–107 / L149–186
> (§2.2/§2.4); **U-LEX** shipped `\(expr)` interpolation
> ([ADR-0022](../../../adr/0022-string-interpolation-backslash-paren-sigil.md),
> surface-only, no catalog rows change); **U11** added `True`/`False` (§2.2).

> ⚠️ **`implementation-status.md` is stale for this purpose.**
> [`../implementation-status.md`](../implementation-status.md) is Draft 0.1 and
> describes the *pre-U1* tree ("Wren/clox-style VM … no blocks"). The forge
> spine (U1–U7, U-LIST) has since landed selectors, blocks, operators-as-sends,
> `Option`, the metaclass-tower fix, `construct`, and native `List`. **This
> document supersedes that file's §4-relevant rows**; `implementation-status.md`
> should be re-baselined or deprecated (tracked as a follow-up).

## 1. Legend

| Column | Meaning |
|---|---|
| **Row** | A `ClassId` exists in `CoreClasses` (`universe.rs`) — the tower row is real. |
| **Global** | The name is bound as a surface global in `install_core`. |
| **Floor** | Native primitives installed (see [`floor-census.md`](./floor-census.md) §2). |
| **`.ph`** | Surface protocol defined in `core.ph`. |
| **Pending** | Catalog-specified protocol *not yet* implemented (the delta). |
| **Unit** | Proposed owning work unit. |

Status glyphs: ✅ present · ◐ partial · ❌ absent · ⚠️ catalog↔impl divergence (see §4).

## 2. Delta by catalog group

### 2.1 Kernel (the metaclass tower)

| Class | Row | Global | Floor | `.ph` | Pending (catalog − have) | Unit |
|---|:--:|:--:|---|---|---|---|
| `Object` | ✅ | ✅ | `==` `!=` `class` `class=` `name` `toString` `new` `perform` `respondsTo` `doesNotUnderstand` `hash` | `isA(_)` | — (U-CORE-1 landed: `hash` floor + `isA(_)` `.ph`) | **U-CORE-1 — landed** |
| `Behavior` | ✅ | ✅ | `superclass` `superclass=` | — | `name`, method-dictionary reflection, allocation protocol | U-CORE-1 |
| `Class` | ✅ | ✅ | `+(_)` `new()` | empty reopen | reflection surface (name/methods enumeration) | U-CORE-1 |
| `Metaclass` | ✅ | ✅ | *(inherited)* | empty reopen | — (structurally complete; verified by `verify_invariants`) | — |

`Object`/`Class`/`Metaclass` protocols are **◐ partial**: identity, class access,
default `toString`, and — since **U8** — the reflective/dispatch surface
(`perform`, `respondsTo`, `doesNotUnderstand`, plus the `Message` reification,
census §2.14) all exist. dNU is now an overridable hook, not a hard miss.
`hash` and `isA(_)` have **since landed with U-CORE-1** (`hash` native, `isA(_)`
derived in `core.ph`; see §4.5).

### 2.2 Primitives & singletons

| Class | Row | Global | Floor | `.ph` | Pending (catalog − have) | Unit |
|---|:--:|:--:|---|---|---|---|
| `Bool` | ✅ | ✅ | `and` `or` `not` `ifTrue` `ifFalse` `ifTrue(_, ifFalse)` `new` | empty reopen | — (§4.2 half-Option divergence resolved) | U-CORE-2 |
| `True` / `False` | ✅ | ✅ | — (own primitives: none) | empty reopen | — (concrete singleton subclasses of `Bool`, dispatch via inheritance) | **U11 — landed** |
| `Number` | ✅ | ✅ | `+ - * / %` `< <= > >=` `negated` `new` | empty reopen | ⚠️ numeric `toString` (today inherits `Object#toString` → class name, not value); `toNumber`, richer math | U-CORE-4 |
| `String` | ✅ | ✅ | `+(_)` `new` | empty reopen | length, indexing→`Option`, comparison, interpolation, `toSymbol`/`toNumber`, value `toString` | U-CORE-4 |
| `Symbol` | ✅ | ✅ | `toString` `new(_)` | empty reopen | `asString`/interning-identity protocol, `==` semantics | U-CORE-4 |
| `Option` | ✅ | ✅ | `match(some, none)` (on `Option`); `Some.new(_)` | `ifNone(_)` `orElse(_)` `isSome` `isNone` `map(_)` `flatMap(_)` `filter(_)` `ifSome(_)` `unwrapOr(_)` on `Option`; empty reopen of `Some` | — (§2.2 transform/extract combinators landed) | U-CORE-2 + **U-STD — landed** |

`Some` and `None` are **✅ complete** for the combinator surface: construction,
the `match` eliminator, the shared `None` singleton, the effect/query
combinators (`ifNone`, `orElse`, `isSome`, `isNone`), and the transform/extract
combinators (`ifSome`, `map`, `flatMap`, `filter`, `unwrapOr`) all exist —
U-STD landed the latter group in [`core.ph`](../../../../phalcom-core/core/core.ph)
L77–107. `None` is a value global, not a class global (values-and-absence.md
§3.1). There is **no `Nil`/`nil`** surface — forbidden by Invariant 4 (§4.3).

### 2.3 Callables & reflection

| Class | Row | Global | Floor | `.ph` | Pending (catalog − have) | Unit |
|---|:--:|:--:|---|---|---|---|
| `Function` | ✅ | ✅ | `arity` `name` `callWith(_)` `call()…call(_,_,_,_)` | — | (abstract root) higher-arity `call`, variadic `callWith` semantics | U-CORE-3 |
| `Block` | ✅ | ✅ | same as `Function` + `whileTrue(_)` | — | non-local-return surface protocol (mechanism exists, ADR-0013) | U-CORE-3 |
| `Method` | ✅ | ✅ | `new(_)` | *(inherits `Function` call protocol)* | `signature`, `holder`, `bind(_)` (re-parent `< Function` **landed** U-CORE-1, §4.1) | U-CORE-3 |

`Function`/`Block` are **✅ largely complete** for the call protocol (the U-CORE-0
floor already covers arities 0–4). `Method` is **◐ partial** and carries a
structural divergence.

### 2.4 Collections

| Class | Row | Global | Floor | `.ph` | Pending | Unit |
|---|:--:|:--:|---|---|---|---|
| `List` | ✅ | ✅ | `new` `rawLength` `rawAt` `rawSet` `rawPush` `toString` | `size` `at(_)` `add(_)` `each(_)` `at(_, put)` `map(_)` `filter(_)` `reduce(_,_)` `includes(_)` `isEmpty` | literal syntax `[a,b,c]` only (DEFERRED #6) | **U-STD — combinators landed**; literal syntax still open |
| `Tuple` | ❌ | ❌ | — | — | **entire class** | U-STD |
| `Map` | ❌ | ❌ | — | — | **entire class** (name reserved in `ClassName`) | U-STD |
| `Set` | ❌ | ❌ | — | — | **entire class** | U-STD |
| `Range` | ❌ | ❌ | — | — | **entire class** (name reserved) | U-STD |

Only `List` exists (ADR-0020). U-STD landed its combinator layer
(`map`/`filter`/`reduce`/`includes`/`isEmpty`, plus the `at(_,put:)` wrapper
over `rawSet`) in [`core.ph`](../../../../phalcom-core/core/core.ph) L149–186
— only list-**literal** syntax (`[a, b, c]`) remains deferred (needs an ADR +
parser work, DEFERRED #6), not part of U-STD's combinator scope. Per ADR-0020
each remaining collection is its own unit; **U-CORE-5's job is the shared
collection-protocol *contract*, not these classes.** `Map`/`Set` additionally
block on `Object#hash` (§4.5, Q1).

### 2.5 Runtime & namespaces

| Class | Row | Global | Floor | `.ph` | Pending | Unit |
|---|:--:|:--:|---|---|---|---|
| `Module` | ✅ | ✅ | `new()` | — | namespace/import surface protocol | (module unit, deferred) |
| `System` | ✅ | ✅ | `print(_)` `new()` | `static print()` shell | `clock`, `gc`, scheduler | (system unit, deferred) |

Both rows exist and back real runtime objects; only `print` of the `System`
surface is live. `clock`/`gc`/scheduler are out of core scope.

### 2.6 Concurrency — *out of core scope (forward-compat only)*

| Class | Row | Global | Pending | Unit |
|---|:--:|:--:|---|---|
| `Fiber` | ❌ | ❌ | entire class (name reserved in `ClassName`) | concurrency unit |
| `Future` | ❌ | ❌ | entire class (name reserved) | concurrency unit |

Non-goals for the core library. The requirement here is the **"must not
preclude"** clause (deliverable #7): the `Object`/`Module` layout and the
callable protocol must leave room for these without a breaking change.

### 2.7 Errors — *blocked on the mechanism decision*

| Class | Row | Global | Pending | Unit |
|---|:--:|:--:|---|---|
| `Error` | ❌ | ❌ | root raisable class; `message`, `raise` | U-CORE-6 |
| `MessageNotUnderstood` | ❌ | ❌ | raised by default dNU | U-CORE-6 |
| `DeadFrameError` | ❌ | ❌ | non-local return to dead frame (blocks.md) | U-CORE-6 |
| `TypeError` / `ArgumentError` / `RangeError` | ❌ | ❌ | typed error subclasses | U-CORE-6 |

None are **surface** classes yet. The *mechanism* exists natively as the
`RuntimeError` enum (`error.rs`: `Type`, etc.) but is **not reified** into a
Phalcom `Error` hierarchy. Since **U8** the `doesNotUnderstand(_)` *hook* exists
(`object_does_not_understand`) and the miss path reifies a `Message` (census
§2.14); what is still missing is the surface `MessageNotUnderstood` **class** the
hook should raise. The reify-vs-native split, and exceptions-vs-`Result`, are
governed by **[ADR-0008](../../../adr/0008-layered-exceptions-and-result.md)**
(layered exceptions + Result) — so requirements-analysis **Q2 is partially
pre-decided**; U-CORE-6 must be read against ADR-0008, not designed from scratch.

## 3. Rollup

| Catalog group | Rows exist | Globals | Fully ✅ | Partial ◐ | Absent ❌ |
|---|:--:|:--:|:--:|:--:|:--:|
| Kernel tower | 4 / 4 | 4 / 4 | 1 (`Metaclass`) | 3 | 0 |
| Primitives & singletons | 6 / 6 | 6 / 6 | 2 (`True`/`False`, `Option`) | 4 | 0 |
| Callables & reflection | 3 / 3 | 3 / 3 | 2 | 1 | 0 |
| Collections | 1 / 5 | 1 / 5 | 0 | 1 (`List`) | 4 |
| Runtime & namespaces | 2 / 2 | 2 / 2 | 0 | 2 | 0 |
| Concurrency | 0 / 2 | 0 / 2 | 0 | 0 | 2 |
| Errors | 0 / 6 | 0 / 6 | 0 | 0 | 6 |
| **Total** | **16 / 28** | **16 / 28** | **5** | **11** | **12** |

Plus two **impl-only** rows absent from the `object-model.md` §4 catalog:
**`Nil`** — exists in the tower to back `Value::Nil.class`, bound to no global,
no primitives (§4.3); the catalog correctly omits it — and **`Message`** — the
U8 reified miss-send (a real global with four accessor primitives, census
§2.14), which is simply catalogued in `messages-and-selectors.md` §5 rather than
the object-model core catalog.

**Reading:** the tower and value/callable spine are *present but thin* (11 of 16
existing rows are partial protocol). The genuine greenfield is **collections
beyond `List`, all of errors, and all of concurrency** — 12 absent rows, of
which errors are core (U-CORE-6) and the rest are U-STD/deferred.

**Recomputed 2026-07-12 (was "not yet recomputed" against the pre-U-STD/U11
baseline):** U-STD moved `Option` from ◐ to ✅ (§2.2 — `map`/`flatMap`/`filter`/
`ifSome`/`unwrapOr` landed, plus the U-CORE-2 half-Option fix) and landed
`List`'s combinator layer (still ◐ — literal syntax remains open, §2.4). U11
adds `True`/`False` as a **fully-✅** row under Primitives & singletons (§2.2,
ADR-0004 — no pending protocol; `Bool` itself stays ◐ pending `hash`/richer
protocol), growing that group 5/5 → 6/6 rows and the catalog total 27 → 28
(U11's landing makes `True`/`False` a real, distinct tower row, +1 net vs. the
prior count). Fully-✅ count moves 3 → 5 (`Metaclass`, `True`/`False`, `Option`,
plus the pre-existing 2 in Callables & reflection unchanged); Partial moves
12 → 11.

## 4. Catalog ↔ implementation divergences (decisions required)

These are places the catalog and the code disagree, or where a catalog claim is
unverified. Each needs a ruling before the owning unit proceeds.

### 4.1 ✅ `Method` superclass: catalog says `Function`, code said `Object` — **re-parented (U-CORE-1, landed)** ([`decisions.md`](./decisions.md) §4.1)
- **Catalog** (§4 Callables): `Method | Function` — "Sibling of `Block`, not a subtype of it."
- **Was:** `make_core_class(heap, "Method", object_class, …)` → `Method < Object` — an ADR-0006 violation.
- **As-built (resolved):** **U-CORE-1 re-parented the code to `Method < Function`** (`universe.rs`, in `create_core_classes`, with the load-order fix that allocates `Method` *after* `Function`). [ADR-0006](../../../adr/0006-function-as-abstract-callable-root.md) (Accepted) is explicit that `Block`/`Method` are siblings under `Function`; the catalog was correct and was **not** amended. The re-parent preserves the allocate-then-patch order and the parallel rule (R-INV-0.2 / R-INV-3.1).

### 4.2 ✅ `Bool#ifTrue`/`ifFalse` return a half-Option — **resolved in U-CORE-2**
- **Catalog / spec:** `ifTrue`/`ifFalse` return `Option`. Ratified twice — `object-model.md` §4 and `control-flow.md` §1, whose `if/else === c.ifTrue { A }.ifNone { B }` desugaring only composes if `ifTrue` yields an `Option` that `.ifNone` (an `Option` method) can be sent to.
- **Code** (`boolean.rs` L109–130): the *absent* arm is `None` ✅, but the *present* arm returns the block result **raw**, never `Some(_)`:

  | receiver | `ifTrue { A }` — current | required |
  |---|---|---|
  | `true`  | `A` (unwrapped) | `Some(A)` |
  | `false` | `None` ✅ | `None` ✅ |

  So the result was `A ∪ None`, **not** a well-formed `Option`. This was a deliberate U5 deferral (`U5-plan.md` §4.1, "independent of U6's `Option`"), latent only because the `Option` combinators (`ifNone`/`orElse`/…) didn't exist yet (§2.2). It would have broken the moment they landed: `c.ifTrue { 42 }.ifNone { 0 }` would have sent `ifNone` to a raw `Number`.
- **Resolution (U-CORE-2, landed):** `Some`-lifted the *one-armed* `ifTrue`/`ifFalse` taken arm (`primitive/boolean.rs`, `primitive/nil.rs`'s new `wrap_some` helper); the untaken arm's `None` was already correct. The paired `ifTrue(_, ifFalse)` (what `if/else` desugars to) and `and`/`or` still return raw values — those are correct as-is and untouched. The sacred inliner ([ADR-0018](../../../adr/0018-sacred-selector-inliner-and-override-guard.md)) `Some`-lifts in lockstep via a new `Bytecode::WrapSome` opcode so the fast path ≡ the deopt path; the wrap is elided in statement (pop) context to avoid the allocation when the result is discarded unread. `core.ph`'s `Option` reopen gained the four combinators the catalog assigned to this unit — `ifNone(_)`, `orElse(_)`, `isSome`, `isNone`, all derived over `match` — so `control-flow.md` §1's `if/else === c.ifTrue { A }.ifNone { B }` desugaring is now executable end to end (see the ADR-0018 amendment and the `control-flow`/`absence` golden corpus).

### 4.3 `Nil` row present in impl, absent in catalog — **intentional, keep**
- Not a defect: the catalog deliberately omits `Nil` (Invariant 4). The internal row is required to answer `Value::Nil.class`. Documented here so a future reader does not "fix" the catalog by adding it.

### 4.4 ⚠️ `Number#toString` inherits the class-name default — **confirmed**
- `Object#toString` aliases `object_name` (class name), and `Number` registers no
  override, so `42.toString` (the *message*) yields `"Number"`, not `"42"`. The
  value-rendering path is a **separate** Rust method, `Value::to_string(vm)` (used
  by `System.print`, seen at `boolean.rs` L32/34), so `System.print(42)` still
  shows `42` — the gap is specifically the `toString` **message**. U-CORE-4 must
  override `toString` on `Number` (and `String`, `Symbol`, `Bool`, `Option`).

### 4.5 `hash` / `isA` landed (U-CORE-1); dNU / `perform` / `respondsTo` / `Message` landed in U8
- **Update:** `doesNotUnderstand(_)`, `perform`, `respondsTo`, and the `Message`
  reification landed in **U8** — a missed send now forwards to an overridable dNU
  hook, not a hard error (census §2.1/§2.14). What remains is the surface
  `MessageNotUnderstood` **class** (§2.7, U-CORE-6) the hook should raise.
- `Object#hash` is **present as of U-CORE-1** — a native floor primitive
  admitted by [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md),
  with per-immediate overrides on `Number`/`String`/`Bool`/`Symbol` (census
  §2.1/§2.4–2.7). `isA(_)` also landed, derived in `core.ph`. `Map`/`Set`
  (§2.4) can now build on `hash`; requirements-analysis **Q1** is closed.

## 5. Traceability

| Claim | Source |
|---|---|
| Catalog rows | `object-model.md` §4 L86–166 |
| Tower rows / superclasses | `universe.rs::create_core_classes` L90–185 |
| Globals | `vm.rs::install_core` L307–362 |
| Floor primitives | [`floor-census.md`](./floor-census.md) §2 |
| `.ph` protocol | `core.ph` L1–79 |
| Reserved-but-unbuilt names | `primitive/mod.rs::ClassName` (`Range`,`Map`,`Fiber`,`Future`) |
| Error mechanism | [ADR-0008](../../../adr/0008-layered-exceptions-and-result.md); `error.rs::RuntimeError` |
| Staleness of status doc | `implementation-status.md` L5–9 (self-describes pre-U1 state) |
