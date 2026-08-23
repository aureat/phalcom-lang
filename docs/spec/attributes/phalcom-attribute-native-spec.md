# `@native` — Native-Backed Source Declaration Attribute

**Status:** Implemented in a limited member-only form; this document specifies the target semantics
**Applies to:** Canonical Phalcom universe source and other explicitly privileged native-source environments
**Primary implementation areas:** parser/AST, compiler attributes, native-source verifier, bootstrap, LSP
**Related attributes:** `@internal`, `@class`
**Native counterpart:** Rust `#[primitive(...)]`

---

## 1. Purpose

`@native` declares that the live implementation or runtime identity represented by a Phalcom source declaration is provided externally by the native runtime.

It exists so native-backed language entities remain present in canonical Phalcom source for API definition, type annotations, Phaldoc, source navigation, reflection presentation, source/native consistency checking, optional reference implementations, and readable definition of the core language universe.

`@native` does not install a Rust function. It asserts that a corresponding native definition already exists and must be verified.

The canonical relation is:

```text
Phalcom @native declaration
            ⇕ verified against
Rust #[primitive(...)] descriptor
            ↓
live runtime primitive
```

The source declaration and Rust descriptor serve different purposes and must not become unverified competing authorities.

---

## 2. Native Member Forms

A native member may be written either as a declaration or with a reference body.

### 2.1 Declaration-only native member

```phalcom
@native
+(_ other: Number) -> Number
```

There are no braces.

This is the preferred form for irreducible primitive operations where a Phalcom implementation would be misleading, redundant, or impossible.

Typical declaration-only primitives include raw storage access, allocation, runtime dispatch hooks, primitive numeric arithmetic, scheduler/fiber transitions, VM-owned reflection gateways, raw I/O, and operations whose semantics depend on a native heap representation.

### 2.2 Native member with reference body

```phalcom
@native
toString -> String {
  "[" + joined(", ") + "]"
}
```

The body is parsed and retained for source tooling, but is not the live implementation and emits no executable method.

A reference body is appropriate when a truthful Phalcom formulation materially improves understanding. It is optional.

---

## 3. Real Source Wrappers Are Not `@native`

A Phalcom method that actually executes must not carry `@native`, even if it calls native implementation primitives.

Preferred architecture:

```phalcom
class String {
  @internal
  @native
  _$byteCount -> Int

  size -> Int {
    _$byteCount
  }

  isEmpty -> Bool {
    size == 0
  }
}
```

Here:

```text
_$byteCount  native implementation floor
size         real Phalcom wrapper
isEmpty      real Phalcom implementation
```

This distinction preserves the small native floor and keeps derivable behavior in the language itself.

---

## 4. Native Classes

The target specification permits `@native` on selected canonical core classes.

```phalcom
@native
class String {
  ...
}
```

Class-level `@native` means:

> The canonical runtime class identity and native representation already exist before this source declaration is compiled. The source declaration completes and presents that existing class; it does not allocate a second class identity.

This is distinct from member-level `@native`:

```text
@native class
    native runtime owns primordial identity/representation

@native member
    native runtime owns callable implementation
```

Class-level `@native` is initially restricted to privileged universe/core source.

The compiler must resolve the declaration to the canonical bootstrapped class identity. It must not create a fresh class merely because the source parser encountered a class declaration.

This mechanism is especially important for primordial runtime classes and special bindings whose source name may not map trivially to the class object.

---

## 5. Grammar

Target forms:

```text
native-member :=
    "@native" attributes? member-signature member-body?

member-body :=
    "{" statements "}"

native-class :=
    "@native" attributes? class-declaration
```

`@native` takes no arguments.

Invalid:

```phalcom
@native("string_add")
+(_ other: String) -> String
```

The Rust function is never selected by a source argument. Native identity is structural.

---

## 6. Native Identity

A native member is matched to a Rust primitive using:

```text
PrimitiveKey = (owner, dispatch side, canonical selector)
```

Type metadata never participates in selector identity or ordinary dispatch.

Example:

```phalcom
class String {
  @native
  +(_ other: String) -> String
}
```

maps to:

```text
owner    = String
side     = instance
selector = +(_)
```

and:

```phalcom
class Some {
  @class
  @native
  call(_ value: Object) -> Option
}
```

maps to:

```text
owner    = Some
side     = class
selector = call(_)
```

The source does not name the Rust implementation function.

---

## 7. Legal Targets

| Target | Legal |
|---|---:|
| Class | yes, for privileged native-backed classes |
| Method | yes |
| Getter | yes |
| Setter | yes |
| Constructor/factory method | yes when represented through ordinary method/constructor semantics |
| Field | no as a native callable anchor |
| Variant | no |
| Index member | only after native index identity is specified consistently |

Class support is a target-state extension over the current member-only implementation.

A native-field mechanism, if ever required, should be specified separately rather than overloading callable `@native` semantics.

---

## 8. Compilation Semantics

For a native member:

