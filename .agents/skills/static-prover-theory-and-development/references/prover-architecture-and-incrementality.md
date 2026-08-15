# Prover Architecture and Incrementality

## Suggested component split

```text
proof-ir / logic terms
contract lowering
vc generation
abstract simplifier
solver interface
proof cache
diagnostics/model reconstruction
```

Keep solver backend isolated from Phalcom AST.

## Semantic IDs

Key obligations by stable semantic entities/program points:

```text
CallableId
ContractId
AssertionId
CFG/IR point ID
ModuleId
Type/protocol IDs
```

Source range is provenance/location, not durable identity across edits.

## Query shape

Conceptually:

```text
prove_callable_contract(CallableId, Revision) -> ProofBundle
prove_assertion(AssertionId, Snapshot) -> ProofResult
```

Inputs include semantic/type/effect summaries by dependency.

## Incremental dependencies

Proof of caller depends on:

- caller body/CFG;
- referenced contracts;
- type relation versions;
- callee summaries/contracts;
- native specs;
- module import resolution;
- solver configuration if proof cache stores backend results.

## Parallelism

Independent callable obligations can be solved concurrently, but solver contexts and immutable semantic snapshots must be thread-safe. Deterministic diagnostics should sort results after parallel computation.

## Budgets

IDE proof mode needs strict latency limits and may run only local/cheap obligations. CLI checker/prover can use deeper budgets. Same core engine, different resource policy.

## Staging

Recommended order:

```text
constant/flow proof
abstract interpretation
simple decision procedures
SMT fallback
```

Avoid invoking SMT for `x` known constant `3`.

## Persistence

Disk proof caches are only worthwhile after semantic IDs/content hashes and trust/version invalidation are robust. Incorrect cache reuse is unsound.

---

## Deep treatment: query architecture, dependency ownership, and publication

### One semantic truth, prover-specific derivations

The prover should not own duplicate scope/name/type/dispatch databases. Its durable inputs are shared semantic facts. It may own derived proof artifacts:

```text
ContractLowering(ContractId)
ProofIR(CallableId)
Obligations(CallableId)
ProofResult(ObligationId)
Counterexample(ObligationId)
```

Each derived query must declare dependencies on shared facts.

### Query graph

A conceptual dependency graph:

```text
source text/revision
  -> parsed/recovered syntax
  -> semantic identities/scopes
  -> module graph / class hierarchy
  -> CFG/control outcomes
  -> type/effect/call summaries
  -> contract normalization
  -> proof IR
  -> obligation set
  -> proof result
  -> diagnostics/runtime-check policy
```

Changing a comment may stop at parse/source provenance; changing a callee `ensures` should invalidate callers that depend on that contract; changing a method body may require re-proving that method but need not invalidate modular callers until its public proof summary changes.

### Dependency granularity

Too coarse:

```text
proof depends on whole workspace revision
```

Correct but slow.

Too fine and missing dependencies:

```text
proof depends only on caller body hash
```

Fast but unsound.

Aim for semantic dependency units: callable body revision, contract revision, type relation generation, class/dispatch surface revision, native summary version, module init summary, proof model version.

### Stable identity versus revision

Keep separate:

```text
CallableId        semantic identity across compatible edits
BodyRevision      current definition contents
SourceRange       current location
ProofGeneration   published proof snapshot
```

Renaming/moving can alter source ranges without changing identity depending on semantic-model design. A stale result attached to a recycled ID is dangerous; use generation checks.

### Snapshots

An editor request should see one coherent snapshot. Proof work can run concurrently from immutable inputs and be discarded if a newer revision supersedes it. Never merge a result computed with old type facts into new source simply because obligation text hashes match.

### Incremental fixed points

If call/effect summaries are interprocedural, they may themselves require SCC fixed points. The prover should consume the published summary generation after that fixed point stabilizes. Avoid cyclic query recursion where proving caller asks for callee proof which asks for caller proof. Recursive proof should be mediated by explicit contracts/SCC rules, not accidental query recursion.

### Cache validity formula

Conceptually:

```text
Valid(result, snapshot) iff
  result.model_version == snapshot.model_version
  and all dependency fingerprints match
  and trust policy permits every assumption
  and solver configuration remains compatible
```

If solver version changes, logically reusable `Proven` might still be valid if proof evidence is backend-independent, but a simple backend-result cache should conservatively invalidate unless compatibility is established.

### IDE versus CLI budgets

Use the same semantic/proof core with different resource policy:

```text
IDE:
  cheap/local obligations
  short solver timeout
  cancellation on edit
  partial diagnostics

CLI checker:
  full project dependency closure
  larger budgets
  deterministic complete reporting for supported obligations

CI/verification mode:
  strict unknown policy
  native conformance/version checks
  optional proof artifact persistence
```

Do not fork semantics between these modes.

### Proof scheduling

Prioritize obligations by dependency and user value:

```text
current file/local assertion
public contract changed
callers affected by changed contract
background low-priority proof
```

Batch independent solver queries but cap concurrency to avoid memory/CPU collapse. Solver processes can be expensive; profile the concurrency sweet spot.

### Invalidation tests

Construct edit sequences:

1. prove caller using callee contract;
2. edit only callee body preserving contract and re-prove callee;
3. verify caller cache policy behaves as designed;
4. edit callee contract;
5. verify caller invalidates;
6. edit unrelated module;
7. verify caller does not invalidate;
8. add reflective override/class hierarchy change;
9. verify dispatch-dependent caller invalidates.

Then compare final incremental results with a clean full rebuild.
