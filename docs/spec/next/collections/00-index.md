# Phalcom Collections and Argument-Pack Specification Suite

**Status:** Draft normative candidate for systematic language review  
**Scope:** Tuples, records, sets, argument packs, rest/spread operators, callable domains, reflection, normalization, diagnostics, and conformance  
**Audience:** Language architects, parser/compiler implementers, VM/runtime implementers, standard-library authors, tooling authors, and specification reviewers

## 1. Purpose

This suite converts the tuple/record/argument-pack design discussion into a reviewable specification. It is intentionally split into small documents so another agent can evaluate each semantic layer independently and then check cross-document consistency.

Every rule is tagged with one of four statuses:

- **RATIFIED** — explicitly accepted during the design discussion.
- **AMENDED** — explicitly replaces an earlier accepted rule.
- **PROVISIONAL** — recommended here to close a gap, but not yet explicitly ratified.
- **OPEN** — deliberately unresolved and listed for review.

Normative keywords **MUST**, **MUST NOT**, **SHOULD**, and **MAY** use their usual specification meanings.

## 2. Reading order

1. [`01-foundations-and-notation.md`](01-foundations-and-notation.md)
2. [`02-tuples.md`](02-tuples.md)
3. [`03-records.md`](03-records.md)
4. [`04-sets.md`](04-sets.md)
5. [`05-argument-packs.md`](05-argument-packs.md)
6. [`06-rest-spread-and-pack-operators.md`](06-rest-spread-and-pack-operators.md)
7. [`07-callable-domains.md`](07-callable-domains.md)
8. [`08-reflection-and-type-values.md`](08-reflection-and-type-values.md)
9. [`09-normalization-subtyping-and-satisfaction.md`](09-normalization-subtyping-and-satisfaction.md)
10. [`10-diagnostics.md`](10-diagnostics.md)
11. [`11-conformance-tests.md`](11-conformance-tests.md)
12. [`12-open-questions-and-gap-analysis.md`](12-open-questions-and-gap-analysis.md)

## 3. Core model

A Phalcom call is represented abstractly as a two-lane pack:

```text
CallPack = ⟨PositionalLane, LabeledLane⟩
```

Where:

```text
PositionalLane = finite ordered sequence of values
LabeledLane    = finite insertion-ordered mapping from call labels to values
```

A Tuple is the principal value representation capable of carrying both lanes. A Record carries a labeled structural shape and embeds into a call pack with an empty positional lane. A Set is not a pack and has no lane semantics.

The operator family is **AMENDED** to be lane-uniform:

```text
*     positional lane
**    labeled lane
***   complete pack, preserving both lanes
```

The same lane meaning applies in three distinct operations:

```text
capture   incoming CallPack → local binding
expansion value             → outgoing CallPack contribution
type unpack Type            → callable-domain contribution
```

The grammar and direction differ, but the selected lane does not.

## 4. Principal decisions

### 4.1 First-class types

**RATIFIED:** Complete type expressions evaluate to immutable, reflective, first-class `Type` values. Implementations MAY intern, constant-fold, or represent them through compact handles.

### 4.2 Contextual tuple typing

**RATIFIED:** A tuple expression is an ordinary Tuple in value context and may be contextually interpreted as a structural Tuple Type in a type-consuming context.

```phalcom
const values = (Int, String)
values.class == Tuple

type PairType = (Int, String)
```

### 4.3 Ellipsis and repeated tails

**RATIFIED:** `...` is a first-class immutable `Ellipsis` singleton. In a tuple Type, a final `...` repeats the immediately preceding positional type zero or more times.

```phalcom
type Ints = (Int, ...)
type Requests = (Context, Request, ...)
```

### 4.4 Symbolic and selector-valued structural labels

**RATIFIED:** Tuple and Record syntax may use symbolic labels and selector-shaped labels.

```phalcom
const operations = (
  +: family,
  +(): unary,
  +(_): binary
)

operations[#+]
operations[#+()]
operations[#+(_)]
```

The exact call-label domain remains separate and is tracked as an open question.

### 4.5 Argument-pack type

**PROVISIONAL, REQUIRED FOR CONSISTENCY:** Pack interpretation MUST produce a reflective `ArgumentPackType`, distinct from an ordinary exact `TupleType`.

```phalcom
type LiteralKeys = (*: Int, **: String)
// Exact TupleType with literal keys #* and #**.

const callback: (*: Int, **: String) -> Result
// CallableType whose domain is an ArgumentPackType with open lanes.
```

The source expression may be identical, but the consuming type constructor and resulting Type object differ.

### 4.6 Duplicate labels

**RATIFIED:** Duplicate labels during call assembly are errors. Source order never provides implicit overriding.

```phalcom
target(x: 1, x: 2)              // error
target(**first, **second)        // error if both contain #x
target(***pack, x: 2)            // error if pack contains #x
```

### 4.7 Spread scope

**RATIFIED:** Value spread syntax is restricted to call argument lists. Tuple, Record, and Set construction use explicit APIs for composition.

```phalcom
target(*positionals, **labels)

(prefix, *values)       // invalid
{ base: value, **more } // invalid
```

## 5. Cross-document invariants

All documents MUST preserve these invariants:

1. A Tuple value has one positional lane followed canonically by one labeled lane.
2. A Record has only labeled structural entries.
3. A Set has no lanes and cannot be projected or expanded as a pack.
4. `*`, `**`, and `***` always select positional, labeled, and complete lanes respectively.
5. Capture, expansion, and type unpacking are distinct operations.
6. A rest binder cannot describe or capture a lane it does not own.
7. Complete `***` capture/expansion is mutually exclusive with split `*`/`**` forms in the same declaration or call.
8. Call assembly evaluates operand expressions exactly once, left to right.
9. Duplicate call labels always fail; no implicit override semantics exist.
10. Callable types classify accepted call packs, not local parameter-binding strategy.
11. Structural annotations are reflective and inert unless explicitly checked.
12. `TupleType` and `ArgumentPackType` are not equal merely because they originated from the same tuple source spelling.

## 6. Review deliverables

A systematic review should produce:

- a contradiction list;
- a parser ambiguity list;
- a runtime representation audit;
- a static-checking soundness audit;
- a reflection/equality audit;
- an error-diagnostic audit;
- a list of unratified provisional rules;
- acceptance-test additions for every discovered gap.
