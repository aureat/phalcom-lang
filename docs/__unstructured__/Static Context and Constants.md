##### Requirements
- `ValuePack<T>`
- `TypePack`
- `LabeledPack`
- `CompleteTypePack`
- associated types, intrinsic methods of some intrinsic types
- pack semantics both for value and static contexts

```ph
ValuePack<*: Int, **: String>
ValuePack<***: Type>
```

```ph
data TypePack<***Es>(***Es)

TypePack(String, age: Int)
```

How to describe the static type of a tuple?

```ph

(Int, ...)
(*: Int)
(*: )

Tuple<Int, ...>
Tuple<Int, String, *Ps>

data ValuePack<***Es: Type>(***elements: ***Es)

data TypePack<***Es>(***Es)

data Tuple<***Es: TypePack>

Tuple<***TypePack(String, age: Int)>
	== Tuple(String, age: Int)
	== (String, age: Int)

data MyTuple<***Es: TypePack>

NamedTuple<***TypePack(String, age: Int)>
	== NamedTuple(String, age: Int)

```

```ph
Tuple<***TypePack(***: Int)>
	== Tuple<***: Int>
  == (***: Int)

Tuple<***TypePack(Int, ...)>
	== Tuple<Int, ...>
	== (Int, ...)
```
