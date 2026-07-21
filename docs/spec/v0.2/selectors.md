# Phalcom — Selectors, Symbols, and Method References

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

**Governing ADRs:**
[ADR-0012](../../adr/0012-selector-signature-encoding-and-dispatch.md) (label-encoded selectors and inline-cache-ready dispatch)

Scope: selector identity, `#` symbol literals, `::` method references, `@`
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
```

Grammar of the canonical string:

```
selector  := name "(" [ slot { "," slot } ] ")"
slot      := "_" | label
name      := ident | operator
label     := ident
```

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
| `#move(_,to,duration)` | **Selector symbol** | A complete method identity. | `perform`, pinned method refs |
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

## 3. Method references (`::`) — **PARTIALLY IMPLEMENTED** (U16-Open: Open bound form; U16-Pinned: Pinned bound form; unbound `Type::name`/`Type::#sel(...)` deferred)

`::` produces a **Family** — a callable value. Two forms:

```
obj::move                     // Open family   — receiver bound, name only
obj::#move(_,to,duration)     // Pinned family — receiver bound, selector fixed
Point::move                   // Open, unbound   — receiver is the first argument
Point::#move(_,to,duration)   // Pinned, unbound
```

Grammar is LR(1)-clean: after `::`, peek for `#`.

### Representation

```rust
enum Family {
    Open   { recv: Option<Value>, name: Symbol },      // obj::move
    Pinned { recv: Option<Value>, selector: Symbol },  // obj::#move(_,to,duration)
}
```

`recv: None` = unbound.

### Semantics

**Open families resolve at call time, not at reference time.** The call site knows its own labels statically, so the selector is built from `family.name` + the call's label suffix, and then dispatched as an ordinary send.

```
let f = obj::move
f(to: p, duration: 2)    // dispatches move(to,duration)
f(p, duration: 2)        // dispatches move(_,duration)
f(p, 2)                  // dispatches move(_,_)
```

This means an Open family is **never stale** — the method table is consulted on every call — and an unbound `Point::move` dispatches on the *actual receiver passed in*, so subclass overrides work correctly.

**Pinned families** have their selector fully known at compile time. No re-interning; straight to the send. This is the fast path and the way to name one specific overload.

**A family call *is* a send.** There is no second dispatch mechanism:

```mermaid
flowchart TD
    accTitle: Family Call Resolution
    accDescr: An open family call builds its selector from the family name and the call site labels, then enters the ordinary send path. A pinned family skips straight to the send. A lookup miss becomes a normal doesNotUnderstand, enriched with the family's candidate list.

    call["📞 family(args...)"]
    kind{"Open or Pinned?"}
    build["Build selector:<br/>name + call-site labels"]
    use["Use fixed selector"]
    send["✉️ Ordinary send(recv, selector, args)"]
    hit["✅ Method found — invoke"]
    dnu["⚠️ doesNotUnderstand<br/>(error enriched with family candidates)"]

    call --> kind
    kind -->|Open| build
    kind -->|Pinned| use
    build --> send
    use --> send
    send --> hit
    send --> dnu

    classDef entry fill:#dbeafe,stroke:#2563eb,stroke-width:2px,color:#1e3a5f
    classDef step fill:#f1f5f9,stroke:#64748b,stroke-width:1px,color:#0f172a
    classDef good fill:#dcfce7,stroke:#16a34a,stroke-width:2px,color:#14532d
    classDef bad fill:#fef9c3,stroke:#ca8a04,stroke-width:2px,color:#713f12

    class call entry
    class kind,build,use,send step
    class hit good
    class dnu bad
```

### Performance

An Open call costs the same as a normal send: the call site emits a constant label-suffix, and a monomorphic inline cache keyed by `(call_site, class_id)` collapses the intern step after the first hit.

### Error behavior

| Situation | Behavior |
| --- | --- |
| **Empty family** — `obj::typo` where no method named `typo` exists | Error **at reference time**, naming the class. Checked against a per-class base-name index (§3.1). |
| Empty family, but class defines `doesNotUnderstand` | **Not an error.** The family is callable and routes to the DNU hook. The check is `empty && no DNU hook`. |
| **Call-time miss** — labels match no member of the family | Ordinary `doesNotUnderstand`, with the error enriched by the family's candidate list. |
| Call-time miss where the supplied labels are a strict subset of exactly one candidate | Report the *specific* missing label (`missing label 'duration'`) rather than dumping all candidates. |

### 3.1 Base-name index

Built per class at class-finalization time, flattened through inheritance like a vtable:

```rust
base_names: HashMap<Symbol /* "move" */, SmallVec<[Symbol; 2]> /* full selectors */>
```

Serves three purposes: the empty-family check, the candidate list in error messages, and reflection.

