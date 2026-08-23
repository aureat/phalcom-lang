# Phalcom Product Normalization and Unit Specification

**Status:** Ratified language design specification
**Scope:** `Unit`, the zero-arity product, normalization of empty Tuple and closed empty Record forms, compile-time and runtime canonicalization, interaction with variadic capture/expansion, runtime representation, and distinctions from empty collection canonicalization.
**Out of scope:** Full Tuple and Record semantics, row-polymorphism syntax, generic specialization internals beyond required normalization behavior, complete collection literal rules, and nominal empty-class semantics except where contrasted with structural products.

---

## 1. Purpose

Phalcom follows the mathematical and type-theoretic interpretation of `Unit` as the canonical zero-arity product.

Tuple and Record are both structural product families:

```text
Tuple
    ordered positional coordinates
    +
    ordered labeled coordinates

Record
    unordered Symbol-labeled coordinates
```

For positive arity, these product families are semantically distinct.

At arity zero, there are no coordinates on which positionality, labels, or order can differ. The two empty structural product constructions therefore normalize to the same canonical type and value:

```phalcom
()
#{}
```

Both denote:

```text
Unit
```

This normalization is definitional. It is not an implicit conversion between distinct runtime values.

---

## 2. Normative Terminology

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

### 2.1 Product

A **product** is a structural value formed from zero or more component coordinates.

Tuple and Record are Phalcom's two structural product families.

### 2.2 Zero-arity product

A **zero-arity product** is a product with no component coordinates.

It has exactly one possible value.

### 2.3 Unit

`Unit` is Phalcom's canonical zero-arity product type.

Its unique value is written:

```phalcom
()
```

### 2.4 Product normalization

**Product normalization** is the semantic canonicalization rule by which any closed zero-coordinate structural product is represented as `Unit`.

### 2.5 Definitional equality

Two type/value constructions are **definitionally equal** when semantic elaboration canonicalizes them to the same type/value rather than inserting a conversion between them.

### 2.6 Closed Record

A **closed Record** has an exact, complete field set.

A closed Record with zero fields normalizes to `Unit`.

### 2.7 Open Record row

An **open Record row** denotes a Record shape that may contain additional fields beyond those explicitly known.

A zero-explicit-field open row does not imply zero actual fields and therefore does not normalize to `Unit`.

---

## 3. Unit as the Zero-Product Type

`Unit` is not merely a conventional "no return value" marker.

It is the canonical type-theoretic product of zero component types.

Conceptually:

```text
Product(A, B, C)
    has one coordinate of A,
    one coordinate of B,
    one coordinate of C

Product(A, B)
    has two coordinates

Product(A)
    has one coordinate

Product()
    has zero coordinates
    and exactly one inhabitant
```

Phalcom names the zero-coordinate product:

```text
Unit
```

and writes its unique value as:

```phalcom
()
```

---

## 4. Unit as Successful Completion

Phalcom also uses `Unit` as the result type of successful operations that produce no payload.

Examples:

```phalcom
list.clear
// Unit
```

```phalcom
list.append(value)
// Unit
```

```phalcom
map.clear
// Unit
```

This is not a separate "void" concept.

The same type-theoretic `Unit` serves both roles:

```text
zero-component product
successful completion with no payload
```

The sole value is always:

```phalcom
()
```

`None` is not used as a generic success-without-payload sentinel.

---

## 5. Empty Tuple Normalization

A Tuple with zero positional and zero labeled coordinates normalizes to `Unit`.

Surface syntax:

```phalcom
()
```

is therefore simultaneously:

```text
the empty Tuple/product
the unique Unit value
```

There is no distinct `EmptyTuple` type.

Conceptually:

```text
normalize(
    TupleProduct {
        positional = [],
        labeled = [],
    }
)
    → Unit
```

The type system MUST NOT expose a distinct semantic type such as:

```text
Tuple<>
```

for the zero-coordinate case.

If such a form exists internally during parsing or intermediate type construction, it MUST canonicalize to `Unit`.

---

## 6. Empty Record Normalization

A closed Record with zero fields also normalizes to `Unit`.

Surface syntax:

```phalcom
#{}
```

denotes the same semantic type and value as:

```phalcom
()
```

Therefore:

```text
typeOf(())
    ≡ Unit

typeOf(#{})
    ≡ Unit
```

and:

```phalcom
() == #{}
```

holds because both expressions elaborate to the same canonical value.

There is no distinct `EmptyRecord` type for a closed anonymous structural Record.

Conceptually:

```text
normalize(
    RecordProduct {
        fields = {},
        openness = closed,
    }
)
    → Unit
```

---

## 7. Why Tuple and Record Converge at Zero Arity

Tuple and Record differ by the structure of their coordinates.

Tuple coordinates are:

```text
ordered positional coordinates
+
ordered labeled coordinates
```

Record coordinates are:

```text
unordered labeled coordinates
```

At positive arity these distinctions are observable.

At zero arity:

```text
Tuple coordinates:
    positional = []
    labeled = []

Record coordinates:
    fields = {}
```

There are:

- no positions;
- no labels;
- no field/value associations;
- no ordering decisions;
- no permutations;
- no coordinate assignments.

The empty ordered coordinate structure and empty unordered coordinate structure carry no distinguishing information.

There is exactly one assignment of values to zero coordinates: the empty assignment.

Phalcom therefore identifies both with the terminal/unit product.

---

## 8. Canonicalization, Not Conversion

The following model is rejected:

```text
EmptyTuple
    ↔ conversion ↔
Unit
    ↔ conversion ↔
EmptyRecord
```

Phalcom instead uses:

```text
empty Tuple construction ─┐
                          ├──→ Unit
empty Record construction ┘
```

No runtime conversion is inserted.

No source-level coercion occurs.

No distinct empty-product wrapper objects are created and subsequently unwrapped.

The semantic result is canonical from the point at which product arity and closedness are known.

---

## 9. Parsing Versus Semantic Elaboration

The parser MAY preserve the syntactic distinction between:

```phalcom
()
```

and:

```phalcom
#{}
```

for purposes such as:

- diagnostics;
- formatting;
- source mapping;
- IDE tooling;
- syntax-directed analysis.

For example, an AST MAY contain distinct syntax nodes:

```text
TupleLiteral {
    positional = []
    labeled = []
}
```

and:

```text
RecordLiteral {
    fields = []
}
```

However, semantic elaboration MUST normalize both to the same zero-product meaning.

A later semantic IR SHOULD represent both as a canonical unit constant, conceptually:

```text
ConstUnit
```

---

## 10. Type Constructor Normalization

Type construction MUST canonicalize zero products.

Conceptually:

```text
makeTupleType(positionals, labeled):
    if positionals.isEmpty
       and labeled.isEmpty:
        return UnitType

    return TupleType(positionals, labeled)
```

and:

```text
makeRecordType(fields, openness):
    if openness == closed
       and fields.isEmpty:
        return UnitType

    return RecordType(fields, openness)
```

The exact implementation is not normative, but every type-construction path MUST produce canonical results.

---

## 11. Type Interning and Semantic Canonicalization

If Phalcom interns or semantically canonicalizes type objects, zero-product normalization occurs before or as part of interning.

The following conceptual constructions:

```text
TupleType([], [])
RecordType({}, closed)
UnitType
```

MUST resolve to the same semantic type identity.

A conforming implementation MUST NOT retain three semantically distinct runtime type descriptors for these constructions.

Conceptually:

```text
Tuple([], []) ─────┐
                   ├──→ UnitType
Record({}, closed) ┘
```

This is compatible with Phalcom's broader semantic type canonicalization strategy.

---

## 12. Runtime Value Representation

The runtime SHOULD expose one canonical representation for the `Unit` value.

Conceptually:

```text
UNIT
```

Both:

```phalcom
()
```

and:

```phalcom
#{}
```

lower to this representation.

The implementation MAY represent `UNIT` as:

- an immediate tagged value;
- a distinguished singleton object;
- another zero-allocation representation.

The exact representation is implementation-defined.

The implementation SHOULD NOT allocate separate zero-sized Tuple or Record heap objects for closed zero-product values.

---

## 13. Suggested Runtime Value Universe

A runtime MAY conceptually distinguish:

```text
Value
├── Unit
├── Tuple(TupleObject)
├── Record(RecordObject)
├── List(...)
├── Map(...)
└── ...
```

under these construction laws:

```text
Tuple construction:
    zero total components
        → UNIT

    one or more components
        → TupleObject
```

```text
Record construction:
    zero closed fields
        → UNIT

    one or more fields
        → RecordObject
```

This representation is illustrative, not mandatory.

---

## 14. Runtime Construction Normalization

Normalization is not limited to literals known at compile time.

If a product is constructed dynamically and its final closed shape has zero coordinates, runtime construction MUST produce `UNIT`.

### 14.1 Dynamic Tuple construction

Conceptually:

```text
finishTuple(builder):
    if builder.positionalCount == 0
       and builder.labeledCount == 0:
        return UNIT

    return Tuple(...)
```

### 14.2 Dynamic Record construction

Conceptually:

```text
finishRecord(builder):
    if builder.isClosed
       and builder.fieldCount == 0:
        return UNIT

    return Record(...)
```

The exact implementation is not normative.

---

## 15. Variadic Capture

Variadic capture may produce a zero-coordinate Tuple at runtime.

Example:

```phalcom
fn proxy(***args) {
    target(***args)
}

proxy()
```

The capture contains:

```text
zero positional values
zero labeled values
```

and therefore normalizes to `Unit`.

Conceptually inside `proxy`:

```text
args = UNIT
```

This normalization MUST NOT lose forwarding semantics.

Expanding the zero product contributes nothing:

```phalcom
target(***args)
```

behaves as:

```phalcom
target()
```

---

## 16. Unit and Expansion

`Unit` is the identity value for complete argument-product composition.

Conceptually:

```text
***UNIT
    → zero positional contributions
      +
      zero labeled contributions
```

Thus:

```phalcom
foo(***())
```

is equivalent to:

```phalcom
foo()
```

Because `#{}` also normalizes to Unit:

```phalcom
foo(***#{})
```

denotes the same zero-contribution product where the grammar permits the expression.

Similarly, projecting zero lanes yields zero contributions.

The full expansion semantics are specified in the argument-pack and expansion specification.

---

## 17. Unit and Product Capabilities

Zero-product normalization does not imply nominal inheritance relationships.

Phalcom MUST NOT infer merely from this specification that:

```text
Unit <: Tuple
Unit <: Record
```

or:

```text
Tuple <: Unit
Record <: Unit
```

The relationship is a normalization law over structural product constructors.

A future product capability MAY define operations that naturally include the zero-product case.

Examples that may be coherent through such a capability include:

```phalcom
().size
// 0
```

or zero-product reflection.

The exact capability hierarchy is deferred.

---

## 18. Product Operations at Zero Arity

Where an operation is defined over products generally, the zero-product case SHOULD follow the corresponding mathematical identity.

Examples:

```text
size(Unit)
    = 0
```

```text
positional component count
    = 0
```

```text
labeled component count
    = 0
```

```text
field count
    = 0
```

```text
iteration over components
    = empty iteration
```

Whether these appear as direct methods on `Unit` or through a shared structural/product capability is deferred.

---

## 19. Indexing the Zero Product

If generic product indexing is applied to `Unit`, no valid index exists.

Conceptually:

```phalcom
()[0]
```

must fail under the normal strict indexing rule because the product size is zero.

For integer indexing this corresponds to `IndexError`.

A label lookup likewise cannot succeed because the zero product has no labeled coordinate.

The precise exposed method set on `Unit` is deferred, but any generic product operation MUST preserve these semantics.

---

## 20. Equality and Hashing

There is exactly one `Unit` value.

Therefore:

```phalcom
() == ()
```

is true.

Because:

```phalcom
#{}
```

normalizes to the same value:

```phalcom
() == #{}
```

is also true.

`Unit` is hashable.

All zero-product constructions MUST have the same canonical hash behavior.

The exact numeric hash is implementation-defined.

---

## 21. Nominal Empty Types Do Not Collapse

Zero-product normalization applies to anonymous structural product constructions.

It does not collapse declared nominal types merely because they contain no user fields.

For example, a nominal declaration conceptually like:

```phalcom
class Marker {
}
```

retains the identity of `Marker`.

It does not normalize to `Unit`.

The distinction is:

```text
anonymous closed empty structural product
    → Unit

declared nominal zero-field type
    → retains nominal identity
```

This allows marker/sentinel nominal types to remain meaningful.

---

## 22. Open Rows Do Not Normalize to Unit

Only a closed exact empty Record normalizes to `Unit`.

A future open Record type or row variable may express:

```text
zero fields currently required
+
unknown additional fields
```

This is not a zero-coordinate product.

Conceptually:

```text
Record<{}, closed>
    → Unit
```

but:

```text
Record<{ ...ρ }>
    → not Unit unless ρ is proven empty and the row is closed
```

A type with an unknown/open row MUST NOT normalize to Unit merely because no fields are explicitly listed.

---

## 23. Width Constraints Are Not Empty Products

If Phalcom later supports width subtyping or Record constraints interpreted as:

```text
a Record containing at least these fields
```

