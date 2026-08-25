# Phalcom Semantic Correctness / Single-World Takeover — Part 1 Corrections and Amendments

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to apply these amendments task-by-task. This document is a companion to the saved WIP Part 1 specification; it does **not** replace or rewrite that file wholesale.

**Status:** Normative amendment to the WIP Part 1 implementation specification.

**Baseline reviewed:** `aureat/phalcom-lang` at `23e6ca126e96b11504a275fa3e777e18fe4d9ef5`.

**Companion spec:** `phalcom_semantic_correctness_single_world_takeover_part1_formal_epistemic_foundation_spec.md` (saved WIP copy).

**Goal:** Record the final correctness corrections discovered during verification without modifying the saved WIP document.

**Precedence rule:** Where this amendment conflicts with the WIP Part 1 specification, **this amendment wins**. Where it is silent, the WIP remains authoritative.

**Scope:** These amendments are intentionally narrow. They correct the generic-inference failure model, generic-result epistemic support/taint, suppression-cause representation, a few naming/consistency issues, and the corresponding tests/release gates. They do not pull Part 2 identity/advisory/LSP-takeover work into Part 1.

---

# 1. Why this amendment exists

The WIP Part 1 specification correctly establishes the primary semantic model:

- formal `TypeKnowledge` separates `Established`, `Assumed`, `Unknown`, and `Dynamic`;
- declarations/contracts do not overwrite stronger current knowledge;
- `BindingState` separates persistent contracts from current flow facts;
- relation outcomes remain explicit instead of collapsing to a boolean;
- `CausalInvalidity` is orthogonal to type knowledge and expression status;
- flow joins do not fabricate knowledge;
- expected types are context rather than evidence;
- exact call/constructor contracts can derive formal result facts;
- real language types such as `Unit` and `Never` are not missing-information sentinels.

The verification pass found one remaining high-risk area: the current generic solver still destroys evidence at precisely the point where Part 1 is intended to preserve it.

At the repository baseline, `phalcom-semantic/src/checker/inference.rs` does all of the following:

1. `bind()`, `unify_terms()`, and `subtype_terms()` communicate failure using `bool`;
2. when a constraint fails, `solve()` fabricates an `InferenceConflict` with `InferVarId::from_index(0)`;
3. the fabricated conflict also uses `store.never()` for both lower and upper bounds;
4. the caller cannot distinguish a real conflicting bound, a kind mismatch, an occurs-check failure, or a non-variable structural constraint failure from that fabricated record;
5. inference substitutions do not retain whether they were supported by checker-established evidence, developer-contract assumptions, or only contextual selection;
6. therefore a generic return can be incorrectly promoted to `Established` even when the type argument that determines it came only from assumed evidence;
7. conversely, an invalid argument can cause an implementation to throw away a result type that is actually independent of the invalid generic evidence.

Those issues are not presentation details. They affect the meaning of “checker established” and therefore must be corrected before Part 1 can be considered implementation-ready.

---

# 2. Amendment A — generic failures must preserve the actual failed judgment

## 2.1 Supersedes

This amends WIP §§20.2, 20.7, Task 9, and the generic-inference portions of the release gate.

## 2.2 Prohibited behavior

The implementation must not retain or reproduce this baseline pattern:

```rust
InferenceOutcome::Conflicting(InferenceConflict {
    var: InferVarId::from_index(0),
    reason: InferenceFailureReason::ConflictingBounds {
        lower: store.never(),
        upper: store.never(),
    },
})
```

unless variable zero and `Never`/`Never` are literally the real failing evidence, which should be extraordinarily rare.

`InferVarId(0)`, `Never`, `Unit`, `Object`, or any other canonical language type must never be used as an error-reporting placeholder.

## 2.3 Use a failure-carrying solver API

Boolean solver transitions are insufficient because they erase the reason and site of failure.