**Implementation note (U16-Open, U16-Pinned):** the Open empty-family check
queries this index directly (`ClassObject::responds_to_base_name`). A
Pinned reference's empty-family check does **not** use this index — a
Pinned selector is already the exact target identity, so the check is an
ordinary hierarchy lookup for that one selector
(`class::lookup_method_in_hierarchy`), same DNU-override exemption as Open.

---

## 4. Attributes (`@`)

`@` is **reserved for attributes/decorators**. Attributes compile to ordinary method-table entries — they are macros over the method table, not new machinery.

| Attribute | Target | Effect |
| --- | --- | --- |
| `@constructor` | class header | Derives a constructor from the declared fields. |
| `@constructor` | method member | Marks the method a constructor ([Classes §1](classes.md)). |
| `@class` | method / getter / setter / field | Declares the member on the **class side**: a method installs on the metaclass, a field stores on the class object ([Classes §2.1](classes.md)). |
| `@get` | field | Derives an accessor method for a field. |
| `@set` | field | Derives a mutator method for a field. |

```
@constructor
class Point {
  var _x
  var _y
  @get var _label          // derives label()
  @get @set var _color     // derives color() and color(_)
}
```

Per-field escape hatches (e.g. `@get(priv)`) fit without a grammar change.

`@constructor` is **target-polymorphic** on purpose. On a class header it derives a
constructor from the declared fields, in declaration order, with labels stripped of the
leading underscore (`_x` → `x:`); fields carrying a `default` are omitted from the
parameter list and evaluated per instance at construct time. It does **not** chain
`super.new(...)` — own fields only.

Both targets share one mechanism: the header form emits a `@constructor` **method
member** into the AST, which then expands exactly as a hand-written one does. There is
no second code path, and no separate `@construct` attribute — one name for one concept
([ADR-0063](../../adr/accepted/0063-constructors-are-ordinary-class-side-methods.md) §2).

---

## 5. Field visibility

**Fields are always private. There is no visibility syntax.**

Instance variables are reachable only from inside the class. Everything outside goes through a message. Consequences:

- `obj.x` is **unambiguously a send** in every position — one dispatch path, uniform inline caches, no second lookup mechanism.
- Consistent with "everything is a message."
- Exposure is opt-in, via derived accessors (`@get` / `@set`), so fields are invisible until deliberately published.

Explicitly **rejected**: JS-style `#field` privates (would give `#` two meanings), and `pub`/`priv` modifiers (would give field access two lookup paths).

---

## 6. Consequences for the current codebase

| Current | Change |
| --- | --- |
| `SignatureKind::Method(u8)` — arity only | Replace with an interned selector `Symbol` in canonical form. Arity is derivable from slot count. |
| Call opcodes carry arity | Call opcodes carry the interned selector. |
| Interner | Must canonicalize (whitespace-strip, R2-validate) at intern time. |
| Lexer (Logos) | Add atomic `#`-symbol token + operator branch; add shebang special-case at offset 0. |
| Parser (hand-written, `phalcom-ast`) | Add postfix `::` with `#`-lookahead. (LALRPOP is being removed per [ADR-0016](../../adr/0016-hand-written-lexer-and-recursive-descent-parser.md); the parser is hand-written — see [Implementation Status](implementation-status.md).) |
| Class finalization | Build the `base_names` index. |
| AST | Add `Expr::MethodRef { receiver: Option<Box<Expr>>, target: NameOrSelector }`. |

---

## 7. Open questions (not decided)

These were raised and deliberately deferred. They are **not** part of this spec.

1. **`var x` defaulting to `None`.** If uninitialized variables are `None`, every variable is effectively `T | None` and `nil` returns under a new name. Alternative: a VM-only `Uninit` sentinel that traps on read, keeping `None` meaningful as a *chosen* absence.
2. **`ifTrue` / `ifFalse` returning `Option`.** Chaining is unsound: `cond.ifTrue { a }.ifFalse { b }` sends `ifFalse` to an `Option`, not a `Bool`; and `ifTrue { None }` is indistinguishable from the branch not being taken. A paired `ifTrue(_)ifFalse(_)`-style selector as primary, with single-branch forms as `Option`-returning sugar, resolves both.
3. **Default arguments.** Largely incompatible with selector-identity dispatch: a call omitting a defaulted argument produces a *different* selector, so lookup misses. Options are arity-family expansion (combinatorial) or static callee knowledge (unavailable). **Decide before shipping** — retrofitting is expensive.
4. **`Option` bootstrap.** If `Option` is a plain stdlib class and fields default to `None`, constructing `None` requires a class whose fields default to `None`. `Option` likely needs to be VM-blessed / niche-encoded in `Value`, which also removes an allocation from every optional.
5. **Family introspection.** Whether `Family` exposes arity, candidate lists, etc. as a first-class reflective object, beyond its role in error messages.
