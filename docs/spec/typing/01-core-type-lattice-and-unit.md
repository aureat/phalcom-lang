# Phalcom Core Type Lattice and Unit Semantics

**Status:** Normative design specification  
**Scope:** `Never`, `()`, `Unit`, `None`, `Option<T>`, `Any`, `Dynamic`, `Object`, callable domains, ordinary return behavior, and associated normalization rules.

---

## 1. Purpose

This specification defines the foundational relationships among Phalcom's universal, bottom, unit, and absence types.

The design distinguishes four fundamentally different concepts:

```text
Never
    No possible value.
    Describes a computation that cannot complete normally.

()
    Exactly one possible value: ().
    The empty tuple value and the unit type.
    Describes successful completion without a payload.

None
    The singleton nullary variant of Option<T>.
    Describes the absence of an optional payload.

Any
    The safe top type.
    Accepts every possible runtime value.
```

`Dynamic` remains a separate gradual-typing escape hatch and is not an ordinary top type.

---

## 2. Core type relationships

For every type `T`:

```text
Never <: T <: Any
```

`Object` remains the root of the ordinary runtime class hierarchy:

```text
Object <: Any
```

`Never` and `Any` are type-theoretic extrema. They do not participate in ordinary nominal inheritance as user-extensible superclasses.

The unit type is the exact zero-slot tuple type:

```text
() <: Tuple <: Object <: Any
```

For covariant `Option<T>`:

```text
Never <: T
────────────────────────
Option<Never> <: Option<T>
```

---

## 3. Cardinality model

| Type | Possible values | Meaning |
|---|---:|---|
| `Never` | 0 | No normal result is possible |
| `()` | 1 | Successful completion without a payload |
| `Option<Never>` | 1 | Absence; its sole value is `None` |
| `Bool` | 2 | `true` or `false` |
| `Option<()>` | 2 | `None` or `Some(())` |
| `Any` | Every runtime value | Universal safe supertype |

`()` and `None` are distinct values with distinct meanings:

```phalcom
initialize() -> () {
  ...
}
```

means that initialization completed successfully without returning useful data.

```phalcom
findUser(id: UserId) -> Option<User> {
  ...
  return None
}
```

means that no `User` value is present.

The following type preserves both states:

```phalcom
tryInitialize() -> Option<()> {
  ...
}
```

Its possible values are:

```phalcom
Some(())
None
```

---

## 4. `Never`

### 4.1 Definition

`Never` is the canonical uninhabited bottom type.

There is no runtime value `v` such that:

```text
v : Never
```

The identifier `Never` denotes a first-class reflective type object. That type object is an instance of the relevant metatype, such as `Type`; it is not an inhabitant of `Never`.

### 4.2 Producers of `Never`

Expressions that cannot complete normally have type `Never`, including:

- `throw`;
- `return` within its enclosing control-flow context;
- statically nonterminating loops;
- process termination;
- panic or fatal-error primitives;
- other control-transfer expressions that cannot produce a value at their position.

Example:

```phalcom
fail(message: String) -> Never {
  throw Failure.new(message)
}
```

### 4.3 Bottom-type behavior

`Never` is a subtype of every type:

```text
Never <: T
```

This permits nonreturning branches to coexist with value-producing branches:

```phalcom
parse(text: String) -> Int {
  if text.isEmpty {
    return fail("Empty input")
  }

  return parseInt(text)
}
```

### 4.4 Type normalization

The type system shall apply:

```text
T | Never = T
T & Never = Never
```

A branch whose result is `Never` does not widen the join of the remaining normal branches.

### 4.5 Prohibited operations

The following are invalid:

```phalcom
Never.new()
```

```phalcom
const impossible: Never = ()
```

```phalcom
class UserNever is Never {}
```

`Never` is intrinsic, sealed, and uninstantiable.

---

## 5. Unit and the empty tuple

### 5.1 Identity

Phalcom defines:

```phalcom
()
```

as all of the following:

1. the empty tuple value;
2. the unit value;
3. the exact empty tuple type;
4. the unit result type.

`Unit` is the reflective runtime name of this exact type.

Recommended reflection:

```phalcom
().class
// Unit
```

```phalcom
Unit.superclass
// Tuple
```

The canonical source spelling in type expressions is `()`:

```phalcom
save() -> () {
  ...
}
```

`Unit` may appear in reflection and implementation-facing metadata, but source-level APIs should prefer `()`.

### 5.2 Singleton identity

There is exactly one observable unit value:

```phalcom
() == ()
// true
```

```phalcom
() === ()
// true
```

The runtime shall not expose distinguishable unit instances.

### 5.3 Tuple behavior

Because unit is the empty tuple:

```phalcom
().size
// 0
```

```phalcom
().isEmpty
// true
```

```phalcom
().labels
// ()
```

Iteration visits no elements:

```phalcom
().each { value =>
  // never invoked
}
```

Indexing fails normally:

```phalcom
().at(0)
// BoundsError
```

### 5.4 Expansion

Expanding the empty tuple supplies no arguments:

```phalcom
target(*())
```

is equivalent to:

```phalcom
target()
```

### 5.5 Storage and passing

Unit is a first-class value and may be stored, passed, compared, matched, and included in other values:

```phalcom
const value = ()
const values = [(), (), ()]
const result: Option<()> = Some(())
```

### 5.6 Pattern matching

```phalcom
match value {
  () => System.print("completed")
}
```

A match over the exact unit type is trivially exhaustive.

### 5.7 Hashing

Unit is stably hashable with a fixed hash value.

```phalcom
const map = Map.new()
map.put((), "completed")
```

### 5.8 Prohibited operations

Unit cannot be mutated:

```phalcom
()[0] = value
().append(value)
().removeAt(0)
```

Alternative public construction is prohibited:

```phalcom
Unit.new()
```

Any generic tuple factory that produces zero entries shall canonicalize to `()`.

No implicit conversions exist between unit and absence, numbers, or booleans:

```phalcom
() == None
// false
```

```phalcom
const absent: Option<Int> = ()
// type error
```

---

## 6. Ordinary callable return semantics

### 6.1 Fallthrough

An ordinary brace-bodied callable that reaches the end of its body returns `()`.

```phalcom
log(message: String) -> () {
  System.print(message)
}
```

is semantically equivalent to:

```phalcom
log(message: String) -> () {
  System.print(message)
  return ()
}
```

### 6.2 Bare return

A bare return is equivalent to returning unit:

```phalcom
return
```

is equivalent to:

```phalcom
return ()
```

### 6.3 Missing annotations

An omitted return annotation does not imply `()`.

```phalcom
calculate() {
  return 42
}
```

has no explicitly declared return type. A checker may infer a type or treat it according to Phalcom's gradual-typing rules.

Reflection must distinguish:

```text
declaredReturnType = absent
```

from:

```text
declaredReturnType = ()
```

### 6.4 Fallthrough from a non-unit declaration

```phalcom
calculate(flag: Bool) -> Int {
  if flag {
    return 42
  }
}
```

has a path that returns `()`. A checker shall diagnose that the declared result `Int` is not satisfied on all normal paths.

Runtime behavior remains defined: actual fallthrough returns `()`.

### 6.5 No tail-expression implication

Unit semantics do not imply tail-expression returns. Brace-bodied named callables return a non-unit value only through explicit `return expression`.

Expression-bodied methods are specified separately.

---

## 7. Callable domains

The left side of a callable type is an ordered tuple-shaped parameter domain.

```phalcom
() -> Result
```

means a callable with zero parameters.

```phalcom
() -> ()
```

means a callable with zero parameters that completes with unit.

A callable taking one positional unit argument uses a one-slot tuple domain:

```phalcom
((),) -> Result
```

These are distinct:

| Callable type | Meaning |
|---|---|
| `() -> R` | Zero parameters |
| `(Int,) -> R` | One positional `Int` parameter |
| `((),) -> R` | One positional unit parameter |
| `(signal: ()) -> R` | One labeled unit parameter |

---

## 8. `None` and `Option<T>`

### 8.1 Variant model

`None` is the singleton nullary variant of covariant `Option<T>`:

```phalcom
@sealed @data
class Option<out T> {
  @variant Some(value: T)
  @variant None
}
```

### 8.2 Principal type

Because `None` contains no `T`, its principal unconstrained type is:

```phalcom
Option<Never>
```

Therefore:

```phalcom
const absent = None
```

infers `Option<Never>`.

Covariance permits widening:

```phalcom
const absentUser: Option<User> = absent
```

### 8.3 Runtime singleton

All absent options may share one runtime singleton:

```phalcom
const a: Option<Int> = None
const b: Option<String> = None

a === b
// true when generic arguments are erased at runtime
```

Their static contextual types remain distinct.

### 8.4 Public and internal naming

Public APIs use `None` and `Option<T>`.

