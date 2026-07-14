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
- **Distinct native functions** — the Rust `fn` behind the bindings.
  `call`/`arity`/`name`/`callWith` are shared between `Function` and `Block`.

| Metric | Count |
|---|---|
| Installed `(class, selector)` bindings | **113** |
| Distinct native Rust functions | **98** |
| Classes carrying floor primitives | **22** (of 28 named kernel classes) |
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

> **U-CORE-3 amendment ([ADR-0028](../../../adr/0028-amend-floor-admit-method-reflection.md)).**
> The `Method` reflection surface admits **+5** bindings (80 → 85) and **+5**
> distinct fns (64 → 69): `Object#methodFor(_)` (`object_method_for`),
> `Method#invokeOn(_,_)` (`method_invoke_on`), `Method#bind(_)`
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
> **+14** distinct fns (73 → 87): `Map` — `new()`, `size_`, `get_(_)`,
> `put_(_,_)`, `has_(_)`, `remove_(_)`, `keyAt_(_)`, `valueAt_(_)`
> (8, `primitive/map.rs`); `Set` — `new()`, `size_`, `add_(_)`, `has_(_)`,
> `remove_(_)`, `at_(_)` (6, `primitive/set.rs`). `Set` shares `Map`'s Rust
> backing struct ([`MapObject`](../../../../phalcom-core/src/map.rs), DEC-CT-B)
> but every binding is its own distinct native fn — no rehome subtlety.
> Floor-carrying classes move **17 → 19** (`Map`/`Set` are new rows, neither
> previously carrying a primitive). `get_`/`put_`/`has_`/`remove_`
> re-enter the VM to send Phalcom `hash`/`==` on keys (not Rust `Value: Hash`)
> and `put_`/`add_` reject a mutable-collection key (DEC-CT-C,
> collection-protocol.md law 4). R-INV-0.1 (`tests/invariants.rs`) audits this
> set from a live `VM::new()` and fails on drift.

> **U-COLLTYPES Phase 2 amendment ([ADR-0039](../../../adr/0039-amend-floor-admit-collection-container-primitives.md)).**
> The `Tuple` floor admits **+3** bindings (102 → 105) and **+3** distinct fns
> (87 → 90): `fromList(_)` (static, `tuple_class_from_list`), `size_`
> (`tuple_raw_size`), `at_(_)` (`tuple_raw_at`) — all in `primitive/tuple.rs`.
> **No mutation primitive** — immutability is a representation guarantee
> ([`TupleObject`](../../../../phalcom-core/src/tuple.rs)'s `Box<[Value]>`, no
> mutable accessor exists). Floor-carrying classes move **19 → 20**. `hash`
> stays `.ph` (DEC-CT-D: an order-sensitive fold over `at_`+element `.hash`,
> zero new floor) — it is **not** a binding here. R-INV-0.1 audits this set.

