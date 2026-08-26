# Phalcom Semantic Correctness / Single-World Takeover — Part 1 of 3

## Formal Semantic Epistemic Foundation Implementation Specification

> **Implementation order:** This is **Part 1 of 3** of the Semantic Correctness / Single-World Takeover program. Part 1 must merge and pass its release gate before Part 2 begins. Part 2 must merge before Part 3 begins.
>
> **For implementation agents:** treat this document as normative for the semantic model and architectural boundaries described here. Repository implementation is the source of truth for current code shape; this specification is the source of truth for the intended changes. If the repository has moved materially from the baseline commit, re-run the archaeology gate before editing.

**Goal:** Make the existing compiler-owned formal semantic analyzer epistemically sound before expanding semantic coverage or migrating advisory/LSP semantics into it. After this part, declarations, persistent binding contracts, current value knowledge, assumptions, contradictions, dynamic boundaries, contextual expectations, invalidity, and genuine unknowns are represented and propagated without overwriting one another or fabricating formal facts.

**Baseline repository:** `aureat/phalcom-lang`, `main` at commit `23e6ca126e96b11504a275fa3e777e18fe4d9ef5` (`test(semantic): cover incrementality hardening regressions`).

**Primary crate:** `phalcom-semantic`.

**Secondary compatibility surface:** `phalcom-lsp` must continue compiling against public semantic APIs, but **Part 1 does not migrate or delete the LSP semantic engine**. That is deliberately sequenced into Parts 2 and 3.

**Core architectural principle:** preserve information rather than overwrite it. A developer declaration is a contract. A checker-established fact is a fact. An assumption is an assumption. A contradiction is a relation result. An expected type is context. None of these may silently impersonate another.

---

# 1. Three-part decomposition of the full correctness/takeover program

The Part 1–3 split is a dependency split, not a document-length split.

## Part 1 — Formal Semantic Epistemic Foundation — **this specification**

Implements the work previously grouped under SC-0 through SC-9, plus the checker-local code-organization hardening required to make those changes safe:

- semantic knowledge/evidence representation;
- declaration/contract/current separation;
- binding initialization and reassignment state machines;
- removal of duplicate local-binding current-state stores;
- relation-outcome preservation;
- causal invalidity without fake types;
- epistemically sound branch joins and loop widening;
- contextual/expected-type integrity;
- call-result contract-to-fact promotion;
- the soundness-critical subset of generic inference;
- strict `Unknown` discipline;
- eradication of `Unit`/`Never` sentinel abuse;
- explanation/provenance corrections;
- semantic product fingerprint updates;
- a dedicated semantic epistemic-conflation audit and release gate.

Part 1 intentionally leaves broad AST coverage incomplete. Missing checker paths are made **honest and fail-closed**, not completed here.

## Part 2 — Canonical Semantic Identity, Projection, and Advisory Evidence Takeover

Part 2 will consume the sound formal model from Part 1 and implement SC-10 through SC-13:

- compiler-owned source semantic identities for bindings/expressions/occurrences;
- compiler-owned semantic presentation indices in snapshots;
- canonical site attachment for formal and advisory facts;
- migration of useful `phalcom-lsp/src/semantic` runtime-shape inference into a compiler-owned advisory evidence subsystem;
- deletion/demotion of duplicate LSP class surfaces, dispatch authority, semantic module graph, invalidation, and type inference authority.

Part 2 must not reopen Part 1's epistemic model by stuffing advisory facts into formal `TypeKnowledge`.

## Part 3 — Persistent Workspace/Module Lifecycle and LSP Single-World Cutover

Part 3 will implement SC-14 through SC-19 and the remaining cross-crate cleanup:

- persistent compiler-owned project/module/source lifecycle;
- removal of production `run_static_workspace_analysis(...)` reconstruction;
- replacement of the LSP semantic engine by one compiler semantic session;
- migration of diagnostics, hover, completion, navigation, references, inlays and semantic tokens to compiler-owned products;
- physical deletion of the second semantic system;
- final single-world, cold-vs-incremental, lifecycle, LSP and performance release gates.

This ordering is mandatory. It is unsafe to centralize or expose semantics before the semantics being centralized are sound.

---

# 2. Re-grounding gate before implementation

Before modifying code:

```bash
git status --short
git rev-parse HEAD
git log -n 12 --oneline
```

Expected baseline:

```text
23e6ca126e96b11504a275fa3e777e18fe4d9ef5
```

If HEAD differs, inspect the current implementations—not only filenames or documentation—of at least:

```text
phalcom-semantic/src/types/evidence.rs
phalcom-semantic/src/types/denotation.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/outcome.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/typed_expr.rs
phalcom-semantic/src/checker/expected.rs
phalcom-semantic/src/checker/policy.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/flow/state.rs
phalcom-semantic/src/checker/flow/predicate.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/explain/node.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/presentation.rs
phalcom-semantic/tests/
```

Also inspect the branch test file if the branch still exists:

```text
branch: tests/semantic-authority-composition
file:   phalcom-semantic/tests/semantic_authority_composition.rs
```

Run the baseline tests before editing:

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp
```

Do not silently adapt the intended semantic model merely because names moved. If implementation changes invalidate a design assumption in this specification, record the conflict and resolve it against the invariants below before proceeding.

---

# 3. Semantic Knowledge and Evidence Invariants — normative

These invariants are the release criteria for Part 1. Every implementation choice must preserve them.

## Invariant A — Established knowledge is never overwritten by a declaration

If the checker establishes the type of a value, a compatible or incompatible developer annotation must not replace that fact.

```phalcom
let a: Number = 42
```

means, conceptually:

```text
persistent contract: Number
current knowledge:    Int, Established
contract relation:    Validated
```

not `current = Number`.

## Invariant B — Contradiction is relational, not a replacement type

```phalcom
let b: String = 42
```

means:

```text
persistent contract: String
current knowledge:    Int, Established
contract relation:    Refuted(Int <: String)
diagnostic:           one owning mismatch diagnostic
```

The contradiction must not mutate `current` to `String` or `Unknown` merely to recover.

`Refuted` is not a `TypeKnowledge` status. It describes a failed relation between independently retained semantic facts.

## Invariant C — Assumptions are usable but not established

When formal value evidence is genuinely unavailable and an explicit language contract supplies a usable static type, the result is an assumption.

```phalcom
run(value) {
  let x: Int = value
}
```

If `value` legitimately has no formal type evidence, then:

```text
x current knowledge: Int, Assumed
assumption basis:     developer binding contract
```

The analyzer may use the assumption for ordinary static checking, including rejecting code inconsistent with that contract. It may not present it as checker-established runtime precision or use it as proof for optimizations requiring established facts.

## Invariant D — Advisory evidence is not formal evidence

Part 1 does not introduce advisory evidence into the formal `TypeKnowledge` representation. Advisory evidence remains outside this crate's formal knowledge path until Part 2 provides a separate compiler-owned advisory channel.

No `EvidenceStatus::Advisory`, no `TypeKnowledge::Advisory`, and no compatibility alias that lets LSP `ValueShape` enter formal relations is permitted in Part 1.

This structural separation is intentional: advisory disagreement alone must be incapable of producing a hard compiler rejection.

## Invariant E — Expected types are constraints/context, not value facts

An expected type may guide bidirectional analysis. It may participate in a sound derivation. It does not, by itself, prove that the analyzed expression has that type.

No checker path may implement checking mode by manufacturing a fake `TypeKnowledge` whose origin is “declared” or “expected” and then treating that object as actual value evidence.

## Invariant F — Invalidity and knowledge are orthogonal

A semantic operation can be invalid while its result type is independently known.

Example:

```phalcom
let x = CellNum.fromInt("bad")
```

If dispatch and the callable return contract establish `CellNum`, but the argument is refuted against `Int`, the call expression is invalid **and** its result knowledge remains `CellNum`.

Likewise a bad binding annotation does not erase the initializer's established type.

Recovery uses explicit diagnostic causes/status, not fake types.

## Invariant G — Flow cannot increase epistemic certainty

A branch/loop merge may widen type information, but it may not become more certain than its incoming facts justify.

```text
Established(Int) + Established(Float)
    -> Established(Int | Float), origin Flow

Established(Int) + Assumed(Number)
    -> Assumed(Int | Number), origin Flow

Established(Int) + Unknown(...)
    -> Unknown(...)
```

An arbitrary sample branch may never become the merged fact.

## Invariant H — Persistent binding contracts and current flow knowledge are different objects

For an annotated binding, the persistent assignment contract is the annotation while current flow knowledge may be narrower.

For an unannotated binding whose initializer yields usable typed knowledge, Phalcom retains current monomorphic behavior by deriving a persistent **inferred binding contract**, but this contract must not be misreported as a developer declaration.

Therefore:

```phalcom
let x = 1
x = "text"
```

continues to be checked against the inferred `Int` binding contract, while presentation may still truthfully state that there was no explicit type annotation.

This preserves current static behavior without perpetuating the false `declared = inferred current` representation. **Part 1 therefore makes an explicit language-checker decision:** unannotated bindings with a usable inferred type are monomorphic for reassignment unless and until a later language ruling deliberately changes that policy. If Phalcom later chooses type-changing unannotated `let`, the representation remains valid—the policy would stop creating `InferredInitializer` contracts rather than re-conflating contract and current knowledge.

## Invariant I — Real language types are never unknown sentinels

`Unit`, `Never`, `Object`, or another canonical type may be produced only because a language rule establishes that type.

They may not stand for:

- missing parameter type information;
- unresolved `Self` during inference;
- an unknown record component;
- a missing generic argument;
- a solver failure;
- an unsupported checker path.

`Never` remains valid for genuine bottom semantics such as throw/unreachable and for an empty construction only where the language's type rule genuinely defines bottom as the element type. `Unit` remains valid for genuine unit-valued constructs.

## Invariant J — Unknown is honest and classified

A developer contract may supply an assumption only for an explicitly eligible “no value evidence” state.

It may not hide:

- `UncheckedExpression` / checker coverage gaps;
- unresolved names;
- syntax errors;
- recursive/cyclic analysis blockage;
- an underconstrained generic solver that failed to solve after contextual constraints were applied;
- suppressed invalid dependencies;
- infrastructure cancellation, budget exhaustion, or internal failure.

Semantic completeness will later eliminate coverage gaps. Part 1 makes them impossible to mistake for successful type checking.

---

# 4. Verified current-state defects at the baseline commit

This section is repository-grounded against `23e6ca126...` and explains why each implementation slice exists.

## 4.1 `TypeKnowledge::Known` claims establishment while accepting advisory authority

File:

```text
phalcom-semantic/src/types/evidence.rs
```

Current model:

```rust
pub enum TypeKnowledge {
    Known(TypeEvidence),
    Unknown(UnknownReason),
    Dynamic(DynamicReason),
}

pub struct TypeEvidence {
    pub ty: TypeId,
    pub authority: EvidenceAuthority,
    pub provenance: EvidenceSet,
}

