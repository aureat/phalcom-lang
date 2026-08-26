# Phalcom Semantic Correctness Program — Technical Specification 02
## Canonical Callable Application and Operation Semantics

> **Status:** Technical implementation specification  
> **Intended repository path:** `docs/impl/semantic/semantic-correctness/technical/02-canonical-callable-application-and-operation-semantics-implementation-spec.md`  
> **Repository:** `aureat/phalcom-lang`  
> **Verified baseline:** `main` at `6ced2afd83ee89d2a09f45b8ba3821482abf3752` (`docs(spec): consolidate semantic analyzer specification`, 2026-08-26)  
> **Program position:** Semantic Correctness Technical Specification 02 of 10  
> **Preceded by:** Technical Specification 01 — Formal Knowledge, Required-Premise Composition, and Expression-Result Integrity  
> **Followed by:** Technical Specification 03 — Generic Inference and Proof Integrity  
> **Implementation discipline:** RED regression first. No call-like syntax may retain a weaker semantic application path after migration.

---

# 1. Purpose

This specification establishes one canonical semantic application model for every Phalcom expression whose meaning is based on selecting and applying a callable contract.

The current checker already has a comparatively rich ordinary method-call path:

```text
receiver analysis
    ↓
dispatch
    ↓
resolve_call(...)
    ↓
argument analysis
    ↓
assignability / generic inference
    ↓
return knowledge
    ↓
CallCheckResult
```

The central correctness problem is that several other expression forms bypass substantial parts of that path.

Current specialized paths include:

```text
binary operator
    resolve selector
    read return contract
    publish return

subscript get
    inspect List/Map type directly
    OR resolve selector
    read return contract
    publish return

property setter
    perform a local relation check
    return assigned-value knowledge

subscript setter
    special-case List
    check assigned value only
    return assigned-value knowledge
```

These paths are not semantically equivalent to callable application.

Consequences include:

- `1 + "hello"` can resolve `Int.+(Int)` and publish its return without checking `"hello"` against `Int`;
- `list["wrong"]` can infer the list element type without checking the index contract;
- a resolved subscript getter can publish its return without validating index arguments;
- property/subscript setters do not use the ordinary call engine;
- property/subscript assignment expressions currently return the assigned value even though the language specifies that every assignment expression returns `Unit`;
- a callable-valued local fabricates an `Established` return from an assumed callable premise;
- call arguments can fail to be analyzed when dispatch fails;
- dynamic labels and expansions can be approximated as positional selector slots;
- exact return promotion is callable-kind-aware but not receiver/callee-authority-aware;
- syntax-specific paths publish call identity, status, causality, and explanations differently.

The objective is not to add call features.

The objective is:

> Once Phalcom claims that an expression applies a callable contract, every syntax form must use the same semantic application law.

---

# 2. End-state invariant

After this specification:

```text
syntax-specific receiver/callee discovery
                ↓
static selector / operation shape
                ↓
canonical target resolution
                ↓
CallableApplicationTarget
                ↓
canonical application
    ├─ argument mapping
    ├─ contextual argument analysis
    ├─ parameter relations
    ├─ generic-inference delegation
    ├─ fixed/generic return derivation
    ├─ target/premise authority
    ├─ terminal status
    ├─ causal invalidity
    ├─ callable identity
    └─ explanations
                ↓
CallCheckResult
                ↓
syntax-specific result projection, if any
                ↓
TypedExpression
```

The critical law is:

> Syntax may select or construct a callable target. Syntax may not select a weaker callable-application algorithm.

Therefore:

```phalcom
receiver.method(value)
receiver + value
receiver[index]
receiver.property
```

may construct different selectors, but once a callable signature is selected they use the same application engine.

Likewise:

```phalcom
receiver.property = value
receiver[index] = value
```

use that same engine for the underlying setter/indexer call, then apply Phalcom's independent assignment-value rule:

```text
assignment expression result = Unit
```

while preserving the underlying operation's status, causal invalidity, callable identity, and explanation.

---

# 3. Normative basis

This implementation is constrained by the current language and semantic-analyzer specifications.

`docs/spec/semantic-analyzer/04-expression-analysis-and-contextual-typing.md` requires calls to distinguish:

```text
receiver/callee analysis
callable identity resolution
argument mapping
argument checking
generic inference
return derivation
status/invalidity
publication
```

and explicitly requires operators, getters, setters, subscripts, constructors, and iteration to perform every relation/identity judgment their semantics require.

`docs/spec/semantic-analyzer/08-callable-analysis-and-publication.md` distinguishes exact ordinary callables, constructors, trusted native callables, generic specialized returns, callable-valued premises, and unknown/dynamic callees. It also explicitly allows:

```text
known fixed return
+
invalid argument relation
```

to publish:

```text
known result
+
Invalid(C)
```

when the return proposition is independent of the failed argument relation.

`docs/spec/callables/arguments.md` distinguishes:

```text
ArgumentShape
ParameterShape
ParameterLayout
BindingPlan
```

and states that labels are part of selector identity and spread must be evaluated exactly once in source order.

`docs/spec/callables/dispatch.md` defines explicit receiver sends, lexical value calls, and implicit-self sends, and specifies value application as ordinary `call(...)` semantics.

`docs/spec/callables/execution.md` states that every assignment expression returns `Unit`, including local, field, property, and subscript assignment, even when an underlying setter returns another value.

---

# 4. Scope

This technical slice owns:

1. canonical resolved-callable application;
2. explicit target authority;
3. explicit receiver/callee premise authority;
4. static call-shape derivation;
5. fixed argument-to-parameter mapping;
6. argument expected-type propagation;
7. structured relation handling;
8. fixed return derivation;
9. invalid-but-known call products;
10. ordinary method calls;
11. implicit-self calls;
12. callable-valued local calls;
13. unary operators;
14. binary operators;
15. getter callables;
16. setter callables;
17. direct field-write result correctness;
18. subscript getters;
19. subscript setters;
20. constructor result propagation;
21. iteration protocol application boundary;
22. unresolved/dynamic call child analysis;
23. assignment `Unit` semantics;
24. removal/restriction of result-only callable fast paths;
25. metamorphic tests proving syntax-equivalent application behavior.

---

# 5. Non-goals

Technical Specification 03 owns generic proof integrity:

- Unknown generic argument participation;
- substitution solvability versus call validity;
- inference-support authority;
- expected-result constraints versus value evidence;
- generic result dependency on receiver/callee/arguments;
- generic conflict/terminal proof integrity.

Technical Specification 04 owns complete generic receiver specialization.

Technical Specification 05 owns source/formal identity takeover.

Technical Specification 06 owns advisory authority.

