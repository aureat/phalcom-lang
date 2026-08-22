# 03 — User-Facing Typing Reflection API and Capabilities

**Status:** Ratified API and implementation specification

**Authority:** Normative user-visible behavior; exact parser grammar for declarations remains assigned to the later syntax specification

**Primary owners:** universe `typing` package, `phalcom-core` runtime adapters, `phalcom-semantic` query facade, metadata from Spec 02

**Dependencies:** [01 — Implementation Architecture](01-implementation-architecture.md), [02 — Runtime Reification and Metadata](02-runtime-reification-and-metadata.md)

## 1. Scope, terminology, and non-goals

This document answers four user-visible questions:

1. What objects represent static types and kinds at runtime?
2. How do class objects, synthetic type forms, source occurrences, runtime classes, and metaclasses interact?
3. Which reflection operations are pure observation, bounded semantic queries, or dynamic invocation boundaries?
4. What exact API names and result states do users see?

`TypeForm` is the common semantic/behavioral role of values denoting type-level forms. It is not `Type`, not a superclass inserted into the object model, and not the runtime class of class objects. When structural protocols land, `TypeForm` becomes a standard signature-only protocol. Until then, `Behavior` and trusted descriptor classes expose the same surface directly.

Non-goals:

- no `Type.currentApplication`;
- no applied-type descriptor forwarding to origin class-side methods;
- no hidden call-frame generic context;
- no `typeOf(runtimeValue)` that guesses an erased static type;
- no type-directed selector identity, method lookup, allocation, or inline-cache key;
- no public constructor for raw descriptor objects;
- no access-control bypass through reflection;
- no reflection result that turns blocked/dynamic/proof-unknown into success.

## 2. Current-state evidence

