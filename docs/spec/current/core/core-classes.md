# Core Classes Reference (`docs/spec/current/core/`)

> **Status:** Normative, as-built. The single **class-by-class** definition of
> Phalcom's core library — each kernel class's *role, structure, interface
> (methods), architecture, and status*. Where the sibling docs slice the same
> ground by a different axis, this one slices it **by class**:
>
> | Doc | Axis | Answers |
> |---|---|---|
> | [`../object-model.md`](../object-model.md) §4 | *target* catalog | what each class **should** be |
> | [`catalog-delta.md`](./catalog-delta.md) | *delta* (catalog − built) | what is **missing**, and who owns the gap |
> | [`floor-census.md`](./floor-census.md) | *primitive* | which native `(class, selector)` bindings exist |
> | **this file** | *class* | for each class: role · interface · native/`.ph` split · ADR · status |
>
> It is derived from ground-truth source — the tower rows in
> `universe.rs::create_core_classes`, the globals in `vm.rs::install_core`, the
> floor in [`floor-census.md`](./floor-census.md), and the `.ph` surface in
> [`core.ph`](../../../../phalcom-core/core/core.ph) — not aspiration. When code and
> the *target* catalog disagree, the divergence is called out inline and cross-linked
> to [`catalog-delta.md`](./catalog-delta.md) §4.

> **Baseline:** post-U-CORE-1. The authoritative pin + full landing history live in
> [`README.md`](./README.md) §"Baseline & drift policy"; this doc inherits them and
> carries only this one-line back-reference. When a forge unit lands new protocol,
> re-baseline here in lockstep with the census and catalog.

---

## 1. How to read an entry

Each class entry gives:

- **Superclass · Kind · Representation.** Kind — **A**bstract (defines protocol,
  never the direct class of a live value) · **I**mmediate (values live in a
  non-`Instance` VM representation) · **U** ordinary heap instance. Representation
  is the `Value`/`Object` arm that backs it ([ADR-0010](../../../adr/0010-tagged-value-enum.md)).
- **Interface.** The selectors the class *carries* (own dictionary), split into
  **floor** (native Rust, [floor-census](./floor-census.md)) and **`.ph`**
  (self-hosted in [`core.ph`](../../../../phalcom-core/core/core.ph) over the floor).
  Inherited protocol is noted, not repeated.
- **Architecture.** The governing ADR(s) and *why* the native/`.ph` line falls
  where it does (the [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md)
  derivability test).
- **Status.** ✅ landed · ◐ partial · ❌ absent, plus the owning unit for any gap.

**Selector notation.** Human-facing per [floor-census §1.2](./floor-census.md): a
getter is a bare name (`size`), a setter `name=(_)`, an arity-*n* method
`name(_, …)`, labeled args named (`match(some, none)`). The interner currently
emits the colon form (`match(some:none:)`); migrating it to the canonical comma
form is [U-CORE-4](../../../forge/units/U-CORE-4/as-built.md) (BD-CORE4-2).

**Two invariants frame every entry:** *everything is an object* (even `true`,
`42`, a class, a method) and *message-send is the only computational primitive*
([object-model §1](../object-model.md)). Absence is **never** a surface `nil` — it
is `Option` ([ADR-0007](../../../adr/0007-option-as-abstract-with-some-none.md),
[ADR-0021](../../../adr/0021-no-truthiness-enforcement.md), Invariant 4).

---

## 2. Structure — the kernel (metaclass) tower

Four classes form the self-describing spine; every other core class is an ordinary
row hung off it. The shape is fixed by
[ADR-0002](../../../adr/0002-metaclass-tower-parallel-rule.md) (parallel rule) and
[ADR-0003](../../../adr/0003-introduce-behavior-kernel-class.md) (`Behavior`).

```
Object ──superclass──▶ (none, root)
  ▲
  ├── Behavior          shared home for anything that HAS instances
  │     ├── Class       the class of every named class
  │     └── Metaclass   the class of every metaclass
  │
  ├── Bool (A) ── True, False          ├── Function (A) ── Block, Method
  ├── Number (A) ── Int, Float         ├── Option (A) ── Some, None
  ├── String, Symbol                   ├── List, (Tuple/Map/Set/Range — future)
  ├── Module, System                   └── Error ── MessageNotUnderstood, … (future)
  └── Nil (internal, no global)        + Message (U8 reified miss-send)
```

