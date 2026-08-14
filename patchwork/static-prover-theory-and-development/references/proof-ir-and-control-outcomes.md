# Proof IR and Control Outcomes

## Why a proof IR is justified

A prover needs a representation that is simpler than the surface AST but richer than raw SMT terms. The AST preserves syntax; the shared semantic IR/CFG should preserve language control/data semantics; proof IR expresses logical obligations and state transitions. These layers should not be collapsed merely to reduce file count.

A proof IR becomes justified when several proof algorithms otherwise reimplement the same transformations: SSA renaming, heap versioning, call summaries, exceptional exits, non-local returns, loop cut points, source provenance, and solver translation. The proof IR should normalize irrelevant syntax while retaining semantic identity and explanation provenance.

The crucial discipline is:

```text
source AST/CST
   -> shared semantic lowering
   -> proof lowering
   -> solver-independent logic
   -> backend encoding
```

Name resolution, selector identity, and dynamic dispatch rules should already be represented semantically before proof lowering. The prover must not guess them again.

## Control outcomes as an algebra

A Phalcom body does not have one continuation. A useful abstract outcome domain is:

```text
Outcome<V,H> =
    Normal(V, H)
  | Return(V, H)
  | Throw(ErrorValue, H)
  | NonLocalReturn(HomeFrameId, V, H)
  | Break(LoopId, H)
  | Continue(LoopId, H)
  | Suspend(SuspensionKind, H, ContinuationId)
```

Not every IR phase needs every variant. For example, `break` and `continue` may be resolved into CFG edges before proof lowering. But the semantic distinction must exist somewhere. Conflating throw with return makes postcondition proof unsound; conflating non-local return with local block return changes higher-order semantics; ignoring suspension can preserve stale shared-state facts.

A continuation-style weakest-precondition interface can make these distinctions explicit:

```text
wp(stmt, K) -> Proposition

K.normal(value, heap)
K.return(value, heap)
K.throw(error, heap)
K.nonlocal(home, value, heap)
K.suspend(kind, heap, cont)
```

For an ordinary expression statement `e`, the lowering evaluates `e` according to Phalcom order and feeds its value/heap into `K.normal`. For `return e`, it feeds `K.return`. For a call, it may split into normal and exceptional continuations according to the callee summary.

## A compact Rust representation

One possible representation is typed IDs into arenas, not recursive boxed source trees:

```rust
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct TermId(u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct PropId(u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct HeapVersionId(u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct OriginId(u32);

enum TermKind {
    Bool(bool),
    Int(BigIntId),
    Local(SsaLocalId),
    ResultValue(CallableId),
    FieldRead { heap: HeapVersionId, object: TermId, field: FieldId },
    AdtCtor { ctor: ConstructorId, args: SmallVec<[TermId; 2]> },
    Ite { cond: PropId, then_t: TermId, else_t: TermId },
    Uninterpreted { symbol: LogicSymbolId, args: SmallVec<[TermId; 4]> },
}

enum PropKind {
    True,
    False,
    Not(PropId),
    And(SmallVec<[PropId; 4]>),
    Or(SmallVec<[PropId; 4]>),
    Implies(PropId, PropId),
    Eq(TermId, TermId),
    Lt(TermId, TermId),
    Le(TermId, TermId),
    IsCtor { value: TermId, ctor: ConstructorId },
}
```

The exact enum is not a recommendation to copy mechanically. The important invariants are typed identity, explicit sort information, hash-consable/canonical terms, and origin/provenance metadata externalized or indexed so terms can be shared without duplicating large span structures.

## Sort discipline

Every term has a semantic sort:

```text
sort : TermId -> ProofSort
```

The builder should reject ill-sorted terms before the solver sees them. For example, integer comparison cannot accept an arbitrary runtime object term unless a prior projection/type-test establishes an integer value representation. A typed builder can encode this partially at the Rust type level, but a runtime sort table is often necessary for heterogeneous interned terms.

Do not make `Unknown` a universal solver sort. “Analysis does not know the type” is not itself a logical domain. If a value may be any Phalcom object, model a runtime-object sort plus class/tag predicates or stop with a dynamic proof boundary.

