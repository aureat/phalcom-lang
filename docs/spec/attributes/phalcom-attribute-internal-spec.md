# `@internal` — Implementation-Namespace Declaration Attribute

**Status:** Proposed language specification
**Applies to:** Phalcom source declarations
**Primary implementation area:** `phalcom-ast`, `phalcom-core`, `phalcom-lsp`
**Related namespaces:** `_$name` implementation selectors, `__name` implementation fields
**Related attributes:** `@native`, `@class`, `@private`, `@protected`

---

## 1. Purpose

`@internal` marks a source declaration as an intentional declaration of Phalcom implementation protocol or implementation storage.

It exists to make implementation-only declarations explicit and readable in canonical source, especially in the core `universe` project. It is an assertion and presentation attribute. It is **not** the authority that creates internal access.

The implementation namespace remains authoritative:

```text
_$name   implementation selector
__name   implementation field
```

A declaration such as:

```phalcom
@internal
_$byteCount -> Int
```

does not become internal because `@internal` is present. It is internal because the selector belongs to the `_$` implementation namespace. `@internal` states that the author intended this fact and allows the compiler, bootstrap verifier, LSP, documentation generator, and reflection tooling to check and present it consistently.

The same rule applies to implementation fields:

```phalcom
@internal
let __handle: Int
```

The field is implementation storage because its name is in the `__` namespace. `@internal` makes that status explicit in canonical source.

---

## 2. Design Principle

Phalcom distinguishes **namespace semantics** from **declaration metadata**.

The namespace establishes authority:

```text
_$selector  => implementation selector
__field     => implementation storage
```

The attribute establishes an explicit assertion:

```text
@internal   => this source declaration intentionally belongs to that namespace
```

Therefore the following identity must hold for authored canonical implementation declarations:

```text
implementation namespace
        ⇕
@internal assertion
        ⇕
internal semantic visibility
```

For native implementation selectors there is a fourth agreement:

```text
Rust NativeVisibility::Internal
```

Bootstrap and compilation must detect disagreement rather than silently choosing one representation.

---

## 3. Syntax

`@internal` takes no arguments.

```phalcom
@internal
_$byteCount -> Int
```

```phalcom
@internal
_$slice(_ start: Int, _ end: Int) -> String
```

```phalcom
@internal
let __handle: Int
```

Class-side implementation members combine `@internal` with `@class`:

```phalcom
@class
@internal
@native
_$allocate(_ size: Int) -> Object
```

Attribute order has no semantic significance unless a more general attribute-composition rule explicitly says otherwise.

The following is invalid:

```phalcom
@internal(reason)
_$byteCount -> Int
```

Diagnostic:

```text
attr.arguments_not_allowed
```

---

## 4. Legal Targets

`@internal` is legal on declarations that can belong to an implementation namespace.

| Target | Legal | Requirement |
|---|---:|---|
| Method | yes | selector name begins with `_$` |
| Getter | yes | selector name begins with `_$` |
| Setter | yes | selector name begins with `_$` |
| Index member | no initially | no implementation index namespace is defined |
| Instance field | yes | field name begins with `__` |
| Class-side field | yes | field name begins with `__` |
| Class declaration | no | class internality is not defined by this attribute |
| Variant declaration | no | variants do not use the implementation-selector/storage namespace |

The compiler must reject `@internal` on an ordinary public selector:

```phalcom
@internal
size -> Int
```

Diagnostic:

```text
attr.internal_requires_implementation_namespace
```

The compiler must also reject it on an ordinary source field:

```phalcom
@internal
let _size: Int
```

because `_size` is source-owned field storage, not implementation-owned `__size`.

---

## 5. Privilege

Implementation namespaces are privileged independently of `@internal`.

Ordinary application modules must not gain implementation authority by writing the attribute:

```phalcom
class UserCode {
  @internal
  _$stealRuntimeState -> Object
}
```

This must fail even though the declaration is syntactically marked `@internal`.

