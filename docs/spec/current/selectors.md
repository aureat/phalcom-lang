# Phalcom — Selectors, Symbols, and Callable References

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

**Governing ADRs:**
[ADR-0012](../../adr/0012-selector-signature-encoding-and-dispatch.md) (label-encoded selectors and inline-cache-ready dispatch)

Scope: selector identity, `#` symbol literals, `&` callable references, `@`
attributes, field visibility. Supersedes `SignatureKind::Method(u8)`
(arity-only) in `phalcom-vm`.

---

## 1. Selector identity

A **selector** is the full identity of a method: its base name plus the argument labels, in declared order. It is interned to a single `Symbol` (`u32`) and is the sole key for method lookup — one hashmap hit, no overload resolution at dispatch time.

### Canonical form

```
move(_,to,duration)     // 1 positional, 2 labeled
move(_,_)               // 2 positional
size()                  // nullary
+(_)                    // binary operator
~()                     // unary operator
name=(_)                // named setter; RHS is a dedicated value lane
[_,debug]=(_)           // subscript setter; RHS is outside index slots
```

Grammar of the canonical string:

```
selector  := name "(" [ slot { "," slot } ] ")"
slot      := "_" | label
name      := ident | operator
label     := ident
```

Getter, setter, and subscript selectors are also canonical identities:

```text
name                      // Getter
name=(_)                  // Setter
[_,debug]                 // SubscriptGet
[_,debug]=(_)             // SubscriptSet
```

The setter `(_)` is not an ordinary selector slot. A named setter has no
ordinary slots, and a subscript setter contains only its bracket/index slots;
the assigned value is one distinguished value lane. Setter declarations MUST
therefore use exactly `=(_ local)` after the property or bracket shape.

### Slot escaping and transitional rest

Call-site Symbol labels use total reversible escaping before entering a
selector slot. Literal labels `_`, `*`, `**`, `***`, labels beginning with
`~`, delimiters, and Unicode labels cannot collide with structural slot
markers. The implementation's escaped forms include `#_`, `#*`, `#**`, and
`#***`; selector reconstruction must use the shared encoder, never splice
raw labels into comma form.

Before F.3, the valid U9 positional-rest declaration spelling is `name(*)`.
It intentionally loses fixed-prefix arity and is transitional. F.3 changes
core and LSP selector formatting together when structural rest identity lands.

### Rules

| Rule | Statement |
| --- | --- |
| **R1 — Labels are identity** | `move(_,to,duration)` and `move(_,_)` are *distinct* selectors and may both be defined on one class. |
| **R2 — Positionals precede labels** | `move(_,to,duration)` is legal. `move(to,_)` is **illegal** — no interior positionals. Validated at method-definition time. |
| **R3 — Label order is identity** | `move(to,duration)` ≠ `move(duration,to)`. Callers must supply labels in declared order. There is no keyword-argument reordering. |
| **R4 — No sorting/normalization of labels** | Follows from R3. Reordering at a call site would require knowing the callee's declared order, which is only known *after* dispatch — circular. Named after Swift, not Python. |
| **R5 — Arity is implied** | Arity = slot count. It is not stored separately. |

### Why R3/R4 are non-negotiable

Under dynamic dispatch the selector must be computable from the call site alone. Any scheme that normalizes label order requires callee knowledge before dispatch. This is a structural consequence of choosing labels-as-identity, and is intentional.

---

## 2. Symbol literals (`#`) — **IMPLEMENTED** (U-LEX-HASH)

Two distinct value types, both backed by an interned `Symbol`:

| Literal | Type | Meaning | Used for |
| --- | --- | --- | --- |
| `#move` | **Name symbol** | A bare method name; identifies a *family*, not a method. | `respondsTo`, map keys, reflection queries |
| `#move(_,to,duration)` | **Selector symbol** | A complete method identity. | `perform`, selector-based reflection |
| `#+`, `#==`, `#&`, `#~` | Selector symbol | Operator selectors (`#~` is nullary). | same |

`perform` accepts **only** selector symbols. Passing a name symbol is a type error. (`perform` itself is not yet implemented — U-LEX-HASH lexes and interns both symbol shapes only.)

**Implementation note (U-LEX-HASH):** `#[]` (the bracket-subscript operator
selector) is **not yet lexed** — the language has no user-facing `[](...)`
method-definition syntax to fix its arity/canonical-form convention against
(ADR-0016's hand-written parser doesn't parse a subscript method name at all
yet), so it is deferred rather than guessed at. See `DEFERRED.md`. Every other
row in the table above is implemented.

### Lexing

Lexed as a **single atomic token** by the hand-written scanner (ADR-0016;
`phalcom-ast::lexer`) — the grammar below is unchanged from the original
Logos-era design, only the implementation strategy moved:

```rust
#[regex(r"#[a-zA-Z_][a-zA-Z0-9_]*(\([^)]*\))?", callback = canon_selector)]
```

with a separate branch for operator selectors.

**Whitespace rules:**

- **Outside the parens: adjacency is required.** `#`, the name, and `(` must be contiguous.
  - `# move` — not a symbol.
  - `#move (a, b)` — `#move` (name symbol) followed by a parenthesized expression.
  - This is what prevents the ASI hazard where `#move` on one line greedily eats `(a + b)` on the next.
- **Inside the parens: whitespace is free.** Spaces and newlines are permitted and are stripped.

```
#move(_, to, duration)
#move(_,to,duration)
#move(
  _,
  to,
  duration
)
```

All three intern to the same `u32`.

**Canonicalization happens at intern time.** The lexer callback strips whitespace, validates R2, and interns. The canonical spelling is the no-space form. Malformed contents (e.g. `#move(to,_)`) are a **lex-time error** with a precise span.

### Reserved sigil interactions

| Construct | Resolution |
| --- | --- |
| Shebang `#!/usr/bin/env phalcom` | Special-cased in the lexer: `#!` is skipped **only at byte offset 0**. |
| JS-style private fields (`this.#x`) | **Not adopted.** See §5. `#` means "symbol" and nothing else. |
| Attributes / decorators | Spelled with `@`, not `#`. See §4. |
| Comments, interpolation, numerics | No conflict. |

---

## 3. Callable references (`&`), selector patterns, and associated lookup

`&` introduces a callable reference. A dot reference binds an ordinary
receiver, including a class object:

```text
&receiver.name
&receiver.name()
&receiver.name(_)
&receiver.name=(_)
&receiver.name=
&receiver.name(...)
&receiver.name...
&receiver[_, debug]
&receiver[_, debug]=(_)
&receiver[...]
&receiver[...]=(_)
&receiver[...]=
```

The reference names the selector shape that is written. A bare named reference
is an exact Getter; `name()` and `name(_)` are exact Methods; `name=(_)` is an
exact Setter. A trailing `=` selects named accessors, `name(...)` is a
Method-only pattern, and `name...` captures the complete named family
(Getter, Setter, or Method). Bracket references follow the same rule for
SubscriptGet and SubscriptSet; `[...]` and `[...]=(_)` are structural patterns,
while `[...]=` selects both subscript accessor kinds.

The receiver expression is evaluated once and stored. Construction never
probes receiver behavior and never rejects an absent selector. The exact
Getter capability is activated through `get()` or `value`, not ordinary `()`;
ordinary `()` remains Method-kind.

`::` is reserved for declaration-associated lookup. Associated callable
families are selected with `&`:

```text
&Owner::member
&Owner::member(_)
&Owner::member(...)
```

Direct associated invocation remains `Owner::member(args)`, while class-side
behavior is an ordinary dot send such as `Foo.bar(args)`. There is no runtime
receiver fallback from an associated lookup to a bound behavioral family.

The selector specification preserves labels as labels, not destructuring
bindings. Operators retain Method identity, so `&receiver.+(_)` is an exact
operator Method and `&receiver.+...` is its complete named family.

### Selector-pattern grammar and laws

    selector_spec := exact_selector | pattern_selector | accessor_pattern
    exact_selector := name | name "(" [ slot { "," slot } ] ")" | name "=(_)"
    pattern_selector := name "(" pattern_slots ")" | name "..."
    accessor_pattern := name "="
    pattern_slots := slot { "," slot } [ "," "..." ] | "..."
    slot := "_" | label

The concrete parser accepts its canonical `...` gap spelling and preserves
fixed prefix/suffix slots around that gap. Matching requires the same base and
kind, exact kind for an exact pattern, and ordered slot equality for every
fixed prefix/suffix slot. `AnyNamed` patterns match named getters, setters,
and methods; `NamedAccessors` matches only Getter and Setter; `AnySubscript`
matches only SubscriptGet and SubscriptSet. Pattern matching is a predicate,
not a dispatch key, and does not reorder labels or invent positional slots.

For indexed assignment, the receiver and every index expression are evaluated
left-to-right, followed by the setter RHS. The assignment expression evaluates
to the original RHS, independently of the setter body's return value. The RHS
is never appended to the ordinary bracket slot list, so the normal
positional-before-label rule remains unchanged.

### Call and mutation laws

