# Phalcom Tuple, Record, and Symbol Specification

**Status:** Ratified language design specification  
**Scope:** Tuple and Record semantic models; Symbol-backed labels; bare, explicit, and computed labels; Tuple lane structure and linearization; Record field structure; equality/hashability direction; encounter order; interaction with expansion; zero-product boundary references.  
**Out of scope:** Full generic typing and row-polymorphism rules, complete Record update/merge APIs, complete Map specification, iterator protocol details, full parser grammar productions, and implementation-specific memory layout except where required to preserve semantics.

---

## 1. Purpose

This specification defines three tightly related Phalcom concepts:

1. `Symbol` as the native identity type used for labels;
2. `Tuple` as an immutable ordered product with positional and ordered labeled lanes;
3. `Record` as an immutable unordered named product with Symbol fields and preserved encounter order.

The design intentionally separates ordered argument/product structure from unordered named structural data.

The core distinction is:

```text
Tuple
    ordered positional product
    +
    ordered labeled product

Record
    unordered Symbol-labeled product
```

Both are structural product types.

At zero arity, both product families normalize to `Unit`; the zero-product normalization law is specified separately.

---

## 2. Normative Terminology

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

### 2.1 Symbol

A **Symbol** is Phalcom's native immutable symbolic identity value.

Symbols are not Strings.

No implicit conversion exists:

```text
String → Symbol
```

for Tuple labels, Record fields, argument labels, or selector construction.

### 2.2 Label

A **label** is a Symbol used in a labeled product or argument position.

Tuple labeled components, Record fields, and argument-pack labeled entries all use Symbol labels.

### 2.3 Bare label

A **bare label** is a statically written Symbol accepted directly in label-head position without a leading `#`.

Examples include:

```phalcom
name:
Type?:
*:
**:
***:
?:
+(other):
method(_,_,*,a,b,**):
```

Bare-label syntax denotes a Symbol; it does not evaluate a variable.

### 2.4 Explicit Symbol label

An **explicit Symbol label** uses Symbol literal syntax directly in label-head position.

Examples:

```phalcom
#name:
#"content:type":
#":":
```

### 2.5 Computed label

A **computed label** evaluates an expression at runtime to obtain a Symbol:

```phalcom
[labelExpression]: value
```

For Tuple, Record, and argument labels, the expression MUST evaluate to a Symbol.

No implicit conversion occurs.

### 2.6 Encounter order

**Encounter order** is the stable order in which components are observed by order-sensitive traversal, printing, reflection, or expansion.

Encounter order is distinct from value equality semantics.

### 2.7 Positional lane

The Tuple **positional lane** is an ordered sequence of unlabeled values.

### 2.8 Labeled lane

The Tuple **labeled lane** is an ordered sequence of `(Symbol, Value)` components.

---

## 3. Symbol Semantics

`Symbol` represents symbolic identity rather than textual data.

The following are conceptually distinct:

```phalcom
"name"
#name
```

The first is a String. The second is a Symbol.

Two Symbols are equal according to symbolic identity, not because a String conversion happens to produce the same characters.

Implementations MAY intern Symbol values, but interning is an implementation strategy rather than the semantic definition.

Symbols MUST be hashable.

---

## 4. Symbol Literal Forms

Phalcom supports a compact unquoted Symbol form for directly representable symbolic spellings and a quoted Symbol form for arbitrary Symbol content.

### 4.1 Unquoted Symbol literals

Examples:

```phalcom
#name
#*
#**
#***
#?
#+
```

Canonical selector spellings MAY also be represented directly as Symbol literals where grammar permits:

```phalcom
#+(other)
#method(_,_,a,b)
#method(_,_,*,a,b,**)
```

The full selector grammar is specified separately.

### 4.2 Quoted Symbol literals

Symbols that cannot be represented safely in bare or unquoted syntax use quoted Symbol literals:

```phalcom
#"label:withAColon"
#":"
#"a symbol with spaces"
```

Quoted Symbol syntax provides an explicit lexical boundary.

Quoted Symbols SHOULD be non-interpolating literals. Runtime Symbol construction from dynamic text, if supported, SHOULD use an explicit conversion/construction API rather than implicit interpolation.

---

## 5. Bare-Label Domain

Bare-label syntax is intentionally broader than ordinary variable identifiers.

