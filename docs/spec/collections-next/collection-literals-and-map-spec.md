# Phalcom Collection Literals and Map Specification

**Status:** Ratified language design specification  
**Scope:** Collection literal classification, Map literal syntax, Map key syntax, Map encounter-order semantics, Map equality and mutation behavior, duplicate handling during literal construction, Record-to-Map conversion, Set literal classification where required to disambiguate brace syntax, and future `HashMap` / `OrderedMap` boundaries.  
**Out of scope:** Full Set and `ImmutableSet` APIs, complete Record semantics, generic Map typing and variance, iterator/view mutation rules, complete hashing protocol definitions, and collection protocol hierarchy.

---

## 1. Purpose

This specification defines Phalcom's literal taxonomy for core collection/product forms and the semantic model of the default mutable `Map`.

Phalcom distinguishes:

```text
Tuple
List
Record
Map
Set
```

using syntax that remains locally classifiable without contextual type inference.

The canonical literal family is:

```phalcom
(...)       // Tuple / product
[...]       // List
#{...}      // Record
{...}       // Map or nonempty Set, distinguished structurally
```

The empty Set is written:

```phalcom
Set()
```

because `{}` is reserved for the empty Map.

---

## 2. Normative Terminology

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

### 2.1 Bare Symbol key

A **bare Symbol key** is a Map entry head written directly using bare-label syntax:

```phalcom
name: value
timeout: value
?: value
```

The key is a `Symbol`, not a `String`.

### 2.2 Computed key

A **computed key** is a key expression written:

```phalcom
[expression]: value
```

The expression is evaluated normally and may produce any key value valid for the Map.

### 2.3 Encounter order

For Map, **encounter order** is insertion order.

Encounter order is observable through iteration, views, printing, and order-sensitive expansion.

Encounter order does not participate in Map equality.

### 2.4 First insertion

The **first insertion** of a key is the operation that first establishes an association for a key-equivalence class in a Map.

Updating an existing key does not count as a new insertion.

---

## 3. Literal Classification

The following literal forms are ratified:

```phalcom
()
```

denotes the canonical zero-product `Unit` value.

```phalcom
[]
```

denotes an empty `List`.

```phalcom
#{}
```

is empty Record syntax and normalizes definitionally to `Unit`.

```phalcom
{}
```

denotes an empty `Map`.

```phalcom
Set()
```

constructs an empty mutable `Set`.

For nonempty forms:

```phalcom
(a, b)
```

is a Tuple.

```phalcom
[a, b]
```

is a List.

```phalcom
#{
    a: b,
    c: d,
}
```

is a Record.

```phalcom
{
    a: b,
    c: d,
}
```

is a Map.

```phalcom
{a, b, c}
```

is a Set.

Literal classification MUST NOT depend on contextual typing.

---

## 4. Brace-Literal Disambiguation

Bare braces are used by both Map and nonempty Set literals.

The parser distinguishes them structurally.

### 4.1 Set literal

A brace literal containing element entries is a Set:

```phalcom
{a, b, c}
```

Element expansion also identifies Set construction:

```phalcom
{
    *values,
}
```

### 4.2 Map literal

A brace literal containing association entries is a Map:

```phalcom
{
    a: b,
    c: d,
}
```

Association expansion identifies Map construction:

```phalcom
{
    **mapping,
}
```

### 4.3 Empty braces

Because no entry exists to classify the literal:

```phalcom
{}
```

is defined to mean an empty `Map`.

The empty Set is written:

```phalcom
Set()
```

### 4.4 Mixed brace entries

A single brace literal MUST NOT mix Set-style element entries and Map-style association entries.

Invalid:

```phalcom
{
    a,
    b: c,
}
```

Invalid:

```phalcom
{
    *values,
    key: value,
}
```

Such forms are syntax errors rather than dynamically classified values.

---

## 5. Record Literal Boundary

