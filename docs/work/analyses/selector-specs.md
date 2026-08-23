I investigated the repository structure and the existing selector implementation before writing this specification. The current design already has an important foundation: Phalcom’s selector specification treats exact selectors as symbolic values and reserves selector patterns as a distinct runtime concept. The repository currently implements `SelectorPatternObject` as a VM heap object with a compiled matcher representation (`phalcom-core/src/heap/selector_pattern.rs`) and exposes matching through the primitive bridge (`phalcom-core/src/primitive/selector_pattern.rs`). The current behavior diverges from the desired model because selector-pattern-shaped literals are promoted eagerly into objects while exact selector literals remain symbols. The change below normalizes both paths.

The existing selector specification already states the intended semantic direction: exact selectors are represented through symbols, selector-aware consumers interpret them, and patterns are different because they represent a set of selectors rather than one selector.

# Implementation Specification: Lazy Selector and SelectorPattern Materialization

## 1. Objective and semantic contract

## Goal

Change selector literal semantics so that:

```phalcom
#foo
#foo()
#foo(_)
#foo(_, bar)
```

always produce ordinary interned `Symbol` values.

No selector-shaped literal should automatically allocate a `Selector` object.

Selector objects are explicitly constructed:

```phalcom
Selector(#foo(_))
```

which semantically means:

```phalcom
Selector.call(#foo(_))
```

because classes are callable objects in Phalcom.

Selector patterns become explicit:

```phalcom
SelectorPattern(#foo(_, ...))
```

which semantically means:

```phalcom
SelectorPattern.call(#foo(_, ...))
```

The implicit constructor call is the normal user-facing API.

Explicit constructors remain available for atomic construction or native/runtime use cases:

```phalcom
Selector.from(#foo(_))
SelectorPattern.from(#foo(_, ...))
```

---

# Semantic invariants

The following rules become mandatory:

| Expression | Result |
|-|-|
| `#foo` | Symbol |
| `#foo()` | Symbol |
| `#foo(_)` | Symbol |
| `#foo(...)` | Symbol |
| `Selector(#foo(_))` | Selector object |
| `SelectorPattern(#foo(...))` | SelectorPattern object |

The following is forbidden:

```phalcom
#foo(...)
```

directly producing a `SelectorPattern`.

The parser must preserve the selector-pattern structure inside the symbol representation.

---

# 2. Current implementation analysis

## Existing architecture

The current architecture already separates:

```
source selector syntax
        |
        v
phalcom_common selector representation
        |
        v
runtime selector pattern object
        |
        v
matcher
```

The runtime pattern implementation exists in:

```
phalcom-core/src/heap/selector_pattern.rs
```

The current runtime object contains:

```rust
pub struct SelectorPatternObject {
    pub pattern: SelectorPattern,
    pub(crate) runtime: RuntimeSelectorPattern,
}
```

The implementation compiles the semantic pattern into an optimized matcher:

```rust
RuntimeSelectorPattern::compile(...)
```

This is good architecture and should remain.

The problem is not the matcher.

The problem is **when the matcher object is created**.

Currently:

```
selector-pattern literal
        |
        v
SelectorPatternObject allocation
```

The new architecture becomes:

```
selector-pattern literal
        |
        v
Symbol
        |
        |
        +---- SelectorPattern(...)
                  |
                  v
              SelectorPatternObject
```

---

# 3. Design decisions

## Decision 1: Symbols are the universal literal representation

Chosen:

```
literal syntax -> Symbol
```

Rejected:

```
literal syntax -> Selector / SelectorPattern object
```

Reason:

- Symbols are interned.
- Symbols are cheap.
- Symbols are identity values.
- Symbols are useful as map keys.
- Symbols do not require heap allocation.

Example:

```phalcom
metadata[#foo(_)] = documentation
```

must remain possible.

---

## Decision 2: Construction is explicit

Chosen:

```phalcom
Selector(#foo(_))
```

Rejected:

```phalcom
#foo(_)
```

becoming a Selector.

Reason:

The syntax should not decide semantic domain.

The user decides whether a symbolic structure participates in:

- reflection,
- dispatch,
- pattern matching,
- metadata,
- serialization.

---

## Decision 3: Parsing happens once

The system must not do:

```
Symbol
 |
 stringify
 |
 parse again
 |
 Selector
```

Instead:

```
lexer/parser
 |
 selector metadata
 |
 Symbol
 |
 Selector view/object
```

The existing selector parser should remain the source of truth.

---

# 4. Proposed architecture

## Symbol representation enhancement

The symbol representation needs to retain selector-shape metadata.

Current:

```rust
Symbol {
    name
}
```

becomes conceptually:

```rust
Symbol {
    name,
    selector_shape: Option<SelectorShape>
}
```

Where:

```rust
struct SelectorShape {
    kind: SelectorKind,
    slots: Vec<SelectorSlot>,
    is_pattern: bool
}
```

Example:

```phalcom
#foo(_,bar,...)
```

stores:

```
Symbol
 |
 + selector_shape
       |
       + Method
       + Positional
       + Label(bar)
       + Gap
```

No runtime SelectorPattern allocation happens.

---

# 5. Selector construction

## New callable behavior

The Selector class receives:

```phalcom
Selector.call(symbol)
```

Implementation:

1. Validate argument is Symbol.
2. Extract selector metadata.
3. Reject pattern symbols.
4. Construct Selector object.

Pseudo:

```rust
fn selector_call(value):
    symbol = require_symbol(value)

    shape = symbol.selector_shape
        or error InvalidSelector

    if shape.is_pattern:
        error PatternCannotBecomeSelector

    return SelectorObject(shape)
```

---

# 6. SelectorPattern construction