The authority check is based on the compiler's trusted module identity, not on module spelling. A user-created package or module named `core`, `universe`, or any similar name must not acquire implementation access.

Expected diagnostic:

```text
member.implementation_namespace_requires_privileged_core
```

The canonical bootstrapped universe is privileged.

Compiler-synthesized implementation selectors are also privileged but are not required to carry source attributes because they are not authored source declarations. For example, a compiler-generated `_$matchArm` may exist without a synthetic `@internal` attribute.

---

## 6. Requirement on Canonical Universe Source

For source-authored declarations in the canonical universe, implementation namespace declarations must explicitly carry `@internal`.

Thus this is invalid in canonical universe source:

```phalcom
_$byteCount -> Int
```

even though the namespace itself already makes the member internal.

The purpose of the error is not security. It is source integrity: canonical universe source must visibly distinguish implementation protocol from user-facing protocol.

Recommended diagnostic:

```text
attr.internal_required
```

For compiler-synthesized implementation members, this rule does not apply.

---

## 7. Visibility Semantics

`@internal` is not equivalent to `@private` or `@protected`.

The visibility categories are distinct:

```text
public
private
protected
internal
```

`@private` expresses lexical access scoped to the declaring class.

`@protected` expresses access from the declaring class and eligible subclasses.

`@internal` asserts membership in the language/runtime implementation namespace.

The following combinations are invalid:

```phalcom
@internal
@private
_$foo
```

```phalcom
@internal
@protected
_$foo
```

Expected diagnostic:

```text
member.visibility_conflict
```

Implementation selectors always use internal visibility. They are not private methods with an unusual spelling.

---

## 8. Interaction with `@native`

A native implementation selector uses both attributes:

```phalcom
@internal
@native
_$byteAt(_ index: Int) -> Option<Int>
```

These attributes assert different facts.

`@internal` asserts:

```text
the source declaration belongs to the implementation namespace
```

`@native` asserts:

```text
the live implementation is supplied by a native primitive rather than this
source declaration
```

Bootstrap must verify all of the following:

```text
source selector begins with "_$"
source has @internal
source has @native
native descriptor visibility == Internal
native descriptor selector == source selector
native descriptor owner/side == source owner/side
```

A mismatch is a bootstrap/source-integrity failure.

---

## 9. Interaction with `@class`

`@class` changes the dispatch/storage side; `@internal` changes neither side nor namespace identity.

```phalcom
@class
@internal
@native
_$allocate(_ size: Int) -> Object
```

means:

```text
owner       = enclosing class
side        = class
selector    = _$allocate(_)
visibility  = internal
implementation = native
```

The bootstrap `PrimitiveKey` must agree with that identity.

---

## 10. Interaction with Types

`@internal` does not weaken typing requirements.

An implementation declaration may carry ordinary parameter and return annotations:

```phalcom
@internal
@native
_$slice(_ start: Int, _ end: Int) -> String
```

For native declarations, those annotations are cross-checked against native primitive metadata.

For real source-defined implementation helpers, annotations participate in ordinary static semantic analysis exactly as they would on a public method.

Internality must never make types part of selector identity or dispatch.

---

## 11. Reflection and LSP

The LSP and reflection presentation layer should expose internality as semantic metadata.

A semantic member representation should report at least:

```text
selector
owner
dispatch side
visibility = internal
source location
native/source implementation status
typing metadata
```

Normal user completion should generally hide internal members unless an explicit internal/debug view is requested.

Go-to-definition from privileged universe source may navigate to implementation declarations.

Documentation generators may include internal members in implementation/reference documentation while excluding them from ordinary public API documentation.

`@internal` itself should not be represented as Phaldoc text. It is machine-readable declaration metadata.

---

## 12. Fields

Implementation fields use the `__` namespace:

```phalcom
@internal
let __handle: Int
```

They must not be confused with implementation selectors.

Correct classification:

```text
_$foo   selector namespace
__foo   field/storage namespace
```

