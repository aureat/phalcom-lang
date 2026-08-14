# Proof Engineering in Rust

## Objective

The prover should be semantically disciplined and operationally cheap enough to serve both batch checking and editor queries. Rust representations therefore matter, but representation choices must follow proof invariants rather than drive them.

## Typed identity

Use typed newtypes for semantic/proof entities:

```rust
struct ObligationId(u32);
struct ProofTermId(u32);
struct ProofPropId(u32);
struct ContractId(u32);
struct SolverSymbolId(u32);
struct ProofGeneration(u64);
```

Avoid using strings or source offsets as semantic identity. Source positions move under edits; names collide across scopes/modules; solver-generated symbols are backend artifacts.

If entities can be deleted/reused in long-lived stores, use generational IDs or generation-scoped arenas so stale IDs fail loudly instead of aliasing unrelated new entities.

## Arena and interning strategy

Proof formulas are DAGs. Recursive `Box<Expr>` trees cause duplicate allocation and expensive clones. Prefer arenas/interners with immutable nodes:

```text
TermKey -> TermId
PropKey -> PropId
```

Canonicalize only semantics-preserving forms. Useful transformations include:

- constant folding;
- flattening associative `And`/`Or`;
- sorting commutative children under stable IDs;
- deduplicating identical terms;
- normalizing integer literals;
- sharing common subexpressions.

Do not canonicalize operations whose source evaluation order is semantically relevant before they have been lowered into pure logical terms.

## Separate origin from structural identity

Two source expressions may lower to the same proposition. If origin is embedded in the intern key, sharing is lost; if origin is discarded, diagnostics degrade. Keep:

```text
PropId -> structural formula
OriginUseId -> (PropId, OriginId, role)
```

or a many-to-many provenance index.

## Immutable snapshots

Editor and parallel prover work should consume coherent immutable semantic generations. A query should never see class hierarchy from revision N and contracts from revision N+1.

One pattern:

```rust
struct ProverSnapshot {
    semantic: Arc<SemanticSnapshot>,
    proof_inputs: Arc<ProofInputIndex>,
    generation: ProofGeneration,
}
```

Derived caches may be concurrent, but their keys must include generation/content dependencies. Publication of a new snapshot should be atomic at the logical level.

## Borrow-friendly ownership

Long-lived references into mutable arenas make incremental replacement painful and unsafe. Prefer IDs + snapshot-owned immutable storage. Temporary builders can borrow mutably while constructing a generation; published facts should be immutable.

Do not solve borrow-checker friction by cloning whole semantic graphs into the prover. That creates stale shadow state and memory blow-up.

## Solver isolation

Run external/in-process solvers behind a narrow interface:

```rust
trait SolverBackend {
    fn check(&mut self, query: &EncodedQuery, budget: SolverBudget)
        -> Result<SolverOutcome, SolverFailure>;
}
```

Separate:

```text
SolverOutcome = Sat(model) | Unsat(evidence?) | Unknown(reason)
SolverFailure = Crash | ProtocolError | InvalidEncoding | ResourceFailure
```

Backend failure must not panic the compiler/LSP and must never become `Proven`.

If using a native solver library through FFI, isolate unsafe handles, lifetimes, thread-affinity constraints, and cancellation behavior in one module. Do not spread raw solver pointers through semantic code.

## Determinism

Parallel proving is attractive because callables/obligations can often be independent. Deterministic output still matters for snapshots and reproducible builds.

- assign stable obligation ordering;
- sort diagnostics by semantic/source order after parallel solving;
- use deterministic canonical serialization for hashes;
- avoid iteration-order-dependent formula generation from hash maps;
- include solver seed/configuration where backend behavior depends on it.

## Caching

A correct cache definition must name:

```text
key
value
validity condition
dependency set
invalidation event
concurrency policy
memory bound
```

Example:

```text
Key:
  normalized obligation hash
  semantic dependency fingerprint
  proof-model version
  solver/theory configuration
  trust-policy version

Value:
  ProofResult
  evidence/provenance summary
  dependency record
```

A body hash alone is insufficient. A caller proof can become stale when a callee contract, native summary, class hierarchy, or type relation changes.

Use bounded caches (LRU/generation eviction/content-addressed disk store) with metrics. An unbounded term/proof cache can turn long editor sessions into memory leaks.

## Cancellation and budgets

IDE proving must be cancellable. Check cancellation between expensive phases and use backend time/memory limits. Distinguish:

```text
CancelledByNewRevision
BudgetExceeded
SolverTimeout
```

These are `Unknown`/discarded query outcomes, not semantic failures. Never publish proof results computed from an obsolete semantic generation.

## Performance model

Measure separately:

- proof-lowering time;
- term interning/allocation;
- simplification;
- solver serialization;
- solver wall time;
- model reconstruction;
- dependency/invalidation work;
- cache hit/miss rate;
- per-obligation term count and peak memory.

A solver may appear to be the bottleneck when formula duplication in the frontend is the real cause.

## Rust error handling

Use structured errors/reasons. Avoid `String`-only internal failures for soundness-critical boundaries:

```rust
enum UnknownReason {
    MissingLoopInvariant { loop_id: LoopId },
    DynamicBoundary { send_id: SendId },
    UnsupportedTheory { feature: TheoryFeature },
    SolverTimeout,
    BudgetExceeded,
    UntrustedNative { callable: CallableId },
    MissingDependency { id: SemanticId },
}
```

The diagnostic layer can render these; proof policy can branch on them; tests can assert exact semantics.

## Unsafe and native boundaries

The proof engine may itself use unsafe FFI to a solver. This is part of the prover TCB for process integrity, though solver correctness is a separate logical trust issue. Encapsulate unsafe code, validate lengths/handles, and test cancellation/destruction ordering.

Native Phalcom primitives are a different boundary: their semantic contracts are prover assumptions. Keep the implementation-side FFI safety problem distinct from the program-proof trust problem.

## Testing Rust representations

Property tests are valuable for:

- interning idempotence;
- canonicalization preserving evaluation-independent logical meaning;
- serialization round trips;
- hash stability within a declared proof-model version;
- generation mismatch rejection;
- cache invalidation closure;
- deterministic diagnostics under parallel schedules.

Fuzz logic IR builders and solver encoders for panics and malformed solver input. A malformed encoded query should produce an internal prover error/Unknown according to policy, never silently weaken the formula.

## Review questions

1. Can any ID outlive the storage generation that defines it?
2. Are large formula structures cloned or referenced by IDs?
3. Is provenance retained without defeating interning?
4. Can solver cancellation leave poisoned backend state?
5. Is every cache dependency explicit?
6. Is memory growth bounded in a day-long LSP session?
7. Are parallel results deterministic after merge?
8. Does any backend error path accidentally default to success?
