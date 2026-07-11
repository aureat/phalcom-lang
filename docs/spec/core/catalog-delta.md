# Core Class Catalog Delta (U-CORE-0)

> **Status:** Normative reconciliation. Maps every row of the aspirational
> catalog in [`../object-model.md`](../object-model.md) §4 against the
> **as-built** tree — the tower rows in `create_core_classes`, the globals in
> `install_core`, the floor primitives in [`floor-census.md`](./floor-census.md),
> and the `.ph` protocol in [`core.ph`](../../../phalcom-core/core/core.ph).
> Its job is to say, per class, *what exists, what is missing, and who owns the
> gap.*

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
| `Object` | ✅ | ✅ | `==` `!=` `class` `class=` `name` `toString` `new` | empty reopen | `isA(_)`, `hash`, `perform(_,_)`, `respondsTo(_)`, `doesNotUnderstand(_)` | U-CORE-1 |
| `Behavior` | ✅ | ✅ | `superclass` `superclass=` | — | `name`, method-dictionary reflection, allocation protocol | U-CORE-1 |
| `Class` | ✅ | ✅ | `+(_)` `new()` | empty reopen | reflection surface (name/methods enumeration) | U-CORE-1 |
| `Metaclass` | ✅ | ✅ | *(inherited)* | empty reopen | — (structurally complete; verified by `verify_invariants`) | — |

`Object`/`Class`/`Metaclass` protocols are **◐ partial**: identity, class access
and default `toString` exist; the reflective/dispatch surface (`hash`, `isA`,
`perform`, `respondsTo`, `doesNotUnderstand`) is the U-CORE-1 body of work. dNU
in particular is still a hard miss, not a hook (see §4.5).

### 2.2 Primitives & singletons

| Class | Row | Global | Floor | `.ph` | Pending (catalog − have) | Unit |
|---|:--:|:--:|---|---|---|---|
| `Bool` | ✅ | ✅ | `and` `or` `not` `ifTrue` `ifFalse` `ifTrue(_, ifFalse)` `new` | empty reopen | ⚠️ confirm `ifTrue`/`ifFalse` **return `Option`** per catalog | U-CORE-2 |
| `Number` | ✅ | ✅ | `+ - * / %` `< <= > >=` `negated` `new` | empty reopen | ⚠️ numeric `toString` (today inherits `Object#toString` → class name, not value); `toNumber`, richer math | U-CORE-4 |
| `String` | ✅ | ✅ | `+(_)` `new` | empty reopen | length, indexing→`Option`, comparison, interpolation, `toSymbol`/`toNumber`, value `toString` | U-CORE-4 |
| `Symbol` | ✅ | ✅ | `toString` `new(_)` | empty reopen | `asString`/interning-identity protocol, `==` semantics | U-CORE-4 |
| `Option` | ✅ | ✅ | `match(some, none)` (on `Option`); `Some.new(_)` | empty reopens of `Option`,`Some` | `ifSome(_)`, `ifNone(_)`, `map(_)`, `orElse(_)`, `unwrapOr(_)`, `isSome`, `isNone` — all derivable over `match` | U-CORE-2 |

`Some` and `None` are **◐ partial**: construction + the `match` eliminator +
the shared `None` singleton exist; every combinator is deferred. `None` is a
value global, not a class global (values-and-absence.md §3.1). There is **no
`Nil`/`nil`** surface — forbidden by Invariant 4 (§4.3).

### 2.3 Callables & reflection

| Class | Row | Global | Floor | `.ph` | Pending (catalog − have) | Unit |
|---|:--:|:--:|---|---|---|---|
| `Function` | ✅ | ✅ | `arity` `name` `callWith(_)` `call()…call(_,_,_,_)` | — | (abstract root) higher-arity `call`, variadic `callWith` semantics | U-CORE-3 |
| `Block` | ✅ | ✅ | same as `Function` + `whileTrue(_)` | — | non-local-return surface protocol (mechanism exists, ADR-0013) | U-CORE-3 |
| `Method` | ✅ | ✅ | `new(_)` | — | ⚠️ **superclass mismatch** (§4.1); `signature`, `holder`, `bind(_)` | U-CORE-1/3 |

`Function`/`Block` are **✅ largely complete** for the call protocol (the U-CORE-0
floor already covers arities 0–4). `Method` is **◐ partial** and carries a
structural divergence.

### 2.4 Collections

| Class | Row | Global | Floor | `.ph` | Pending | Unit |
|---|:--:|:--:|---|---|---|---|
| `List` | ✅ | ✅ | `new` `rawLength` `rawAt` `rawSet` `rawPush` `toString` | `size` `at(_)` `add(_)` `each(_)` | `at(_, put)` (wrap `rawSet`), `map`/`reduce`/`filter`/`includes`/`isEmpty`, literal syntax | U-STD |
| `Tuple` | ❌ | ❌ | — | — | **entire class** | U-STD |
| `Map` | ❌ | ❌ | — | — | **entire class** (name reserved in `ClassName`) | U-STD |
| `Set` | ❌ | ❌ | — | — | **entire class** | U-STD |
| `Range` | ❌ | ❌ | — | — | **entire class** (name reserved) | U-STD |

