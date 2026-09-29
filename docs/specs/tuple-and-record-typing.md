# Tuple and Record Structural Type Semantics

## 1. Scope

This specification defines the structural type semantics of Phalcom tuples and records, including:

- closed and open structural types;
- positional and labeled tuple lanes;
- required and optional labeled members;
- homogeneous and unconstrained remainders;
- the structural meaning of `...`;
- the lane operators `*`, `**`, and `***`;
- tuple and record width subtyping.

This specification defines type semantics only. Runtime rest parameters, generic pack binding, pack expansion, and the runtime `Ellipsis` value are specified separately, except where their `*`, `**`, `***`, or `...` syntax shares the structural conventions defined here.

---

## 2. Structural Closedness

Tuple and record types are **closed by default**.

A closed type admits no structural members other than those explicitly described by the type, except for members explicitly declared optional.

```phalcom
(Int, String)
```

describes a tuple with exactly two positional members and no labeled members.

```phalcom
(id: Int, name: String)
```

describes a tuple with exactly the labeled members `id` and `name` and no positional members.

```phalcom
{
    id: Int,
    name: String,
}
```

describes a record with exactly the fields `id` and `name`.

The empty forms are therefore exact empty structures:

```phalcom
()
{}
```

Neither `()` nor `{}` denotes an arbitrary tuple or record.

A structure becomes open only through an explicit remainder.

---

## 3. Tuple Lanes

A tuple type consists of two independent structural lanes:

1. the **positional lane**, containing ordered positional members;
2. the **labeled lane**, containing uniquely labeled members.

The positional lane precedes the labeled lane in source syntax.

```phalcom
(
    Int,
    String,
    name: String,
    active: Bool,
)
```

has two positional members and two labeled members.

The two lanes are independently closed or open. Opening one lane does not open the other.

---

## 4. Lane Operators

The asterisk operators identify tuple lanes:

| Operator | Lane |
|---|---|
| `*` | positional lane |
| `**` | labeled lane |
| `***` | complete tuple structure: positional and labeled lanes |

When used in an anonymous structural remainder of the form `*: C`, `**: C`, or `***: C`, `C` is the constraint applied to entries admitted by that remainder.

The complete-lane form is shorthand for opening both lanes under the same constraint:

```phalcom
(***: T)
```

is equivalent to:

```phalcom
(*: T, **: T)
```

In particular:

```phalcom
(***: ...)
```

is equivalent to:

```phalcom
(*: ..., **: ...)
```

and both normalize to:

```phalcom
(...)
```

Records have only a labeled field lane. Consequently, record remainders use `**`; `*` and `***` are not record-field remainder operators.

---

## 5. Remainders

A remainder admits zero or more additional members in its lane.

A lane with no remainder is closed.

### 5.1 Homogeneous remainder

A typed remainder constrains every additional member in that lane to the specified type.

```phalcom
(*: T)
```

admits zero or more positional members, each of a type that is a subtype of `T`. The labeled lane is closed.

```phalcom
(**: T)
```

admits zero or more labeled members, each of a type that is a subtype of `T`. The positional lane is closed.

```phalcom
(*: T, **: U)
```

admits arbitrary positional members constrained by `T` and arbitrary labeled members constrained by `U`.

Fixed members may precede their lane's remainder:

```phalcom
(
    Int,
    *: Number,
    name: String,
    **: JsonValue,
)
```

The positional remainder applies only after the fixed positional prefix. The labeled remainder applies only to labels not explicitly declared by the tuple type.

A tuple type may contain at most one positional remainder and at most one labeled remainder.

The positional remainder is the final component of the positional lane. The labeled remainder is the final component of the labeled lane.

### 5.2 Unconstrained remainder

`...` denotes an **unconstrained structural remainder**. It does not denote `Any`, does not synthesize an element type, and imposes no type constraint on the admitted remainder.

The explicit lane forms are:

```phalcom
(*: ...)
(**: ...)
(***: ...)
```

`(*: ...)` is the structural top of positional-only tuples.

`(**: ...)` is the structural top of labeled-only tuples.

`(***: ...)`, `(*: ..., **: ...)`, and `(...)` are equivalent and denote the structural top of all tuple types.

Therefore:

```phalcom
(*: ...) <: (...)
(**: ...) <: (...)
```

and every tuple type is a subtype of:

```phalcom
(...)
```

The empty tuple is a subtype of all three unconstrained open forms because both of its lanes are empty.

---

## 6. Contextual Ellipsis in Tuple Types

Within tuple type syntax, bare `...` opens the current lane with an unconstrained remainder.