**The parallel rule.** Every class `X` has exactly one metaclass `X class`, created
with it, and the metaclass hierarchy *parallels* the class hierarchy:

```
(X class).superclass == (X.superclass) class      anchored by   (Object class).superclass == Class
```

This is what makes `static`/`construct` methods inherit uniformly (no class is
special-cased to lack a metaclass). The apex closes with `Metaclass.class ==
Metaclass class` and `(Metaclass class).class == Metaclass`. The exact
apex relationships and the boot wiring order are in
[object-model §5–§6](../object-model.md#5-the-metaclass-tower); the boot check that
asserts them is `Universe::verify_invariants` (`universe.rs`), extended by
**U-CORE-1** from a `Number`-only check to all ordinary rows (R-INV-0.2).

---

## 3. Kernel tower classes

### `Object` — the root

| | |
|---|---|
| Superclass · Kind | *(none)* · U (root) |
| Representation | every value answers it; `x.class` is total |
| Status | ◐ partial (universal protocol landed through U-CORE-1; `toString` still class-name default until U-CORE-4) |

**Interface — floor** ([census §2.1](./floor-census.md)): `class` · `class=(_)` ·
`==(_)` · `!=(_)` (identity by default; value types override) · `name` ·
`toString` (aliases `name`, [ADR-0015](../../../adr/0015-object-default-tostring.md)) ·
`hash` (identity digest of the heap handle — **landed U-CORE-1**,
[ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)) ·
`perform(_)` / `perform(_, _)` · `respondsTo(_)` (pure probe) ·
`doesNotUnderstand(_)` (overridable miss hook — **U8**) · `new()` (static
`object_class_new`, the default allocator — see §9.2).

**Interface — `.ph`**: `isA(_)` — **landed U-CORE-1**, derived purely over
`class`/`==`/`superclass` ([`core.ph`](../../../../phalcom-core/core/core.ph) L9). It
is the clean win the roadmap expected to be native but is ordinary `.ph`.

**Architecture.** `hash`/`name`/`methods` are native because they read
representation below the `.ph` boundary (the `ObjRef` handle, the class's own
name, the method map); `isA(_)` is `.ph` because its three ingredients are already
floor. That split is the spine of the whole library
([U-CORE-1 §2](../../../forge/units/U-CORE-1/as-built.md)).

**Planned.** `methodFor(_)` (reflective method reification) — U-CORE-3.
Surface `MessageNotUnderstood` for dNU to raise — U-CORE-6.

---

### `Behavior` — shared class-side protocol

| | |
|---|---|
| Superclass · Kind | `Object` · A |
| Role | the home of everything that *has instances*: method dictionary, `superclass`, `name`, allocation, reflection. Superclass of `Class` **and** `Metaclass`, so both inherit it ([ADR-0003](../../../adr/0003-introduce-behavior-kernel-class.md)). |
| Status | ◐ partial |

**Interface — floor** ([census §2.2](./floor-census.md)): `superclass` ·
`superclass=(_)` · `name` (the receiver class's **own** name — **shadows**
`Object#name` for class receivers, **landed U-CORE-1**) · `methods` (own method-dict
selector `Symbol`s as a fresh `List` — **landed U-CORE-1**).

**Architecture.** `Behavior#name` shadows `Object#name` *only for class receivers*:
a class `C` is an instance of its metaclass, so `C.name` walks the metaclass chain
`C class → … → Behavior → Object`, and `Behavior#name` sits below `Object#name`. A
non-class receiver has `Behavior` nowhere in its chain, so `3.name` still resolves
to `Object#name` → `"Number"` ([U-CORE-1 §3.2](../../../forge/units/U-CORE-1/as-built.md)).

**Planned.** Inherited/`allMethods` reflection, `includesSelector`, instance-var
reflection — U-STD (derivable over `methods` + `superclass`). Full allocation /
`construct` machinery — [classes.md](../classes.md) / U7.

---

### `Class` — the instantiation apex

| | |
|---|---|
| Superclass · Kind | `Behavior` · U |
| Role | the class of every *named* class |
| Status | ◐ partial (empty `.ph` reopen; reflection surface pending) |

**Interface — floor** ([census §2.3](./floor-census.md)): `+(_)` (`class_add`) ·
`new()` (`class_new`, the deeper allocator fallback — see §9.2).

---

### `Metaclass` — the class of metaclasses

| | |
|---|---|
| Superclass · Kind | `Behavior` · U |
| Role | each metaclass has exactly one instance (its class); `(X class).class == Metaclass` |
| Status | ✅ structurally complete — verified by `verify_invariants`; carries no own floor primitive |

---

## 4. Primitives & singletons

### `Bool` / `True` / `False` — booleans without truthiness

| | |
|---|---|
| `Bool` | `Object` · **A** · holds the protocol; no value is directly of class `Bool` |
| `True` / `False` | `Bool` · **I** · concrete singleton subclasses; `true.class == True` — **landed U11** |
| Status | ✅ (`True`/`False` fully landed; `Bool` ◐ pending `hash`-consumers/richer protocol) |

**Interface — floor on `Bool`** ([census §2.6](./floor-census.md), inherited by
`True`/`False`): `not()` · `and(_)` · `or(_)` · `ifTrue(_)` · `ifFalse(_)` ·
`ifTrue(_, ifFalse)` · `hash` (1/0 — **landed U-CORE-1**) · static `new()`/`new(_)`.
`True`/`False` carry **zero** own bindings — all behaviour is reached by
inheritance; their `.ph` bodies are empty.

**Architecture** ([ADR-0004](../../../adr/0004-boolean-as-abstract-bool-with-true-false.md),
[ADR-0021](../../../adr/0021-no-truthiness-enforcement.md)). The six control
selectors are **sacred** (§9.3): the compiler inlines their literal-block call
sites and deopts to exactly these sends on override/mismatch. No implicit coercion —
they dispatch on real `True`/`False` receivers only. **`ifTrue`/`ifFalse` return an
`Option`** (`Some(result)` on the taken arm, `None` on the untaken) so
`control-flow.md`'s `if/else === c.ifTrue { A }.ifNone { B }` desugaring composes —
the one-armed `Some`-lift **landed U-CORE-2** via a `Bytecode::WrapSome` that the
sacred inliner mirrors (elided in statement position). The paired
`ifTrue(_, ifFalse)` and `and`/`or` return raw values, which is correct.

---

### `Number` (`Int` / `Float`) — arithmetic

| | |
|---|---|
| Target | `Number` (`Object`, **A**) with immediate subclasses `Int` (exact, unbounded) and `Float` (`f64`) — [ADR-0024](../../../adr/0024-numeric-surface-split-int-float-and-division.md) |
| As-built | a single **flat `f64` `Number`** ([ADR-0005](../../../adr/0005-number-as-flat-f64.md)); the `Int`/`Float` split is **surface-normative, not yet built** (substrate is future work, [deferred-work.md §3](../deferred-work.md)) |
| Status | ◐ partial |

**Interface — floor** ([census §2.4](./floor-census.md)): `+(_)` `-(_)` `*(_)`
`/(_)` `%(_)` · `<(_)` `<=(_)` `>(_)` `>=(_)` · `negated()` · `hash` (digest of the
**mathematical value**, class-agnostic, so a future `Int 2` and `Float 2.0` hash
equal — **landed U-CORE-1**) · static `new()`/`new(_)`. Arithmetic operators are
**ordinary sends**, never opcodes ([control-flow.md](../control-flow.md) §1).

**Divergence ([catalog-delta §4.4](./catalog-delta.md)).** `Number#toString` (the
*message*) inherits `Object#toString` → the class name `"Number"`, **not** `"42"`.
The value-rendering path is the *separate* native `Value::to_string`, so
`System.print(42)` still shows `42`. **U-CORE-4** owns the per-type `toString`
override that closes this gap (floor **+1**).

**Planned.** Per-type `toString`, `toNumber`, richer math, the `Int`/`Float`
substrate — U-CORE-4 / deferred.

---

### `String` — immutable UTF-8 text

| | |
|---|---|
| Superclass · Kind | `Object` · U/I · immutable, interpolating |
| Status | ◐ partial (concat + hash + full Wren-modelled protocol over a raw-byte floor, **landed U-STRING**) |

**Interface — floor** ([census §2.5](./floor-census.md)): `+(_)` (concatenation) ·
`hash` (cached djb2 **content** hash — equal content ⇒ equal hash, **landed
U-CORE-1**) · static `new()`/`new(_)` · `rawByteCount`/`rawByteAt(_)`/`rawSlice(_,_)`
(UTF-8 byte access, **landed U-STRING**, [ADR-0062](../../../adr/accepted/0062-amend-floor-admit-string-raw-byte-accessors-supersedes-0049-naming.md)).

**Interface — `.ph`-derived** (U-STRING, over the floor above):
`size`/`isEmpty` · `at(_) → Option` · `codePointAt(_)`/`leadByteLen_(_)` (UTF-8
decode via division/modulo, no bitwise ops per ADR-0024) · `indexOf(_)` ·
`split(_)` · `replace(_,_)` · `trim()`/`trimStart()`/`trimEnd()` and their
custom-charset forms `trim(_)`/`trimStart(_)`/`trimEnd(_)` · `*(count)` ·
`bytes`/`codePoints` (`StringByteSequence`/`StringCodePointSequence` sub-accessors,
ADR-0048-shaped). Argument-type/range violations raise `ArgumentError`.

**Surface syntax.** `\(expr)` interpolation landed with **U-LEX**
([ADR-0022](../../../adr/0022-string-interpolation-backslash-paren-sigil.md)) — a
lexer feature, adds no class rows.

**Planned.** Character indexing, comparison, `toSymbol`/`toNumber` — deferred
past U-STRING (see `deferred-work.md`).

---

### `Symbol` — interned identifier / selector

| | |
|---|---|
| Superclass · Kind | `Object` · I · interned |
| Status | ◐ partial |

**Interface — floor** ([census §2.7](./floor-census.md)): `toString` ·
`hash` (digest of the interned id — equal symbols agree, **landed U-CORE-1**) ·
static `new(_)` (interning constructor). **Caveat:** `value_eq` makes distinct
symbol values never `==` today ([catalog-delta §2.3](./catalog-delta.md)); U-CORE-4
owns the `==`/interning-identity protocol.

---

### `Option` / `Some` / `None` — the sole expression of absence

| | |
|---|---|
| `Option` | `Object` · **A** · the eliminator lives here so `Some`/`None` inherit it |
| `Some` | `Option` · U · one field `_value` at slot 0 ([ADR-0011](../../../adr/0011-static-instance-slot-layout.md), seeded in `VM::new`) |
| `None` | `Option` · a **shared singleton value** — bound as a *value* global, not a class global; carries no floor primitives |
| Status | ✅ combinator surface complete |

**Interface — floor** ([census §2.8](./floor-census.md)): `Some.new(_)` (static,
present-value construction) · `Option#match(some, none)` (the eliminator, on
abstract `Option`).

**Interface — `.ph`** ([`core.ph`](../../../../phalcom-core/core/core.ph) L70–124,
every one derived over `match`): `ifNone(_)` · `orElse(_)` · `isSome` · `isNone`
(**landed U-CORE-2**) · `map(_)` · `flatMap(_)` · `filter(_)` · `ifSome(_)` ·
`unwrapOr(_)` (**landed U-STD**).

**Architecture** ([ADR-0007](../../../adr/0007-option-as-abstract-with-some-none.md),
[values-and-absence.md](../values-and-absence.md) §3). The whole combinator suite is
`.ph` over the single `match` floor capability — the template for "push protocol
into `core.ph`, keep the floor minimal." `None` deliberately has **no** `core.ph`
reopen: `Statement::Class` always emits a `DefineGlobal`, and reopening `None` would
clobber its value-global binding back to the class object (see the `core.ph`
comment). There is **no `nil` surface** — forbidden by Invariant 4.

---

## 5. Callables & reflection

### `Function` / `Block` / `Method`

| | |
|---|---|
| `Function` | `Object` · **A** · abstract root of everything callable |
| `Block` | `Function` · U · first-class closure / block literal; adds non-local return + home frame |
| `Method` | `Function` · U · a reified compiled method; **sibling of `Block`, not a subtype** — re-parent `Method < Function` **landed U-CORE-1** (was `Method < Object`, an ADR-0006 violation) |
| Status | `Function`/`Block` ✅ call protocol; `Method` ◐ (reflection surface pending U-CORE-3) |

**Interface — floor** ([census §2.9–§2.10](./floor-census.md)). Installed on **both**
`Function` and `Block` (identical native fns, so a `Function` responds without a
`Block`): `arity` · `name` · `callWith(_)` · `call()` … `call(_,_,_,_)` (arities
**0–4**, `MAX_CALL_ARITY = 4`; dispatch keys on arity). `Block` adds `whileTrue(_)`
(**sacred** loop fallback). `Method` carries only static `new(_)` and **inherits**
the call protocol from `Function` after the re-parent.

**Architecture** ([ADR-0006](../../../adr/0006-function-as-abstract-callable-root.md),
[ADR-0013](../../../adr/0013-closure-upvalues-and-frame-token-return.md)). The block
*mechanism* (closures, upvalues, frame tokens, non-local `return`, `DeadFrameError`)
landed in **U4 + U10**; the call protocol is complete. An **unbound** `Method` has no
receiver, so raw `call` on it is an error — you must `bind` or `invokeOn`.

**Planned — U-CORE-3** (native, floor **+5**; each reads representation below `.ph`):
`Object#methodFor(_)` (reify a method by selector; `None` on miss) ·
`Method#invokeOn(_, _)` (run the exact reified method, no re-dispatch) ·
`Method#bind(_)` (close over a receiver → a `BoundMethod` whose surface class is
`Block`) · `Method#selector` · `Method#holder`. A first-class `Signature` object and
`isPrimitive` are deferred (no `Signature` kernel class in the catalog).

---

## 6. Collections

### `List` — the only built collection

| | |
|---|---|
| Superclass · Kind | `Object` · U · a native array-backed heap object (`ListObject`), **not** an `InstanceObject` ([ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md)) |
| Status | ◐ partial — combinator layer ✅ landed; only **literal syntax `[a,b,c]`** remains deferred |

**Interface — floor** ([census §2.13](./floor-census.md)): static `new()` ·
`length_` · `at_(_)` · `set_(_, _)` · `push_(_)` · `toString` (native this
unit — element stringification was blocked on U-CORE-4).

**Interface — `.ph`** ([`core.ph`](../../../../phalcom-core/core/core.ph) L142–212, all
over the raw floor): `size` · `at(_)` · `add(_)` · `each(_)` (**U-LIST**) ·
`map(_)` · `filter(_)` · `reduce(_, _)` · `includes(_)` · `isEmpty` ·
`at(_, put:)` (wraps `set_`) (**U-STD**).

**Architecture.** The hybrid pattern in miniature: five raw native primitives that
touch the backing `Vec`, everything else self-hosted. `List` is **mutable ⇒ not
hashable by value** ([decisions.md](./decisions.md) Q5); it inherits identity
`Object#hash`. Literal syntax `[a, b, c]` needs a new ADR + parser work (DEFERRED
#6) — **not** part of U-STD's combinator scope.

**Planned.** `Tuple`, `Map`, `Set`, `Range` — **absent** (names reserved in
`ClassName`). Per ADR-0020 each is its own U-STD/deferred unit; `Map`/`Set` also
depend on `Object#hash` (now landed). **U-CORE-5's job is the shared
collection-protocol *contract* + conformance harness, not these classes** — it adds
zero floor primitives and makes `List` the reference implementation (`.ph`
`List#==`/`!=`).

---

## 7. Runtime & namespaces

### `Module`

| | |
|---|---|
| Superclass · Kind | `Object` · U · a compilation unit / namespace |
| Interface — floor | static `new()` ([census §2.12](./floor-census.md)) |
| Status · Planned | ◐ — namespace/import surface deferred ([ADR-0027](../../../adr/0027-modules-as-files-with-public-by-default-imports.md) rules modules-as-files, public-by-default) |

### `System`

| | |
|---|---|
| Superclass · Kind | `Object` · U · the runtime service surface (class-side) |
| Interface — floor | static `print(_)` (the **sole I/O primitive**) · static `new()` ([census §2.11](./floor-census.md)) |
| Interface — `.ph` | an empty `static print()` shell backed by the native primitive |
| Status · Planned | ◐ — `clock` / `gc` / scheduler are **out of core scope** |

---

## 8. Errors & concurrency — greenfield

### Errors — reification pending (U-CORE-6, [ADR-0008](../../../adr/0008-layered-exceptions-and-result.md))

The *mechanism* exists natively as the `RuntimeError` enum (`error.rs`), but is
**not reified** into a surface `Error` hierarchy — none of these are surface classes
yet.

| Class | Superclass | Role | Status |
|---|---|---|---|
| `Error` | `Object` | root of raisable errors; `message`, `raise` | ❌ → U-CORE-6 (floor **+2**) |
| `MessageNotUnderstood` | `Error` | raised by default `doesNotUnderstand(_)` | ❌ → U-CORE-6 |
| `DeadFrameError` | `Error` | non-local `return` to a dead frame | ❌ surface (native `RuntimeError` exists, U10) |
| `TypeError` / `ArgumentError` / `RangeError` | `Error` | typed error subclasses | ❌ → U-CORE-6 |

**U-CORE-6 scope** is the *minimal reification slice* of ADR-0008: reify `Error` +
`MessageNotUnderstood` with `message`/`raise`, and rewire U8's native miss path to
**raise a surface `MessageNotUnderstood`** carrying the reified `Message` through the
**unified unwind** (the sibling *Raise* payload to U10's *Return* payload). The
exceptions-vs-`Result` layering is pre-decided by ADR-0008 — read against it, do not
redesign.

### Concurrency — out of core scope (forward-compat only)

`Fiber` and `Future` are **non-goals** for the core library (names reserved in
`ClassName`). The only requirement is the *"must not preclude"* clause: the
`Object`/`Module`/callable layout must leave room for them without a breaking change
([forward-compat.md](./forward-compat.md) §1). U-CORE-1/3's `.ph` `isA` and
open-`Value`-arm hashing already clear that gate.

---

## 9. Cross-cutting architecture

### 9.1 The frozen floor ([ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md))

A **floor primitive** is native Rust bound onto a kernel class at bootstrap because
it *cannot* be expressed in `.ph` over lower-level `.ph` (it touches heap
representation, an immediate's bits, control flow, or I/O). The floor is **closed**:
the default answer to "add a primitive" is **no**. Every new capability is either
**derivable** (write it in `core.ph`) or an **ADR-recorded amendment**. The language
is self-hosting above a small fixed native boundary.

Current floor (post-U-CORE-1): **80** installed `(class, selector)` bindings · **64**
distinct native fns · **16** floor-carrying classes (of 21 named) · **7** sacred
selectors. The count is **machine-checked** — `floor_census_matches_installed_bindings`
(`tests/invariants.rs`, R-INV-0.1, landed U-CORE-1) reconstructs the set from a live
`VM::new()` and fails on drift. Planned floor deltas: U-CORE-3 **+5**, U-CORE-4
**+1**, U-CORE-6 **+2** → **88** if all land.

### 9.2 Two `new`s — the allocator ordering ([census §4](./floor-census.md))

`new()` is bound twice: `object_class_new` on **`Object class`** (metaclass) and
`class_new` on **`Class`**. For `Foo.new`, the metaclass chain
`Foo class → Object class → Class → Behavior → Object` reaches `Object class`
**first**, so `object_class_new` is the effective default allocator and `class_new`
is a deeper fallback. Specialized static `new`s (`Number`, `String`, `Bool`,
`Symbol`, `Method`, `List`, `System`, `Module`) override on their own metaclass. This
ordering is **load-bearing for `construct`** (U7 / ADR-0011) — preserve it.

### 9.3 Sacred selectors (R-SACRED, [census §5](./floor-census.md), [ADR-0018](../../../adr/0018-sacred-selector-inliner-and-override-guard.md))

Seven floor selectors are compiler-coupled: the sacred inliner special-cases their
literal-block call sites and emits a `GuardBool` deopt that falls back to *exactly
these* sends on override or receiver mismatch.

| Receiver | Sacred selectors |
|---|---|
| `Bool` | `and(_)`, `or(_)`, `not()`, `ifTrue(_)`, `ifFalse(_)`, `ifTrue(_, ifFalse)` |
| `Block` | `whileTrue(_)` |

**Any unit that reopens `Bool` or `Block` must keep these exact selector shapes and
budget for the deopt if it replaces a sacred body.**

### 9.4 The floor ↔ `core.ph` boundary

Today `List` and `Option` carry `.ph` surface; every other `core.ph` class is an
**empty reopen** that only makes the name surface-visible (`Object` also adds `isA`).
The boundary is the template for U-CORE-2…5: a method belongs on the floor **only**
if it fails the §9.1 derivability test. The bootstrap layering rule (R-BOOT-2,
[bootstrap-phases.md](./bootstrap-phases.md) §5) requires each `.ph` body to send only
already-resolvable selectors — `isA` qualifies because it sends only category-(a)
native floor selectors plus the `None` global.

### 9.5 Bootstrap order (allocate-then-wire, [object-model §6](../object-model.md#6-bootstrap-construction-order))

The tower's circularity is resolved by allocating the kernel classes and metaclasses
as bare objects, then wiring instance-of, then instance-side superclasses, then
metaclass-side superclasses (by the parallel rule), then the remaining core classes,
then primitives, then `verify_invariants()`. The phase-scoped invariant ledger is
[bootstrap-phases.md](./bootstrap-phases.md).

---

## 10. Status matrix

Rollup by catalog group (rows that exist / catalog rows), from
[catalog-delta §3](./catalog-delta.md), recomputed post-U-CORE-1:

| Group | Rows | Fully ✅ | Partial ◐ | Absent ❌ | Owning units |
|---|:--:|:--:|:--:|:--:|---|
| Kernel tower | 4/4 | 1 (`Metaclass`) | 3 | 0 | U-CORE-1 |
| Primitives & singletons | 6/6 | 2 (`True`/`False`, `Option`) | 4 | 0 | U-CORE-2/4, U11 |
| Callables & reflection | 3/3 | 2 (`Function`/`Block`) | 1 (`Method`) | 0 | U-CORE-3 |
| Collections | 1/5 | 0 | 1 (`List`) | 4 | U-STD, U-CORE-5 (contract) |
| Runtime & namespaces | 2/2 | 0 | 2 | 0 | deferred |
| Concurrency | 0/2 | 0 | 0 | 2 | out of scope |
| Errors | 0/6 | 0 | 0 | 6 | U-CORE-6 |
| **Total** | **16/28** | **5** | **11** | **12** | |

Plus two **impl-only** rows outside the object-model §4 catalog: **`Nil`** (backs
`Value::Nil.class`; no global, no primitives — unreachable by construction, §8 of
[catalog-delta](./catalog-delta.md)) and **`Message`** (the U8 reified miss-send;
four accessor primitives `selector`/`name`/`labels`/`args`, [census §2.14](./floor-census.md)).

**Reading.** The tower and value/callable spine are *present but thin* (11 of 16
existing rows are partial). The genuine greenfield is collections beyond `List`, all
of errors (core, U-CORE-6), and all of concurrency (out of scope).

---

## 11. Traceability

| Claim | Source |
|---|---|
| Tower rows / superclasses | `universe.rs::create_core_classes` |
| Globals | `vm.rs::install_core` |
| Floor primitives (per class) | [`floor-census.md`](./floor-census.md) §2 |
| `.ph` surface | [`core.ph`](../../../../phalcom-core/core/core.ph) |
| Target catalog & tower rules | [`../object-model.md`](../object-model.md) §4–§8 |
| Delta / pending / divergences | [`catalog-delta.md`](./catalog-delta.md) §2–§4 |
| Gating decisions (hash, errors, toString, Method re-parent, collections) | [`decisions.md`](./decisions.md) |
| Landed units | [U-CORE-1](../../../forge/units/U-CORE-1/as-built.md) (`hash`/`isA`/`Behavior`/`Method<Function`), [U-CORE-2](../../../forge/units/U-CORE-2/as-built.md) (`Some`-lift + `Option`), [U-STD](../../../forge/units/U-STD/as-built.md) (combinators), [U11](../../../forge/units/U11/as-built.md) (`True`/`False`), [U-LIST](../../../forge/units/U-LIST/as-built.md) (native `List`) |
| Planned units | [U-CORE-3](../../../forge/units/U-CORE-3/as-built.md) (callables), [U-CORE-4](../../../forge/units/U-CORE-4/as-built.md) (value `toString`/Int-Float), [U-CORE-5](../../../forge/units/U-CORE-5/as-built.md) (collection contract), [U-CORE-6](../../../forge/units/U-CORE-6/as-built.md) (errors) |
| Baseline pin & drift policy | [`README.md`](./README.md) §"Baseline & drift policy" |
