# Phalcom Tuples, Records, Sets, Maps, and Associative Value Semantics

**Status:** Normative design specification  
**Scope:** Tuples, callable argument domains, records, sets, maps, brace literals, access, concatenation, spread, equality, hashing, structural typing, reflection, conversion, diagnostics, and runtime obligations  
**Revision:** 2026-07-29

---

## 1. Purpose

This specification defines Phalcom’s core product and associative collection model.

The ratified concrete abstractions are:

```text
Tuple
    Immutable ordered heterogeneous product.
    Slots may be positional or labeled.
    Slot order and labels are semantic.

Unit
    Exact empty-tuple type.
    Its sole value is ().

Record
    Immutable associative value.
    Its key set and bindings are fixed after construction.
    Equality and structural type identity are order-independent.

Set
    Immutable unordered unique-value collection.
    Populated unkeyed brace literals create Set values.

Map
    Mutable associative collection.
    Bindings may be inserted, replaced, and removed.

MutableSet
    Mutable unique-value collection.
```

The design intentionally separates:

```text
ordered product structure
from
unordered key/value structure
```

and:

```text
immutable values
from
mutable identity-bearing collections
```

The principal semantic relationship is:

```text
Tuple
    models ordered evaluated argument structure

Record
    models fixed associative structure

Set
    models immutable membership

Map
    models evolving associative state
```

---

## 2. Normative terminology

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

A decision marked **ratified** is part of this specification.

A decision marked **deferred** is intentionally left for another specification.

---

# Part I — Common Semantic Foundation

## 3. Values, entities, and shallow immutability

Tuple, Unit, Record, and Set are immutable structural values.

Map and MutableSet are mutable collection entities.

Immutability is shallow.

```phalcom
const value = {
  preferences: Map<Symbol, Any>.new()
}
```

The record binding from `#preferences` to the map cannot change:

```phalcom
value.preferences = anotherMap
// invalid
```

The referenced map may still mutate:

```phalcom
value.preferences.put(#theme, #dark)
```

Therefore:

```text
immutable aggregate binding
≠
deep transitive immutability
```

Deep freezing is outside this specification.

---

## 4. Concrete hierarchy

The conceptual runtime hierarchy is:

```text
Object
├── Tuple
│   └── Unit
├── Record
├── Set
├── Map
└── MutableSet
```

Phalcom MUST NOT require a large monolithic collection superclass containing operations that do not apply coherently to all collection kinds.

Shared behavior SHOULD be expressed through focused protocols.

---

## 5. Shared protocols

Illustrative protocol shapes:

```phalcom
@protocol
class Iterable<out T> {
  iterator() -> Iterator<T>
}
```

```phalcom
@protocol
class Sized {
  size() -> Int
}
```

```phalcom
@protocol
class Mapping<K, out V> {
  at(key: K) -> V
  find(key: K) -> Option<V>
  containsKey(key: K) -> Bool

  keys() -> Iterable<K>
  values() -> Iterable<V>
  entries() -> Iterable<(K, V)>
}
```

```phalcom
@protocol
class MutableMapping<K, V> {
  put(key: K, value: V) -> ()
  remove(key: K) -> Option<V>
}
```

```phalcom
@protocol
class SetLike<T> {
  contains(value: T) -> Bool
}
```

The final variance of key parameters must be validated against complete protocol signatures. This document ratifies the protocol-oriented architecture, not every illustrative variance marker.

---

## 6. Canonical size operation

The canonical generic cardinality operation is:

```phalcom
size() -> Int
```

The `Sized` protocol therefore requires:

```phalcom
value.size()
```

### 6.1 Collision-sensitive aggregates

Tuple, Record, and Map MUST expose cardinality through `size()`.

They MUST NOT reserve the getter selector `size` for cardinality.

```phalcom
const tuple = (size: 42)
const record = { size: 42 }

tuple.size
// 42

record.size
// 42

tuple.size()
// 1

record.size()
// 1
```

### 6.2 Collision-free built-ins

Built-ins that do not expose arbitrary user-defined getter fields MAY also provide:

```phalcom
string.size
bytes.size
list.size
set.size
```

They SHOULD still implement:

```phalcom
string.size()
bytes.size()
list.size()
set.size()
```

Generic code MUST use `size()`.

### 6.3 Deferred magic-size hook

No Python-like hidden `_size`, `__size__`, or global `len` mechanism is introduced here. The ordinary selector `size()` already avoids field collisions.

---

# Part II — Tuples

## 7. Definition

A tuple is an immutable ordered heterogeneous product.

Each slot has:

```text
index
optional label
value
```

Example:

```phalcom
(a, b, c: d)
```

Conceptual shape:

```text
slot 0
    index: 0
    label: None
    value: a

slot 1
    index: 1
    label: None
    value: b

slot 2
    index: 2
    label: Some(#c)
    value: d
```