After positional members:

```phalcom
(Int, String, ...)
```

is equivalent to:

```phalcom
(Int, String, *: ...)
```

After labeled members:

```phalcom
(id: Int, name: String, ...)
```

is equivalent to:

```phalcom
(id: Int, name: String, **: ...)
```

Both lanes may be opened independently:

```phalcom
(
    Int,
    ...,
    name: String,
    ...
)
```

is equivalent to:

```phalcom
(
    Int,
    *: ...,
    name: String,
    **: ...,
)
```

The standalone tuple form:

```phalcom
(...)
```

is the complete open tuple type and is equivalent to:

```phalcom
(***: ...)
```

Bare `...` in tuple type syntax does **not** mean homogeneous repetition of the preceding type. Homogeneous repetition is expressed by a typed remainder such as `*: T` or `**: T`.

---

## 7. Required and Optional Labeled Members

A labeled member without `?` is required:

```phalcom
name: String
```

A labeled member followed by `?` is optional:

```phalcom
name?: String
```

An optional member may be absent. If present, its value must satisfy its declared type.

Optionality concerns **member presence** and is distinct from `Option<T>`.

```phalcom
{
    name?: String,
}
```

permits the field `name` to be absent.

```phalcom
{
    name: Option<String>,
}
```

requires the field `name` to be present; its value has type `Option<String>`.

The same optional-member semantics apply to labeled tuple members.

This specification does not define optional positional members.

Explicitly declared required and optional labels are excluded from the labeled remainder. A labeled remainder constrains only labels not explicitly declared by the enclosing type.

---

## 8. Record Types

A record is a structural type consisting solely of labeled fields.

A record is closed unless it contains a remainder.

```phalcom
{
    id: Int,
    name: String,
}
```

is closed.

An unconstrained open record is written:

```phalcom
{
    ...
}
```

and is the structural top of all record types.

Therefore every record type `R` satisfies:

```phalcom
R <: { ... }
```

A record may require specific fields while leaving the remaining row unconstrained:

```phalcom
{
    id: Int,
    ...
}
```

This is equivalent in remainder semantics to:

```phalcom
{
    id: Int,
    **: ...
}
```

A record may instead constrain every additional field to a common type:

```phalcom
{
    id: Int,
    **: String,
}
```

The remainder applies only to labels not explicitly declared by the record.

Records and tuples are distinct structural type constructors. Neither `{ ... }` nor any concrete record type is a subtype of `(...)`, and no tuple type is a subtype of `{ ... }`, solely by virtue of structural openness.

---

## 9. Positional Tuple Subtyping

Let a positional lane consist of:

- a fixed positional prefix `P`;
- optionally, a positional remainder `R`.

A source positional lane `S` is a subtype of a target positional lane `T` only if all of the following hold.

### 9.1 Required prefix

The source must contain at least as many fixed positional members as the target.

For every fixed target position `i`:

```text
Source[i] <: Target[i]
```

An open source remainder cannot satisfy a required target position because a remainder may contain zero members.

Consequently:

```phalcom
(Int, String) <: (Int, ...)
```

but:

```phalcom
(Int, ...) </: (Int, String, ...)
```

because the source does not guarantee the second positional member.

### 9.2 Additional fixed positions

Every fixed source position beyond the target's fixed prefix must be admitted by the target remainder.

If the target positional lane is closed, no additional source positions are permitted.

If the target has:

```phalcom
*: ...
```

additional source positions are unconstrained.

If the target has:

```phalcom
*: T
```

each additional fixed source position must have a type that is a subtype of `T`.

### 9.3 Source remainder

If the source positional lane is open, the target positional lane must also be open.

The source remainder must be no less restrictive than the target remainder:

```text
source typed remainder U <: target typed remainder T
    iff U <: T
```

A typed source remainder is a subtype of an unconstrained target remainder.

An unconstrained source remainder is not a subtype of a typed target remainder.

An unconstrained source remainder is a subtype of an unconstrained target remainder.

A closed source lane may be a subtype of either a closed or open target lane, subject to the fixed-position rules above.

---

## 10. Labeled Structural Subtyping

The labeled lane of a tuple and the field row of a record use the same member-compatibility rules.

Let `S` be the source labeled structure and `T` the target.

### 10.1 Required target members

Every required member declared by `T` must be declared as required by `S`.

For each required target label `k`:

```text
S[k] <: T[k]
```

An optional source member does not satisfy a required target member.

An open source remainder does not establish the presence of a required target label.

### 10.2 Optional target members

For an optional target member:

```phalcom
k?: T
```

the source may omit `k`.

