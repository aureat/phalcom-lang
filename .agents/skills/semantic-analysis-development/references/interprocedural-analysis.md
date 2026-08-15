# Interprocedural Analysis and Callable Summaries

## Why summaries

A call site needs useful callee facts without embedding the callee's internal AST/flow state.
Summaries provide a finite contract for fixed-point solving and invalidation.

## Current summary structure

Current `CallableSummary` includes:

```text
CallableId
Vec<InferredValue> params
InferredValue returns
Vec<CallableId> dependencies
SummaryEffects
SemanticGeneration revision
```

Extend only when a new cross-call semantic question cannot be answered from existing fields.

## Parameter facts

Observed arguments at resolved call sites can be joined by `(CallableId, parameter name)`.
This is useful for advisory LSP inference.

Do not automatically treat observed parameter facts as normative types. External/dynamic call
sites may exist.

## Mapping arguments to parameters

Respect selector labels and pack semantics. A robust call-site record should include:

- static label if known;
- value fact;
- source range;
- referenced binding if any;
- literal block effects if any;
- whether dynamic pack prevents exact mapping.

Do not zip positional vectors blindly for labeled/rest calls.

## Return summaries

For advisory shape:

```text
join all reachable explicit return values + reachable tail value
constructor -> guaranteed instance when semantics says so
no evidence -> Unknown
```

For future language type inference, use the typing spec's join/Unit/recursion rules separately.

## Dependency extraction

Summary dependencies should include statically resolved callable targets whose summary can affect
this callable's summary/effects.

Dynamic sends should set conservative effect/unknown dependency state rather than fabricate a
specific edge.

## Reverse dependencies

Maintain:

```text
callee -> set(callers/dependents)
```

so a changed callee summary can enqueue only affected modules/callables.

Rebuild this map from solved summaries or update transactionally. Do not leave stale reverse
edges after file removal.

## Fixed point

Generic solver:

```text
seed summaries
repeat affected SCC/frontier:
  analyze callable using current summaries
  join/replace according to analysis definition
  if summary changed -> enqueue dependents
until stable
```

The current engine solves affected state in project batches; inspect `infer.rs`/`engine.rs`
before adding another loop.

## Recursion

### Advisory runtime-shape inference

Can conservatively seed recursive returns/parameters as unknown and iterate. Widen if shape
growth exceeds limits.

### Future typed return inference

Typing proposal may require explicit return annotations for recursive methods. That is a checker
policy and can coexist with advisory summary inference.

Do not force the LSP to lose all advisory information because checker inference intentionally
rejects recursive omission.

## Higher-order callables

Current effects can identify callable parameter positions that are invoked. Propagate a literal
block's effects only through parameters known to be invoked.

Future summary dimensions:

```text
invokes parameter may/must
invocation cardinality
escapes/stores parameter
invokes synchronously/deferred
invokes on another fiber
```

Add only if language/library semantics need them.

## Dynamic calls

A dynamic call can affect:

- return precision;
- effect closure;
- invalidation assumptions;
- proof soundness;
- optimizer call graph.

Use conservative fallback. For editor shape inference, unknown return may be acceptable. For a
purity/no-yield proof, dynamic call may force `Unknown`/`MayEffect` unless constrained by a type
or sealed surface.

## Native calls

Native methods need semantic summaries from trusted metadata/source stubs. Include effects that
matter:

- return shape/type;
- may throw;
- may block/yield;
- mutation;
- callback invocation;
- FFI safety boundary.

A missing native summary should degrade to conservative unknown, not a guessed pure call.

## Summary versioning

Summary equality should reflect fields that affect dependents. Provenance-only changes may not
need to invalidate all dependents unless diagnostics query the provenance transitively.

Separate semantic hash/version from display/debug metadata if this becomes a performance issue.

## Tests

- one-hop return propagation;
- two-hop propagation;
- parameter call-site join;
- recursion;
- mutual recursion;
- changed callee invalidates caller;
- unchanged summary does not fan out unnecessarily;
- dynamic call widens/marks effect;
- higher-order block propagation;
- file removal removes summaries/reverse edges;
- same selector in different modules/classes remains distinct.

## Formal summary equations

Let each callable `c` have a summary `S_c` in domain `D`. Let `Deps(c)` be statically resolved callees and let `Analyze_c` compute a new summary from the current environment of summaries and call-site facts:

```text
S_c' = Analyze_c({S_d | d ∈ Deps(c)}, Params_c, Surface_c)
```

The workspace solver seeks a simultaneous fixed point:

```text
S = F(S)
```

For a recursive SCC `C`, solve all members together because no topological order exists inside the SCC. Callers outside the SCC consume only the stable/widened summary, not arbitrary partial body state.

If summary transfer is monotone over a finite-height/widened domain, chaotic/worklist iteration terminates. A hard iteration budget is a resource guard, not the mathematical reason a solver is correct. If the budget is exceeded, publish a coherent conservative state; do not expose half of one round.

CURRENT `infer.rs` already follows this principle in an important way: cancellation returns no partial result, and the full-workspace fallback widens summaries/parameter facts coherently if a derived solver budget is exceeded in release builds. Preserve this publication property when extending the summary domain.

## Context sensitivity

A context-insensitive summary computes one fact per `CallableId`:

```text
Summary : CallableId -> D
```

This is scalable but merges all callers. A context-sensitive analysis uses a key such as:

```text
(CallableId, CallString_k)
(CallableId, ReceiverClass)
(CallableId, AbstractArgumentTuple)
```

Context sensitivity can dramatically improve precision but multiplies states and invalidation edges. Do not introduce it globally because one completion case is imprecise.

