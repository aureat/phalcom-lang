# Reflection and First-Class Type Values

## 1. Type values

**RATIFIED:** Complete type expressions evaluate to immutable, reflective, first-class `Type` values.

```phalcom
const pairType = (Int, String).asType
const packType = ArgumentPackType.new(
  openPositional: Int,
  openLabeled: String
)
```

Implementations MAY intern and canonicalize Type values.

## 2. Required reflective hierarchy

Recommended public model:

```phalcom
@abstract
class Type {
  satisfiedBy(value) -> Bool
  normalized -> Type
  sourceForm -> Option<TypeSource>
}
```

```phalcom
class TupleType is Type {
  positionals -> const List<Type>
  labels -> const Record<LabelKey, Type>
  repeatedTail -> Option<Type>
}
```

```phalcom
class RecordType is Type {
  fields -> const Record<LabelKey, Type>
}
```

```phalcom
class SetType is Type {
  element -> Type
}
```

```phalcom
class ArgumentPackType is Type {
  fixedPositionals -> const List<Type>
  openPositional -> Option<Type>
  fixedLabels -> const Record<Symbol, Type>
  openLabeled -> Option<Type>
}
```

```phalcom
class CallableType is Type {
  domain -> ArgumentPackType
  result -> Type
}
```

## 3. TupleType versus ArgumentPackType

These Types MUST remain distinct:

```phalcom
type Literal = (*: Int)
const Domain = CallableType.new(
  domain: (*: Int),
  result: Result
)
```

Conceptually:

```phalcom
Literal.class == TupleType
Domain.domain.class == ArgumentPackType
Literal != Domain.domain
```

The shared source syntax does not imply semantic equality.

## 4. Source and normalized forms

Reflection SHOULD preserve both:

```text
source annotation
contextual interpretation
canonical normalized type
```

For:

```phalcom
method(*args: Int)
```

reflection may report:

```text
sourceAnnotation     = Int
interpretedPackType  = (*: Int)
localBindingType     = (Int, ...)
```

For:

```phalcom
const callback: (...) -> R
```

reflection may report:

```text
sourceDomain         = (...)
normalizedDomain     = ArgumentPackType.any
```

## 5. Equality and interning

Type equality is structural after normalization.

```phalcom
(*: Int) -> R == (*: Int) -> R
```

Equivalent generic unpacking normalizes identically:

```phalcom
type P = (Int, name: String)

(***P,) -> R == (Int, name: String) -> R
```

Implementations SHOULD intern canonical Type values so identity comparison may also succeed, but semantic correctness MUST rely on equality, not identity.

## 6. Satisfaction API

**RATIFIED IN PRINCIPLE:** Explicit predicates such as:

```phalcom
List<Int>.satisfiedBy([1, 2, 3])
```

are allowed. Annotations do not invoke them automatically.

Representative examples:

```phalcom
(Int, String).satisfiedBy((1, "a"))
Set<Int>.satisfiedBy(Set.new(1, 2))
(*: Int, **: String).asArgumentPackType.satisfiedBy(
  (1, 2, name: "x")
)
```

The exact contextual conversion API remains provisional.

## 7. Parameter reflection

A reflected parameter SHOULD expose:

```phalcom
parameter.name -> Symbol
parameter.label -> Option<Symbol>
parameter.position -> Int
parameter.sourceType -> Option<TypeSource>
parameter.type -> Option<Type>
parameter.restMode -> Symbol
parameter.packType -> Option<ArgumentPackType>
parameter.bindingType -> Option<Type>
parameter.attributes -> const List<Attribute>
```

## 8. Method reflection

A reflected method SHOULD expose:

```phalcom
method.selector -> Selector
method.parameters -> const List<Parameter>
method.callableType -> CallableType
method.returnType -> Option<Type>
method.typeParameters -> const List<TypeParameter>
```

The callable type represents accepted calls after normalization. Parameter objects preserve the implementation's binding strategy.

## 9. Inert annotations

Type annotations MUST NOT automatically:

- change dispatch;
- wrap values;
- reject calls at runtime;
- alter allocation layout;
- insert collection element checks;
- mutate Tuple, Record, Set, or pack values.

Static checking and explicit reflective satisfaction may use them.