pub enum EvidenceAuthority {
    Declared,
    Proven,
    ExactSyntax,
    TrustedNative,
    Advisory,
}
```

`EvidenceAuthority` combines two unrelated dimensions:

- where a claim came from; and
- what epistemic conclusions a consumer may draw from it.

`Known(Advisory)` also contradicts the documented meaning that the semantic engine established the type.

The current `is_sound_for_rejection()` method bakes consumer policy into this mixed enum.

## 4.2 `Statement::Let` explicitly discards the initializer fact

File:

```text
phalcom-semantic/src/checker/statement.rs
function: check_statement(), Statement::Let arm
```

The checker correctly resolves the annotation, analyzes the initializer, and checks assignability. It then does the wrong thing:

```rust
let effective_fact = if let Some(decl_k) = declared_k {
    let denotation = if is_assignable { val_typed.denotation } else { None };
    ValueSemanticFact { knowledge: decl_k, denotation }
} else {
    val_typed.fact()
};
```

This overwrites the initializer in all annotated cases:

- compatible supertype declarations lose precision;
- refuted declarations replace the checker-known actual type;
- genuinely unknown actuals and contradictions collapse to the same representation.

## 4.3 `bind_local()` independently reintroduces the same conflation

File:

```text
phalcom-semantic/src/checker/context.rs
functions: bind_local_var(), bind_local()
```

Current convenience API:

```rust
pub fn bind_local(&mut self, name: impl Into<String>, fact: ValueSemanticFact, range: SourceRange) {
    let declared = fact.knowledge.ty();
    self.bind_local_var(name, declared, fact.knowledge, true, fact.denotation, range);
}
```

This has two semantic bugs and one API bug:

1. it claims every known initializer type was developer-declared;
2. it cannot represent a broad declaration plus narrow current fact;
3. it hardcodes `mutable = true`.

The structurally healthier `bind_local_var()` already accepts declaration and current separately, but it still lacks contract origin and consistency state.

## 4.4 Local current value knowledge has two mutable stores

File:

```text
phalcom-semantic/src/checker/context.rs
```

Current `CheckingContext` owns both:

```rust
pub local_envs: Vec<LocalEnv>,
pub scopes: Vec<HashMap<String, LocalBindingInfo>>,
pub flow: FlowState,
```

`LocalEnv` stores a `ValueSemanticFact`; `FlowState` independently stores `BindingState.current`. `assign_existing()` updates both. `lookup_local_knowledge()` prefers flow, while variable expression analysis currently reads `lookup_local()` from `LocalEnv`.

Once flow refinement and richer epistemic state exist, this is an unacceptable split-brain representation. Part 1 removes `LocalEnv` as a second owner of current binding knowledge.

## 4.5 Reassignment checks against the previous current fact, not the persistent contract

File:

```text
phalcom-semantic/src/checker/expression.rs
Expr::Assignment arm
```

The code correctly chooses `info.declared` as the downward expectation when available, but final enforcement uses:

```rust
enforce_assignability(..., val_k, &target_fact.knowledge, ...)
```

`target_fact.knowledge` is the current flow fact.

After fixing precision preservation:

```phalcom
let x: Number = 1
x = Float.new()
```

would incorrectly test `Float <: Int` rather than `Float <: Number`.

## 4.6 `FlowState::join()` can fabricate established knowledge

File:

```text
phalcom-semantic/src/checker/flow/state.rs
function: FlowState::join()
```

If all reachable branches have concrete types, the implementation unions them. If any branch does not, it falls back to `sample_binding.current.clone()`.

Therefore an incoming set equivalent to:

```text
Established(Int)
Unknown(...)
```

may merge to `Established(Int)` depending on the sample.

## 4.7 Loop widening substitutes the declaration as current truth

Same file, function:

```text
FlowState::widen_loop_state()
```

Current code replaces changing current knowledge with:

```rust
TypeKnowledge::known(decl, EvidenceAuthority::Declared)
```

whenever a declared type exists.

A declaration is not automatically a sound current-flow widening fact—especially if the declaration was refuted.

Part 1 removes this shortcut. Sophisticated contract-bounded widening may be reintroduced later only with an explicit proof law.

## 4.8 `ExpectedType` loses the reason the expectation exists

File:

```text
phalcom-semantic/src/checker/expected.rs
```

Current representation:

```rust
None
Proper(TypeId)
Inference(InferenceTerm)
```

`ExpectedType::from_knowledge()` strips origin/status/provenance. Derived collection, map and callable expectations also become bare type terms.

`check_typed_expr()` then reconstructs the expected type as `EvidenceAuthority::Declared`, fabricating value evidence from contextual information.

Block parameters derived from an expected callable type are similarly labeled `ExactSyntax` even though their type came from context.

## 4.9 Expression invalidity is inferred by matching diagnostic ranges

File:

```text
phalcom-semantic/src/checker/expression.rs
function: analyze_expression()
```

The wrapper scans existing diagnostics for an error whose primary range equals the expression range, synthesizes a `DiagnosticCauseId` from the local expression ID, and marks the expression invalid.

This is not causal ownership. It can:

- mark an independently known child invalid because a surrounding relation shares its range;
- miss a real cause whose range differs;
- make diagnostic ordering affect semantic status.

The existing causal-suppression test only verifies this same-range behavior; it does not establish correct dependency-based suppression.

## 4.10 Explanation nodes hardcode `Proven` for any concrete expression

Same file, `analyze_expression()`, plus:

```text
phalcom-semantic/src/explain/node.rs
```

Whenever an analyzed expression has a `TypeId`, the wrapper currently records a derivation with `EvidenceAuthority::Proven`, regardless of whether the knowledge came from a declaration, advisory source, or another weaker path.

This makes the explanation graph capable of upgrading evidence independently of the checker result.

## 4.11 Parameters and return contracts are represented as current value evidence

File:

```text
phalcom-semantic/src/checker/body.rs
```

Callable parameters are bound directly from signature `TypeKnowledge`, with the same type also recorded as the binding declaration. Return checking reconstructs a `Declared` `TypeKnowledge`.

A parameter contract is a valid static assumption at body entry. It is not an established exact runtime subtype.

## 4.12 Call-result status differs by implementation path

File:

```text
phalcom-semantic/src/checker/call.rs
```

A solved generic call returns:

```rust
TypeKnowledge::known(specialized_ret, EvidenceAuthority::Proven)
```

while a non-generic call returns:

```rust
signature.return_type.clone().with_range(call_range)
```

Thus equivalent semantic facts receive different authority merely because one call required generic substitution.

Further, blocked/cancelled/budget generic inference can fall back to the raw signature return, potentially publishing a result without a completed specialization.

## 4.13 Generic inference contains real-type placeholders and ignores kind/constraint data

Files:

```text
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/call.rs
```

Confirmed defects:

- `instantiate_generic_signature()` allocates every variable at `KindId::TYPE` even though `TypeParameterData` stores the real parameter kind;
- `TypeTerm::SelfType(_)` converts to `store.unit()`;
- missing parameter type information becomes `InferenceTerm::Canonical(store.unit())` in call inference;
- `GenericSignature.constraints` exist but are not fed into the call inference session;
- call/argument provenance uses dummy `ExpressionId`s;
- variable binding does not robustly enforce the inference variable's recorded kind;
- malformed/unsupported pack shapes can still participate in inference rather than first establishing a sound argument-to-parameter mapping.

## 4.14 Existing synthesis paths use `Unit`/`Never` as information-loss sentinels

Examples in `phalcom-semantic/src/checker/expression.rs` and `call.rs` include:

- unknown record field type -> `Unit`;
- missing generic argument information -> `Never` padding;
- unresolved generic parameter type -> `Unit`;
- unresolved `Self` in inference -> `Unit`.

Some uses of `Unit` and `Never` elsewhere are legitimate language semantics. Part 1 requires a semantic audit rather than a mechanical replacement.

## 4.15 `UnknownReason::UncheckedExpression` can be mistaken for ordinary uncertainty

`expression.rs` still ends with:

```rust
_ => TypedExpression::unknown(UnknownReason::UncheckedExpression)
```

and statement checking still has uncovered statement arms.

Part 1 does not complete those AST paths. It prevents a declaration or expected type from laundering those implementation gaps into apparently successful formal knowledge.

## 4.16 Product fingerprints currently encode the old evidence model

File:

```text
phalcom-semantic/src/db/fingerprint.rs
```

`hash_type_knowledge()` hashes `evidence.authority`. Callable-body product fingerprints hash expression knowledge, binding declaration/current state, flow summaries and statuses.

If Part 1 changes semantic status without updating these fingerprints, Step 5.5's invalidation graph can incorrectly reuse products whose semantic meaning changed.

## 4.17 Binding kind/lifecycle semantics are ignored by the formal checker

Files:

```text
phalcom-ast/src/ast.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/context.rs
```

The AST already records `LetBinding.kind: BindingKind` and documents the implemented language rules: `let` is mutable, `const` is immutable, `const` requires an initializer, a bare-name `let` without an initializer reads the surface `None` value, and same-scope redeclaration is illegal. The runtime compiler has explicit `AssignToImmutable`, `ConstWithoutInitializer`, and `BindingRedeclared` errors.

The formal semantic checker currently ignores `binding.kind` and routes every name binding through `bind_local()`, which hardcodes `mutable = true`; a missing initializer becomes the broad `UnknownReason::UnannotatedDeclaration`; scope insertion can overwrite an existing same-scope name.

These are correctness defects on an already-handled statement form, not future syntax completeness. Part 1 must make the formal binding state honor the binding kind and fail closed where the canonical `None` value cannot yet be represented.

## 4.18 Existing branch regressions already encode part of the target behavior

Branch:

```text
tests/semantic-authority-composition
```

The branch test file covers:

- constructor results as proof;
- factory propagation;
- refuted annotations preserving actual constructor type;
- compatible supertype annotations preserving narrow current knowledge;
- unknown initializer + declaration fallback;
- argument mismatch preserving independently known call result;
- return mismatch preserving independently known tail-expression evidence.

Part 1 ports these concepts to the new representation. It must not preserve the branch's old `EvidenceAuthority::Declared` proxy for an assumption.

---

# 5. Target semantic representation

This section is normative. Equivalent names are acceptable only if they preserve every distinction and API constraint described here.

## 5.1 Replace mixed `EvidenceAuthority` with origin + status

Modify:

```text
phalcom-semantic/src/types/evidence.rs
```

Target conceptual shape:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EvidenceStatus {
    Established,
    Assumed,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EvidenceOrigin {
    Syntax,
    DeclarationSemantics,
    ConstructorSemantics,
    CallableSignature,
    NativeSignature,
    DeveloperAnnotation,
    GenericInference,
    Flow,
    ContextualDerivation,
    PatternDecomposition,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeEvidence {
    ty: TypeId,
    status: EvidenceStatus,
    origin: EvidenceOrigin,
    provenance: EvidenceSet,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeKnowledge {
    Known(TypeEvidence),
    Unknown(UnknownReason),
    Dynamic(DynamicReason),
}
```

Important decisions:

1. **No advisory status exists here.** Advisory moves into a separate evidence channel in Part 2.
2. **Refuted does not exist here.** Refutation is a relation/contract state.
3. **Unknown does not exist as an evidence status.** It remains a `TypeKnowledge` variant carrying a reason.
4. **Dynamic remains a separate `TypeKnowledge` variant.** It is not an assumption.
5. `Known` means “a concrete static type is available to the checker,” not “the checker proved the exact runtime class.” `EvidenceStatus` tells consumers whether that availability is established or assumed.

Make `TypeEvidence` fields private or `pub(crate)` and expose getters. Do not retain unrestricted construction that lets arbitrary callers invent status/origin combinations.

Required constructors:

```rust
impl TypeKnowledge {
    pub fn established(ty: TypeId, origin: EvidenceOrigin) -> Self;
    pub fn assumed(ty: TypeId, origin: EvidenceOrigin) -> Self;

    pub fn ty(&self) -> Option<TypeId>;
    pub fn status(&self) -> Option<EvidenceStatus>;
    pub fn origin(&self) -> Option<EvidenceOrigin>;
    pub fn is_established(&self) -> bool;
    pub fn is_assumed(&self) -> bool;
    pub fn is_unknown(&self) -> bool;
    pub fn is_dynamic(&self) -> bool;

    /// Applies a type transformation while preserving epistemic status,
    /// origin and provenance.
    pub fn map_type(&self, f: impl FnOnce(TypeId) -> TypeId) -> Self;
}
```

Keep `with_range()` or replace it with a provenance helper, but it must preserve status/origin.

Remove `EvidenceAuthority::is_sound_for_rejection()`. In the post-Part-1 formal world, both `Established` and `Assumed` are trusted *static* premises. Their difference matters for epistemic presentation/optimization, not whether a program contract can constrain downstream code. Advisory evidence, which cannot reject, is structurally outside this type.

### Required status/origin examples

| Semantic situation | Status | Origin |
|---|---|---|
| integer literal | Established | Syntax |
| class-object/name resolution | Established | DeclarationSemantics |
| constructor `Self` result | Established | ConstructorSemantics |
| exact source callable return at call site | Established | CallableSignature |
| exact native callable return | Established | NativeSignature |
| successful generic specialization | Established | GenericInference or CallableSignature with generic provenance |
| flow union/refinement from established facts | Established | Flow |
| flow derived from any assumed premise | Assumed | Flow |
| annotation supplying missing local value evidence | Assumed | DeveloperAnnotation |
| typed callable parameter at body entry | Assumed | CallableSignature |
| block parameter supplied by expected callable context | Assumed | ContextualDerivation |

Do not label context-derived block parameter types as `Syntax`.

## 5.2 Keep provenance orthogonal to origin/status

`EvidenceSet` remains the compact source/provenance payload. Do not overload `EvidenceOrigin` with source ranges, callable IDs, or long derivation chains.

The intended split is:

```text
EvidenceOrigin  -> primary derivation category
EvidenceStatus  -> epistemic strength
EvidenceSet     -> compact source/provenance references
ExplanationArena-> rich derivation graph
```

This keeps small hot-path facts small while allowing rich explanations.

---

# 6. Binding contracts, current knowledge, and causal invalidity

## 6.1 Replace `declared: Option<TypeId>` as the only persistent-constraint representation

Create:

```text
phalcom-semantic/src/checker/binding.rs
```

and expose focused types from `checker/mod.rs` as required.

