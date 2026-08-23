# `#[primitive(...)]` — Native Primitive Declaration Attribute

**Status:** Partially implemented and in active migration
**Applies to:** Rust functions implementing Phalcom native primitives
**Surface spelling:** Rust `#[phalcom_native_macros::primitive(...)]`; abbreviated here as `#[primitive(...)]`
**Not a Phalcom source attribute:** Phalcom source uses `@native`, not `@primitive`

---

## 1. Purpose

`#[primitive(...)]` is the machine-authoritative declaration of a Rust function as a Phalcom native primitive.

It binds a Rust implementation to a Phalcom semantic primitive description without making the Rust function name part of language identity.

The attribute is the machine source of truth for:

- owner class;
- dispatch side;
- canonical selector;
- parameter tuple shape;
- parameter types;
- return type;
- callable type;
- native visibility;
- effects;
- declared raises;
- return/control-flow behavior;
- primitive ABI;
- intrinsic identity;
- trust classification;
- lifecycle/stability metadata;
- native source provenance;
- source-anchor policy.

Its output drives:

```text
runtime primitive installation
VM-free native semantic surface generation
bootstrap source/native verification
LSP native metadata
runtime typing metadata
reflection metadata
native implementation navigation
```

---

## 2. Primitive Identity

A primitive's language identity is:

```text
PrimitiveKey = (UniverseKey owner, NativeDispatch side, canonical selector)
```

The Rust function name is implementation provenance only.

Renaming:

```rust
fn string_add(...)
```

to:

```rust
fn string_concatenate_primitive(...)
```

must have no Phalcom-visible effect if the attribute declaration is unchanged.

Types likewise do not participate in selector identity.

---

## 3. Canonical Syntax

Representative declaration:

```rust
#[phalcom_native_macros::primitive(
    String,
    "+(_)",

    params = [String],
    returns = String,
    types = "(String) -> String",

    raises = [],
    effects = pure,

    side = instance,
    visibility = public,
    stability = stable,

    abi = value,
    flow = value,
    trust = ordinary,

    anchor = required,
)]
pub fn string_add(
    vm: &mut VM,
    receiver: &Value,
    args: &[Value],
) -> PhResult<Value> {
    // ...
}
```

Only owner and selector should be positional. Semantic options are named.

---

## 4. Fields

Target descriptor surface:

| Field | Meaning | Default |
|---|---|---|
| owner | canonical `UniverseKey` | required |
| selector | canonical exact selector | required |
| `params` | Phalcom argument tuple metadata | required after migration |
| `returns` | return type | required after migration |
| `types` | complete callable type | required after migration |
| `side` | instance/class dispatch | `instance` |
| `visibility` | public/internal | `public`, except `_$` forces internal |
| `raises` | language-level raised errors | unknown if omitted |
| `effects` | coarse semantic effects | unknown if omitted |
| `stability` | lifecycle stability | unspecified |
| `abi` | Rust primitive calling convention | `value` |
| `flow` | result provenance/control flow | `value` |
| `intrinsic` | compiler-recognized intrinsic identity | absent |
| `trust` | native audit class | `ordinary` |
| `since` | lifecycle version | absent |
| `deprecated_since` | deprecation version | absent |
| `replacement` | replacement selector | absent |
| `anchor` | canonical `.ph` anchor requirement | `required` |

A compatibility window may temporarily permit incomplete type metadata, but the completed primitive migration should require the full semantic contract for every language-visible primitive.

---

## 5. Parameter Metadata

`params` represents the Phalcom argument tuple, excluding the receiver.

Example:

```rust
params = [String]
```

for:

```text
+(_)
```

A labeled selector must preserve label spelling and ordering.

The attribute parser must cross-check:

- positional count;
- labeled count;
- labels;
- label order;
- rest lanes;
- duplicate labels;
- selector arity.

The normalized descriptor must contain rest-layout information so downstream consumers do not need to parse selector text merely to rediscover argument structure.