Record literals always use the `#{...}` form.

Example:

```phalcom
#{
    name: "Ada",
    age: 36,
}
```

The `#` prefix is semantically aligned with Record's Symbol-field structure.

Record fields are Symbol-labeled and Record semantics are specified separately.

A bare `{...}` literal MUST NOT produce a Record.

---

## 6. Default Map Semantics

`Map<K, V>` is Phalcom's default mutable dynamic association collection.

Its semantic model is:

```text
mutable
dynamic key set
arbitrary hashable keys
order-insensitive value equality
insertion encounter order preserved
unhashable while mutable
```

Map preserves deterministic encounter order while remaining conceptually unordered with respect to value identity.

---

## 7. Map Key Domain

Map keys MAY be any value satisfying the language's hashability/equality requirements.

Examples of possible key categories include:

- Symbols;
- Strings;
- integers;
- hashable Tuples;
- hashable Records;
- other user-defined hashable values.

Map does not restrict keys to Symbols.

This is a defining distinction from Record.

---

## 8. Bare Map Keys Are Symbols

Inside a Map literal, bare-label syntax produces a Symbol key.

Example:

```phalcom
{
    timeout: 10,
    retries: 3,
}
```

is semantically equivalent to a Map whose keys are:

```text
#timeout
#retries
```

It is NOT equivalent to using String keys `"timeout"` and `"retries"`.

The same bare-label syntax therefore has a consistent meaning across:

- argument labels;
- Tuple labeled components;
- Record fields;
- Symbol-keyed Map entries.

---

## 9. Computed Map Keys

Arbitrary Map keys use computed-key syntax:

```phalcom
[expression]: value
```

Examples:

```phalcom
{
    ["timeout"]: 10,
    [42]: "answer",
    [user.id]: metadata,
    [(1, 2)]: pointData,
}
```

The expression is evaluated normally.

The resulting key is not coerced to Symbol.

Therefore:

```phalcom
{
    timeout: 10,
}
```

uses the Symbol key:

```phalcom
#timeout
```

while:

```phalcom
{
    ["timeout"]: 10,
}
```

uses the String key:

```phalcom
"timeout"
```

These are distinct keys.

---

## 10. No Implicit String-to-Symbol Conversion

Map syntax MUST NOT implicitly convert Strings into Symbols.

Likewise, labeled expansion from Map into a call, Tuple, or Record MUST NOT convert String keys to Symbols.

The distinction is explicit:

```phalcom
{
    timeout: 10,
}
```

uses `#timeout`.

```phalcom
{
    ["timeout"]: 10,
}
```

uses `"timeout"`.

If dynamic Symbol construction from text is supported, it must be explicit through the Symbol API.

---

## 11. Map Literal Evaluation Order

Map literal entries and expansion sources are evaluated in lexical source order.

For:

```phalcom
{
    firstKey(): firstValue(),
    **mappingExpr(),
    [secondKey()]: secondValue(),
}
```

evaluation follows source order.

Encounter-order insertion is determined by the sequence of associations contributed after each source expression is evaluated.

Evaluation order MUST remain distinct from equality semantics.

---

## 12. Map Insertion Encounter Order

Map iteration order is first-insertion order.

Given:

```phalcom
const map = {}

map[#a] = 1
map[#b] = 2
map[#c] = 3
```

encounter order is:

```text
#a
#b
#c
```

This order is used by:

- Map iteration;
- `keys`;
- `values`;
- `entries`;
- printing/debug output;
- `**` expansion when Map is used as an ordered labeled source.

---

## 13. Updating Existing Keys

Updating an existing key MUST retain its original encounter position.

Example:

```phalcom
const map = {
    a: 1,
    b: 2,
}

map[#a] = 3
```

encounter order remains:

```text
#a
#b
```

The update changes only the associated value.

It does not count as a new insertion.

---

## 14. Removal and Reinsertion