Recommended shape:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceFailure {
    OccursCheck {
        var: InferVarId,
    },
    KindMismatch {
        var: InferVarId,
        expected: KindId,
        actual: KindId,
    },
    ConflictingBounds {
        var: InferVarId,
        lower: TypeId,
        upper: TypeId,
    },
    UnsatisfiedConstraint {
        relation: InferenceRelation,
    },
    MaterializationBlocked {
        reason: BlockReason,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceConflict {
    pub constraint_index: Option<u32>,
    pub origin: ConstraintOrigin,
    pub failure: InferenceFailure,
}
```

The exact enum names may differ, but the following information is mandatory:

- the real constraint origin;
- the real variable when the failure belongs to a variable;
- the real conflicting bound types when known;
- a structural failure variant when no single inference variable owns the conflict;
- no fake canonical type IDs.

`constraint_index` may be `None` for failures discovered outside a stored constraint, such as final bound reconciliation. Do not invent an index.

## 2.4 Change lossy helpers from `bool` to `Result`

At minimum, revise these conceptual APIs:

```rust
fn bind(...) -> Result<BindEffect, InferenceFailure>;
fn unify_terms(...) -> Result<SolveEffect, InferenceFailure>;
fn subtype_terms(...) -> Result<SolveEffect, InferenceFailure>;
```

Suggested progress type:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SolveEffect {
    Unchanged,
    Changed,
}
```

A caller may choose a simpler `Result<(), InferenceFailure>` if it has another correct way to detect fixed-point progress. What is not acceptable is a `bool` whose `false` means several semantically different failures.

## 2.5 Keep variable state and terminal conflict consistent

If `InferenceVariable.state` remains:

```rust
pub enum InferVarState {
    Unsolved,
    Solved(TypeId),
    Failed(InferenceFailureReason),
}
```

then a variable-owned terminal failure must update the variable state with the same reason exposed by `InferenceOutcome::Conflicting`.

Do not publish one reason in the variable and another synthetic reason in the outcome.

For structural constraint failures with no unique variable, no arbitrary variable should be marked failed.

## 2.6 Kind checking uses existing canonical store data

The baseline `TypeStore` already exposes:

```rust
pub fn type_parameter(&self, id: TypeParameterId) -> &TypeParameterData;
pub fn kind_of(&self, ty: TypeId) -> KindId;
```

Use those APIs. Do not infer kind from `TypeData` by hand and do not default unknown higher-kinded parameters to `KindId::TYPE`.

## 2.7 Conflicting inference is not a syntax error

Do not map a generic inference contradiction to:

```rust
TypeKnowledge::Unknown(UnknownReason::SyntaxError)
```

A type-inference contradiction is not a parse/syntax failure.

Add a dedicated reason, for example:

```rust
UnknownReason::InferenceConflict
```

or preserve the conflict through a more specific formal analysis status if the implementation already has an appropriate carrier.

Required semantic shape when the result itself cannot be established:

```text
AnalysisStatus = Invalid(C)
TypeKnowledge  = Unknown(InferenceConflict)
Diagnostic     = ArgumentMismatch / TypeMismatch / constraint-specific diagnostic
root cause     = C
```

This rule is refined by Amendment C for result types that are independent of the failed generic evidence.

---

# 3. Amendment B — generic inference must track epistemic support, not just substitutions

## 3.1 Supersedes

This amends the WIP statement that a solved generic specialization may always be promoted directly to `Established`.

A solver can determine a `TypeId` without establishing that type with checker-independent evidence. The substitution and its epistemic support are different facts.

## 3.2 The required distinction

Consider:

```phalcom
class Box<T> {
  @class id(value: T) -> T { value }
}

run(value: Int) {
  let x = Box.id(value)
}
```

At callable entry, `value: Int` is a contract-backed assumption under the Part 1 model. If `T = Int` is inferred solely from that assumed parameter fact, then the call result `T` must not become checker-established merely because generic unification succeeded.

Required result:

```text
specialization T = Int
result type       = Int
result status     = Assumed
result origin     = GenericInference (with assumed support)
```

By contrast:

```phalcom
let x = Box.id(42)
```

may produce:

```text
specialization T = Int
support           = established literal evidence
result            = Established(Int)
```

This is the exact “inference is not authority” rule applied inside generic solving.

## 3.3 Do not attach epistemic status to `TypeId`

Keep canonical types canonical. Do not introduce “assumed TypeId” or duplicate type-store entries.

Track support in solver-local metadata.

Recommended compact representation:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InferenceSupport {
    Established,
    Assumed,
}

impl InferenceSupport {
    pub fn join(self, other: Self) -> Self {
        if matches!(self, Self::Assumed) || matches!(other, Self::Assumed) {
            Self::Assumed
        } else {
            Self::Established
        }
    }
}
```

Contextual generic selection is addressed separately in §3.7; it should not be encoded as “assumed runtime evidence.”

Maintain support at the union-find representative or equivalent canonical solver variable:

```rust
support: HashMap<InferVarId, InferenceSupport>
```

If the solver already stores variable records densely, a field on `InferenceVariable` is also acceptable.

Initialize fresh variables to `InferenceSupport::Established` **only as a weakening summary meaning “no assumed value premise has influenced this variable yet.”** This does not mean the variable is solved and does not by itself justify publishing a type. The support table is consulted for result classification only after `InferenceOutcome::Solved`. An unsolved variable remains underconstrained regardless of its default support value.

Thus a variable solved solely by a valid callable generic constraint or by an explicitly supported contextual-instantiation rule remains non-assumption-tainted and may contribute established support.

## 3.4 Seed support from actual evidence

When an argument generates a constraint, inspect the argument `TypeKnowledge`:

```text
Known(Established, ...) -> InferenceSupport::Established
Known(Assumed, ...)     -> InferenceSupport::Assumed
Unknown                 -> no usable type constraint; underconstrained/blocked path
Dynamic                 -> dynamic boundary; do not pretend it is formal generic evidence
```

A relation proof over an assumed premise does not upgrade that premise.

For example:

```text
Assumed(Derived) <: Base = proven relation
```

still contributes **assumed** support to an inference variable derived from that value.

## 3.5 Propagate support through aliases and compound constraints

The implementation does not need a second heavyweight dependency graph.

Use the existing solver structure plus a small variable collector:

```rust
fn collect_infer_vars(term: &InferenceTerm, out: &mut SmallVec<[InferVarId; 4]>);
```

`Vec<InferVarId>` is acceptable if `smallvec` is not already a dependency; do not add a crate solely for this.

When a constraint is added from evidence with support `S`:

1. collect every inference variable referenced by that constraint;
2. map each through `find_var()`;
3. join `S` into the representative's support;
4. when variable alias classes merge, join their support values;
5. if a variable becomes related to a compound term containing other variables, propagate/join support to all referenced representatives conservatively.

Because occurs-checking forbids recursive self-dependence, a conservative bounded propagation over the solver's existing passes is sufficient for Part 1. Do not build a general graph framework unless the existing solver genuinely requires one.

Sound over-taint is acceptable; unsound upgrading is not.

## 3.6 Return support is computed only from variables that influence the return

This is critical.

After solving, identify inference variables referenced by the callable's return inference term **before materialization**.

The result's epistemic status is determined by the support of those return-influencing variables, not by every argument in the call.

Example:

```phalcom
@map<T>(value: T) -> CellNum
```

Even if `T` is inferred from an assumed value, `T` does not occur in `CellNum`.

Therefore, if dispatch and the fixed `CellNum` return contract are valid:

```text
result = Established(CellNum)
```

The call may still carry causal invalidity or an invalid status for a bad argument. Result knowledge and call validity remain orthogonal.

For:

```phalcom
identity<T>(value: T) -> T
```

`T` occurs in the return; assumed support for `T` yields:

```text
result = Assumed(Int)
```

## 3.7 Expected-result inference is contextual selection, not runtime evidence

Expected types may participate in generic instantiation only through an explicit bidirectional typing rule.

Do not join `ExpectationOrigin` into `InferenceSupport::Assumed`.

Instead distinguish:

```text
value evidence support      -> Established / Assumed
contextual instantiation    -> selects/constrains type arguments
```

A contextual expected result may select a generic instantiation when the callable contract actually contains the relevant generic relationship.

Example:

```text
empty<T>() -> List<T>
expected: List<Int>
```

If Phalcom's bidirectional rule permits expected-result inference, the checker may select `T = Int`, then derive `List<Int>` from the now-concrete callable contract. That result may be established because the expectation selected an instantiation; it did not assert a runtime fact about an already-existing value.

However, the expected type must **not**:

- fill an unknown/missing return contract component;
- override an incompatible fixed return contract;
- rescue `Unknown(UncheckedExpression)`;
- turn a failed/blocked solver into `Solved` by fiat;
- suppress a real generic constraint conflict.

Implementation rule:

> Context may choose among valid instantiations. Context may not manufacture callable contract information that is absent or refuted.

## 3.8 Store enough support information in the solution

Revise the solver result conceptually from:

```rust
pub struct InferenceSolution {
    pub substitutions: HashMap<InferVarId, TypeId>,
}
```

to something that also exposes support:

```rust
pub struct InferenceSolution {
    pub substitutions: HashMap<InferVarId, TypeId>,
    pub support: HashMap<InferVarId, InferenceSupport>,
}
```

A denser vector keyed by `InferVarId` is preferable if the ID is guaranteed dense and this is already how variable storage works. Use the representation that fits the existing solver; the semantic requirement is that support survives solving.

No support map is published into `TypeStore` or snapshots as canonical type identity.

---

# 4. Amendment C — terminal generic failure may preserve only an inference-independent return

## 4.1 Why this is distinct from ordinary call invalidity

The WIP correctly states that a bad argument does not necessarily erase an independently known return type:

```phalcom
CellNum.fromInt("bad")
```

with exact signature:

```text
fromInt(Int) -> CellNum
```

can be:

```text
status = Invalid(C)
result = Established(CellNum)
```

Generic specialization adds a dependency question, but Part 1 must answer it without inventing a second proof/dependency graph inside `InferenceSession`.

## 4.2 Part 1 deliberately rejects partial specialization after terminal solver failure

Use this conservative rule:

```text
InferenceOutcome::Solved
    -> a generic return may be materialized and classified using InferenceSupport

InferenceOutcome::Conflicting / Blocked / Underconstrained
    -> do not publish a partially specialized return containing inference variables
    -> a fixed concrete return that contains no inference variables may still survive
```

This is sound and deliberately simpler than attempting to prove that one solved generic variable is transitively independent of another failed variable.

Do **not** add a generic-variable dependency graph merely to recover partial results in Part 1.

A later generic-completeness phase may introduce principled partial solutions if there is a demonstrated need.

## 4.3 Determine fixed-return independence before solving

Convert the callable return contract to its inference term and collect its inference variables.

Let:

```text
R = inference variables referenced by the return term
```

If `R` is empty, the return is inference-independent.

Examples:

```text
f<T>(x: T) -> CellNum
    R = ∅

identity<T>(x: T) -> T
    R = {T}

pair<A, B>(a: A, b: B) -> Pair<A, B>
    R = {A, B}
```

The check must inspect the inference term, not string syntax or generic parameter count.

## 4.4 Required result matrix

### Fully solved generic return with established support

```text
identity<T>(42) -> T
```

Required:

```text
solver outcome = Solved
T support      = Established
result         = Established(Int)
```

### Fully solved generic return with assumed support

```text
identity<T>(assumed_int) -> T
```

Required:

```text
solver outcome = Solved
T support      = Assumed
result         = Assumed(Int)
```

### Terminal conflict with generic return

```text
identity<T>(...) -> T
```

if inference terminates `Conflicting`:

```text
call status = Invalid(C)
R           = {T}
result      = Unknown(InferenceConflict)
```

Do not publish a partial `T` substitution even if the solver happened to write one before discovering the terminal conflict elsewhere.

### Terminal blocked/underconstrained with generic return

```text
solver outcome = Blocked / Underconstrained
R               != ∅
result          = corresponding honest Unknown/Blocked state
```

No unspecialized return contract may be cloned into the result.

### Terminal conflict with fixed return

```text
f<T>(value: T) -> CellNum
```

if dispatch remains exact and the concrete `CellNum` return contract itself is valid:

```text
solver outcome      = Conflicting
R                   = ∅
call status         = Invalid(C)
result              = Established(CellNum)
causal invalidity   = includes C
```

The result survives because its type does not require generic specialization at all.

### Terminal blocked inference with fixed return

If inference is blocked because a generic argument cannot be analyzed, but dispatch and the fixed concrete return contract are independently valid, preserving `Established(CellNum)` is permitted only when the call semantics do not require the missing generic specialization to determine which callable/return contract applies.

If exact callable identity itself depends on the blocked generic information, fail closed instead.

## 4.5 Successful solve: only return-influencing support affects result status

For `InferenceOutcome::Solved`, classify the specialized return from `R`:

```text
R = ∅
    -> fixed concrete return: Established

all variables in R have Established support
    -> Established(specialized return)

any variable in R has Assumed support
    -> Assumed(specialized return)
```

This prevents an assumed generic parameter that is irrelevant to the return from weakening an independent fixed result.

Example:

```text
f<T>(value: T) -> CellNum
```

with assumed `value`:

```text
T support = Assumed
R         = ∅
result    = Established(CellNum)
```

For:

```text
f<A, B>(a: A, b: B) -> Pair<A, B>
```

if `A` is established and `B` is assumed:

```text
result = Assumed(Pair<A, B>)
```

## 4.6 Do not solve with constraints from already-invalid call-shape mappings

Argument-pack shape validation remains before inference.

If a labelled/positional argument is not mapped to a real parameter, it cannot contribute a constraint to that parameter's inference variable.

For a structurally invalid call shape:

- analyze child expressions;
- emit/retain the call-shape diagnostic;
- do not feed fabricated matches into generic solving;
- preserve a fixed return only when exact callable identity and that return contract are independently known despite the malformed pack;
- otherwise fail closed.

This is intentionally conservative until full call-pack completeness lands later.

---

# 5. Amendment D — make suppression cause a real Rust type

## 5.1 Supersedes ambiguous wording in WIP §14.2

The WIP says `SuppressionCause` is “the non-clean type from §6.4.” Rust has no automatic subtype that means “`CausalInvalidity` except `Clean`.” Make it explicit.

Use:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CausalInvalidity {
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

Provide a checked conversion:

```rust
impl CausalInvalidity {
    pub fn suppression_cause(self) -> Option<SuppressionCause> {
        match self {
            Self::Clean => None,
            Self::One(cause) => Some(SuppressionCause::One(cause)),
            Self::Multiple => Some(SuppressionCause::Multiple),
        }
    }
}
```

Then:

```rust
AnalysisStatus::Suppressed(SuppressionCause)
```

cannot represent `Suppressed(Clean)`.

## 5.2 `Multiple` is intentionally cardinality-class information

Do not replace `Multiple` with “first cause” or a nondeterministically ordered set in hot semantic facts.

If rich diagnostic explanation later needs the full root set, store that in explanation/diagnostic arena data, not in every `BindingState`/`TypedExpression`.

The semantic hot-state invariant remains:

```text
Clean
One(C)
Multiple
```

---

# 6. Amendment E — normalize terminology: `invalidity`, not `invalid_cause`

The WIP's canonical `BindingState` field is:

```rust
pub invalidity: CausalInvalidity
```

Any later WIP prose/test assertion that says:

```text
invalid_cause is present
```

must be read as:

```text
invalidity is non-clean
```

Specifically amend the mandatory refuted-annotation regression to assert:

```text
x.invalidity = One(C)  // or equivalent non-clean causal assertion
```

Do not add a second `invalid_cause: Option<DiagnosticCauseId>` field for compatibility. That would reintroduce the single-root representation the WIP deliberately removed.

---

# 7. Amendment F — generic-call result promotion must be one explicit operation

## 7.1 Replace unconditional `establish_call_result` for specialized generic returns

The WIP helper:

```rust
fn establish_call_result(...)
```

is correct for concrete fixed return contracts after exact dispatch, but generic returns need support-aware promotion.

Use two conceptual paths:

```rust
fn establish_fixed_call_result(
    concrete_return: TypeId,
    origin: EvidenceOrigin,
    range: SourceRange,
) -> TypeKnowledge;

fn promote_specialized_generic_result(
    concrete_return: TypeId,
    return_vars: &[InferVarId],
    solution: &InferenceSolution,
    range: SourceRange,
) -> TypeKnowledge;
```

Exact names may differ.

The second helper is used only for `InferenceOutcome::Solved` and must implement the support-classification matrix from Amendment C. Terminal non-solved outcomes use the fixed-return/generic-return rule from Amendment C instead of partial specialization.

## 7.2 Preserve fixed-return independence

Do not conservatively mark every generic-call return `Assumed` merely because one argument was assumed.

Only return-influencing generic variables affect result epistemic status.

Likewise, do not erase a fixed return merely because a generic parameter unrelated to that return encountered an invalid relation.

The call's `AnalysisStatus` and `CausalInvalidity` still record the invalid invocation.

---

# 8. Amendment G — inference provenance must be deterministic and bounded

Part 1 needs enough provenance to explain failures and status, not a second permanent proof graph inside the solver.

Store:

- `ConstraintOrigin` on every stored constraint;
- real argument/call `ExpressionId`s as already required by the WIP;
- per-variable epistemic support summary;
- the terminal failed constraint index/origin when conflicting.

Do not store an unbounded copy of every constraint on every inference variable.

Recommended complexity:

```text
argument matching                 O(args + params)
constraint insertion              O(size of referenced inference term)
union-find representative lookup  near-amortized O(alpha(n)) if path compression is added,
                                  otherwise preserve current bounded small-n behavior
support join                      O(number of vars touched by constraint)
return support classification     O(size of return inference term)
```

Adding path compression to `find_var()` is optional in Part 1. Correctness does not depend on it. If added, ensure mutable compression does not complicate immutable inspection APIs unnecessarily.

---

# 9. Amendment H — fingerprinting of causal state must ignore allocator identities

The WIP correctly requires cause-ID renumbering not to alter semantic product identity. Make the implementation rule exact.

For semantic **product** fingerprints:

```text
CausalInvalidity::Clean    -> hash tag 0
CausalInvalidity::One(_)   -> hash tag 1
CausalInvalidity::Multiple -> hash tag 2
```

For expression status:

```text
Ready                  -> status tag
Invalid(_)             -> invalid tag, not raw DiagnosticCauseId
Suppressed(One(_))     -> suppressed-one tag, not raw DiagnosticCauseId
Suppressed(Multiple)   -> suppressed-multiple tag
Blocked(...)           -> hash semantic block reason
DynamicBoundary(...)   -> hash semantic dynamic reason
Cancelled/Budget/etc.  -> existing semantic terminal representation
```

If a source/provenance-sensitive **input** fingerprint includes diagnostics, do not let a local allocator number become an accidental source input either. Hash the semantic presence/class of the root cause when relevant, not the raw integer identity.

The full root cause ID remains available inside the snapshot for intra-analysis explanation and suppression linking; it is simply not semantic cache identity.

---

# 10. Mandatory additional regression tests

These tests are additive to WIP §28.

## 10.1 Real generic conflict evidence — no fabricated variable or sentinel bounds

Create a solver-level constraint that fails structurally or through incompatible bounds.

Assert:

```text
InferenceOutcome::Conflicting
conflict.origin == actual inserted ConstraintOrigin
conflict.failure contains actual variable/bounds when applicable
conflict does not contain fabricated InferVarId(0)
conflict does not contain Never/Unit placeholders unless they were real source terms
```

Do not write the test by merely asserting “is conflicting.” Inspect the conflict payload.

## 10.2 Kind mismatch retains real kind evidence

Create a `Type -> Type` inference variable and attempt to bind it to a proper `Type` value.

Assert:

```text
expected kind = Type -> Type
actual kind   = Type
real variable = the allocated variable
outcome       = KindMismatch
```

No generic `ConflictingBounds(Never, Never)` fallback is permitted.

## 10.3 Assumed argument yields assumed generic return

Use a callable parameter contract as the only evidence for `T`:

```phalcom
identity<T>(value: T) -> T

run(value: Int) {
  let result = identity(value)
}
```

Assert:

```text
value current   = Assumed(Int)
T               = Int with assumed support
result current  = Assumed(Int)
result origin   = GenericInference
```

## 10.4 Established literal yields established generic return

```phalcom
let result = identity(42)
```

Assert:

```text
T              = Int with established support
result current = Established(Int)
```

## 10.5 Mixed generic return is only as strong as its weakest return dependency

For a generic return containing two variables, infer one from established evidence and one from assumed evidence.

Assert the specialized composite result is `Assumed`, not `Established`.

## 10.6 Assumed generic argument does not weaken an independent fixed return

```text
f<T>(value: T) -> CellNum
```

with `value` assumed.

Assert:

```text
T support       = Assumed
return vars     = empty
call result     = Established(CellNum)
```

## 10.7 Invalid generic evidence blocks only dependent generic return

For:

```text
identity<T>(value: T) -> T
```

construct a real refuted constraint that would otherwise determine `T`.

Assert:

```text
call status = Invalid
return      = Unknown(InferenceConflict) or dedicated invalid-specialization unknown
not Established
not Assumed
```

## 10.8 Invalid generic evidence does not erase independent fixed return

For:

```text
f<T>(value: T) -> CellNum
```

produce a real invalid generic argument/constraint while dispatch remains exact.

Assert:

```text
call status       = Invalid
result knowledge  = Established(CellNum)
causal invalidity = non-clean
```

## 10.9 Expected-result inference selects but does not fabricate

Use an existing generic callable whose return type genuinely contains `T` and can be selected from expected context.

Assert:

```text
context selects T
specialized return contract is concrete
result derives from callable contract
```

Control case: use an unknown/missing return contract and the same expected type. Assert the expected type does not create a known result.

## 10.10 `Suppressed(Clean)` is unrepresentable

Unit-test the conversion API:

```text
Clean -> None
One(C) -> Some(SuppressionCause::One(C))
Multiple -> Some(SuppressionCause::Multiple)
```

There must be no public constructor that accepts `CausalInvalidity::Clean` directly as an `AnalysisStatus::Suppressed` payload.

## 10.11 Product fingerprint ignores cause-number allocation

Build semantically equivalent expression/binding analyses with different `DiagnosticCauseId` numbers but equal:

- status class;
- causal cardinality class;
- type knowledge;
- relation/contract state;
- semantic diagnostics.

Assert equal semantic product fingerprints.

Control case: `One` -> `Multiple` must change the product fingerprint because causal shape changed.

---

# 11. Amend Task 9 in the WIP implementation sequence

Replace the conceptual content of WIP Task 9 with the following stronger task.

## Task 9 — Harden generic inference without evidence loss

**Files:**

```text
Modify: phalcom-semantic/src/checker/inference.rs
Modify: phalcom-semantic/src/checker/call.rs
Modify: phalcom-semantic/src/checker/expected.rs as needed for expected-result constraint metadata
Test:   phalcom-semantic/tests/spec04_5_inference_session.rs
Test:   phalcom-semantic/tests/inference_soundness.rs
Test:   source-level generic semantic integration tests
```

**Required deliverables:**

1. instantiate generic variables from `TypeParameterData.kind`;
2. enforce `TypeStore::kind_of()` compatibility on binding;
3. replace boolean failure APIs with structured `Result` failures;
4. remove fabricated `InferVarId(0)` and `Never/Never` conflicts;
5. convert `Self` with receiver context or structured failure—never `Unit`;
6. feed `GenericSignature.constraints` into the solver with real origins;
7. use real expression IDs for call/argument origins;
8. retain per-variable `InferenceSupport` (`Established`/`Assumed`);
9. distinguish contextual generic selection from value evidence;
10. classify a **successfully solved** generic result from return-influencing variables only;
11. preserve a fixed concrete return across terminal inference failure only when the return contains no inference variables and exact callable identity/contract remain independently known;
12. do not publish partial generic specialization after `Conflicting`, `Blocked`, or `Underconstrained`;
13. propagate underconstrained/conflicting/blocked/cancelled/budget outcomes without cloning the unspecialized return contract.

**TDD order:**

- first add solver conflict-payload tests;
- then kind mismatch tests;
- then assumed-vs-established support tests;
- then fixed-return independence tests;
- then expected-result contextual selection tests;
- only after those pass migrate source-level generic call behavior.

The task is not complete if only the final `TypeId` is correct. The tests must assert support/status and conflict evidence.

---

# 12. Amend the Part 1 completion gate

The WIP's completion gate remains in force. Add the following mandatory items after its existing entries:

32. Generic solver failures preserve the actual failed constraint origin and real failure evidence; no fabricated `InferVarId(0)`, `Never`, `Unit`, or other sentinel stands in for missing conflict data.
33. Solver variable binding/unification APIs cannot erase kind/occurs/bounds/structural failure reasons behind `bool`.
34. A generic substitution retains whether its determining value evidence is established or assumed.
35. A specialized generic return is `Assumed` when any return-influencing inference variable depends on assumed value evidence.
36. A generic fixed return that is independent of assumed inference variables remains `Established` when its callable contract is exact.
37. `Conflicting`, `Blocked`, or `Underconstrained` inference never publishes a partially specialized generic return; only an inference-independent fixed concrete return may survive when exact callable identity/contract remain independently known.
38. An invalid generic call may retain such an independently known fixed return type while remaining `AnalysisStatus::Invalid` with non-clean causal invalidity.
39. Expected-result context may select a valid generic instantiation but cannot fabricate missing callable contract information or rescue a failed solver.
40. `SuppressionCause` is an explicit non-clean representation; `Suppressed(Clean)` is unrepresentable.
41. Semantic product fingerprints hash causal **shape**, not raw local `DiagnosticCauseId` allocation numbers.
42. Generic inference contradictions use a dedicated inference/type-semantic unknown/failure reason rather than `UnknownReason::SyntaxError`.

Part 1 must not advance to Part 2 until all original WIP gates plus these amendment gates pass.

---

# 13. Amend the final semantic epistemic-conflation audit

Add these searches to the WIP final audit:

```bash
rg 'InferVarId::from_index\(0\)' phalcom-semantic/src/checker
rg 'ConflictingBounds.*never|store\.never\(\).*store\.never\(' phalcom-semantic/src/checker
rg 'InferenceOutcome::Conflicting.*SyntaxError|UnknownReason::SyntaxError' phalcom-semantic/src/checker/call.rs phalcom-semantic/src/checker/inference.rs
rg 'fn (bind|unify_terms|subtype_terms).*-> bool' phalcom-semantic/src/checker/inference.rs
rg 'EvidenceStatus::Established|TypeKnowledge::established' phalcom-semantic/src/checker/call.rs phalcom-semantic/src/checker/inference.rs
rg 'Suppressed\(' phalcom-semantic/src/checker
```

Manual review requirements:

- every `Established` creation in generic call code must be justified as fixed-contract or established-support derivation;
- every generic result based on assumed argument evidence must remain assumed if that evidence influences the return;
- every remaining `UnknownReason::SyntaxError` in semantic inference code must correspond to a real syntax/parser failure, not a type conflict;
- every `Suppressed(...)` construction must come from a non-clean `SuppressionCause` conversion;
- no conflict constructor may invent a variable, bound, type, or source origin.

---

# 14. Implementation notes for the agent

## 14.1 Do not over-engineer the inference dependency model

Part 1 does not require a general-purpose proof graph or a new incremental dependency graph inside `InferenceSession`.

The existing solver already has:

- a variable arena;
- variable aliasing;
- lower/upper bounds;
- compound inference terms;
- stored constraints.

Add only enough metadata to retain:

```text
representative -> support class
constraint     -> real origin
terminal error -> failed constraint/origin/reason
```

Return-dependency classification can be computed by collecting variables from the return term.

## 14.2 Prefer dense vectors where IDs are dense

`InferVarId` is allocated monotonically from `next_var_index`. Therefore, for per-variable support, a `Vec` indexed by `InferVarId::index()` is likely simpler and faster than a `HashMap` if the existing ID API exposes stable indexing.

Use maps only where alias representative sparsity or existing code structure makes them clearer.

This is an implementation-quality recommendation, not a semantic requirement.

## 14.3 Keep support monotone

Within one inference session:

```text
Established support -> may weaken to Assumed
Assumed support     -> never upgrades to Established
```

A fresh independent inference session starts fresh; monotonicity is per session, not global.

## 14.4 Do not conflate invalidity with assumption

These are separate axes:

```text
Assumed + clean
Established + clean
Established + invalid call status
Assumed + invalid call status
Unknown + invalid specialization
```

If an implementation enum cannot represent those combinations, the representation is still conflated.

## 14.5 Do not let expected type become support evidence

Expected type participates in solver constraints under `ConstraintOrigin::ExpectedResult` or equivalent, but it does not contribute `InferenceSupport::Assumed` or `Established` as though it were an observed value.

The eventual result is justified by the selected callable instantiation plus callable contract, not by pretending the expected type was a runtime fact.

---

# 15. Verification record for this amendment

This amendment was checked against the WIP specification and repository baseline for internal consistency.

## 15.1 Repository facts re-verified

At baseline `23e6ca126e96b11504a275fa3e777e18fe4d9ef5`:

- `InferenceSession::instantiate_generic_signature()` hardcodes `KindId::TYPE`;
- `TypeStore::type_parameter(id).kind` exists and is canonical parameter kind metadata;
- `TypeStore::kind_of(TypeId)` exists;
- `type_term_to_inference()` maps `SelfType` to `store.unit()`;
- `bind()`, `unify_terms()`, and `subtype_terms()` use boolean success/failure;
- `solve()` can construct a conflict using `InferVarId::from_index(0)` and `store.never()` placeholders;
- `call.rs` maps conflicting inference to `UnknownReason::SyntaxError`;
- generic solved results are currently promoted to `EvidenceAuthority::Proven` without retaining the epistemic support of the constraints that selected the substitution.

## 15.2 Consistency with the WIP semantic laws

The amendments preserve the WIP's core laws:

- inference mechanism does not determine epistemic authority;
- assumed evidence is usable for static checking but remains assumed;
- contradictions do not overwrite independently established facts;
- invalidity/status/type knowledge remain orthogonal;
- expected context is not evidence;
- no real type acts as a sentinel;
- exact fixed callable contracts can provide independently established results;
- formal knowledge remains separate from future advisory evidence.

## 15.3 Scope check

This amendment does **not** require:

- compiler-owned advisory `ValueShape` migration;
- LSP semantic-engine deletion;
- semantic presentation-index publication;
- canonical LSP/semantic identity takeover;
- persistent project/module lifecycle migration;
- full HKT inference completeness;
- a new global dependency graph.

Those remain Part 2/Part 3 or later completeness work.

## 15.4 Final implementation-readiness rule

An implementing agent must read the WIP Part 1 specification **and this amendment together**.

The WIP defines the overall formal epistemic architecture and task sequence. This amendment supplies the final generic-inference and suppression/fingerprint corrections discovered by verification.

Neither document should be implemented in isolation.
