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
  `install_primitives`. `call` is bound at five arities on two classes, so it
  contributes ten bindings.
- **Distinct native functions** — the Rust `fn` behind the bindings. `name` and
  `toString` on `Object` share `object_name`; `call`/`arity`/`name`/`callWith`
  are shared between `Function` and `Block`.

| Metric | Count |
|---|---|
| Installed `(class, selector)` bindings | **80** |
| Distinct native Rust functions | **64** |
| Classes carrying floor primitives | **16** (of 21 named kernel classes) |
| Sacred selectors (§5) | **7** |

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

> **Baseline:** HEAD `0f84232`. Folds in **U8** (`Object` reflective surface —
> `perform`/`respondsTo`/`doesNotUnderstand` — and the `Message` class,
> §2.1/§2.14), **U9** (variadics — no new floor bindings), **U-CORE-2**
> (Bool `Some`-lift + core `Option` combinators — **no new floor bindings**; the
> `wrap_some` helper is `pub(crate)`, not installed, and `WrapSome` is a bytecode
> op, not a bound selector), **U-LEX** (comments, numeric separators, newline
> suppression, string interpolation — pure lexer/parser work, **no new floor
> bindings**), **U-STD** (`Option`/`List` combinators — pure `.ph` over
> existing floor primitives, **no new floor bindings**), and **U11** (`True`/
> `False` singleton subclasses of `Bool`, ADR-0004 — **+2 kernel classes,
> +0 floor bindings**; dispatch is via inheritance, see §2.6). None of
> U8/U9/U-CORE-2/U-LEX/U-STD/U11 added a floor binding, so through those bumps
> the counts held at 73 bindings / 57 native fns — only the kernel-class
> denominator moved (19 → 21). Do not infer "classes added" implies "bindings
> added": U11 is the concrete counterexample. **U-CORE-1 has since landed**
> (commits `03764e3`/`b1109c2`), the first bump to *add* floor bindings: **+7**
> bindings (73 → **80**) and **+7** native fns (57 → **64**), per the ADR-0023
> amendment above — so the current figures are the **80 / 64** in the §1.1
> table, not the old 73 / 57. See [`README.md`](./README.md) for the
> baseline-pin policy.

### 1.2 Selector notation

Selectors are shown in **human-facing notation**: a getter is a bare name
(`size`), a setter is `name=(_)`, an arity-*n* method is `name(_, …)` with *n*
positional holes (`+(_)`, `new()`), and labeled arguments are named
(`ifTrue(_, ifFalse)`, `match(some, none)`).

> **Notation vs the interned string.** This differs from the canonical selector
> string that `make_signature`/`encode_selector`
> ([`method.rs`](../../../../phalcom-core/src/method.rs),
> [ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md))
> actually intern, which writes each positional hole as `_:` and each label as
> `label:`. So `+(_)` interns as `+(_:)`, `class=(_)` as `class=(_:)`, and
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
([`universe.rs`](../../../../phalcom-core/src/universe.rs) L213–358).

### 2.1 `Object` — root protocol

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `name` | instance | `object_name` | class-name string ([ADR-0015](../../../adr/0015-object-default-tostring.md)) |
| `class` | instance | `object_class` | |
| `class=(_)` | instance | `object_set_class` | reflective class reassignment |
| `toString` | instance | `object_name` | **aliases** `name` (ADR-0015 default) |
| `new()` | static | `object_class_new` | generic instance allocator — the default `new` for user classes (see §4) |
| `==(_)` | instance | `object_eq` | ordinary send, **not** an opcode (control-flow.md §1) |
| `!=(_)` | instance | `object_neq` | ordinary send |
| `perform(_)` | instance | `object_perform` | reflective send (U8, messages-and-selectors.md §5) |
| `perform(_, _)` | instance | `object_perform_with` | reflective send with a packed args argument |
| `respondsTo(_)` | instance | `object_responds_to` | pure probe; never triggers dNU |
| `doesNotUnderstand(_)` | instance | `object_does_not_understand` | terminal miss handler; overridable so a proxy subclass can intercept |
| `hash` | instance | `object_hash` | identity digest of the heap handle (ADR-0023); immediates override below |