At call time an exact Family derives the selector from its stored identity,
while a pattern Family derives a candidate selector from its predicate and
incoming shape. The selected Method is then activated through current
ordinary dispatch; the bound receiver does not participate in route selection.

    let family = &object.render(_)
    // Replacing object/render(_) after this line changes the next family call.
    family(value)

    let family = &object.render(...)
    // Adding/replacing matching methods after construction affects the next call.
    family(value)

A pattern Family applies the current caller's access authority during ordinary
dispatch. `MethodFamily#selectors`, `size`, and `methodFor(_)` belong to the
separate immutable snapshot returned by `Behavior#>>(pattern)`; a missing
Family route reaches ordinary `doesNotUnderstand(_)` at call time. No
empty-family reference-time error exists.

The old `MethodRefKind::Open`/`Pinned` split, bound `::` references,
string-punctuation heuristic, and empty-family construction rule are retired
and superseded by `&` references with exact-selector versus structural-pattern
normalization.

## 4. Attributes (`@`)

`@` is **reserved for attributes/decorators**. Attributes compile to ordinary method-table entries — they are macros over the method table, not new machinery.

| Attribute | Target | Effect |
| --- | --- | --- |
| `@construct` | class header | Derives a constructor from the declared fields ([`@construct`](decorators/construct.md)). |
| `@constructor` | method member | Marks the method a constructor ([`@constructor`](decorators/constructor.md)). |
| `@class` | method / getter / setter / field | Declares the member on the **class side**: a method installs on the metaclass, a field stores on the class object ([Classes §2.1](classes.md)). |
| `@get` | field | Derives an accessor method for a field. |
| `@set` | field | Derives a mutator method for a field. |

```
@construct
class Point {
  var _x
  var _y
  @get var _label          // derives label()
  @get @set var _color     // derives color() and color(_)
}
```

Per-field escape hatches (e.g. `@get(priv)`) fit without a grammar change.

`@construct` is class-only. It derives a constructor from declared fields, in
declaration order, with labels stripped of the leading underscore (`_x` → `x:`).
`@constructor` is method-only. It marks the method that performs construction.
`@class` is target-polymorphic for class-side fields and methods. These are three
separate placement/meaning decisions ([PDR-0028](../../pdr/0028-class-and-constructor-decorator-canon.md)).

---

## 5. Field visibility

**Fields are always private. There is no visibility syntax.**

Instance variables are reachable only from inside the class. Everything outside goes through a message. Consequences:

- `obj.x` is **unambiguously a send** in every position — one dispatch path, uniform inline caches, no second lookup mechanism.
- Consistent with "everything is a message."
- Exposure is opt-in, via derived accessors (`@get` / `@set`), so fields are invisible until deliberately published.

Explicitly **rejected**: JS-style `#field` privates (would give `#` two meanings), and `pub`/`priv` modifiers (would give field access two lookup paths).

---

## 6. Current implementation boundary

| Current | Status |
| --- | --- |
| Interned selector Symbols | Canonical exact identity, ordered positional/labeled slots, and selector kind. |
| `MakeFamily` | Stores the evaluated receiver plus exact Symbol or immutable SelectorPattern specification. |
| Family call gateway | Derives exact shape or matches a live structural predicate, then activates the selected Method directly. |
| MethodFamily | Immutable exact map plus captured compatible rest chain; inaccessible routes are omitted during capture. |
| Compiler specialization | Associated exact references may lower to a resolved target; bound references retain `MakeFamily` semantics. |
| Old Open/Pinned and empty-family rules | Retired; they are not compatibility semantics. |

---

## 7. Open questions (not decided)

These were raised and deliberately deferred. They are **not** part of this spec.

1. **`var x` defaulting to `None`.** If uninitialized variables are `None`, every variable is effectively `T | None` and `nil` returns under a new name. Alternative: a VM-only `Uninit` sentinel that traps on read, keeping `None` meaningful as a *chosen* absence.
2. **`ifTrue` / `ifFalse` returning `Option`.** Chaining is unsound: `cond.ifTrue { a }.ifFalse { b }` sends `ifFalse` to an `Option`, not a `Bool`; and `ifTrue { None }` is indistinguishable from the branch not being taken. A paired `ifTrue(_)ifFalse(_)`-style selector as primary, with single-branch forms as `Option`-returning sugar, resolves both.
3. **Default arguments.** Largely incompatible with selector-identity dispatch: a call omitting a defaulted argument produces a *different* selector, so lookup misses. Options are arity-family expansion (combinatorial) or static callee knowledge (unavailable). **Decide before shipping** — retrofitting is expensive.
4. **`Option` bootstrap.** If `Option` is a plain stdlib class and fields default to `None`, constructing `None` requires a class whose fields default to `None`. `Option` likely needs to be VM-blessed / niche-encoded in `Value`, which also removes an allocation from every optional.