Full `*` / `**` / `***` semantic parameter-shape completeness is not implemented here. Current `CallableParameter` has only:

```rust
pub external_label: Option<String>,
pub local_name: String,
pub ty: TypeKnowledge,
pub rest: bool,
```

which cannot represent the complete public rest domain. Unsupported shapes must fail closed or enter a deliberate dynamic boundary rather than be invented from `rest: bool`.

This specification does not change VM/compiler runtime call semantics.

---

# 6. Verified current repository state

## 6.1 `CallCheckResult` is already the right class of result

File:

```text
phalcom-semantic/src/checker/call.rs
```

Current:

```rust
pub struct CallCheckResult {
    pub knowledge: TypeKnowledge,
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
    pub explanation_parents: Vec<ExplanationId>,
    pub callable: Option<CallableId>,
}
```

Keep this result-rich model.

The defect is how it is reached and produced.

## 6.2 `resolve_call` is richer than the fast paths

Current ordinary call handling already:

- begins call causal capture;
- analyzes arguments;
- applies non-generic relations;
- delegates generic inference;
- records terminal status;
- derives return knowledge;
- returns `CallCheckResult`.

This becomes the implementation starting point.

## 6.3 Exact return promotion is exposed too widely

Current `call.rs` exposes:

```rust
pub(crate) fn promote_exact_return(...)
pub(crate) fn exact_return_origin(...)
```

and `expression.rs` imports both.

That permits:

```text
resolve target
-> read return
-> publish result
```

without application.

After this spec, fixed-return derivation is private to the canonical application engine.

## 6.4 Method calls already use `resolve_call`

`synthesize_method_call` is closest to the target architecture but still has gaps:

- target identity is recovered from context-side current-expression state;
- receiver authority does not cap result authority;
- arguments may be skipped on dispatch miss;
- expansion/dynamic labels can be approximated as positional slots;
- generic correctness remains incomplete.

## 6.5 Binary operators bypass RHS relation

Current `synthesize_binary_expr` analyzes both operands, resolves an operator selector, and directly promotes the return.

It never applies the selected parameter contract to the RHS.

Canonical regression:

```phalcom
1 + "hello"
```

for a contract equivalent to:

```text
Int.+(Int) -> Int
```

must become:

```text
knowledge   = Established(Int)
status      = Invalid(C1)
invalidity  = One(C1)
callable    = exact + callable
diagnostic  = ArgumentMismatch
```

assuming the receiver premise is Established.

## 6.6 Unary operators bypass the call engine

There are no explicit arguments, but unary paths still bypass receiver authority, call status, and common identity/explanation handling. They must migrate.

## 6.7 Getter callables bypass the call engine

Direct fields are structural member operations and may remain so.

Getter methods are real callable applications and must use the canonical engine.

## 6.8 Property setters are ad hoc and return the wrong value

Current `synthesize_set_property` manually checks field/setter parameter compatibility and returns RHS knowledge.

This violates the language rule:

```text
assignment result = Unit
```

Setter-call application and assignment-value projection must be separated.

## 6.9 Subscript get has result-only List/Map fast paths

Current `synthesize_index_expr` can derive:

```text
List<T> -> T
Map<K,V> -> V
```

without checking:

```text
List index <: Int
Map key <: K
```

and can publish Established result regardless of receiver evidence status.

## 6.10 Resolved indexer callables also bypass application

Subscript selector dispatch can also directly promote return without checking index arguments.

## 6.11 Subscript set checks only part of the operation

Current `synthesize_set_index_expr`:

- analyzes index expressions;
- does not relate them to index parameters;
- special-cases `List<T>`;
- checks only assigned value against `T`;
- returns assigned-value knowledge.

It must check both index and value contracts and return Unit.

## 6.12 Callable-valued locals can launder authority

Current local callable conversion constructs parameter assumptions but creates:

```rust
TypeKnowledge::established(
    c.return_type,
    EvidenceOrigin::Flow,
)
```

for the synthetic signature return.

An:

```text
Assumed((Int) -> String)
```

callee must not produce:

```text
Established(String)
```

merely because application occurred.

## 6.13 Non-callable local invocation degrades to a value read

A local binding that wins lexical resolution but is not `TypeData::Callable` can fall through to returning the local fact itself.

Thus:

```phalcom
let x = 1
x()
```

can behave semantically like:

```phalcom
x
```

This must become an explicit invalid invocation.

## 6.14 Dispatch miss can skip arguments

Method-call arguments are analyzed through `resolve_call`, which is entered only after dispatch succeeds.

On dispatch miss, argument ASTs can be omitted from semantic analysis.

This violates the required-operand rule.

## 6.15 Static selector construction can lie about dynamic shape

Current code maps:

```text
PackItem::Expand -> SelectorSlot::Positional
```

and can treat non-static labels as positional.

An expansion is not one positional argument.

It makes final selector shape dynamic unless statically normalized by a real pack analysis.

## 6.16 Non-generic argument matching is not a full binding plan

Current code walks positionals/labels but does not model one explicit complete argument-binding plan and does not systematically distinguish missing, extra, duplicate, or unsupported-rest shape.

The canonical engine requires that stage.

## 6.17 Constructor/native semantic kinds already exist

`CallableSemanticKind` already provides:

```rust
Ordinary
Constructor
Native
```

and constructor surfaces already use a `Self`-based return with `ConstructorSemantics`.

Reuse these distinctions.

---

# 7. Architectural decision

Do **not** repair this by adding one local relation check to every expression helper.

That would leave multiple semantic application algorithms.

Introduce one canonical application funnel:

```rust
resolve target
    -> CallableApplicationTarget

apply_resolved_callable(
    ctx,
    target,
    premise,
    arguments,
    expected,
    range,
)
    -> CallCheckResult
```

Syntax helpers own only:

```text
receiver/callee discovery
selector construction
direct-field precedence
syntax-specific result projection
```

They do not own:

```text
argument binding
argument contextual analysis
parameter relations
generic delegation
fixed return authority
call terminal status
call causal invalidity
callable result identity
```

Primary implementation ownership remains:

```text
phalcom-semantic/src/checker/call.rs
```

Do not create parallel `operator_call`, `index_call`, or `setter_call` engines.

---

# 8. Canonical target model

Add to `checker/call.rs`:

```rust
#[derive(Clone, Debug)]
pub(crate) struct CallableApplicationTarget {
    pub signature: CallableSignature,
    pub callable: Option<CallableId>,
    pub authority: CallTargetAuthority,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallTargetAuthority {
    ExactDispatch,
    CallableValue(EvidenceStatus),
    StructuralBuiltin,
}
```