Tuple order is semantic.

Tuple labels are semantic.

---

## 8. Syntax

### 8.1 Empty tuple

```phalcom
()
```

This is the empty tuple and unit value.

### 8.2 One positional slot

```phalcom
(value,)
```

Without the comma:

```phalcom
(value)
```

is grouping.

### 8.3 Multiple positional slots

```phalcom
(first, second)
```

### 8.4 Labeled slots

```phalcom
(x: 10, y: 20)
```

### 8.5 Mixed slots

```phalcom
(request, encoding: #utf8, timeout: duration)
```

Positional and labeled tuple slots may coexist.

---

## 9. Tuple types

Tuple types use the same structural form:

```phalcom
(Int, String)
```

```phalcom
(name: String, age: Int)
```

```phalcom
(Request, encoding: Symbol, timeout: Duration)
```

One slot:

```phalcom
(Int,)
```

Empty tuple type:

```phalcom
()
```

Tuple type identity includes:

- arity;
- slot order;
- positional-versus-labeled status;
- exact label;
- slot type.

Therefore:

```phalcom
(x: Int, y: Int)
```

is distinct from:

```phalcom
(y: Int, x: Int)
```

and:

```phalcom
(x: Int)
```

is distinct from:

```phalcom
(Int,)
```

---

## 10. Unit

`Unit` is the reflective runtime type name of the exact empty tuple type.

```text
Unit <: Tuple
```

The sole value is:

```phalcom
()
```

The runtime MUST canonicalize it:

```phalcom
() === ()
// true
```

Valid uses:

```phalcom
const completed: () = ()
```

```phalcom
const values = [(), (), ()]
```

```phalcom
const optional: Option<()> = Some(())
```

Unit is distinct from absence:

```phalcom
() == None
// false
```

Unit is immutable and hashable.

```phalcom
().size()
// 0
```

---

## 11. Tuple access

### 11.1 Positional access

Canonical form:

```phalcom
tuple.at(index)
```

Bracket syntax MAY lower to the same operation:

```phalcom
tuple[index]
```

Invalid index:

```text
BoundsError
```

### 11.2 Getter label access

A labeled slot MAY be read through getter syntax when ordinary getter dispatch does not resolve.

```phalcom
const point = (x: 10, y: 20)

point.x
// 10
```

### 11.3 Explicit label access

Labels MUST remain accessible even when they collide with methods.

```phalcom
tuple.at(label: #name)
```

Bracket syntax MAY support:

```phalcom
tuple[#name]
```

Example:

```phalcom
const value = (class: Int)

value.class
// Tuple

value.at(label: #class)
// Int
```

### 11.4 Getter precedence

For:

```phalcom
receiver.identifier
```

the runtime MUST:

1. perform ordinary getter-message dispatch;
2. if dispatch fails, attempt tuple-label fallback;
3. if no matching label exists, produce the ordinary message-not-understood failure.

Tuple labels never override ordinary getter methods.

---

## 12. Collisions are general, not special-cased

`class` is an ordinary getter selector inherited by all objects.

`#class` is a selector value like any other.

Therefore:

```phalcom
const value = (class: Int)

value.class
// Tuple
```

Phalcom MUST NOT special-case only `class`. The general dispatch rule applies to every collision.

---

## 13. Duplicate labels

A tuple MUST NOT contain the same label more than once.

Invalid:

```phalcom
(x: 10, x: 20)
```

Static duplicate: compile-time error.

Runtime/reflection-produced duplicate:

```text
DuplicateTupleLabelError
```

The diagnostic SHOULD identify the label and conflicting slot positions.

---

## 14. Immutability

Tuple slots, labels, order, and arity are immutable.

Invalid:

```phalcom
tuple[0] = value
```

```phalcom
tuple.x = value
```

```phalcom
tuple.append(value)
```

```phalcom
tuple.removeAt(0)
```

Mutable internal construction is permitted only before publication.

---

## 15. Tuple concatenation with `+(_)`

Tuple defines the ordinary operator method:

```phalcom
+(_)
```

For tuples:

```phalcom
left + right
```

produces a tuple containing all left slots followed by all right slots.

```phalcom
const left = (1, name: "Altun")
const right = (true, age: 30)

const combined = left + right
// (1, name: "Altun", true, age: 30)
```

### 15.1 Type effect

```text
(A, label: B) + (C, other: D)
    =
(A, label: B, C, other: D)
```

```phalcom
const a: (Int, name: String) = ...
const b: (Bool, age: Int) = ...

const c = a + b
// (Int, name: String, Bool, age: Int)
```

### 15.2 Identity

```phalcom
() + tuple == tuple
tuple + () == tuple
```

Because tuples are immutable, identity MAY be preserved:

```phalcom
() + tuple === tuple
tuple + () === tuple
```

### 15.3 Associativity

Where labels remain globally unique:

```text
(a + b) + c = a + (b + c)
```