The existing machinery remains.

Current:

```rust
SelectorPatternObject::compile(pattern, interner)
```

is retained.

New flow:

```
SelectorPattern.call(symbol)

        |
        v

extract SelectorPattern metadata

        |
        v

SelectorPatternObject::compile(...)
```

The only change is the construction trigger.

---

# 7. Compiler changes

## Current behavior

The compiler currently has a special case:

```
selector pattern literal
        |
        v
construct runtime object
```

Remove this.

All selector literals lower identically:

```
#anything
        |
        v
LoadSymbol
```

The compiler must not distinguish:

```phalcom
#foo(_)
```

and:

```phalcom
#foo(...)
```

for runtime allocation purposes.

The distinction remains inside symbol metadata.

---

# 8. Bytecode changes

Remove any bytecode lowering that directly creates:

```
SelectorPatternObject
```

from literal syntax.

Required bytecode behavior:

```phalcom
#foo(...)
```

becomes:

```
CONSTANT_SYMBOL foo(...)
```

Then:

```phalcom
SelectorPattern(#foo(...))
```

becomes:

```
LOAD_SYMBOL
SEND call(_)
```

which follows ordinary object semantics.

---

# 9. Runtime allocation optimization

## Before

Example:

```phalcom
for item in items {
    handlers.match(#foo(...))
}
```

creates:

```
SelectorPatternObject
SelectorPattern matcher
heap allocation
```

every evaluation unless optimized.

---

## After

The same code:

```
interned symbol lookup
pointer reuse
```

Only:

```phalcom
SelectorPattern(#foo(...))
```

allocates.

Even then, optimization should be added:

## SelectorPattern cache

The symbol table should optionally contain:

```rust
Symbol {
    selector_shape,
    cached_pattern: OnceCell<SelectorPatternObject>
}
```

Then:

```phalcom
SelectorPattern(#foo(...))
```

becomes:

first call:

```
allocate pattern object
store cache
```

future calls:

```
return cached immutable object
```

---

# 10. Reflection changes

Reflection must distinguish:

```
Symbol
 |
 + selector-shaped
```

from:

```
Selector object
```

Example:

```phalcom
#foo(_).is_selector
```

should not exist.

Instead:

```phalcom
Selector(#foo(_))
```

has selector behavior.

Recommended APIs:

```phalcom
symbol.selector?
symbol.selector_pattern?
```

may exist as introspection helpers.

---

# 11. Error handling

## Invalid Selector construction

Example:

```phalcom
Selector(#foo(...))
```

Error:

```
Cannot construct Selector from selector pattern symbol.
Use SelectorPattern instead.
```

---

## Invalid SelectorPattern construction

Example:

```phalcom
SelectorPattern(#foo(_))
```

Error:

```
SelectorPattern requires a selector pattern.
Received exact selector.
```

---

## Non-symbol input

Example:

```phalcom
Selector(123)
```

Error:

```
Expected Symbol, received Int.
```

---

# 12. Testing strategy

## Parser tests

Verify:

```phalcom
#foo(...)
```

produces:

```
Symbol
selector_shape.pattern=true
```

not:

```
SelectorPatternObject
```

---

## Runtime tests

Add:

```phalcom
Selector(#foo(_))
```

Expected:

```
Selector object
```

---

```phalcom
SelectorPattern(#foo(...))
```

Expected:

```
SelectorPattern object
```

---

## Identity tests

Verify:

```phalcom
#foo(...) == #foo(...)
```

is true.

---

## Allocation tests

Add benchmark:

Before:

```
100000 selector pattern literals
```

After:

```
100000 symbol loads
```

Verify no heap growth.

---

## Regression tests

Existing selector pattern matching tests in:

```
phalcom-core/src/heap/selector_pattern.rs
```

remain valid.

Only construction path changes.

---

# 13. Implementation sequence

## Phase 1 — Preserve selector metadata

Modify symbol representation.

Do not change syntax.

Ensure:

```
#foo(...)
```

still parses.

---

## Phase 2 — Remove eager pattern allocation

Delete compiler/runtime path:

```
literal -> SelectorPatternObject
```

Replace with:

```
literal -> Symbol
```

---

## Phase 3 — Implement callable constructors

Add:

```phalcom
Selector.call(_)
SelectorPattern.call(_)
```

using existing runtime machinery.

---

## Phase 4 — Add caching

Add:

```
Symbol -> cached SelectorPatternObject
```

only after correctness.

---

## Phase 5 — Update tooling

Update:

- reflection
- debugger display
- LSP hover
- documentation generation

so:

```
#foo(...)
```

is shown as:

```
Symbol (selector pattern shape)
```

not:

```
SelectorPattern object
```

---

# 14. Acceptance criteria

Implementation is complete when:

- [ ] All selector-shaped literals produce Symbols.
- [ ] No selector literal directly allocates Selector objects.
- [ ] No selector-pattern literal directly allocates SelectorPattern objects.
- [ ] `Selector(#foo(_))` works through ordinary `call(_)`.
- [ ] `SelectorPattern(#foo(...))` works through ordinary `call(_)`.
- [ ] Existing selector matching behavior is unchanged.
- [ ] Selector pattern matching still uses the optimized `RuntimeSelectorPattern`.
- [ ] Repeated literal usage does not allocate runtime selector objects.
- [ ] Reflection distinguishes symbolic representation from selector objects.
- [ ] Tests verify semantic identity and allocation behavior.

The existing `SelectorPatternObject` implementation is therefore not removed; it becomes the **materialized runtime form** produced only when the user explicitly requests selector-pattern semantics. This keeps Phalcom’s object model consistent: syntax creates values, and messages create richer semantic objects.