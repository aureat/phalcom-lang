# Phalcom Argument-Pack Type Interpretation

## 1. Principle

Tuple syntax has no intrinsic argument-pack semantics.

```
(*: Int, **: String)
```

is fundamentally an ordinary tuple expression with two labeled entries:

```
(
  [#*]: Int,
  [#**]: String
)
```

The keys `#*` and `#**` are ordinary `Symbol` values.

Their structural meanings arise only when the tuple is consumed by an argument-pack interpretation context.

Therefore:

> Tuple construction supplies structure. The consuming context supplies argument-pack meaning.

---

## 2. Ordinary interpretation

In an ordinary value context:

```
const description = (
  *: Int,
  **: String
)
```

the result is an ordinary Tuple:

```
description.class == Tuple
description[#*] == Int
description[#**] == String
```

In an ordinary tuple-type context:

```
type Description = (
  *: Int,
  **: String
)
```

the result is an exact labeled tuple Type requiring two labels:

```
#*  : Int
#** : String
```

Neither entry represents an open argument lane in this context.

---

## 3. Pack schemas

When a tuple Type is passed to an argument-pack context, it is interpreted as a `PackSchema`.

Conceptually:

```
PackSchema {
    fixedPositionals: List<Type>
    openPositional: Option<Type>
    fixedLabels: Map<Symbol | Selector, Type>
    openLabeled: Option<Type>
}
```

The reserved keys are interpreted as follows:

```
#*  → open positional lane
#** → open labeled lane
```

For example:

```
(
  Request,
  *: Bytes,
  timeout: Duration,
  **: Metadata
)
```

is interpreted as:

```
fixedPositionals = [Request]
openPositional   = Bytes
fixedLabels      = { #timeout: Duration }
openLabeled      = Metadata
```

The original tuple remains an ordinary reflective object. `PackSchema` is the contextual interpretation of that tuple.

---

## 4. Pack-consuming contexts

Phalcom defines three argument-pack contexts.

### 4.1 Callable-domain context

A callable domain owns both argument lanes:

```
owned lanes = { positional, labeled }
```

It may therefore interpret both `#*` and `#**`.

```
(
  Request,
  *: Bytes,
  timeout: Duration,
  **: Metadata
) -> Response
```

This callable accepts:

1. one required positional `Request`;
    
2. zero or more positional `Bytes`;
    
3. one required `timeout: Duration`;
    
4. zero or more additional labeled arguments whose values satisfy `Metadata`.
    

The following domain is valid:

```
(*: Int, **: String) -> Result
```

because a callable domain owns both lanes.

---

### 4.2 Positional-rest context

A positional-rest parameter owns only the positional lane:

```
owned lanes = { positional }
```

Example:

```
method(*arguments: (*: Int)) {
  ...
}
```

The annotation is interpreted as an open positional lane of `Int`.

The declaration accepts:

```
method()
method(1)
method(1, 2, 3)
```

Inside the method:

```
arguments.class == Tuple
arguments: (Int, ...)
```

A call such as:

```
method(10, 20, 30)
```

binds:

```
arguments == (10, 20, 30)
```

The method declaration therefore elaborates to:

```
external domain:
    (*: Int)

local binding:
    arguments: (Int, ...)
```

The captured value is an ordinary positional-only Tuple.

---

### 4.3 Labeled-rest context

A labeled-rest parameter owns only the labeled lane:

```
owned lanes = { labeled }
```

Example:

```
method(**labels: (**: String)) {
  ...
}
```

The declaration accepts:

```
method()
method(name: "Phalcom")
method(name: "Phalcom", mode: "strict")
```

Inside the method, `labels` is an ordinary labeled-only Tuple view:

```
labels[#name]
labels[#mode]
```

Its structural type is conceptually:

```
(**: String)
```

The method declaration elaborates to:

```
external domain:
    (**: String)

local binding:
    labels: labeled-only Tuple<String>
```

---

## 5. Lane-ownership rule

Let `lanes(S)` be the set of lanes described by pack schema `S`.

A rest declaration is well formed only when:

```
lanes(annotationSchema) ⊆ lanesOwnedByBinder
```