Concatenation is not commutative.

### 15.4 Duplicate labels

```phalcom
(x: 10) + (x: 20)
// error
```

Static duplicate: compile-time error.

Dynamic duplicate:

```text
DuplicateTupleLabelError
```

### 15.5 No implicit flattening

```phalcom
(1, (2, 3)) + (4,)
// (1, (2, 3), 4)
```

It does not become:

```phalcom
(1, 2, 3, 4)
```

---

## 16. Iteration

Ordinary tuple iteration yields values in slot order.

```phalcom
for value in tuple {
  ...
}
```

A reflective API MAY expose:

```phalcom
tuple.entries()
```

with conceptual entries:

```phalcom
@data @immutable
class TupleSlot {
  const _index: Int
  const _label: Option<Symbol>
  const _value: Any
}
```

Tuple iteration order is semantic.

---

## 17. Equality

Tuple equality is structural and order-sensitive.

Two tuples are equal when:

- arity matches;
- positional/labeled status matches at each slot;
- labels match;
- values match.

```phalcom
(1, 2) == (1, 2)
// true
```

```phalcom
(x: 1, y: 2) == (x: 1, y: 2)
// true
```

```phalcom
(x: 1, y: 2) == (y: 2, x: 1)
// false
```

```phalcom
(x: 1) == (1,)
// false
```

---

## 18. Hashing

A tuple is hashable only when every slot value is stably hashable.

Its hash MUST incorporate:

- arity;
- slot index;
- positional/labeled status;
- label;
- value hash.

Hashing MUST agree with equality.

Unit has a constant stable hash.

---

## 19. Reflection

Tuple type reflection SHOULD expose ordered slot descriptors:

```phalcom
tupleType.slots()
```

Conceptual descriptor:

```phalcom
@data @immutable
class TupleSlotType {
  const _index: Int
  const _label: Option<Symbol>
  const _type: Type
}
```

Value reflection SHOULD expose:

```phalcom
tuple.size()
tuple.labels()
tuple.entries()
```

---

# Part III — Callable Domains and Rest

## 20. Tuple-shaped callable domains

Callable types use a tuple-shaped parameter domain:

```phalcom
(Int, String, labeled: Bool) -> Result
```

Examples:

```phalcom
() -> Result
(Int,) -> Result
(Int, String) -> Result
(Request, timeout: Duration) -> Response
() -> ()
```

Normative relationship:

```text
tuple shape
≈ evaluated argument shape
≈ callable parameter domain
```

---

## 21. No separate public `Arguments`

An evaluated argument packet is a Tuple.

```phalcom
forward(prefix, *rest) {
  return target(*rest)
}
```

`rest` preserves:

- positional slots;
- labels;
- exact order;
- evaluated values.

A separate public `Arguments` class MUST NOT be introduced unless a genuinely distinct semantic responsibility is discovered.

A reflective `Message` MAY contain a tuple plus selector, receiver, and source metadata.

---

## 22. Rest position and selector spelling

Rest capture MUST be the final parameter.

Valid conceptual selector spelling:

```text
method(_,labeled,*)
```

Invalid:

```text
method(*,_,labeled)
method(_,*,labeled)
```

No fixed parameter may follow rest capture.

Selector examples MUST use the compiler/VM’s exact encoded spelling.

A selector-shaped symbol stored in a collection remains data:

```phalcom
const handlers = {
  #method(_,labeled,*): handler
}
```

It does not install behavior.

---

## 23. Empty expansion

```phalcom
target(*())
```

is equivalent to:

```phalcom
target()
```

The compiler MAY eliminate empty expansion.

---

## 24. Argument composition

```phalcom
forward(prefix, *rest) {
  return target(*((prefix,) + rest))
}
```

Duplicate labels remain errors.

---

# Part IV — Records

## 25. Definition

A Record is an immutable associative value.

A record has:

- a finite key set;
- one binding per key;
- a value per key;
- non-semantic presentation order.

After construction, keys and bindings cannot change.

```phalcom
const user = {
  name: "Altun"
  age: 30
}
```

Invalid:

```phalcom
user[#age] = 31
```

A change creates a new record.

---

## 26. Literal grammar

Associative brace literals produce Record.

Supported entries:

```phalcom
{
  identifier: value
  #selectorLikeSymbol: value
  [computedKey]: value
  ...spreadSource
}
```

Example:

```phalcom
{
  name: "Altun"
  #method(_,labeled,*): handler
  [Symbol("display name")]: "Altun Hasanli"
  [extensionKey]: extensionValue
  ...base
}
```

Unkeyed entries belong to Set and MUST NOT mix with associative entries.

---

## 27. Identifier keys are symbols

```phalcom
name: value
```

uses:

```phalcom
#name
```

Therefore:

```phalcom
{ name: "Altun" }
```

and:

```phalcom
{ #name: "Altun" }
```