Removing a key deletes its encounter-order position.

Reinserting an equivalent key later places it at the end.

Example:

```phalcom
const map = {
    a: 1,
    b: 2,
}

map.remove(#a)
map[#a] = 3
```

encounter order becomes:

```text
#b
#a
```

This rule is normative.

---

## 15. Map Equality

Map equality is order-insensitive extensional mapping equality.

Two Maps are equal if and only if they contain equal key/value associations.

Insertion history and encounter order are ignored.

Thus conceptually:

```phalcom
{
    a: 1,
    b: 2,
}
==
{
    b: 2,
    a: 1,
}
// true
```

provided the key/value equality rules consider the corresponding associations equal.

Equal Maps MAY expose different encounter orders.

This is intentional.

---

## 16. Map Hashability

The default `Map` is mutable and therefore unhashable.

A Map MUST NOT be accepted as a hash key while it remains mutable under the standard Map semantics.

Future immutable mapping types may define hashability separately.

---

## 17. Map Mutation Results

The following mutation-result semantics are ratified.

### 17.1 Explicit insert

```phalcom
map.insert(value, for: key)
// Option<V>
```

returns:

```text
None
    when the key was newly inserted

Some(previous)
    when an existing association was replaced
```

Updating an existing key through `insert` retains encounter position.

### 17.2 Subscript assignment

```phalcom
map[key] = value
```

stores the association and evaluates to the original right-hand-side value according to the general subscript-assignment rule.

The setter method's internal return value does not determine the assignment expression value.

### 17.3 Remove

```phalcom
map.remove(key)
// Option<V>
```

returns the removed value when present.

### 17.4 Clear

```phalcom
map.clear
// Unit
```

removes all associations.

---

## 18. Strict and Safe Lookup

Strict Map subscript lookup:

```phalcom
map[key]
```

returns the value or raises `KeyError`.

Safe lookup:

```phalcom
map.get(key)
```

returns:

```text
Option<V>
```

Stored `None` remains distinguishable from absence.

Example:

```phalcom
map[key] = None

map.get(key)
// Some(None)
```

The full lookup/fallback model is specified in the collections core semantics specification.

---

## 19. Map Views

Map exposes lightweight ordered views:

```phalcom
map.keys
map.values
map.entries
```

These views preserve Map encounter order.

They are not specified as copied Lists.

The exact mutation-during-iteration behavior of live views is deferred to the iterator/view specification.

---

## 20. Entry Type

Map entry iteration/reflection uses an immutable semantic entry type:

```text
Entry<K, V>
```

with:

```phalcom
entry.key
entry.value
```

Entries are destructurable in appropriate contexts:

```phalcom
for (key, value) in map.entries {
    ...
}
```

The detailed destructuring specification is deferred.

---

## 21. Map Association Expansion

Map literals accept `**` association expansion.

Example:

```phalcom
const defaults = {
    timeout: 10,
    retries: 3,
}

const options = {
    **defaults,
    cache: true,
}
```

Associations from `defaults` are contributed in Map encounter order.

Because the destination is another Map, keys MAY be arbitrary valid Map keys.

The Symbol-only restriction applies only when Map is expanded into an ordered labeled destination such as a call, Tuple, or Record.

---

## 22. Target-Sensitive `**Map`

`**Map` has target-sensitive validation.

### 22.1 Map destination

For:

```phalcom
{
    **mapping,
}
```

keys may be any valid Map keys.

### 22.2 Call, Tuple, or Record destination

For:

```phalcom
foo(**mapping)
```

```phalcom
(
    **mapping,
)
```

```phalcom
#{
    **mapping,
}
```

every contributed key MUST be a Symbol because these destinations have Symbol-labeled structure.

Map encounter order is preserved.

No String-to-Symbol conversion occurs.

---

## 23. Map Literal Duplicate Keys

Map literal construction rejects duplicate keys.

