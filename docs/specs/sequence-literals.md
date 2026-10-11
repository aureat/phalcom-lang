# Sequence Literals

**Status:** Normative

**Semantic ownership:** homogeneous bracket syntax, collection-family selection,
contextual element inference, and runtime/static sequence defaults. Static
evaluation details and runtime materialization are owned by
[`static-evaluation.md`](static-evaluation.md) and the VM specifications.

## 1. Sequence families

Phalcom has three homogeneous bracket-sequence families:

| Family | Extent | Mutability |
|---|---|---|
| `List<T>` | Runtime, dynamically resizable | Mutable elements and size |
| `Array<T>` | Runtime, fixed after construction | Mutable elements for ordinary runtime values |
| `Vector<T, N>` | Exact static extent `N` | Immutable |

The families are nominally distinct. Equal contents do not make a value of one
family equal to a value of another family.

## 2. Syntax

An unheaded bracket literal is written as `[...]`. It is a primary expression
that constructs a homogeneous sequence. Elements and spread contributions are
evaluated in source order.

An explicit literal head selects a fixed family:

```telos
let array: Array<Int> = <Int>[1, 2, 3]
let vector: Vector<Int, 3> = <Int, 3>[1, 2, 3]
```

The one-parameter head `<T>[...]` explicitly selects `Array<T>`. The
two-parameter head `<T, N>[...]` explicitly selects `Vector<T, N>`.

Bracket construction is separate from subscript expressions such as `items[i]`
and `items[i] = value`, whose selector semantics are specified by
[`callables/selectors.md`](callables/selectors.md). A computed Record field key
written `[expression]` is also a separate syntax form.

## 3. Family selection

The checked sequence family is selected in this order:

1. An explicit literal head selects `Array` or `Vector`.
2. A unique expected `List<T>`, `Array<T>`, or `Vector<T, N>` selects that
   family.
3. Otherwise, an unheaded runtime expression selects `Array<T>` and an
   unheaded static expression selects `Vector<T, N>`.

An ordinary unique expected type can guide the element type and, for a Vector,
its extent. This includes a type context with inference holes such as
`List<_>` or `Array<_>`. The literal constructs the selected family directly;
it does not first allocate another family and convert it.

The phase default applies only when no explicit or unique expected family has
selected the result. It does not rank overload candidates or resolve an
otherwise ambiguous call. When more than one collection family remains viable,
the program must provide a family context or explicit literal head.

`List<T>` remains available through an explicit expected type or a named List
constructor. A bare literal does not default to List.

The phase of a binding is specified by
[`bindings-and-fields.md`](bindings-and-fields.md): runtime `let`/`let mut`
expressions use the runtime default, and `const` initializers use the static
default.

## 4. Homogeneity and extent

Every element and spread contribution must be compatible with one element type
`T`, inferred from its values and context. A broader element type such as
`Dynamic` may be supplied explicitly when values from different nominal types
are intended to share a sequence.

`Array<T>` records its produced runtime extent and cannot grow or shrink after
construction. Element replacement remains permitted for an ordinary runtime
Array.

`Vector<T, N>` is immutable, and `N` equals the exact number of elements
produced after its spreads are resolved. An extent must be an exact nonnegative
static integer. A phase default or an empty literal does not invent an element
type when no element or contextual evidence provides one.

## 5. Static sequence values

Static evaluation and constant materialization follow
[`static-evaluation.md`](static-evaluation.md). A static bare literal selects
`Vector<T, N>`. A contextual or explicit `Array<T>` may also be evaluated as a
constant Array; its runtime materialization preserves the frozen const-Array
rule owned by static evaluation.

A closed static Vector value requires an exact extent. If Vector identity
spread evaluation still has a symbolic extent, the evaluator returns a
symbolic outcome and publishes no closed sequence value. It may reduce the
literal after the extent becomes a closed exact static integer. Static
evaluation does not introduce symbolic extent arithmetic or perform generic
type substitution to force that reduction.

## 6. Examples

```telos
let runtime = [1, 2, 3]
// Array<Int>

const fixed = [1, 2, 3]
// Vector<Int, 3>

let resizable: List<Int> = [1, 2, 3]
let explicitArray = <Int>[1, 2, 3]
let explicitVector = <Int, 3>[1, 2, 3]
```

An empty literal has known extent zero, but no element-type evidence. The
surrounding context or ordinary inference rules must determine `T`; there is no
implicit fallback element type.

## 7. Authority

Semantic analysis owns family and element-type selection. Lowering and runtime
construction consume the checked result and must not reconstruct family
identity from syntax or infer it from the execution phase. TDR-0091 amends the
historical List-literal rules in TDR-0025 and TDR-0028.