---

## 6. Type Metadata

Native type metadata is symbolic and VM-free.

It must not instantiate runtime Phalcom `Type` objects during Rust compilation or primitive bootstrap.

The symbolic model should support the same relevant structural vocabulary used by Phalcom typing, including:

```text
Name
Self
Never
Unknown
Origin<A>
Origin<A, B>
A | B
tuple/product forms
callable forms
generic parameters where supported
```

Applied types use the ordinary generic model:

```text
Option<Int>
Result<T, E>
List<String>
```

Primitive-specific type constructors are forbidden.

---

## 7. Selector Validation

The selector string must be canonical and structurally valid.

Examples:

```rust
"+(_)"
"hash"
"match(some,none)"
"_$byteAt(_)"
```

The macro must validate selectors with the same canonical selector model used by runtime installation.

A malformed or non-canonical selector must fail Rust compilation rather than being silently repaired at runtime.

---

## 8. Dispatch Side

`side` is either:

```rust
side = instance
```

or:

```rust
side = class
```

It corresponds to Phalcom source placement:

```phalcom
@native
foo(...)
```

versus:

```phalcom
@class
@native
foo(...)
```

Dispatch side is part of `PrimitiveKey` and must be cross-checked during source/native bootstrap verification.

---

## 9. Visibility

Native visibility is:

```rust
visibility = public
```

or:

```rust
visibility = internal
```

Implementation selectors beginning with `_$` must always be internal.

Therefore this is invalid:

```rust
#[primitive(
    String,
    "_$byteAt(_)",
    visibility = public,
    ...
)]
```

The target invariant is:

```text
selector begins "_$"
    ⇔
NativeVisibility::Internal
```

for language implementation selectors.

Native internal visibility is distinct from Phalcom `@private` and `@protected`.

---

## 10. Effects

`effects` describes coarse semantic effects useful to tooling, static analysis, contracts, optimization, and documentation.

Expected categories include:

```text
pure
mutation
io
scheduling
reflection
nondeterminism
blocking
```

Omission means unknown, not pure.

A positive `pure` declaration is a machine claim and must not be inferred merely from a selector name.

Effects never participate in selector dispatch.

---

## 11. Raises

`raises` records known language-level error behavior.

Example:

```rust
raises = []
```

means known not to raise a language-level error under its declared contract.

Omission means:

```text
unknown
```

not:

```text
none
```

The distinction must survive code generation and semantic presentation.

Host panics and process-level failures are not automatically language-level raises declarations.

---

## 12. Return Flow

`flow` records result/control-flow provenance.

Expected forms include:

```text
value
receiver
argument(INDEX)
never
unknown
```

Examples:

- an identity operation may return `receiver`;
- a forwarding primitive may return an argument;
- an always-raising primitive may be `never`.

Flow metadata is useful to semantic tooling and optimization but does not affect selector dispatch.

---

## 13. ABI

`abi` identifies the native invocation convention.

At minimum:

```text
value
shape
```

A value primitive follows the ordinary receiver-plus-values ABI.

A shape primitive receives structured argument-shape information and may implement rest/forwarding gateways.

The proc macro must validate the Rust function signature against the declared ABI at compile time.

---

## 14. Intrinsics

`intrinsic` identifies primitives explicitly recognized by compiler optimization.

It must never be inferred from selector spelling.

Example:

```rust
intrinsic = BoolAnd
```

means the compiler may maintain a specialized optimization path for that semantic operation.

Selector identity and intrinsic identity are separate metadata dimensions.

---

## 15. Trust

`trust` distinguishes ordinary native implementation from privileged operations that deserve stronger audit treatment.

Typical values:

```text
ordinary
privileged
```

Trust is implementation/audit metadata. It does not grant source-level visibility.

---

## 16. Stability and Lifecycle

Machine-readable lifecycle metadata may include:

```text
stability
since
deprecated_since
replacement
```