There is no implicit first-wins or last-wins behavior.

Invalid:

```phalcom
{
    a: 1,
    a: 2,
}
```

Likewise, if expansion introduces an already-present equivalent key:

```phalcom
const defaults = {
    a: 1,
}

const map = {
    **defaults,
    a: 2,
}
```

construction fails.

If a duplicate can be proven statically, the compiler SHOULD reject it statically.

Otherwise construction MUST fail dynamically before the Map value is finalized.

Explicit post-construction assignment remains the ordinary way to overwrite:

```phalcom
map[#a] = 2
```

Future merge APIs may define deliberate conflict-resolution behavior.

---

## 24. Map Construction From Record

A Record can be explicitly converted into a Map.

Canonical construction:

```phalcom
Map.from(record: record)
```

Example:

```phalcom
const record = #{
    name: "Ada",
    age: 36,
}

const map = Map.from(record: record)
```

The conversion:

1. uses each Record field Symbol as a Map key;
2. preserves each field value;
3. preserves Record encounter order as initial Map insertion order;
4. produces a mutable Map;
5. is semantically independent from the original immutable Record.

Mutation of the resulting Map MUST NOT mutate the Record.

---

## 25. Record-to-Map Type Widening

Record fields may have heterogeneous statically known types.

Converting to `Map` may therefore lose exact per-field type precision.

Conceptually:

```text
#{
    name: String,
    age: Int,
}
```

may become a Map whose value type is a normal type join or other common supertype determined by the generic type system.

The exact inference rule is deferred.

The conversion MUST remain explicit because it changes:

- mutability;
- shape guarantees;
- key-domain semantics;
- type precision;
- hashability.

---

## 26. Record-to-Map Runtime Optimization

Although `Map.from(record:)` is semantically a copy, an implementation MAY share immutable backing storage internally and detach lazily on first mutation.

Conceptually:

```text
Record
    ─┐
     ├── shared immutable association storage
Map  ─┘
```

followed by copy-on-write on Map mutation.

This optimization MUST NOT be observable.

Record and Map remain distinct semantic types.

---

## 27. Map Literal Versus Record Literal

The distinction is always explicit:

```phalcom
#{
    name: "Ada",
}
```

is a Record.

```phalcom
{
    name: "Ada",
}
```

is a Map with Symbol key `#name`.

A programmer can convert deliberately:

```phalcom
Map.from(record: record)
```

There is no automatic promotion from Record to Map based on mutation or contextual typing.

---

## 28. Rejected Contextual Literal Typing

The same brace literal MUST NOT silently become Record or Map depending on expected type.

Rejected model:

```phalcom
const x = {
    a: 1,
}
// one type

const y: SomeOtherType = {
    a: 1,
}
// same syntax, different collection family
```

Phalcom literal family is syntactically determined.

Type context may infer generic parameters but MUST NOT change the literal's collection family.

---

## 29. Rejected `const { ... }` Record Construction

`const` controls binding/declaration semantics and MUST NOT determine whether a brace expression constructs a Record or Map.

Therefore:

```phalcom
const map = {
    a: 1,
}
```

means:

```text
an immutable binding
to a mutable Map value
```

unless some other language feature changes binding semantics.

Record construction uses `#{...}` explicitly.

This preserves the distinction:

```text
binding immutability
≠
value immutability
```

---

## 30. Set Literal Classification

A nonempty Set uses bare braces with element entries:

```phalcom
{a, b, c}
```

Set literal element expansion uses `*`:

```phalcom
{
    *values,
}
```

The full Set API is specified separately.

For brace parsing, the key requirement is:

```text
element entries
    → Set

association entries
    → Map
```

---

## 31. Empty Set

Because:

```phalcom
{}
```

is the empty Map, the empty mutable Set is written:

```phalcom
Set()
```

No `Set{}` syntax is ratified at this stage.

This is an intentional asymmetry.