then a constraint requiring zero fields would match many Records.

Such a constraint is not the singleton zero-product type.

Therefore the type system MUST distinguish:

```text
exact closed empty Record
```

from:

```text
open/constraint Record with zero required fields
```

Only the former normalizes to `Unit`.

---

## 24. Dynamic Record Shapes

A Record may eventually be constructed from runtime-determined Symbol fields.

For example:

```phalcom
#{
    **mapping,
}
```

may produce zero fields if `mapping` is empty.

If the completed Record is closed and contains zero fields, the runtime value MUST normalize to `UNIT`.

This remains true even if the compiler could not predict the final field count.

The static type of dynamically shaped Records is deferred to the Record/generic typing specification.

---

## 25. Static Type Precision for Dynamic Product Construction

A runtime operation may construct a product whose exact shape is unknown statically.

The type system MUST NOT assume nonzero arity merely because a syntactic operation is called "Record construction" or "Tuple capture."

When runtime construction can produce zero coordinates, `Unit` is a possible normalized runtime result.

Future typing may model this through:

- row variables;
- existential structural types;
- generalized product capabilities;
- another type-theoretic mechanism.

This specification does not choose among them.

---

## 26. No Observable "Pre-Normalized" Empty Product State

User code MUST NOT be able to observe a transient distinct empty Tuple or empty Record object before normalization.

The following conceptual states are not part of the language value model:

```text
EmptyTupleObject
EmptyRecordObject
```

followed by later conversion.

Normalization occurs as part of semantic construction/finalization.

---

## 27. Compile-Time Constant Folding

Implementations SHOULD constant-fold statically known empty product constructions to `UNIT`.

Examples:

```phalcom
()
```

and:

```phalcom
#{}
```

SHOULD require no runtime allocation.

If an expression is statically known to construct an empty closed product after expansion, an implementation MAY likewise lower it directly to `UNIT`.

Example, if statically provable:

```phalcom
#{
    **knownEmptyRecord,
}
```

may lower directly to Unit.

This optimization must preserve diagnostics and evaluation semantics of any source expressions.

---

## 28. Evaluation Semantics Still Apply

Normalization does not permit the implementation to skip required expression evaluation merely because the final product is empty.

For dynamically evaluated construction:

```phalcom
#{
    **expr,
}
```

if `expr` must be evaluated to determine that it contributes zero fields, that evaluation still occurs.

If `expr` raises/fails, the failure propagates.

Only once the product has successfully completed with zero closed coordinates does it normalize to `UNIT`.

---

## 29. Interaction With Type Reflection

Reflection should expose the canonical semantic type.

For either:

```phalcom
()
```

or:

```phalcom
#{}
```

type reflection MUST report `Unit`.

There is no reflected runtime type identity corresponding to "empty Tuple" or "empty Record."

Source-level reflection that explicitly asks for syntax form, AST form, or parsed literal origin is a separate tooling concern and MAY preserve the syntactic distinction.

---

## 30. Interaction With Generic Specialization

If generic/type constructors can conceptually produce zero-product structural types, specialization canonicalization MUST still return `Unit`.

A generic specialization cache MUST NOT retain semantically distinct objects for:

```text
Tuple zero-shape
Record closed zero-row
Unit
```

This rule is consistent with semantic canonicalization of equivalent type constructions elsewhere in Phalcom.

---

## 31. Unit Versus Empty Collections

Zero-product normalization MUST NOT be generalized into a rule that "all empty values collapse."

The reason `()` and `#{}` normalize together is structural product theory, not mere emptiness.

These remain distinct:

```phalcom
[]
{}
Set()
ImmutableSet()
```

Their collection families carry semantics even when empty.

For example:

```text
[]              → mutable List
{}              → mutable Map
Set()           → mutable Set
ImmutableSet()  → immutable Set
```

None of these normalize to Unit.

---

## 32. Mutable Set Empty Values

`Set` is mutable.

Each semantic `Set()` construction produces an independent mutable Set value.

Example:

```phalcom
const a = Set()
const b = Set()

a.add(1)

b.empty?
// true
```

The runtime MAY share immutable empty backing storage internally until mutation, but mutation of one Set MUST NOT affect another.

Therefore mutable empty Sets are not globally canonicalized as one observable mutable object.

---

## 33. ImmutableSet Empty Canonicalization

`ImmutableSet` is immutable.

All empty `ImmutableSet` values of a compatible runtime specialization MAY therefore share one canonical empty instance.

Conceptually:

```phalcom
ImmutableSet()
ImmutableSet.empty
```

