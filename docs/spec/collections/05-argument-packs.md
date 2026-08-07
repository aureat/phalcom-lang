# Argument-Pack Specification

## 1. Definition

An argument pack is a call-shaped structure with two lanes:

```text
ArgumentPack = ⟨PositionalLane, LabeledLane⟩
```

At runtime, a Tuple is the canonical value capable of preserving both lanes.

A Record embeds as a labeled-only pack. A Set is not a pack.

## 2. ArgumentPackType

**PROVISIONAL, REQUIRED FOR CONSISTENCY:** The type system MUST expose a distinct reflective type:

```phalcom
class ArgumentPackType is Type {
  fixedPositionals -> const List<Type>
  openPositional -> Option<Type>
  fixedLabels -> const Record<Symbol, Type>
  openLabeled -> Option<Type>
}
```

This is not an ordinary `TupleType`, even if it was constructed from tuple syntax.

```phalcom
type Literal = (*: Int)
// TupleType with exact literal label #*.

const callback: (*: Int) -> Result
// CallableType with ArgumentPackType(openPositional: Int).
```

## 3. Pack-schema interpretation

In an argument-pack context, tuple keys receive the following meanings:

```text
#*  open positional lane
#** open labeled lane
```

Example:

```phalcom
(
  Request,
  *: Bytes,
  timeout: Duration,
  **: Metadata
)
```

normalizes to:

```text
ArgumentPackType {
  fixedPositionals: [Request]
  openPositional: Bytes
  fixedLabels: { #timeout: Duration }
  openLabeled: Metadata
}
```

## 4. Exact and open domains

Exact:

```phalcom
(Int, name: String)
```

Open positional:

```phalcom
(*: Int)
```

Open labeled:

```phalcom
(**: String)
```

Open complete:

```phalcom
(*: Int, **: String)
```

Fixed plus open:

```phalcom
(
  Context,
  *: Request,
  format: Format,
  **: Metadata
)
```

## 5. Satisfaction

A call pack:

```text
P = ⟨[p₀, …, pₙ], {k₀ ↦ v₀, …, kₘ ↦ vₘ}⟩
```

satisfies domain `D` exactly when:

1. every fixed positional slot exists and satisfies its corresponding type;
2. additional positional values exist only if `openPositional` is present;
3. every additional positional value satisfies `openPositional`;
4. every fixed label exists and satisfies its corresponding type;
5. additional labels exist only if `openLabeled` is present;
6. every additional labeled value satisfies `openLabeled`;
7. labels are unique;
8. every key is a legal `CallLabel`.

## 6. Rest-binder lane ownership

The binders own these lanes:

```text
*parameter    { positional }
**parameter   { labeled }
***parameter  { positional, labeled }
```

Let `lanes(D)` be the lanes described by annotation domain `D`.

A binder is legal only when:

```text
lanes(D) ⊆ ownedLanes(binder)
```

Valid:

```phalcom
method(*args: (*: Int))
method(**labels: (**: String))
method(***arguments: (*: Int, **: String))
```

Invalid:

```phalcom
method(*args: (**: String))
method(**labels: (*: Int))
method(*args: (*: Int, **: String))
method(**labels: (*: Int, **: String))
```

## 7. Homogeneous shorthand

**RATIFIED:** A non-tuple rest annotation applies homogeneously to the lane or lanes owned by the binder.

```phalcom
*args: T
```

is shorthand for:

```phalcom
*args: (*: T)
```

```phalcom
**labels: T
```

is shorthand for:

```phalcom
**labels: (**: T)
```

```phalcom
***arguments: T
```

is shorthand for:

```phalcom
***arguments: (*: T, **: T)
```

## 8. Exact capture

**RATIFIED:** Tuple-shaped annotations are exact unless openness is explicit.

```phalcom
method(
  ***arguments: (
    Int,
    name: String
  )
)
```

accepts exactly one positional `Int` and exactly one labeled `name: String`.

```phalcom
method(10, name: "Phalcom")
```

Additional arguments fail.

## 9. Split capture

A method may split the residual lanes:

```phalcom
method(
  fixed: Fixed,
  *args: Int,
  option: Bool,
  **labels: String
) {
  ...
}
```

Binding order:

1. fixed positional parameters consume their positions;
2. `*args` captures remaining positionals;
3. fixed labeled parameters consume matching labels;
4. `**labels` captures remaining labels.

The exact parameter-order grammar is specified in the rest/spread document.

## 10. Complete residual capture

**RATIFIED:** `***arguments` is a terminal residual binder that captures both remaining lanes after fixed parameters are matched.

```phalcom
method(
  request: Request,
  timeout: Duration,
  ***remaining: P
)
```

Given:

```phalcom
method(
  requestValue,
  extra1,
  extra2,
  timeout: second,
  debug: true
)
```

bindings are conceptually:

```phalcom
request == requestValue
timeout == second
remaining == (extra1, extra2, debug: true)
```

The annotation describes the residual pack, not the original call.

## 11. Mutual exclusion of capture modes

**RATIFIED:** A declaration chooses one rest-capture mode.

Split mode:

```phalcom
method(*args: A, **labels: B)
```

Complete mode:

```phalcom
method(***arguments: P)
```

They MUST NOT be combined:

```phalcom
method(*args: A, ***remaining: P)      // error
method(**labels: B, ***remaining: P)   // error
```

## 12. Structural kind of local bindings

**RATIFIED:** Capture preserves the annotation's structural kind where meaningful.

```phalcom
method(**labels: String)
```

binds a labeled-only Tuple view.

```phalcom
method(**config: ConnectionConfig)
```

binds a `ConnectionConfig` Record.

```phalcom
method(***arguments: P)
```

binds one Tuple preserving both lanes, unless `P` is a labeled-only Record Type and the residual positional lane is empty.

## 13. Call-label legality

**PROVISIONAL:** An ArgumentPack's labeled lane contains Symbol keys only.

A Tuple or Record with Selector keys remains valid structurally but cannot be expanded into a call pack unless call labels are generalized.

```phalcom
const operations = (+(_): handler)
target(**operations)
// provisional error: Selector-valued call label
```

## 14. Reflection

A parameter reflection object SHOULD expose:

```phalcom
parameter.restMode
// #none | #positional | #labeled | #complete

parameter.sourceAnnotation
parameter.interpretedPackType
parameter.localBindingType
```

For:

```phalcom
method(***arguments: Int)
```

reflection conceptually reports:

```text
sourceAnnotation     = Int
interpretedPackType  = (*: Int, **: Int)
localBindingType     = ArgumentPackType(openPositional: Int, openLabeled: Int)
```