1. The declaration is lexed and parsed normally.
2. Attributes are attached normally.
3. Type annotations are parsed normally.
4. A reference body, if present, must be syntactically valid Phalcom.
5. Namespace and attribute legality are checked.
6. Bootstrap/source verification checks the member against the native descriptor.
7. The member is removed from executable lowering.
8. No bytecode method is emitted.
9. No source method is installed into the runtime method table.
10. The native primitive installed from its Rust descriptor remains the live implementation.

The compiler must not treat a declaration-only native member as an empty-body method. An empty executable body and an absent body are distinct AST states.

Recommended representation:

```rust
pub enum MemberBody {
    Declaration,
    Statements(Vec<Statement>),
}
```

The general parser may understand declaration bodies independently of `@native`; the compiler decides whether a declaration-only member is legal in the current semantic context.

---

## 9. Reference Bodies

A native reference body is source documentation with stronger structure than prose.

It must:

- parse as ordinary Phalcom;
- use ordinary language syntax;
- have ordinary source spans;
- be indexable by tooling;
- be available to LSP/documentation presentation;
- never silently become the installed runtime implementation.

It need not initially:

- be proved semantically equivalent to Rust;
- be executable;
- be property-tested against the primitive;
- precisely model native allocation or representation behavior.

A reference body must not contain intentionally false pseudocode merely to fill space. If a truthful representation is not useful, use a declaration-only native member.

---

## 10. Source/Native Verification

Before native-backed universe source is executed, bootstrap must verify native anchors against registered native descriptors.

For every native member, verify:

```text
owner
dispatch side
canonical selector
selector/member kind
parameter structure
parameter type annotations
return type annotation
visibility/internality
```

The descriptor remains authoritative for native-only implementation facts:

```text
Rust function
ABI
effects
raises
return/control flow
intrinsic identity
trust classification
Rust source provenance
lifecycle/stability metadata
```

These facts should not be copied into `.ph` merely to create a second comparison target.

---

## 11. Bijection and Anchor Policy

Eventually every language-visible native primitive must have exactly one canonical source anchor unless the primitive explicitly declares that it is hidden.

Target invariant:

```text
source @native keys
    ==
native descriptors with anchor policy = required
```

A primitive descriptor may explicitly opt out:

```text
anchor = hidden
```

for VM-private primitives that intentionally have no universe declaration.

`required` should be the default.

A separately maintained exemption list should be avoided.

Expected failures:

```text
native.missing_anchor
native.orphan_anchor
native.duplicate_anchor
```

---

## 12. Type Integration

Source and native type syntax are structurally compared.

Example:

```phalcom
@internal
@native
_$byteAt(_ index: Int) -> Option<Int>
```

must agree with native metadata equivalent to:

```rust
params = [Int],
returns = Option<Int>,
types = "(Int) -> Option<Int>",
```

The comparison must normalize AST structures rather than compare source strings.

Conceptual bridge:

```text
TypeAnnotation
      ↓
NormalizedSemanticType
      ↑
TypeExprSpec
```

Type metadata does not alter primitive dispatch.

---

## 13. Interaction with `@internal`

Native implementation selectors must explicitly carry `@internal` in canonical source:

```phalcom
@internal
@native
_$slice(_ start: Int, _ end: Int) -> String
```

Bootstrap verifies:

```text
selector namespace == implementation
@internal present
native descriptor visibility == Internal
```

A native public method does not carry `@internal`:

```phalcom
@native
hash -> Int
```

---

## 14. Interaction with `@class`

Class-side native methods use `@class`:

```phalcom
@class
@native
new(_ value: Object) -> String
```

`@class` supplies the dispatch side used in `PrimitiveKey`.

The native descriptor must specify `side = class`.

A source/native side disagreement is a bootstrap error.

---

## 15. Constructors

Constructors remain ordinary class-side methods under Phalcom constructor semantics.

`@native` does not create a second constructor mechanism.

If a native constructor/factory is exposed as a class-side method, its source declaration uses the same ordinary selector identity and applicable constructor metadata as other constructors.

No native declaration may rely on Rust function overloading or Rust function names as language identity.

---

## 16. Runtime Installation

`@native` is not a binding directive.

Primitive installation comes from the Rust descriptor registry:

```text
#[primitive(...)]
       ↓
PrimitiveDescriptor
       ↓
distributed registry
       ↓
deterministic native installer
       ↓
MethodObject
```

The source anchor never performs that installation.

This separation is essential: bootstrap can compare independent representations rather than using `.ph` as an imperative native-registration script.

---

## 17. Runtime Typing and Reflection

Once a native method is installed, the runtime should associate its `MethodObject` with the verified semantic callable metadata through the existing method-semantic side table.

Desired relation:

```text
source @native declaration
        +
PrimitiveSurfaceSpec
        ↓
verified semantic callable record
        ↓
MethodSemanticIndex
        ↓
live MethodObject
```

The source/reference body need not be stored in `MethodObject`.