Native code knowing about a source field does not automatically make that field an implementation field. A field such as `_message` remains source-owned unless the language design explicitly moves it to `__message`.

---

## 13. Compilation Ordering

For `@native` members, `@internal` must be validated before the native declaration is removed from executable lowering.

Required order:

```text
parse
  ↓
attribute attachment
  ↓
implementation-namespace validation
  ↓
@internal assertion validation
  ↓
native-anchor verification, if @native
  ↓
drop native member from executable AST
  ↓
ordinary attribute expansion
  ↓
bytecode lowering
```

---

## 14. Diagnostics

Implementations should provide stable diagnostics equivalent to:

| Diagnostic | Meaning |
|---|---|
| `attr.arguments_not_allowed` | `@internal` received arguments |
| `attr.illegal_target` | target category cannot carry `@internal` |
| `attr.internal_requires_implementation_namespace` | attribute is present on an ordinary selector/field |
| `attr.internal_required` | authored canonical implementation declaration lacks `@internal` |
| `member.implementation_namespace_requires_privileged_core` | ordinary source attempted to declare implementation namespace |
| `member.visibility_conflict` | `@internal` combined with private/protected visibility |
| `native.visibility_mismatch` | native descriptor visibility disagrees with source/internal namespace |

Exact diagnostic spelling may follow the compiler's centralized diagnostic conventions, but these cases are normative.

---

## 15. Examples

### Internal declaration-only native primitive

```phalcom
class String {
  @internal
  @native
  _$byteCount -> Int
}
```

### Internal native primitive with a reference body

```phalcom
class String {
  @internal
  @native
  _$byteAt(_ index: Int) -> Option<Int> {
    // Conceptual source. Runtime implementation reads UTF-8 storage directly.
  }
}
```

The body is reference source because `@native` is present; it is not executed.

### Real internal source helper

```phalcom
class Parser {
  @internal
  _$advance -> Token {
    let token = _tokens[_index]
    _index = _index + 1
    token
  }
}
```

This is real executable Phalcom because it is not `@native`.

---

## 16. Non-Goals

`@internal` does not:

- create a new selector namespace;
- replace the `_$` / `__` naming rules;
- grant privileged access;
- imply native implementation;
- imply private or protected visibility;
- affect selector identity;
- affect method dispatch;
- make internal members part of the ordinary public API.

---

## 17. Required Conformance Tests

The implementation must test at least:

1. `@internal` parses as a built-in attribute.
2. Arguments are rejected.
3. `@internal` on `_$` method/getter/setter is accepted in privileged universe source.
4. `@internal` on `__` field is accepted in privileged universe source.
5. `@internal` on ordinary method is rejected.
6. `@internal` on `_sourceField` is rejected.
7. Authored canonical `_$` declaration without `@internal` is rejected.
8. Authored canonical `__` declaration without `@internal` is rejected.
9. User module cannot gain implementation authority by adding `@internal`.
10. `@private` / `@protected` combinations are rejected.
11. `@internal @native` is validated before native-member removal.
12. LSP classifies `__field` as implementation storage.
13. LSP reports `_$selector` visibility as internal.
14. Native visibility mismatch fails bootstrap verification.

---

## 18. Repository Integration

Primary implementation points:

```text
phalcom-ast/src/ast.rs
  Add BuiltinAttr::Internal.

phalcom-core/src/compiler/attributes.rs
  Register and validate @internal.
  Validate source-authored implementation declarations before @native removal.

phalcom-core/src/compiler/lib/*
  Reuse privileged-core authority checks for implementation namespaces.

phalcom-lsp/src/semantic/surface.rs
  Recognize "__" fields as implementation fields.
  Preserve internal visibility in semantic surfaces.

phalcom-native-macros / shared primitive declaration validator
  Require visibility=internal for "_$" native selectors.
```

The defining rule is:

> `@internal` is an explicit source assertion over an independently defined implementation namespace; it is not an access-granting modifier.