have the same contents.

No hidden `FieldName` key type is introduced.

---

## 28. Selector-like keys remain data

```phalcom
const handlers = {
  #render(_): renderer
}
```

This does not install `render(_)`.

```phalcom
handlers[#render(_)]
```

is data lookup.

```phalcom
handlers.render(value)
```

is ordinary method dispatch.

---

## 29. Getter fallback

```phalcom
const user = {
  name: "Altun"
}

user.name
// "Altun"
```

is permitted only when ordinary getter dispatch fails.

### 29.1 Precedence

For `receiver.identifier`:

1. ordinary getter dispatch;
2. record lookup for `#identifier`;
3. ordinary dispatch failure.

### 29.2 Collisions

```phalcom
const value = {
  class: Int
}

value.class
// Record
```

Explicit lookup:

```phalcom
value[#class]
// Int
```

or:

```phalcom
value.at(#class)
// Int
```

### 29.3 Parenthesized methods

Keys never participate in parenthesized dispatch:

```phalcom
record.size()
record.entries()
record.find(key)
```

---

## 30. Lookup API

Throwing lookup:

```phalcom
record.at(key) -> V
```

Absent key:

```text
MissingKeyError
```

Optional lookup:

```phalcom
record.find(key) -> Option<V>
```

Absent key:

```phalcom
None
```

Stored `None` remains distinguishable:

```phalcom
const record = {
  value: None
}

record.find(#value)
// Some(None)
```

Bracket lookup SHOULD lower to throwing lookup:

```phalcom
record[key]
```

---

## 31. Size

```phalcom
record.size()
```

reports binding count.

```phalcom
const record = { size: 42 }

record.size
// 42

record.size()
// 1
```

---

## 32. Functional updates

Record bindings are immutable.

Conceptual API:

```phalcom
record.setting(key, to: value)
record.removing(key)
record.merging(other)
```

The exact selector names may be refined, but the semantics are ratified.

```phalcom
const person = {
  name: "Altun"
  age: 30
}

const changed =
  person.setting(#age, to: 31)
```

`person` remains unchanged.

---

# Part V — Record Types

## 33. Exact structural types

Known-key literals infer exact structural types.

```phalcom
const person = {
  name: "Altun"
  age: 30
}
```

Type:

```phalcom
{
  name: String
  age: Int
}
```

Field order does not affect type identity.

```phalcom
{
  name: String
  age: Int
}
```

and:

```phalcom
{
  age: Int
  name: String
}
```

are the same type.

Exact-shape constraint syntax is deferred.

---

## 34. Width subtyping

```text
{name: String, age: Int}
    <:
{name: String}
```

```phalcom
displayName(user: { name: String }) -> () {
  System.print(user.name)
}
```

Valid:

```phalcom
displayName({
  name: "Altun"
  age: 30
})
```

---

## 35. Field covariance

```text
Dog <: Animal
──────────────────────────────
{pet: Dog} <: {pet: Animal}
```

This is sound because immutable record fields cannot be replaced.

---

## 36. Broad records

Fully dynamic-key records use:

```phalcom
Record<K, V>
```

```phalcom
const record = {
  [firstKey]: firstValue
  [secondKey]: secondValue
}
```

The checker infers joins for `K` and `V`.

---

## 37. Mixed static and computed keys

```phalcom
const metadata = {
  name: "Altun"
  [extensionKey]: extensionValue
}
```

Normative type:

```phalcom
{ name: String } & Record<K, V>
```

Known fields are preserved through intersection.

A first implementation MAY widen to `Record<K, V>` only as a temporary implementation limitation.

No dedicated row-polymorphism syntax is required.

---

## 38. Update type transformations

Starting type:

```phalcom
{
  name: String
  age: Int
}
```

Replacing `age` with `Int` preserves the type.

Replacing `age` with `String` produces:

```phalcom
{
  name: String
  age: String
}
```

Adding `active: Bool` produces:

```phalcom
{
  name: String
  age: Int
  active: Bool
}
```

Removing `name` produces:

```phalcom
{
  age: Int
}
```

Dynamic updates may add or widen a `Record<K, V>` intersection component.

---

# Part VI — Spread and Duplicate Rules

## 39. Spread evaluation

```phalcom
{
  ...source
  key: value
}
```

Entries and spreads evaluate left to right.

Spread sources must provide mapping entries.

---

## 40. Last-wins spread

Collisions involving a spread are source-order last-wins.

```phalcom
{
  ...person
  age: 31
}
```

The explicit `age` wins.

```phalcom
{
  age: 31
  ...person
}
```

The spread wins if it contains `age`.

Layering:

```phalcom
{
  ...defaults
  ...environment
  ...commandLine
}
```

Later sources override earlier sources.

---

## 41. Direct duplicates are errors

Invalid:

```phalcom
{
  name: "Altun"
  name: "Hasanli"
}
```

