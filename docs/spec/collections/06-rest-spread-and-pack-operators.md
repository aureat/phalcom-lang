# Rest, Spread, and Pack Operators

## 1. Operator family

**AMENDED AND RATIFIED:**

```text
*     positional lane
**    labeled lane
***   complete pack
```

Each token is reserved in pack grammar and is not a freely overloadable arithmetic operator.

The lexer MUST prefer longest-token matching:

```text
*** before ** before *
```

## 2. Three operations, one lane algebra

### 2.1 Capture

```phalcom
*args
**labels
***arguments
```

select lanes from an incoming call pack and bind local variables.

### 2.2 Expansion

```phalcom
target(*value)
target(**value)
target(***value)
```

select lanes from a value and contribute them to an outgoing call.

### 2.3 Type unpacking

```phalcom
(*P,) -> R
(**P,) -> R
(***P,) -> R
```

select lanes from a tuple/pack Type and insert them into a callable domain.

## 2.4 Active outgoing-expansion rules (F.1/F.2)

Sections 3–6 below are retained as superseded design history where they
contradict this amendment. Current source and runtime rules are:

```text
Tuple construction and calls share pack source phases.

positional phase: ordinary positional, *expr, ***expr
labeled phase:    explicit/computed label, **expr

explicit/computed label and ** start labeled phase.
*** does not start labeled phase.
multiple *** forms are legal before labeled phase.
* / ** / *** may mix subject to those phases.
```

Operand behavior:

```text
*   Tuple/Unit direct positional lane; otherwise Iterable cursor protocol
    iterate(_) + iteratorValue(_)
**  Unit, Tuple labeled lane, Record, or Map with Symbol keys
*** Unit or Tuple only
```

Generic `*` is compiled as ordinary cursor sends, so iterator exceptions,
visibility, fibers, and user overrides retain their normal semantics. Record
is not a `***` operand. Incoming `**rest`/`***rest` remain parsed but rejected
until F.3; current U9 captures only final positional `*rest` into `List`.

## 3. Value expansion scope — superseded history

> Superseded by §2.4. Kept only as historical design record; its call-only
> restriction is not current behavior.

**RATIFIED:** Value expansion is legal only in call argument lists.

Valid:

```phalcom
target(*positionals)
target(**labels)
target(***arguments)
```

Invalid:

```phalcom
(prefix, *values)
{ base: value, **fields }
Set{*values}
```

## 4. Split call expansion — superseded history

> Superseded by §2.4. In particular, explicit labels may follow `**`, and
> Tuple construction participates in the same source-phase grammar.

**RATIFIED:** A call may contain multiple `*` and multiple `**` expansions.

```phalcom
target(
  fixed,
  *first,
  *second,
  timeout: duration,
  **defaults,
  **metadata
)
```

Rules:

1. Every expression is evaluated exactly once, left to right.
2. All ordinary positionals and `*` expansions precede the first explicit label or `**` expansion.
3. Once the first `**` appears, only further `**` expansions may follow.
4. `*` contributes only positional entries.
5. `**` contributes only labeled entries.
6. Duplicate labels fail.
7. No source-order overriding exists.

Canonical grammar:

```text
CallArguments(split) ::= PositionalItem* FixedLabel* LabeledExpansion*
PositionalItem ::= Expression | "*" Expression
LabeledExpansion ::= "**" Expression
```

## 5. Complete call expansion — superseded history

> Superseded by §2.4. `***` may mix with `*` and `**` when the shared source
> phases permit it, and multiple `***` contributions are legal.

**RATIFIED:** A call using `***` MUST NOT also use `*` or `**` expansion.

Valid:

```phalcom
target(***arguments)
```

Invalid:

```phalcom
target(*prefix, ***arguments)
target(***arguments, **metadata)
```

### 5.1 Multiplicity

**PROVISIONAL:** At most one `***` expansion may occur in a call.