`ExactDispatch` means a canonical declaration/surface target was selected.

`CallableValue(E)` means a first-class callable value supplied the callable contract and its evidence status caps result authority.

`StructuralBuiltin` is transitional and exists only to convert existing direct List/Map indexing behavior into a **complete operation contract** rather than a result-only shortcut.

Do not add structural fallbacks for new features.

---

# 9. Call premise model

Add:

```rust
#[derive(Clone, Debug)]
pub(crate) struct CallPremise {
    pub knowledge: TypeKnowledge,
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
    pub explanation: Option<ExplanationId>,
}
```

It represents the semantic premise required to select the target:

```text
explicit receiver
implicit self
first-class callee
```

Provide constructors from:

```text
TypedExpression
binding flow fact
established implicit environment/self
```

Do not include arguments inside this object.

---

# 10. Premise authority rule

For fixed non-generic returns:

```text
effective result authority
    =
minimum(
    target base authority,
    receiver/callee premise authority
)
```

Base target authority:

```text
ExactDispatch     -> Established
StructuralBuiltin -> Established
CallableValue(E)  -> E
```

Premise authority:

```text
Known Established -> Established
Known Assumed     -> Assumed
Unknown/Dynamic   -> no exact static application
```

Examples:

```text
Established receiver + ExactDispatch + fixed String
    -> Established(String)

Assumed receiver + ExactDispatch + fixed String
    -> Assumed(String)

Assumed callable value + fixed String
    -> Assumed(String)
```

Causal invalidity does not weaken epistemic authority.

A causally invalid but independently Established receiver can still select an exact callable and establish its fixed return while carrying upstream invalidity.

---

# 11. Ordinary argument authority does not automatically weaken fixed return

Do not globally take the weakest status across receiver plus every argument for a fixed result.

The evidence specification distinguishes:

```text
premises determining the result proposition
```

from:

```text
premises determining operation validity
```

For an exact fixed contract:

```text
foo(Int) -> String
```

an assumed `Int` argument does not automatically weaken the fixed `String` proposition.

The argument affects call proof/status.

A generic return whose substitution depends on the argument is different and belongs to Technical Specification 03.

---

# 12. Normalized application arguments

Add a call-oriented view:

```rust
pub(crate) enum ApplicationArgument<'a> {
    Positional {
        expression: &'a Expr,
        range: SourceRange,
    },
    Labeled {
        label: &'a str,
        expression: &'a Expr,
        range: SourceRange,
    },
    DynamicLabel {
        expression: &'a Expr,
        range: SourceRange,
    },
    Expansion {
        expression: &'a Expr,
        range: SourceRange,
    },
}
```

Provide an adapter from `PackItem`.

Preserve source order.

Specialized syntax constructs these directly:

```text
binary -> one Positional RHS
unary -> zero args
setter -> one Positional RHS
subscript setter -> index args + put value
```

Do not fabricate AST nodes merely to reuse the engine.

---

# 13. Static call-shape model

Add:

```rust
pub(crate) enum StaticCallShape {
    Exact(Vec<SelectorSlot>),
    Dynamic(DynamicReason),
}
```

Mapping:

```text
Positional            -> Positional slot
static labeled        -> Label slot
dynamic/computed label -> DynamicRestPack boundary
Expansion             -> DynamicRestPack boundary
```

Forbidden:

```text
Expansion -> one positional slot
Dynamic label -> positional slot
```

The checker must not invent selector identity.

---

# 14. Dynamic shape semantics

If receiver/callee is known but final selector shape depends on unsupported expansion/dynamic labels:

1. analyze all supplied child expressions exactly once;
2. publish a deliberate dynamic call boundary, e.g.:

```text
knowledge = Dynamic(DynamicRestPack)
status    = DynamicBoundary(DynamicRestPack)
```

3. preserve child causal/terminal states.

This is intentionally less precise than eventual semantic completeness and more correct than a false static selector.

---

# 15. Argument binding plan

Separate shape mapping from value analysis.

Add:

```rust
#[derive(Clone, Debug)]
pub(crate) struct ArgumentBinding {
    pub argument_index: usize,
    pub parameter_index: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct ArgumentBindingPlan {
    pub bindings: Vec<ArgumentBinding>,
}
```

and internal shape failures:

```rust
pub(crate) enum ArgumentShapeFailure {
    MissingRequiredParameter { parameter_index: usize },
    UnexpectedPositional { argument_index: usize },
    UnknownLabel { argument_index: usize, label: String },
    DuplicateParameterBinding { parameter_index: usize },
    UnsupportedRestShape,
    DynamicShape,
}
```

`bind_static_arguments(...)` performs no expression analysis and no type relation.

---

# 16. Fixed parameter mapping

For current fixed parameters:

- a positional argument binds the next unmatched parameter with `external_label == None`;
- a labeled argument binds exactly one parameter with matching `external_label`;
- a parameter cannot be bound twice;
- every non-rest required parameter must be bound;
- every ordinary supplied argument must be accounted for.

When target is a canonical exact dispatch, an impossible target/signature shape mismatch can indicate an internal invariant failure rather than a second source error.

When target is a callable value, wrong shape is a source-level call-shape error.

---

# 17. Rest correctness boundary

Current `rest: bool` cannot faithfully represent all public rest modes.

Therefore:

> The canonical engine may only claim acceptance that the current parameter representation can actually prove.

If complete rest semantics are required but unavailable, fail closed or Block.

Do not:

- treat a rest parameter as one ordinary fixed slot;
- accept extra arguments silently;
- infer `**`/`***` behavior from one boolean;
- broaden rest semantics in this correctness slice.

---

# 18. Static call-shape diagnostics

Add if needed:

```rust
DiagnosticCode::CallShapeMismatch
DiagnosticCode::NotCallable
```

canonical strings:

```text
type.call.shape_mismatch
type.call.not_callable
```

Use `ArgumentMismatch` for type relation refutation after shape binding succeeds.

Do not diagnose unsupported dynamic spread as static shape mismatch.

---

# 19. Argument analysis exactly once

After a binding plan is known, analyze arguments in **source order**.

For matched known parameter:

```rust
ExpectedType::proper_from(
    parameter_ty,
    ExpectationOrigin::CallableSignature,
)
```

For an unknown parameter contract or unmatched argument, use no proper expected type.

Do not:

```text
analyze with None
then analyze again with expected type
```

to recover contextual typing.

One AST argument must have one analysis traversal at that call site.

---

# 20. Parameter relations

For every bound fixed non-generic argument, canonical application performs:

```rust
ctx.apply_assignability(
    &argument_typed.knowledge,
    &parameter.ty,
    DiagnosticCode::ArgumentMismatch,
    message,
    argument_range,
)
```

