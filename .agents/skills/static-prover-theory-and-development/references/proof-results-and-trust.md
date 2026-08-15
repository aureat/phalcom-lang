# Proof Results and Trust

## Proof result lattice

At minimum:

```text
Proven
Disproven(counterexample/witness)
Unknown(reason)
```

Do not merge `Disproven` and `Unknown`: one says a violating execution/model exists in the encoded semantics; the other says the prover could not decide.

## Evidence levels

A proof can derive from:

- syntax/constant fact;
- sound abstract analysis;
- solver proof result (`unsat` for negated obligation);
- verified callee summary;
- trusted native axiom;
- user assumption.

Retain provenance because trust differs.

## Trusted computing base

A prover's guarantee depends on:

```text
parser/semantic lowering
VC generator
abstract-domain soundness
solver encoding
solver correctness
trusted native contracts
runtime/compiler correspondence
```

Minimize handwritten axioms. Every axiom can prove false programs if wrong.

## Caller/callee contracts

When verifying a method:

- assume its `requires` at entry;
- do not assume its own `ensures` to prove itself;
- prove `ensures` for every relevant normal exit;
- prove invariant obligations according to policy.

When verifying a call to a verified/trusted method:

- prove callee `requires` before call;
- assume callee `ensures` after normal return;
- apply effect/frame summary;
- include exceptional outcomes.

## Assume versus assert

Internal IR should distinguish:

```text
assume(P)  narrows analyzed executions, creates no obligation
assert(P)  creates obligation that P holds
```

User contracts are translated according to their role, not all to the same node.

## Trusted native contracts

A Rust primitive can be marked trusted only through controlled metadata/source definitions. User packages must not forge trust flags.

## Proof-carrying cache

Cached `Proven` results should record:

- obligation identity/hash;
- semantic generation/dependency revisions;
- solver/theory configuration version;
- trusted contract versions;
- assumptions used.

If any dependency changes, proof is stale.

---

## Deep treatment: proof judgments, assumptions, vacuity, and trust accounting

### Proof as a judgment

A useful conceptual judgment is not merely `⊢ P`; it records the semantic world and assumptions under which the conclusion was established:

```text
Σ ; Γ ; A ⊢ P ✓
```

where:

- `Σ` is the semantic environment: modules, class hierarchy, selector semantics, callable contracts, native summaries, type relations, and proof-model version;
- `Γ` contains local typed/symbolic facts for the current program point;
- `A` is the explicit assumption set, including preconditions, path conditions, trusted contracts, and world-closure assumptions;
- `P` is the obligation;
- `✓` means the proof engine established validity according to its trusted rules/backends.

In implementation terms, `Proven` should retain a compact dependency/evidence record corresponding to `Σ` and the subset of `A` that matters for invalidation and explanation. It need not retain a giant formal proof object, but it must retain enough to know when the result is stale.

### A richer result type

A production result type should distinguish logical status from operational metadata:

```rust
enum ProofStatus {
    Proven,
    Disproven,
    Unknown,
}

struct ProofResult {
    status: ProofStatus,
    obligation: ObligationId,
    evidence: EvidenceSummary,
    assumptions: AssumptionSetId,
    dependencies: DependencyFingerprint,
    diagnostics: SmallVec<[ProofNote; 2]>,
}
```

For `Disproven`, evidence includes a model/witness plus path feasibility information. For `Unknown`, it includes a precise reason and residual obligation. This lets policy differ between, for example, a missing invariant and a solver timeout.

### Soundness versus completeness

A prover is **sound** for a class of obligations if every reported `Proven` obligation is valid in the modeled Phalcom semantics. It is **complete** if every valid obligation in that class can eventually be proved. Production static provers routinely sacrifice completeness while protecting soundness.

That implies:

```text
valid program/property + weak prover -> Unknown       acceptable
invalid property + sound prover      -> never Proven  mandatory
```

For IDE usefulness, heuristics may produce suggestions or confidence, but they need a separate result channel. Do not widen the definition of `Proven` to improve success rates.

### Trust tiers are semantic inputs

A callee summary may be:

```text
Verified       body proved against contract
TrustedCore    system-owned axiom/native contract
RuntimeChecked contract enforced on executed paths only
Declared       user wrote it but it is not verified
Inferred       static analysis derived approximation
Unknown        no usable summary
```

These are not interchangeable. A strict proof mode may assume only `Verified` and controlled `TrustedCore`. A hybrid typed-runner mode may use `RuntimeChecked` to justify subsequent facts only on the concrete execution, not for universal static proof. `Inferred` summaries are usable only for claims their soundness guarantees cover.

### Vacuity

A VC `Pre => Post` is valid whenever `Pre` is unsatisfiable. That may be logically correct yet operationally useless:

```text
@requires(false)
@ensures(impossibleClaim)
f() { ... }
```

The postcondition is vacuously verified because no legal entry exists. The prover should be capable of distinguishing:

```text
Proven(non-vacuous)
Proven(vacuous because assumptions inconsistent)
```

or at least emitting a separate diagnostic when the precondition/assumption set is unsatisfiable. This is especially important for inferred contracts and generated invariants, where inconsistency can hide a translation bug.

A practical check is to ask satisfiability of the entry assumptions independently when useful:

```text
sat(A)      -> proof may be meaningful
unsat(A)    -> vacuity warning / dead callable contract
unknown(A)  -> do not claim non-vacuity
```

### Evidence composition

Suppose obligation `P` is discharged using a verified callee postcondition `Q`, which itself depended on a trusted native contract `N`. The resulting proof transitively depends on `N`. Dependency/evidence composition must not stop at the immediate callee if cache validity or trust reports need transitive information.

One scalable pattern is content-addressed proof summaries:

```text
Summary(f) = hash(contract, body-proof dependencies, native assumptions, model version)
CallerProof depends on Summary(f)
```

Changing an internal body while preserving a re-verified summary can leave modular callers valid; changing the contract or trust basis invalidates them.

### Proof and runtime modes

A useful policy matrix is:

```text
                           Proven     Disproven     Unknown
strict checker             accept     error         error/warn by policy
runtime-checked typed mode  omit chk*  error         keep runtime check
normal dynamic mode         advisory   advisory      no static claim
optimizer                   may use*   never use     never use
```

`*` only when all proof assumptions are guaranteed by the compiled runtime mode. For example, a proof depending on closed-world dispatch cannot justify check removal in a mode that allows reflective method replacement after compilation unless a runtime guard preserves the assumption.

### TCB decomposition

The trusted computing base is not one blob. Track at least:

```text
source semantic correctness
semantic-to-proof lowering
proof IR sort/normalization correctness
VC/WP algorithm correctness
abstract fact soundness
solver encoding correctness
solver result correctness
trusted contracts/axioms
compiler/runtime correspondence for removed checks
```

This decomposition guides testing. Solver-independent VC snapshots test the frontend. Native conformance tests test trusted contracts. Differential runtime testing validates counterexamples. Proof-dependent optimization tests validate compiler/runtime correspondence.

### Competency questions

- Can a user-declared but unverified `@ensures` be assumed by callers in strict static proof? Under what policy?
- If a solver returns `unknown` after five seconds, which status and reason are emitted?
- If `requires false` makes every postcondition valid, has useful correctness been established?
- If a proof depended on a native `String#length` contract and that Rust primitive changes, what invalidates?
- If the same obligation is proved using a heuristic LSP fact and a solver, which one may justify runtime-check elimination?