> **U-COLLTYPES Phase 3 amendment ([ADR-0039](../../../adr/0039-amend-floor-admit-collection-container-primitives.md)).**
> The `Range` floor admits **+4** bindings (105 → 109) and **+4** distinct fns
> (90 → 94): `new(_,_,_)` (static, `range_class_new`), `start_`
> (`range_raw_start`), `end_` (`range_raw_end`), `inclusive_`
> (`range_raw_inclusive`) — all in `primitive/range.rs`. This is the **whole**
> floor for `Range` — three field reads plus the allocator; everything else
> (`size`/`at(_)`/`includes(_)`/`first`/`last`/`each(_)`/`toList`/`==`/`hash`/
> `iterate`/`iteratorValue`) is `.ph` over these + `Number` arithmetic
> ([`RangeObject`](../../../../phalcom-core/src/range.rs) holds **no** element
> storage — RG-2 laziness). Floor-carrying classes move **20 → 21**. R-INV-0.1
> audits this set.

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
> **U16-Open amendment ([ADR-0047](../../../adr/0047-amend-floor-admit-family-call-router.md)).**
> The `::` method-reference call router admits **+1** binding (112 → 113) and
> **+1** distinct fn (97 → 98): `Family#doesNotUnderstand(_)`
> (`family_does_not_understand`, `primitive/family.rs`) — overrides `Object`'s
> default miss handler so a `Family` value's own bare-call sends (`call()`,
> `call(_:)`, `call(to:duration:)`, …), which `Family` never defines directly,
> reach a handler that rebuilds the real selector from the family's base name
> plus the missed call's decoded labels and re-dispatches it as an ordinary
> send (selectors.md §3 "a family call *is* a send"). This is the *only*
> selector `Family` carries — there is no reflective surface in this unit (Q14
> ruling, `open-questions.md`). Floor-carrying classes go **21 → 22** (`Family`
> is new). R-INV-0.1 audits this set.
>
> **Baseline:** post-U15 — the figures above (**112 / 97 / 21 / 7**) are the
> current floor (was **111 / 96 / 21 / 7** post-U-ERR, **109 / 94 / 21 / 7**
> post-U-COLLTYPES-Phase-3, **105 / 90 / 20 / 7** post-Phase-2,
> **102 / 87 / 19 / 7** post-Phase-1, **88 / 73 / 17 / 7** post-U-CORE-6). The
> authoritative pin + full landing history live in [`README.md`](./README.md)
> §"Baseline & drift policy"; this census is the ground-truth enumeration
> behind that count. One census-specific caution: of the post-U-CORE-0
> landings, **U-CORE-1 added +7 (73 → 80, ADR-0023), U-CORE-3 added +5
> (80 → 85, ADR-0028), U-CORE-4 added +1 (85 → 86, ADR-0036), U-CORE-6 added
> +2 (86 → 88, ADR-0037), U-COLLTYPES Phase 1 added +14 (88 → 102, ADR-0039),
> U-COLLTYPES Phase 2 added +3 (102 → 105, ADR-0039), U-COLLTYPES Phase 3
> added +4 (105 → 109, ADR-0039), U-ERR added +2 (109 → 111, ADR-0038), and
> U15 added +1 (111 → 112, ADR-0045)** — every other unit either landed
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
| `toString` | instance | `object_to_string` | default display, `"<ClassName>"` for an instance / own name for a class receiver (ADR-0015; U-CORE-4 re-home off `object_name`, fixes DEFERRED F4) |
| `new()` | static | `object_class_new` | generic instance allocator — the default `new` for user classes (see §4) |
| `==(_)` | instance | `object_eq` | ordinary send, **not** an opcode (control-flow.md §1) |
| `!=(_)` | instance | `object_neq` | ordinary send |
| `perform(_)` | instance | `object_perform` | reflective send (U8, messages-and-selectors.md §5) |
| `perform(_, _)` | instance | `object_perform_with` | reflective send with a packed args argument |
| `respondsTo(_)` | instance | `object_responds_to` | pure probe; never triggers dNU |
| `doesNotUnderstand(_)` | instance | `object_does_not_understand` | terminal miss handler; overridable so a proxy subclass can intercept |
| `hash` | instance | `object_hash` | identity digest of the heap handle (ADR-0023); immediates override below |
| `methodFor(_)` | instance | `object_method_for` | reifies the resolved `Method` for a selector; `None` on a miss; pure probe, never fires dNU (U-CORE-3, ADR-0028) |
| `__invariantEnter()` | instance | `object_invariant_enter` | `@invariant` re-entrancy guard entry (U-ANNOT-CONTRACTS, [ADR-0052](../../../adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md) Fix 1); returns whether this call owns the receiver's `checking` entry |
| `__invariantExit()` | instance | `object_invariant_exit` | `@invariant` re-entrancy guard exit; unconditionally removes the receiver from `checking` |

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
| `toString` | instance | `number_to_string` | decimal-string render of the `f64` value, delegates to `Value::to_string` (U-CORE-4, ADR-00NN amendment) |
| `new()` , `new(_)` | static | `number_class_new` | coercion / zero |

### 2.5 `String`

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `+(_)` | instance | `string_add` | concatenation |
| `hash` | instance | `string_hash` | cached djb2 **content** hash — equal content ⇒ equal hash (ADR-0023) |
| `new()` , `new(_)` | static | `string_class_new` | |
| `rawByteCount` | instance | `string_raw_byte_count` | UTF-8 buffer length in bytes (ADR-0049, U-STRING) |
| `rawByteAt(_)` | instance | `string_raw_byte_at` | raw byte at a byte offset, or `None` out of bounds (ADR-0049, U-STRING) |
| `rawSlice(_,_)` | instance | `string_raw_slice` | substring by byte range `[start, end)`, validates UTF-8 char boundaries, never panics (ADR-0049, U-STRING) |

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
call protocol (`arity`/`name`/`call…`/`callWith`) from `Function` — but does
not answer raw `call` while unbound (`resolve_callable` rejects a bare
`Object::Method` receiver with a dedicated error) — and carries its own
static `new(_)` plus the U-CORE-3 reflection surface
([ADR-0028](../../../adr/0028-amend-floor-admit-method-reflection.md)):
applying a reified method to an explicit receiver (`invokeOn`), closing one
over a receiver (`bind`), and reading its selector/holder.

| Selector | Side | Native fn |
|---|---|---|
| `new(_)` | static | `method_class_new` |
| `invokeOn(_,_)` | instance | `method_invoke_on` |
| `bind(_)` | instance | `method_bind` |
| `selector` | instance | `method_selector` |
| `holder` | instance | `method_holder` |