inside the call causal frame.

The structured relation result remains observable:

```text
Proven
Refuted
DynamicBoundary
Blocked
Cancelled
BudgetExceeded
InternalFailure
```

Do not convert relation outcome to boolean.

---

# 21. Invalid argument does not erase fixed return

Canonical example:

```phalcom
1 + "x"
```

with exact target:

```text
Int.+(Int) -> Int
```

result:

```text
knowledge         Established(Int)
status            Invalid(C1)
causalInvalidity  One(C1)
callable          exact + callable
```

The call is invalid but the fixed return proposition remains independently known.

This same rule applies to non-generic methods, getters/indexers with fixed returns, and constructor `Self` returns.

---

# 22. Unknown/dynamic argument with fixed return

If an argument is Unknown and its parameter relation is Blocked, a fixed return may remain known while call status is Blocked if the return is independent.

If an argument crosses a dynamic boundary, the fixed return may remain known while call status is DynamicBoundary.

Do not silently convert either case to Ready.

Do not discard the argument because it has no `TypeId`.

Generic dependency semantics are deferred to Spec 03.

---

# 23. Fixed return derivation

Replace externally callable return promotion with one private operation:

```rust
fn derive_fixed_return(
    target: &CallableApplicationTarget,
    premise: &CallPremise,
    range: SourceRange,
) -> TypeKnowledge
```

Origin mapping:

```text
ExactDispatch + Ordinary    -> CallableSignature
ExactDispatch + Constructor -> ConstructorSemantics
ExactDispatch + Native      -> NativeSignature
StructuralBuiltin           -> DeclarationSemantics
CallableValue               -> CallableSignature
```

Then cap EvidenceStatus using target/premise authority.

Do not use `Flow` merely because application occurs inside flow analysis.

---

# 24. Weakening helper

Add a private helper equivalent to:

```rust
fn weaken_known_to_status(
    knowledge: TypeKnowledge,
    maximum: EvidenceStatus,
    origin: EvidenceOrigin,
    range: SourceRange,
) -> TypeKnowledge
```

Rules:

```text
Established + max Established -> Established
Established + max Assumed     -> Assumed
Assumed + max Established     -> Assumed
Assumed + max Assumed         -> Assumed
Unknown/Dynamic               -> preserve
```

Do not upgrade an already Assumed result.

Keep outer semantic origin separate from evidence strength.

---

# 25. Canonical application API

Introduce:

```rust
pub(crate) fn apply_resolved_callable(
    ctx: &mut CheckingContext<'_>,
    target: &CallableApplicationTarget,
    premise: &CallPremise,
    arguments: &[ApplicationArgument<'_>],
    expected: &ExpectedType,
    call_range: SourceRange,
) -> CallCheckResult
```

Required top-level flow:

```text
begin call causal capture
        ↓
generic?
    yes -> apply_generic_callable
    no  -> bind static arguments
           analyze arguments
           apply relations
           derive fixed return
        ↓
end call causal capture
        ↓
merge receiver/callee causal invalidity
append premise explanation
apply premise authority
copy explicit callable identity
        ↓
CallCheckResult
```

---

# 26. Explicit callable identity

`CallableApplicationTarget.callable` is explicit input.

For exact dispatch:

```text
Some(CallableId)
```

For first-class callable structural type:

```text
None
```

For structural List/Map fallback:

```text
None
```

Do not produce call result identity via:

```rust
ctx.resolved_callable_for_current_expression()
```

inside the application engine.

The side table may remain temporarily for compatibility, but canonical application does not depend on it.

---

# 27. Explicit dispatch target API

Current `CheckingContext::resolve_dispatch` already obtains `ResolvedDispatch` internally, including:

```text
CallableId
CallableSignature
visited owners
```

but strips identity before returning.

Add:

```rust
pub(crate) fn resolve_dispatch_target(
    &mut self,
    receiver: TypeId,
    selector: &Selector,
    lookup: DispatchLookup,
) -> ResolvedDispatchResult
```

It must retain the current dependency recording and perform the same:

```text
applied receiver substitution
Self specialization
```

before returning the resolved target.

Existing `resolve_dispatch` becomes a compatibility projection over this API so specialization/dependency logic remains single-source.

---

# 28. Exact target conversion

Use existing `ResolvedDispatch`.

Application conversion:

```rust
impl CallableApplicationTarget {
    pub(crate) fn from_dispatch(
        resolved: ResolvedDispatch,
    ) -> Self {
        Self {
            signature: resolved.signature,
            callable: Some(resolved.callable),
            authority: CallTargetAuthority::ExactDispatch,
        }
    }
}
```

Do not create a second dispatch result model unless required.

---

# 29. Call causal capture

Keep:

```rust
ctx.begin_call_causal_capture()
ctx.end_call_causal_capture()
```

Arguments analyzed inside the call frame already contribute:

```text
causal invalidity
explanation parents
terminal status
```

The receiver/callee was analyzed before target resolution, so merge its causal invalidity and explanation after call-local capture.

Do not create an unbounded cause list in `CallCheckResult`.

---

# 30. Generic application seam

Split current `resolve_call_inner` conceptually into:

```rust
apply_non_generic_callable(...)
apply_generic_callable(...)
```

`apply_resolved_callable` chooses by `signature.generics`.

Technical Spec 02 may preserve current generic solver behavior except for canonical outer invariants:

- explicit target identity;
- receiver/callee causal state;
- receiver/callee authority cap;
- one syntax/application path;
- no operator/indexer generic bypass.

Do not perform the full Part 03 generic proof rewrite here.

However, after the generic sub-engine returns, enforce:

```text
generic result authority
    <= receiver/callee premise authority
```

so an Assumed receiver/callee cannot yield an Established generic result through outer application.

---

# 31. Unresolved application helper

Add one helper for no-target paths:

```rust
pub(crate) fn analyze_unresolved_application(
    ctx: &mut CheckingContext<'_>,
    premise: &CallPremise,
    arguments: &[ApplicationArgument<'_>],
    reason: UnresolvedApplicationReason,
) -> CallCheckResult
```

It must:

1. analyze every supplied argument exactly once in source order;
2. preserve their diagnostics/causal state;
3. preserve premise causal state;
4. produce the honest unresolved/dynamic result.

No target failure may erase child analysis.

---

# 32. Unknown receiver/callee

If exact target cannot be resolved because the required premise is:

```text
Unknown(R)
```

preserve `R` when it is the actual blocker.

Example:

```text
Unknown(UnresolvedName("x"))
```

must not become:

```text
Unknown(DynamicMessageSend)
```

merely because the parent syntax is a call.

