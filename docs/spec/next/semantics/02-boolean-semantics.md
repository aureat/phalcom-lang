# Phalcom Boolean Semantics and Explicit Boolean Conversion

**Status:** Normative design specification  
**Scope:** Conditions, Boolean operators, contracts and guards, truthiness rejection, `Bool.from(_)`, `ToBool`, and the recognized `toBool()` conversion method.

---

## 1. Core rule

Every Boolean context in Phalcom requires an actual `Bool` value.

```phalcom
if condition {
  ...
}
```

is valid only when `condition` evaluates to `true` or `false`.

Phalcom does not provide built-in truthiness and does not perform implicit Boolean conversion.

The following are invalid:

```phalcom
if 1 {
  ...
}
```

```phalcom
if "text" {
  ...
}
```

```phalcom
if [] {
  ...
}
```

```phalcom
if None {
  ...
}
```

```phalcom
if () {
  ...
}
```

A runtime failure should identify the actual value or type:

```text
ConditionTypeError:
Expected Bool, received Unit.
```

A static checker should report an equivalent type diagnostic before execution when possible.

---

## 2. Boolean contexts

The strict-`Bool` rule applies uniformly to:

- `if` conditions;
- `while` conditions;
- loop guards;
- match guards;
- comprehension filters;
- `not`;
- both operands of `and` and `or`;
- assertions that accept a condition;
- `Property.assume` and similar APIs;
- `@requires`;
- `@ensures`;
- `@invariant`;
- any future language construct explicitly defined as a Boolean context.

Examples:

```phalcom
@requires(index >= 0)
```

is valid because comparison produces `Bool`.

```phalcom
@requires(index)
```

is invalid because `Int` is not `Bool`.

---

## 3. Boolean operators

`not`, `and`, and `or` operate only on `Bool`.

```phalcom
not true
true and false
false or true
```

are valid.

```phalcom
1 and 2
```

is invalid.

`and` and `or` short-circuit, but always produce a `Bool`. They do not return one of their original operands as Python does.

Conceptually:

```text
not : Bool -> Bool
and : (Bool, lazy Bool) -> Bool
or  : (Bool, lazy Bool) -> Bool
```

The lazy operand notation describes short-circuit evaluation; it does not require these operators to be ordinary methods.

---

## 4. No implicit `toBool()` in conditions

A user-defined `toBool()` method shall not be invoked automatically by control flow.

```phalcom
class Queue {
  toBool() -> Bool {
    return not self.isEmpty
  }
}
```

This remains invalid:

```phalcom
if queue {
  ...
}
```

The caller must convert explicitly:

```phalcom
if Bool.from(queue) {
  ...
}
```

or invoke the method directly:

```phalcom
if queue.toBool() {
  ...
}
```

Strict Boolean conditions therefore remain genuinely strict. Explicit conversion is an ordinary operation chosen by the programmer, not a hidden language coercion.

---

## 5. `ToBool` protocol

Phalcom defines a structural conversion protocol:

```phalcom
@protocol
class ToBool {
  toBool() -> Bool
}
```

A type conforms structurally when it defines the exact selector:

```phalcom
toBool()
```

with compatible reflective typing.

An explicit nominal declaration may be allowed for documentation and eager verification:

```phalcom
@conforms(ToBool)
class Queue {
  toBool() -> Bool {
    return not self.isEmpty
  }
}
```

Nominal declaration is not required for structural conformance.

### 5.1 Why `toBool()` uses parentheses

The conversion is an explicit method rather than a getter because it may perform nontrivial domain logic. The parentheses communicate an operation rather than passive field access.

### 5.2 Recognized conversion hook

`toBool()` is a recognized conversion hook only when reached through explicit Boolean conversion APIs such as `Bool.from(_)`.

Its existence does not alter:

- condition evaluation;
- `and`, `or`, or `not`;
- overload resolution;
- ordinary dispatch;
- equality;
- pattern matching.

---

## 6. `Bool.from(_)`

Phalcom provides an explicit class-side conversion operation:

```phalcom
Bool.from(value)
```

The user requested surface may be represented reflectively by the exact selector corresponding to `Bool::#from(_)`; ordinary source uses Phalcom's established class-side message syntax.

### 6.1 Conversion algorithm

`Bool.from(value)` performs the following steps:

1. If `value` is already a `Bool`, return it unchanged.
2. Otherwise, determine whether `value` structurally satisfies `ToBool`.
3. If it does, invoke `value.toBool()` exactly once.
4. Require the returned runtime value to be an actual `Bool`.
5. If the value does not satisfy `ToBool`, raise `BooleanConversionError`.
6. If `toBool()` returns a non-`Bool`, raise `InvalidBooleanConversionError`.

Conceptually:

```phalcom
@class
from(value: Any) -> Bool {
  if value is Bool {
    return value
  }

  if ToBool.satisfiedBy(value) {
    const converted = value.toBool()

    if not converted is Bool {
      throw InvalidBooleanConversionError.new(
        value: value,
        result: converted
      )
    }

    return converted
  }

  throw BooleanConversionError.new(value)
}
```

The pseudocode uses type tests illustratively. Exact reflection and type-test syntax are defined elsewhere.

### 6.2 No recursive conversion

If `toBool()` returns a non-`Bool`, `Bool.from(_)` shall not recursively attempt to convert the returned value.

This is invalid:

```phalcom
class Broken {
  toBool() {
    return AnotherConvertible.new()
  }
}
```

The result must be an actual `Bool` immediately. This prevents conversion loops and unpredictable chains.

### 6.3 No implicit fallback conversions

`Bool.from(_)` does not automatically define meanings for:

- zero and nonzero numbers;
- empty and nonempty strings;
- empty and nonempty collections;
- `None`;
- `()`;
- arbitrary objects.

Core types may implement `ToBool` only through a separate, explicit language decision. The initial model defines no such built-in conversions.

### 6.4 Identity conversion

```phalcom
Bool.from(true)
// true
```

```phalcom
Bool.from(false)
// false
```

The exact `Bool` value is returned unchanged.

`Bool` does not need a redundant instance method `toBool()` merely to support this identity case.

---

## 7. Error model

Recommended errors:

```phalcom
class BooleanConversionError is TypeError {
  ...
}
```

Raised when no explicit Boolean conversion is defined.

```phalcom
class InvalidBooleanConversionError is TypeError {
  ...
}
```

Raised when `toBool()` returns a non-`Bool` value.

A useful diagnostic includes:

- the original value's class;
- the exact conversion selector;
- the returned value's class, when applicable;
- source location of the explicit `Bool.from(_)` call;
- source location of the faulty `toBool()` implementation where available.

---

## 8. Side effects and failure

`Bool.from(value)` is an explicit ordinary call. Therefore a user-defined `toBool()` may technically:

- mutate state;
- throw;
- perform I/O;
- block;
- return different results across calls.

The language does not silently hide these effects because conversion occurs only through an explicit call.

Libraries should strongly prefer pure, deterministic `toBool()` implementations.

A future `@pure` protocol requirement or lint may enforce or recommend this property, but purity is not assumed by the core runtime.

---

## 9. Generic code

Generic code that requires explicit Boolean convertibility should state the constraint:

```phalcom
check<T: ToBool>(value: T) -> Bool {
  return Bool.from(value)
}
```

A generic function accepting unconstrained `T` cannot use the value as a condition:

```phalcom
check<T>(value: T) {
  if value {
    ...
  }
}
```

This remains invalid even when some possible `T` values define `toBool()`.

---

## 10. Design rationale

Implicit truth conversion is rejected because it would:

- hide message sends inside control flow;
- permit condition evaluation to mutate, throw, block, or perform I/O invisibly;
- make method addition silently change whether existing code type-checks;
- obscure domain intent such as `isEmpty`, `isSome`, or `isOk`;
- complicate generic constraints and static reasoning;
- create ambiguity for nested option and result values;
- weaken the invariant that every Boolean context receives a `Bool`.

Explicit `Bool.from(_)` retains extensibility without weakening control-flow semantics.

---

## 11. Normative decisions summary

- Every Boolean context requires an actual `Bool`.
- Phalcom has no built-in truthiness.
- No built-in value automatically converts to `Bool`.
- `None` and `()` are not Boolean conditions.
- `not`, `and`, and `or` accept and return only `Bool`.
- `and` and `or` short-circuit but do not return arbitrary operands.
- `toBool()` is never invoked implicitly by control flow.
- `ToBool` is a structural protocol with exact requirement `toBool() -> Bool`.
- `Bool.from(value)` is the canonical explicit conversion entry point.
- `Bool.from(Bool)` is identity conversion.
- `Bool.from(_)` invokes `toBool()` at most once.
- The conversion result must be an actual `Bool`.
- Conversion is not recursive.
- Core types initially do not implement automatic Boolean conversions.
