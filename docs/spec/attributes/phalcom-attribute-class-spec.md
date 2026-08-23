# `@class` — Class-Side Placement Attribute

**Status:** Implemented; this document specifies its role in the native/universe model
**Applies to:** Phalcom class members and fields
**Primary implementation areas:** AST attributes, compiler attribute expansion, semantic surface construction
**Related attributes:** `@native`, `@internal`, `@constructor`

---

## 1. Purpose

`@class` places a member or field on the class side of its enclosing class.

It is a placement attribute. It does not create a class, alter inheritance, imply construction semantics, imply native implementation, or change selector structure.

Example:

```phalcom
class String {
  @class
  new(_ value: Object) {
    ...
  }
}
```

The member is dispatched to `String` as a class object rather than to instances of `String`.

In the native-source model, `@class` is also part of the structural identity used to match a source `@native` anchor to a Rust primitive descriptor.

---

## 2. Class-Side and Instance-Side Protocol

Given:

```phalcom
class Widget {
  build() {
    ...
  }

  @class
  build() {
    ...
  }
}
```

these are two different dispatch entries:

```text
Widget instance side  build()
Widget class side     build()
```

The selector text may be identical because dispatch side is a separate identity dimension.

For native primitives:

```text
PrimitiveKey = (owner, side, selector)
```

therefore `@class` determines the `side` component of a source anchor.

---

## 3. Syntax

`@class` takes no arguments.

```phalcom
@class
make() {
  ...
}
```

```phalcom
@class
version -> String {
  ...
}
```

```phalcom
@class
let _cache
```

Invalid:

```phalcom
@class(true)
make() {
  ...
}
```

Expected diagnostic:

```text
attr.arguments_not_allowed
```

Attribute order is not semantically significant.

---

## 4. Legal Targets

The class-placement model supports:

| Target | Legal |
|---|---:|
| Method | yes |
| Getter | yes |
| Setter | yes |
| Field | yes |
| Class declaration | no |
| Variant declaration | no |
| Index member | no initially unless class-side index methods are separately specified |

Compiler expansion marks the target member or field as class-side.

`@class` is not a modifier of the enclosing class itself.

---

## 5. Semantics

For methods, getters, and setters:

```text
@class
    ⇒ install/resolve member on the class-object dispatch side
```

For fields:

```text
@class
    ⇒ allocate/use class-side field storage according to ordinary class-field rules
```

The attribute does not change the canonical selector.

Example:

```phalcom
@class
fromString(_ text: String) -> Number
```

has:

```text
selector = fromString(_)
side     = class
owner    = Number
```

not a selector such as:

```text
class::fromString(_)
```

---

## 6. Interaction with `@native`

A class-side native primitive is declared:

```phalcom
@class
@native
new(_ value: Object) -> String
```

and matched to a Rust descriptor equivalent to:

```rust
#[primitive(
    String,
    "new(_)",
    side = class,
    ...
)]
```

Bootstrap must reject disagreement.

Source:

```phalcom
@class
@native
new(_ value: Object) -> String
```

Native:

```rust
side = instance
```

must produce:

```text
native.side_mismatch
```

`@native` does not imply `@class`, and `@class` does not imply `@native`.

---

## 7. Interaction with `@internal`

A class-side implementation selector uses both attributes:

```phalcom
@class
@internal
@native
_$allocate(_ size: Int) -> Object
```

This yields:

```text
owner       enclosing class
side        class
selector    _$allocate(_)
visibility  internal
implementation native
```

Each attribute represents one independent axis.

---

## 8. Interaction with `@constructor`

Constructor/factory semantics remain a separate concern.

`@class` means class-side placement.

`@constructor` means the method participates in Phalcom's constructor/factory lowering and constructor rules.

In the current compiler architecture, constructor lowering already establishes class-side factory behavior and explicit `@class` on a constructor is rejected as redundant/invalid. The constructor specification remains authoritative for constructor-specific cases.

Users should therefore not write:

```phalcom
@class
@constructor
new() {
  ...
}
```

when `@constructor` already determines the placement required by constructor semantics.