A syntactically self-delimiting Symbol MAY appear directly before the label separator `:`.

The accepted class includes:

1. ordinary identifier-shaped Symbols;
2. supported identifier suffix forms such as `?` where valid in Phalcom;
3. supported operator Symbols;
4. canonical selector spellings where the parser can recognize them unambiguously.

Examples:

```phalcom
name: value
Type?: value
*: value
**: value
***: value
?: value
+(other): handler
method(_,_,*,a,b,**): handler
```

The parser MAY parse a bare label as a structured grammar production rather than a single lexer token.

The lexer is not required to emit an entire selector-shaped label as one token.

---

## 6. Limits of Bare-Label Syntax

Bare-label syntax does not attempt to encode every possible Symbol.

Symbols whose spelling conflicts with the label delimiter or structural grammar use explicit Symbol syntax.

Examples that SHOULD use explicit quoting include Symbols containing:

- `:`;
- whitespace;
- commas where not structurally self-delimiting;
- parser delimiters;
- otherwise ambiguous punctuation.

Example:

```phalcom
#":": value
#"foo:bar": value
```

The first colon pair in each example belongs to the quoted Symbol literal; the following `:` is the label separator.

---

## 7. Computed Labels

Computed label syntax is:

```phalcom
[expression]: value
```

The expression is evaluated according to ordinary lexical evaluation rules.

For Tuple, Record, and argument-pack construction:

```text
result type MUST be Symbol
```

Example:

```phalcom
const label = #timeout

const args = (
    [label]: 10,
)
```

The following is invalid:

```phalcom
const label = "timeout"

const args = (
    [label]: 10,
)
```

because the expression produces a String.

No implicit String-to-Symbol conversion occurs.

If runtime Symbol creation from a String is supported, it MUST be explicit, for example conceptually:

```phalcom
const label = Symbol("timeout")
```

The exact Symbol construction API is specified separately.

---

## 8. Tuple Overview

A Tuple is an immutable structural product with two ordered lanes:

```text
Tuple {
    positional: [Value, ...]
    labeled:    [(Symbol, Value), ...]
}
```

The positional lane always precedes the labeled lane semantically.

Tuple is the first-class value form corresponding to Phalcom's ordered argument structure.

Tuple is not a Record.

Tuple labels are ordered product coordinates and participate in Tuple identity.

---

## 9. Tuple Literal Form

Tuple literal syntax uses the same lane model as calls:

```phalcom
(
    positional1,
    positional2,
    label1: value1,
    label2: value2,
)
```

Examples:

```phalcom
(1, 2)
```

```phalcom
(timeout: 10,)
```

```phalcom
(
    url,
    body,
    timeout: 10,
    retries: 3,
)
```

A Tuple literal MUST obey the positional-to-labeled source boundary defined in the argument-pack and expansion specification.

Duplicate Tuple labels are invalid.

---

## 10. Tuple Singleton Syntax

A one-component positional Tuple uses a trailing comma:

```phalcom
(value,)
```

A one-component labeled Tuple likewise uses a trailing comma where needed to distinguish the product form:

```phalcom
(label: value,)
```

The zero-component product is `Unit` and is specified by the zero-product normalization rules.

---

## 11. Tuple Total Product Order

Tuple has one linearized total product order:

```text
all positional components
followed by
all labeled components
```

Given:

```phalcom
const t = (
    "request",
    10,
    timeout: 5,
    retries: 2,
)
```

the total order is:

```text
index 0 → "request"
index 1 → 10
index 2 → timeout: 5
index 3 → retries: 2
```

The label metadata remains attached to labeled components even though integer indexing returns component values.

---

## 12. Tuple Integer Indexing

Integer indexing operates over the total linearized product order.

For:

```phalcom
const t = (
    "request",
    10,
    timeout: 5,
    retries: 2,
)
```

the following apply:

```phalcom
t[0]   // "request"
t[1]   // 10
t[2]   // 5
t[-1]  // 2
```

Negative indexing follows the general finite-sequence normalization rules specified in the collections core semantics.

Tuple integer indexing is strict and raises `IndexError` for an invalid normalized index.

The exact static result type for heterogeneous Tuple indexing is deferred to the generic/type-system specification.

---

## 13. Tuple Slicing

Tuple slicing operates over the total linearized product order.

Slicing returns a Tuple and preserves label metadata for labeled components included in the slice.