Use a precision ladder:

```text
0. context-insensitive summary
1. receiver-sensitive dispatch result
2. bounded argument specialization for selected facts
3. k-limited call strings for analyses that prove benefit
4. demand-driven specialized contexts
```

Every added context dimension requires a canonical key, cap/merge policy, cache lifetime, and invalidation dependency set.

## Input-sensitive summaries versus observational parameter facts

Current parameter contributions answer “what analyzed callers passed.” A future checker might instead need a function summary parameterized by assumed input types/refinements:

```text
S_c : InputAbstractState -> OutputAbstractState
```

These are different architectures. The former accumulates observations; the latter computes a transformer. Do not make checker correctness depend on observed call sites because unobserved/dynamic callers may exist.

A staged bridge can reuse identities and CFG while keeping separate products:

```text
ObservedParameterFacts   // advisory, open-world evidence
TypedCallableSignature   // normative declared/inferred contract
AbstractTransformer      // optional proof/optimization analysis
```

## Call graph precision

A dynamic language has several call-graph edge strengths:

```text
ExactTarget(c)
BoundedTargets({c1,...,cn})
OpenFamily(selector/receiver constraint)
UnknownTarget
```

Do not store only `Vec<CallableId>` if consumers also need to know that the set is incomplete. A bounded known set and an exhaustive closed set have different effect/proof meaning.

Conceptually:

```rust
enum CallTargetSet {
    Closed(BTreeSet<CallableId>),
    Open { known: BTreeSet<CallableId>, reason: DynamicReason },
    Unknown(DynamicReason),
}
```

This is a future representation sketch. The required invariant is explicit completeness.

## Return-flow equation

For a callable with reachable exits `E`:

```text
Return(c) = ⊔ { value(e) | e ∈ E_explicit_return }
          ⊔ tail_value(c)  if normal tail completion returns it
```

Abrupt paths that throw/non-locally-return do not contribute normal return values. A callable that cannot return normally should not be summarized as “returns Unknown”; the analysis should eventually represent no-normal-return separately because consumers such as reachability and proof need that distinction.

## Recursion worked example

```phalcom
method even(n) {
    if n == 0 { return true }
    odd(n - 1)
}

method odd(n) {
    if n == 0 { return false }
    even(n - 1)
}
```

For shape inference, seed both returns conservatively, analyze, then iterate the SCC until both stabilize at `Bool` shape. For a future polymorphic/type inference system, recursive inference may require declared signatures or a different constraint solution; do not infer language policy from this advisory fixed point.

## Higher-order summaries as effect transformers

A callback-taking method may need summary information such as:

```text
invokes(callback_0): May
escapes(callback_0): No/May
invocation_phase(callback_0): Synchronous | Deferred | Unknown
```

Then call-site analysis can incorporate block effects only if invocation semantics justify it.

Example:

```phalcom
items each: |x| { shared = x }
```

If `each:` is trusted to invoke synchronously before returning, the captured write affects post-call local state. If the callback is stored for later, the post-call semantics are different. “Parameter is callable” alone is insufficient.

## Effect fixed points

Effects propagate through calls just like return shapes:

```text
Effects(c) = DirectEffects(c)
           ∪ ⋃ Effects(d) for possible callees d
           ∪ DynamicFallbackEffects(c)
```

For may-effects, union is monotone. For guarantees such as `does_not_yield`, intersect guarantees across all possible callees and lose the guarantee at open/dynamic boundaries unless a trusted contract says otherwise.

## Dependency ownership and deletion

Incremental correctness requires dependencies to be replaceable by source contribution. If a callable changes from calling `A` to calling `B`:

```text
old edge c -> A must be removed
new edge c -> B must be inserted
reverse dependents must reflect both changes atomically
```

Appending edges without source ownership creates ghost invalidation and stale call graph facts. File deletion is the strongest test: all summaries, parameter contributions, forward edges, and reverse edges owned by the removed source must disappear from the next published generation.

## Summary semantic hash

As summary dimensions grow, split:

```text
semantic payload   // return/effects/escape/etc. consumed by callers
explanation payload // provenance/debug evidence
publication metadata // generation/source revision
```

Only semantic payload changes should normally re-enqueue semantic dependents. Diagnostics that need transitive provenance can subscribe to a separate explanation version if required.

## Precision/resource policy

Interprocedural blow-up can come from:

- number of call graph edges;
- receiver union alternatives;
- specialized contexts;
- callback effect combinations;
- recursive container shapes;
- provenance;
- module frontier size.

For each bounded dimension define:

```text
cap
widened result
whether soundness is preserved
whether a reason is recorded
whether dependents are re-run after widening
```

Never silently keep the first `N` callees and claim a closed target set.

## Incremental fixed-point invariant

After an edit and eventual quiescence:

```text
IncrementalFacts(new_source) == FullRecomputeFacts(new_source)
```

for semantic payload modulo intentionally non-semantic metadata. This is the primary correctness property of incremental solving. Performance is the additional goal that only the dependency frontier is recomputed.

Test both together: a small rebuild trace can still be wrong if it missed a dependency.

## Review questions

- Is the summary observational evidence or a transformer/contract?
- Is the call-target set exhaustive or merely observed?
- What seeds recursive SCCs?
- What order makes summary updates monotone?
- What guarantees convergence besides an emergency budget?
- Does cancellation publish no partial state?
- Can provenance-only changes cause solver churn?
- Are callback invocation/escape semantics represented where needed?
- What does an unknown dynamic call do to return/effect facts?
- How are old dependency edges removed after edits/deletion?
- Does incremental convergence equal clean recomputation?