A **known** receiver with a genuine dispatch miss may use `DynamicMessageSend`.

---

# 33. Invalid unavailable receiver/callee

If receiver/callee has no usable type because of an invalid/suppressed upstream cause:

```text
knowledge = Unknown(SuppressedByInvalidCause)
status    = Suppressed(...)
```

after analyzing arguments.

If receiver/callee remains Known despite causal invalidity, do not suppress.

---

# 34. Dynamic receiver/callee

A Dynamic required premise means runtime call authority.

Analyze arguments and publish a DynamicBoundary result.

Do not fabricate a static target.

Child InternalFailure/Cancelled/BudgetExceeded status still participates according to the Part 01 dependency-precedence rules.

---

# 35. Dispatch miss

Known receiver + static selector + missing target:

1. analyze all arguments;
2. preserve child products;
3. return unresolved call knowledge such as `Unknown(DynamicMessageSend)`;
4. do not publish a callable return.

Arguments are not conditional on target success.

---

# 36. `CallCheckResult -> TypedExpression`

Current wrappers repeatedly rebuild:

```text
knowledge
status
callable
explanation parents
causal invalidity
```

Add one conversion, either:

```rust
impl From<CallCheckResult> for TypedExpression
```

or one checker-local helper.

There must be exactly one conversion implementation.

Syntax wrappers must not recalculate call state.

---

# 37. Ordinary method-call migration

Rewrite `synthesize_method_call` so it owns only:

```text
receiver analysis
sacred control-method recognition
static call shape
selector construction
target resolution
canonical application
```

Conceptual shape:

```rust
let receiver = analyze_expression(...);
let premise = CallPremise::from_typed(...);
let arguments = application_arguments(&call.args);

match static_call_shape(&arguments) {
    StaticCallShape::Exact(slots) => {
        let selector = Selector::method(...);

        match ctx.resolve_dispatch_target(...) {
            Found(resolved) => apply_resolved_callable(...),
            Missing { .. } => analyze_unresolved_application(...),
            Dynamic => analyze_unresolved_application(...),
            Ambiguous(_) => analyze_unresolved_application(...),
        }
    }

    StaticCallShape::Dynamic(reason) => {
        analyze_unresolved_application(...)
    }
}
```

No fixed-return promotion remains here.

---

# 38. Sacred control-method exception

Current `synthesize_control_method_call` recognizes structured `ifTrue/ifFalse` and `whileTrue` forms under compiler-specific conditions and performs real flow analysis.

This remains a control-flow path.

The language-design direction treats source control flow as control flow, not merely sugar for a method call.

Same-named sends that do not satisfy the structured recognition rule fall through to ordinary canonical callable application.

Do not force branch/loop joins through `apply_resolved_callable`.

---

# 39. Callable-valued local migration

Preserve lexical precedence:

```text
local/captured/module value
before implicit self
```

If a value binding wins and is `TypeData::Callable(c)`, construct a canonical callable-value target.

Parameters:

```rust
TypeKnowledge::assumed(
    parameter.ty,
    EvidenceOrigin::CallableSignature,
)
```

Return:

```rust
TypeKnowledge::assumed(
    c.return_type,
    EvidenceOrigin::CallableSignature,
)
```

Do not construct an Established return during signature conversion.

Target authority comes from the callee fact EvidenceStatus.

---

# 40. Callable-value selector semantics

Value application semantically means:

```text
value.call(...)
```

A synthetic target should use `call` as the semantic selector base, not the lexical variable name.

`f(...)` does not mean a callable named `f`; `f` is the value binding.

---

# 41. Non-callable lexical invocation

If lexical resolution finds a known non-callable value:

1. lexical resolution still wins;
2. analyze all supplied arguments;
3. emit `NotCallable`;
4. publish Invalid call result;
5. do not return the local fact.

Thus:

```phalcom
let value = 1
value()
```

is not semantically equivalent to `value`.

If lexical fact is Unknown or Dynamic, it still shadows implicit self; preserve that state and do not fall back to a method of the same spelling.

---

# 42. Implicit-self migration

If no lexical value wins and current class context supplies implicit self:

1. obtain current static self/class-object type exactly as today;
2. construct an established environment premise;
3. derive static selector shape;
4. resolve target;
5. canonical application.

Implicit-self sends do not get a separate argument checker.

---

# 43. Nominal/type-form unqualified path

Current unqualified logic also resolves nominal/type names and generic type application.

Do not force type-form application through callable application unless the canonical language semantics says it is an ordinary call.

Constructor expressions such as:

```phalcom
CellNum.new()
```

already use class-object method dispatch and therefore enter canonical call application.

---

# 44. Binary operator migration

Delete direct `promote_exact_return` use from `synthesize_binary_expr`.

New flow:

```text
analyze left receiver
build exact operator selector
resolve target
if Found:
    canonical application analyzes RHS as one positional argument
else:
    analyze RHS once with no target-derived expectation
    unresolved/dynamic result
```

Do not pre-analyze RHS when a target can provide contextual parameter typing.

Source evaluation order remains:

```text
left then right
```

---

# 45. Operator regression

Required:

```phalcom
1 + "x"
```

Assert:

```text
result type       Int
EvidenceStatus    Established when receiver is Established
status            Invalid
causal            contains owning cause
diagnostic        ArgumentMismatch
callable          exact operator CallableId where available
```

A type-only assertion is insufficient.

---

# 46. Operator metamorphic law

For equivalent explicit send syntax supported by the parser:

```text
operator form
explicit method-send form
```

must agree on:

```text
argument relation
result TypeId
EvidenceStatus
EvidenceOrigin
AnalysisStatus
CausalInvalidity
CallableId
diagnostic code
```

except source ranges/presentation.

This is a key canonical-application test.

---

# 47. Unary operator migration

Unary operators resolve their zero-argument operator/getter target and invoke canonical application with no arguments.

This ensures receiver authority, target identity, status, and explanation are uniform.

No direct return promotion remains in `expression.rs`.

---

# 48. Getter migration

`GetPropertyExpr` retains two target classes.

Direct field:

```text
structural member operation
```

Getter callable:

```text
canonical callable application with zero args
```

Do not invent a fake getter callable for direct fields.

For direct fields, use Part 01 required-dependency propagation so causally invalid but known receivers remain analyzable.

---

# 49. Setter callable migration

For a setter method:

1. analyze receiver;
2. resolve setter selector before RHS analysis when possible;
3. canonical application analyzes RHS using setter parameter expectation;
4. canonical relation handling owns mismatch;
5. canonical application produces underlying operation result;
6. assignment syntax projects expression knowledge to Unit.

