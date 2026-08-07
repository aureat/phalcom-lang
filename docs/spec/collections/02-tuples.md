# Tuple Specification

## 1. Definition

A Tuple is a finite structural value with:

1. zero or more positional entries;
2. followed canonically by zero or more labeled entries.

```text
Tuple = ⟨PositionalLane, StructuralLabeledLane⟩
```

The structural labeled lane may use `LabelKey`, which is broader than the call-label domain.

## 2. Literal grammar

Conceptual grammar:

```text
TupleLiteral ::= "(" TupleEntries? ")"
TupleEntries ::= PositionalEntries ("," LabeledEntries)?
               | LabeledEntries
PositionalEntries ::= Expression ("," Expression)* ","?
LabeledEntries ::= LabeledEntry ("," LabeledEntry)* ","?
LabeledEntry ::= LabelSyntax ":" Expression
               | "[" Expression "]" ":" Expression
```

Once the first labeled entry appears, no ordinary positional entry may follow.

Valid:

```phalcom
()
(1,)
(1, 2)
(1, name: "Phalcom")
(name: "Phalcom", version: 1)
```

Invalid:

```phalcom
(name: "Phalcom", 1)
```

## 3. Unit and singleton tuples

**RATIFIED:** `()` is the empty Tuple and the `Unit` value.

```phalcom
().class == Tuple
().size == 0
```

A trailing comma distinguishes a one-element Tuple where necessary:

```phalcom
(Int,) -> Result
```

means a callable receiving one positional `Int`.

```phalcom
((Int, String),) -> Result
```

means a callable receiving one positional Tuple.

## 4. Symbolic labels

**RATIFIED:** Tokens legal as symbolic selector heads may be used as labels when followed by `:`.

```phalcom
const metadata = (
  *: positionals,
  **: labels,
  ?: optional,
  !: required,
  +: arithmetic,
  ==: equality
)
```

The colon disambiguates the label from an operator or spread form.

Canonical access uses key lookup:

```phalcom
metadata[#*]
metadata[#**]
metadata[#?]
metadata[#+]
metadata[#==]
```

## 5. Selector-valued labels

**RATIFIED:** Selector-shaped symbolic labels produce first-class `Selector` keys.

```phalcom
const operations = (
  +(_): add,
  -(): negate,
  ==(_): equal
)
```

This is shorthand for:

```phalcom
const operations = (
  [#+(_)]: add,
  [#-()]: negate,
  [#==(_)]: equal
)
```

The following keys are distinct:

```text
#+      Symbol
#+()    Selector
#+(_)   Selector
```

Therefore:

```phalcom
operations[#+(_)]
```

is not equivalent to:

```phalcom
operations[#+]
```

## 6. Duplicate labels

Tuple labels MUST be unique under key equality.

```phalcom
(
  +(_): first,
  [#+(_)]: second
)
// error: duplicate label #+(_)
```

Likewise:

```phalcom
(
  *: first,
  [#*]: second
)
// error: duplicate label #*
```

## 7. Tuple Type interpretation

**RATIFIED:** In a type-consuming context, a tuple expression may denote a structural Tuple Type.

```phalcom
type Pair = (Int, String)
type Named = (Int, name: String)
```

In ordinary value context, the same source syntax remains a Tuple of Type values:

```phalcom
const pairDescription = (Int, String)
pairDescription.class == Tuple
```

A dynamic conversion MAY be exposed:

```phalcom
const PairType = pairDescription.asType
```

The exact conversion protocol remains provisional.

## 8. Repeated positional tail

**RATIFIED:** A final `...` repeats the immediately preceding positional type zero or more times.

```phalcom
type Ints = (Int, ...)
```

Accepts:

```phalcom
()
(1,)
(1, 2, 3)
```

A fixed prefix may precede the repeated type:

```phalcom
type Requests = (Context, Request, ...)
```

This means one required `Context`, followed by zero or more `Request` values.

It does not repeat the entire preceding sequence.

```phalcom
type Values = (Int, String, ...)
```

means one required `Int`, followed by zero or more `String` values.

## 9. Labeled slots after repetition

**RATIFIED:** Fixed labels may follow a repeated positional tail.

```phalcom
type Command = (
  String,
  Int,
  ...,
  timeout: Duration
)
```

Meaning:

- one required positional `String`;
- zero or more positional `Int` values;
- one required labeled `timeout: Duration`.

## 10. Exactness

An ordinary Tuple Type is exact unless it contains an explicit repeated tail or is contextually interpreted as an open argument-pack schema.

```phalcom
type Exact = (Int, name: String)
```

The value must contain exactly one positional entry and exactly one `#name` label.

Additional positionals or labels do not satisfy `Exact`.

## 11. Tuple access

Canonical access forms:

```phalcom
value[index]
value[#label]
value[#+(_)]
```

Identifier labels MAY support member access:

```phalcom
value.name
```

Key lookup remains canonical because it works for all Symbol and Selector keys.

## 12. Tuple spreading

**RATIFIED:** Value spreading is not legal inside Tuple construction.

```phalcom
(prefix, *values) // invalid
```

Composition must use explicit operations whose collision and normalization rules are independently specified:

```phalcom
left.concatPositionals(right)
left.withLabels(right)
```

The exact standard-library API is **OPEN**.

## 13. Tuple and argument-pack distinction

A Tuple value may carry both lanes and may be used as the runtime representation of a pack. However:

```text
TupleType ≠ ArgumentPackType
```

For example:

```phalcom
type LiteralKeys = (*: Int, **: String)
```

is an exact Tuple Type with literal keys `#*` and `#**`.

But:

```phalcom
const callback: (*: Int, **: String) -> Result
```

uses the tuple expression as an argument-pack schema and creates an `ArgumentPackType` with open lanes.