Rationale: two complete expansions may both contribute positionals after the first has closed the positional section.

```phalcom
target(***first, ***second)
// provisional error
```

### 5.2 Explicit arguments around `***`

**PROVISIONAL RECOMMENDATION:** The canonical complete-expansion form is:

```phalcom
target(
  fixedPositionals,
  ***pack,
  fixedLabels
)
```

Rules:

1. explicit positionals may precede `***pack`;
2. no positional argument may follow `***pack`;
3. explicit labels may follow `***pack`;
4. duplicates between pack labels and explicit labels fail;
5. `***pack` is evaluated once at its source position.

This placement was proposed but not explicitly ratified and is listed for review.

## 6. Expansion operand requirements — superseded history

> Superseded by §2.4. Current `*` supports Tuple/Unit direct lanes plus the
> generic Iterable cursor protocol; current `***` accepts Unit/Tuple only.

### 6.1 `*value`

The operand MUST expose a positional lane.

Valid:

```phalcom
target(*(1, 2, 3))
```

Invalid:

```phalcom
target(*record)
target(*set)
```

### 6.2 `**value`

The operand MUST expose a labeled lane or embed as labeled structure.

Valid:

```phalcom
target(**tuple)
target(**record)
```

For a Tuple, positionals are ignored.

### 6.3 `***value`

The operand MUST be a pack-capable Tuple or a labeled-only Record embedding.

```phalcom
target(***tuple)
target(***record)
```

A Record contributes an empty positional lane.

## 7. Rest parameter ordering

### 7.1 Split mode

**RATIFIED:** Parameter declarations follow Python-like lane ordering:

```text
fixed positionals
*positionalRest
fixed labeled parameters
**labeledRest
```

Example:

```phalcom
log(
  category: Symbol,
  *values: Any,
  format: Format,
  **fields: LogValue
)
```

`**fields` MUST be final.

### 7.2 Complete mode

**RATIFIED:** `***remaining` is terminal and may follow fixed parameters.

```phalcom
method(
  request: Request,
  timeout: Duration,
  ***remaining: P
)
```

No parameter follows `***remaining`.

### 7.3 Mutual exclusion

Split and complete modes MUST NOT coexist in one declaration.

## 8. Type-level unpack expressions

**RATIFIED:** Type-level unpacking follows the same lane meanings.

For:

```phalcom
type P = (
  Request,
  timeout: Duration
)
```

then:

```phalcom
(*P,) -> R
```

normalizes to:

```phalcom
(Request) -> R
```

```phalcom
(**P,) -> R
```

normalizes to:

```phalcom
(timeout: Duration) -> R
```

```phalcom
(***P,) -> R
```

normalizes to:

```phalcom
(Request, timeout: Duration) -> R
```

## 9. First-class pack projections

**RATIFIED IN PRINCIPLE:** An unpack/projection expression may be stored when observed.

```phalcom
const expansion = ***P
```

The runtime value SHOULD be a reflective immutable descriptor:

```phalcom
PackProjection.new(
  mode: #complete,
  operand: P
)
```

In a pack-consuming context, the compiler/VM MAY consume the projection without allocating the descriptor.

The exact public class name and API are **OPEN**.

## 10. Evaluation and failure timing

Call-argument expressions are evaluated left to right.

A dynamically invalid expansion operand fails when its contribution is assembled.

```phalcom
target(
  sideEffectA(),
  *invalidValue,
  sideEffectB()
)
```

`sideEffectA()` occurs before the expansion error. `sideEffectB()` does not occur if assembly fails first.

Static analysis MAY diagnose invalid operands earlier when their types are known.

## 11. Duplicate labels

All call contributions participate in one duplicate check:

```phalcom
target(x: 1, **(x: 2))
target(**first, **second)
target(***pack, x: 2)
```

Each fails if `#x` appears more than once.

Duplicate detection is based on `CallLabel` identity, not textual spelling.