| Finding | Evidence | Classification | API consequence |
|---|---|---|---|
| Every surface value has one runtime class; every class is an object with a metaclass | [Object model principles](../../../spec/current/object-model.md#1-principles) | **Ratified/normative design** + **Observed current implementation** | `.class` keeps its existing meaning; static type reflection uses different selectors. |
| `ClassObject` stores its metaclass, superclass, method tables, fields, and attributes | [`class.rs`](../../../../phalcom-core/src/heap/class.rs#L21) | **Observed current implementation** | No type descriptor or generic application is inserted into these links/tables. |
| Kernel wiring applies the parallel metaclass rule | [`core_classes.rs`](../../../../phalcom-core/src/universe/core_classes.rs#L17) | **Observed current implementation** | New descriptor classes are ordinary classes under the same rule. |
| Runtime `perform`, `respondsTo`, `methodFor`, and DNU are ordinary Object behavior with authority checks | [`primitive/object.rs`](../../../../phalcom-core/src/primitive/object.rs#L194), [`primitives.rs`](../../../../phalcom-core/src/universe/primitives.rs#L77) | **Observed current implementation** | Static member queries are separate APIs; invocation keeps existing runtime semantics. |
| Current object-model policy applies the same access checks to lookup, cached dispatch, `perform`, `respondsTo`, `methodFor`, and reified methods | [Object model §2.1](../../../spec/current/object-model.md#21-member-namespaces-and-access) | **Ratified/normative design** | Typing reflection never reveals/invokes private metadata without matching authority. |
| `Behavior` currently exposes attributes and `Class` exposes allocation | [`behavior.ph`](../../../../phalcom-core/core/universe/src/object/behavior.ph#L1), [`class.ph`](../../../../phalcom-core/core/universe/src/object/class.ph#L1) | **Observed current implementation** | Add type-form observation to `Behavior`; do not replace class behavior. |
| Static facts distinguish value type knowledge from type/kind denotation | [`ValueSemanticFact`](../../../../phalcom-semantic/src/types/denotation.rs#L12) | **Observed current implementation** | `TypeUse` exposes both facts independently. |
| Static dispatch already models selector, labels, side, inheritance, and `super` lookup start | [`dispatch.rs`](../../../../phalcom-semantic/src/dispatch.rs#L12) | **Observed current implementation** | `TypingContext.member(...)` delegates to this source of truth. |
| LSP retains separate formal and advisory snapshot domains | [`semantic/snapshot.rs`](../../../../phalcom-lsp/src/semantic/snapshot.rs#L49) | **Observed current implementation** | Runtime/source reflection comes from formal metadata; advisory `ValueShape` is never exposed as authoritative TypeForm. |
| Old typing docs reserve `Type.currentApplication` and `out`/`in` | [`02-type-expression-foundation.md`](../../../spec/typing/02-type-expression-foundation.md#L337), [`STATUS.md`](../../../spec/typing/STATUS.md) | **Untracked forward-design input** superseded here | Remove from future implementation; use explicit context and `+`/`-`. |

### 2.1 Review of the implemented tower

**Ratified/normative design.** Implementation is sound at the object/type boundary:

- `TypeData::ClassObject` is internal static value typing, not a wrapper object;
- a class object directly denotes its nominal form/type constructor;
- `.class`, `:`, and `::` remain distinct;
- class/instance-side dispatch and `super` stay runtime-faithful;
- raw semantic IDs do not need runtime identity.

No corrective change to the core class-object model is authorized. Reflection implementation must build on it.

## 3. Object model of reflected semantics

### 3.1 Runtime values and static meanings

| Expression/value | Runtime `.class` | Static value type | Semantic denotation |
|---|---|---|---|
| `42` | `Int` | `Int` | none |
| `Int` | `Int class` | internal class-object type for `Int` | nominal type form `Int :: Type` |
| `List` | `List class` | internal class-object type for `List` | constructor `List :: Type -> Type` |
| reflected `List<Int>` | `AppliedType` | nominal descriptor-object runtime type | applied form `List<Int> :: Type` |
| `Type` | `AtomicKind` | runtime type `AtomicKind` | atomic kind `Type` |
| reflected `Type -> Type` | `ArrowKind` | runtime type `ArrowKind` | arrow kind |
| `Typing.current` result | `TypingContext` | `TypingContext` | no type/kind denotation |

The static value-type column and denotation column are independent. `TypeUse` reports both without substituting one for the other.

### 3.2 Runtime class catalog

Add ordinary universe classes, preferably in `phalcom-core/core/universe/src/typing/` and materialize them through normal module/class machinery:

```text
Object
├── Kind                         abstract runtime base for kind descriptors
│   ├── AtomicKind
│   └── ArrowKind
├── TypeDescriptor               abstract implementation base, not TypeForm ontology
│   ├── AppliedType
│   ├── UnionType
│   ├── TupleType
│   ├── RecordType
│   ├── CallableType
│   ├── SpecialType
│   └── Future descriptor classes
├── TypeParameter                also fulfills TypeForm behavior
├── TypeUse
├── TypingContext
├── TypingResult
├── TypeRelationResult
├── RelationEvidence
├── RelationFailure
├── DynamicBoundary
├── ProofResult
│   ├── ProvenProof
│   ├── DisprovenProof
│   └── UnknownProof
└── ReflectionCapability
```

These classes receive ordinary metaclasses under the existing parallel rule. They do not become new roots or meta-levels. Descriptor payloads use Spec 02's single boxed `Object::SemanticDescriptor` representation.

`Type` is a canonical global singleton value of class `AtomicKind`; it is not a `Class` object. Future `RecordRow` is another atomic-kind singleton. `Kind` is the runtime class vocabulary for reflected kind values, not a kind in the static calculus.

### 3.3 TypeForm role

Normative logical protocol:

```phalcom
@protocol class TypeForm {
  kind -> Kind
  displayName -> String
  declaration -> Option<Object>
  origin -> Option<TypeForm>
  arguments -> Tuple
  typeParameters -> Tuple
  freeParameters -> Tuple
  equivalentTo(_ other: TypeForm) -> Bool
  hash -> Int
}
```

This declaration is the eventual protocol surface, not an instruction to implement protocol syntax as part of this unit. Before protocols land, exact selectors exist on `Behavior`, `TypeDescriptor`, and `TypeParameter`.

Rules:

- class objects return their semantic declaration kind and parameters from loaded formal metadata;
- a nongeneric class has `kind === Type` and empty parameter collections;
- a generic class object has arrow kind and declared parameters;
- an applied form returns its base `origin` and canonical `arguments`;
- `origin == None` for bare nominal/constructor/special/parameter forms;
- `arguments` is empty unless the canonical form is an application;
- `declaration` is the existing class/protocol/alias declaration object when one exists;
- `freeParameters` are canonical occurrence binders, distinct from declared `typeParameters`;
- `equivalentTo` is total for validated forms under one semantic-model version and is separate from `==`, `===`, subtyping, consistency, assignability, conformance, and member lookup;
- class-object equivalence for nominal forms is declaration identity;
- descriptor instances are immutable and their `hash` never changes.

`TypeForm` is never inserted as superclass of `Class`, `Protocol`, or descriptors. Runtime inheritance and static protocol satisfaction remain distinct.

### 3.4 Kind surface

```phalcom
class Kind {
  displayName -> String
  parameters -> Tuple
  result -> Option<Kind>
  equivalentTo(_ other: Kind) -> Bool
  hash -> Int
}
```

Semantics:

- `Type.parameters == ()`; `Type.result == None`;
- arrow kind parameters/results are canonical kind descriptor values;
- kind application is performed by `TypingContext.applyKind`, not user construction of `ArrowKind`;
- `Type.kind` is intentionally absent: reflected kinds are values representing classifiers, not TypeForms classified by themselves;
- `Type :: Type` is never derivable from runtime `.class` or reflection methods.

### 3.5 Type parameters and variance

```phalcom
class TypeParameter {
  owner -> Object
  index -> Int
  name -> Symbol
  variance -> Variance
  kind -> Kind
  upperBound -> Option<TypeForm>
  constraints -> Tuple
  default -> Option<TypeForm>
  validate(_ candidate: TypeForm, in context: TypingContext) -> TypeRelationResult
}
```

Identity is owner plus zero-based declaration index. Name is display/resolution data only. Repeated reflection in one context returns the same live descriptor while cached.

**Ratified/normative design.** Source variance spelling is:

```phalcom
class Producer<+T> {}
class Consumer<-T> {}
class Cell<T> {}
```

`+` means covariance, `-` contravariance, absence invariance. These tokens use Phalcom's unary operator vocabulary and are conceptually the positive/negative transform of the binder. Declaration lowering is compiler-authoritative; it must not perform overridable runtime dispatch or place `+`/`-` in selector identity. Mutable abstractions remain invariant unless a capability/usage proof establishes sound covariance.

The current parser does not support generic class surface syntax; these examples are ratified future syntax input, not a claim of implementation. Full grammar belongs to the syntax specification.

## 4. Result objects: honest terminal states

### 4.1 TypingResult

All fallible lookups/constructions return immutable `TypingResult`:

```phalcom
class TypingResult {
  status -> Symbol
  value -> Option<Object>
  reason -> Option<Object>
  diagnostics -> Tuple

  isKnown -> Bool
  isUnknown -> Bool
  isInvalid -> Bool
  isUnavailable -> Bool
  isCancelled -> Bool
  isBudgetExceeded -> Bool
}
```

Allowed statuses:

```text
#known
#unknown
#invalid
#unavailable
#cancelled
#budgetExceeded
#internalFailure
```

Rules:

- only `#known` has `value = Some(...)`;
- `#unknown` carries a structured epistemic reason;
- `#invalid` carries diagnostics for malformed annotation/type operation;
- `#unavailable` reports missing metadata/profile/unloaded declaration/capability;
- `#cancelled` and `#budgetExceeded` are terminal non-successes;
- `#internalFailure` carries a stable incident identifier, not an arbitrary object;
- helper `unwrap` may raise a typed reflection error, but no implicit coercion to `None` erases the reason.

Once sealed variants/ADTs are ratified, source pattern matching may expose ergonomic cases without changing these statuses or payload semantics.

### 4.2 TypeRelationResult

```phalcom
class TypeRelationResult {
  status -> Symbol
  evidence -> Option<RelationEvidence>
  failure -> Option<RelationFailure>
  boundary -> Option<DynamicBoundary>
  reason -> Option<Object>

  isSatisfied -> Bool
  isRejected -> Bool
  isDynamicBoundary -> Bool
  isBlocked -> Bool
}
```

Statuses are exactly `#satisfied`, `#rejected`, `#dynamicBoundary`, and `#blocked`.

`isSatisfied` is true only for a proved relation. A dynamic boundary is not satisfied merely because execution is permitted. Blocked includes unknown facts, unresolved dependencies, invalid annotations, opaque natives, recursive fixed points, cancellation, and budgets with exact reasons.

### 4.3 ProofResult

```phalcom
class ProofResult {
  status -> Symbol             // #proven | #disproven | #unknown
  obligation -> Object
  trust -> Option<Symbol>      // #kernelChecked | #trustedBackend | #assumedAxiom
  artifact -> Option<Object>
  counterexample -> Option<Object>
  assumptions -> Tuple
  reason -> Option<Object>
  backend -> Option<Object>
}
```

`#proven` always reports trust and evidence identity. `#disproven` may carry a counterexample. `#unknown` carries reason such as timeout, unsupported theory, dynamic boundary, opaque native, stale world, or budget. Runtime contract success is not `#proven` static evidence.

## 5. TypingContext API

### 5.1 Acquiring and pinning context

```phalcom
class Typing {
  @class current -> TypingResult
  @class contextFor(_ module: Module) -> TypingResult
}

class TypingContext {
  snapshot -> Object
  profile -> Symbol
  capabilities -> Tuple
  world -> Object
  refresh -> TypingResult
  restrictTo(_ capabilities: Tuple) -> TypingContext
}
```

`Typing.current` returns the VM/program metadata context. It is not call-frame ambient generic state. A context is immutable with respect to semantic snapshot/profile/capability set/world stamp; its bounded type-construction overlay is semantically immutable and only memoizes canonical values.

`refresh` returns a new context. It does not mutate an existing context or proof artifact. If no metadata was emitted, result is `#unavailable` with `DisabledByBuild`/`NotLoaded` reason.

### 5.2 Forms and occurrences

```phalcom
class TypingContext {
  typeOfDeclaration(_ declaration: Object) -> TypingResult
  signatureOf(_ method: Method) -> TypingResult
  typeUseAt(_ module: Module, range: SourceRange) -> TypingResult
  typeUsesOf(_ declaration: Object) -> TypingResult
  contractsOf(_ method: Method) -> TypingResult
  proofsOf(_ declaration: Object) -> TypingResult
}
```

There is deliberately no `typeOf(value)` overload.

- `value.class` answers runtime class.
- `typeOfDeclaration` answers a declared static form.
- `signatureOf(method)` maps a reified runtime method to its compiler/native signature when identity, world, metadata, and authority match.
- `typeUseAt` answers source-occurrence facts only under `ToolingDebug`/source-use capability.
- arbitrary live values cannot recover erased generic arguments.

### 5.3 Checked form construction

```phalcom
class TypingContext {
  apply(_ origin: TypeForm, arguments: Tuple) -> TypingResult
  applyKind(_ kind: Kind, arguments: Tuple) -> TypingResult
  unionOf(_ members: Tuple) -> TypingResult
  tupleOf(_ elements: Tuple) -> TypingResult
  recordOf(_ fields: Record) -> TypingResult
  callable(_ parameters: Tuple, returns: TypeForm) -> TypingResult
  normalize(_ form: TypeForm) -> TypingResult
  substitute(_ form: TypeForm, using environment: TypeEnvironment) -> TypingResult
}
```

Rules:

- raw descriptor constructors are private/native-internal;
- application validates origin kind, arity, argument kinds, bounds, constraints, and residual kind;
- partial application is allowed only when the source/type-language specification permits that form; internal HKT support alone does not imply every surface context accepts an unsaturated constructor;
- union/record fields normalize deterministically;
- `Unknown`, `Dynamic`, missing, and invalid type uses cannot be construction arguments;
- context node/step budgets apply;
- capture-safe substitution uses owner/index identities and supports inherited specialization;
- record construction is closed until `RecordRow` syntax/semantics implementation lands.

### 5.4 Relations

```phalcom
class TypingContext {
  equivalent(_ left: TypeForm, to right: TypeForm) -> TypeRelationResult
  subtype(_ candidate: TypeForm, of supertype: TypeForm) -> TypeRelationResult
  assignable(_ actual: TypeForm, to expected: TypeForm) -> TypeRelationResult
  consistent(_ left: TypeForm, with right: TypeForm) -> TypeRelationResult
  conforms(_ candidate: TypeForm, to protocol: Object) -> TypeRelationResult
  member(
    on receiver: TypeForm,
    selector: Selector,
    side: Symbol,
    lookup: LookupMode
  ) -> TypeRelationResult
}
```

`equivalent` returns a relation result so invalid/cross-version/budget conditions remain visible; `TypeForm.equivalentTo` is the total convenience predicate for already validated, compatible forms.

`member`:

- uses complete selector identity including labels and positional shape;
- accepts `#instance` or `#class` side;
- `LookupMode.normal` starts at receiver owner;
- `LookupMode.superFrom(definingClass, side)` retains actual receiver and changes lookup start exactly like source `super`;
- returns specialized member view after generic substitution;
- enforces private/protected authority;
- reports constructors explicitly without treating class-side type arguments as runtime dispatch inputs;
- dynamic selector/DNU/opaque native states return boundary/blocked outcomes.

### 5.5 Explicit reflective construction

```phalcom
class TypingContext {
  construct(_ form: TypeForm, arguments: Tuple) -> TypingResult
}
```

This is the sole type-directed construction API and is explicit:

1. validate `form` is a constructible nominal/applied nominal proper type;
2. resolve the origin's existing runtime class object;
3. specialize and validate the declared constructor surface;
4. invoke the same ordinary class-side selector and access checks the source call would use;
5. optionally validate the result at the reflection boundary;
6. record dynamic/reflection/effect/proof-boundary provenance.

It does not expose type arguments to the constructor body, install a current application, clone a class, or change method selection.

Therefore:

```phalcom
List.new()                         // ordinary runtime send
context.apply(List, (Int,))       // returns reflected List<Int>
context.construct(listOfInt, ())  // explicit checked reflective operation
listOfInt.new()                    // ordinary miss; no transparent forwarding
```

## 6. TypeUse and source reflection

```phalcom
class TypeUse {
  owner -> Object
  role -> Symbol
  status -> Symbol
  form -> TypingResult
  denotation -> TypingResult
  written -> Option<String>
  source -> Option<SourceRange>
  evidence -> Tuple
  constant -> Option<Object>
  diagnostics -> Tuple
}
```

Statuses distinguish:

```text
#known
#missingAnnotation
#invalidAnnotation
#unknown
#dynamic
#unresolvedDependency
#internalClassObject
```

Example for source expression `Int`:

```text
form.status       = #internalClassObject
denotation.status = #known
denotation.value  = Int
```

Example for unannotated declaration:

```text
status            = #missingAnnotation
form.status       = #unknown or #unavailable according to inference result
written           = None
```

No API rewrites these states to `Object`, `Any`, or `Dynamic`.

`ConstantFact` reflection is profile/capability/budget controlled. It is evidence associated with the occurrence, not a default singleton TypeForm.

## 7. Runtime reflection versus static semantic queries

These APIs remain separate by design:

| Runtime question | API | Static question | API |
|---|---|---|---|
| What class does this live value have? | `value.class` | What type did source analysis establish? | `TypeUse.form` |
| Does current runtime world have this accessible method? | `value.respondsTo(selector)` | Does the declared receiver type have a member? | `context.member(...)` |
| Which live runtime method resolves now? | `value.methodFor(selector)` | Which specialized signature follows from static surfaces? | `context.member(...)` result |
| Invoke a runtime selector | `value.perform(...)` | Check a statically known send | checker/dispatch query |
| Handle runtime miss | `doesNotUnderstand` | Explain dynamic boundary | relation/member result reason |

`methodFor` and `respondsTo` remain sensitive to the live world and caller authority. Static queries are pinned to snapshot metadata and world assumptions. A mismatch reports stale/dynamic state; one side never silently overwrites the other.

## 8. Capabilities, privacy, and resource policy

### 8.1 Capability set

```rust
bitflags! {
    pub struct ReflectionCapabilities: u32 {
        const OBSERVE_PUBLIC_TYPES   = 1 << 0;
        const CONSTRUCT_TYPES        = 1 << 1;
        const EVALUATE_RELATIONS     = 1 << 2;
        const OBSERVE_SOURCE_USES    = 1 << 3;
        const OBSERVE_PRIVATE_TYPES  = 1 << 4;
        const INSPECT_PROOFS         = 1 << 5;
        const INVOKE_REFLECTIVELY    = 1 << 6;
    }
}
```

Default runtime context grants public observation, bounded type construction, and bounded relations. Source-use/private/proof capabilities require matching metadata profile and caller/tool authority. Reflective invocation requires explicit capability and existing member access checks.

Capabilities are trusted unforgeable payloads. `restrictTo` may remove capabilities only. Deserializing a symbol/list never creates authority.

### 8.2 Privacy

- public signature metadata is observable by default when enabled;
- private/protected metadata requires lexical/module authority equivalent to ordinary reflection;
- a method may be physically known but unavailable for invocation; result explains access denial without leaking its body/source details;
- proof artifacts may redact source/counterexample values according to profile and authority;
- source paths and local facts do not appear in `RuntimePublic` artifacts.

### 8.3 Resource control

Each context exposes read-only limits/usage:

```phalcom
context.limits
context.usage
```

Operations charge nodes, relation pairs, steps, source-use results, diagnostic notes, and proof bytes. Exhaustion returns `#budgetExceeded`. It never falls back to a cheaper unsound relation.

## 9. Open-world, native, FFI, `perform`, and DNU boundaries

### 9.1 World mutation

A context captures `world_version` and surface fingerprints. After method installation/removal:

- pure kind/type structure remains usable;
- member, conformance, runtime-contract, and proof queries return stale-world blocked/unknown until refreshed or explicitly evaluated against a dynamic overlay;
- methods added without typed metadata are runtime-visible and statically dynamic;
- descriptor identity and `equivalentTo` do not change.

### 9.2 Native and FFI

Authoritative versioned native metadata yields normal static/reflection results. Explicit opaque native/FFI surfaces yield boundary objects naming the native callable, missing fact, expected runtime check, effects, and proof consequence. Missing metadata is not treated as `Object -> Object` or `Any`.

### 9.3 `perform`

If selector and complete argument shape are constant, static analysis may validate the same member signature and still records reflection-boundary provenance. Otherwise the result is a dynamic boundary. Runtime behavior remains current `perform` dispatch, including labels, spreads/rest, visibility, inheritance, and DNU.

### 9.4 DNU

DNU is not an implicit structural protocol or universal overload. A class may publish an explicit checked DNU contract in a future specification; until then, a possible DNU path blocks precise member/result proof and produces actionable diagnostics. Runtime proxy behavior remains legal.

## 10. Diagnostics and developer experience

Required diagnostic/runtime error codes:

| Code | Surface | Meaning |
|---|---|---|
| `reflection.metadata.unavailable` | `TypingResult #unavailable` | Build/profile/module lacks requested metadata |
| `reflection.capability.denied` | `#unavailable` or runtime error for invocation | Context lacks authority |
| `reflection.form.invalid` | `#invalid` | Value does not denote/represent a validated TypeForm |
| `reflection.kind.not_applicable` | `#invalid` | Tried to apply atomic/non-arrow kind |
| `reflection.application.arity` | `#invalid` | Wrong type argument count |
| `reflection.application.kind_mismatch` | `#invalid` | Argument kind mismatch |
| `reflection.application.bound` | `#rejected` | Bound/constraint failure |
| `reflection.member.missing` | `#rejected` | Static member absent without accepted dynamic boundary |
| `reflection.member.access_denied` | `#rejected` | Existing authority rules deny access |
| `reflection.dynamic_boundary` | relation `#dynamicBoundary` | Runtime check/dispatch remains |
| `reflection.context.stale_world` | `#blocked`/`#unknown` | World-sensitive result invalidated |
| `reflection.budget_exceeded` | `#budgetExceeded` | Bounded operation stopped |
| `reflection.internal_failure` | `#internalFailure` | Stable incident ID, no fabricated value |
| `proof.unknown` | `ProofResult #unknown` | Reasoned inability to prove/disprove |

CLI/LSP rendering uses the semantic diagnostic contract from Spec 01. Runtime `TypingResult` may carry the same codes/ranges when metadata includes them. Error strings are presentation; code/status/payload are stable.

## 11. Implementation architecture

### 11.1 Universe source and metadata

Create:

```text
phalcom-core/core/universe/src/typing/package.ph
phalcom-core/core/universe/src/typing/kind.ph
phalcom-core/core/universe/src/typing/type-form.ph
phalcom-core/core/universe/src/typing/type-descriptor.ph
phalcom-core/core/universe/src/typing/type-parameter.ph
phalcom-core/core/universe/src/typing/type-use.ph
phalcom-core/core/universe/src/typing/result.ph
phalcom-core/core/universe/src/typing/context.ph
phalcom-core/core/universe/src/typing/proof-result.ph
```

Register these through `phalcom-modules/src/builtin.rs` and native surface catalog. Use source definitions for pure convenience methods/status predicates; native primitives only access validated payloads, context operations, and weak cache.

Add `TypeForm` protocol metadata only when protocol declarations are implemented. Until then, tests assert exact selector parity across `Behavior`, descriptor classes, and `TypeParameter`.

### 11.2 Runtime primitives

Create `phalcom-core/src/primitive/typing.rs` and small domain modules as needed. Primitives:

- never accept raw numeric metadata IDs from Phalcom values;
- downcast trusted descriptor/context payloads;
- delegate semantic operations to validated runtime arena/query adapters;
- preserve ordinary caller authority for member/invocation operations;
- use iterative algorithms and context budgets;
- return immutable result objects;
- bump no world version for pure reflection or canonical memoization.

### 11.3 Semantic query facade

Add `phalcom-semantic/src/reflection.rs`:

```rust
pub struct ReflectionQuery<'s> {
    snapshot: &'s SemanticSnapshot,
    capabilities: ReflectionCapabilities,
    budget: QueryBudget,
}
```

It exposes declaration type, callable signature, type use, relation, member, contract, and proof queries using Spec 01 outcomes. `phalcom-core` converts results to metadata/runtime descriptors. No VM type appears in `phalcom-semantic`.

### 11.4 LSP integration

Formal hover/completion/signature-help should gradually consume the same reflection-query views from the static snapshot. `ValueShape` continues as advisory fallback and is labeled. Never route LSP requests through runtime descriptor objects; both LSP and runtime reflection adapt the same formal semantic products independently.

## 12. Dependency-ordered implementation plan

### Unit C1 — kind and nominal TypeForm observation

**Depends on:** Spec 02 B1–B4 nominal loader/reification.

**Files:** universe typing sources, builtins registry, native metadata, `primitive/typing.rs`, semantic reflection facade.

Implement `Type`, `Kind`, `AtomicKind`, `ArrowKind`, `Typing`, `TypingContext`, `TypingResult`, and `Behavior` selectors `kind/typeParameters/freeParameters/equivalentTo`. Reifying a nominal returns existing class object.

Acceptance: `Int.kind === Type`; generic class kind is arrow; `Type.class == AtomicKind`; `Type.kind` absent; no core tower diff.

### Unit C2 — synthetic TypeForms and checked construction

Implement descriptor classes, TypeForm parity, weak identity, `apply/applyKind/unionOf/tupleOf/recordOf/callable/normalize/substitute`, caps, and exact diagnostics.

Acceptance: same-context identity, cross-context equivalence, kind/arity/bound failures, GC reclamation, no public raw descriptor constructor.

### Unit C3 — TypeUse and source metadata

Implement `TypeUse`, status distinctions, denotation, source spelling/range, evidence/constant facts, and profile/capability checks.

Acceptance: missing/invalid/unknown/dynamic/internal-class-object cases are distinct; source expression `Int` reports internal class-object value type plus nominal denotation; release profile reports exact unavailable reason.

### Unit C4 — relations and member lookup

Implement `TypeRelationResult`, evidence/failure/boundary objects, five distinct relations, member lookup with selector/labels/side/inheritance/`super`/substitution/visibility/constructors, stale-world policy, and budgets.

Acceptance: dynamic never satisfies; cross-module related evidence owns correct module; runtime `respondsTo` and static member result can truthfully differ after world mutation.

### Unit C5 — explicit reflective construction and invocation boundary

Implement `construct` and optional member invocation only through `INVOKE_REFLECTIVELY`. Reuse ordinary dispatch/access/DNU paths. Never introduce current-application state or applied forwarding.

Acceptance: ordinary `List.new` behavior unchanged; `listOfInt.new` misses; explicit construct validates and invokes origin constructor; constructor body has no ambient type arguments.

### Unit C6 — proof and contract reflection

Wait for proof subsystem implementation. Add immutable `ProofResult`/artifact/counterexample/assumption views and trust/status behavior without granting proof authority to runtime values.

Deletion criteria across C1–C6:

- old future `Type.currentApplication` code/spec paths have no implementation references;
- no `out`/`in` variance parsing/reflection remains;
- no duplicate LSP formal query implementation;
- no strong synthetic descriptor cache;
- no nominal wrapper object.

## 13. Verification and acceptance

### Object-model and API invariants

- existing `Object`/`Behavior`/`Class`/`Metaclass` class, superclass, and metaclass relations remain byte-for-byte/behaviorally unchanged;
- descriptor classes obey ordinary parallel metaclass wiring;
- `.class` is total and never returns a static TypeForm;
- `Type` is an `AtomicKind` singleton, not a class and not self-kinded;
- nominal reification is existing class-object identity;
- synthetic descriptors are immutable, hash/equivalence compatible, and weakly canonical;
- no current application, class forwarding, runtime generic token, or selector alteration.

### API/result-state tests

- every API operation reaches exactly one documented terminal status;
- only `#known` exposes a value; only `#satisfied` claims a relation;
- cancellation/budget/internal failure never return cached success;
- `Dynamic`, `Unknown`, missing, invalid, unresolved, and internal class-object status remain distinct;
- equivalence/subtyping/assignability/consistency/conformance/member lookup produce different evidence paths and are independently tested;
- variance reflection uses `+`/`-` semantics and owner/index identity.

### Reflection/runtime integration

- runtime `respondsTo/methodFor/perform/DNU` semantics and visibility tests remain green;
- static `member` uses selector labels, class/instance side, inheritance, `super`, substitution, constructor policy, and current authority;
- constant selector `perform` may validate but remains tagged reflection boundary;
- nonconstant `perform`, DNU, FFI, and opaque native return dynamic/blocked results;
- method installation invalidates world-sensitive queries only;
- private/source/proof metadata cannot be observed without capability/profile.

### GC, fuzz, and performance

- weak cache never retains descriptors or returns wrong reused slot;
- fuzz arbitrary composition sequences under node/step budgets;
- fuzz invalid runtime objects passed to primitives; typed error, no panic;
- deep descriptors render/query iteratively;
- lazy root inspection allocates O(1) descriptors;
- context lifetime and cap prevent unbounded historical form retention;
- benchmark nominal query, cached/uncached application, relation, member lookup, source-use query, and GC with/without descriptors.

### Manual developer validation

In a debug/profile-enabled build:

1. inspect `Int`, `List`, `List<Int>`, a type parameter, union, callable, and arrow kind;
2. compare `.class`, `.kind`, and TypeUse denotation visibly;
3. trigger wrong arity/kind, missing metadata, denied private access, dynamic send, stale world, and budget states;
4. compare CLI/LSP source diagnostics with runtime TypeUse diagnostics;
5. verify REPL cells receive context/source identities and old contexts remain pinned after later cells.

Implementation reports distinguish passing, baseline/unrelated, deferred, and unverified scope. This documentation task relied on supplied Task 13 evidence and ran no tests.

## 14. Risks and ratification gates

| Risk | Gate |
|---|---|
| `TypeForm` protocol implementation pressures object hierarchy | Protocol conformance must remain metadata; no superclass edit |
| `Typing.current` becomes ambient type-directed execution | API review: context is snapshot/world service only; no call-frame type arguments |
| `construct` becomes second dispatch path | Must reuse ordinary origin-class send and access/DNU machinery |
| `equivalentTo` masks version/error state | Only validated compatible objects expose convenience bool; general query returns result object |
| Reflection leaks private source/proof data | Profile plus unforgeable capability plus existing caller authority |
| Runtime world mutation is mistaken for static proof | Stale-world/dynamic outcomes and explicit fingerprints |
| Result API becomes ergonomically noisy | Later sealed variants/pattern syntax may improve use without collapsing states |
| Variance operator becomes overridable semantic behavior | Declaration lowering is compiler-authoritative; no runtime operator dispatch |

**Proposed design needing ratification.** Public syntax for record rows, kind schemes, proof/totality annotations, and protocol declarations remains outside this document. Reflection names and state semantics are fixed; parser implementation must not infer these open grammars from API spelling.

## 15. Take directly / adapt / reject

### Take directly

**Pyrefly architectural transfer.** Immutable snapshot-backed queries, explicit query/result states, stable identities, bounded evaluation, cancellation, deterministic evidence, and shared compiler/editor facts.

### Adapt

**Pyrefly architectural transfer.** Reflection operates on Phalcom class objects, kinds, selectors, labels, class/instance side, `super`, native surfaces, runtime world versions, message sends, and proof boundaries. It uses ordinary Phalcom objects without making them semantic authority.

### Reject

**Ratified/normative design.** Python attribute/descriptor/protocol/overload/import semantics; `Any`-style fallback; editor-only results; type-directed selector/dispatch; `Type.currentApplication`; transparent applied forwarding; raw-pointer caches; global mutable solver state; runtime object identity as proof.