## Statements and obligations

Proof IR may include executable-like statements:

```text
Assume(P)
Assert(P, ObligationId)
Assign(x1, term)
HeapStore(H1, H0, obj, field, value)
CallSummary(...)
Havoc(variable/location)
Branch(P, then, else)
```

or compile them immediately into formulas. Keeping an intermediate statement form is useful when diagnostics need program order and when loop/call transformations are easier before formula construction.

`Assume` and `Assert` must remain distinct:

```text
assume(P)  restricts executions under consideration
assert(P)  asks the prover to establish P on all incoming executions
```

A common unsound shortcut is translating both to conjunction. The logical handling differs under WP and symbolic execution.

## Source provenance

Represent provenance as a graph or indexed records, not merely one span per term:

```text
OriginId -> {
    semantic_entity,
    source_range,
    obligation_kind,
    parent_origins,
    explanatory_label,
}
```

A type-derived fact may point to the parameter annotation; an effect fact to a callee summary; a path constraint to a branch expression. This makes it possible to explain a proof or counterexample structurally.

## Worked lowering example

Source-style example:

```phalcom
@requires(x >= 0)
@ensures(result > x)
inc(x) {
  return x + 1
}
```

Conceptually:

```text
entry assumption: x0 >= 0
r0 = x0 + 1
obligation: r0 > x0
return r0
```

VC:

```text
x0 >= 0 => x0 + 1 > x0
```

For exact unbounded integers this is valid. If `+` is actually a dynamically dispatched method send rather than a primitive whose semantic contract is known, the proof lowering must justify the arithmetic interpretation from type/dispatch facts and a trusted or verified `Int#+` summary. Syntactic appearance alone is not enough.

## Calls

A call summary should expose at least:

```text
precondition(args, receiver, heap_in)
normal_post(result, receiver, args, heap_in, heap_out)
exceptional_posts(...)
modifies/effects
control effects
trust/evidence level
```

Lowering a call:

1. Generate an obligation for the precondition.
2. Create fresh result and post-call heap versions.
3. Havoc or functionally update locations allowed by effects.
4. Assume normal postcondition on the normal edge.
5. Create throw/non-local/suspend edges permitted by the summary.

If no sound summary exists, do not preserve facts across the call as though it were pure.

## Interaction with incomplete source

The editor may contain malformed code. Proof IR should not be fabricated from syntactically recovered nodes whose semantics are not established. The semantic layer should mark recovery boundaries. A local proof request can still prove an unaffected callable if its dependencies are complete, but an unresolved expression in the obligation path should yield a structured `Unknown(MalformedSourceBoundary)` or `Unknown(MissingDependency)` rather than a guessed model.

## Implementation failure modes

- Storing raw AST pointers in long-lived proof nodes across revisions.
- Using source spans as proof identity.
- Letting solver backend objects leak into semantic/proof IR APIs.
- Recomputing name/dispatch resolution during solver encoding.
- Erasing exceptional/non-local outcomes during SSA conversion.
- Canonicalizing terms in a way that drops distinct diagnostic origins without retaining a many-to-one provenance map.
- Modeling source evaluation as a pure term when evaluation can execute user code.

## Tests

At minimum test:

- builder rejects sort-invalid terms;
- `assume` and `assert` generate distinct VCs;
- return and throw reach different postconditions;
- nested blocks with non-local return target the correct continuation;
- heap write creates a new version and preserves framed fields;
- call with unknown effects invalidates relevant heap facts;
- canonicalized term DAG retains all source origins;
- malformed/recovered AST does not become a false `Proven` result.

## Review questions

1. Which semantic layer owns selector and target resolution?
2. What are all control outcomes of the construct being lowered?
3. Can evaluation invoke user code, mutate, throw, or suspend?
4. Is each proof term sorted according to the actual language domain?
5. What source and assumption provenance survives canonicalization?
6. Can the backend be replaced without rewriting semantic lowering?
7. Does the representation make stale identity across edits impossible or detectable?