Also invalid:

```phalcom
{
  name: "Altun"
  #name: "Hasanli"
}
```

The compiler normalizes identifier keys to symbols for duplicate detection.

---

## 42. Computed direct duplicates

```phalcom
const key = #name

{
  name: "Altun"
  [key]: "Hasanli"
}
```

raises:

```text
DuplicateKeyError
```

No spread merge boundary exists, so silent replacement is forbidden.

---

## 43. Computed override after spread

```phalcom
{
  ...base
  [key]: replacement
}
```

may override a binding from `base`.

Likewise:

```phalcom
{
  [key]: initial
  ...base
}
```

allows `base` to win.

---

## 44. Presentation position

Override preserves first insertion position.

```phalcom
const base = {
  name: "Altun"
  age: 30
}

const changed = {
  ...base
  name: "Altun Hasanli"
}
```

Presentation remains:

```phalcom
{
  name: "Altun Hasanli"
  age: 30
}
```

Replacement does not move the key.

---

# Part VII — Record Equality, Hashing, and Order

## 45. Equality

Record equality is structural and order-independent.

```phalcom
{
  name: "Altun"
  age: 30
}
==
{
  age: 30
  name: "Altun"
}
// true
```

---

## 46. Record versus Map

Record equality applies only between records.

```phalcom
record == map
// false
```

Explicit mapping-content comparison MAY be provided separately.

---

## 47. Hashability

A record is hashable only when every key and value participating in equality is stably hashable.

The hash is order-independent.

Equal records with different presentation orders must hash equally.

A record containing hash-unstable mutable contents is not structurally hashable.

---

## 48. Presentation order

Records preserve construction order for:

- iteration;
- printing;
- debugging;
- reflection;
- default serialization.

Presentation order does not affect:

- equality;
- hashing;
- structural type identity;
- width subtyping.

A canonical serializer may explicitly reorder keys.

---

## 49. Iteration

Record iteration is deterministic and presentation ordered.

Explicit APIs SHOULD include:

```phalcom
record.keys()
record.values()
record.entries()
```

Whether direct `for` iteration yields entries or requires `entries()` is deferred.

---

## 50. Reflection

Exact record type reflection SHOULD expose:

```phalcom
recordType.fields()
```

Conceptual descriptor:

```phalcom
@data @immutable
class RecordFieldType {
  const _key: Any
  const _type: Type
  const _presentationIndex: Int
}
```

Type equality ignores `presentationIndex`.

Broad reflection SHOULD expose:

```phalcom
recordType.keyType
recordType.valueType
recordType.isExact
```

Mixed shapes use general intersection types.

---

# Part VIII — Sets

## 51. Definition

`Set<T>` is an immutable unordered collection of unique values.

```phalcom
{a, b, c}
```

creates a Set.

---

## 52. Literal classification

Unkeyed brace entries create Set.

Associative entries create Record.

Mixing is invalid:

```phalcom
{
  a
  b
  name: value
}
```

---

## 53. Duplicates

Set construction is idempotent.

```phalcom
{a, a, b}
```

has the same membership as:

```phalcom
{a, b}
```

Statically obvious duplicates MAY warn.

Runtime duplicates collapse and do not error.

---

## 54. Ordering

Set semantic order is undefined.

The runtime SHOULD preserve first insertion order for deterministic:

- iteration;
- printing;
- debugging;
- serialization.

Insertion order does not affect equality, hashing, membership, or type identity.

---

## 55. Equality and hashing

Set equality is membership-based and order-independent.

An immutable set is hashable only when every member is stably hashable.

A mutable set is not structurally hashable.

---

## 56. Empty set

Bare `{}` is not an empty set.

Canonical construction:

```phalcom
Set<T>.new()
```

or, where inferred:

```phalcom
Set.new()
```

---

## 57. MutableSet

Mutable set behavior belongs to:

```phalcom
MutableSet<T>
```

```phalcom
const values = MutableSet<String>.new()

values.add("a")
values.remove("a")
```

---

# Part IX — Maps

## 58. Definition

`Map<K, V>` is a mutable associative collection.

Conceptual operations:

```phalcom
map.put(key, value) -> ()
map.remove(key) -> Option<V>
map.at(key) -> V
map.find(key) -> Option<V>
map.containsKey(key) -> Bool
map.size() -> Int
```

---

## 59. Empty construction

Canonical typed form:

```phalcom
Map<K, V>.new()
```

```phalcom
const users = Map<String, User>.new()
```

The canonical form is not:

```phalcom
Map.new<K, V>()
```

### 59.1 Inference

Expected type may infer arguments:

```phalcom
const users: Map<String, User> = Map.new()
```

Checked code without context:

```phalcom
const values = Map.new()
```

SHOULD require type arguments or produce a strong inference diagnostic.

Unchecked code may treat it as `Map<Dynamic, Dynamic>`.