### 2.2 `Behavior` — class-side reflection

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `superclass` | instance | `class_superclass` | on `Behavior` so `Class` and `Metaclass` both inherit it ([ADR-0003](../../../adr/0003-introduce-behavior-kernel-class.md)) |
| `superclass=(_)` | instance | `class_set_superclass` | |
| `name` | instance | `behavior_name` | the receiver class's OWN name; **shadows** `Object#name` for class receivers (ADR-0023) |
| `methods` | instance | `behavior_methods` | own method-dictionary selector Symbols, as a fresh `List` (ADR-0023) |

### 2.3 `Class` — instantiation apex

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `+(_)` | instance | `class_add` | |
| `new()` | instance | `class_new` | allocator reachable through the metaclass chain apex (see §4) |

### 2.4 `Number` — flat `f64` ([ADR-0005](../../../adr/0005-number-as-flat-f64.md))

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `+(_)` `-(_)` `*(_)` `/(_)` `%(_)` | instance | `number_add` … `number_mod` | never inlined; ordinary sends (control-flow.md §1) |
| `<(_)` `<=(_)` `>(_)` `>=(_)` | instance | `number_lt` … `number_ge` | |
| `negated()` | instance | `number_negated` | |
| `hash` | instance | `number_hash` | digest of the mathematical value, class-agnostically (ADR-0023; forward-compat §4) |
| `new()` , `new(_)` | static | `number_class_new` | coercion / zero |

### 2.5 `String`

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `+(_)` | instance | `string_add` | concatenation |
| `hash` | instance | `string_hash` | cached djb2 **content** hash — equal content ⇒ equal hash (ADR-0023) |
| `new()` , `new(_)` | static | `string_class_new` | |

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

> **U11 landed** (`true_class`/`false_class`, [`universe.rs`](../../../../phalcom-core/src/universe.rs)
> L550/L555): `True`/`False` are now concrete singleton subclasses of `Bool`,
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
| `new(_)` | static | `Some` | `some_new` | present-value construction; `Some` has one field `_value` at slot 0 (ADR-0011, seeded in `VM::new`) |
| `match(some, none)`† | instance | `Option` | `option_match` | the eliminator, on abstract `Option` so `Some`/`None` inherit it (values-and-absence.md §3.2); encoded explicitly; interns as `match(some:none:)` |

† rendered from `encode_selector("match", [Some("some"), Some("none")], Method(2))`.
`None` carries **no** floor primitives of its own — it is a shared singleton
value, not a constructed instance. The combinator suite (`map`, `flatMap`,
`orElse`, `ifSome`, `unwrapOr`, …) is deliberately **not** on the floor; it is
`core.ph`/U-STD work layered over `match`.

### 2.9 `Method`

`Method < Function` as of U-CORE-1 (decisions.md §4.1 / ADR-0006 re-parent,
applied in `create_core_classes` with the load-order fix). It inherits the
call protocol (`arity`/`name`/`call…`/`callWith`) from `Function` and carries
only its own static `new(_)`.

| Selector | Side | Native fn |
|---|---|---|
| `new(_)` | static | `method_class_new` |

### 2.10 `Function` / `Block` — callables ([ADR-0006](../../../adr/0006-function-as-abstract-callable-root.md), [ADR-0013](../../../adr/0013-closure-upvalues-and-frame-token-return.md))

`Block` is a subclass of `Function`; the shared callable protocol is installed
on **both** rows (identical native fns) so a `Function` value responds even
without a `Block`.

| Selector | Side | Class(es) | Native fn | Sacred? |
|---|---|---|---|---|
| `arity` | instance | Function, Block | `block_arity` | |
| `name` | instance | Function, Block | `block_name` | |
| `callWith(_)` | instance | Function, Block | `block_call_with` | one packed argument |
| `call()` … `call(_,_,_,_)` | instance | Function, Block | `block_call` | arities **0–4** (`MAX_CALL_ARITY = 4`); dispatch keys on arity, so one entry per arity (functions.md §1) |
| `whileTrue(_)` | instance | Block | `block_while_true` | ★ sacred loop fallback |

