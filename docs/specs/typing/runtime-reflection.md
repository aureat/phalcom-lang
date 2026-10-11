# Runtime Typing Reflection

**Status:** Normative platform specification

**Semantic owner:** the public runtime typing-reflection API and its authority boundary

**Related specifications:** [Type Algebra reflection laws](type/type.reflection-pattern.md), [type-level domains](../type-level-domains.md), [type members](../type-members.md), [callable reflection](../callables/reflection.md), and [the Universe package](../universe.md)

## 1. Scope and authority

This specification owns the runtime API exposed by `universe.reflection.typing`, including reified type and kind descriptors, metadata-query contexts, capability checks, and explicit query outcomes.

It does not define the canonical type, kind, identity, subtype, conformance, or projection calculus. Those meanings are owned by the language specifications linked above. Runtime reflection projects those meanings; it is not a second semantic authority. The Type Algebra registry states cross-system coherence requirements and must agree with this API contract.

Reflection of a live value's runtime class, a compiler's static type judgment, and a reified type form are distinct facts:

```text
value.class       live runtime class used for dispatch
value : T         static semantic judgment
TypeDescriptor    reified view of a semantic type form
```

An implementation must not infer erased generic arguments from a runtime class, and reflection must not change allocation, dispatch, selector identity, or object identity.

## 2. Universe package and descriptor families

The package is `universe.reflection.typing`. Its exported modules define these public descriptor families:

| Family | Public forms |
|---|---|
| Kinds | `KindDescriptor`, `AtomicKind`, `FunctionKind`, and the `Type` kind value |
| Type forms | `TypeDescriptor`, `AppliedType`, `UnionType`, `TupleType`, `RecordType`, `CallableType`, `TypeLambda`, `SpecialType`, and `SelfType` |
| Generic declarations | `TypeParameter`, `GenericSignature`, and `GenericConstraint` |
| Callable and field metadata | `CallableSignature`, `CallableParameter`, and `FieldSignature` |
| Source type occurrences | `TypeUse` |
| Query outcomes | `TypingResult`, `TypeRelationResult`, and `MemberLookupResult` variants |
| Evidence and authority metadata | `RelationEvidence`, `RelationFailure`, `DynamicBoundary`, and `ReflectionCapability` |
| Query entry point | `Typing` and `TypingContext` |

A nominal type declaration is represented by its existing class object. Forms without a corresponding nominal class may have a synthetic `TypeDescriptor` subclass. Reifying a type form must preserve its canonical declaration and specialization identity; it must not create a new runtime class for a generic application.

Descriptors are observations of canonical semantic handles, not user-constructed numeric IDs. Where the API supports constructing a type form, it must validate the input forms and preserve the kind and domain rules owned by the type system.

## 3. `TypingContext`

`Typing.current` and `TypingContext.new()` acquire a `RuntimePublic` query context. `Typing.contextFor(_)` acquires a context for an explicitly selected profile or capability set and returns a `TypingResult`. A context exposes its profile, capabilities, semantic-model version, metadata snapshot, and runtime-world version. Static metadata queries remain pinned to the context's snapshot; `refresh` creates a view for current metadata. Operations that cross into the live runtime must reconcile the runtime-world version.

A context is an explicit authority boundary. Query methods must check the capability required for the operation. `restrictTo(_)` is attenuating: the result context may retain only capabilities already held by the original context. A request expressed as a symbol or other ordinary source value is not, by itself, authority to inspect private declarations, invoke methods reflectively, inspect proofs, or validate arbitrary runtime values. Granting privileged capabilities remains subject to the runtime's trusted profile or host policy.

The standard profile names have these authority ceilings:

| Profile | Maximum standard capability set |
|---|---|
| `RuntimeMinimal` | No general declaration discovery; only information required by explicitly materialized runtime type forms |
| `RuntimePublic` | `OBSERVE_PUBLIC_TYPES`, `OBSERVE_SIGNATURES`, `CONSTRUCT_TYPE_FORMS`, and bounded `EVALUATE_RELATIONS` |
| `ToolingDebug` | `RuntimePublic` plus `OBSERVE_SOURCE_USES`; implementation-provenance or private information remains policy-controlled |
| `Proof` | `ToolingDebug` plus `INSPECT_PROOFS`, subject to the proof trust policy |

Runtime-value validation and reflective invocation require their own explicit grants. Profile names are not proof that every underlying metadata record is present. The capability vocabulary is:

```text
OBSERVE_PUBLIC_TYPES
OBSERVE_SIGNATURES
CONSTRUCT_TYPE_FORMS
EVALUATE_RELATIONS
OBSERVE_SOURCE_USES
OBSERVE_PRIVATE_TYPES
VALIDATE_RUNTIME_VALUES
INSPECT_PROOFS
INVOKE_REFLECTIVELY
OBSERVE_IMPLEMENTATION_PROVENANCE
```

The API must not expose raw semantic handles or let a descriptor act as proof of a relation. A typing capability does not replace ordinary visibility or invocation authorization.

## 4. Query families

`TypingContext` groups operations by the semantic information they consume:

| Operation family | Entry points |
|---|---|
| Declaration and generic metadata | `typeOfDeclaration(_)`, `genericSignatureOf(_)`, `declaredSupertypeOf(_)`, `signatureOf(_)` |
| Type-form construction | `apply(_, _)`, `unionOf(_)`, `tupleOf(_)`, `recordOf(_)`, `callable(_, _)` |
| Type relations | `equivalent(_, _)`, `subtype(_, _)`, `assignable(_, _)`, `consistent(_, _)`, `conforms(_, _)` |
| Member lookup | `member(_, _, _, _)` |
| Source occurrences | `typeUseAt(_, _)`, `typeUsesOf(_)` |
| Runtime-facing operations | `matches(_, _)`, `validate(_, _)`, `construct(_, _)` |
| Proof inspection | `proofsOf(_)` |

Descriptor accessors are demand-oriented. For example, a caller can request an argument, parameter, or field by index without requiring an eager walk of every nested descriptor. Indexed access must not silently change source order, canonical identity, or the structural rules owned by the relevant language specification.

The public descriptor families provide the following observation groups:

- `KindDescriptor`: display form, argument count/index/sequence, result kind, and kind equivalence.
- `TypeDescriptor`: kind and display form; remaining parameters; applied arguments and origin; callable parameters and return type; field keys/names/types; body where defined; equivalence and subtype queries; and validated application.
- `TypeParameter` and `GenericSignature`: owner, parameter order, kind/variance, and declared constraints.
- `CallableSignature`, `CallableParameter`, and `FieldSignature`: callable/field type information and parameter labels as provided by their semantic signature.
- `TypeUse`: the denoted/value type, written spelling, and any source/evidence/inference/constant facts available to the context.

An absent fact is represented by an explicit unavailable or optional result according to the accessor's declared result type; an accessor must not synthesize source or proof facts from a display string.

Type-form relations use semantic identity and subtype semantics. Runtime descriptor object identity is not semantic type identity. Relation operations must distinguish a proved relation, a disproved relation, a dynamic boundary, and a blocked or otherwise unresolved query.

`matches` is a runtime observation against the supplied value and reified form. `validate` is an explicit, capability-checked operation that may perform bounded validation. Neither operation installs a permanent type token, changes the value's runtime class, or changes later dispatch. `construct` is explicit type-directed construction; it must preserve ordinary construction and access rules.

## 5. Result algebra

Queries that can lack information return tagged result values instead of treating missing metadata or an unresolved relation as success or as a definitive negative result.

`TypingResult` distinguishes:

```text
TypingKnown
TypingUnknown
TypingInvalid
TypingUnavailable
TypingCancelled
TypingBudgetExceeded
TypingInternalFailure
```

`TypeRelationResult` distinguishes `RelationSatisfied`, `RelationRejected`, `RelationDynamicBoundary`, `RelationBlocked`, `RelationCancelled`, `RelationBudgetExceeded`, and `RelationInternalFailure`.

`MemberLookupResult` distinguishes `MemberFound`, `MemberMissing`, `MemberDynamicBoundary`, `MemberBlocked`, `MemberCancelled`, `MemberBudgetExceeded`, and `MemberInternalFailure`.

The variants have distinct meanings:

- **Unknown** means available information does not decide the requested semantic fact.
- **Invalid** means the query's operands or requested operation are not well formed for that API.
- **Unavailable** means required metadata or an operation is not available in the current context/profile.
- **DynamicBoundary** means the query reaches a value or behavior whose static meaning is dynamic.
- **Blocked** means the semantic query cannot currently proceed; it is not a negative answer.
- **Cancelled** and **BudgetExceeded** preserve their control/resource outcomes and must not be collapsed into Unknown or Blocked.
- **InternalFailure** reports an implementation failure and must not be converted into a semantic answer.

Only `TypingKnown`, `RelationSatisfied`, `RelationRejected`, `MemberFound`, and `MemberMissing` assert the corresponding positive or negative query result. `MemberMissing` asserts absence only when the context has enough metadata and authority to complete the lookup. A caller must inspect the result variant before consuming a value or using a result as evidence.

## 6. Coherence, access, and lifetime

For fully reified operands, descriptor identity, kind, subtype, union, intersection, exact-type, projection, and normalization observations must satisfy the laws in [the reflection registry](type/type.reflection-pattern.md). Incomplete reification must remain conservative: reflection must not invent a definitive relation.

Static semantic queries and live runtime operations remain separate. Source declarations, generic signatures, and source type occurrences describe the metadata snapshot represented by the context. Live invocation or construction crosses into the current runtime and must apply ordinary access checks and runtime rules at that boundary.

Reflective member lookup does not grant invocation. Reflective invocation, where available, must use the same authorization relation as ordinary invocation. Private/protected language access is not implied by possession of a type descriptor or by the ability to query a public signature.

Descriptor caching or object reuse is not observable semantic identity. Collection and recreation of a descriptor must preserve the represented semantic form, while runtime object identity may differ where the API does not promise canonical live-object reuse.

## 7. Ownership boundary

This specification owns runtime typing reflection only. The following remain with their existing owners:

- type and kind formation, identity, application, subtyping, and normalization: the type-level and registry specifications;
- associated type ownership and projection resolution: [Type Members](../type-members.md) and [Associated Types](../associated-types.md);
- structural Product and Sum shape semantics: [Structural Products](../structural-products.md) and [Structural Sums](../structural-sums.md);
- callable signature and callable-object reflection: [Callable Reflection](../callables/reflection.md);
- heap payloads, descriptor object representation, and tracing: [VM Values and Objects](../vm/values-and-objects.md).

The reflective API must consume those canonical products. It must not reconstruct types, relations, declarations, access policy, or proof authority from display strings, source spellings, or runtime class names.