`Object#methodFor(_)` (`object_method_for`, §2.1) reifies the `MethodObject` a
selector resolves to on a receiver, as a bare `Method` value; the `None`
singleton on a miss. `bind(_)` returns a new heap representation,
`Object::BoundMethod` (method handle + receiver, no closure or frame token —
it must work for primitive methods too), whose surface class is `Block`; see
§2.10 for how it answers the call protocol.

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
| `on(_,_)` | instance | Block | `block_on` | U-ERR, ADR-0038 — typed catch (`try`/`on`/`catch` desugar target) |
| `ensure(_)` | instance | Block | `block_ensure` | U-ERR, ADR-0038 — always-runs cleanup (`try`/`ensure` desugar target) |

**U-CORE-3 behavior completions (zero new bindings, ADR-0028).**
`block_arity`/`block_name` learn an `Object::Method` receiver (reading
`signature.positional_arity`/`signature.selector` directly) and an
`Object::BoundMethod` receiver (delegating to the wrapped method); `block_call`
intercepts `Object::BoundMethod` **before** `resolve_callable` and funnels it
through `VM::invoke_method_object` — the same engine `Method#invokeOn(_,_)`
uses — so `bound.call(args) ≡ method.invokeOn(recv, args)` holds by
construction (R-INV-3.3).

### 2.11 `System` — I/O floor

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `print(_)` | static | `system_class_print` | the sole I/O primitive |
| `new()` | static | `system_class_new` | |
| `rawWrite(_)` | static | `system_raw_write` | raw stdout write, no newline; `write(_)`/`writeObject_(_)` are `.ph`-derived over it (ADR-0049, U-STRING) |

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
| `length_` | instance | `list_raw_length` | internal; wrapped by `size` |
| `at_(_)` | instance | `list_raw_at` | internal; wrapped by `at(_)` |
| `set_(_, _)` | instance | `list_raw_set` | **installed but unwrapped** — no `at(_, put)` yet (§6) |
| `push_(_)` | instance | `list_raw_push` | internal; wrapped by `add(_)`; amortized growth folds into `Vec::push` |
| `toString` | instance | `list_to_string` | native this unit (see U-LIST return contract) |

### 2.13a `Map`/`Set` — native hash collections (U-COLLTYPES Phase 1, [ADR-0032](../../../adr/0032-collections-representation-and-literals.md) §1, [ADR-0039](../../../adr/0039-amend-floor-admit-collection-container-primitives.md))