Example:

```phalcom
const t = (
    "request",
    10,
    timeout: 5,
    retries: 2,
)
```

Then:

```phalcom
t[1..3]
```

produces:

```phalcom
(
    10,
    timeout: 5,
)
```

and:

```phalcom
t[2..]
```

produces a labeled-only Tuple:

```phalcom
(
    timeout: 5,
    retries: 2,
)
```

General range normalization and slice-bound rules are specified separately.

---

## 14. Tuple Size

Tuple `size` is the total number of product components:

```text
size
    =
positional component count
    +
labeled component count
```

For:

```phalcom
const t = (
    1,
    2,
    x: 3,
    y: 4,
)
```

```phalcom
t.size
// 4
```

Whether separate convenience accessors such as `positionalSize` or `labeledSize` are provided is deferred.

---

## 15. Tuple Iteration

Tuple iteration follows the total linearized product order and yields component values.

Example:

```phalcom
const t = (
    1,
    2,
    x: 3,
    y: 4,
)

for value in t {
    ...
}
```

encounters:

```text
1
2
3
4
```

Ordinary Tuple iteration does not yield tagged wrapper variants such as `Positional(...)` or `Labeled(...)`.

Labels remain observable through labeled-lane APIs, reflection, lookup, and expansion.

Tuple iteration MUST NOT redefine `*Tuple`; positional expansion is a lane projection and includes only the positional lane.

---

## 16. Tuple Label Lookup

Tuple supports direct lookup by label in addition to integer indexing.

A label-addressed lookup addresses the labeled lane by Symbol identity.

Conceptually:

```phalcom
const args = (
    url,
    timeout: 10,
)

args[#timeout]
// 10
```

A suitable direct-label surface shorthand MAY exist where grammar permits; the semantic key is a Symbol.

Strict label lookup returns the associated value or raises `KeyError` if the label is absent.

Safe lookup follows the general `get` conventions:

```phalcom
args.get(#timeout)
// Option<T>
```

The exact selector spelling for literal-label subscript sugar is parser-syntax detail; semantically Tuple `[]` accepts integer index and Symbol label domains.

---

## 17. Tuple Lane Projections

Tuple exposes each lane as another Tuple value.

Conceptual API:

```phalcom
tuple.positionals
tuple.labeled
```

For:

```phalcom
const t = (
    1,
    2,
    x: 3,
    y: 4,
)
```

the projections are:

```phalcom
t.positionals
// (1, 2)
```

```phalcom
t.labeled
// (x: 3, y: 4)
```

Each projection remains immutable and uses Tuple semantics.

These values correspond directly to expansion behavior:

```phalcom
*t.positionals
**t.labeled
```

The exact method/property names are ratified as the intended API unless superseded by a later protocol naming pass.

---

## 18. Tuple Expansion Semantics

Tuple participates in all three expansion operators:

```text
*tuple
    → positional lane

**tuple
    → ordered labeled lane

***tuple
    → both lanes
```

This behavior is specified normatively in the argument-pack and expansion specification.

Tuple's normal iteration over the total product MUST NOT cause labeled values to be included in `*tuple`.

---

## 19. Tuple Equality

Tuple equality is exact ordered structural equality.

Two Tuples are equal only if all of the following hold:

1. positional counts are equal;
2. corresponding positional values are equal in order;
3. labeled counts are equal;
4. corresponding labels are equal in the same order;
5. corresponding labeled values are equal in order.

Examples:

```phalcom
(1, 2) == (1, 2)
// true
```

```phalcom
(1, 2) == (2, 1)
// false
```

```phalcom
(1, x: 2) == (1, 2)
// false
```

```phalcom
(a: 1, b: 2) == (b: 2, a: 1)
// false
```

```phalcom
(a: 1) == (b: 1)
// false
```

Tuple labels and lane placement are part of value semantics.

---

## 20. Tuple Hashing

Tuple is hashable if and only if every contained value is hashable.

Tuple hashing MUST incorporate:

- positional/labeled lane structure;
- positional order;
- labeled order;
- label identity;
- contained value hashes.

Therefore differently labeled or differently ordered Tuples MUST NOT rely on the same structural hash calculation merely because their linearized values match.

The exact hash-combining algorithm is implementation-defined.

---

## 21. Tuple Mutability

Tuple is immutable.