Target shape:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BindingContractOrigin {
    SourceAnnotation,
    InferredInitializer,
    CallableParameter,
    ContextualBlockParameter,
    PatternBinding,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingContract {
    pub ty: TypeId,
    pub origin: BindingContractOrigin,
    pub source: Option<SourceRange>,
}
```

`source` is provenance/presentation metadata. The semantic product fingerprint must include the contract type and origin, but not source movement when product semantics are meant to be range-insensitive.

### Why an inferred contract is required

Current code implicitly treats `let x = 1` as type-constrained by `Int` because `bind_local()` copies the initializer type into `declared` and assignment checks against it.

Part 1 preserves that monomorphic binding behavior but corrects the representation:

```text
no explicit annotation
current = Established(Int, Syntax)
contract = Int, InferredInitializer
```

This makes the following two programs semantically distinguishable:

```phalcom
let x: Number = 1
```

```text
contract = Number, SourceAnnotation
current  = Int, Established
```

versus:

```phalcom
let x = 1
```

```text
contract = Int, InferredInitializer
current  = Int, Established
```

The first may be presented as explicitly declared `Number`; the second may not.

## 6.2 Introduce explicit binding consistency

Target conceptual types:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssumptionBasis {
    MissingValueEvidence(UnknownReason),
    CallableParameterContract,
    ContextualParameterContract,
    DerivedEvidence(EvidenceOrigin),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingConsistency {
    Unconstrained,
    Validated,
    Assumed { basis: AssumptionBasis },
    Refuted {
        actual: TypeId,
        expected: TypeId,
        reason: RefutationReason,
    },
    DynamicBoundary {
        obligation: DynamicBoundaryObligation,
    },
    Blocked(BlockReason),
}
```

`BindingConsistency` describes the relation between the binding's current usable knowledge and its persistent contract at the represented program point.

The distinction between `Validated` and `Assumed` is normative:

- `Validated` means the current fact is **Established** and the relation from that established fact to the persistent contract is proven.
- `Assumed` means the checker has usable static knowledge whose epistemic basis is still assumed. A subtype relation may be structurally true while the premise itself remains assumed; that does **not** upgrade the binding to `Validated`.
- `Refuted` means the current formal fact and contract are inconsistent. It does not replace `current`.
- `DynamicBoundary` records a runtime obligation and keeps the current value dynamic.
- `Blocked` means the relation could not be established or refuted for an honest reason.

Do not store a second “effective type.” `current` already is the usable flow knowledge.

Do not place `DiagnosticCauseId` inside the `Refuted` payload; diagnostic identity and semantic relation identity have different stability.

## 6.3 Evolve `BindingState`, do not create a parallel replacement model

Modify:

```text
phalcom-semantic/src/checker/analysis.rs
```

Target:

```rust
pub struct BindingState {
    pub binding: BindingId,
    pub name: String,
    pub range: SourceRange,
    pub contract: Option<BindingContract>,
    pub current: TypeKnowledge,
    pub denotation: Option<SemanticDenotation>,
    pub consistency: BindingConsistency,
    pub invalidity: CausalInvalidity,
    pub mutable: bool,
    pub version: u32,
    pub explanation: Option<ExplanationId>,
}
```

The important change is not the exact field count. It is that there is one explicit persistent contract, one current value fact, one relation state, and one compact causal-invalidity carrier.

Provide convenience queries rather than duplicating data:

```rust
impl BindingState {
    pub fn contract_type(&self) -> Option<TypeId>;

    /// Only a developer-authored binding annotation.
    pub fn explicit_declared_type(&self) -> Option<TypeId>;

    pub fn inferred_contract_type(&self) -> Option<TypeId>;
}
```

`explicit_declared_type()` returns a value only for `BindingContractOrigin::SourceAnnotation`. A callable parameter contract or contextual block parameter is a real contract, but it is not an explicit local binding annotation and must not be presented as one. Never synthesize “declared” from `current`.

## 6.4 Causal invalidity is a compact algebra, not an optional arbitrary cause

Create:

```text
phalcom-semantic/src/checker/causal.rs
```

Use:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CausalInvalidity {
    #[default]
    Clean,
    One(DiagnosticCauseId),
    Multiple,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SuppressionCause {
    One(DiagnosticCauseId),
    Multiple,
}
```

`SuppressionCause` deliberately has no `Clean` variant. Provide a focused conversion:

```rust
impl CausalInvalidity {
    pub fn join(self, other: Self) -> Self;
    pub fn suppression_cause(self) -> Option<SuppressionCause>;
}
```

Normative join:

```text
Clean      ⊔ Clean      = Clean
Clean      ⊔ One(A)     = One(A)
One(A)     ⊔ One(A)     = One(A)
One(A)     ⊔ One(B)     = Multiple, when A != B
Multiple   ⊔ anything   = Multiple
```

The operation must be commutative, associative and idempotent; `Clean` is the identity. Add direct algebraic tests. Do not represent multiple independent invalid roots by choosing the first predecessor's cause. Exact root sets are intentionally not stored in hot flow facts; diagnostics/explanations remain the rich source of individual roots.

`CausalInvalidity` answers “does this value/fact depend on one or more invalid source judgments?” It is orthogonal to both `TypeKnowledge` and `AnalysisStatus`. In particular, `Established(T) + One(C)` is valid and required for invalid-but-analyzable recovery.

A clean sequential write whose RHS and own relation are clean may reset the current **value's** invalidity to `Clean`: the new stored value no longer depends on the previous invalid value. This is not the same as erasing diagnostics already emitted for the program.

# 7. One owner for local current binding state

## 7.1 Delete `LocalEnv` as a second current-fact store

Modify:

```text
phalcom-semantic/src/checker/context.rs
```

Current architecture keeps both `LocalEnv` and `FlowState`. Part 1 removes that duplication.

Target ownership:

```text
lexical scope map:
    name -> BindingId

FlowState.bindings:
    BindingId -> BindingState
```

`BindingState` owns:

- current `TypeKnowledge`;
- current `SemanticDenotation`;
- persistent contract;
- consistency;
- mutation version;
- causal invalidity.

The lexical scope map owns only visibility/name-to-identity resolution.

A suitable shape is:

```rust
pub struct CheckingContext<'a> {
    // ...
    pub scopes: Vec<HashMap<String, BindingId>>,
    pub flow: FlowState,
    // no local_envs
}
```

If `LocalBindingInfo` remains useful for later lexical metadata, it may wrap `BindingId`, but it must not cache contract/current/denotation fields that can diverge from `BindingState`.

## 7.2 Reads derive a complete current fact from `BindingState`

A local read needs both value semantics and causal semantics. Do not return only a `ValueSemanticFact` and then attempt to reconstruct invalidity from `AnalysisStatus`.

Use an internal result such as:

```rust
pub struct LocalReadFact {
    pub fact: ValueSemanticFact,
    pub causal_invalidity: CausalInvalidity,
}

pub fn lookup_local_fact(&self, name: &str) -> Option<LocalReadFact> {
    let id = self.lookup_binding_id(name)?;
    let state = self.flow.get_binding(id)?;
    Some(LocalReadFact {
        fact: ValueSemanticFact {
            knowledge: state.current.clone(),
            denotation: state.denotation,
        },
        causal_invalidity: state.invalidity,
    })
}
```

Returning by value is intentional. It prevents callers from retaining a reference into an auxiliary environment that can diverge from flow state.

`Expr::Var` must read from this authoritative path. A read of `Established(CellNum)` from a binding with `CausalInvalidity::One(C1)` remains a semantically analyzable `Established(CellNum)` expression; it simply carries `One(C1)` as upstream invalid dependence. It is **not** automatically `AnalysisStatus::Suppressed`.

## 7.3 Writes update only `FlowState`

`assign_existing()` must no longer synchronize two stores.

It should become one explicit operation over a `BindingId`, for example:

```rust
pub fn apply_binding_write(
    &mut self,
    binding: BindingId,
    fact: ValueSemanticFact,
    consistency: BindingConsistency,
    invalidity: CausalInvalidity,
);
```

The operation atomically updates current knowledge, denotation, consistency and causal invalidity, increments version, and invalidates flow predicates referencing the binding.

Do not expose separate setters for `current`, `consistency` and `invalidity` to ordinary checker transfer functions; that makes semantically impossible intermediate states easy to publish accidentally.

# 8. Pure contract reconciliation state machine

Create the semantic transition logic in `checker/binding.rs`. It must be testable without a full `CheckingContext`.

Do not encode this as a boolean `is_assignable`.

## 8.1 Unknown eligibility

Extend `UnknownReason` with a precise legitimate no-evidence reason:

```rust
NoTypeEvidence
```

and define an explicit classifier:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractAssumptionEligibility {
    MaySupplyAssumption,
    MustRemainUnknown,
}

impl UnknownReason {
    pub fn contract_assumption_eligibility(&self) -> ContractAssumptionEligibility;
}
```

Normative policy for Part 1:

| Unknown reason | Contract may supply current assumption? |
|---|---:|
| legitimate `NoTypeEvidence` | yes |
| unresolved name | no |
| checker coverage / `UncheckedExpression` | no |
| syntax error | no |
| recursive fixpoint / blocked dependency | no |
| underconstrained generic after solving | no |
| suppressed invalid cause | no |
| opaque native without a formal contract | no |
| dynamic message send represented as unknown | no; prefer a real `Dynamic` state when semantically dynamic |

Do not reuse broad `UnannotatedDeclaration` as the eligible reason unless every remaining producer is audited and proven to mean legitimate lack of value evidence. At the baseline commit it is used for multiple unrelated fallbacks, so it is not safe as a global eligibility key.

This classifier is intentionally small and explicit. Adding a new `UnknownReason` later should require a deliberate decision about whether it is assumption-eligible rather than falling through to “yes.”

## 8.2 Reconciliation result

Use a pure result:

```rust
pub struct BindingReconciliation {
    pub current: TypeKnowledge,
    pub consistency: BindingConsistency,
}

pub fn reconcile_binding_contract(
    store: &TypeStore,
    hierarchy: &dyn TypeHierarchy,
    contract: Option<&BindingContract>,
    actual: &TypeKnowledge,
) -> BindingReconciliation;
```

The function does not emit diagnostics, allocate causes, mutate flow state, or rewrite provenance.

### Required transition table

#### No contract

```text
actual Established/Assumed/Unknown/Dynamic
    -> current unchanged
    -> Unconstrained
```

A caller creating an unannotated ordinary binding may choose to derive an `InferredInitializer` contract first when the initializer has usable `Known` knowledge.

#### Established actual + contract, relation proven

```text
current      = actual unchanged
consistency  = Validated
```

#### Established actual + contract, relation refuted

```text
current      = actual unchanged
consistency  = Refuted { actual, expected, reason }
```

#### Established actual + contract, relation nonterminal

```text
Blocked          -> current unchanged; BindingConsistency::Blocked
DynamicBoundary  -> current unchanged; BindingConsistency::DynamicBoundary
Cancelled/Budget/Internal -> do not collapse to Validated or Assumed;
                             propagate the terminal analysis state through the caller
```

The pure reconciler may return a richer internal reconciliation outcome if needed so infrastructure terminal states are not forced into `BindingConsistency`.

#### Assumed actual + contract, relation proven

The type relation is structurally proven, but the premise remains assumed:

```text
current      = actual unchanged, still Assumed
consistency  = Assumed { basis = basis derived from actual/origin }
```

Do **not** mark this `Validated`. `Validated` is reserved for a relation proven from established current evidence.

Typical bases:

```text
Assumed(_, CallableSignature)     -> CallableParameterContract
Assumed(_, ContextualDerivation)  -> ContextualParameterContract
Assumed(_, Flow/other)            -> DerivedEvidence(origin)
```

#### Assumed actual + contract, relation refuted

The two accepted static premises contradict each other:

```text
current      = assumed actual unchanged
consistency  = Refuted { actual, expected, reason }
```

This is a hard static contradiction. “Assumed” means unverified runtime fact, not advisory suggestion.

#### Eligible Unknown + explicit/source contract

```text
current      = Assumed(contract.ty, DeveloperAnnotation)
consistency  = Assumed { MissingValueEvidence(original_reason) }
```

This declaration-backed promotion is permitted only when the contract itself is an appropriate language premise for the binding. Do not blindly use an inferred initializer contract to fill the same initializer's unknown evidence; that would be circular.

#### Ineligible Unknown + contract

```text
current      = original Unknown unchanged
consistency  = Blocked(UnknownType(original_reason))
```

The contract remains visible, but it does not launder a checker failure into value knowledge.

#### Dynamic + contract

```text
current      = Dynamic unchanged
consistency  = DynamicBoundary { obligation }
```

Do not replace dynamic knowledge with an assumed declaration.

## 8.3 Reconciliation does not own causal invalidity

`reconcile_binding_contract()` answers a type/contract relation. The caller separately joins:

```text
actual expression causal invalidity
⊔
new cause allocated for a Refuted relation, if this site owns that diagnostic
```

This separation is required because an established actual type can already depend on an earlier invalid site, and because the same pure reconciliation function is reused at branch joins where diagnostics must **not** be re-emitted.

# 9. Binding declaration APIs

The current generic `bind_local()` API is prohibited after Part 1 because it cannot express the semantic distinctions above.

## 9.1 Introduce explicit seed/state construction

A recommended internal shape is:

```rust
pub struct BindingSeed {
    pub name: String,
    pub range: SourceRange,
    pub contract: Option<BindingContract>,
    pub current: TypeKnowledge,
    pub denotation: Option<SemanticDenotation>,
    pub consistency: BindingConsistency,
    pub invalidity: CausalInvalidity,
    pub mutable: bool,
}

impl CheckingContext<'_> {
    pub(crate) fn declare_binding(&mut self, seed: BindingSeed) -> BindingId;
}
```

The point of `BindingSeed` is not object-oriented ceremony. It prevents another high-arity function from encouraging callers to omit or infer contract/current/consistency/invalidity fields implicitly.

`declare_binding()` is the only ordinary lexical-registration path. It must:

1. enforce same-scope identity rules before insertion;
2. allocate exactly one `BindingId`;
3. insert that ID into the current lexical scope;
4. create exactly one authoritative `BindingState` inside `FlowState`;
5. publish/update `BindingAnalysisIndex` from that same state;
6. never reconstruct contract from current knowledge.

## 9.2 Higher-level constructors are semantic operations

Provide focused helpers for the currently implemented binding sources:

```rust
bind_initialized_local(...)
bind_callable_parameter(...)
bind_contextual_block_parameter(...)
bind_pattern_binding(...)
```

Exact names may differ. Each helper must make its semantic source explicit.

Do not add a new “convenience” API that reconstructs contract from current unless its name and type make `InferredInitializer` explicit.

Existing name-only `if let` / `while let` / `for` pattern paths that currently call `bind_local()` must migrate too. Use `PatternBinding`/`PatternDecomposition` semantics rather than pretending those names were source type annotations. Recursive pattern completeness remains later work.

# 10. `let`/`const` initialization semantics

Modify:

```text
phalcom-semantic/src/checker/statement.rs
```

For `Statement::Let` / `const`:

1. resolve the explicit source annotation into a `BindingContract`;
2. derive an `ExpectedType` from the contract **as contextual checking information**, not as value knowledge;
3. analyze the initializer and retain its `TypedExpression`, including `knowledge`, denotation and `causal_invalidity`;
4. if no explicit contract exists and initializer has usable `Known` knowledge, create an `InferredInitializer` contract from that current type to preserve monomorphic inferred-binding behavior;
5. reconcile current knowledge against the persistent contract;
6. allocate a new owning mismatch cause only when this declaration's contract relation is definitively refuted;
7. emit exactly one `BindingInitializerMismatch` diagnostic for that refutation;
8. join the initializer's causal invalidity with the new mismatch cause, if any;
9. create the binding with the actual/reconciled current fact, consistency and joined invalidity.

Pseudocode:

```rust
let explicit_contract = resolve_annotation_as_binding_contract(...);

let expected = explicit_contract
    .as_ref()
    .map(|contract| {
        ExpectedType::proper(
            contract.ty,
            ExpectationOrigin::BindingInitializer,
        )
    })
    .unwrap_or_default();

let typed = analyze_expression(ctx, initializer, &expected);

let contract = explicit_contract.or_else(|| {
    typed.knowledge.ty().map(|ty| BindingContract {
        ty,
        origin: BindingContractOrigin::InferredInitializer,
        source: None,
    })
});

let reconciliation = reconcile_binding_contract(
    ctx.store,
    &ctx.hierarchy,
    contract.as_ref(),
    &typed.knowledge,
);

let own_mismatch_cause =
    emit_binding_mismatch_if_refuted(ctx, &reconciliation.consistency, ...);

let invalidity = typed.causal_invalidity.join(
    own_mismatch_cause
        .map(CausalInvalidity::One)
        .unwrap_or(CausalInvalidity::Clean),
);

ctx.declare_binding(BindingSeed {
    current: reconciliation.current,
    denotation: typed.denotation,
    consistency: reconciliation.consistency,
    invalidity,
    // contract, mutable, ...
});
```

Normative consequences:

```phalcom
let x: Number = 42
```

becomes:

```text
contract     = Number / SourceAnnotation
current      = Int / Established / Syntax
consistency  = Validated
invalidity   = Clean
```

while:

```phalcom
let x: String = 42
```

becomes:

```text
contract     = String / SourceAnnotation
current      = Int / Established / Syntax
consistency  = Refuted(Int <: String)
invalidity   = One(binding-mismatch cause)
```

The second binding remains analyzable as `Int`; the program remains invalid.

Do not null out the initializer denotation merely because the annotation is wrong. A type-form/denotation fact that is independently established remains independently established unless the denotation itself depended on the invalid relation.

Pattern completeness is outside Part 1. Route currently supported name patterns through this machinery; leave unsupported recursive patterns fail-closed for the later completeness stage.

# 11. Binding kind, missing initializer, and same-scope declaration rules

These rules are part of the currently parsed/compiled binding semantics and must not remain incorrect after the binding-state refactor.

## 11.1 Derive mutability from `LetBinding.kind`

When creating `BindingSeed`:

```text
BindingKind::Let   -> mutable = true
BindingKind::Const -> mutable = false
```

Delete every helper that hardcodes local mutability independently of the AST/source binding kind. Callable/contextual parameters continue to use their own language mutability policy.

## 11.2 `const` without an initializer is an owning semantic error

The runtime compiler already has `CompilerError::ConstWithoutInitializer`. Add/use a corresponding canonical `phalcom-semantic::DiagnosticCode` rather than silently producing unknown value knowledge.

The invalid `const` binding may still be registered for recovery/name resolution if the checker architecture requires it, but its current knowledge must be an explicit invalid/blocked unknown state; it must not receive an annotation-backed assumption as if a valid initializer were merely untyped.

## 11.3 Bare-name `let` without an initializer is the language's surface `None` value

The AST contract states that `let x` reads `None`. Do not classify this as `NoTypeEvidence`.

Implementation order:

1. first attempt to obtain the canonical formal type/denotation of the surface `None` value through existing core/universe semantic products;
2. if Part 1's current formal products can represent that value soundly, bind it as established knowledge with the appropriate declaration/core origin;
3. if canonical `None` value semantics depend on later global-value closure and cannot yet be represented without inventing a type, return a dedicated **ineligible coverage/block reason** and keep the contract/current distinction honest.

Do **not** substitute `Unit`, `Object`, `NoTypeEvidence`, or an annotation merely to make `let x` look typed. Part 1 prefers an explicit known coverage boundary over a false fact.

## 11.4 Same-scope redeclaration is rejected without corrupting the first binding

`declare_binding()` must detect an existing spelling in the current lexical scope before inserting the new binding. Nested-scope shadowing remains legal.

Use/add a canonical semantic diagnostic corresponding to the runtime compiler's `BindingRedeclared`. On same-scope redeclaration:

- emit one diagnostic owned by the second declaration;
- retain the first declaration as the scope's resolution target;
- do not silently overwrite its `BindingId`, contract, mutability, or flow state;
- if a recovery-only second `BindingState` is allocated for diagnostics, it must not become the lexical resolution target. Prefer not allocating it unless another analysis product requires it.

## 11.5 Assignment to immutable bindings

Use/add a canonical semantic diagnostic corresponding to `CompilerError::AssignToImmutable`. The invalid assignment expression receives an invalid cause/status, but the semantic write must **not** mutate the immutable binding's current value state.

This differs from a type-invalid write to a mutable binding: a mutable but type-refuted write updates current recovery knowledge (§12.2 below), while an immutable write is not a legal storage transition at all and leaves the binding state unchanged.

---

# 12. Reassignment semantics

Modify:

```text
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/flow/state.rs
phalcom-semantic/src/diagnostic.rs
```

The persistent contract, not the previous current fact, controls assignment admissibility.

## 12.1 Assignment algorithm

For a local binding:

```text
resolve target BindingId
    -> reject immutable write without mutating binding state
    -> read persistent binding contract
    -> analyze RHS with contract as contextual expectation when one exists
    -> reconcile RHS current knowledge against that contract
    -> allocate an assignment mismatch cause only for a definitive refutation
    -> join RHS causal invalidity with this assignment's own mismatch cause
    -> atomically write RHS fact + consistency + joined invalidity to FlowState
```

If the unannotated binding received an `InferredInitializer` contract, that contract participates exactly like an explicit contract for assignment checking.

The check target is:

```text
new actual value <: persistent binding contract
```

Never:

```text
new actual value <: previous flow-current type
```

### Required regression

```phalcom
let x: Number = 1
x = Float.new()
```

Must check:

```text
Float <: Number
```

and then update:

```text
current      = Float / Established
contract     = Number / SourceAnnotation
consistency  = Validated
invalidity   = causal invalidity of Float.new()
```

It must never check `Float <: Int` merely because `Int` was the prior current fact.

## 12.2 Invalid assignment recovery

For:

```phalcom
let x: Number = 1
x = "bad"
```

produce:

```text
contract = Number
current  = String / Established
consistency = Refuted(String <: Number)
assignment expression status = Invalid(C1)
binding invalidity = rhs.causal_invalidity ⊔ One(C1)
```

Why retain the RHS as current? Because source execution, if reached, attempted that write, and downstream recovery should reason from the actual known value rather than lying that the variable still contains `Number` or `Int`.

The program remains invalid due to the owning assignment diagnostic.

A later independent clean write may replace this current value and reset the current value's causal invalidity if the new write itself validates. Diagnostics already emitted for the earlier invalid write remain diagnostics of the program.

## 12.3 Invalid RHS and independently owning write errors

Suppose the RHS is itself invalid but still has established result knowledge:

```text
rhs knowledge           = Established(CellNum)
rhs status              = Invalid(C0)
rhs causal invalidity   = One(C0)
assignment relation     = Refuted(CellNum <: Number), own cause C1
```

The binding after the attempted mutable write is:

```text
current       = Established(CellNum)
consistency   = Refuted(...)
invalidity    = One(C0) ⊔ One(C1) = Multiple
```

Do not choose C0 or C1 arbitrarily. The RHS and assignment diagnostics remain independently owned.

If the assignment relation validates, no new cause is allocated and the binding simply inherits the RHS causal invalidity.

## 12.4 Mutability

The new write API must enforce `BindingState.mutable`.

For an immutable target:

- emit/route the canonical immutable-assignment diagnostic;
- mark the assignment expression `Invalid(own cause)`;
- do **not** change current value, denotation, version, consistency, contract or binding invalidity;
- the illegal RHS expression may still be analyzed for its own diagnostics/evidence, but it does not become the stored value because no legal storage transition occurred.

This differs from a type-invalid write to a mutable binding: a mutable but type-refuted write updates current recovery knowledge, while an immutable write is not a legal storage transition at all.

# 13. Relation APIs: preserve outcomes, separate diagnostics from semantics

Modify:

```text
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/checker/policy.rs
```

## 13.1 Remove authority-gated refutation from formal knowledge

Current `check_assignability_bounded()` only propagates a refutation when both old `EvidenceAuthority`s are rejection-sound.

After Part 1, formal `Known` knowledge contains only established or language-contract-assumed facts. Both are valid premises for static checking. Advisory evidence is absent.

Therefore the formal relation layer no longer needs `is_sound_for_rejection()`.

For two `Known` formal types, evaluate the type relation and return its real result.

## 13.2 Add a knowledge-to-contract relation API

Do not manufacture expected `TypeKnowledge` merely to call the relation engine.

Add a focused API, conceptually:

```rust
pub fn check_knowledge_against_type(
    store: &TypeStore,
    hierarchy: &dyn TypeHierarchy,
    actual: &TypeKnowledge,
    expected: TypeId,
) -> Assignability;
```

Bounded variant if required:

```rust
pub fn check_knowledge_against_type_bounded(
    store: &TypeStore,
    hierarchy: &dyn TypeHierarchy,
    actual: &TypeKnowledge,
    expected: TypeId,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
) -> RelationOutcome;
```

Semantics:

```text
Known(actual)      -> subtype(actual.ty, expected)
Unknown(reason)    -> Blocked(UnknownType(reason))
Dynamic(reason)    -> DynamicBoundary(obligation)
```

This API is the canonical path for binding, assignment, return, argument, field and expected-type contracts.

## 13.3 Diagnostic policy returns the full judgment

Current `enforce_assignability() -> bool` loses semantic information.

Replace/augment it with an adapter that returns `Assignability` after optional diagnostic emission:

```rust
pub fn diagnose_assignability_to_type(...) -> Assignability;
```

Only `Refuted` emits a mismatch diagnostic. `Blocked`, `DynamicBoundary`, cancellation, budget and internal states remain non-refuting and propagate honestly.

Callers that mutate semantic state must inspect the full enum. A boolean may remain as a tiny presentation/test helper only where no state decision depends on it.

## 13.4 Do not use `TypeId::DUMMY` for user-visible refutation payloads

The current `From<RelationOutcome> for Assignability` maps some nominal failures to dummy IDs. Preserve real `actual` and `expected` IDs at the wrapper boundary. If `RelationFailure` itself carries declaration IDs, the caller that initiated the relation already knows the concrete `TypeId`s and should populate the public `Assignability::Refuted` with those real operands.

A diagnostic/explanation must not report a dummy semantic type for a real contradiction.

---

# 14. Expression result status and causal invalidity

Modify:

```text
phalcom-semantic/src/checker/typed_expr.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/tests/spec04_5_causal_suppression.rs
```

This section is a core correctness boundary. The checker must carry three orthogonal dimensions:

```text
TypeKnowledge       -> what static type knowledge is available?
AnalysisStatus      -> what happened to the judgment owned by this expression?
CausalInvalidity    -> does this expression's value depend on invalid upstream judgments?
```

Do not derive any one of these dimensions from another.

## 14.1 `TypedExpression` carries status and causal invalidity explicitly

Target:

```rust
pub struct TypedExpression {
    pub knowledge: TypeKnowledge,
    pub denotation: Option<SemanticDenotation>,
    pub dispatch_lookup: DispatchLookup,
    pub constraints: Vec<TypeConstraint>,
    pub provenance: EvidenceSet,
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
}
```

`ExpressionAnalysis` must publish the same causal field:

```rust
pub struct ExpressionAnalysis {
    // existing identity/range/knowledge/denotation...
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
    // explanation/call...
}
```

Representative legal combinations:

```text
Ready
+ Established(Int)
+ Clean
    -> ordinary valid expression

Invalid(C1)
+ Established(CellNum)
+ One(C1)
    -> this expression owns a contradiction, but its result type is independently known

Ready
+ Established(CellNum)
+ One(C1)
    -> expression is analyzable and owns no new error, but its value depends on an earlier invalid write

Suppressed(One(C1))
+ Unknown(SuppressedByInvalidCause)
+ One(C1)
    -> this expression cannot form an independent judgment because an invalid upstream premise removed it
```

Constructors should default ordinary established/assumed/unknown expressions to `Clean`, but child-derived expressions must join child causal invalidity explicitly. Do not provide a constructor that silently discards a supplied child's causal state.

## 14.2 Evolve `AnalysisStatus` with first-class suppression

Modify `checker/analysis.rs` conceptually to:

```rust
pub enum AnalysisStatus {
    Ready,
    Invalid(DiagnosticCauseId),
    Suppressed(SuppressionCause),
    Blocked(BlockReason),
    DynamicBoundary(DynamicReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(AnalysisIncidentId),
}
```

`SuppressionCause` is the non-clean type from §6.4, so `Suppressed(Clean)` is unrepresentable.

Semantics:

- `Invalid(C)` — this expression owns a hard semantic diagnostic/root cause.
- `Suppressed(...)` — this expression owns no new diagnostic because a required premise is invalid upstream.
- `Blocked(...)` — non-diagnostic semantic unavailability not caused by a hard invalid root.
- `Ready` — this expression's own judgment succeeded; it may still carry non-clean `causal_invalidity`.
- `DynamicBoundary` — this expression crosses an intentional runtime-dynamic boundary.
- cancellation/budget/internal remain terminal infrastructure states.

Do not make parent status contagious. A child `Invalid(C)` does not imply every parent is `Invalid(C)`. The parent decides whether it can make an independent judgment from remaining sound premises.

## 14.3 Delete diagnostic-range inference of expression status

Current `analyze_expression()` scans diagnostics for an error whose range matches the expression and synthesizes `Invalid`.

Delete that mechanism.

Target:

```text
analyze_expression_inner()
    returns TypedExpression with explicit status + causal_invalidity

analyze_expression()
    records exactly those fields into ExpressionAnalysis
```

Diagnostic ordering, overlapping source ranges and nested expressions must have no effect on semantic status.

## 14.4 Allocate diagnostic causes explicitly

Add a monotonic cause allocator to the body checking context rather than deriving causes from expression IDs:

```rust
pub fn alloc_diagnostic_cause(&mut self) -> DiagnosticCauseId;
```

Use `SemanticDiagnostic::with_root_cause(cause)` for root diagnostics.

A judgment site that owns a new hard error allocates one cause exactly once. The same cause is written to:

```text
AnalysisStatus::Invalid(cause)
CausalInvalidity::One(cause) joined with upstream invalidity
SemanticDiagnostic.root_cause
```

where applicable.

Do not derive `DiagnosticCauseId` from `ExpressionId`, source range, diagnostic vector position, or hash order.

## 14.5 Causal propagation is dependency-sensitive

When a parent expression's result semantically depends on a child value, join the child's `causal_invalidity` into the parent result.

Examples:

```text
literal
    -> Clean

variable read
    -> binding.invalidity

tuple/list element composition
    -> join invalidity of elements whose values form the result

receiver method call
    -> receiver invalidity participates in call-result invalidity
    -> argument invalidity participates when call semantics depend on evaluating that argument

assignment stored value
    -> RHS invalidity plus own assignment relation cause

flow join
    -> join incoming binding invalidities
```

This is value/result dependency propagation, not status propagation.

If a child is invalid but the parent's result is independently established by a contract, the parent may remain `Ready` or own a different `Invalid` status while still carrying the child's causal invalidity.

## 14.6 Causal suppression rule

Suppress a downstream judgment only when its necessary semantic premise truly disappeared because of invalidity.

Example that must **not** suppress:

```phalcom
let x: Int = CellNum.new()
let y = x.cellOnly()
```

Required behavior:

```text
CellNum.new()
    -> Ready + Established(CellNum) + Clean

binding x
    -> contract Int
    -> current Established(CellNum)
    -> consistency Refuted
    -> invalidity One(C1)

read x in x.cellOnly()
    -> Ready + Established(CellNum) + One(C1)

x.cellOnly()
    -> dispatch can independently resolve from established CellNum
    -> Ready + Established(Int) + One(C1)
    -> no second annotation-mismatch diagnostic
```

`x.current` must not become `Unknown(SuppressedByInvalidCause)` simply because the annotation is wrong.

A true suppression case is different: if the only premise required to perform a judgment is unavailable specifically because an upstream invalid site destroyed that premise, produce:

```text
status             = Suppressed(SuppressionCause::One(C1))
knowledge          = Unknown(SuppressedByInvalidCause)
causal_invalidity  = One(C1)
no duplicate diagnostic
```

If two independent invalid roots are required:

```text
status             = Suppressed(SuppressionCause::Multiple)
causal_invalidity  = Multiple
```

Never select an arbitrary one of multiple roots.

`BlockReason::SuppressedDependency` may remain for internal query/relation plumbing, but expression-level suppression must be distinguishable from ordinary blocked analysis.

## 14.7 Call argument mismatch illustrates invalid-but-analyzable expressions

For:

```phalcom
CellNum.fromInt("bad")
```

if dispatch exactly resolves `fromInt(Int) -> CellNum`:

```text
argument literal
    -> Ready + Established(String) + Clean

argument relation
    -> Refuted; diagnostic root C2

call expression
    -> Invalid(C2)
    -> Established(CellNum) from exact callable return contract
    -> causal_invalidity One(C2)
```

The result fact survives because it does not depend on the false proposition that the argument is an `Int`.

If the argument expression itself already carries `One(C1)`, the call joins that invalidity with its own argument-mismatch cause C2; the result may therefore carry `Multiple`.

## 14.8 Status combination is local, not a universal lattice

Some helpers need to combine an existing status with a new judgment owned by the **same expression**—for example synthesis followed by an expected-type check. Define that helper narrowly.

A reasonable local precedence for terminal ownership is:

```text
InternalFailure / Cancelled / BudgetExceeded
    > Invalid
    > Suppressed
    > Blocked / DynamicBoundary
    > Ready
```

but do not expose this as a general parent-child status join. Parent expressions decide their own status from their semantic rule; only `CausalInvalidity` has the general dependency join algebra.

# 15. Flow-state epistemic algebra

Modify:

```text
phalcom-semantic/src/checker/flow/state.rs
phalcom-semantic/src/checker/expression.rs
```

Create one reusable knowledge-join function. Do not implement slightly different branch semantics in flow state and expression synthesis.

A suitable interface:

```rust
pub fn join_type_knowledge(
    store: &mut TypeStore,
    incoming: &[TypeKnowledge],
) -> TypeKnowledge;
```

`FlowState::join()` itself must additionally receive the tracked hierarchy/relation context because contract consistency has to be recomputed after the current knowledge is merged:

```rust
pub fn join(
    states: &[FlowState],
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
) -> FlowState;
```

Equivalent relation callback/context plumbing is acceptable. Production callers must pass `CheckingContext`'s tracking hierarchy, not a raw untracked hierarchy, so the Step 5.5 dependency capture remains intact when a join proves/refutes a contract through inheritance.

## 15.1 Join law for `TypeKnowledge`

Ignore unreachable predecessor states before calling the knowledge join.

### All `Known`

Form the canonical union of all incoming `TypeId`s.

Status:

```text
all Established -> Established
any Assumed     -> Assumed
```

Origin is `EvidenceOrigin::Flow`.

Merge provenance deterministically and boundedly. Do not discard all provenance merely because origin becomes `Flow`.

### Any `Unknown`

The result is `Unknown`, even if other branches are known.

Do not choose one branch. Do not use the declaration as a substitute.

When multiple unknown reasons occur, use a deterministic conservative merge reason. Prefer a compact dedicated flow-merge reason over arbitrarily selecting the first incoming reason; do not allocate recursively nested reason trees.

### Any `Dynamic`

If there are no `Unknown` predecessors, the merged value is `Dynamic`: at least one reachable path deliberately escapes static type guarantees.

### Unknown + Dynamic

`Unknown` wins. This is deliberately fail-closed. A dynamic predecessor must not hide a checker coverage gap, unresolved dependency, underconstrained solver state or other unknown premise.

If a future stage proves a particular producer is semantically dynamic, normalize that producer to `TypeKnowledge::Dynamic` **before** the join. The join never launders `Unknown` into `Dynamic`.

## 15.2 Binding membership at a join

A binding remains present only if it is semantically in scope on every reachable predecessor.

Implement the conservative intersection of binding IDs across reachable incoming states. Do not copy a sample binding that exists on only one path.

Lexical scope should normally make sets align. Add debug diagnostics/assertions around unexpected divergence rather than choosing arbitrary metadata.

## 15.3 Persistent contract/mutability invariance

For the same `BindingId`, all reachable incoming states must have equal persistent contracts and mutability.

Different contracts for the same binding identity are an internal semantic invariant failure, not a user annotation mismatch.

Do not pick the first contract. Fail closed in release behavior and assert richly in debug/tests.

## 15.4 Join current denotation and causal invalidity independently

Current denotation:

```text
same denotation on every reachable predecessor -> preserve
otherwise                                    -> None
```

Causal invalidity:

```text
joined.invalidity = fold(CausalInvalidity::join, incoming.invalidity)
```

The causal join is independent of whether current type knowledge becomes known/unknown/dynamic.

Examples:

```text
Established(Int), Clean
⊔ Established(Int), One(C1)
= Established(Int, Flow), One(C1)

Established(Int), One(C1)
⊔ Established(Float), One(C2)
= Established(Int | Float, Flow), Multiple
```

Do not choose the invalidity of the predecessor from which the type happened to be sampled—there is no sampling.

## 15.5 Recompute consistency from merged current knowledge

Do not build a second lattice for `BindingConsistency`.

After joining current knowledge, call the same pure reconciliation logic used by initialization/assignment against the invariant binding contract.

This yields:

```text
branch A: Established(Int), contract Number, Validated
branch B: Established(String), contract Number, Refuted
joined current = Established(Int | String, Flow)
relation (Int | String) <: Number -> Refuted
```

No new diagnostic is emitted at the join. The owning write-site diagnostic already exists. Joined causal invalidity carries the upstream invalid root(s).

If joined current is `Assumed`, a structurally proven subtype relation yields `BindingConsistency::Assumed { DerivedEvidence(Flow) }`, not `Validated`.

## 15.6 Flow joins must not strengthen epistemic status

Directly test at least:

```text
Established(Int) ⊔ Established(Float)
    -> Established(Int | Float)

Established(Int) ⊔ Assumed(Float)
    -> Assumed(Int | Float)

Established(Int) ⊔ Unknown(reason)
    -> Unknown(merged reason)

Assumed(Int) ⊔ Assumed(Int)
    -> Assumed(Int)

Dynamic ⊔ Established(Int)
    -> Dynamic, when no Unknown path exists
```

No combination containing weaker reachable knowledge may produce stronger knowledge than every input.

## 15.7 Loop widening in Part 1

Delete the current shortcut that replaces varying current knowledge with the declared type.

For Part 1:

1. use the same sound flow join between header and next-header states;
2. recompute contract consistency using the tracked hierarchy;
3. join causal invalidity;
4. preserve the sound result through the existing bounded fixed-point process;
5. if an emergency nonconvergence widening is required, fail closed to explicit unknown/blocked analysis rather than setting `current = contract`.

Do not add a sophisticated abstract-domain widening algorithm in Part 1. The key invariant is that the contract is not fabricated as current knowledge.

## 15.8 Determinism

`FlowState` already uses `BTreeMap`; preserve deterministic iteration.

Canonical union construction already normalizes ordering through `TypeStore::union`. Do not introduce hash-order-sensitive predecessor selection, cause selection or provenance ordering.

Property/unit tests for `CausalInvalidity::join` and representative `join_type_knowledge` combinations should validate commutativity/idempotence where semantically applicable.

# 16. Expected/contextual type representation

Modify:

```text
phalcom-semantic/src/checker/expected.rs
```

Replace bare expectations with typed contextual roles.

Recommended shape:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExpectationOrigin {
    BindingInitializer,
    Assignment,
    Return,
    Argument { parameter_index: u16 },
    CollectionElement,
    MapKey,
    MapValue,
    CallableParameter { parameter_index: u16 },
    CallableReturn,
    GenericExpectedResult,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ExpectedType {
    #[default]
    None,
    Proper {
        ty: TypeId,
        origin: ExpectationOrigin,
    },
    Inference {
        term: InferenceTerm,
        origin: ExpectationOrigin,
    },
}
```

Exact enum shape may differ, but every non-empty expectation must retain an explicit role.

Delete `ExpectedType::from_knowledge()`.

Required constructors:

```rust
ExpectedType::proper(ty, origin)
ExpectedType::inference(term, origin)
```

Collection/map/callable projection helpers must return expectations with the appropriate derived role rather than silently preserving an unrelated label.

## 16.1 Expected types never become `TypeKnowledge` by conversion

`check_typed_expr()` must use the knowledge-to-contract relation API:

```rust
if let Some(expected_ty) = expected.ty() {
    diagnose_assignability_to_type(..., &typed.knowledge, expected_ty, ...);
}
```

It must not construct `TypeKnowledge::assumed(expected_ty, DeveloperAnnotation)` merely to perform checking.

The checking obligation also affects `TypedExpression.status` without overwriting its knowledge:

| Relation outcome | Expression status effect | Knowledge effect |
|---|---|---|
| Proven | preserve synthesis status/Ready | unchanged |
| Refuted | `Invalid(cause)` | unchanged |
| DynamicBoundary | `DynamicBoundary` when the check itself crosses a dynamic boundary | keep `Dynamic`/actual knowledge |
| Blocked | `Blocked(reason)` unless a more severe existing status already owns the expression | unchanged |
| Cancelled | `Cancelled` | unchanged |
| BudgetExceeded | `BudgetExceeded` | unchanged |
| InternalFailure | `InternalFailure` | unchanged |

Status precedence must be defined in one helper rather than ad hoc at call sites. A reasonable order is infrastructure terminal states > invalid > blocked/dynamic > ready, while never using precedence to rewrite `TypeKnowledge`.

## 16.2 Context can participate in a proof only through a language rule

Examples:

- an empty list literal may use an expected `List<Int>` element type if the bidirectional typing rule defines that result;
- an unknown non-empty list element may not be silently replaced by `Int` merely because `List<Int>` was expected;
- a block parameter type supplied by an expected callable type is an `Assumed` current parameter fact with `ContextualDerivation` origin, not `ExactSyntax`;
- an expected result may constrain generic inference, but a solver that remains underconstrained does not return the expected result as if inferred.

Part 1 does not need to maximize bidirectional inference. It needs every existing bidirectional path to be sound.

---

# 17. Callable parameters and return contracts

Modify:

```text
phalcom-semantic/src/checker/body.rs
```

## 17.1 Parameter entry

For a parameter whose callable signature contains a concrete type:

```text
binding contract = parameter type, CallableParameter
current          = same type, Assumed, CallableSignature
consistency      = Assumed { CallableParameterContract }
mutable          = false unless language parameter semantics say otherwise
```

This means the checker may reject `p + something` inconsistent with the parameter contract, but cannot claim the runtime value's exact type was independently proven at body entry.

For an untyped parameter:

```text
contract    = None
current     = Unknown(NoTypeEvidence)
consistency = Unconstrained
```

This is the canonical eligible no-evidence case used by declaration-backed local assumptions.

## 17.2 Return checking

Store expected return as contextual contract data, not `TypeKnowledge::Declared`.

If changing the `CheckingContext.expected_return` field is practical, make it:

```rust
pub expected_return: Option<ExpectedType>
```

or a dedicated `ReturnContract` wrapping a `TypeId`/`TypeTerm` plus source.

Do not retain `Option<TypeKnowledge>` for a pure contract.

Tail expression and explicit return checking call `check_knowledge_against_type()` and diagnose only a real `Refuted` result.

A return mismatch does not replace the tail expression's independent knowledge.

---

# 18. Contract-to-fact promotion for dispatch/call results

Modify:

```text
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/dispatch.rs (only as needed to retain resolved identity)
```

## 18.1 Retain resolved callable identity through checker dispatch

The lower dispatch layer already has `ResolvedDispatch` containing:

```text
callable
signature
visited_owners
```

Do not discard the callable identity before call checking.

Add/use a checker-facing full-resolution API that returns the resolved callable and specialized signature. This is needed for:

- real call provenance;
- generic constraint ownership;
- constructor-vs-normal callable origin;
- dependency explanations;
- eliminating dummy callable/expression identities.

A compatibility wrapper returning only `DispatchResult` may remain temporarily for non-call consumers, but new call logic must use the full resolved form.

## 18.2 Exact dispatch + known return contract establishes a call result

After receiver substitution and `Self` specialization:

```text
exact resolved callable identity
+ concrete return contract
= Established call-result knowledge
```

Choose origin:

```text
constructor result -> ConstructorSemantics
trusted native     -> NativeSignature
ordinary callable -> CallableSignature
solved generic     -> GenericInference (with callable signature provenance)
```

This rule applies consistently to:

- non-generic method calls;
- generic method calls;
- getters;
- unary/binary operator dispatch;
- property getter dispatch;
- direct field reads from a known field storage contract;
- index getter dispatch;
- constructor calls.

Do not clone a signature's assumed/declared contract status into the call result unchanged.

Direct field surfaces are also contracts. A field read with a concrete canonical field contract derives a fresh established expression fact from that storage contract; a field write checks actual knowledge against the field contract using the knowledge-to-contract relation API. Do not use the field contract object itself as if it were the current assigned value fact.

## 18.3 Argument refutation does not erase an independent result contract

Call validity and call-result type knowledge are separate outputs.

A call checker should conceptually return:

```rust
pub struct CallCheckResult {
    pub result: TypeKnowledge,
    pub status: AnalysisStatus,
}
```

Additional match/provenance fields are acceptable.

For a call with one or more argument diagnostics, allocate one deterministic **call-check cause** for the invalid invocation and attach it as `root_cause` to the call's argument-mismatch diagnostics. `AnalysisStatus::Invalid` references that call-check cause. Individual child argument expressions remain independently analyzed; do not derive their invalidity from the parent's cause.

If an argument is refuted but dispatch and return contract remain established:

```text
result = Established(return type)
status = Invalid(argument mismatch cause)
```

Do not turn the result into `Unknown` merely to signal the diagnostic.

## 18.4 Malformed call shape is different from a bad argument type

A wrong argument type still invokes an exactly identified callable contract and therefore does not destroy independently known return information.

A malformed selector/argument pack for which the checker has not established a valid callable mapping is different. Do not derive the return from a signature that the source invocation did not soundly match.

This distinction is critical.

---

# 19. Deterministic argument-to-parameter matching before generic inference

Modify:

```text
phalcom-semantic/src/checker/call.rs
```

Introduce a dedicated match phase before generating inference constraints.

Recommended internal result:

```rust
pub struct MatchedArgument {
    pub argument_index: u16,
    pub parameter_index: u16,
    pub expression_id: ExpressionId,
}

pub enum CallShapeState {
    Exact,
    DynamicPack,
    Invalid,
}

pub struct ArgumentMatch {
    pub state: CallShapeState,
    pub matches: Vec<MatchedArgument>,
    // compact structured issues as required
}
```

Implementation does not need a graph algorithm. Use deterministic linear/indexed matching:

- positional cursor for positional parameters;
- precomputed `HashMap`/`BTreeMap` from external label to parameter index for labelled parameters;
- consumed-parameter bitset/`Vec<bool>`;
- explicit rest-parameter handling only where current language semantics are already implemented and understood.

Complexity should be `O(arguments + parameters)` apart from small selector normalization costs.

### Fail-closed pack rule for Part 1

For dynamic labels or expansion packs whose complete static mapping is not currently implemented, do not approximate them as positional arguments merely to make dispatch/inference succeed.

Return `DynamicPack`/blocked/dynamic semantics as appropriate and preserve analyzed child expressions. Full rest/pack completeness belongs later.

### Inference constraints are generated only from matched pairs

No unmatched argument may add a generic constraint to an arbitrary parameter.

No missing parameter gets a fake `Unit` constraint.

---

# 20. Generic inference soundness subset

Modify:

```text
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/call.rs
```

This section does **not** attempt to finish all generic/HKT semantics. It removes currently unsound shortcuts from paths that already publish formal results.

## 20.1 Instantiate inference variables at their real kinds

Current:

```rust
let var = self.fresh_variable(KindId::TYPE);
```

Target:

```rust
let kind = store.type_parameter(param).kind;
let var = self.fresh_variable(kind);
```

Change `instantiate_generic_signature()` to accept the store or a kind lookup.

Do not default a higher-kinded parameter to `Type`.

## 20.2 Enforce kind compatibility when binding a solver variable

`InferenceVariable.kind` must be operational, not documentation.

When binding an inference variable to a canonical type/form:

```text
candidate kind == variable kind -> continue
otherwise -> InferenceFailureReason::KindMismatch
```

Use `TypeStore::kind_of(...)` or the canonical existing store API.

Change `bind()` from a lossy `bool` if necessary so callers can retain the actual failure reason.

## 20.3 Eliminate `SelfType -> Unit`

Change `type_term_to_inference()` to require a self-specialization context or return a structured conversion error.

Conceptually:

```rust
pub fn type_term_to_inference(
    &self,
    term: &TypeTerm,
    subst: &HashMap<TypeParameterId, InferenceTerm>,
    self_type: Option<TypeId>,
    store: &TypeStore,
) -> Result<InferenceTerm, InferenceBuildError>;
```

`TypeTerm::SelfType(_)`:

- specialize against the resolved receiver when available;
- otherwise return `UnresolvedSelf`, blocked/underconstrained;
- never return `Unit`.

## 20.4 Feed declared generic constraints into the solver

`GenericSignature.constraints` is already canonical data. For every constraint:

```text
GenericConstraint::Subtype { lower, upper }
    -> InferenceRelation::Subtype(convert(lower), convert(upper))

GenericConstraint::Equivalent { left, right }
    -> InferenceRelation::Equivalent(convert(left), convert(right))
```

Record:

```rust
ConstraintOrigin::GenericWhere {
    callable,
    constraint_index,
}
```

Do this before solving.

If a constraint cannot be converted because of unresolved `Self` or another blocked premise, the inference result must be blocked/underconstrained rather than silently ignoring the constraint.

## 20.5 Use real expression identities for constraints

Current generic calls fabricate:

```text
BodyId(0), LocalExpressionId(0)
```

Pass the actual call `ExpressionId` into call resolution. When analyzing each argument, retain its real `ExpressionId` and use it in `ConstraintOrigin::Argument`.

A practical refactor is:

```rust
fn analyze_expression_inner(
    ctx: &mut CheckingContext<'_>,
    expression_id: ExpressionId,
    expr: &Expr,
    expected: &ExpectedType,
) -> TypedExpression;
```

and/or an internal helper returning `(ExpressionId, TypedExpression)` for child analysis.

Do not expose fake IDs merely to satisfy the provenance type.

## 20.6 Do not use missing parameter types as `Unit`

If a generic parameter contract itself is unknown, skip that constraint and mark the inference problem underconstrained/blocked as required.

`Unit` is not an inference wildcard.

## 20.7 Solver terminal states are terminal for specialization

Required call-result behavior:

```text
Solved            -> materialize specialization, establish result
Underconstrained  -> Unknown(UnderconstrainedTypeVariable)
Conflicting       -> diagnostic + invalid/unknown specialized result
Blocked           -> blocked/unknown result; do not clone unspecialized return
Cancelled         -> propagate cancelled status
BudgetExceeded    -> propagate budget status
```

Do not fall back from a failed specialization to `signature.return_type.clone()`.

A raw generic return containing unspecialized parameters is not a concrete call-site result.

## 20.8 Hard-coded solver iteration count is not a proof mechanism

The current fixed 16-pass loop is acceptable in Part 1 only if exhausting it fails closed as `Underconstrained`/`Blocked` and never publishes an arbitrary substitution.

Do not “fix” nonconvergence by choosing bounds or `Object`/`Unit` defaults.

A later generic-completeness/performance stage may replace the solver strategy; Part 1 only requires sound terminal behavior.

---

# 21. Strict Unknown and sentinel audit

This is a named implementation task, not a cleanup suggestion.

Search all formal checker code for real type constructors used as fallback values:

```bash
rg 'store\.unit\(\)|store\.never\(\)|unwrap_or.*unit|unwrap_or.*never' phalcom-semantic/src/checker phalcom-semantic/src/types
```

Classify every occurrence as one of:

```text
LANGUAGE SEMANTICS
    the construct genuinely has Unit/Never/bottom semantics

TYPE-THEORETIC CONSTRUCTION
    e.g. an empty union/list element genuinely uses bottom by rule

ILLEGAL SENTINEL
    the type stands for missing information and must be replaced by Unknown/Blocked/underconstrained state
```

Known illegal baseline examples include:

```text
TypeTerm::SelfType -> Unit
missing generic parameter type -> Unit
unknown record field -> Unit
missing generic argument padding -> Never
```

## 21.1 Composite synthesis with unknown components

A canonical `TypeId` cannot express “tuple/record with one unknown slot” unless the type model explicitly supports such a component.

Therefore, when a composite requires a component type and that component remains unknown:

- do not substitute `Unit`;
- do not drop the component and build a shorter product;
- return an honest unknown/blocked composite result unless a sound expected-type rule establishes the component.

This applies to current tuple/record/collection helper paths.

Part 1 does not need to invent partial type terms for every product. Fail closed.

## 21.2 Remove the generic-argument iteration heuristic

`statement.rs::resolve_iteration_element()` currently treats the first argument of an arbitrary applied generic type as the iteration element when the origin has generic parameters.

That is not protocol evidence.

Remove this fallback in Part 1. Keep only formally resolved iteration protocol evidence already available. If the protocol exists but the element result cannot be established, return an honest unknown/blocked state.

Full iteration protocol completeness is later work; unsound first-generic-argument inference is not allowed to survive until then.

---

# 22. Expression/control-flow synthesis audit required in Part 1

Part 1 is not semantic coverage closure, but existing implemented arms must stop fabricating facts.

At minimum audit:

```text
Expr::IfLet result combination
block parameter contextual typing
list/set/map literal element synthesis
tuple/record literal synthesis
method/unqualified calls
binary/unary operator calls
property/index getters
property/index setters
assignment expressions
class/type-form references
```

Specific baseline defect:

```text
IfLet known branch + unknown branch
```

must not produce a `Proven` known branch type by choosing whichever branch has a `TypeId`. Use the common epistemic join.

Class/type-form resolution is formal name/declaration semantics and should be `Established(DeclarationSemantics)`, not “Declared value evidence.”

---

# 23. Explanation graph migration

Modify:

```text
phalcom-semantic/src/explain/node.rs
phalcom-semantic/src/explain/arena.rs (if constructor signatures change)
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
```

Replace:

```rust
pub authority: EvidenceAuthority
```

with explicit semantic fields, conceptually:

```rust
pub status: EvidenceStatus,
pub origin: EvidenceOrigin,
```

or a `TypeEvidenceDescriptor` that carries the same information without a `TypeId` duplicate.

## 23.1 Explanations cannot strengthen their subject

When recording an explanation for a concrete expression, take status/origin from the actual `TypeEvidence`.

Do not hardcode `Established`/`Proven` merely because `knowledge.ty()` returned `Some`.

## 23.2 Record contract relations as relations

Use existing `ExplanationStep::Declared` / `Subtyping` or add a focused binding-contract step so explanations can preserve both:

```text
actual/current type
contract type
relation outcome
```

Do not encode “annotation won” as the explanation for a compatible or refuted binding.

## 23.3 Constructor/call explanations use the real resolved callable

The current expression wrapper synthesizes a callable identity from the current class and method spelling for explanation. Once full resolved dispatch is retained, use the actual `CallableId` returned by dispatch.

This is required before Part 2 exposes explanation data to LSP presentation.

---

# 24. Semantic product fingerprint updates

Modify:

```text
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/tests/semantic_fingerprints.rs
phalcom-semantic/tests/product_stability_invalidation.rs
```

Step 5.5's dependency graph assumes product fingerprints encode every semantic property that can affect a dependent query.

## 24.1 Hash the new formal evidence semantics

`hash_type_knowledge()` product mode must hash:

```text
Known:
    type ID
    EvidenceStatus
    EvidenceOrigin

Unknown:
    reason

Dynamic:
    reason
```

Source ranges/descriptive provenance remain excluded from semantic product fingerprints when `include_provenance == false`.

Changing `Established(Int)` to `Assumed(Int)` is a semantic change and must alter the product fingerprint.

Changing only the source range of the same evidence should not invalidate semantic consumers through a range-insensitive product fingerprint.

## 24.2 Hash binding contract semantics

Callable-body product fingerprints must include:

```text
binding ID
contract presence
contract type
contract origin
current TypeKnowledge
current denotation if semantically relevant to expression resolution
BindingConsistency semantic payload
mutable
version where version remains an observable dependency input
```

Do not hash the source range inside `BindingContract` in the range-insensitive product fingerprint.

## 24.3 Diagnostic cause IDs are not semantic type meaning

`DiagnosticCauseId` is local causal identity. Do not let allocator-order changes force dependent semantic recomputation if the actual type/status/relation did not change.

For an `AnalysisStatus::Invalid`, product hashing should encode “invalid” and any semantically relevant invalidity class, not an incidental raw cause number.

Likewise `BindingState.invalid_cause` should not change a body product fingerprint solely because the local cause ID was renumbered.

Diagnostics themselves retain root causes for presentation/suppression.

## 24.4 Flow summaries must not erase epistemic state

`FlowStateSummary` currently stores only `BindingId -> TypeId` for known bindings.

If the summary participates in semantic products, replace it with a compact representation that preserves at least established-vs-assumed-vs-dynamic/unknown distinctions needed by consumers, or explicitly prove that the summary is presentation-only and remove it from semantic fingerprinting.

Do not hash a summary that says `Int` while the actual binding changed from Established(Int) to Assumed(Int) and expect dependency reuse to remain sound.

## 24.5 Preserve Step 5.5 dependency-graph ownership

Part 1 changes the semantic payload of callable-body products; it does not introduce a second dependency graph or per-expression query cache. Local binding/flow transfers remain inside the owning `CallableBody` query.

Every formal result derived from external semantic products must continue to record the existing query dependencies that justify it:

```text
resolved callable result
    -> resolved callable signature dependency
    -> every declaration surface/hierarchy owner traversed by dispatch

generic constraint consumed from callable signature
    -> same callable-signature dependency; no hidden side table

type/declaration resolution used by binding contract
    -> existing declaration shell/interface dependencies

native result
    -> canonical native/declaration signature product already participating in the semantic snapshot/query inputs
```

The baseline `CheckingContext::resolve_dispatch()` already records visited declaration-surface dependencies and consumed callable signatures. The refactor that retains full `ResolvedDispatch` identity must preserve that behavior exactly; do not bypass it by calling the raw resolver directly from expression code.

Add a regression in `checker_dependency_tracking.rs`: refactoring non-generic/generic call-result promotion must not reduce the dependency set compared with the baseline exact dispatch path. A caller that consumes a changed callable signature must still be invalidated; unrelated callers must remain reusable.

---

# 25. File-level implementation map

This is the minimum expected change surface. The implementing agent should re-ground exact line ranges before editing.

## Create

```text
phalcom-semantic/src/checker/binding.rs
```

Responsibilities:

- `BindingContract` / origin;
- `BindingConsistency` / assumption basis;
- pure reconciliation helpers;
- no LSP code;
- no diagnostic rendering;
- no AST traversal.

Optionally create a very small focused helper module for expression/call status if the existing files become unwieldy, but do not fragment the checker into one-file-per-enum.

## Modify — formal knowledge and relations

```text
phalcom-semantic/src/types/evidence.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/outcome.rs       # only if precise reasons are added
phalcom-semantic/src/types/denotation.rs    # only if fact APIs need adjustment
```

## Modify — checker state and transfer functions

```text
phalcom-semantic/src/checker/mod.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/typed_expr.rs
phalcom-semantic/src/checker/expected.rs
phalcom-semantic/src/checker/policy.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/flow/state.rs
```

## Modify — dispatch/signature integration only as needed

```text
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/types/native.rs
```

Do not redesign declaration surfaces wholesale in Part 1. If `dispatch::CallableSignature` continues temporarily to use `TypeKnowledge` as its contract payload, enforce the following transitional invariant:

> A callable signature's `Known` parameter/return entries are contract descriptions. Checker consumers must convert them into `ExpectedType` for checking or into newly **Established** call-result knowledge after exact dispatch; they must never treat the signature object's evidence status as the runtime/current value status by direct cloning.

The long-term canonical signature type is already `CallableSemanticSignature` with `TypeTerm`; Parts 2/3 can reduce transitional duplication after the formal checker is sound.

## Modify — explanations/fingerprints/presentation compatibility

```text
phalcom-semantic/src/explain/node.rs
phalcom-semantic/src/explain/arena.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/lib.rs               # public reexports if needed
```

`presentation.rs` only needs to understand the new formal status in Part 1. Publishing richer indices is Part 2.

## Tests — create/port

Recommended focused files:

```text
phalcom-semantic/tests/semantic_knowledge_invariants.rs
phalcom-semantic/tests/binding_contract_semantics.rs
phalcom-semantic/tests/flow_epistemics.rs
phalcom-semantic/tests/inference_soundness.rs
```

Extend existing:

```text
phalcom-semantic/tests/spec04_5_causal_suppression.rs
phalcom-semantic/tests/spec04_5_bidirectional_and_calls.rs
phalcom-semantic/tests/spec04_5_inference_session.rs
phalcom-semantic/tests/spec04_5_flow_graph.rs
phalcom-semantic/tests/semantic_fingerprints.rs
phalcom-semantic/tests/product_stability_invalidation.rs
phalcom-semantic/tests/checker_dependency_tracking.rs
```

Port the semantic intent from:

```text
tests/semantic-authority-composition:phalcom-semantic/tests/semantic_authority_composition.rs
```

Do not copy its old `EvidenceAuthority` assertions literally.

---

# 26. Implementation task sequence

Each task should land with tests. Do not perform a flag-day rewrite without intermediate compile/test checkpoints.

## Task 1 — Freeze regressions before changing representation

Add failing end-to-end semantic tests for the required examples in §28 below, using the current public analysis API where possible.

The initial tests may refer to old fields temporarily, but commit the behavioral source programs and expected type relations first. Representation assertions should be updated in Task 2 when the new model lands.

Run the new tests individually and confirm they fail for the expected reason, not because of parsing/module setup.

## Task 2 — Introduce origin/status and migrate formal construction

Implement `EvidenceStatus`, `EvidenceOrigin`, controlled constructors and preservation helpers.

Migrate all `phalcom-semantic/src` uses of `EvidenceAuthority`.

At the end of this task:

```bash
rg 'EvidenceAuthority' phalcom-semantic/src
```

must return no production hits.

Tests may retain temporary compatibility assertions only until the same change set updates them.

## Task 3 — Introduce binding contracts/reconciliation and remove `LocalEnv`

Create `checker/binding.rs`, evolve `BindingState`, make `FlowState` the sole owner of current fact/denotation, replace scope metadata, and remove `bind_local()`. Make declaration insertion return an explicit redeclaration result rather than overwriting the scope map.

Do not yet fix every call/generic path in the same task.

Run focused binding/context tests plus full semantic compile tests.

## Task 4 — Repair initializer and assignment transfer functions

Route `Statement::Let` and local reassignment through pure reconciliation. Honor `BindingKind`, const initialization, implicit-`None`/fail-closed missing initializer semantics, same-scope redeclaration and immutable assignment.

Add explicit cause allocation/diagnostic ownership for binding and assignment mismatches.

Verify compatible-supertype and contradiction regressions.

## Task 5 — Make expression status explicit and causal

Add explicit `TypedExpression` status, remove range-scanning invalidity, update causal suppression tests, preserve known results under invalid relation contexts.

This task is a prerequisite for call mismatch changes.

## Task 6 — Replace flow join/widening with epistemic algebra

Implement one shared knowledge join; merge denotations conservatively; intersect binding IDs; enforce contract invariance; recompute contract consistency; remove declaration-as-widening shortcut.

Add direct unit tests against `FlowState`, not only source-program tests.

## Task 7 — Replace bare expected-type conversion

Introduce `ExpectationOrigin`, delete `ExpectedType::from_knowledge`, migrate initializer/assignment/return/argument/collection/block call sites.

Run bidirectional tests and add “expected type does not fabricate actual” regressions.

## Task 8 — Standardize exact dispatch result promotion and call shape matching

Retain resolved callable identity, introduce `CallCheckResult`, implement deterministic argument matching, ensure invalid argument types do not erase independent return knowledge, and ensure malformed packs do not manufacture return facts.

Migrate binary/unary/getter/index call-result paths to the same contract-to-fact rule.

## Task 9 — Harden generic inference

Implement real parameter kinds, kind checking, `Self` conversion without `Unit`, generic where constraints, real expression IDs and safe terminal outcomes.

Run solver unit tests plus source-level generic call tests.

## Task 10 — Perform Unknown/sentinel and existing-expression soundness audit

Remove illegal `Unit`/`Never` fallbacks, fix composite synthesis with unknown components, remove arbitrary first-generic-argument iteration inference, fix branch known+unknown promotion, and classify every remaining `UncheckedExpression` path as fail-closed.

This is not optional merely because broader completeness comes later.

## Task 11 — Migrate explanations and fingerprints

Make explanations status/origin preserving; hash new semantic fields; remove cause-number sensitivity from semantic product identity; upgrade flow summaries or remove their epistemically lossy fingerprint role.

Add product-stability tests.

## Task 12 — Run the semantic epistemic-conflation audit and release gate

Run all search gates in §30, inspect every surviving hit manually, run all tests, then run cold/incremental regression checks.

Do not start Part 2 until this task passes.

---

# 27. Required helper/API behavior in detail

This section exists to prevent technically compiling but semantically weak implementations.

## 27.1 Type transformation preserves epistemics

Substitution and `Self` specialization commonly change only `TypeId`.

Do this:

```rust
let specialized = knowledge.map_type(|ty| subst.apply(store, ty));
```

not this:

```rust
TypeKnowledge::established(subst.apply(store, old_ty), EvidenceOrigin::GenericInference)
```

unless the transformation itself genuinely changes the derivation status/origin.

A substitution operation must not upgrade `Assumed` to `Established` merely because it materialized a `TypeId`.

## 27.2 Call-result promotion is explicit

Conversely, exact call resolution **does** represent a new semantic derivation. Use a dedicated helper rather than `map_type()`:

```rust
fn establish_call_result(
    specialized_return_contract: &TypeKnowledge,
    origin: EvidenceOrigin,
    range: SourceRange,
) -> TypeKnowledge;
```

If the return contract is concrete, produce established result knowledge. If it is unknown/dynamic, preserve the non-concrete state appropriately.

This avoids conflating “transform existing knowledge” with “derive a new fact from a contract.”

## 27.3 Relation proof does not rewrite actual knowledge

A successful subtype proof:

```text
Int <: Number
```

updates `BindingConsistency`/explanation. It does not widen current `Int` to `Number`.

A refutation similarly does not replace `Int` with `Number` or `Unknown`.

## 27.4 Expected context cannot rescue a coverage gap

For an expression that returns `Unknown(UncheckedExpression)`:

```phalcom
let x: Int = <unchecked-expression>
```

must leave the value current unknown/blocked and retain the `Int` contract separately.

No declaration assumption is allowed because the reason is a checker implementation gap.

## 27.5 Advisory cannot accidentally reappear through compatibility code

During Part 1, `phalcom-lsp` may require API adapters because `EvidenceAuthority` was public.

Adapters must translate **formal** status/presentation only. Do not map LSP `Confidence`, `ValueShape`, or old advisory facts into `TypeKnowledge::Assumed` as a convenience.

That would defeat the Part 2 boundary.

---

# 28. Mandatory semantic regression matrix

These are release-gate tests, not illustrative examples.

For every positive concrete fact, assert at least:

```text
type
EvidenceStatus
EvidenceOrigin where semantically stable
binding contract/contract origin where relevant
BindingConsistency where relevant
owning diagnostic count/code where relevant
callable dependency where relevant
expression AnalysisStatus where relevant
```

## 28.1 Compatible broad annotation preserves precise fact

```phalcom
class Base {}
class Derived is Base {
  @constructor new() {}
  derivedOnly() -> Int { 1 }
}
class Probe {
  @class run() {
    let x: Base = Derived.new()
    let y = x.derivedOnly()
  }
}
```

Assert:

```text
x contract type/origin = Base / SourceAnnotation
x current              = Derived / Established
x consistency          = Validated
x.derivedOnly resolves against Derived
no BindingInitializerMismatch
```

## 28.2 Refuted annotation preserves actual fact and downstream dispatch

```phalcom
class CellNum {
  @constructor new() {}
  cellOnly() -> Int { 1 }
}
class Probe {
  @class run() {
    let x: Int = CellNum.new()
    let y = x.cellOnly()
  }
}
```

Assert:

```text
x contract = Int / SourceAnnotation
x current  = CellNum / Established / ConstructorSemantics
x consistency = Refuted
x invalid_cause is present
exactly one binding-initializer mismatch
initializer expression remains Ready + Established(CellNum)
x.cellOnly remains semantically analyzable and returns Established(Int)
```

## 28.3 Literal contradiction

```phalcom
let x: String = 42
```

Assert `current = Established(Int, Syntax)`, contract `String`, refuted relation, no current replacement.

## 28.4 Genuine unknown permits assumption

Use an untyped callable parameter or another explicitly legitimate no-evidence producer:

```phalcom
run(value) {
  let x: Int = value
}
```

Assert:

```text
parameter use = Unknown(NoTypeEvidence)
x current = Int / Assumed / DeveloperAnnotation
x consistency = Assumed(MissingValueEvidence(...))
no mismatch diagnostic
```

## 28.5 Checker coverage gap cannot be hidden by annotation

Construct the smallest parser-supported expression currently routed to `UncheckedExpression`.

```phalcom
let x: Int = <coverage-gap-expression>
```

Assert:

```text
initializer = Unknown(UncheckedExpression)
x current remains Unknown
x contract remains Int
x consistency = Blocked/Unverified-ineligible, not Assumed
```

The test may need updating when later completeness implements that AST arm; when that happens replace it with another explicit internal coverage-unit test so the assumption classifier remains covered.

## 28.6 Dynamic remains dynamic under annotation

For a current dynamic-producing path:

```text
contract = CellNum
actual = Dynamic(...)
```

Assert:

```text
current = Dynamic
consistency = DynamicBoundary
not Assumed(CellNum)
```

## 28.7 Assignment checks persistent explicit contract

```phalcom
let x: Number = 1
x = Float.new()
```

Assert `Float <: Number` succeeds and final current is `Established(Float)`.

## 28.8 Assignment checks persistent inferred contract

```phalcom
let x = 1
x = "bad"
```

Assert:

```text
contract = Int / InferredInitializer
initial current = Established(Int)
write current = Established(String)
write relation = Refuted(String <: Int)
explicit declared type = None
```

## 28.9 Invalid reassignment retains actual current for recovery

After the invalid write above, a downstream operation specific to `String` should resolve from current `String` if independently valid, while the assignment diagnostic remains.

## 28.10 Flow established + established

Direct `FlowState` unit test:

```text
branch A current = Established(Int)
branch B current = Established(Float)
join current = Established(Int | Float), origin Flow
```

## 28.11 Flow established + assumed

Assert joined type is union and status is `Assumed`, not `Established`.

## 28.12 Flow established + unknown

Assert merged current is `Unknown`; no sample fallback.

## 28.13 Flow denotation disagreement

One branch type-form denotation, one ordinary/no denotation -> joined denotation `None`.

## 28.14 Loop widening does not replace with declaration

Construct header/current values that change under a `Number` contract. Assert widening produces the knowledge join or explicit conservative unknown, never `Assumed/Established(Number)` solely because the contract exists.

## 28.15 Expected type does not overwrite an actual literal

Check `42` under expected `Number`. Expression remains `Established(Int, Syntax)` while relation to `Number` is proven.

## 28.16 Contextual block parameter is assumed, not syntax-established

Analyze a block under a callable expectation. Assert parameter current status is `Assumed`, origin contextual, contract origin contextual.

## 28.17 Constructor result established

Port branch test for `CellNum.new()` and assert constructor dependency plus `Established(CellNum, ConstructorSemantics)`.

## 28.18 Factory result propagation

A source factory whose inferred tail is `CellNum.new()` must provide an established call result at the caller, with the correct callable dependencies.

## 28.19 Bad argument preserves independent return

```phalcom
CellNum.fromInt("bad")
```

Assert:

```text
call result type = CellNum / Established
call status      = Invalid
exactly one ArgumentMismatch
argument child   = Established(String) and remains independently valid
```

## 28.20 Bad return annotation preserves tail fact

```phalcom
make() -> Int {
  CellNum.new()
}
```

Assert tail expression remains `Established(CellNum)` and one return mismatch is emitted.

## 28.21 Generic variable kind

Create a generic parameter with kind `Type -> Type`; instantiate inference and assert the solver variable records that exact kind rather than `Type`.

## 28.22 Generic `Self` never becomes Unit

Exercise `TypeTerm::SelfType` conversion with and without receiver context:

```text
with receiver -> specialized receiver/self term
without receiver -> structured blocked/conversion error
never Unit
```

## 28.23 Generic where constraint participates

Create a call that would solve from arguments but violates a declared generic constraint. Assert inference is conflicting/refuted and no successful specialized result is published.

## 28.24 Underconstrained inference does not clone raw return contract

Assert an unsolved generic return remains `Unknown(UnderconstrainedTypeVariable)` rather than returning a type containing arbitrary/fake substitutions.

## 28.25 Product fingerprint changes with epistemic status

Same `TypeId`, change `Established` -> `Assumed`: product fingerprint must change.

## 28.26 Product fingerprint ignores cause-number renumbering

Semantically equivalent invalid analysis with a different local `DiagnosticCauseId` allocation must retain the same range-insensitive semantic product fingerprint, assuming all actual type/status/relation facts are equal.

## 28.27 Step 5.5 reuse regression remains green

Range-only body edit continues to reuse semantic callers when the callable body product remains semantically unchanged.

## 28.28 Binding kind controls mutability

Analyze one `let` and one `const` binding with otherwise identical initializers. Assert `BindingState.mutable == true` for `let` and `false` for `const`; no generic binding helper may flatten them.

## 28.29 Immutable assignment does not mutate recovery state

Attempt to reassign a `const`. Assert one immutable-assignment diagnostic, invalid assignment expression status, and unchanged binding `current`, `version`, contract and denotation.

## 28.30 Same-scope redeclaration preserves the first identity

Declare the same name twice in one scope. Assert one redeclaration diagnostic and that subsequent name resolution still targets the first binding. Add a nested-scope shadowing control case that remains legal.

## 28.31 Missing initializer is not generic no-evidence

For `const x` with no initializer, assert the canonical missing-initializer diagnostic and no annotation-backed assumption. For bare `let x`, assert either the canonical established `None` value if the current formal core surface supports it or the dedicated ineligible coverage/block reason chosen by this implementation—never `NoTypeEvidence`, `Unit` sentinel, or annotation laundering.

---

# 29. Tests that must be updated, not merely made to compile

Search for old authority assertions:

```bash
rg 'EvidenceAuthority|\.authority\(\)|authority:' phalcom-semantic/tests phalcom-semantic/src
```

Update tests semantically:

```text
EvidenceAuthority::ExactSyntax
    -> status Established + origin Syntax

EvidenceAuthority::Proven
    -> usually status Established + the actual derivation origin
       (Flow, CallableSignature, ConstructorSemantics, GenericInference, ...)

EvidenceAuthority::Declared on parameter/current fallback
    -> status Assumed + contract/developer origin where it is current value knowledge

EvidenceAuthority::Declared on a contract
    -> stop representing the contract as value knowledge; assert BindingContract/ExpectedType instead

EvidenceAuthority::TrustedNative
    -> status Established/contract-derived as appropriate + NativeSignature origin

EvidenceAuthority::Advisory
    -> no formal replacement in Part 1; advisory tests remain in LSP until Part 2
```

Do not mechanically map every `Declared` to `Assumed`. Some current `Declared` values are actually declaration/name facts that should become `Established(DeclarationSemantics)`, while others are contracts that should cease being `TypeKnowledge` at all.

This distinction is one purpose of the audit.

---

# 30. Semantic epistemic-conflation audit — mandatory final pass

After implementation, run:

```bash
rg 'EvidenceAuthority' phalcom-semantic/src
rg 'TypeKnowledge::known' phalcom-semantic/src
rg 'ExpectedType::from_knowledge' phalcom-semantic/src
rg '\bbind_local\(' phalcom-semantic/src
rg 'local_envs|LocalEnv' phalcom-semantic/src/checker
rg '\.declared\b' phalcom-semantic/src/checker
rg 'UncheckedExpression' phalcom-semantic/src
rg 'store\.unit\(\)|store\.never\(\)' phalcom-semantic/src/checker phalcom-semantic/src/types
rg 'TypeId::DUMMY' phalcom-semantic/src
rg 'DiagnosticCauseId\(.*expr|DiagnosticCauseId\(expr' phalcom-semantic/src/checker
```

Interpretation:

- `EvidenceAuthority`: zero production hits.
- `TypeKnowledge::known`: zero hits if the old unrestricted constructor is removed; every fact should choose `established` or `assumed`.
- `ExpectedType::from_knowledge`: zero hits.
- old conflating `bind_local`: zero hits.
- `LocalEnv`: zero checker-state ownership hits.
- `.declared`: any surviving field must be audited; there should be no old “current copied into declared” path.
- `UncheckedExpression`: may remain until semantic completeness, but every producer must be fail-closed and ineligible for declaration assumption.
- `Unit`/`Never`: every surviving hit must be classified and justified by a real language/type rule.
- `TypeId::DUMMY`: no user-facing assignability/refutation payloads.
- diagnostic causes: no expression-range/id heuristic allocation.

Additionally search for direct cloning of callable return contract into expression facts:

```bash
rg 'return_type\.clone\(\)|sig\.return_type|signature\.return_type' phalcom-semantic/src/checker
```

Every hit must be classified as:

```text
contract read
expected-type creation
explicit contract-to-fact promotion after exact dispatch
```

There must be no unreviewed “clone contract into current result” path.

---

# 31. Performance/data-structure constraints

Part 1 is a semantic-correctness patch, not permission to regress Step 5.5 architecture.

## 31.1 Hot facts stay compact

Do not attach unbounded proof trees to `TypeKnowledge` or `BindingState`.

Use:

- compact enum status/origin;
- existing bounded provenance representation;
- `ExplanationArena` IDs for rich derivations.

## 31.2 Flow joins are deterministic

Use canonical `TypeStore::union(...)` and deterministic binding iteration (`BTreeMap` already exists in `FlowState`).

Do not introduce hash-order-dependent union or diagnostic behavior.

## 31.3 Argument matching is linear/indexed

Do not solve label matching by nested search inside every generic inference pass. Build the argument-to-parameter mapping once, then generate constraints from the mapping.

## 31.4 No clone-heavy “safety” workaround

Do not solve ownership friction by cloning the whole `TypeStore`, dispatch surface, or flow state on every expression. The existing checker deliberately borrows the workspace dispatch and detaches only on mutation; preserve that architecture.

## 31.5 Product fingerprints must track semantic—not incidental—changes

Adding evidence status/origin must not cause range-only edits to invalidate semantic callers. Keep source/provenance input identity separate from product semantic identity, matching Step 5.5's existing design.

---

# 32. Explicit non-goals for Part 1

Do not expand this specification into the later completeness/takeover projects.

Part 1 does **not** require:

- exhaustive implementation of every `Expr` AST variant;
- exhaustive statement handling;
- recursive tuple/list/record/map/variant pattern binding;
- type alias completeness;
- module-level global value closure;
- complete rest-pack and dynamic-label call semantics;
- final HKT/generic feature breadth;
- final iteration protocol implementation;
- compiler-owned advisory `ValueShape` migration;
- canonical LSP occurrence/binding identity takeover;
- publishing `SemanticPresentationIndex` in the workspace snapshot;
- persistent `ProjectUniverse`/module lifecycle takeover;
- removal of `phalcom-lsp/src/semantic`;
- LSP consumer migration.

However, an existing incomplete path covered by one of these non-goals must be **honest** after Part 1. It may return Unknown/Blocked/Dynamic with a precise reason. It may not fabricate an established type to look complete.

---

# 33. Commit/review decomposition

A practical reviewable commit sequence is:

```text
1. test(semantic): freeze epistemic correctness regressions
2. refactor(semantic): split evidence origin from epistemic status
3. refactor(semantic): introduce binding contracts and single current-state owner
4. fix(semantic): preserve actual knowledge across binding and assignment checks
5. fix(semantic): make expression invalidity causal and knowledge-preserving
6. fix(semantic): make flow joins and widening epistemically monotone
7. refactor(semantic): make expected types explicit contextual constraints
8. fix(semantic): establish exact dispatch results without contract cloning
9. fix(semantic): harden generic inference kinds constraints and self handling
10. fix(semantic): remove unknown sentinel types and fail closed on coverage gaps
11. fix(semantic): preserve epistemics in explanations and fingerprints
12. test(semantic): complete epistemic audit and incrementality regressions
```

Do not merge a commit in this chain if `cargo test -p phalcom-semantic` is red unless the branch is explicitly being used as a short-lived stacked implementation branch and the final PR review can still inspect coherent commits. Prefer every semantic behavior commit to be independently green.

---

# 34. Verification commands

At minimum after each major slice:

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
```

Before Part 1 completion:

```bash
cargo fmt --check
cargo clippy -p phalcom-semantic --all-targets -- -D warnings
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp
```

If workspace CI uses broader commands, run the repository-standard workspace checks as well.

Run focused gates explicitly so a broad test failure cannot hide which invariant regressed:

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic_knowledge_invariants
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test binding_contract_semantics
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test flow_epistemics
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test inference_soundness
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic_fingerprints
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test product_stability_invalidation
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test checker_dependency_tracking
```

If exact test filenames differ because the implementation agent sensibly consolidates them, retain equivalent focused command coverage in the PR description.

---

# 35. Part 1 completion gate

Part 1 is complete only when all statements below are true in code and tests.

1. There is no `EvidenceAuthority` in `phalcom-semantic/src`.
2. Formal concrete knowledge explicitly distinguishes `Established` from `Assumed`.
3. Advisory evidence cannot enter formal `TypeKnowledge`.
4. A binding has one persistent contract representation whose origin distinguishes explicit and inferred contracts.
5. `FlowState` is the single owner of current local binding knowledge/denotation; no parallel `LocalEnv` current-fact store exists.
6. `BindingKind::Let/Const` controls mutability; same-scope redeclaration cannot overwrite the first binding; `const` without an initializer is diagnosed; bare `let` without an initializer is never misclassified as generic no-evidence.
7. Compatible annotations preserve narrower checker-established current facts.
8. Refuted annotations preserve actual current facts and explicit contradiction state.
9. Genuine no-evidence cases may receive explicit assumptions from contracts.
10. Checker coverage gaps, unresolved names, blocked inference and invalid dependencies cannot be laundered into assumptions.
11. Reassignment validates against the persistent binding contract rather than the previous current fact.
12. Invalid writes preserve the actual new current knowledge for recovery and retain an owning invalid cause.
13. Flow join never chooses an arbitrary known branch when another reachable branch is unknown/weaker.
14. Flow joins involving assumptions remain at most assumed.
15. Loop widening never substitutes a declaration as current truth merely because the declaration exists.
16. Expected types retain contextual role and are never converted directly into fake current evidence.
17. Expression status is produced causally, not by diagnostic-range scanning.
18. Invalid expressions may retain independently established type knowledge.
19. Exact dispatch/call results are established by a uniform contract-to-fact rule.
20. Generic inference uses actual parameter kinds.
21. Generic `Self` never becomes `Unit` because receiver context is missing.
22. Generic declared constraints participate in inference.
23. Generic solver blocked/underconstrained/cancelled/budget states do not fall back to an unspecialized return contract.
24. No real type (`Unit`, `Never`, `Object`, etc.) remains as a missing-information sentinel.
25. The arbitrary first-generic-argument iteration heuristic is gone.
26. Explanations preserve the result's actual status/origin and real resolved callable identity.
27. Callable-body product fingerprints include all new semantically observable epistemic/contract state.
28. Cause-ID renumbering does not by itself alter semantic product identity.
29. Step 5.5 product-stability and dependency-invalidation tests remain green.
30. `phalcom-lsp` still compiles/tests against the changed formal API without importing advisory facts into formal knowledge.
31. The semantic epistemic-conflation audit in §30 has been manually reviewed, not merely executed.

Only after all 31 are true may implementation proceed to **Part 2 — Canonical Semantic Identity, Projection, and Advisory Evidence Takeover**.
---

# 36. Handoff contract to Part 2

Part 2 may assume the following stable seams from Part 1:

```text
TypeKnowledge
    contains only formal Established/Assumed/Unknown/Dynamic states

BindingState
    exposes persistent contract + current knowledge + consistency + causal invalidity

ExpectedType
    is contextual and non-evidentiary

relation layer
    returns explicit outcomes and supports knowledge-vs-contract checking

FlowState
    is the sole current binding-fact owner and has epistemically sound joins

TypedExpression / ExpressionAnalysis
    can be Invalid while retaining independently known type knowledge

call checker
    returns established results only through exact semantic derivation

UnknownReason
    distinguishes assumption-eligible no-evidence from fail-closed implementation/dependency failure

ExplanationArena
    preserves evidence status/origin rather than upgrading known facts

SemanticDb product fingerprints
    observe epistemic/contract semantic changes correctly
```

Part 2 must build compiler-owned advisory evidence **beside** these products, not by weakening them.

---

# 37. Specification verification record

This specification was reviewed against both the ratified semantic requirements and the baseline repository implementation before publication.

## 37.1 Ratified requirement coverage

| Requirement | Covered by |
|---|---|
| inferred does not mean advisory | §§3, 5 |
| declaration/current independence | §§3, 6, 8, 10 |
| compatible declaration preserves precision | §§8, 10, 28.1 |
| refuted declaration never replaces checker fact | §§8, 10, 28.2–28.3 |
| unknown may permit assumption | §§8, 28.4 |
| assumptions remain unverified | §§5–8, 16 |
| dynamic remains distinct | §§8, 28.6 |
| advisory cannot hard reject | §§3, 5, 27.5 |
| causal suppression, no fake types | §14 |
| expected type cannot fabricate fact | §16 |
| call/constructor exact results established | §18 |
| flow preserves epistemic distinction | §15 |
| Unknown cannot hide checker omission | §21, 28.5 |
| positive tests assert evidence/provenance | §28 |
| explanation preserves actual + contract | §23 |
| named epistemic-conflation audit | §30 |
| semantic correctness precedes completeness | §§1, 32, 35 |

## 37.2 Repository defect coverage

| Verified baseline defect | Required implementation |
|---|---|
| mixed `EvidenceAuthority` | §§5, 28 |
| annotated let overwrites initializer | §§8, 10 |
| `bind_local()` conflates contract/current | §§6–9 |
| duplicate `LocalEnv`/`FlowState` current facts | §7 |
| assignment checks prior current | §12 |
| flow sample fallback | §15 |
| declaration loop widening | §15.6 |
| `ExpectedType::from_knowledge` | §16 |
| range-based invalidity | §14 |
| explanation hardcodes `Proven` | §23 |
| parameters represented as exact current facts | §17 |
| generic/non-generic call authority inconsistency | §18 |
| generic kind hardcoded to `Type` | §20.1 |
| `SelfType -> Unit` | §20.3 |
| generic constraints ignored | §20.4 |
| dummy expression IDs | §20.5 |
| missing type -> Unit | §§20.6, 21 |
| solver failure returns signature result | §20.7 |
| record/generic sentinel types | §21 |
| first generic arg used as iteration proof | §21.2 |
| `UncheckedExpression` is generic unknown | §§8.1, 21 |
| old epistemics omitted from fingerprints | §24 |
| binding kind/mutability/missing-init/redeclaration ignored | §§11, 28.28–28.31 |

## 37.3 Scope-boundary verification

The specification was checked not to absorb Part 2/3 responsibilities prematurely.

It does not require migration of `phalcom-lsp::semantic::ValueShape`, LSP semantic IDs, compiler presentation-index publication, persistent module lifecycle, production LSP worker replacement, or deletion of the LSP semantic engine. It changes `phalcom-lsp` only as necessary to compile against the corrected formal API.

At the same time, it does not defer any known formal-semantic behavior that can currently fabricate a false type merely because that behavior also belongs to a later feature area. This is why generic soundness, sentinel removal, the iteration heuristic and existing branch/control-flow joins are included here.

## 37.4 Algorithm/data-structure verification

The selected designs were checked for the failure modes most likely to recur during implementation:

- no second “effective” binding type is introduced;
- no second local current-state map remains;
- contract consistency is recomputed from joined knowledge rather than maintained as a complex parallel lattice;
- flow joins use deterministic existing ordered maps/canonical unions;
- generic argument matching is separated from solving and can remain linear/indexed;
- inference variables use existing canonical kind metadata;
- rich proof detail remains arena-backed rather than bloating hot facts;
- diagnostic cause identity is separated from semantic product identity;
- Step 5.5 input/product fingerprint distinction is preserved;
- exact dispatch/generic-result refactors continue to capture callable-signature and traversed declaration-surface dependencies through the compiler query graph rather than hidden side tables.

This closes the major correctness/representation decisions needed for implementation of Part 1 without attempting to teach or redesign the entire type system.