Only `List` exists (ADR-0020). Per ADR-0020 each remaining collection is its own
unit; **U-CORE-5's job is the shared collection-protocol *contract*, not these
classes.** `Map`/`Set` additionally block on `Object#hash` (§4.5, Q1).

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
Phalcom `Error` hierarchy. The reify-vs-native split, and exceptions-vs-`Result`,
are governed by **[ADR-0008](../../adr/0008-layered-exceptions-and-result.md)**
(layered exceptions + Result) — so requirements-analysis **Q2 is partially
pre-decided**; U-CORE-6 must be read against ADR-0008, not designed from scratch.

## 3. Rollup

| Catalog group | Rows exist | Globals | Fully ✅ | Partial ◐ | Absent ❌ |
|---|:--:|:--:|:--:|:--:|:--:|
| Kernel tower | 4 / 4 | 4 / 4 | 1 (`Metaclass`) | 3 | 0 |
| Primitives & singletons | 5 / 5 | 5 / 5 | 0 | 5 | 0 |
| Callables & reflection | 3 / 3 | 3 / 3 | 2 | 1 | 0 |
| Collections | 1 / 5 | 1 / 5 | 0 | 1 (`List`) | 4 |
| Runtime & namespaces | 2 / 2 | 2 / 2 | 0 | 2 | 0 |
| Concurrency | 0 / 2 | 0 / 2 | 0 | 0 | 2 |
| Errors | 0 / 6 | 0 / 6 | 0 | 0 | 6 |
| **Total** | **15 / 27** | **15 / 27** | **3** | **12** | **12** |

Plus one **impl-only** row absent from the catalog: **`Nil`** — exists in the
tower to back `Value::Nil.class`, bound to no global, carries no primitives
(§4.3). It is correct that the catalog omits it.

**Reading:** the tower and value/callable spine are *present but thin* (12 of 15
existing rows are partial protocol). The genuine greenfield is **collections
beyond `List`, all of errors, and all of concurrency** — 12 absent rows, of
which errors are core (U-CORE-6) and the rest are U-STD/deferred.

## 4. Catalog ↔ implementation divergences (decisions required)

These are places the catalog and the code disagree, or where a catalog claim is
unverified. Each needs a ruling before the owning unit proceeds.

### 4.1 ⚠️ `Method` superclass: catalog says `Function`, code says `Object`
- **Catalog** (§4 Callables): `Method | Function` — "Sibling of `Block`, not a subtype of it."
- **Code:** `make_core_class(heap, "Method", object_class, …)` → `Method < Object` (`universe.rs` L133).
- **Decision:** re-parent `Method` under `Function` (aligns the catalog; `Method` gains the `call` protocol as a sibling of `Block`), **or** amend the catalog to `Method < Object`. Touches U-CORE-1/3 and the ADR-0006 callable-root story.

### 4.2 ⚠️ `Bool#ifTrue`/`ifFalse` return type
- **Catalog:** "`ifTrue`/`ifFalse` return `Option`."
- **Code:** native `bool_if_true`/`bool_if_false` — return type unverified against the `Option` claim in this pass.
- **Decision:** confirm the primitives return `Some(_)`/`None` (not raw values / sentinel), or reconcile the catalog. Load-bearing for no-truthiness composition (U-CORE-2).

### 4.3 `Nil` row present in impl, absent in catalog — **intentional, keep**
- Not a defect: the catalog deliberately omits `Nil` (Invariant 4). The internal row is required to answer `Value::Nil.class`. Documented here so a future reader does not "fix" the catalog by adding it.

### 4.4 ⚠️ `Number#toString` inherits the class-name default
- `Object#toString` aliases `object_name` (class name). A `Number` therefore
  stringifies as `"Number"`, not its value — almost certainly wrong for the
  catalog's "Arithmetic, comparison, `toString`." U-CORE-4 must override
  `toString` on `Number` (and `String`, `Symbol`, `Bool`, `Option`).

### 4.5 dNU / `hash` are absent — gate several rows
- `doesNotUnderstand(_)` is not installed; a missed send is a hard error, and
  `MessageNotUnderstood` (§2.7) cannot be raised until the hook exists.
- `Object#hash` is absent; `Map`/`Set` (§2.4) block on it. Whether `hash` is a
  floor primitive is requirements-analysis **Q1** (an ADR-0019 amendment if yes).

## 5. Traceability

| Claim | Source |
|---|---|
| Catalog rows | `object-model.md` §4 L86–166 |
| Tower rows / superclasses | `universe.rs::create_core_classes` L90–185 |
| Globals | `vm.rs::install_core` L307–362 |
| Floor primitives | [`floor-census.md`](./floor-census.md) §2 |
| `.ph` protocol | `core.ph` L1–79 |
| Reserved-but-unbuilt names | `primitive/mod.rs::ClassName` (`Range`,`Map`,`Fiber`,`Future`) |
| Error mechanism | [ADR-0008](../../adr/0008-layered-exceptions-and-result.md); `error.rs::RuntimeError` |
| Staleness of status doc | `implementation-status.md` L5–9 (self-describes pre-U1 state) |
