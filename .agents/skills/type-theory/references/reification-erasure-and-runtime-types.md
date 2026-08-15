# Reification, Erasure, Runtime Type Metadata, and Reflection

## Purpose

For every type feature, separate four questions:

1. Does the static checker know it?
2. Is normalized metadata stored in compiled/runtime descriptors?
3. Can Phalcom reflection observe it?
4. Does ordinary runtime execution use it for checking, specialization, or dispatch?

These axes are independent.

## 1. Erasure

An erased type influences compile/check time but is absent from ordinary runtime representation unless passed explicitly.

Classic consequences:

- runtime cannot inspect generic argument `T` directly;
- `List<Int>` and `List<String>` instances may share same runtime class/layout;
- casts involving erased arguments require other metadata or unchecked operations.

Erasure reduces metadata but constrains reflection and typed-runner checks.

## 2. Reification

Reified type metadata preserves structure such as:

```text
origin: List
arguments: [Int]
```

Benefits:

- reflection;
- runtime contract checking;
- serialization/schema tooling;
- diagnostics;
- possible specialization guidance.

Costs:

- descriptor allocation/canonicalization;
- GC rooting/lifetime;
- metadata validation/security;
- bootstrap ordering;
- package/ABI compatibility.

## 3. Reification is not runtime class specialization

A reified descriptor:

```text
List<Int>
```

can coexist with runtime instances whose class remains origin `List`.

Separate:

```text
semantic TypeId(List<Int>)
reflected AppliedType object
runtime ClassId(instance) = List
code specialization maybe none
```

This separation preserves Phalcom's doctrine that ordinary selector dispatch and allocation are not implicitly changed by type metadata.

## 4. Canonical descriptor identity

If semantic type applications are canonicalized, repeated construction should map to one canonical semantic identity:

```text
canonical_apply(List,[Int]) -> same TypeId
```

Whether reflection returns the same *runtime object identity* is an additional promise. A runtime can expose a canonical descriptor object or several equal wrappers around one semantic type; Phalcom's design can choose.

Do not infer reflective `===` semantics from internal interning without a normative statement.

## 5. Source annotation versus normalized metadata

Source:

```text
value: T
```

inside `Box<T>` has several forms:

```text
source syntax/spelling: "T"
resolved semantic annotation: TypeParam(Box,0)
applied view at Box<Int>: Int
```

Reflection/documentation may need all of them.

Do not overwrite source annotation with applied result and lose authorship information.

## 6. Metadata trust boundary

Compiled metadata may be malformed, forged, stale, or produced by older packages.

Validate:

- referenced descriptor IDs exist;
- descriptor kind is recognized/trusted where authority is required;
- type-argument arity matches origin signature;
- binder owners/indexes are valid;
- generic restrictions are satisfied if metadata claims normalized application;
- recursive graphs obey legal formation rules;
- trusted/native flags cannot be forged;
- selector/type metadata cannot smuggle executable behavior into trusted normalization.

A checker must not segfault/panic on malformed metadata.

## 7. Purity of type normalization

Type normalization/canonicalization should not invoke arbitrary user code. Otherwise merely reading an annotation could:

- mutate state;
- yield;
- throw unpredictably;
- install methods;
- create nondeterministic type identity.

Phalcom's proposed typing design favors recognized immutable descriptor kinds for authoritative metadata. This is a sound security/semantic boundary.

## 8. Runtime validation

A typed runner may use reified metadata:

```text
parameter contract T
actual runtime value v
check_runtime(v,T)
```

But runtime validation semantics need a relation distinct from static subtyping.

Questions:

- Does `Dynamic` skip check?
- How are protocols checked at runtime?
- Are generic arguments erased from instances, requiring deep/container checks?
- Are checks shallow class tests or contracts at operations?
- What happens for callables/higher-order contracts?
- What is blame/error provenance?

Do not equate `runtime_is_instance(v,T)` with `A <: T`.

## 9. Runtime protocol checks

Structural protocol conformance can be checked against class/member metadata rather than probing user code dynamically.

If class surfaces are mutable, a runtime conformance cache needs invalidation like static conformance cache.

A runtime "respondsTo" probe is not necessarily same as static protocol contract because annotations, visibility, callable variance, and effects may matter.

## 10. FFI bridge

Rust's type system/runtime does not map directly to Phalcom `TypeId`.

Rust generics are commonly monomorphized; trait objects use vtables; `std::any::TypeId` identifies Rust `'static` concrete types under Rust-specific semantics.

Never transmute or equate:

```text
Phalcom TypeId == Rust TypeId
```

A mixed package needs explicit adapter metadata:

```text
Phalcom type contract
Rust ABI/concrete type
conversion/check function
ownership/lifetime policy
error mapping
```

## 11. Reified generic reflection and parametricity

If generic code can inspect:

```text
T === Int
```

then it is not fully parametric in the classic System F sense. This affects free-theorem proofs and optimization assumptions.

Reification is an observable capability; type theory must include it when making uniformity claims.

## 12. Serialization/versioning

Persisted type metadata should not rely on ephemeral arena indexes alone.

Across modules/packages/builds, need stable references such as:

```text
package/module identity + declaration identity/versioned symbol
```

or compiler-managed relocation tables.

This is implementation architecture, but semantic identity rules determine what must survive serialization.

## 13. GC and weak canonical caches

Runtime reflected descriptors may reference:

- origin classes/protocols;
- type arguments;
- owner declarations;
- source metadata.

A strong global interner can retain every type application ever constructed. Consider:

- bounded caches;
- weak references for runtime wrappers;
- canonical semantic IDs owned by module/compiler lifetime;
- explicit generation cleanup.

Memory policy must preserve promised identity semantics.

## 14. Dispatch non-interference

Even with full reification:

```text
receiver.class = List
static type = List<Int>
```

ordinary send cache can remain keyed by runtime class + selector, not static `TypeId`.

If an optimizer specializes based on `List<Int>`, it must guard assumptions and deopt/invalidate correctly under actual runtime semantics. Static metadata alone is not a dispatch key.

## 15. Security implications

If untrusted package can construct fake authoritative type descriptors, it may:

- bypass typed-runner validation;
- forge protocol conformance;
- confuse reflection/security annotations;
- trigger unsafe native conversions.

Hence trusted descriptor construction/validation is part of typing soundness at runtime.

## 16. Testing obligations

- repeated canonical application behavior;
- source annotation remains distinguishable from substituted view;
- malformed arity/owner metadata rejected;
- forged trusted descriptor rejected;
- runtime class remains origin under reified generic metadata if that is normative;
- ordinary selector dispatch unaffected by annotations;
- typed-runner violation points to correct contract;
- FFI adapter rejects incompatible runtime values;
- cache/GC stress does not break promised descriptor identity.

## 17. Failure modes

- Reified generic type automatically becomes runtime subclass.
- Runtime `ClassId` used as complete semantic type ID.
- Type normalizer invokes arbitrary user methods.
- Internal interning accidentally exposed as unratified `===` semantics.
- Source annotation overwritten by normalized/applied view.
- Rust `TypeId` treated as Phalcom type identity.
- Strong unbounded descriptor cache leaks forever.

## 18. Competency questions

1. List the four independent questions for every type feature: static knowledge, metadata retention, reflection, runtime use.
2. Why can `List<Int>` be reified without changing instance runtime class?
3. What metadata must be validated at a package/native boundary?
4. Why does type reflection weaken classic parametricity assumptions?
5. What is the difference between internal canonical `TypeId` and reflected runtime descriptor identity?
