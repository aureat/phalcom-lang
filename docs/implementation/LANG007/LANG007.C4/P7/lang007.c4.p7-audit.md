# LANG007.C4.P7 — Implementation Status Audit and Full-Correctness Requirements

**Repository:** [`aureat/phalcom-lang`](https://github.com/aureat/phalcom-lang)  
**Audited branch:** `main`  
**Latest observed repository revision:** `dbcdcd13ec054b84a358ca5d05c6414f75307d0d`  
**Audit date:** October 13, 2026  
**Program:** `LANG007` — Structural Static Upgrade  
**Checkpoint:** `LANG007.C4`  
**Plan:** `LANG007.C4.P7` — Recursive Generic Static Computation, Normalized Residualization, Specialization, and Memoization

## 1. Executive assessment

### 1.1 Overall verdict

**P7 is substantially implemented, but it is not complete and should not be promoted to `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.**

The repository contains the core machinery for recursive static computation, canonical application identity, residual construction, and residual specialization. These are not merely proposed interfaces: the implementation exists on `main`.

The most important completed architectural capabilities include:

- Argument-sensitive recursive static applications.
- Exact application-key cycle detection.
- Evaluator-local memoization.
- Canonical session-local residual identity and interning.
- Composite generic `TypeForm` substitution.
- Canonical proof residuals.
- Checker-published member-projection resolution.
- Specialization of parameters, applications, proofs, and member projections.
- Separation of application identity, residual capture, and supplied specialization environments.
- Explicit evaluator work frames and structural iteration cursors.

However, **the completed machinery does not yet cover the entire semantic residualization model promised by P7**.

In particular, the plan requires more than representing symbolic applications. It requires correct handling of partially known control flow, structural construction, bounded residual growth, incremental dependencies, diagnostics, and performance.

The durable checkpoint explicitly records:

> T0–T2 acceptance remains open; T3/G3 through T7/G7 have focused implementation evidence; T8–T13 and broad certification remain open.

This is the current authoritative recorded status, not proof that every reported gate remains valid after subsequent modifications. LANG007.C4-CHECKPOINT

### 1.2 Important distinction: implementation versus acceptance

Three different completion conditions must be distinguished.

| Dimension | Assessment |
|---|---|
| Architectural foundation | Substantially implemented |
| Recursive application execution | Implemented, with focused historical evidence |
| Canonical residual identity | Implemented, requiring full normalization audit |
| Core residual specialization | Implemented, with later corrective changes |
| Static conditional residualization | Not certified |
| Static match residualization | Not certified |
| Structural construction residualization | Not certified |
| Residual growth/resource control | Not certified |
| Incremental integration | Not certified |
| End-to-end correctness | Not established |
| Full P7 acceptance | **Open** |

An additional distinction is essential: **a Rust enum variant existing in the repository does not establish that the evaluator produces, normalizes, specializes, and correctly consumes that variant.**

For example, the live `StaticResidualData` already defines representations for `If`, `Match`, `Pack`, `Tuple`, `Record`, `Data`, and `Variant`. Their presence is valuable implementation progress. But the currently inspected `specialize.rs` implements four principal specialization families: `Parameter`, `Apply`, `Proof`, and `MemberProjection`.

Consequently, P7 is beyond a minimal prototype, but its residual language is not yet fully operational.

### 1.3 Architectural direction

The established architecture is the right foundation to continue.

The implementation should preserve:

```text
Ordinary Telos source
        |
        v
Canonical semantic checking
        |
        v
CheckedStaticBody
        |
        +---- Checked static applications
        +---- Canonical proof predicates
        +---- Member projections
        +---- Match/pattern resolutions
        +---- Static iteration resolutions
        +---- Semantic dependencies
        |
        v
StaticEvaluator
        |
        +---- Exact application identity
        +---- Explicit execution frames
        +---- Recursion/cycle detection
        +---- Evaluator-local memoization
        +---- Canonical StaticValueStore
        +---- Canonical StaticResidualStore
        |
        v
StaticEvalOutcome
        |
        +---- ReducedValue
        +---- ReducedKinded
        +---- Symbolic residual
        +---- Invalid
        +---- Recursive
        +---- BudgetExceeded
        +---- Cancelled
```

The architecture must not branch into an alternative compile-time interpreter, a second type/proof engine, or AST-based residual evaluation.

Those restrictions are explicit in the checkpoint guidance. LANG007.C4-GUIDANCE

### 1.4 Immediate recommendation

Do not begin P8 on the assumption that P7 is substantially equivalent to complete.

The correct sequence is:

1. Revalidate the completed T3–T7 behavior after the latest corrective changes.
2. Close T0–T2 and their integration requirements.
3. Complete T8–T10 using the existing residual substrate.
4. Enforce T11 growth/resource invariants.
5. Close T12 incremental integration and fingerprint correctness.
6. Complete T13 diagnostics, performance, and certification.

P8 can subsequently take responsibility for durable cross-module residual identities and metadata serialization without inheriting an unstable or semantically incomplete P7 model.

---

## 2. Repository audit and evidence

### 2.1 Revision and delivery history

The GitHub repository is accessible through the connected repository integration. I inspected the current `main` branch and its recent commit history.

The relevant delivery sequence includes:

| Commit | Change | Significance |
|---|---|---|
| [`608c8c6c`](https://github.com/aureat/phalcom-lang/commit/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac) | `feat(LANG007.C4.P7.T7): specialize static residuals` | Introduces the T7 specialization implementation |
| [`db016f07`](https://github.com/aureat/phalcom-lang/commit/db016f0727b8330ac0c5f3590de871c6053242f6) | `fix(static-eval): harden residual specialization` | Corrective specialization work |
| [`0651b762`](https://github.com/aureat/phalcom-lang/commit/0651b762ac2a0c5cb1a1abd38d01c14e3afe8297) | `fix(static-eval): validate residual specialization` | Additional validation work |
| [`b00d75b3`](https://github.com/aureat/phalcom-lang/commit/b00d75b3ac9f50d37cf9abfddfde00917e099031) | `fix(static-eval): preserve residual specialization context` | Addresses context preservation |
| [`10f7297c`](https://github.com/aureat/phalcom-lang/commit/10f7297cf530139e4c4803be91f7769aa40c850d) | `fix(semantic): make workspace publication transactional` | Changes adjacent semantic publication behavior |

The important observation is that **T7 underwent several corrections after its initial implementation and focused acceptance**.

The attached `INC-003` report explicitly states that the post-completion corrections had passed `cargo check`, but behavioral tests had not been rerun at that point. LANG007.C4.P7-INC-003-residual-…

Subsequent commits strengthen the implementation, but their existence does not independently establish that the full G7 acceptance matrix was re-executed against the final branch state.

**Audit finding:** G7 should be re-certified, not rewritten.

### 2.2 Live source inventory

The following modules are present on `main`.

| Source file | Audited responsibility | Assessment |
|---|---|---|
| [`static_eval/mod.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-semantic/src/static_eval/mod.rs) | Public exports and module composition | Implemented |
| [`static_eval/application.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-semantic/src/static_eval/application.rs) | Application keys, environments, normalization, cycle identity | Implemented |
| [`static_eval/residual.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-semantic/src/static_eval/residual.rs) | Residual IR, store, interning, fingerprints | Implemented, incomplete operational coverage |
| [`static_eval/specialize.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-semantic/src/static_eval/specialize.rs) | Residual specialization | Four families implemented |
| [`static_eval/machine.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-semantic/src/static_eval/machine.rs) | Evaluator, application execution, control flow, residual construction | Substantial implementation |
| [`db/fingerprint.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-semantic/src/db/fingerprint.rs) | Semantic product fingerprints | Symbolic fingerprint correctness remains open |

The existence of `specialize.rs` and its registration in `mod.rs` confirms that the T7 specialization engine is integrated, not merely an experimental file.

---

# 3. Task-by-task implementation assessment

## 3.1 Overall task matrix

The task names and required scope below follow the active P7 implementation plan. LANG007.C4.P7-luna-patch-grade-…

| Task | Required deliverable | Implementation status | Acceptance status |
|---|---|---|---|
| **T0** | Takeover hardening and lifecycle rebaseline | Partial / mixed | Open |
| **T1** | Residual store and structured terminal model | Substantially implemented | Requires closure |
| **T2** | Exact application key and environment projection | Substantially implemented | Requires closure |
| **T3** | Exact cycle detection | Implemented | Historically focused-tested |
| **T4** | Evaluator-local memoization | Implemented | Historically focused-tested |
| **T5** | Parameter and Apply residualization | Implemented | Historically focused-tested |
| **T6** | Proof and member-projection residuals | Implemented | Focused-tested |
| **T7** | Residual specialization engine | Implemented with subsequent corrections | Re-certification required |
| **T8** | Conditional residualization | IR substrate present | Open |
| **T9** | Match residualization and branch-proof specialization | IR substrate present | Open |
| **T10** | Structural construction residuals and recursive algorithms | IR substrate present | Open |
| **T11** | Explicit residual growth budget | Basic node charging present | Full certification open |
| **T12** | Incremental dependency/fingerprint integration | Partial | Open |
| **T13** | Diagnostics, performance certification, lifecycle closure | Partial foundational support | Open |

This matrix deliberately distinguishes **source implementation** from **verified completion**.

## 3.2 T0 — Takeover hardening

**Assessment: PARTIAL; requires a dedicated acceptance audit.**

T0 contains several important inherited correctness requirements:

- Canonical proof-predicate recognition.
- Canonical `Conforms` propositions.
- Correct equivalence proof behavior.
- Per-item refinement for heterogeneous structural iteration.
- Checked iteration-mode authority.
- O(1) iteration cursors.
- Fixed-field iteration over open unordered tails.
- Fail-closed handling of missing semantic authorities.
- Compatibility with canonical record and Pack patterns.

The live implementation confirms that `StaticIterationCursor` exists in `machine.rs`, beginning around line 355.

It also includes cursor variants for Packs, tuples, sequences, records, data, and type structures. The `ForNext` work item uses a cursor rather than the earlier eagerly materialized item vector.

This establishes substantial progress on T0's iteration-performance requirements.

However, the broader T0 contract concerns **semantic precision**, not merely iterator storage.

For example, replacing:

```rust
Vec<StaticValueRef>
```

with:

```rust
StaticIterationCursor
```

does not prove that a heterogeneous static loop is checked under every exact item refinement.

Those are separate invariants.

### Required completion

T0 should close through focused tests demonstrating:

1. Different exact iteration items retain their respective type refinements.
2. Canonical checked iteration modes control evaluator behavior.
3. Fixed unordered fields remain iterable when an anonymous tail is open.
4. Missing semantic metadata is not interpreted as an empty structure.
5. Ordinary pattern resolutions remain authoritative.
6. Proof predicate recognition cannot be hijacked by shadowing or spelling collisions.

The P4 `FieldKey`/Record-pattern migration remains a dependency where generalized record patterns are required.

**Do not implement a P7-specific Record-pattern resolver to circumvent unfinished P4 work.**

---

## 3.3 T1–T2 — Residual foundations and application identity

**Assessment: IMPLEMENTED, but acceptance needs explicit closure.**

The live residual store contains the required core types:

```rust
StaticResidualStoreId
StaticResidualId
StaticResidualRef
StaticEvalTerm
StaticResidualParameter
StaticResidualApplication
StaticResidualData
StaticResidualStore
```

The application module contains:

```rust
StaticSpecializationEnvironmentKey
StaticApplicationEnvironmentKey
StaticApplicationKey
StaticArgumentIdentity
StaticCycle
StaticMemoResult
```

This architecture correctly separates source application sites from semantic application identities.

A checked source-site identity answers:

> Where did this application occur?

A canonical application key answers:

> Which static computation is being performed under which semantically relevant arguments and facts?

These cannot be conflated.

The application module also implements environment normalization and substitution-related operations. This is consistent with the approved T7 amendment.

### Remaining requirements

The final T1–T2 audit must establish that:

- Every residual reference belongs to its owning store.
- All residual construction routes through normalization.
- Store identity cannot leak into language-level semantic equality.
- Canonically equivalent environments normalize identically.
- No source range or transient execution state participates in semantic residual identity.
- Application keys contain only result-relevant ambient facts.
- Invalid or incomplete computations never enter the successful memo cache.
- Residual stores cannot admit cycles through public construction APIs.

One point deserves particular attention: **store qualification and durable identity are different concepts**.

P7 may use canonical IDs within one store lineage. It must not claim that those identifiers retain semantic identity across independent sessions or serialized metadata.

That remains P8's responsibility.

---

## 3.4 T3 — Exact recursive application identity

**Assessment: IMPLEMENTED / HISTORICALLY FOCUSED_TESTED.**

Incident `INC-001` identified an important correctness failure:

```telos
data Boxed<T> {
    value: T
}

type Grow<T: Type>: Type {
    Grow<Boxed<T>>
}

type Start: Type = Grow<Int>
```

The original evaluator incorrectly classified changing-argument recursion as an exact cycle because composite type arguments were not specialized.

The adopted architecture uses the active session-owned `TypeStore`, canonical substitution, and copy-on-write snapshot publication. LANG007.C4.P7-INC-001-composite…

The checkpoint subsequently records that the regression progresses until the evaluator budget, while exact direct and mutual cycles still produce `Recursive`. LANG007.C4-CHECKPOINT

This is the correct semantic distinction:

\[
F(A)\rightarrow F(A)
\]

is an exact recursive cycle.

Whereas:

\[
F(A)\rightarrow F(G(A))\rightarrow F(G(G(A)))\rightarrow\cdots
\]

is not an exact cycle merely because the declaration identity repeats.

The latter computation may eventually terminate, or it may exhaust the deterministic resource budget.

### Required revalidation

Retain coverage for direct cycles, mutual cycles, changing arguments, composite arguments, snapshot isolation, cancellation, and budget exhaustion.

Also verify that ambient proof environments cannot accidentally transform an exact cycle into an unbounded computation by including irrelevant activation history in application identity.

This was one of the central problems addressed by T7.

---

## 3.5 T4 — Evaluator-local memoization

**Assessment: IMPLEMENTED / HISTORICALLY FOCUSED_TESTED.**

The current architecture implements memoization using semantic application keys rather than checked source-site IDs.

The checkpoint records that repeated exact applications use fewer evaluator steps than applications with distinct arguments.

This is meaningful evidence.

However, the remaining certification should inspect cache validity under differing proof environments and verify that cached symbolic results retain their complete specialization context.

The essential invariant is:

\[
\operatorname{MemoKey}
=
(\text{Definition},\text{CanonicalArguments},\text{RelevantFacts})
\]

Two applications with identical arguments but semantically different relevant proof facts must not collide.

Conversely, irrelevant caller history must not create different memo entries for the same computation.

**Required test:** Repeated applications under equivalent normalized environments must reuse computation; applications under genuinely different semantic facts must remain distinct.

---

## 3.6 T5 — Parameter and Apply residualization

**Assessment: IMPLEMENTED / HISTORICALLY FOCUSED_TESTED.**

The evaluator can preserve unresolved static parameters and applications as canonical residuals.

Conceptually:

```text
F<T>
    |
    v
Apply(
    definition = F,
    arguments = [
        Parameter(T)
    ]
)
```

The checkpoint confirms that repeated open applications reuse residual identity and that partially known applications preserve their concrete arguments.

This satisfies the central T5 design goal.

An especially important requirement is that residualization must retain result-domain information.

A residual returning `Type` is not interchangeable with a residual returning an ordinary static value, such as `Bool` or `ProductShape`.

The existing domain-aware residual model addresses this distinction.

### Remaining audit requirements

Verify partial specialization with mixed value and kinded arguments, nested applications, and residuals that escape the original evaluator frame.

These are particularly important because later T7 corrections changed the treatment of escaped residual context.

---

## 3.7 T6 — Proof and member-projection residuals

**Assessment: IMPLEMENTED / FOCUSED_TESTED.**

Incident `INC-002` correctly identified a missing bridge between checked member projections and the structural reflection projector.

The approved resolution introduces checker-owned canonical projection products carrying:

- Member identity.
- Arguments and evidence.
- Result domain.
- Checked source-site association.
- Relevant semantic dependencies.

The evaluator then consumes these products through the existing structural projector.

This matters because:

```telos
Product::Shape<T>
```

can produce an ordinary static value, not merely a kinded type result.

The implementation resolution records successful focused tests covering open projection residualization, exact reduction, and distinct terminal outcomes. LANG007.C4.P7-INC-002-member-pr…

The architecture should be retained.

**Do not introduce a second member-projection evaluator in P7.**

---

## 3.8 T7 — Residual specialization

**Assessment: IMPLEMENTED / RE-CERTIFICATION REQUIRED.**

This is currently P7's most mature advanced capability.

The live `specialize.rs` implements:

```rust
specialize_parameter(...)
specialize_apply(...)
specialize_proof(...)
specialize_member_projection(...)
```

The top-level dispatcher starts around line 43.

It correctly recognizes these four residual families.

The adopted three-environment model is also architecturally important:

| Environment | Purpose |
|---|---|
| Activation capture | Preserve the current semantic context of an escaping residual |
| Application identity | Determine cycle and memo identity |
| Supplied specialization | Compose new exact facts with captured context |

The distinction prevents recursive activation history from being mistaken for semantic environment identity.

### Outstanding re-certification

The final G7 test matrix should include all of the following:

| Test | Expected property |
|---|---|
| Environment normalization | Equivalent facts produce equal canonical environments |
| Recursive shadowing | Current frame bindings correctly shadow historical bindings |
| Argument identity | Historical caller arguments do not pollute application keys |
| Substitution closure | Transitive substitutions resolve before conflict checks |
| Conflicting bindings | Contradictions fail closed |
| Escaped proof | Preserves all necessary context |
| Escaped projection | Preserves canonical member identity and domain |
| Trusted branch facts | Cannot be forged through public residual propositions |
| Exact parity | Cold evaluation equals open-then-specialize |
| Idempotence | Repeating the same specialization does not change the result |
| No-new-facts identity | Empty specialization preserves canonical identity where appropriate |
| Budget/cancellation | Terminal failures do not become cached success |

The dedicated T7 implementation plan records completion of G7.A–G7.E. LANG007.C4.P7.T7-residual-speci…

The subsequent incident report, however, documents corrective changes after that evidence.

Therefore, **the latest implementation should be tested against the entire G7 matrix before its completion claim is inherited by T8–T10.**

---

# 4. Confirmed gaps and correctness findings

This section separates confirmed source-level gaps from risks requiring further verification.

## Finding P7-AUD-001 — Residual IR exceeds specializer coverage

**Severity: HIGH**  
**Classification: Confirmed implementation gap**

The live residual enum supports:

```rust
StaticResidualData::Parameter
StaticResidualData::Apply
StaticResidualData::Proof
StaticResidualData::MemberProjection
StaticResidualData::If
StaticResidualData::Match
StaticResidualData::Pack
StaticResidualData::Tuple
StaticResidualData::Record
StaticResidualData::Data
StaticResidualData::Variant
```

However, the live specialization dispatcher contains:

```rust
match residual_data {
    StaticResidualData::Parameter { ... } => { ... }

    StaticResidualData::Apply { ... } => { ... }

    StaticResidualData::Proof { ... } => { ... }

    StaticResidualData::MemberProjection { ... } => { ... }

    _ => Err(
        Stop::invalid(
            StaticEvalError::UnsupportedExpression
        )
    ),
}
```

**Source:** `phalcom-semantic/src/static_eval/specialize.rs:62–75`

The consequence is unambiguous: if a residual of one of the other families reaches this dispatcher, it is rejected.

This does not establish that currently accepted source programs routinely produce those residuals. It does establish that the operational specialization contract is incomplete.

### Required correction

Implement specialization for the remaining required residual families, with semantic normalization and correct result-domain preservation.

Do not interpret the existing enum definitions as completing T8–T10.

---

## Finding P7-AUD-002 — Symbolic match does not yet preserve canonical match semantics

**Severity: HIGH**  
**Classification: Confirmed T9 gap**

The evaluator's `TryMatchArm` logic handles:

```rust
StaticPatternMatchOutcome::Matched(...)
StaticPatternMatchOutcome::NoMatch
StaticPatternMatchOutcome::Symbolic
StaticPatternMatchOutcome::Invalid(...)
StaticPatternMatchOutcome::Cancelled
StaticPatternMatchOutcome::BudgetExceeded(...)
```

On a symbolic pattern outcome, it currently executes:

```rust
StaticPatternMatchOutcome::Symbolic => {
    let r = self.make_symbolic_residual()?;

    return Err(Stop::Symbolic(r));
}
```

**Source:** `phalcom-semantic/src/static_eval/machine.rs:2298–2301`

This is not sufficient to establish a normalized `Match` residual containing the unresolved scrutinee, canonical pattern resolution, ordered arms, and proof environments.

The returned symbolic residual may preserve an enclosing application, but that is not equivalent to an independently represented residual match.

### Required correction

T9 must produce a semantic residual match using the canonical P6 match resolution.

It must preserve:

- Scrutinee semantics.
- Source arm order.
- Canonical pattern identities.
- Branch-specific proof environments.
- Result domain.
- Correct first-match behavior.
- Non-evaluation of untaken branches.

Later specialization must evaluate the newly known scrutinee using the existing pattern matcher.

No second pattern language should be introduced.

---

## Finding P7-AUD-003 — Symbolic incremental fingerprints use arena identity

**Severity: HIGH**  
**Classification: Confirmed implementation gap; T12/P8 boundary**

The live fingerprint implementation contains:

```rust
StaticEvalOutcome::Symbolic(residual_ref) => {
    2u8.hash(&mut hasher);

    residual_ref.store_id().raw().hash(&mut hasher);

    residual_ref.residual_id().raw().hash(&mut hasher);
}
```

**Source:** `phalcom-semantic/src/db/fingerprint.rs:3087–3091`

This hashes the residual's store and allocation identifiers rather than a normalized semantic fingerprint.

Two semantically equivalent residual DAGs created in independent store lineages can therefore produce different product fingerprints.

That is a correctness concern for incremental reuse, not simply an optimization detail.

### Required correction

T12 must define and consume a semantic residual fingerprint that is independent of transient arena allocation.

The correct direction is:

```text
Normalized residual
        |
        v
Canonical semantic node data
        |
        +---- Stable definition identities
        +---- Stable argument meanings
        +---- Normalized proof propositions
        +---- Canonical projection identities
        +---- Relevant semantic environment
        +---- Recursively fingerprinted child terms
        |
        v
Semantic content fingerprint
```

Care is required for `TypeId` and `StaticValueRef`: their raw identities are not durable cross-lineage semantic identifiers.

P7 should establish correct semantic fingerprinting at the incremental product boundary while leaving P8 responsible for durable serialized representations.

This gap is explicitly acknowledged in the T7 incident's architect ruling. LANG007.C4.P7-INC-003-residual-…

---

## Finding P7-AUD-004 — Residual growth charging exists, but resource certification remains incomplete

**Severity: MEDIUM–HIGH**  
**Classification: Partial implementation**

The live residual store charges the budget when a new node is interned:

```rust
self.validate_data(&data)?;

let key = (
    data.clone(),
    trusted_context.clone()
);

if let Some(&existing) = self.interner.get(&key) {
    return Ok(
        StaticResidualRef::new(self.id, existing)
    );
}

budget
    .charge_residual_node()
    .map_err(StaticResidualStoreError::from)?;
```

**Source:** `phalcom-semantic/src/static_eval/residual.rs:255–270`

This is a positive finding: a dedicated residual-node budget exists.

However, node count alone is not automatically sufficient to bound all relevant costs.

A single residual node can contain large argument arrays, structural fields, nested environments, or other variable-size payloads.

The current function also constructs and clones the prospective interning key before charging the new-node budget.

That is not, by itself, evidence of an exploitable resource failure. It does mean that the complete allocation and work-cost accounting requires testing.

### Required correction

T11 must establish bounded behavior for:

1. Number of residual nodes.
2. Variable-size node payloads.
3. Environment normalization and substitution.
4. Structural construction.
5. Deep specialization.
6. Repeated interning attempts.
7. Cancellation responsiveness.
8. Large argument lists and nested structural values.

Avoid introducing unrelated arbitrary recursion limits.

Use deterministic resource accounting aligned with the existing query-budget system.

---

## Finding P7-AUD-005 — Residual construction and normalization need full-family coverage

**Severity: HIGH**  
**Classification: Open T8–T10 correctness requirement**

The residual store exposes constructors for:

```rust
if_then_else(...)
match_term(...)
pack(...)
tuple(...)
record(...)
data(...)
variant(...)
```

But there is an important distinction between interning and normalization.

A constructor such as:

```rust
self.intern(
    StaticResidualData::If {
        condition,
        then_term,
        else_term,
        result_domain,
    },
    budget,
)
```

establishes an interned residual node.

It does not, on its own, prove reductions such as:

```text
If(true, A, B)  -> A
If(false, A, B) -> B
```

or correct conditional proof-environment transport.

Likewise, interning a structural residual does not automatically prove that fully concrete children collapse into the canonical reduced static value.

### Required correction

Every supported residual family needs explicit normalization laws, plus tests proving them.

Interning is necessary, but **semantic normalization is the acceptance criterion**.

---

# 5. Remaining implementation requirements

## 5.1 T8 — Conditional residualization

**Priority: HIGH**

T8 should be the next major functional implementation after recertifying the core foundation.

The target behavior is:

```text
Known condition:
    Evaluate selected branch only

Unknown but valid generic condition:
    Preserve canonical conditional semantics

Later specialization:
    Resolve condition
    Evaluate selected branch only
    Preserve proof facts
    Produce canonical reduced or residual result
```

The residual form must preserve the information required for later specialization.

A conceptual representation is:

```rust
If {
    condition: StaticResidualTerm,
    then_term: StaticResidualTerm,
    else_term: StaticResidualTerm,
    result_domain: StaticValueDomain,
}
```

This existing shape may require additional context handling depending on the checked branch products and deferred execution strategy.

### Critical semantic requirement

The evaluator must not eagerly execute both branches merely to construct the residual.

For example:

```telos
type Choose<T: Type>: Type {
    if Equivalent<T, Int> {
        Int
    } else {
        String
    }
}
```

When the condition is unresolved, the residual must preserve the conditional.

When `T = Int`, specialization must choose `Int`.

When `T = Bool`, specialization must choose `String`, assuming the equivalence proof establishes non-equivalence.

If one branch contains invalid or nonterminating static work, that branch must not execute when the other is selected.

**Acceptance:** Cold exact evaluation and open-then-specialize must agree on results, diagnostics, budget behavior where specified, and branch selection.

---

## 5.2 T9 — Match residualization

**Priority: HIGH**

T9 is more complex because matching carries canonical pattern resolution and potentially branch-local refinements.

The correct pipeline is:

```text
Canonical checked MatchResolution
        |
        v
Known scrutinee?
        |
        +---- Yes
        |       |
        |       v
        |   Existing P6 matcher
        |       |
        |       v
        |   Selected arm
        |
        +---- No
                |
                v
        Canonical Match residual
                |
                v
        Later specialization
                |
                v
        Existing P6 matcher
                |
                v
        Selected arm + proofs
```

### Required invariants

The residual must not reconstruct pattern identity from source spelling.

It must preserve the checked match semantics and the distinction between public propositions and trusted branch facts.

The trusted proof-context separation introduced by the T7 corrections should be reused.

### Required coverage

Test exhaustive and non-exhaustive matches, ordered alternatives, heterogeneous data variants, nested matches, branch-specific generic refinements, and unreachable/untaken branches.

Particular care is needed when specializing a residual from a branch whose original proof environment is no longer available as an active checker frame.

The residual must contain sufficient canonical information to reproduce its legitimate semantics without manufacturing proof authority.

---

## 5.3 T10 — Structural construction residuals and recursive algorithms

**Priority: HIGH**

This task is central to the practical value of P7.

The checkpoint does not merely require recursive type predicates; it requires ordinary Telos source to perform useful structural computations.

Expected capabilities include constructing and transforming Packs, tuples, records, data products, and variants from partially known static inputs.

### Required semantics

| Structural family | Required behavior |
|---|---|
| Pack | Preserve positional/labeled structure and exact element order |
| Tuple | Preserve arity, element domains, and order |
| Record | Preserve canonical `FieldKey` identity and order |
| Data | Preserve canonical data declaration and component identity |
| Variant | Preserve canonical variant identity and payload semantics |
| Recursive algorithm | Support changing-argument recursion with deterministic resource limits |

Fully reduced structural residuals must collapse into ordinary static values.

Partially reduced structures must preserve known components rather than discarding them.

### Performance requirement

P7 must not accidentally turn linear structural algorithms into quadratic algorithms through repeated immutable copying.

The implementation should exploit existing Pack slicing, canonical value sharing, and structural construction facilities.

The guidance explicitly identifies recursive source-defined structural algorithms and asymptotic efficiency as checkpoint objectives. LANG007.C4-GUIDANCE

---

## 5.4 T11 — Resource safety and bounded growth

**Priority: HIGH**

The existing residual-node budget is the foundation.

Completion requires a systematic accounting model.

At minimum, the evaluator must charge work for operations proportional to:

\[
\text{Work}
=
\text{NodeCreation}
+
\text{PayloadTraversal}
+
\text{Substitution}
+
\text{Normalization}
+
\text{StructuralConstruction}
\]

This is a conceptual accounting equation, not a requirement to introduce five independent public budget settings.

The actual implementation can reuse existing query-budget infrastructure.

The essential requirement is that the accounting be deterministic, bounded, and enforced before expensive operations where practicable.

### Acceptance

A deliberately growing static computation must terminate with a budget outcome, not:

- Rust stack overflow.
- Unbounded memory growth.
- Process abort.
- Silent truncation.
- A memoized incomplete result.
- An incorrectly reported recursive cycle.

Deep static evaluation must remain independent of equivalent-depth Rust recursion.

---

## 5.5 T12 — Incremental integration and fingerprint correctness

**Priority: CRITICAL**

This is the most important remaining cross-layer correctness task.

The current symbolic fingerprinting implementation is unsuitable as a durable semantic dependency identity.

The completed T12 implementation must establish:

```text
Checked static definition
        |
        v
Canonical semantic dependencies
        |
        v
Static evaluation
        |
        +---- Reduced semantic value
        |
        +---- Normalized residual DAG
        |
        v
Stable semantic content fingerprint
        |
        v
Incremental dependency tracking
        |
        v
Correct downstream invalidation
```

### Required test matrix

| Scenario | Expected outcome |
|---|---|
| Unchanged definition, unchanged arguments | Reuse where valid |
| Changed static definition body | Affected clients recompute |
| Changed imported static constant | Dependent clients invalidate |
| Changed relevant proof | Dependent specialization invalidates |
| Unrelated source edit | Unaffected products remain reusable |
| Equivalent residual allocated differently | Same semantic fingerprint |
| Different residual semantics | Different semantic identity |
| Failed prior evaluation becomes valid | Previously blocked client recomputes |
| Removed dependency | Stale products are retired |
| Cold versus incremental reconstruction | Equivalent semantic outcomes and diagnostics |

A particularly important test is a **negative-to-positive incremental transition**.

A client that previously could not reduce a static computation must become valid when the missing canonical dependency appears.

Do not rely on positive-result caching tests alone.

### Ownership boundary

P7 owns correct semantic dependency participation and fingerprints for its residual products.

P8 owns durable cross-module transport, serialized stable identities, and metadata compatibility.

These responsibilities overlap at the interface, but they must not be conflated.

---

## 5.6 T13 — Diagnostics, performance, and final certification

**Priority: HIGH**

T13 should certify the already implemented architecture, not invent another evaluator design.

### Required diagnostics

Provide diagnostic-quality reporting for:

- Exact recursive cycles.
- Budget exhaustion.
- Residual growth exhaustion.
- Unresolved closed static computations.
- Invalid specialization arguments.
- Conflicting specialization environments.
- Unsupported static operations.
- Invalid member projection.
- Missing or blocked semantic dependencies.

Cycle diagnostics should identify the relevant static definitions and argument transitions without constructing enormous diagnostic strings.

### Required performance evidence

At minimum, measure:

| Workload | Primary concern |
|---|---|
| Deep exact recursion | Host stack safety |
| Changing-argument recursion | Correct cycle distinction |
| Repeated recursive subproblems | Memoization effectiveness |
| Deep residual DAG | Normalization and specialization cost |
| Large Pack traversal | Linear iteration |
| Structural construction | Copying/allocation complexity |
| Repeated specialization | Canonical reuse |
| Cancellation | Bounded response latency |
| Incremental rebuild | Reuse and invalidation correctness |

The tests should record concrete observations, not merely assertions that operations are fast.

A benchmark passing does not establish acceptable asymptotic behavior unless the workload meaningfully discriminates between the intended and pathological implementations.

---

# 6. Critical architectural consistency checks

## 6.1 Exact evaluation and residual specialization must be equivalent

This is the central P7 semantic law.

For supported checked static definitions:

\[
\operatorname{Eval}(F,\vec a,E)
\equiv
\operatorname{Specialize}
\left(
\operatorname{EvalOpen}(F,\vec p,E_0),
\vec a,
E
\right)
\]

Here, equivalence concerns semantic result and applicable diagnostics, not incidental arena IDs or execution traces.

This law should govern every new residual family.

It is more important than matching internal implementation structure.

## 6.2 Public proof propositions are not trusted proof facts

The post-T7 corrections introduced an important trust boundary.

A residual may contain a proposition:

```text
Subtype<T, Number>
```

But possession of that proposition does not establish that it is true.

Trusted branch refinements must come from the canonical checked match/proof process.

The specializer must not promote arbitrary supplied propositions into trusted proof evidence.

The incident explicitly records this distinction. LANG007.C4.P7-INC-003-residual-…

This invariant must carry forward into T8 and T9.

## 6.3 Canonical identity must remain independent of execution history

An application key cannot depend on irrelevant historical frames.

A residual must nevertheless retain the legitimate semantic context needed for later independent specialization.

Those are different responsibilities.

The current three-way environment model should remain authoritative.

Do not simplify it back into one generic environment map.

## 6.4 Semantic residuals must not become serialized AST

P7 should preserve checked semantic meaning.

It must not preserve or re-execute raw parser AST as the durable residual contract.

The static evaluator can use checked source bodies during the current session, but P8's future durable products must not require recreating ephemeral parser or checker state.

That distinction is fundamental to the P7/P8 boundary.

## 6.5 Result domains must remain explicit

A residual's result may belong to a kinded domain or to an ordinary static-value domain.

These cannot be interchangeable.

For example:

```text
Type
Bool
ProductShape
```

have different semantic completion requirements.

T6 already addressed this issue for member projections.

T8–T10 must preserve it universally.

---

# 7. Recommended completion sequence

I recommend completing P7 through six bounded implementation packages, while retaining the existing T0–T13 task identities.

| Package | Scope | Exit gate |
|---|---|---|
| **A — Foundation recertification** | T0–T2, corrected T7 behavior | Canonical authority and core invariants proven |
| **B — Conditional residuals** | T8 | Exact/open parity and untaken-branch safety |
| **C — Match residuals** | T9 | Canonical pattern/proof specialization |
| **D — Structural residuals** | T10 | Structural normalization and recursive algorithms |
| **E — Resources and incrementality** | T11–T12 | Deterministic bounds and semantic fingerprints |
| **F — Certification** | T13 | Diagnostics, performance, integration, lifecycle closure |

### Package A: Foundation recertification

**Required before substantial T8 work.**

The objective is not to revisit settled architectural decisions.

It is to prove that the existing implementation and post-T7 corrections satisfy the required contracts.

Recommended commands include:

```bash
RUSTFLAGS='' cargo check -p phalcom-semantic --tests

RUSTFLAGS='' cargo test \
  -p phalcom-semantic \
  --test semantic \
  foundations::static_residuals

RUSTFLAGS='' cargo test \
  -p phalcom-semantic \
  --test semantic \
  integration::static_recursive_evaluation

RUSTFLAGS='' cargo test \
  -p phalcom-semantic \
  --test semantic \
  static_definition_bodies

RUSTFLAGS='' cargo test \
  -p phalcom-semantic \
  --test semantic \
  static_constants
```

These commands are recommended verification steps; I have **not executed Cargo tests in this audit**.

The final evidence record must include nonzero selected-test counts, failures, and revision identity.

### Package B: Conditional residualization

Implement T8 completely, including construction, normalization, specialization, proof propagation, and exact/open parity.

Do not begin generalized match residualization until conditional residual semantics are stable.

### Package C: Match residualization

Reuse canonical P6 match products.

Add the necessary typed residual behavior without a second pattern resolver.

Verify branch-local proof authority and trusted-context isolation.

### Package D: Structural residualization

Implement the existing residual structural families operationally.

Ensure fully reduced structures collapse into canonical `StaticValueStore` values.

Certify recursive source-defined algorithms and structural performance.

### Package E: Resources and incrementality

Close the remaining growth-accounting requirements and eliminate arena-identity-based symbolic product fingerprinting.

Verify cold/incremental equivalence and dependency invalidation.

### Package F: Final P7 certification

Execute the named integration, diagnostics, performance, and lifecycle gates.

At this point, P7 should be a stable foundation for P8's durable products rather than an implementation that P8 must repair.

---

# 8. Priority register

| ID | Finding / requirement | Severity | Owner | Status |
|---|---|---|---|---|
| AUD-001 | Residual specializer supports only four families | High | T8–T10 | Open |
| AUD-002 | Symbolic match lacks complete operational residual handling | High | T9 | Open |
| AUD-003 | Incremental symbolic fingerprints use arena IDs | High | T12/P8 boundary | Open |
| AUD-004 | Residual-node budget exists; full growth accounting unproven | Medium–High | T11 | Open |
| AUD-005 | Structural/conditional residual normalization incomplete | High | T8–T10 | Open |
| AUD-006 | Post-correction T7 acceptance must be rerun | High | T7 certification | Open |
| AUD-007 | T0 heterogeneous iteration precision requires verification | Medium–High | T0 | Open |
| AUD-008 | P4 Record-pattern dependency must not be bypassed | Medium | P4/P7 | Dependency |
| AUD-009 | Cold/incremental static-result equivalence unproven | High | T12 | Open |
| AUD-010 | Full diagnostics and performance certification absent | Medium–High | T13 | Open |

These are not all independently demonstrated runtime bugs. AUD-001 through AUD-003 are grounded in inspected source-level gaps. The remaining items are acceptance deficiencies, architectural dependencies, or risks requiring discriminating tests.

That distinction matters: **unfinished implementation must not automatically be reported as a regression in already supported behavior.**

---

# 9. Final architectural ruling

## 9.1 What should be preserved

The following P7 decisions are sound and should not be reopened without contradictory evidence:

**Canonicalization:** The session-owned `TypeStore` remains the type-formation authority.

**Recursion:** Exact application identity, not declaration-name repetition, determines recursive cycles.

**Memoization:** Application caching remains evaluator-local by default.

**Residual semantics:** Residuals are normalized semantic IR, not source AST or VM continuations.

**Specialization:** Captured residual context, application identity, and supplied specialization remain distinct.

**Proof authority:** Existing equivalence, subtype, conformance, and checked branch-proof systems remain authoritative.

**Member projection:** The canonical structural projector supplies projection results.

**Identity:** P7 uses session/store-lineage-local identities; P8 owns durable identity.

**Execution:** Static computation remains bounded semantic execution, not runtime VM execution.

## 9.2 What must change

The remaining work is primarily to **complete and certify the operational semantics over the architecture already established**.

The most consequential corrections are:

1. Extend specialization beyond `Parameter`, `Apply`, `Proof`, and `MemberProjection`.
2. Implement canonical conditional and match residual behavior.
3. Complete structural residual construction and normalization.
4. Ensure expensive residual operations are budgeted appropriately.
5. Replace transient symbolic fingerprint identity at the incremental product boundary.
6. Revalidate T7 after its corrective commits.
7. Complete the explicit full-P7 acceptance matrix.

None of these requirements inherently calls for a second evaluator, an alternative residual ontology, or a redesign of P2–P6.

## 9.3 Completion definition

P7 can legitimately be marked:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

only after:

- T0–T13 have individually documented accepted outcomes.
- The supported residual families have complete normalization and specialization behavior.
- Cold exact evaluation agrees with symbolic specialization.
- Incremental semantic products have correct dependencies and fingerprints.
- Cancellation, cycle detection, and resource limits are correct.
- The named focused and cross-layer tests pass, with unrelated failures classified.
- Diagnostics and performance gates have actual recorded evidence.
- The P7 walkthrough, handoff, and C4 checkpoint have been updated against the final revision.

## 9.4 Final verdict

**P7 is architecturally well advanced but operationally incomplete.**

Its core recursive evaluation and specialization infrastructure exists on `main`. The three major incidents resulted in explicit architectural decisions and corresponding implementation work, rather than unresolved design alternatives.

The remaining work is significant. In particular, T8–T10 represent an extension from a four-family specialization engine into the richer static residual semantics promised by the checkpoint. T11–T12 then determine whether that system is sufficiently bounded and incrementally correct to support the durable artifacts planned for P8.

**My recommendation is to preserve the existing P7 architecture, recertify T0–T7, and complete T8–T13 in the six bounded packages above.** Do not promote P7 based solely on the existing T7 evidence, and do not defer P7's semantic fingerprint correctness to P8 under the assumption that serialization will resolve it.

### Audit limitations

This audit inspected the connected GitHub `main` branch, its recent commit history, and selected live implementation paths against the supplied plans and incident records. It was a **read-only source audit**, not a complete line-by-line examination of every P7 consumer or a freshly executed test run. Consequently, the report identifies confirmed source gaps and the requirements needed for completion, but does not claim exhaustive proof that no additional defects exist.