Therefore:

```
*arguments: (*: Int)
```

is valid:

```
annotation lanes = { positional }
binder lanes     = { positional }
```

And:

```
**labels: (**: String)
```

is valid:

```
annotation lanes = { labeled }
binder lanes     = { labeled }
```

But:

```
**labels: (*: SomeType)
```

is invalid:

```
annotation lanes = { positional }
binder lanes     = { labeled }
```

Likewise:

```
*arguments: (**: SomeType)
```

is invalid.

---

## 6. Mixed schemas and one-lane binders

A mixed pack schema describes both lanes:

```
(*: Int, **: String)
```

It is valid as a callable domain:

```
const callback:
  (*: Int, **: String) -> Result
```

It is not valid as the annotation of either individual rest binder:

```
method(*arguments: (*: Int, **: String))
// error
```

```
method(**labels: (*: Int, **: String))
// error
```

The first binder cannot capture the labeled lane. The second binder cannot capture the positional lane.

The lanes must instead be declared separately:

```
method(
  *arguments: (*: Int),
  **labels: (**: String)
) {
  ...
}
```

This declaration has the callable domain:

```
(*: Int, **: String)
```

Its local bindings are:

```
arguments = positional-only Tuple<Int>
labels    = labeled-only Tuple<String>
```

Thus the complete callable domain may be represented by one mixed tuple schema even though the method body receives two lane-specific bindings.

---

## 7. Non-tuple shorthand

A non-tuple annotation on a rest parameter describes each captured value.

Therefore:

```
method(*arguments: Int)
```

is shorthand for:

```
method(*arguments: (*: Int))
```

Both bind:

```
arguments: (Int, ...)
```

Similarly:

```
method(**labels: String)
```

is shorthand for:

```
method(**labels: (**: String))
```

Both bind an arbitrary labeled-only Tuple whose values satisfy `String`.

The explicit forms expose the underlying pack theory. The short forms optimize ordinary authoring.

---

## 8. Exact positional pack annotations

A positional-rest annotation may describe an exact captured positional pack:

```
method(*arguments: (Int, String)) {
  ...
}
```

This accepts exactly two remaining positional arguments:

```
method(10, "value")
```

Inside:

```
arguments == (10, "value")
arguments: (Int, String)
```

It rejects:

```
method()
method(10)
method(10, "value", true)
```

A fixed positional prefix may be followed by an open positional lane:

```
method(
  *arguments: (
    Context,
    *: Request
  )
)
```

This requires one `Context`, followed by zero or more `Request` values.

Its local captured type normalizes to:

```
(Context, Request, ...)
```

---

## 9. Exact labeled pack annotations

A labeled-rest annotation may require exact labels:

```
method(
  **options: (
    host: String,
    port: Int
  )
) {
  ...
}
```

This requires exactly the labeled shape:

```
method(
  host: "localhost",
  port: 8080
)
```

Inside:

```
options: (
  host: String,
  port: Int
)
```

An exact prefix may be combined with an open labeled lane:

```
method(
  **options: (
    format: Format,
    **: Metadata
  )
)
```

This requires `format` and accepts arbitrary additional labels whose values satisfy `Metadata`.

A Record annotation may be used instead:

```
method(**config: ConnectionConfig) {
  config.host
  config.port
}
```

The captured result preserves the annotation’s structural kind:

- tuple annotation produces a labeled Tuple;
    
- record annotation produces a Record.
    

---

## 10. Invalid cross-lane structures

A positional-rest annotation cannot require fixed labels:

```
method(
  *arguments: (
    Int,
    mode: Symbol
  )
)
```

This fails because the annotation describes a labeled slot that the positional binder cannot capture.

A labeled-rest annotation cannot require positional slots:

```
method(
  **labels: (
    Int,
    name: String
  )
)
```

This fails because the annotation describes a positional slot that the labeled binder cannot capture.

The rule applies equally to open lanes:

```
method(**labels: (*: Int))
// error: positional lane specified for labeled-rest binder
```

```
method(*arguments: (**: String))
// error: labeled lane specified for positional-rest binder
```

---

## 11. Distinction from nested tuple elements