No ad hoc `sig.parameters.first()` relation loop remains in `expression.rs`.

---

# 50. Assignment result projection

Add one helper, conceptually:

```rust
fn assignment_result_from_call(
    ctx: &mut CheckingContext<'_>,
    operation: CallCheckResult,
    range: SourceRange,
) -> TypedExpression
```

It creates:

```text
knowledge = Established(Unit, Syntax)
```

then copies:

```text
operation.status
operation.causal_invalidity
operation.callable
operation.explanation_parents
```

Assignment syntax owns the Unit proposition.

The setter's return type does not.

---

# 51. Direct field-write migration

Direct fields remain non-call operations.

Flow:

```text
analyze receiver
resolve field contract
analyze RHS with field expected type
apply assignability
combine receiver/RHS dependencies
return Unit
```

The structured relation result must affect the field-write `TypedExpression.status`.

Do not call `apply_assignability` and then ignore Blocked/Dynamic/Cancelled/Budget/InternalFailure outcomes.

Add a small direct-relation-to-expression adapter if needed.

---

# 52. Direct relation result adapter

For non-call structural operations, add helper semantics:

```text
Proven
    -> keep result status

Refuted + cause
    -> Invalid(cause)
       causal includes cause

DynamicBoundary
    -> DynamicBoundary

Blocked
    -> Blocked

Cancelled
    -> Cancelled

BudgetExceeded
    -> BudgetExceeded

InternalFailure
    -> InternalFailure
```

Callable argument relations continue to use call causal capture.

Do not create a second general call-status model.

---

# 53. Assignment value under invalid write

The language independently establishes assignment value as Unit.

Therefore:

```phalcom
object.field = bad
```

may correctly publish:

```text
knowledge  = Established(Unit)
status     = Invalid(C)
invalidity = One(C)
```

This is another invalid-but-known case.

Likewise unresolved/blocked assignment target can still retain Unit knowledge because Unit comes from assignment syntax, while operation status remains non-Ready.

---

# 54. Subscript-get callable migration

User-defined/resolved indexer:

1. analyze receiver;
2. derive static index selector shape;
3. resolve `subscript_get`;
4. canonical application analyzes index arguments against parameters;
5. return canonical result.

Delete result-only return promotion.

---

# 55. Structural List get fallback

If canonical dispatch does not provide the current List indexer semantics, convert the existing shortcut into a complete synthetic target:

```text
receiver: List<T>
parameter: Int
return: T
authority: StructuralBuiltin
```

Then run canonical application.

Do not return `T` without checking index.

Receiver authority applies:

```text
Assumed(List<T>) -> result at most Assumed(T)
```

---

# 56. Structural Map get fallback

For exact:

```text
Map<K,V>
```

construct:

```text
parameter: K
return: V
authority: StructuralBuiltin
```

Then canonical application validates the key.

Do not infer key contract from the actual key expression.

---

# 57. Structural fallback ordering

Preferred end-state:

```text
canonical surface dispatch
    ↓ on Missing
existing transitional structural fallback
    ↓
unresolved
```

Do not override a real user/native canonical indexer with hard-coded List/Map logic.

The implementation plan must verify current core surface coverage before deleting any compatibility fallback.

---

# 58. Subscript-set migration

For a resolved subscript-set callable:

```text
underlying arguments =
index arguments
+
put value
```

canonical application checks all bound parameters.

Then source assignment projects expression knowledge to Unit.

No direct result from put-value knowledge.

---

# 59. Structural List set fallback

Convert current List set special case to a complete target:

```text
receiver: List<T>
parameter 0: Int
parameter 1: T
```

Both relations are mandatory:

```text
index <: Int
value <: T
```

Current behavior checks only the value.

The source assignment result is Unit.

---

# 60. Map structural set is not added automatically

Do not add a new Map set feature merely because Map get has a structural fallback.

If canonical surfaces already support it, use them.

Otherwise leave it to completeness work.

Correctness must not broaden language surface.

---

# 61. Setter/indexer underlying return versus assignment result

The registered subscript-set callable may have an underlying return contract different from Unit.

Do not rewrite callable signature merely to make assignment syntax correct.

Keep layers separate:

```text
underlying callable return
source assignment result Unit
```

This also applies if a user-defined setter method can return another value.

---

# 62. Constructor application

Constructors already arrive through class-object method dispatch.

Canonical application preserves:

```text
CallableSemanticKind::Constructor
```

and derives:

```text
EvidenceOrigin::ConstructorSemantics
```

after current `Self` specialization.

No syntax wrapper may flatten constructor origin.

---

# 63. Invalid constructor argument

For exact constructor with fixed Self result and wrong argument:

```text
knowledge = concrete instance type
origin    = ConstructorSemantics
status    = Invalid(C)
```

when return is independent of bad argument relation.

This proves constructor calls use canonical application rather than a result-only special case.

---

# 64. Native application

Exact `CallableSemanticKind::Native` target:

```text
fixed return origin = NativeSignature
```

but uses the same argument mapping/relation engine.

Native status is not an argument-check bypass.

---

# 65. Iteration protocol boundary

Current `resolve_iteration_element` directly dispatches to `iteratorValue`/`iterate` and returns only `TypeKnowledge`.

No protocol callable with parameters may supply result evidence while required protocol arguments are skipped.

At minimum, parameterized protocol sends must either:

- route through canonical application; or
- fail closed until the loop implementation models required arguments.

Do not grow a parallel call engine in iteration code.

Full iteration protocol completeness is out of scope.

---

# 66. Legacy `check_arguments`

Current `check_arguments` is not the canonical application engine.

Search repository use.

If unused, delete it.

If compatibility requires it, make it a narrow adapter to the same relation helper.

No production expression path may use a second argument checker.

---

# 67. Legacy `match_callable_arguments`

Current `match_callable_arguments` returns only `TypeKnowledge`, discarding call status/causal information.

Delete if unused.

Otherwise document it as compatibility/test projection only.

No new production code may consume a type-only call projection.

---

# 68. Fixed-return helper visibility gate

After migration:

```bash
rg "promote_exact_return|exact_return_origin" phalcom-semantic/src
```

must show no `expression.rs` usage.

Fixed-return derivation must be private to canonical call implementation.

This is a structural correctness gate.

---

# 69. Result-only subscript fast-path gate

Patterns like:

```text
List<T> -> directly create result T
Map<K,V> -> directly create result V
```

may remain only inside structural **target construction**.

They may not construct final `TypedExpression`.

---

# 70. Ad hoc setter relation gate

Resolved setter syntax must not contain:

```text
read first parameter
apply_assignability
return RHS
```

after migration.

The only direct relation path remaining is direct field storage.

---

