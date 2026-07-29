
### Core

```ts
@data @sealed
class TypeVariance {
  @variant Covariant
  @variant Contravariant
  @variant Invariant
}
```

```ts
@data @immutable
class TypeParameter {
  const _name: Symbol
  const _owner: TypeConstructor
  const _variance: Variance
  const _bound: Option<Type>
  const _constraints: List<Type>
  const _default: Option<Type>
}
```

```ts
/// 
class Object {
	<>(*args) {}
}
```

```ts
class Mapping<K, V> {
	get(key: K) -> Option<V>
}

Mapping.typeParameters // [TypeParameter(T)]

Vehicle & Car
Automobile | Airplane

// Same as
const [
	TypeParameter(
		name: #K, 
		variance: TypeVariance.Invariant
	),
	TypeParameter(
		name: #V, 
		variance: TypeVariance.Invariant
	),
]
```

```ts
@protocol
class Repository<K, out V> {
  get(key: K) -> Option<V>
  contains(key: K) -> Bool
}
```

```ts
@sealed @data
class Result<T, E> {
	@variant Ok(value: T)
	@variant Err(err: E)
}
```

```ts
@sealed @data
class Result<T, E> {}

@sealed @data
class Ok<T> is Result<T, _> { 
  const _value: T 
}

@sealed @data
class Err<E> is Result<_, E> {
	const _err: E
}
```

```python
@type const CoordinatePairType<T> = (x: T, y: T)

@data
class Point<T> {
	const _x: T
	const _y: T
	
	distance(to other: Point<T>) -> T =>
		Math.sqrt((other.x - x)**2 + (other.y - y)**2)
	
	-(other) -> Point<T> =>
		Point.at(x - other.x, y - other.y)
}

class Shape<T> {
	move(from origin: Coordinate, to destination: Coordinate) {}
}

class Triangle<T> is Shape {
	
}
```