may load the same canonical immutable value.

This is an immutable-value canonicalization optimization.

It is conceptually different from zero-product normalization.

The distinction is:

```text
Unit normalization
    = definitional type/product identity

ImmutableSet empty interning
    = safe immutable runtime canonicalization
```

---

## 34. Mutable Set Backing-Storage Sharing

An implementation MAY optimize repeated empty mutable Set construction using shared immutable backing storage.

Conceptually:

```text
Set wrapper A ─┐
               ├──→ shared immutable empty storage
Set wrapper B ─┘
```

On mutation:

```text
A.add(value)
    → detach/copy/allocate mutable storage for A
```

while B remains empty.

This optimization is not observable and is not required.

---

## 35. Unit Runtime Singleton

Unlike mutable `Set`, the `Unit` value can be globally canonical because it is immutable and has exactly one inhabitant by definition.

The runtime SHOULD therefore expose a single canonical Unit representation.

Object-identity behavior, if Phalcom exposes identity comparison for immediate/singleton values, MUST remain consistent with the fact that every Unit expression denotes the same unique value.

---

## 36. Normalization Pipeline

A recommended compiler/runtime pipeline is:

```text
parse syntax
    ↓
retain literal/source distinction if useful
    ↓
semantic product construction
    ↓
determine product family, shape, and closedness
    ↓
normalize zero closed product to UnitType / UNIT
    ↓
lower canonical semantic IR
    ↓
execute
```

For dynamic product construction:

```text
evaluate component/expansion sources
    ↓
build product shape/values
    ↓
finalize shape
    ↓
if closed arity == 0:
    return UNIT
else:
    allocate/finalize Tuple or Record
```

---

## 37. Required Compiler Invariants

A conforming implementation MUST maintain these invariants:

```text
no distinct semantic EmptyTuple type

no distinct semantic closed EmptyRecord type

no distinct runtime empty Tuple value

no distinct runtime closed empty Record value

all closed zero-product construction
    → Unit

all closed zero-product runtime finalization
    → UNIT

open/unknown Record rows
    → do not normalize without proof of closed emptiness
```

---

## 38. Diagnostic Implications

Because `#{}` elaborates to Unit, diagnostics SHOULD describe it as an empty Record syntax form that normalizes to `Unit` where source context matters.

For example, a type diagnostic MAY say conceptually:

```text
`#{}` is a closed empty Record and normalizes to `Unit`
```

rather than misleadingly claiming that the user wrote `()`.

The parser/source layer may preserve syntax origin for this purpose.

---

## 39. Optimization Freedom

Implementations MAY choose any representation consistent with this specification.

Permitted strategies include:

- immediate tagged Unit values;
- singleton Unit objects;
- compile-time constant folding;
- specialized zero-product IR;
- shared immutable backing storage for empty collections;
- copy-on-write for mutable empty collection backing stores.

Implementations MUST NOT expose differences that contradict semantic normalization.

---

## 40. Deferred Issues

The following are intentionally deferred:

1. the full product-capability hierarchy;
2. whether `Unit` directly exposes `size`, iteration, or reflection methods;
3. complete Tuple/Record generic type syntax;
4. open-row and row-polymorphism surface syntax;
5. dynamic Record-shape typing;
6. width subtyping for Records;
7. nominal struct/class interaction with product typing;
8. runtime identity operator semantics for immediate singleton values;
9. specialization-level interning rules for `ImmutableSet`;
10. full `ImmutableSet` API and hashing semantics;
11. complete reflection APIs for product shapes;
12. compiler IR representation details.

Future specifications MAY refine these areas but MUST preserve the zero-product normalization laws unless explicitly superseded.

---

## 41. Conformance Summary

A conforming Phalcom implementation MUST satisfy:

```text
Unit
    = canonical zero-arity product type

()
    = unique Unit value
    = empty Tuple/product

#{}
    = closed empty Record syntax
    = definitionally normalized to Unit

empty Tuple semantic type
    = Unit

closed empty Record semantic type
    = Unit

zero-product normalization
    = canonicalization, not conversion

runtime zero-product finalization
    = canonical UNIT value

open/unknown Record row
    ≠ Unit unless proven closed and empty

nominal zero-field declared type
    ≠ Unit merely because it has no fields

[]
{}
Set()
ImmutableSet()
    do not normalize to Unit

Set
    = mutable
    repeated Set() values are semantically independent

ImmutableSet
    = immutable
    empty values may share a canonical singleton instance
```