Dedicated `Object::Map`/`Object::Set` heap variants over the shared
`crate::map::MapObject` ordered-hash backing struct (DEC-CT-B: `Set` is a
keys-only sibling, distinct heap variant, distinct bindings). Both are
**mutable** ⇒ inherit identity `Object#hash` (Q5) — neither installs its own
`hash`, so neither is a valid `Map`/`Set` key. `get_`/`put_`/`has_`/
`remove_`/`add_` re-enter the VM to send **Phalcom** `hash`/`==` on keys
(`primitive/map.rs`'s `locate`); `put_`/`add_` reject a mutable-collection
key (`List`/`Map`/`Set`, DEC-CT-C) with a raised catchable `Error`. The public
protocol (`at(_)`/`at(_,put:)`/`size`/`includes(_)`/`remove(_)`/`keys`/
`values`/`each(_)` for `Map`; `add(_)`/`includes(_)`/`size`/`remove(_)`/
`each(_)`/`at(_)` for `Set`) is `core.ph` over these (§3).

| Selector | Side | Class | Native fn | Notes |
|---|---|---|---|---|
| `new()` | static | `Map` | `map_class_new` | |
| `size_` | instance | `Map` | `map_raw_size` | wrapped by `size` |
| `get_(_)` | instance | `Map` | `map_raw_get` | wrapped by `at(_)`; total (raw value on hit, `None` on miss) |
| `put_(_, _)` | instance | `Map` | `map_raw_put` | wrapped by `at(_, put:)`; DEC-CT-C mutable-key rejection |
| `has_(_)` | instance | `Map` | `map_raw_has` | wrapped by `includes(_)` |
| `remove_(_)` | instance | `Map` | `map_raw_remove` | wrapped by `remove(_)`; idempotent |
| `keyAt_(_)` | instance | `Map` | `map_raw_key_at` | backs `keys`/`each(_)`/`iteratorValue(_)` |
| `valueAt_(_)` | instance | `Map` | `map_raw_value_at` | backs `values`/`each(_)` |
| `new()` | static | `Set` | `set_class_new` | |
| `size_` | instance | `Set` | `set_raw_size` | wrapped by `size` |
| `add_(_)` | instance | `Set` | `set_raw_add` | wrapped by `add(_)`; idempotent; DEC-CT-C mutable-key rejection |
| `has_(_)` | instance | `Set` | `set_raw_has` | wrapped by `includes(_)` |
| `remove_(_)` | instance | `Set` | `set_raw_remove` | wrapped by `remove(_)`; idempotent |
| `at_(_)` | instance | `Set` | `set_raw_at` | wrapped by `at(_)`/`each(_)`; insertion-order indexed read |

### 2.13b `Tuple` — native fixed-arity immutable product (U-COLLTYPES Phase 2, [ADR-0032](../../../adr/0032-collections-representation-and-literals.md) §1, [ADR-0039](../../../adr/0039-amend-floor-admit-collection-container-primitives.md))

A dedicated `Object::Tuple` heap variant (`crate::tuple::TupleObject`, a fixed
`Box<[Value]>`), **not** an `InstanceObject`. The floor is three raw
primitives — **no mutation primitive**, since immutability is structural (no
`at(_, put:)`/`add(_)` accessor exists at all). Immutable ⇒ value-hashable and
a valid `Map`/`Set` key (Q5). The public protocol (`size`/`at(_)`/`each(_)`/
`==`/`!=`/`hash`) is `core.ph` over these (§3); `hash` is a `.ph` fold over
`at_`+element `.hash` (DEC-CT-D), not a floor primitive.

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `fromList(_)` | static | `tuple_class_from_list` | freezes a `List`'s current elements into a fresh `Tuple`; the `(a, b)` literal's construction target |
| `size_` | instance | `tuple_raw_size` | wrapped by `size` |
| `at_(_)` | instance | `tuple_raw_at` | wrapped by `at(_)`/`each(_)`; total (raw value on hit, `None` on miss) |

### 2.13c `Range` — native lazy numeric interval (U-COLLTYPES Phase 3, [ADR-0032](../../../adr/0032-collections-representation-and-literals.md) §1, [ADR-0039](../../../adr/0039-amend-floor-admit-collection-container-primitives.md))

A dedicated `Object::Range` heap variant (`crate::range::RangeObject`) — three
fields (`start`/`end`/`inclusive`), **no element storage** (RG-2 laziness).
The floor is the allocator plus the three field reads — the smallest floor of
the four container arms. The public protocol (`size`/`at(_)`/`includes(_)`/
`first`/`last`/`each(_)`/`toList`/`==`/`!=`/`hash`/`iterate`/`iteratorValue`)
is entirely `.ph` over these + `Number` arithmetic (§3); `each`/`toList`
*generate* elements, never allocate a buffer up front.

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `new(_, _, _)` | static | `range_class_new` | `(start, end, inclusive)`; RG-1's bound convention |
| `start_` | instance | `range_raw_start` | wrapped by `first` |
| `end_` | instance | `range_raw_end` | wrapped by `last`/`size` |
| `inclusive_` | instance | `range_raw_inclusive` | wrapped by `size`/`includes(_)`/`last` |

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

### 2.16 `Family` — `::` method-reference call router ([selectors.md §3](../selectors.md#3-method-references-), U16-Open, [ADR-0047](../../../adr/0047-amend-floor-admit-family-call-router.md))

A native `Object::Family` heap variant (no `Value::Family` arm — reached
through `Value::Obj` exactly as `Object::List` is), sitting directly under
`Object`. `obj::name`/`Type::name` build one bound to the receiver; `Family`
itself carries **no other selector** — every call shape misses its table by
construction and lands on the router below, which rebuilds the real selector
(`family`'s base name + the missed call's decoded labels) and re-dispatches
as an ordinary send. Open form only — the Pinned `recv::#sel(...)` form is
deferred to **U-LEX-HASH** (`#`-symbol-literal lexing); there is no
reflective surface (Q14 ruling, `open-questions.md`).

| Selector | Side | Native fn | Notes |
|---|---|---|---|
| `doesNotUnderstand(_)` | instance | `family_does_not_understand` | overrides `Object`'s default miss handler; the uniform call router |

## 3. The floor ↔ `core.ph` boundary

Two classes now carry `.ph` surface protocol self-hosted over the floor
([`core.ph`](../../../../phalcom-core/core/core.ph)):

**`List`** (ADR-0020) —

```
size    => self.length_
at(i)   { return self.at_(i) }
add(v)  { self.push_(v); return self }
each(f) { var i = 0; while (i < self.size) { f.call(self.at(i)); i = i + 1 } }
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
| `List#at(_, put)` (wrap `set_`) | primitive exists, unwrapped | U-STD |
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
   the census here (count = 117, after U-ANNOT-CONTRACTS's `__invariantEnter`/
   `__invariantExit` +2, ADR-0052 Fix 1). This turns silent floor drift into a
   red test; §1.1 is no longer a manual checksum.

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
