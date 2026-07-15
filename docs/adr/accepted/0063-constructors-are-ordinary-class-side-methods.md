# 63. Constructors are ordinary class-side methods: `@constructor`/`@static` decorators, `new_` allocator

- Status: **Accepted** (ratified by the user 2026-07-15, DEC-CTOR-B "ratify in full")
- Date: 2026-07-15
- Rulings folded in: **DEC-CTOR-A** — `@static` does *not* extend to fields; static
  fields get their own decorator, **`@classField`** (§2.1, DEC-CTOR-A1) ·
  **DEC-CTOR-A2** — per-class static-field storage is working-as-designed; fixture it ·
  **DEC-CTOR-C** — the §5 desugar is measurement-gated, not ratified here ·
  **DEC-CTOR-E** — `@construct` unifies into `@constructor` ·
  **DEC-CTOR-D** — one-shot codemod, no deprecation window
- Related: [ADR-0002](../accepted/0002-metaclass-tower-parallel-rule.md) (the parallel
  tower is the mechanism this ADR stops opting out of) ·
  [ADR-0012](../accepted/0012-selector-signature-encoding-and-dispatch.md)
  (`SignatureKind::Initializer` is retired here) ·
  [ADR-0019](../accepted/0019-freeze-vm-blessed-primitive-floor.md) (**this is a floor
  amendment** — see §7) · [ADR-0043](../accepted/0043-no-default-arguments-keep-selector-identity-pristine.md)
  (selector identity stays pristine; nothing here varies effective arity) ·
  [ADR-0061](0061-underscore-prefix-reservation-fields-internals-reserved.md)
  (Proposed — reserves *leading* `_`; `new_` is trailing and disjoint, see
  Consequences) · [ADR-0011](../accepted/0011-static-instance-slot-layout.md)
  (**miscited** by the current constructor code — see Context)

## Context

### The `(ADR-0011)` citations in the constructor code are wrong

`class_decl.rs` and `expr.rs` cite ADR-0011 for the constructor *alias* mechanism.
ADR-0011 is *static-instance-slot-layout*; it specifies field slots and says nothing
about constructor installation. **No ADR has ever specified how constructors install.**
The overlay repeats the miscitation (`| Instance layout / construct | … `construct`
keyword on metaclass | ADR-0011, classes §1–2 |`). This ADR is the missing record.

### `classes.md` §1 describes a language Phalcom does not implement

Three of its claims are false against the tree today:

| `classes.md` §1 claim | Reality |
|---|---|
| "There is no user-visible allocator" | `Object class >> new()` is public and 80% of `.ph` classes (532/666) rely on it |
| "no implicit zero-arg `new`" | that *is* the bare allocator, reachable on every class |
| "`new` is not special — `construct anonymous()` is equally legitimate" | aspirational; two compiler sites hardcode `"new"` |

The spec is not merely stale here — its **normative direction was never built**, and
the shipped design is the opposite one. Ratifying this ADR is a deliberate reversal of
that direction, not a doc fix. See "Alternatives considered".

### The alias bug, and why it was invisible for so long

Until 2026-07-15, `construct new(x)` installed on the metaclass under the *prefixed*
selector `"init new(_)"`, which **no call site encodes**. A compile-time table rewrote
`Foo.new(1)` → `init new(_)` — but only when the receiver was a bare identifier. Every
other receiver shape (`var C = Foo; C.new(1)`, `M.Foo.new(1)`, `list[0].new(1)`) kept
the plain selector, walked past the constructor, and hit the inherited bare allocator,
**silently returning an uninitialized instance**.

The kernel never had this bug: `List.new()`, `Map.new()`, `Range.new(_,_,_)` register
under plain `Method` selectors and resolve through the tower from any receiver. The
alias was user-constructors opting out of ADR-0002's tower for no stated reason.

A partial fix landed 2026-07-15 (install under the ordinary selector). This ADR
records the *design* that fix was reaching for, and closes what it left open.