These facts belong in structured metadata and should not be duplicated as Phaldoc tags.

The attribute parser must validate invalid lifecycle combinations.

---

## 17. Source Anchor Policy

Every language-visible primitive should normally have a canonical Phalcom `@native` source declaration.

Default:

```rust
anchor = required
```

A VM-private primitive may explicitly use:

```rust
anchor = hidden
```

`hidden` means the primitive intentionally has no canonical user-facing universe declaration.

The bootstrap verifier computes:

```text
required descriptor keys
        ==
source @native keys
```

A separately maintained exemption list should be avoided.

---

## 18. Descriptor Generation

The proc macro emits a static runtime descriptor conceptually equivalent to:

```rust
PrimitiveDescriptor {
    surface: &'static PrimitiveSurfaceSpec,
    abi,
    entry,
    source: NativeSourceSpec,
}
```

`NativeSourceSpec` should contain:

```text
Rust module path
Rust function name
Rust file path
Rust source line
```

This provenance supports navigation and diagnostics but is not language identity.

Descriptors are collected through the distributed primitive registry.

---

## 19. Runtime Installation

Primitive installation must be descriptor-driven.

Target path:

```text
PRIMITIVES
   ↓ sort by PrimitiveKey
registry uniqueness validation
   ↓
resolve UniverseKey -> ClassId
   ↓
select instance/class target
   ↓
construct MethodObject
   ↓
apply visibility/access metadata
   ↓
install
```

Legacy manual registration tables must be removed after migration.

A primitive must not require both:

```text
#[primitive(...)]
```

and:

```text
primitive!(...)
```

as permanent sources of installation truth.

---

## 20. Generation of the VM-Free Native Surface

The VM-free native surface used by LSP/tooling must be derived from `#[primitive(...)]`, not maintained separately by hand.

Recommended architecture:

```text
                 shared primitive declaration parser
                    /                     \
                   /                       \
Rust proc macro parser                 build-time scanner
       |                                      |
runtime PrimitiveDescriptor         VM-free PrimitiveSurfaceSpec
```

The primitive attribute grammar and validation should be extracted from the proc-macro crate into a shared VM-free crate so both consumers use exactly the same schema implementation.

A build script may scan Rust primitive source and emit generated Rust metadata into `OUT_DIR`.

The generator must not contain a second hand-coded interpretation of the attribute grammar.

A parity test must compare the generated VM-free surface with the runtime descriptor surface.

---

## 21. Source/Descriptor Verification

During universe bootstrap, every required descriptor is matched to a source `@native` declaration.

The verifier checks:

```text
owner
side
selector
member kind
parameter structure
parameter types
return type
visibility
```

It does not compare the Rust function name to source.

It does not attempt to prove a reference body semantically equivalent to Rust.

Descriptor-only metadata such as effects and ABI is merged into the semantic member without being duplicated into source.

---

## 22. Runtime Typing Integration

After installing a primitive, runtime must associate the resulting `MethodObject` with semantic callable metadata through the existing method-semantic side table.

Conceptually:

```text
PrimitiveSurfaceSpec
      +
verified source declaration
      ↓
callable metadata record
      ↓
MethodSemanticIndex[MethodObject]
```

This keeps `MethodObject` compact while allowing reflection to inspect static typing metadata.

---

## 23. Diagnostics

The proc macro and shared validator should report stable errors for:

```text
invalid selector
non-canonical selector
parameter/selector arity mismatch
parameter label mismatch
duplicate labels
invalid type metadata
callable type inconsistency
invalid ABI/function signature
_$ selector declared public
invalid effect declaration
invalid flow declaration
invalid lifecycle combination
invalid anchor policy
unknown UniverseKey
duplicate PrimitiveKey
```

Bootstrap separately reports source/descriptor mismatches.

---

## 24. Example: Public Primitive

