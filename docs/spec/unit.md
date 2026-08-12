`Unit` is canonical zero-arity product value, represented directly as `Value::Unit`, not heap object:

> `/// The canonical zero-arity product. Unlike Nil, this is surface-visible.`  
> `Unit,`

[value/mod.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/value/mod.rs:42)

Its runtime class resolves to `Unit`:

> `Value::Unit => vm.universe.classes.unit_class`

[value/mod.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/value/mod.rs:142)

`Unit` inherits directly from `Object`, not `Tuple`, `Record`, or `Iterable`:

> `let unit_class = make_core_class(heap, "Unit", object_class, metaclass_class);`

[core_classes.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/universe/core_classes.rs:99)

Its own public methods are only:

```phalcom
class Unit {
  toString => "()"
  hash => 0
}
```

[core.ph](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/core/core.ph:1432)

It also inherits `Object`’s kind/class checks: `is(_)`, `isExactly(_)`, and `isA(_)`. [core.ph](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/core/core.ph:17)

There is no separate empty `Tuple` or empty `Record` instance/class.

- `()` compiles directly to `Value::Unit`.
- `#{}` compiles directly to `Value::Unit`.
- Runtime product finalizers repeat this invariant, preventing empty native heap objects.

```rust
if tuple_expr.entries.is_empty() {
    let idx = self.add_constant(Value::Unit);
```

[expr.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/compiler/lib/expr.rs:706)

```rust
if record_expr.fields.is_empty() {
    let idx = self.add_constant(Value::Unit);
```

[expr.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/compiler/lib/expr.rs:797)

```rust
if positionals.is_empty() && labeled.is_empty() {
    return Ok(Value::Unit);
}
```

[product.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/product.rs:31)

```rust
if fields.is_empty() {
    return Ok(Value::Unit);
}
```

[product.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/product.rs:51)

`Tuple` exists only for positive-arity products. It inherits `Iterable`:

> `let tuple_class = make_core_class(heap, "Tuple", iterable_class, metaclass_class);`

[core_classes.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/universe/core_classes.rs:127)

Public `Tuple` methods include `size`, `positionals`, `labeled`, `labelAt(_)`, `toString`, `at(_)`, `get(_)`, subscript forms, `iteratorValue(_)`, `==(_)`, `!=(_)`, and `hash`. Its raw native backing selectors include `_$size`, `_$at(_)`, `_$positionals`, `_$labeled`, and `_$slice(_,_)`. [core.ph](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/core/core.ph:1442) [primitives.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/universe/primitives.rs:401)

The heap type enforces positivity:

> `assert!(!values.is_empty(), "TupleObject must be positive-arity");`

[tuple.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/heap/tuple.rs:36)

`Record` also exists only for positive arity, but inherits directly from `Object`:

> `let record_class = make_core_class(heap, "Record", object_class, metaclass_class);`

[core_classes.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/universe/core_classes.rs:128)

Current public `Record` methods: `size`, `labelAt(_)`, `==(_)`, and `hash`. Raw native methods: `_$size`, `_$labelAt(_)`, `_$valueAt(_)`. [core.ph](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/core/core.ph:1609) [primitives.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/universe/primitives.rs:410)

Its heap representation likewise rejects an empty value:

> `assert!(!labels.is_empty(), "RecordObject must be positive-arity");`

[record.rs](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/heap/record.rs:13)

The language regression demonstrates identity of both empty literal spellings:

```phalcom
const emptyTuple = ()
const emptyRecord = #{}
System.print(emptyTuple.class == Unit)
System.print(emptyRecord.class == Unit)
System.print(emptyTuple == emptyRecord)
```

[product_regression_tests.ph](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/tests/lang/collections/product_regression_tests.ph:3)

Expected output begins with three `true` lines. [product_regression_tests.expected](/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/tests/lang/collections/product_regression_tests.expected:1)