# 71. Callable-value authority gate

No callable-value conversion may contain:

```rust
TypeKnowledge::established(
    c.return_type,
    EvidenceOrigin::Flow,
)
```

or equivalent promotion.

---

# 72. Argument analysis on target failure

Every call-like producer must analyze supplied arguments exactly once even when:

```text
target missing
target ambiguous
receiver Unknown
receiver Dynamic
callee Unknown
callee Dynamic
shape static-invalid
shape dynamic
```

This is mandatory.

Missing target is not permission to omit child semantic analysis.

---

# 73. Evaluation order

Receiver/callee first.

Arguments next in source order.

Do not reorder labeled arguments into parameter order for analysis.

`ArgumentBindingPlan` maps source argument index to parameter index, then the engine iterates source arguments.

This preserves runtime/source evaluation order while allowing parameter-derived contextual typing.

---

# 74. Shape mismatch child analysis

Even for wrong shape:

```text
extra argument
wrong label
duplicate binding
```

analyze supplied expressions.

Matched arguments may receive parameter expected types.

Unmatched supplied arguments receive no parameter expectation.

A missing parameter has no child to analyze and is diagnosed at call range.

---

# 75. Shape mismatch versus internal invariant

For a callable-value target, wrong shape is source-level.

For an exact canonical dispatch target, selector identity should normally guarantee compatible shape. If exact target/signature mapping is impossible anyway, treat that as semantic invariant failure rather than blaming source twice.

---

# 76. Diagnostics

Recommended additions:

```rust
DiagnosticCode::CallShapeMismatch
DiagnosticCode::NotCallable
```

Use existing:

```text
ArgumentMismatch
```

for argument type relations.

Property setter via callable uses underlying argument diagnostic; do not emit a second assignment mismatch.

Direct field write uses `FieldMismatch`.

Dynamic spread/label shape is not a static shape error.

---

# 77. Callable identity propagation

Every exact resolved call-like expression should carry the exact `CallableId`:

```text
ordinary method
operator
unary operator
getter
setter
subscript get
subscript set
constructor
```

Direct fields and structural fallback targets may have `None`.

Setter/subscript assignment expression retains underlying callable identity even though its expression value is Unit.

---

# 78. Explanation propagation

Canonical application captures argument explanation parents through the existing call frame.

Receiver/callee explanation is appended explicitly once.

Syntax wrappers must not append argument explanations again.

The current expression publication layer may still use generic presentation nodes, but callable identity and causal parents must survive.

---

# 79. Cancellation/budget/internal failure

Canonical application consumes existing shared checker control.

Do not create independent relation budgets.

If fixed result is independently known before cancellation/budget terminal work:

```text
knowledge may remain known
status = Cancelled / BudgetExceeded
```

where semantically justified.

Internal analyzer failure must stay `InternalFailure`, not become `ArgumentMismatch`, `DynamicMessageSend`, or ordinary Unknown.

---

# 80. Generic outer authority cap now, proof rewrite later

Even before Technical Spec 03:

```text
generic result authority
    <= receiver/callee premise authority
```

must hold.

Example:

```text
Assumed receiver
current generic solver returns Established(Int)
```

outer canonical application publishes at most:

```text
Assumed(Int)
```

Part 03 later adds argument/inference-support authority and Unknown-premise participation.

---

# 81. Generic operators/indexers use the same engine

If an operator/indexer target is generic, canonical application delegates to the same generic sub-engine used by an ordinary method.

Do not implement separate generic operator inference.

Part 03 defects must become centralized defects, not syntax-dependent defects.

---

# 82. Existing generic expected-result sequencing

Preserve current intent:

```text
solve value-derived constraints
then add expected-result constraints
```

so expected context does not erase a value-supported substitution.

Do not redesign this in Part 02.

---

# 83. Test file

Create:

```text
phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs
```

Register:

```rust
mod canonical_call_application;
```

in:

```text
phalcom-semantic/tests/semantic/foundations/mod.rs
```

Low-level target/argument-plan tests can remain in `checker/call.rs #[cfg(test)]`.

---

# 84. Required end-to-end regressions

At minimum:

```text
ordinary method valid
ordinary method wrong positional type
ordinary method wrong labeled type
operator valid
operator wrong RHS type
unary operator
getter callable
setter callable valid
setter callable wrong value
direct field write valid
direct field write wrong value
subscript get valid
List subscript wrong index
Map subscript wrong key
user-defined indexer wrong key
subscript set valid
List subscript set wrong index
List subscript set wrong value
callable local valid
callable local assumed authority
non-callable local invocation
dispatch miss with unresolved argument
dynamic spread call shape
constructor valid
constructor bad argument
implicit-self call
assumed receiver fixed-return call
invalid-but-known receiver call
```

---

# 85. Assertion standard

Correctness tests must assert the relevant full semantic product:

```text
TypeKnowledge category
TypeId
EvidenceStatus
EvidenceOrigin
AnalysisStatus
CausalInvalidity
CallableId when exact
diagnostic code/count
```

A test that asserts only final `TypeId` is insufficient.

A test that asserts only “has errors” is insufficient.

---

# 86. Critical regressions

## `1 + "x"`

Expected:

```text
type known from exact fixed operator return
status Invalid
ArgumentMismatch
cause retained
callable retained
```

## `list["x"]`

Expected:

```text
list element return may remain known
index relation non-Ready
```

not Ready success.

## `list[0] = "x"`

Expected:

```text
index checked against Int
value checked against element type
expression value Unit
```

## `obj.field = "x"`

Expected:

```text
field relation checked
expression value Unit
status reflects relation
```

## assumed callable

Expected:

```text
Assumed((Int)->String) call
-> result at most Assumed(String)
```

## non-callable local

Expected:

```text
not callable diagnostic
invocation not equivalent to variable read
```

## dispatch miss with bad argument

Argument expression must still exist in `ExpressionAnalysisIndex`.

---

# 87. Metamorphic laws

## Operator equivalence

Equivalent operator syntax and explicit method send must agree on semantic application.

## Getter equivalence

Getter-call syntax and equivalent callable send must agree when both target the same getter method.

## Setter underlying-operation equivalence

Setter assignment and direct setter invocation share:

```text
target
argument relation
operation status
causal invalidity
```

but intentionally differ in expression value:

```text
assignment -> Unit
direct setter call -> callable return
```

## Fast-path equivalence

Any retained structural fast path must match canonical application in all observable semantic dimensions.

---

# 88. Primary production file changes

Expected:

```text
phalcom-semantic/src/checker/call.rs
    canonical application engine

phalcom-semantic/src/checker/context.rs
    explicit resolved-dispatch target API

phalcom-semantic/src/checker/expression.rs
    syntax wrappers migrated

phalcom-semantic/src/checker/typed_expr.rs
    one CallCheckResult conversion if useful

phalcom-semantic/src/diagnostic.rs
    call-shape / not-callable diagnostics

phalcom-semantic/src/checker/statement.rs
    iteration protocol canonical-call seam

phalcom-semantic/src/checker/mod.rs
    exports / legacy API cleanup
```

Possible minimal change:

```text
phalcom-semantic/src/dispatch.rs
```

only if existing resolved-target visibility requires adjustment.

Tests:

```text
phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs
phalcom-semantic/tests/semantic/foundations/mod.rs
```

Potential fixture enhancement:

```text
phalcom-semantic/tests/semantic/support/fixture.rs
```

---

# 89. `call.rs` required end-state components

The implementation must own equivalents of:

```text
CallableApplicationTarget
CallTargetAuthority
CallPremise
ApplicationArgument
StaticCallShape
ArgumentBinding
ArgumentBindingPlan
ArgumentShapeFailure

application_arguments(...)
static_call_shape(...)
bind_static_arguments(...)
apply_resolved_callable(...)
apply_non_generic_callable(...)
apply_generic_callable(...)
derive_fixed_return(...)
analyze_unresolved_application(...)
```

Current:

```text
resolve_call
resolve_call_inner
promote_exact_return
```

must be refactored around these concepts.

---

# 90. `expression.rs` required end-state

These functions must become syntax/resolution wrappers:

```text
synthesize_method_call
synthesize_unqualified_call
synthesize_binary_expr
synthesize_unary_expr
synthesize_get_property
synthesize_set_property
synthesize_index_expr
synthesize_set_index_expr
```

A long argument relation loop in any of them is a failed migration.

A direct callable return promotion in any of them is a failed migration.

---

# 91. Forbidden implementations

The following are review blockers:

```text
resolve signature -> directly promote return
```

outside canonical application.

```text
List<T> -> directly return T
```

for indexing.

```text
setter relation -> return RHS knowledge
```

for assignment.

```text
PackItem::Expand -> Positional selector slot
```

for static dispatch.

```text
dynamic label -> positional selector slot
```

for static dispatch.

Skipping call arguments on target failure.

Analyzing one argument twice.

Establishing callable-value return from an assumed callee.

Returning a non-callable local fact from invocation syntax.

Using expected result as actual result evidence.

Reducing relation outcome to boolean.

Duplicating setter/index diagnostics after canonical application.

Reordering argument evaluation into parameter order.

Implementing a separate generic operator/index inference path.

---

# 92. Structural acceptance gates

At completion:

```bash
rg "promote_exact_return|exact_return_origin" \
  phalcom-semantic/src
```

must show no production expression-layer use.

Search for direct `CallableSignature.return_type` result publication in `expression.rs`; none may remain for callable targets.

Search for:

```text
PackItem::Expand -> SelectorSlot::Positional
```

in semantic static selector construction; none may remain.

Search for direct List/Map result-only index fast paths; they may exist only as full structural target constructors.

Search setter/index syntax for ad hoc parameter relation loops; resolved callables must delegate.

---

# 93. Behavioral acceptance gates

The following must all hold:

```text
ordinary method -> canonical application
binary operator -> canonical application
unary operator -> canonical application
getter callable -> canonical application
setter callable -> canonical application
subscript getter callable -> canonical application
subscript setter callable -> canonical application
callable-valued local -> canonical application
constructor -> canonical application via method dispatch
```

Every supplied argument is analyzed exactly once even on target failure.

Every bound fixed non-generic argument receives one canonical relation judgment.

Assumed receiver/callee can no longer manufacture Established fixed result.

Bad argument + fixed return produces invalid-but-known result when independent.

Property/subscript/field assignment expression returns Unit.

Dynamic spread shape is explicit rather than approximated.

---

# 94. Verification requirements

Implementation must pass at least:

```bash
cargo fmt --check

cargo test -p phalcom-semantic --lib

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_engine

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::bidirectional_calls

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::semantic_correctness_regressions

cargo test -p phalcom-semantic
```

Then run repository workspace/CI-equivalent verification before merge.

This document does not claim those commands currently pass. They are acceptance requirements.

---

# 95. Correctness versus completeness boundary

At the end of this specification Phalcom may still honestly publish:

```text
Dynamic(DynamicRestPack)
Blocked(...)
Unknown(...)
```

for call/pack shapes it cannot yet model.

That is acceptable.

What it must no longer do is:

```text
pretend expansion is one positional argument
skip arguments
ignore operator/index parameter relations
return RHS from assignment syntax
establish callable-value results from assumed callees
```

This is the correctness/completeness boundary.

---

# 96. Handoff to Technical Specification 03

Part 03 inherits one canonical outer application entry point.

Its work belongs inside:

```text
apply_generic_callable(...)
```

and inference structures.

It must repair:

```text
Unknown generic argument omission
substitution solved != call proven
generic support/result authority
receiver/callee + argument support integration
expected-result constraint authority
generic constraint failure causality
generic terminal outcome proof dependencies
```

without creating another syntax-specific call path.

---

# 97. Final semantic model

For a fixed call:

```text
Γ ⊢ premise ⇒ Kp / Sp / Cp
shape(call) = S
dispatch(Kp.type, S) = target
bind(arguments, target.parameters) = plan

for each bound argument:
    Γ ⊢ arg ⇐ parameter
    relation(arg, parameter) = outcome

deriveFixedReturn(target, premise) = Kr
combine relation outcomes = Sc
combine causal dependencies = Cc

------------------------------------------------

Γ ⊢ call
    ⇒ knowledge Kr
      status Sc
      causal Cc
      callable target.id
```

For assignment syntax:

```text
underlying setter/index/field operation:
    status S
    causal C
    callable target

assignment expression:
    knowledge Established(Unit)
    status S
    causal C
    callable target
```

This separates:

```text
operation validity
```

from:

```text
expression value semantics
```

---

# 98. Completion definition

Technical Specification 02 is complete when Phalcom has one semantic answer to:

> What does it mean to apply this resolved callable contract to these argument expressions?

and syntax wrappers can no longer answer that question differently.

Completion requires:

```text
one target model
one argument mapping model
one fixed-argument relation model
one fixed-return authority model
one call status/causal model
one generic delegation seam
one callable identity propagation path
one unresolved/dynamic child-analysis path
```

with:

```text
methods
operators
getters
setters
subscripts
callable values
constructors
```

converging on that model.

At that point remaining generic proof defects are isolated to one engine, which is the prerequisite for Technical Specification 03.