---

## 60. Trailing closures are not map-entry syntax

Phalcom already defines:

```phalcom
Something.method(a, b) { value =>
  value ** 2
}
```

as sugar for:

```phalcom
Something.method(
  a,
  b,
  { value => value ** 2 }
)
```

Therefore:

```phalcom
Map.new {
  name: value
}
```

is a call receiving a closure.

It is not special associative-entry syntax.

---

## 61. Conversion from entries

Canonical form:

```phalcom
Map<K, V>.from(entries: source)
```

where `source` conforms to:

```phalcom
Iterable<(K, V)>
```

```phalcom
const users =
  Map<String, User>.from(entries: userPairs)
```

This supports lists, tuples, generators, and custom iterables of pairs.

Duplicate keys in declarative `from(entries:)` construction raise:

```text
DuplicateKeyError
```

---

## 62. Conversion from mappings

Canonical form:

```phalcom
Map<K, V>.from(mapping: source)
```

```phalcom
const mutable =
  Map<Symbol, Any>.from(mapping: record)
```

The result is a mutable copy.

---

## 63. Inferred conversion

Where source types suffice:

```phalcom
const map = Map.from(entries: pairs)
```

```phalcom
const map = Map.from(mapping: record)
```

the checker MAY infer `K` and `V`.

Explicit specialized forms remain canonical.

---

## 64. Mutation

```phalcom
map.put(key, first)
map.put(key, second)
```

The second call replaces the first binding.

`remove(key)` returns:

```phalcom
Some(previousValue)
```

or:

```phalcom
None
```

---

## 65. Equality and hashing

Map is mutable and identity-bearing.

Ordinary Map equality is identity-oriented.

Map is not structurally hashable by contents.

Explicit content-comparison helpers may exist separately.

---

## 66. Presentation order

Map SHOULD preserve first insertion order.

Replacing an existing key preserves its position.

Removing and later reinserting may move it to the end.

---

## 67. Freezing

```phalcom
const record = map.freeze()
```

returns an immutable Record snapshot.

The result generally has type:

```phalcom
Record<K, V>
```

Freezing does not recover an exact compile-time key shape.

A future checked refinement API is deferred.

---

# Part X — Brace Literal Classification

## 68. Populated forms

```phalcom
{a, b, c}
```

creates immutable Set.

```phalcom
{
  name: value
  age: value
}
```

creates immutable Record.

```phalcom
{
  [key]: value
}
```

creates immutable Record.

```phalcom
{
  ...record
  age: 31
}
```

creates immutable Record.

Mixing unkeyed and associative entries is an error.

---

## 69. Empty braces

```phalcom
{}
```

is the empty Record.

Ratified family:

```text
()
    empty ordered product
    Unit

{}
    empty associative product
    Record
```

Bare `{}` is never contextually reinterpreted as Set or Map.

Explicit empty alternatives:

```phalcom
Set<T>.new()
Map<K, V>.new()
MutableSet<T>.new()
```

---

# Part XI — Diagnostics

## 70. Required diagnostics

### 70.1 Duplicate tuple label

```text
DuplicateTupleLabelError
```

### 70.2 Duplicate direct record key

Compile-time error when statically known.

Runtime:

```text
DuplicateKeyError
```

### 70.3 Duplicate declarative map-entry key

```text
DuplicateKeyError
```

### 70.4 Missing key

```text
MissingKeyError
```

### 70.5 Invalid tuple index

```text
BoundsError
```

### 70.6 Invalid rest position

```text
rest parameter must be the final parameter
```

### 70.7 Mixed brace entry categories

```text
brace literal cannot mix unkeyed set elements with associative entries
```

### 70.8 Unconstrained empty Map

```text
cannot infer key and value types for Map.new()
use Map<K, V>.new() or provide an expected type
```

---

## 71. Collision tooling

A tuple label or record key may legally collide with an ordinary getter.

IDE tooling SHOULD distinguish the ordinary method and explicit key access.

```phalcom
const value = {
  class: Int
}
```

`value.class` resolves to the Object getter.

`value[#class]` resolves the record binding.

The compiler must not reject the record merely because of the collision.

---

# Part XII — Runtime and Optimization

## 72. Tuple representation

Implementations MAY use:

- inline fixed slots;
- specialized small-tuple layouts;
- shared shape descriptors;
- canonical label tables;
- boxed or tagged values.

Tuple shape descriptors SHOULD be interned.

---

## 73. Unit optimization

`()` MAY be represented as:

- an immediate tag;
- an immortal singleton;
- a reserved object handle;
- a VM constant.

A dedicated instruction such as:

```text
RETURN_UNIT
```

may implement callable fallthrough.

The compiler may eliminate:

```phalcom
target(*())
```

to:

```phalcom
target()
```

---

## 74. Tuple concatenation optimization

The runtime MAY:

- allocate one combined tuple;
- reuse shape descriptors;
- return an operand for unit identity;
- specialize small arities;
- eliminate intermediate tuples during immediate expansion.

Semantics remain immutable and duplicate-label safe.

---

## 75. Record representation

Record MAY use:

- compact shape plus value array;
- persistent hash trie;
- immutable ordered hash map;
- specialized small-record layouts;
- structural sharing.

Representation is non-normative.

The implementation MUST preserve:

- fixed bindings;
- order-independent equality/hash;
- first-insertion presentation order;
- last-wins spread values;
- first-position retention on override.

---

## 76. Shape interning

Exact structural record types and runtime shapes SHOULD be interned where practical.

Semantic shape identity ignores presentation order.

Presentation order may remain instance-specific.

---

## 77. Persistent updates

Functional updates SHOULD avoid full copying where practical.

```phalcom
const changed =
  record.setting(#age, to: 31)
```

may share all unchanged structure with `record`.

---

## 78. Set representation

Set MAY use persistent set structures or compact immutable layouts.

Duplicate construction elements collapse.

Presentation order metadata may be separate from membership representation.

---

## 79. Map representation

Map MAY use an ordered hash table.

Replacement preserves first insertion position.

No structural hash caching is permitted.

---

# Part XIII — Conformance Examples

## 80. Tuple

```phalcom
const a = (10, x: 20)
const b = (true, y: "value")

const combined = a + b

Assert.equal(combined, (10, x: 20, true, y: "value"))
Assert.equal(combined.size(), 4)
Assert.equal(combined.x, 20)
Assert.equal(combined.at(label: #y), "value")
```

```phalcom
Assert.same(() + a, a)
Assert.same(a + (), a)
```

---

## 81. Getter collisions

```phalcom
const tuple = (
  class: Int,
  size: 42
)

Assert.same(tuple.class, Tuple)
Assert.equal(tuple.size, 42)
Assert.equal(tuple.size(), 2)
Assert.same(tuple.at(label: #class), Int)
```

```phalcom
const record = {
  class: Int
  size: 42
}

Assert.same(record.class, Record)
Assert.equal(record.size, 42)
Assert.equal(record.size(), 2)
Assert.same(record[#class], Int)
```

---

## 82. Record spread

```phalcom
const base = {
  name: "Altun"
  age: 30
}

const updated = {
  ...base
  age: 31
}

Assert.equal(updated.age, 31)
Assert.equal(base.age, 30)
```

---

## 83. Direct duplicate rejection

```phalcom
{
  name: "Altun"
  name: "Hasanli"
}
// compile-time error
```

```phalcom
const key = #name

{
  name: "Altun"
  [key]: "Hasanli"
}
// DuplicateKeyError
```

---

## 84. Record equality

```phalcom
const left = {
  name: "Altun"
  age: 30
}

const right = {
  age: 30
  name: "Altun"
}

Assert.true(left == right)
Assert.equal(left.hash, right.hash)
```

---

## 85. Set

```phalcom
const values = {1, 2, 2, 3}

Assert.equal(values.size(), 3)
Assert.true(values.contains(2))
Assert.true({1, 2} == {2, 1})
```

---

## 86. Map

```phalcom
const users = Map<String, User>.new()

users.put("a", first)
users.put("a", second)

Assert.same(users.at("a"), second)
```

```phalcom
const copied =
  Map<Symbol, Any>.from(mapping: {
    name: "Altun"
    age: 30
  })
```

```phalcom
const entries = [
  (#name, "Altun"),
  (#age, 30)
]

const map =
  Map<Symbol, Any>.from(entries: entries)
```

---

# Part XIV — Ratified Decision Index

## 87. Tuples

| Decision | Status |
|---|---|
| Immutable ordered heterogeneous product | Ratified |
| Positional and labeled slots may coexist | Ratified |
| Order is semantic | Ratified |
| Labels are semantic | Ratified |
| Duplicate labels are forbidden | Ratified |
| Index and explicit-label access | Ratified |
| Getter access is fallback after normal dispatch | Ratified |
| Ordinary getters win collisions | Ratified |
| `size()` is canonical cardinality | Ratified |
| `+(_)` concatenates tuples | Ratified |
| `()` is concatenation identity | Ratified |
| Concatenation rejects duplicate labels | Ratified |
| Equality is order-sensitive and label-sensitive | Ratified |
| Hashability is conditional | Ratified |
| Iteration yields values in order | Ratified |
| Tuple represents evaluated argument packets | Ratified |
| No separate public `Arguments` type | Ratified |

---

## 88. Callable domains and rest

| Decision | Status |
|---|---|
| Callable domain uses tuple shape | Ratified |
| `() -> R` has zero arguments | Ratified |
| `((),) -> R` has one unit argument | Ratified |
| Rest capture is final | Ratified |
| Correct selector example is `method(_,labeled,*)` | Ratified |
| `target(*())` supplies zero arguments | Ratified |