If the source explicitly declares `k`, whether required or optional, its member type must be a subtype of `T`.

If the source does not explicitly declare `k`:

- a closed source satisfies the optional target member because `k` is guaranteed absent;
- a typed open source remainder satisfies it only if the remainder constraint is a subtype of `T`;
- an unconstrained open source remainder does not establish compatibility with `k?: T`, because the source may contain `k` with an incompatible type.

### 10.3 Additional explicit source members

Every explicit source label not declared by the target must be admitted by the target labeled remainder.

If the target labeled lane is closed, such an additional source member makes the subtype relation fail.

If the target remainder is unconstrained, the additional member is admitted.

If the target remainder is typed `**: T`, the additional source member's type must be a subtype of `T`.

This rule applies equally to required and optional additional source members.

### 10.4 Source labeled remainder

If the source has an open labeled remainder, the target must also have an open labeled remainder.

For typed remainders:

```text
source **: U <: target **: T
    iff U <: T
```

A typed source remainder is a subtype of an unconstrained target remainder.

An unconstrained source remainder is not a subtype of a typed target remainder.

An unconstrained source remainder is a subtype of an unconstrained target remainder.

---

## 11. Complete Tuple Subtyping

Tuple subtyping is lane-wise.

For tuple types `A` and `B`:

```text
A <: B
```

holds if and only if:

1. the positional lane of `A` is a subtype of the positional lane of `B`; and
2. the labeled lane of `A` is a subtype of the labeled lane of `B`.

Opening one target lane does not admit additional members in the other lane.

For example:

```phalcom
(*: ...)
```

does not admit labeled members.

```phalcom
(**: ...)
```

does not admit positional members.

Only a complete open target such as:

```phalcom
(...)
```

admits arbitrary structure in both lanes.

---

## 12. Width Subtyping

Phalcom does not apply implicit width subtyping to closed tuple or record types.

A larger closed shape is not a subtype of a smaller closed shape merely because the smaller shape's members are present.

Therefore:

```phalcom
(Int, String) </: (Int)
```

and:

```phalcom
{
    id: Int,
    name: String,
}
</:
{
    id: Int,
}
```

Width subtyping is enabled by an explicit open remainder in the target:

```phalcom
(Int, String) <: (Int, ...)
```

and:

```phalcom
{
    id: Int,
    name: String,
}
<:
{
    id: Int,
    ...
}
```

Typed remainders impose the corresponding additional-member constraint:

```phalcom
(Int, String, String)
<:
(Int, *: String)
```

whereas:

```phalcom
(Int, Bool)
</:
(Int, *: String)
```

Optional members do not open a type. They admit only the explicitly named optional labels.

---

## 13. Structural Top Types

The following are the structural top types defined by this specification:

```phalcom
(...)       // any tuple
{ ... }     // any record
```

The tuple top normalizes as:

```phalcom
(...) == (***: ...) == (*: ..., **: ...)
```

The positional-only and labeled-only tuple tops are:

```phalcom
(*: ...)
(**: ...)
```

respectively.

These structural tops are not universal object types. They range only over their corresponding structural type constructors.

Accordingly, the canonical generic bounds for arbitrary tuple and record types are:

```phalcom
T <: (...)
R <: { ... }
```

---

## 14. Normalization Rules

The following forms are semantically equivalent and MUST normalize to the same structural type.

```phalcom
(...)
(***: ...)
(*: ..., **: ...)
```

```phalcom
(Int, ...)
(Int, *: ...)
```

```phalcom
(id: Int, ...)
(id: Int, **: ...)
```

```phalcom
{
    id: Int,
    ...
}
{
    id: Int,
    **: ...
}
```

For any type `T`:

```phalcom
(***: T)
(*: T, **: T)
```

are equivalent.

Normalization MUST preserve lane closedness, required members, optional members, fixed positional order, and remainder constraints.

---

## 15. Well-Formedness Constraints

A tuple or record structural type is well-formed only if all of the following conditions hold:

- labels within a labeled lane are unique;
- a label cannot be declared both required and optional;
- there is at most one positional remainder in a tuple;
- there is at most one labeled remainder in a tuple or record;
- there is at most one complete `***` remainder in a tuple;
- a complete `***` remainder cannot coexist with a separate `*` or `**` remainder;
- the positional remainder, when written explicitly as `*: ...` or `*: T`, terminates the positional lane;
- the labeled remainder terminates the labeled lane;
- optionality applies only to explicitly labeled members;
- a remainder may contain zero members.

A complete `***` remainder is equivalent to declaring both lane remainders and is subject to the same constraints after normalization.