### `new` is special for exactly one reason

Two sites in the whole compiler hardcode the string:

| Site | Purpose |
|---|---|
| `class_decl.rs:629` `if construct_def.name == "new"` | sets `has_new_construct` — the bare-allocator-drop rule |
| `expr.rs:103` `method_call.method == "new"` | the arity guard |

Both exist because **`new` is the one constructor name the tower root already
occupies**. Named constructors have no collision: nothing named `at()` sits at the
root, so `Ref.at(1,2)` either finds the constructor or raises `doesNotUnderstand`.
This is why named constructors failed *loudly* under the alias bug while `new` failed
*silently* — and why the reported symptom was always about `new`.

**Named constructors are already load-bearing**: 8 of 148 constructor declarations use
a non-`new` name, including `Future`'s `construct value(v)` / `construct error(e)` in
`core.ph` itself, plus `Ref.at`/`Ref.full`, `RefRange.fromTo`, `Cell.of`,
`Countdown.from`, and `Point2.named` — whose fixture asserts "Both matching-arity
`new` and named ctors inherit."

### The keywords buy nothing

`construct` and `static` change **zero grammar**. `construct new(x) {}` and `new(x) {}`
parse identically after the prefix; `static` is a pure modifier bit. Both are member
*metadata* — exactly what `@` attributes exist for (`selectors.md` §4: "attributes
compile to ordinary method-table entries … not new machinery").

They actively cost. `parse_attribute` ([`parser.rs:1046`](../../../phalcom-ast/src/parser.rs))
special-cases `Token::Construct` *purely* because the `@construct` attribute name
collides with the keyword:

```rust
let name = if self.eat(&Token::Construct) { "construct".to_string() } else { … };
```

And `ConstructDef` is the only `ClassMember` with **no `attributes` field**, so
`attach_attributes` ([`parser.rs:1113`](../../../phalcom-ast/src/parser.rs)) raises
`attr.dangling: attributes cannot be attached to a constructor`. Contracts on a
constructor are unrepresentable today.

### The floor carries a duplicated allocator

`class_new` ([`primitive/class.rs:107`](../../../phalcom-core/src/primitive/class.rs))
and `object_class_new` ([`primitive/object.rs:105`](../../../phalcom-core/src/primitive/object.rs))
are **byte-identical**:

```rust
let class_id = expect_class(vm, receiver)?;
let field_count = vm.heap.class(class_id).field_count;
let instance = InstanceObject::new(class_id, field_count);
Ok(Value::Obj(vm.heap.alloc(Object::Instance(instance))))
```

Registered twice — `Object class >> new()` (`primitive_static!`, line 47) and
`Class >> new()` (`primitive!`, line 100). For any user class the metaclass chain
reaches `Object class` before `Class`, so **`object_class_new` always wins and
`class_new` is dead for user classes**. Two floor primitives, one job.

## Decision

### 1. `construct` and `static` cease to be keywords

`Token::Construct` and `Token::Static` are deleted, along with their `scan_keyword`
rows. Both lex as `Token::Identifier`. `construct` becomes a legal method name and
variable.

Migration ergonomics come from a **contextual recovery diagnostic**, not reservation:
in class-member position, an identifier `construct`/`static` followed by another name
token raises

> `member.legacy_keyword: 'construct new(x)' is no longer valid syntax; use the '@constructor' decorator on the member`

### 2. `@constructor` and `@static` are the surface

```phalcom
class Point {
  @constructor
  new(x, y) { _x = x  _y = y }

  @static
  origin() { return Point.new(0, 0) }
}
```

`@constructor` is **target-polymorphic**, replacing the old `@construct`:

| Target | Meaning |
|---|---|
| Class header | derive a constructor from declared fields (today's `@construct`) |
| Method member | this method is a constructor |

One registry row, `legal_targets() = &[Target::Class, Target::Method]`. This is
deliberate: `@construct` (class-header derive) and `@constructor` (member marker) as
two names one character apart, with unrelated meanings, is a trap. `Target::Construct`
is deleted with `ConstructDef`.

`@static` is legal on `Method`/`Getter`/`Setter` only. `@static @constructor` together
is an error, not a redundant no-op.

### 2.1 `@classField` — static fields are a different concept, not a `@static` target

**DEC-CTOR-A/A1.** `static _count = 0` is not a method; it is per-class *storage*
(ADR-0017's `static_slots`). It gets its own decorator:

```phalcom
class Counter {
  @classField var _count = 0

  @static
  bump() { _count = _count + 1 }

  @static
  count => _count
}
```

`@classField` is legal on `Field` only; `@static` is not legal on `Field`.

The split is not bureaucratic — **`@static` would teach the wrong model.** Measured
behavior:

```phalcom
class Base { @classField var _count = 0
             @static bump() { _count = _count + 1 }
             @static count => _count }
class Derived extends Base {}

Base.bump()   Base.bump()
Base.count      // 2
Derived.count   // None   <- its own slot, not Base's
```

A subclass gets a **fresh, unset slot**, exactly as ADR-0011 specifies for instance
fields ("a subclass that writes `_name` gets its own new slot; it does not touch the
superclass's") — ADR-0017 is that rule shifted one tower level up, and it means what
it says. In Smalltalk terms this is a **class-instance variable**, *not* a class
variable.

So the two obvious names are both wrong: `@shared` asserts hierarchy-wide sharing that
does not exist, and `@classvar` names Smalltalk's *class variable*, which **is**
hierarchy-shared. `static` carries the same false connotation from Java/C#. `@classField`
says what it is — a field, obeying the field rule, on the class side.

**DEC-CTOR-A2 — the sharp edge is working as designed.** An inherited `@static` method
touching a `@classField` reads `None` in the subclass:

```phalcom
Derived.bump()   // None does not understand '+(_)'
```

This follows from per-class storage and is **ratified as correct**, not patched. It is
untested today (`inheritance_static_*.ph` all cover static *methods*) and unruled by
ADR-0017, whose DEC-D settled only storage. This unit locks it with a golden fixture
and documents it in `classes.md`. Re-running the declaration's initializer per subclass
was considered and rejected: it would diverge from instance fields, which read `None`
until written, and buy a new per-class initializer-evaluation path.

### 3. The two decorators are different *kinds* of attribute

This is structural, forced by a constraint already in the code:
`AttributeExpander::expand(&mut ClassMember)` mutates **one member in place** and
cannot append siblings — which is exactly why `ConstructExpander`/`GetExpander` are
deliberate no-ops whose real derive runs from `expand_class_attributes`.

| | `@static` | `@constructor` |
|---|---|---|
| Kind | **modifier** | **derive** |
| Real work | `expand()`, in place: `is_static = true` | `derive_constructor` from `expand_class_attributes` |
| Registry row exists for | doing the work | `attr.unknown`/`attr.illegal_target` only |
| Members in → out | 1 → 1 | 1 → **2** |

### 4. `ConstructDef` collapses into `MethodDef`

`ClassMember::Construct` and `ConstructDef` are deleted. A constructor is a
`MethodDef` with two bits:

```rust
pub struct MethodDef {
    pub name: String,
    pub params: Vec<ParameterDef>,
    pub body: Vec<Statement>,
    pub is_static: bool,       // set by @static / @constructor, never the parser
    pub is_constructor: bool,  // set by @constructor
    pub attributes: Vec<Attribute>,
    …
}
```

The parser never sets either bit. Attribute expansion does — which is the contract
attributes already have ("desugar into ordinary AST before the rest of compilation
runs"). `attach_attributes`'s constructor arm and its `attr.dangling` error are
deleted; **contracts on constructors start working as a side effect**.

### 5. `@constructor` desugars to two ordinary methods

```phalcom
@constructor
new(x, y) { _x = x  _y = y }
```

expands to:

```phalcom
@static
new(x, y) {
  let instance = self.new_()
  instance.«init new»(x, y)
  return instance
}

«init new»(x, y) {
  _x = x
  _y = y
  return self
}
```

`«…»` is **notation in this document, not grammar**. The real selector is the string
`init new(_,_)` — name `init new`, two positional slots. `parse_method_name` reads one
identifier token, so a name containing a space is **undeclarable and unoverridable**
in source. The init name derives from the constructor's: `@constructor zero()` →
`init zero`, so named constructors never collide.

This is the retired `init ` alias **relocated to the correct side of the tower**. It
was always the right idea for an *instance-side initializer* and always wrong for a
*class-side constructor*.

What the desugar buys:

- `self` in a constructor body is an instance **because it is an instance method**.
  The current contradiction — instance `self`, class-side dispatch identity — is gone.
- `super.new(x)` inside a `@constructor` body rewrites to `super.«init new»(x)`, an
  ordinary instance-side super-send. **The super-construct metaclass hop in
  `vm/dispatch.rs` and its `SignatureKind::Initializer` gate are deleted.**
- Class-side `new(x)` is a genuinely ordinary static method, so it dispatches from any
  receiver — not because it was fixed, but because there is nothing to fix.

### 6. `new_` is the sole primitive allocator

`Class >> new_()` — arity 0, uninitialized instance, **reserved**: declaring `new_` in
a user class is `selector.reserved_name`.

`class_new` is renamed to `class_new_` and registered **once**, on `Class`,
instance-side (`primitive!`) — the `Behavior >> basicNew` position, reachable from
every class object through the tower. `object_class_new` and its
`Object class >> new()` registration are **deleted as a duplicate**.

`Class >> new()` becomes ordinary Phalcom in `core.ph`:

```phalcom
class Class {
  new() => self.new_()
}
```

A **default at the tower root**, shadowed by ordinary lookup like anything else. No
VM special case.

Naming: trailing `_` marks the primitive floor version of a selector, unoverridable.
`new_` is *public API* — user-written constructors call it — which is why it is not
`_$new` under ADR-0061's internals prefix. See Consequences.

### 7. The arity hole closes with a tombstone, not a guard

> **Rule.** A class declaring **any** class-side `new` of any arity tombstones the
> inherited arity-0 `new()`, unless it declares `new()` itself.

The tombstone is a **real method** installed on the metaclass at class-definition
time, so ordinary lookup finds it before the root default **from any receiver shape**:

```
Error: Point.new() requires arguments. Candidates: new(_, _), new(_)
```

Zero dispatch cost, zero runtime chain-walk, and it lists candidates. This is
Smalltalk's `self shouldNotImplement` on `new`. The rule is keyed on `new` and
**should be** — it is a rule about one root default, not about constructors. Named
constructors need no tombstone because nothing shadows them.

The existing compile-time name-keyed guard stays **on top**: a compile error beats a
runtime error when the receiver is statically a class. The guard becomes an
optimization over a structurally sound runtime rather than the only defense.

### 8. `class.duplicate_selector` replaces the construct/static collision error

> Two members of one class body may not install the same selector on the same side
> (instance or class-side), regardless of decorators.

`@constructor new(x)` + `@static new(x)` both install class-side `new(_)` after
expansion, so the error **falls out of the desugar** with no constructor-specific rule.
It is a duplicate definition — the same species as `foo(){} foo(){}` — not a shadowing
rule. This also catches `@static new(x)` declared twice, which nothing catches today.

Runs on the **post-expansion** member list. Derived members must carry provenance back
to their source member, or the message points at compiler-generated AST instead of:

```
Error: `@constructor new(x)` and `@static new(x)` in class 'Foo' both define
       the class-side selector `new(_)`; rename one.
```

Intra-class only. A **subclass** `@static new()` shadowing a parent's
`@constructor new()` stays legal and silent — that is an override, and overriding is
the point of a hierarchy (Smalltalk's `Point class >> new` does exactly this).

### 9. `SignatureKind::Initializer` is retired

Required, not optional. The instance-side init must install as `Method`, but:

- `encode_selector("init new", labels, Method(2))` → `"init new(_,_)"`
- `decode_selector("init new(_,_)")` → `SignatureKind::Initializer(2)`

Same string, two kinds — and `decode_selector` documents itself as "the exact inverse
of `encode_selector`". The `Initializer` arms of both functions are deleted, after
which `init ` has no meaning in selector-land and the string decodes as a plain
`Method`. Safe: `attributes.rs` uses `Initializer` only for a self-consistent
derived-vs-handwritten comparison, and `gen-core-table`'s `kind:"construct"` output
has no live consumer.

## Consequences

### This is an ADR-0019 floor amendment — and the floor **shrinks**

`floor-census.md` is normative: "Any change to the set below is an ADR-0019 amendment,
not an ordinary commit." Precedent for a naming-shaped amendment is
[ADR-0062](../accepted/0062-amend-floor-admit-string-raw-byte-accessors-supersedes-0049-naming.md).

| Change | Δ fns | Δ bindings |
|---|---|---|
| `object_class_new` deleted (duplicate of `class_new`) | −1 | −1 |
| `class_new` → `class_new_`, `Class >> new()` → `Class >> new_()` | 0 | 0 (rename) |
| `Class >> new()` re-homed to `core.ph` as `new() => self.new_()` | 0 | 0 (derivable, not floor) |
| **Net** | **−1** | **−1** |

**`R-INV-0.1` (`tests/invariants.rs:616`) will go red** — it reconstructs the floor
from a live `VM::new()` and asserts exact selector strings. Updating it is part of the
unit, and the census tables must be edited in the same pass. Per the census's own rule,
do not quote a floor total from this ADR — recount from the census.

### Bootstrap order is safe, but it is a real gate

Moving `Class >> new()` into `core.ph` is the *primitive/library boundary ⊗ bootstrap
order* hazard. Analysis: `class Class {}` sits at `core.ph:34`, and every kernel class
(`List`, `Map`, `Range`, `String`, `Number`, `Bool`, `Module`) carries its **own**
`primitive_static!` `new`, so none depends on the root default. Nothing before line 34
bare-allocates. **Verify, do not assume** — the unit gates on a green bootstrap plus
`verify_invariants()`.

### Cost: +1 send per construction — the one open risk

The desugar replaces one fused constructor call with a class-side send plus an
instance-side init send. Construction is a hot path and allocation is the #1 measured
mechanism ([[perf-baseline-measured]]). Per ADR-0051 (measure-first), **U-CTOR-5 is
gated on a construction benchmark**, and if the hit is real and `inliner.rs` does not
fold the hop, the fallback is to keep the fused single-method constructor and the
`Initializer` gate. Decisions 1–4 and 6–8 do not depend on this and land regardless.

### `new_` and ADR-0061 are disjoint — deliberately

ADR-0061 (Proposed) reserves **leading** `_`: `_name` fields, `_$name` internals,
`__name` reserved, with `parse_method_name` rejecting any leading `_`. `new_` is
**trailing** and untouched by every rule in that ADR.

The near-miss is conceptual: `_$` is the established marker for "not user-definable."
`new_` is *not* an internal — user-written constructors call it, exactly as
`Object#perform` is public reflection. It is public floor API, so `_$new` would be the
wrong prefix and would additionally grant it implicit-`self` sugar it does not want.

Consequence: making `new_` non-definable needs a **name-keyed** ban, since ADR-0061's
is prefix-keyed. That is a new, small check — recorded here so it is not mistaken for
free.

### Reflection still reaches the hidden init — by construction, not oversight

`Object#perform(_,_)` sends a selector built from an arbitrary string, so
`p.perform(Symbol.new("init new(_,_)"), [3, 4])` finds the init and re-runs it on a
live instance. `decode_selector` is deliberately **total** so `Symbol.new("garbage")`
cannot crash the VM; therefore *no* mangling makes anything unreachable.

This is acceptable and consistent: it re-runs an init body on an already-live object —
no allocation, no memory unsafety, reachable state that setters already expose.
**Phalcom has no privacy anywhere**, and ADR-0061 makes the identical concession
("the ban is a footgun guard, not a capability boundary"). The mangle buys
non-declarability and non-overridability, never privacy. Do not read it as a boundary.

The `#`-literal path cannot spell it — the lexer requires `(` adjacent to the name, so
`#init new(_,_)` lexes as `#init` plus separate tokens. `Symbol.new` is the only route.

### Migration is mechanical but wide

148 `construct` declarations across 94 files, 152 `static` across 62, including 28 in
`core.ph`. One-shot codemod, no deprecation window: Phalcom owns 100% of its corpus,
and a dual-syntax window doubles parser surface to buy nothing. The codemod cannot see
`.expected` files quoting old syntax, or the 4 LSP `Construct` references.

### Docs this obsoletes

- `classes.md` §1 — reversed (see Alternatives).
- `selectors.md` §4 — `@construct` listed as "Planned"; it ships. Renamed here.
- `implementation-status.md` row 5 — already stale independently ("No `construct`
  token/node", "`ClassMember` = Method/Getter/Setter only" — both false today).
- The overlay's `| Instance layout / construct |` row — drop the ADR-0011 miscitation,
  cite this ADR.
- `class_attribute_construct_get_set.ph` is `status: PENDING` while live and passing.

### What this must not preclude (P4)

- **Niche/NaN-boxing (ADR-0044/0010).** Untouched — no new `Value` arm, no tag.
- **Inline caches (U-IC).** Improved: constructors become ordinary monomorphic sends
  at ordinary call sites. The tombstone installs pre-instance (ADR-0053's condition),
  so no epoch bump is needed.
- **ADR-0043 (no default args).** Nothing here varies effective arity; the tombstone is
  arity-0-keyed and adds no arity family.
- **ADR-0026/0041 (sealed reparenting).** The tombstone is computed at class-definition
  time and would need recomputation under reparenting — already sealed, so moot.
- **A future real capability boundary.** Retiring `Initializer` removes a *kind*, not a
  hook; a dispatch-level boundary (ADR-0061's deferred idea) stays open.

## Alternatives considered

- **Keep the keywords, fix only the selector.** Rejected: leaves the `parse_attribute`
  hack, leaves `ConstructDef` attribute-less (contracts on constructors stay
  impossible), and leaves two member kinds that compile to the same thing.
- **Keep `classes.md` §1's "no user-visible allocator" direction.** Rejected, and this
  is the ADR's most consequential reversal. That direction requires the constructor to
  be a primitive language concept with its own allocation opcode — precisely the
  special-casing that produced the alias bug. Smalltalk's `basicNew` is public for a
  reason: it makes construction *ordinary code*, so the tower does the work instead of
  the compiler. The corpus already votes this way — `person3.ph` hand-writes
  `let instance = self.new(); // super.new()`, wanting an allocator it had no name for,
  and getting infinite recursion instead.
- **No root `new()` at all; `new_` is the only allocator.** Cleaner and dead on
  arrival: 532 of 666 `.ph` classes rely on the bare allocator.
- **`_$new` instead of `new_`** (ADR-0061's internals prefix). Rejected: `new_` is
  public API called from user constructors, not plumbing, and `_$` would grant
  implicit-`self` sugar that is wrong for an explicit-receiver allocator.
- **A visibility/side bit on the method record** instead of name-mangling the init.
  Rejected for v0.2: it is the principled version, but it costs a field on every method
  plus a lookup check, and the only thing it buys over an undeclarable name is privacy
  Phalcom does not have anywhere else. Adding privacy to one hidden method while
  `perform` opens everything else is a wart, not a fix.
- **Dual-syntax deprecation window.** Rejected: see Migration.