---

## 89. Records

| Decision | Status |
|---|---|
| Immutable fixed bindings | Ratified |
| Shallow immutability | Ratified |
| Identifier keys are Symbols | Ratified |
| `name: value` equals `#name: value` in contents | Ratified |
| Selector-like keys remain data | Ratified |
| Computed keys are permitted | Ratified |
| Getter field access follows ordinary dispatch | Ratified |
| Explicit lookup resolves collisions | Ratified |
| `size()` reports binding count | Ratified |
| `at` throws and `find` returns Option | Ratified |
| Known-key literals get exact structural types | Ratified |
| Field order does not affect type identity | Ratified |
| Width subtyping | Ratified |
| Immutable fields are covariant | Ratified |
| Dynamic records use `Record<K, V>` | Ratified |
| Mixed records preserve known shape through intersection | Ratified |
| Direct duplicate keys are errors | Ratified |
| Spread collisions are last-wins | Ratified |
| Override retains first presentation position | Ratified |
| Equality is structural and order-independent | Ratified |
| Record does not equal Map | Ratified |
| Hashability is conditional and order-independent | Ratified |
| Presentation order is preserved but non-semantic | Ratified |

---

## 90. Sets

| Decision | Status |
|---|---|
| Set is immutable | Ratified |
| `{a, b, c}` creates Set | Ratified |
| Duplicate elements collapse | Ratified |
| Obvious duplicates may warn | Ratified |
| Equality and hashing are order-independent | Ratified |
| Iteration preserves non-semantic insertion order | Ratified |
| Empty Set is explicit | Ratified |
| Mutable behavior belongs to MutableSet | Ratified |

---

## 91. Maps

| Decision | Status |
|---|---|
| Map is mutable | Ratified |
| Canonical construction is `Map<K, V>.new()` | Ratified |
| `Map.new<K, V>()` is not canonical | Ratified |
| `Map.new { ... }` is not entry syntax | Ratified |
| Conversion from entries uses `from(entries:)` | Ratified |
| Conversion from mappings uses `from(mapping:)` | Ratified |
| Declarative entry duplicates are errors | Ratified |
| `put` replaces bindings | Ratified |
| `size()` reports binding count | Ratified |
| Equality is identity-oriented | Ratified |
| Map is not structurally hashable | Ratified |
| `freeze()` produces broad Record | Ratified |

---

## 92. Brace literals

| Syntax | Meaning |
|---|---|
| `()` | Unit / empty Tuple |
| `{}` | Empty Record |
| `{a, b}` | Immutable Set |
| `{name: value}` | Immutable Record |
| `{[key]: value}` | Immutable Record |
| `{...record, key: value}` | Record with last-wins spread |
| `{a, name: value}` | Error |
| `Map<K, V>.new()` | Empty mutable Map |
| `Set<T>.new()` | Empty immutable Set |
| `MutableSet<T>.new()` | Empty mutable Set |

---

# Part XV — Deferred Work

## 93. Exact record constraints

Syntax for demanding exact record shape is deferred.

Possible future form:

```phalcom
exact { name: String }
```

No syntax is ratified here.

---

## 94. Deep immutability

Deep transitive immutability and deep freezing are outside this specification.

---

## 95. Magic collection hooks

Python-like hidden collection hooks are deferred.

The public protocol method `size()` is sufficient for the ratified model.

---

## 96. Selector object model

Selector-shaped symbols are ordinary keys here.

Whether `Selector` is a distinct runtime type from `Symbol` remains outside this specification.

---

## 97. Default record iteration element

Deterministic presentation-order iteration is ratified.

Whether direct iteration yields `(key, value)` or requires `entries()` is deferred.

---

# 98. Final Semantic Model

```text
Tuple
    immutable
    ordered
    heterogeneous
    positional and labeled
    order-sensitive equality
    conditional hashing
    concatenated with +(_)
    models evaluated arguments

Unit
    exact empty Tuple
    sole value ()
    tuple-concatenation identity

Record
    immutable
    associative
    fixed bindings
    symbol identifier keys
    computed keys allowed
    ordinary getter dispatch wins collisions
    exact structural typing for known keys
    width subtyping
    covariant fields
    last-wins spread
    direct duplicates rejected
    order-independent equality/hash
    presentation order preserved

Set
    immutable
    unique membership
    unkeyed brace literal
    duplicates collapse
    order-independent equality/hash
    presentation order preserved

Map
    mutable
    associative
    explicit typed construction
    conversion from entries or mappings
    replacement through put
    identity-oriented equality
    not structurally hashable
    freeze to Record

MutableSet
    mutable unique membership
```

This model gives Phalcom a consistent value-oriented collection foundation while preserving ordinary message dispatch, explicit mutable structures, precise structural typing, deterministic presentation, and exact tuple/call-shape correspondence.