This declaration:

```
method(*arguments: (*: Int))
```

captures zero or more `Int` values.

It does not capture zero or more tuple values containing a `#*` label.

To require each captured value to be such a tuple, an outer repeated tuple type must be explicit:

```
method(
  *arguments: (
    (*: Int),
    ...
  )
)
```

Here the outer tuple describes the captured positional pack. Its repeated element type is the ordinary exact tuple Type:

```
(*: Int)
```

A valid call is:

```
method(
  (*: 1),
  (*: 2),
  (*: 3)
)
```

Inside:

```
arguments == (
  (*: 1),
  (*: 2),
  (*: 3)
)
```

The distinction is:

```
*arguments: (*: Int)
```

Open positional pack of `Int`.

```
*arguments: ((*: Int), ...)
```

Open positional pack whose elements are tuples containing the exact label `#*`.

---

## 12. Type-level unpacking is separate

A leading `*` without a colon is a type-level unpack expression:

```
(*P,) -> Result
```

If:

```
type P = (
  Int,
  debug: Bool
)
```

then:

```
(*P,) -> Result
```

normalizes to:

```
(Int, debug: Bool) -> Result
```

This must not be confused with:

```
(*: Int)
```

The colon changes the grammar:

```
*P       = unpack the tuple Type P
*: Int   = ordinary tuple entry keyed by #*
```

Nor should it be confused with:

```
*arguments: P
```

where the first `*` belongs to the parameter declaration.

---

## 13. Formal interpretation judgment

Let:

```
C ∈ {
    OrdinaryTuple,
    CallableDomain,
    PositionalRest,
    LabeledRest
}
```

The contextual interpretation judgment is:

```
C ⊢ T ⇝ S
```

meaning:

> In context `C`, tuple Type `T` is interpreted as structural schema `S`.

For ordinary tuples:

```
OrdinaryTuple ⊢ (*: A, **: B)
    ⇝ ExactTuple {
         #*: A,
         #**: B
       }
```

For callable domains:

```
CallableDomain ⊢ (*: A, **: B)
    ⇝ PackSchema {
         openPositional: A,
         openLabeled: B
       }
```

For positional rest:

```
PositionalRest ⊢ (*: A)
    ⇝ PackSchema {
         openPositional: A
       }
```

But:

```
PositionalRest ⊬ (*: A, **: B)
```

because the resulting schema contains a labeled lane.

For labeled rest:

```
LabeledRest ⊢ (**: B)
    ⇝ PackSchema {
         openLabeled: B
       }
```

But:

```
LabeledRest ⊬ (*: A)
```

because the resulting schema contains a positional lane.

---

## 14. Diagnostics

Recommended diagnostic for:

```
method(**labels: (*: SomeType))
```

```
Invalid labeled-rest annotation.

`**labels` captures only labeled arguments, but its annotation
specifies the open positional lane `#*`.

Use an open labeled-lane annotation instead:

    **labels: (**: SomeType)

or its shorthand:

    **labels: SomeType
```

Recommended diagnostic for:

```
method(*arguments: (*: Int, **: String))
```

```
Invalid positional-rest annotation.

`*arguments` captures only positional arguments, but its annotation
also specifies the open labeled lane `#**`.

Capture the lanes separately:

    *arguments: (*: Int),
    **labels: (**: String)
```

---

## 15. Core invariants

1. `#*` and `#**` are ordinary Symbol keys.
    
2. Tuple construction never assigns argument-pack semantics.
    
3. Argument-pack semantics belong to the consuming context.
    
4. A callable domain owns both lanes.
    
5. A `*parameter` binder owns only the positional lane.
    
6. A `**parameter` binder owns only the labeled lane.
    
7. A binder rejects annotations describing lanes it does not own.
    
8. Non-tuple rest annotations are shorthand for homogeneous open lanes.
    
9. Rest captures produce ordinary Tuple values or annotation-prescribed Records.
    
10. Type-level unpacking `*P` is distinct from the symbolic tuple label `*:`.
    

This creates a uniform model in which Phalcom’s surface syntax is ordinary object syntax, while argument domains arise through disciplined contextual interpretation.