A native factory that is an ordinary class-side method but is not compiler-lowered as a constructor may use `@class @native`.

---

## 9. Interaction with Types

`@class` does not alter parameter or return type meaning.

```phalcom
@class
parse(_ text: String) -> Result<Number, ParseError>
```

has exactly the written callable type.

The receiver's dispatch side changes, not the selector's type signature.

Type metadata remains excluded from selector identity.

---

## 10. Reflection

Reflection should expose dispatch side separately from selector.

For example:

```text
selector: "new(_)"
side: class
owner: String
```

A class-side method must not require callers or tooling to infer side from naming convention.

Method-family and exact-selector reflection should preserve dispatch side.

---

## 11. LSP

The semantic surface must preserve both sides independently.

A class can contain the same selector on both sides:

```phalcom
class C {
  foo() { ... }

  @class
  foo() { ... }
}
```

Completion, hover, go-to-definition, references, and native-source merging must not collapse them.

For a source `@native` anchor, `@class` must be considered before matching native metadata.

---

## 12. Native Class Distinction

Do not confuse:

```phalcom
@native
class String {
  ...
}
```

with:

```phalcom
class String {
  @class
  ...
}
```

They express unrelated axes.

`@native class String` says the class identity/representation is native-backed.

`@class` on a member says the member is dispatched on the class object.

A native-backed class may contain both instance-side and class-side members.

---

## 13. Examples

### Ordinary class-side factory

```phalcom
class Point {
  @class
  origin -> Point {
    Point.new(x: 0, y: 0)
  }
}
```

### Class-side native primitive

```phalcom
@native
class String {
  @class
  @native
  new(_ value: Object) -> String
}
```

### Class-side source field

```phalcom
class Cache {
  @class
  let _shared
}
```

### Class-side internal native hook

```phalcom
@native
class RuntimeType {
  @class
  @internal
  @native
  _$resolve(_ descriptor: Object) -> Type
}
```

---

## 14. Invalid Uses

Invalid class declaration target:

```phalcom
@class
class Foo {}
```

Invalid argument:

```phalcom
@class(meta)
factory() {}
```

Constructor conflict:

```phalcom
@class
@constructor
new() {}
```

when constructor semantics already imply the required class-side factory.

---

## 15. Diagnostics

Expected diagnostics include:

| Diagnostic | Meaning |
|---|---|
| `attr.arguments_not_allowed` | `@class` has arguments |
| `attr.illegal_target` | target cannot be class-side |
| `native.side_mismatch` | source `@class` side disagrees with native descriptor |
| constructor-specific conflict diagnostic | `@class` redundantly/illegally combined with constructor metadata |

---

## 16. Non-Goals

`@class` does not:

- declare a native class;
- imply static compilation in the Java/C++ sense;
- make a method globally callable;
- alter selector spelling;
- alter parameter or return types;
- imply constructor behavior;
- imply singleton behavior;
- imply internal visibility;
- imply native implementation.

"Class-side" is the preferred semantic term.

---

## 17. Conformance Tests

The implementation must test:

1. `@class` method is class-side;
2. `@class` getter is class-side;
3. `@class` setter is class-side;
4. `@class` field is class-side;
5. instance/class declarations of the same selector remain distinct;
6. argument-bearing `@class(...)` is rejected;
7. illegal targets are rejected;
8. LSP preserves class side;
9. native source matching includes side;
10. class-side source plus instance-side native descriptor fails;
11. `@class @internal @native` produces class/internal/native semantic metadata;
12. constructor conflict rules remain enforced.

---

## 18. Repository Integration

Current implementation is centered on:

```text
phalcom-ast/src/ast.rs
  BuiltinAttr::Class.

phalcom-core/src/compiler/attributes.rs
  ClassExpander marks Method/Getter/Setter/Field as class-side.

phalcom-lsp/src/semantic/surface.rs
  Converts class/static placement into DispatchSide::Class.

phalcom-native-meta
  NativeDispatch::Class.

native source verifier
  Uses source placement as part of PrimitiveKey matching.
```

The defining rule is:

> `@class` selects the class-object dispatch/storage side; it does not change what selector is being declared.