### 2.11 `System` — I/O floor

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `print(_)` | static | `system_class_print` | the sole I/O primitive |
| `new()` | static | `system_class_new` | |

### 2.12 `Module`

| Selector | Side | Native fn |
|---|---|---|
| `new()` | static | `module_class_new` |

### 2.13 `List` — native array-backed kernel collection ([ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md))

A dedicated `Object::List` heap variant (`crate::list::ListObject`), **not** an
`InstanceObject`. The floor is five raw primitives + native `toString`; the
public protocol (`size`/`at`/`add`/`each`) is `core.ph` over them (§3).

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `new()` | static | `list_class_new` | |
| `rawLength` | instance | `list_raw_length` | internal; wrapped by `size` |
| `rawAt(_)` | instance | `list_raw_at` | internal; wrapped by `at(_)` |
| `rawSet(_, _)` | instance | `list_raw_set` | **installed but unwrapped** — no `at(_, put)` yet (§6) |
| `rawPush(_)` | instance | `list_raw_push` | internal; wrapped by `add(_)`; amortized growth folds into `Vec::push` |
| `toString` | instance | `list_to_string` | native this unit (see U-LIST return contract) |

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

## 3. The floor ↔ `core.ph` boundary

Two classes now carry `.ph` surface protocol self-hosted over the floor
([`core.ph`](../../../../phalcom-core/core/core.ph)):

**`List`** (ADR-0020) —

```
size    => self.rawLength
at(i)   { return self.rawAt(i) }
add(v)  { self.rawPush(v); return self }
each(f) { var i = 0; while (i < self.size) { f.call(self.at(i)); i = i + 1 } }
```

`each` closes over three floor capabilities — `Block#call(_)`, `Number#<(_)`,
and `while` lowering (`Block#whileTrue(_)` / sacred inliner) — plus the
same-class `size`/`at` defined above it.

**`Option`** (U-CORE-2, `0da64d6`) — four combinators, each derived purely over
the `match(some, none)` eliminator (the sole floor capability they touch):

```
ifNone(f) => self.match(some: { v => self }, none: { f.call(); self })
orElse(f) => self.match(some: { v => self }, none: { f.call() })
isSome    => self.match(some: { v => true }, none: { false })
isNone    => self.match(some: { v => false }, none: { true })
```

Every other `core.ph` class today is an **empty reopen** (`Object`, `Class`,
`Metaclass`, `Number`, `String`, `Bool`, `Symbol`, `Some`) that only makes the
name surface-visible; `System` carries an empty `static print()` shell backed by
the native primitive. (`None` deliberately has **no** reopen — see the `core.ph`
comment on the `DefineGlobal`-clobber hazard.)

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
| `List#at(_, put)` (wrap `rawSet`) | primitive exists, unwrapped | U-STD |
| `List` `map`/`reduce`/`filter`/literal syntax | derivable over floor | U-STD |
| `Option` combinators (`map`/`flatMap`/`orElse`/`ifSome`/`unwrapOr`) | derivable over `match` | U-STD / U-CORE-2 |
| `Block#repeat(_)` | receiver/semantics unpinned | deferred (U5-plan BD-U5-2) |
| `callWith(_)` packed-arg semantics | bound, but forwards plainly | firms up once `List` is the pack type |
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
   the census here (count = 80). This turns silent floor drift into a red test;
   §1.1 is no longer a manual checksum.

## 8. Traceability

| Section | Source lines |
|---|---|
| §2 all | `universe.rs::install_primitives` L225–388 |
| §2.1 Object reflective surface (U8) | `universe.rs` L240–243 |
| §2.6 encoded `ifTrue(_:ifFalse:)` | `universe.rs` L302–308 |
| §2.8 encoded `match` | `universe.rs` L328–335 |
| §2.8 `Some` field layout | `vm.rs::new` (stamped alongside `Message`) |
| §2.10 `MAX_CALL_ARITY` | `universe.rs` L345 |
| §2.14 `Message` | `universe.rs` create L173, primitives L249–253 |
| §3 `List` protocol | `core.ph` L53–72 |
| §5 sacred set | `universe.rs` L73–79, L214–222 |
