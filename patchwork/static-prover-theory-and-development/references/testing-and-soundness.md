# Testing the Static Prover

## Negative tests are primary

A prover that proves true samples is easy to fake. Test programs where a tempting but unsound rule would incorrectly prove safety.

## VC unit tests

Given tiny IR, snapshot normalized VCs and check source mapping.

## Solver-independent tests

Test simplifier, substitution, SSA/heap versioning, loop VC generation and proof-result propagation without a solver.

## Solver tests

For each supported theory:

```text
valid -> unsat negation -> Proven
invalid -> sat -> Disproven model
unsupported/timeout -> Unknown
```

## Mutation testing

Mutate conditions/contracts/operators and ensure proofs fail where expected. This is exceptionally valuable for catching vacuous VCs.

Examples:

- `>` -> `>=`;
- remove precondition;
- swap branch;
- drop heap write;
- mark call pure incorrectly.

## Differential runtime testing

For `Disproven` concrete models, run reconstructed inputs when safe/possible and verify runtime violates the obligation. This validates encoding-to-runtime correspondence.

## Vacuity tests

Detect proofs caused by inconsistent assumptions/preconditions. Warn or test unsatisfiable `requires` separately.

## Loop tests

Cover:

- valid invariant;
- invariant not initially established;
- invariant not preserved;
- insufficient invariant -> Unknown/failure;
- break/continue;
- zero iterations.

## Native conformance

Execute native primitives against generated inputs to check their trusted contracts. A trusted contract mismatch is a release-blocking soundness issue.

## Incremental tests

Proof result after edit must equal clean full rebuild. Changing callee contract/native summary must invalidate callers.

## Fuzzing

Fuzz logic IR and AST lowering for panics, solver crashes, malformed terms and inconsistent source provenance. Never treat solver exception as proof.

---

## Deep treatment: testing the semantic correspondence

### Test the layers independently

A sound prover has multiple failure surfaces. Organize tests by translation boundary:

```text
source -> semantic IR
semantic IR -> proof IR
proof IR -> VC
VC -> solver encoding
solver model -> Phalcom witness
proof result -> runtime/checker policy
incremental dependencies -> reused result
```

A passing end-to-end sample does not identify which boundary is correct.

### Golden VC tests

For tiny proof IR inputs, snapshot normalized formulas. Goldens should be stable under irrelevant source changes but intentionally update when proof-model semantics change. Include source-origin tables where practical.

Example mutation: remove the precondition obligation at a call. A negative test must fail even if current sample callee happens to accept the arguments at runtime.

### Semantic differential tests

For finite domains, exhaustively compare runtime executions against proof encoding. Example: small integer ranges for branch/arithmetic properties. If prover reports `Proven`, no tested runtime state should violate; if `Disproven` returns concrete witness, runtime should reproduce when the model is fully concrete and safe to execute.

This does not prove the prover sound, but it catches encoding mismatch aggressively.

### Metamorphic tests

Useful properties:

```text
alpha-renaming locals preserves proof status
formatting/whitespace preserves semantic obligation identity where intended
inserting a verified redundant assertion does not turn a valid proof invalid
reordering independent declarations does not change proof
full recomputation == incremental recomputation
verified explicit type annotation equal to inferred type preserves proof result
```

For dynamic/reflection-sensitive code, metamorphic transforms must preserve semantic world assumptions.

### Unsoundness mutation suite

Automate or manually maintain mutations that mimic classic prover bugs:

- replace `mayWrite` union with intersection;
- drop one dynamic target;
- treat `unknown` as unsat;
- model Float as Real;
- omit loop back-edge;
- preserve heap across unknown call;
- assume current method's own ensures;
- ignore exception path;
- reuse stale contract hash;
- remove class/world revision from cache key;
- accept impossible solver model as witness;
- erase non-local return edge.

A high-quality suite proves these mutations are detected.

### Property tests for core algorithms

For term interners/canonicalization:

```text
intern(x) == intern(x)
normalize(normalize(x)) == normalize(x)
```

For substitution, use capture-free logic-term substitution and check free-variable laws. For SSA, ensure each definition dominates uses in generated proof control representation.

### Fuzzing malformed source

Feed parser-recovery outputs into semantic/proof query boundaries. Required outcomes:

- no panic;
- no false proof from missing/recovered expression;
- unaffected complete callable can still be proved if dependencies are valid;
- diagnostics distinguish syntax recovery from proof unknown.

### Solver fault injection

Mock backend outcomes:

```text
Unknown
Timeout
Crash
MalformedModel
ProtocolError
```

Verify none become `Proven`, none crash the LSP/compiler unless explicitly fatal internal policy says so, and diagnostics remain deterministic.

### Native conformance

For every trusted native summary, generate/exercise values near semantic boundaries:

```text
empty/non-empty collections
UTF-8 boundary cases
integer extremes relevant to representation
NaN/infinity/signed zero for Float
aliasing inputs
error/exception paths
callback/reentrancy cases
```

A trusted-contract mismatch is a prover soundness defect, not a normal test failure to waive.

### Proof-dependent optimization tests

When compiler removes a check based on proof, test both:

1. proof present and assumptions stable -> optimized behavior equals checked behavior;
2. dependency/world changes -> proof invalidates or runtime guard fails, check is not incorrectly omitted.

### Performance regressions

Track formula nodes, solver calls, wall time, peak memory, cache hit rate, and invalidation frontier. Performance tests must not weaken unknown/soundness behavior to hit latency targets.

### Release gate questions

- Which unsound mutation would this feature be vulnerable to?
- Is there a negative test for that mutation?
- Can every `UnknownReason` path be produced in tests?
- Does incremental output exactly match clean rebuild?
- Are trusted summaries executable against native implementation?
- Are proof-dependent runtime optimizations separately tested?