No operation mutates Tuple lane structure or contained slot assignments in place.

Any transformation that changes Tuple structure creates a new value.

This immutability is required for stable product semantics and conditional hashability.

---

## 22. Record Overview

A Record is an immutable structural named product.

Record fields are:

```text
Symbol → Value
```

A Record field set is fixed after construction.

Record is not a Map.

Record does not support arbitrary key types.

Strings, integers, and other non-Symbol values cannot serve as Record field identities.

---

## 23. Record Literal Syntax

Record literal syntax is:

```phalcom
#{
    field1: value1,
    field2: value2,
}
```

Examples:

```phalcom
#{
    name: "Ada",
    age: 36,
}
```

```phalcom
#{
    ?: handler,
    #"content:type": mediaType,
}
```

```phalcom
#{
    [computedSymbol]: value,
}
```

Bare labels, explicit Symbol labels, and computed labels all denote Symbol field identities.

---

## 24. Record Field Domain

Every Record field identity MUST be a Symbol.

Valid:

```phalcom
#{
    name: "Ada",
    active?: true,
    *: metadata,
    #"external:name": externalName,
}
```

Valid with runtime Symbol computation:

```phalcom
const key = #name

#{
    [key]: "Ada",
}
```

Invalid:

```phalcom
#{
    ["name"]: "Ada",
}
```

because `"name"` is a String.

Invalid:

```phalcom
#{
    [42]: "answer",
}
```

because `42` is an Int.

Arbitrary hashable keys belong to Map rather than Record.

---

## 25. Record Duplicate Fields

A Record cannot contain duplicate field labels.

Duplicate fields are invalid whether introduced explicitly or through expansion.

Example:

```phalcom
#{
    name: "Ada",
    name: "Grace",
}
```

is invalid.

Likewise, if:

```phalcom
#{
    name: "Ada",
    **metadata,
}
```

causes `metadata` to contribute `#name`, construction fails.

No first-wins or last-wins behavior exists for Record field conflicts.

A future explicit Record merge/update API MAY define deliberate conflict semantics.

---

## 26. Record Encounter Order

Record preserves construction encounter order.

This order is observable through operations such as:

- iteration, where defined;
- field reflection;
- printing/debug representation;
- deterministic serialization, where applicable;
- `**` expansion into ordered labeled destinations.

Encounter order does NOT participate in Record equality.

Example:

```phalcom
const a = #{
    name: "Ada",
    age: 36,
}

const b = #{
    age: 36,
    name: "Ada",
}
```

The Records preserve different encounter orders but may still be equal.

---

## 27. Record Equality

Record equality is order-insensitive structural field equality.

Two Records are equal if and only if:

1. they have the same set of Symbol field labels;
2. corresponding field values are equal.

Field encounter order is ignored.

Thus:

```phalcom
#{
    a: 1,
    b: 2,
}
==
#{
    b: 2,
    a: 1,
}
// true
```

Record equality is therefore distinct from Tuple labeled equality.

The distinction is intentional:

```text
Tuple labeled lane
    order-sensitive

Record field set
    order-insensitive
```

---

## 28. Record Hashing

Record is hashable if and only if every field value is hashable.

Because Record equality ignores encounter order, Record hashing MUST also be order-insensitive.

Equal Records with different construction encounter orders MUST produce equal hashes.

Conceptually, hashing combines `(field Symbol, field value)` contributions independently of encounter order.

The exact algorithm is implementation-defined.

Implementations MAY cache a Record hash because Record is immutable.

---

## 29. Record Mutability

Record is immutable.

Field sets and field values are fixed after construction.

An operation that conceptually updates or extends a Record MUST produce a new value.

The exact Record copy/update/merge API is deferred.

Record immutability is a semantic distinction from mutable Map.

---

## 30. Record Versus Map

Record and Map are distinct semantic types even if implementations share lower-level storage machinery.

The intended distinction is:

| Property | Record | Map |
|---|---|---|
| Mutability | immutable | mutable |
| Key domain | Symbol only | arbitrary hashable key |
| Shape | fixed after construction | dynamic |
| Field/value typing | potentially heterogeneous per field | generic key/value typing |
| Equality order | order-insensitive | order-insensitive |
| Encounter order | construction order preserved | insertion order preserved |
| Hashability | iff field values hashable | unhashable while mutable |
| Literal | `#{...}` | `{...}` |