Reflection should be able to report the method's types and native status without inflating the method object itself.

---

## 18. LSP Semantics

The LSP must merge source and native metadata rather than choose one.

Wrong model:

```text
source member OR native member
```

Target model:

```text
source @native anchor
        +
generated native descriptor surface
        =
one semantic member
```

Source contributes:

- physical source location;
- declaration span;
- parameter names;
- Phaldoc;
- written type syntax;
- `@internal`, `@class`, and other source attributes;
- optional reference body.

Native metadata contributes:

- implementation status;
- canonical machine type metadata;
- effects;
- raises;
- flow;
- stability;
- intrinsic status;
- Rust implementation provenance.

Go-to-definition should normally land in canonical `.ph` universe source. An implementation-oriented navigation command may additionally expose the Rust primitive source.

---

## 19. Documentation Semantics

Phaldoc belongs to the source declaration:

```phalcom
/// Returns the number of bytes in this string's UTF-8 representation.
@internal
@native
_$byteCount -> Int
```

Machine facts should not be duplicated as Phaldoc tags merely because the primitive is native.

Effects, internal visibility, types, native status, and stability should be rendered from structured metadata.

---

## 20. Diagnostics

Expected conformance diagnostics include:

| Diagnostic | Meaning |
|---|---|
| `attr.arguments_not_allowed` | `@native` received arguments |
| `attr.illegal_target` | unsupported declaration target |
| `native.orphan_anchor` | source anchor has no corresponding descriptor |
| `native.missing_anchor` | required descriptor has no source declaration |
| `native.duplicate_anchor` | more than one source anchor maps to the same primitive key |
| `native.side_mismatch` | source and descriptor dispatch side differ |
| `native.selector_mismatch` | selector identity differs |
| `native.kind_mismatch` | getter/method/setter/index kind differs |
| `native.parameter_type_mismatch` | source and descriptor parameter metadata differ |
| `native.return_type_mismatch` | source and descriptor return metadata differ |
| `native.visibility_mismatch` | source/internal namespace and native visibility differ |
| `native.class_identity_mismatch` | `@native class` cannot resolve to its canonical bootstrapped identity |

---

## 21. Examples

### Declaration-only arithmetic primitive

```phalcom
@native
class Number {
  @native
  +(_ other: Number) -> Number

  @native
  -(_ other: Number) -> Number

  @native
  hash -> Int
}
```

### Native internal floor plus real public wrappers

```phalcom
@native
class String {
  @internal
  @native
  _$byteCount -> Int

  @internal
  @native
  _$byteAt(_ index: Int) -> Option<Int>

  size -> Int {
    _$byteCount
  }

  isEmpty -> Bool {
    size == 0
  }
}
```

### Reference body

```phalcom
class List {
  /// Returns a readable list representation.
  @native
  toString -> String {
    "[" + joined(", ") + "]"
  }
}
```

The body never replaces the native binding.

---

## 22. Non-Goals

`@native` does not:

- select a Rust function by name;
- install a primitive;
- alter selector identity;
- introduce type-based dispatch;
- imply internal visibility;
- imply class-side dispatch;
- require a reference body;
- make the reference body executable;
- make all high-level core behavior native;
- replace real `.ph` wrappers around native floor operations.

---

## 23. Required Conformance Tests

The final implementation should test:

1. declaration-only native method parses;
2. reference-bodied native method parses;
3. declaration-only ordinary method is rejected unless another feature explicitly permits declarations;
4. reference body syntax errors are diagnosed;
5. native member emits no bytecode;
6. native member cannot replace the installed native primitive;
7. class-side native identity is checked;
8. internal native visibility is checked;
9. parameter and return types are structurally compared;
10. source anchor with no descriptor fails;
11. required descriptor with no anchor fails;
12. duplicate anchor fails;
13. hidden descriptor needs no source anchor;
14. native class resolves to primordial class identity;
15. native class does not allocate/rebind a second class;
16. LSP merges native metadata into the source declaration;
17. go-to-definition lands in physical universe source;
18. runtime typing reflection sees native callable metadata.

---

## 24. Repository Integration

Primary areas:

```text
phalcom-ast/src/ast.rs
  Add declaration-vs-statements body representation where required.

phalcom-ast/src/parser.rs
  Parse declaration-only members.

phalcom-core/src/compiler/attributes.rs
  Preserve legality checking and remove native members before executable lowering.
  Extend @native to privileged native-backed classes.

phalcom-core/src/native/source.rs
  Extract source anchors.

phalcom-core/src/native/verify.rs
  Verify complete universe source against descriptors.

phalcom-core/src/vm/bootstrap.rs
  Run native preflight before universe execution.

phalcom-lsp/src/semantic/*
  Merge source anchor with generated native semantic surface.

phalcom-core/src/typing/*
  Associate installed primitive MethodObjects with semantic callable metadata.
```

The defining invariant is:

> `@native` makes native implementation visible in Phalcom source without making the source declaration the executable implementation.