The runtime may use an internal behavior such as `_None` or `Option.None`, but user annotations should not require a public `NoneType`.

---

## 9. `Any`, `Object`, and `Dynamic`

### 9.1 `Any`

`Any` is the safe top type:

```text
T <: Any
```

A value statically typed `Any` permits only operations guaranteed for every value, ordinarily those exposed by the universal object protocol.

```phalcom
value: Any
value.toString
value.class
```

An arbitrary domain-specific message requires narrowing.

### 9.2 `Dynamic`

`Dynamic` is the unchecked gradual-typing escape hatch.

```phalcom
value: Dynamic
value.render()
value.fly()
```

The checker permits these sends and runtime dispatch determines success.

`Dynamic` is not an ordinary top type and should not participate in subtype reasoning as though it were `Any`.

### 9.3 `Object`

`Object` is the root of the ordinary runtime class hierarchy:

```text
Object <: Any
```

`Any` and `Never` do not become ordinary superclass entries.

---

## 10. Constructor interaction

A constructor has two conceptual layers:

```text
class-side constructor wrapper:
    allocate instance
    invoke instance initializer
    return instance

instance-side initializer:
    initialize receiver
    complete with ()
```

For:

```phalcom
@constructor
new(name: String) {
  _name = name
}
```

the externally visible callable type is:

```phalcom
(String,) -> Self
```

The initializer body itself falls through with internal unit.

Rules:

- bare `return` may terminate initialization early;
- `return ()` may be allowed but is redundant;
- returning an arbitrary alternative value from a constructor body is forbidden;
- the class-side wrapper returns the allocated instance.

---

## 11. Type normalization and callable relationships

The type engine shall normalize:

```text
T | Never = T
T & Never = Never

T | Any = Any
T & Any = T

Option<Never> <: Option<T>
() <: Tuple
() != Option<Never>
```

Callable result covariance implies:

```text
() -> Never
    <:
() -> T
```

A callable that never returns normally can satisfy any result expectation.

But:

```text
() -> ()
```

is not a subtype of:

```text
() -> Int
```

---

## 12. `Void`

Phalcom shall not define a distinct `Void` type.

| Intended meaning | Phalcom representation |
|---|---|
| Completes without useful data | `()` |
| Never returns normally | `Never` |
| Optional value is absent | `None` within `Option<T>` |
| Return type is unchecked or unknown | `Dynamic` or absent annotation |

Foreign interfaces should map:

```text
C void return      → ()
C noreturn function → Never
```

---

## 13. Optimization requirements and permissions

The language semantics permit the implementation to optimize unit aggressively.

### 13.1 Canonical representation

The VM may represent `()` as:

- a dedicated immediate tag;
- a reserved object handle;
- an immortal singleton;
- a VM constant.

No ordinary heap allocation is required.

### 13.2 Dedicated return

Bytecode may provide a dedicated operation such as:

```text
RETURN_UNIT
```

### 13.3 Elided materialization

When a unit result is unused, optimized code may avoid materializing or moving the singleton, provided observable behavior is unchanged.

### 13.4 Zero-sized fields

A field whose exact type is `()` need not consume payload storage. Reflection and field presence must remain intact.

### 13.5 Compact algebraic representations

```phalcom
Option<()>
```

has two states and may use a compact discriminant.

```phalcom
Result<(), E>
```

needs no success payload storage.

```phalcom
Result<T, Never>
```

has no possible error payload and may be represented like `T`, while preserving reflective type identity.

### 13.6 Empty expansion

```phalcom
target(*())
```

may compile exactly like `target()`.

---

## 14. Normative decisions summary

- `Never` is the intrinsic uninhabited bottom type.
- `Never` is a subtype of every type.
- `()` is the empty tuple value, unit value, empty tuple type, and unit type.
- `Unit` is the reflective name of `()`.
- `Unit <: Tuple`.
- Exactly one observable unit value exists.
- Ordinary callable fallthrough returns `()`.
- Bare `return` means `return ()`.
- Missing return annotation does not mean `()`.
- Zero-argument callable domains use `()`.
- One unit argument uses `((),)`.
- `None` is distinct from `()`.
- `None` is the nullary `Option<T>` variant.
- Unconstrained `None` has principal type `Option<Never>`.
- `Option<out T>` is covariant.
- `Any` is the safe top type.
- `Dynamic` is separate from `Any`.
- `Object` remains the ordinary runtime root.
- No separate `Void` type exists.
- Unit is stably hashable and may be optimized as a zero-payload singleton.