No implicit Record-to-Map or Map-to-Record conversion occurs.

---

## 31. Record-to-Map Conversion

A Record may be explicitly converted to a Map.

Canonical target-side construction:

```phalcom
Map.from(record: record)
```

The resulting Map:

1. uses each Record field Symbol as a Map key;
2. preserves field values;
3. uses Record encounter order as initial Map insertion order;
4. is semantically independent and mutable;
5. may lose exact per-field type precision according to normal Map typing rules.

Example:

```phalcom
const record = #{
    a: 1,
    b: 2,
}

const map = Map.from(record: record)
```

Mutation of `map` MUST NOT mutate `record`.

An implementation MAY share immutable storage and detach lazily, but that is not observable.

---

## 32. Record Expansion

Record supports labeled expansion:

```text
**record
    → Record fields in encounter order
```

When expanded into a call or Tuple, field order becomes the labeled-lane order and may therefore affect selector identity.

This does not conflict with Record's order-insensitive equality because expansion is explicitly an encounter-order-observing operation.

Example:

```phalcom
const a = #{
    x: 1,
    y: 2,
}

const b = #{
    y: 2,
    x: 1,
}

a == b
// true
```

but:

```phalcom
foo(**a)
foo(**b)
```

may derive different selectors because the ordered label sequences differ.

This is intentional.

---

## 33. Dynamic Record Shape

Record construction MAY involve computed Symbol labels or `**` expansion from a stable ordered mapping source.

Therefore a Record's final field set may be determined at runtime.

The resulting Record is still immutable and fixed after construction.

The precise static type assigned to dynamically shaped Records is deferred to the generic/row-type specification.

This specification does not require a particular existential, open-row, or erased structural type representation.

---

## 34. Zero-Product Normalization Boundary

Tuple and Record are both structural product families.

At zero arity, their distinguishing coordinate structure contains no information.

Phalcom therefore normalizes:

```text
Tuple with zero positional and zero labeled components
    → Unit

closed Record with zero fields
    → Unit
```

Thus:

```phalcom
()
```

and:

```phalcom
#{}
```

both denote the canonical zero-product `Unit` value.

This is definitional normalization rather than implicit conversion.

The full normalization and runtime representation rules are specified in the product normalization and Unit specification.

Open record rows or unknown additional fields MUST NOT normalize to Unit merely because zero fields are explicitly known.

---

## 35. Product-Family Relationship

Tuple and Record are distinct positive-arity product families.

Tuple coordinates are structured as:

```text
ordered positional coordinates
+
ordered labeled coordinates
```

Record coordinates are structured as:

```text
unordered finite set of distinct Symbol labels
```

For nonzero arity, the coordinate structures are observably different and therefore the product families remain distinct.

At zero arity:

```text
ordered empty coordinate structure
```

and:

```text
unordered empty coordinate structure
```

carry no distinguishing information and normalize to the same terminal/unit product.

This relationship is conceptual and type-theoretic; it does not require nominal inheritance such as:

```text
Unit <: Tuple
Unit <: Record
```

---

## 36. Cross-Family Equality

Tuple and Record equality is family-sensitive for positive-arity products.

A Tuple and Record do not compare equal merely because their values could be paired in some structural correspondence.

Examples:

```phalcom
(1, 2) != #{
    first: 1,
    second: 2,
}
```

The only canonical collapse occurs at the zero-product case through `Unit` normalization.

Cross-family equality between other collection families is specified separately.

---

## 37. Selector-Shaped Symbols

Phalcom selectors may be represented symbolically where canonical selector spelling is available.

Examples include conceptual Symbols such as:

```text
#+(other)
#method(_,_,a,b)
#method(_,_,*,a,b,**)
```

These Symbols may serve as Tuple labels or Record fields.

Example:

```phalcom
#{
    +(other): addHandler,
    method(_,_,a,b): methodHandler,
}
```

Using a selector-shaped Symbol as data does not perform dispatch.

It is symbolic identity only.

The exact mapping between `Symbol` and any distinct reflective `Selector` runtime type is deferred to the reflection specification.

---

## 38. Operator Symbols as Labels

Supported operator Symbols may serve as labels where syntax is unambiguous.

Examples:

```phalcom
(
    *: Int,
    **: String,
    ***: Bool,
    ?: Handler,
)
```