```rust
#[phalcom_native_macros::primitive(
    String,
    "+(_)",
    params = [String],
    returns = String,
    types = "(String) -> String",
    raises = [],
    effects = pure,
    side = instance,
    visibility = public,
    stability = stable,
    abi = value,
    flow = value,
    trust = ordinary,
    anchor = required,
)]
pub fn string_add(
    vm: &mut VM,
    receiver: &Value,
    args: &[Value],
) -> PhResult<Value> {
    // native implementation
}
```

Canonical source:

```phalcom
@native
class String {
  @native
  +(_ other: String) -> String
}
```

---

## 25. Example: Internal Primitive

```rust
#[phalcom_native_macros::primitive(
    String,
    "_$byteAt(_)",
    params = [Int],
    returns = Option<Int>,
    types = "(Int) -> Option<Int>",
    effects = pure,
    side = instance,
    visibility = internal,
    abi = value,
    flow = value,
    trust = ordinary,
    anchor = required,
)]
pub fn string_raw_byte_at(
    vm: &mut VM,
    receiver: &Value,
    args: &[Value],
) -> PhResult<Value> {
    // native representation access
}
```

Canonical source:

```phalcom
@internal
@native
_$byteAt(_ index: Int) -> Option<Int>
```

---

## 26. Non-Goals

`#[primitive(...)]` does not:

- add a Phalcom source declaration automatically at runtime;
- make Rust function names part of Phalcom identity;
- introduce type-based dispatch;
- require runtime allocation of type objects;
- turn every core function into native code;
- replace real `.ph` wrappers;
- replace Phaldoc;
- permit multiple independent handwritten native-surface registries.

---

## 27. Migration Requirements

The repository remains in a mixed state until every language primitive has a descriptor.

Migration must proceed by primitive module and record for every legacy registration:

```text
owner
side
selector
kind
Rust function
descriptor present?
types complete?
visibility
anchor status
```

After all language primitives are descriptor-backed:

1. remove their legacy `primitive!` / `primitive_static!` registrations;
2. remove `Universe::install_primitives` as a language primitive registry;
3. leave descriptor-driven installation as the sole native installation path;
4. generate VM-free native metadata from `#[primitive(...)]`;
5. enforce source-anchor completeness.

---

## 28. Conformance Tests

At minimum:

1. macro emits a descriptor for a valid value primitive;
2. macro emits a descriptor for a valid shape primitive;
3. invalid Rust ABI is rejected;
4. invalid selector is rejected;
5. selector/params mismatch is rejected;
6. label mismatch is rejected;
7. callable type mismatch is rejected;
8. `_$` plus public visibility is rejected;
9. duplicate primitive keys are rejected;
10. generated VM-free surface equals runtime descriptor surface;
11. required source anchor is enforced;
12. hidden primitive may omit source anchor;
13. source/native type mismatch fails bootstrap;
14. runtime installer uses descriptor identity only;
15. Rust function rename does not change Phalcom identity;
16. runtime typing side table is populated for installed primitives.

---

## 29. Repository Integration

Primary implementation points:

```text
phalcom-native-macros/src/lib.rs
  Procedural macro entry point.

shared native-declaration crate
  Parse and validate #[primitive(...)] syntax for both macro and generator.

phalcom-native-meta/src/primitive.rs
  PrimitiveKey, PrimitiveSurfaceSpec, effects, flow, visibility, lifecycle,
  and anchor policy.

phalcom-core/src/native/descriptor.rs
  Runtime callable descriptor.

phalcom-core/src/native/registry.rs
  Distributed registry and duplicate-key validation.

phalcom-core/src/native/install.rs
  Deterministic descriptor-driven installation.

phalcom-native-surface/build.rs
  Generate VM-free metadata from #[primitive(...)] declarations.

phalcom-core/src/primitive/*.rs
  Migrate every native language primitive to #[primitive(...)].
```

The defining requirement is:

> `#[primitive(...)]` is the machine-authoritative declaration of a native Phalcom primitive; all other machine surfaces are derived from it or verified against it.