The existence of a distinct empty Set value is semantically important for:

- union identity;
- intersection results;
- subset relations;
- graph/permission/state algorithms;
- accumulation;
- natural empty-result values.

---

## 32. Set and ImmutableSet Boundary

Phalcom follows the mutable/immutable split:

```text
Set<T>
    mutable

ImmutableSet<T>
    immutable
```

`Set()` creates an independent mutable empty Set value.

`ImmutableSet()` and a future `ImmutableSet.empty` MAY share a canonical immutable empty instance.

This distinction is specified more fully in the product/Unit normalization document and future Set specification.

---

## 33. Future HashMap

Phalcom may later introduce:

```text
HashMap<K, V>
```

as a specialized mapping whose encounter order is unspecified.

Such a type exists, if added, for representation/performance specialization rather than as the default Map semantics.

A future `HashMap` MUST NOT support `**` into an ordered labeled destination because unspecified iteration order must not determine selector identity.

Example that MUST remain invalid for unspecified-order `HashMap`:

```phalcom
foo(**hashMap)
```

even if every key is a Symbol.

---

## 34. Future OrderedMap

Phalcom may later introduce:

```text
OrderedMap<K, V>
```

if first-class reordering operations are justified.

Its purpose would not merely be "a Map that remembers insertion order," because default Map already preserves insertion encounter order.

A future `OrderedMap` may instead provide operations such as explicit entry repositioning.

Exact equality semantics and API are deferred.

---

## 35. Default Map Versus Future Specialized Maps

The intended semantic hierarchy is:

```text
Map
    default general-purpose mutable association
    insertion encounter order
    order-insensitive equality

HashMap
    possible future specialized unordered-storage map
    unspecified encounter order

OrderedMap
    possible future explicitly reorderable mapping
```

The simple name `Map` belongs to the ergonomic deterministic default.

Specialized behavior receives specialized names.

---

## 36. Map Implementation Freedom

The language specification does not mandate a particular Map storage structure.

Conforming implementations MAY use:

- compact ordered hash tables;
- dense ordered entry arrays plus hash indices;
- hash table plus order vector;
- another equivalent representation.

The implementation MUST preserve:

- expected key lookup semantics;
- first-insertion encounter order;
- stable position on update;
- remove-and-reinsert-to-end behavior;
- order-insensitive equality.

---

## 37. Compact Ordered Implementation Guidance

A recommended optimized architecture is conceptually:

```text
hash index
    ↓
dense entry storage in insertion order
```

Example:

```text
hash/control structure
    bucket → entry index

entries:
    0: key, value
    1: key, value
    2: key, value
```

This design can preserve insertion order without requiring a doubly linked list per entry.

This section is non-normative implementation guidance.

---

## 38. Equality Versus Encounter Order

A Map's preserved encounter order is metadata for traversal, not part of mapping identity.

Therefore equal Maps may iterate differently.

Example:

```phalcom
const a = {
    x: 1,
    y: 2,
}

const b = {
    y: 2,
    x: 1,
}

a == b
// true
```

but:

```text
a.keys encounter order:
    #x, #y

b.keys encounter order:
    #y, #x
```

This is intentional and mirrors the conceptual distinction between extensional association equality and deterministic traversal.

---

## 39. Printing and Serialization

Where Map printing or deterministic serialization preserves entry order, it MUST use insertion encounter order unless the relevant higher-level format specifies otherwise.

Printing order does not alter equality semantics.

Detailed formatting, cycle handling, and serialization policies are deferred.

---

## 40. Grouping Interaction

Operations that produce a default Map, such as grouping, naturally inherit Map encounter semantics.

For a grouping operation that inserts each group key when first encountered:

```phalcom
collection.group by: |value| {
    key
}
```

the resulting Map's key encounter order is first-seen group-key order.

Group member ordering is specified by the collection operation itself.

---

## 41. Conversion to Map

The ratified general conversion direction includes:

```phalcom
entries.toMap
```

with duplicate detection rather than silent overwriting.

Conceptually:

```text
Iterable<Entry<K,V>>.toMap
    → Result<Map<K,V>, DuplicateKeyError<K>>
```

This is consistent with Map literal duplicate rejection.

Explicit conflict resolution may use:

```phalcom
entries.toMap merging: |existing, incoming| {
    ...
}
```

The complete conversion API is specified elsewhere.

---

## 42. Static and Dynamic Duplicate Detection

Duplicate Map literal keys SHOULD be detected statically when provable.

Examples include:

```phalcom
{
    a: 1,
    a: 2,
}
```

When duplicate equivalence depends on runtime key evaluation or expanded associations, construction performs dynamic detection.

Failure occurs before the Map value is finalized.

The exact diagnostic type is deferred.

---

## 43. Key Equality and Hash Contract

Map lookup and duplicate detection depend on Phalcom's general hash/equality contract.

This specification requires only that:

- valid Map keys satisfy the language's hashability requirements;
- key equality determines association identity;
- equal keys correspond to the same logical mapping slot.

The exact object hashing protocol, cross-type numeric equality rules, and user-defined hash contract are specified elsewhere.

---

## 44. Equal-but-Nonidentical Keys

The exact object-identity behavior when inserting a key equal to but not identical to an existing key remains deferred.

At minimum, insertion MUST behave as an update to the existing logical key association rather than creating a second equal key.

Whether the Map retains the original key object or replaces it with the new equal key object is not settled by this specification.

This issue MUST be resolved in the Map edge-semantics specification.

---

## 45. Mutation During Iteration

Map views are intended to be live lightweight views.

The semantics of structural mutation while iterating:

```phalcom
map.keys
map.values
map.entries
```

remain deferred.

A later iterator specification will choose the exact behavior, likely among fail-fast, snapshot, or explicitly defined live semantics.

Nothing in this document authorizes unspecified mutation behavior.

---

## 46. Deferred Issues

The following are intentionally deferred:

1. generic inference for `{}` and heterogeneous Map literals;
2. exact `Map<K,V>` variance;
3. key retention for equal-but-nonidentical inserted keys;
4. Map view mutation-during-iteration behavior;
5. whether views implement `Sized` or indexing;
6. complete Map transformation APIs such as `mapValues`;
7. full `Set` and `ImmutableSet` semantics;
8. `ImmutableMap` or other immutable mapping types;
9. exact `HashMap` and `OrderedMap` APIs;
10. full hash/equality protocol;
11. printing and recursive representation details;
12. serialization rules;
13. explicit type-qualified literal syntax, if any;
14. whether `Map{...}` is later accepted as an optional explicit Map-literal form.

Future specifications MAY refine these areas but MUST preserve the ratified semantics here unless explicitly superseded.

---

## 47. Conformance Summary

A conforming Phalcom implementation MUST satisfy:

```text
()
    → Unit / zero product

[]
    → List

#{...}
    → Record

{}
    → empty Map

{key: value, ...}
    → Map

{value, value, ...}
    → Set

Set()
    → empty mutable Set

mixed Set/Map brace entries
    → syntax error

Map
    mutable
    arbitrary hashable keys
    order-insensitive equality
    insertion encounter order
    unhashable

bare Map key:
    name:
        → Symbol #name

computed Map key:
    [expr]:
        → evaluated key value

implicit String → Symbol conversion
    → forbidden

new Map key
    → appended to encounter order

existing Map key update
    → retains encounter position

remove + reinsert
    → moves to end

Map literal duplicate key
    → error

Map.from(record:)
    → explicit mutable Map conversion
      preserving Record encounter order

future HashMap with unspecified order
    → cannot feed ordered labeled ** expansion

future OrderedMap
    → reserved for stronger reorderable semantics,
      not merely insertion-order preservation
```