and:

```phalcom
#{
    *: positionalMetadata,
    ?: optionalHandler,
}
```

The label separator is the following `:` token.

The parser distinguishes:

```phalcom
*values
```

as expansion from:

```phalcom
*: value
```

as a Symbol-labeled component by context and following grammar.

---

## 39. Parser Model

A conforming parser MAY model label-head syntax conceptually as:

```text
label-entry
    := label-head ':' expression

label-head
    := bare-identifier-symbol
     | bare-operator-symbol
     | canonical-selector-symbol
     | explicit-symbol-literal
     | '[' expression ']'
```

This grammar is illustrative rather than a required parser implementation.

Semantic analysis enforces:

```text
Tuple label   → Symbol
Record field  → Symbol
argument label → Symbol
```

The parser SHOULD preserve source form sufficiently for precise diagnostics.

---

## 40. Suggested Runtime Representation

This section is implementation guidance, not a mandated representation.

### 40.1 Tuple

A positive-arity Tuple may be represented by:

```text
TupleObject {
    positionalValues
    labeledSymbols
    labeledValues
}
```

or by a combined compact layout preserving the lane boundary.

The representation MUST preserve:

- positional order;
- labeled order;
- label identity;
- immutable structure.

### 40.2 Record

A positive-arity Record may be represented by:

```text
RecordShape {
    fieldsInEncounterOrder
    lookup Symbol → slot
    optional canonical unordered shape identity
}

RecordObject {
    shape
    values
}
```

Records with the same encounter-order shape MAY share a shape object.

Implementations MAY additionally maintain canonical unordered shape metadata to accelerate order-insensitive equality/type comparison.

Such optimization MUST NOT erase each Record's required encounter order.

---

## 41. Equality Fast Paths

Implementations MAY optimize equality while preserving semantics.

### 41.1 Tuple

Tuples with identical runtime layouts may compare corresponding slots directly because order and labels are semantic.

### 41.2 Record

Records with identical shapes may compare corresponding slots directly.

Records with different encounter-order shapes but the same field set require order-insensitive field matching.

An implementation MAY use:

- Symbol lookup;
- canonical unordered shape IDs;
- precomputed field maps;
- other equivalent methods.

The observable equality result MUST remain independent of encounter order.

---

## 42. Deferred Issues

The following are intentionally deferred:

1. complete Symbol construction/conversion API;
2. exact Unicode normalization policy for Symbol spelling;
3. full selector literal grammar;
4. whether `Selector` is a distinct runtime type from selector-shaped `Symbol`;
5. full Tuple static typing for heterogeneous indexed access and slicing;
6. full Tuple destructuring grammar and typing;
7. exact Record field-access surface syntax beyond label/key semantics;
8. Record copy/update/merge APIs;
9. dynamic Record row typing;
10. open Record rows and row polymorphism;
11. structural vs nominal protocol satisfaction;
12. complete generic variance rules;
13. complete reflection APIs for Tuple/Record fields and lanes;
14. printing and recursive representation rules;
15. full Map specification and key-equality rules;
16. whether common product capabilities expose shared operations on Unit, Tuple, and Record.

Future specifications MAY refine these areas but MUST preserve the ratified semantics here unless explicitly superseded.

---

## 43. Conformance Summary

A conforming Phalcom implementation MUST preserve these core laws:

```text
Symbol
    = native symbolic identity
    != String

implicit String → Symbol conversion
    = forbidden

Tuple
    = immutable ordered positional lane
      + immutable ordered labeled lane

Tuple total order
    = positionals followed by labeled values

Tuple equality
    = lane-sensitive, label-sensitive, order-sensitive

Tuple hashing
    = structural and order-sensitive
      iff all values are hashable

Record
    = immutable fixed Symbol-field structural product

Record equality
    = field/value structural
      and encounter-order-insensitive

Record hashing
    = order-insensitive
      iff all field values are hashable

Record encounter order
    = construction encounter order

bare label
    = statically written Symbol in label-head position

explicit Symbol label
    = Symbol literal used as label head

computed label
    = [expression]: value
      where expression evaluates to Symbol

Tuple expansion
    *   → positional lane
    **  → labeled lane
    *** → both lanes

Record expansion
    ** → fields in encounter order

zero positional/labeled Tuple
    → Unit

closed zero-field Record
    → Unit
```
