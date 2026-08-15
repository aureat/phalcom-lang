# HIR and Semantic Lowering

## 1. HIR is justified by shared semantic normalization

A high-level semantic IR (HIR) is useful when source syntax contains several forms with the same downstream meaning, or when several consumers repeatedly reconstruct hidden semantic information. It is not valuable merely because “compilers have IRs.” Phalcom should introduce/extend HIR only where it eliminates duplicated semantic interpretation.

A useful equation is:

```text
source syntax = semantic meaning + presentation distinctions
HIR           = semantic meaning + retained source provenance
```

The subtraction is selective: source distinctions needed by diagnostics, formatter/refactorings, reflection, or exact language behavior must survive directly or through provenance.

## 2. What lowering may normalize

Candidates include:

- canonical selectors and argument-label layout;
- implicit receiver sends;
- setter/subscript assignment forms;
- pattern binding/destructuring operations;
- explicit local/field/global read and write categories;
- `super` sends with an explicit lookup-start owner;
- local versus non-local return targets;
- closure capture identities;
- collection/packs/spreads as normalized evaluation steps;
- condition tests/refinement sites;
- source methods/getters/setters/constructors into a shared callable representation.

Do not lower away distinctions that are semantically observable. In particular, evaluation order, argument-label/selector identity, visibility context, block construction versus invocation, and non-local return ownership must remain exact.

## 3. A possible shape

This is a **RECOMMENDATION**, not current Phalcom API:

```rust
enum HirExpr {
    Literal(Literal),
    Read(BindingOrStorage),
    Write { target: BindingOrStorage, value: ExprId },
    Send {
        receiver: ExprId,
        selector: SelectorId,
        args: Vec<ArgId>,
        lookup: LookupContext,
    },
    MakeClosure {
        body: BodyId,
        captures: Vec<BindingId>,
        home: CallableId,
    },
    Product(ProductExpr),
}

enum HirTerminator {
    ReturnLocal { target: CallableId, value: ExprId },
    ReturnNonLocal { home: CallableId, value: ExprId },
    Throw(ExprId),
}
```

HIR IDs should be snapshot/body scoped unless there is a demonstrated need for cross-generation persistence. Every node should retain a `SourceOrigin` capable of mapping diagnostics/refactorings back to authored syntax.

## 4. Lowering is a semantic function

Write lowering as a relation/function with invariants:

```text
L : SourceNode × ResolutionContext -> HirNode + Diagnostics
```

Correctness requires more than “the HIR looks reasonable.” For executable forms, the important correspondence is observational:

```text
Eval_source(e, σ) ≈ Eval_hir(L(e), σ)
```

for all behavior the lowering claims to preserve: value, side effects, send order, exceptions/abrupt completion, closure capture, and source-observable reflection where applicable.

The HIR itself need not execute in production. This equation states the semantic obligation of analyses/compilers that consume it.

## 5. Evaluation order must be explicit

Suppose a send has receiver and arguments that invoke user code:

```phalcom
makeReceiver().send(first(), label: second())
```

Lowering must preserve Phalcom's normative order. A safe normalized sequence often makes order explicit:

```text
t0 = Eval makeReceiver()
t1 = Eval first()
t2 = Eval second()
t3 = Send receiver=t0 selector=S args=[t1, t2]
```

If selector construction depends on labels/packs, canonicalization must not reorder expression evaluation. Packs/spreads require particular care because expansion may be dynamic and can affect dispatch certainty.

## 6. `super` is not a type cast

A `super` send should retain both the actual receiver semantics and an altered lookup start. A normalized representation can encode:

```rust
LookupContext::Super {
    lexical_owner: ClassId,
    side: DispatchSide,
}
```

Do not lower `super.foo()` into “cast `self` to superclass and send.” That conflates dispatch lookup with runtime receiver identity and will later corrupt typing/reflection/optimization.

## 7. Closures, blocks, and non-local returns

Lowering should resolve captures and control targets without assuming block execution. Example:

```phalcom
method() {
  let x = 1
  let b = || { return x }
  consume(b)
}
```

The HIR needs enough information to say:

- `b` construction captures binding `x`;
- its `return` targets the home callable if Phalcom semantics make it non-local;
- passing `b` to `consume` does not itself prove that the return occurs;
- later effect analysis can reason about `consume` invoking the block parameter.

This is a major reason to normalize control ownership before checker/prover work.

## 8. HIR versus CFG

HIR can remain structured; CFG makes control edges explicit. Do not force one representation to serve both purposes.

```text
AST: source fidelity
HIR: normalized semantic operations + IDs + provenance
CFG: basic blocks/program points + explicit control edges
```

If analyses only need normalized expressions and lexical identities, HIR may suffice. If they need joins, reachability, loops, abrupt completion, or path refinement, lower each callable/body to CFG.

## 9. Recovery-aware lowering

Live editor input may contain missing expressions or malformed calls. HIR should have explicit recovery/poison nodes or skip regions while retaining diagnostics:

```rust
enum HirExpr {
    ...
    Missing(SourceOrigin),
    UnresolvedName { spelling: Symbol, origin: SourceOrigin },
    Invalid { origin: SourceOrigin, reason: RecoveryReason },
}
```

A recovery node is not `Dynamic` and not `Unknown` in the language type system. It is a statement about incomplete source/analysis. Consumers choose how much neighboring information to retain.

## 10. Source mapping

Maintain bidirectional or at least HIR-to-source mapping:

```text
HirId -> SourceOrigin { file, range, syntax_kind, recovery? }
Source occurrence -> semantic target / selected HIR node when needed
```

One HIR node may correspond to multiple source tokens; one source construct may lower to multiple HIR operations. Diagnostics should point at the smallest causally relevant source origin, not an arbitrary lowered temporary.

## 11. Lowering verification

Useful tests:

- AST/HIR golden tests for all selector forms;
- evaluation-order cases where each operand records a side effect;
- `super` lookup start and receiver identity;
- getter/setter/subscript normalization;
- patterns with conditional binding success;
- closure captures, reassignment, non-local return target;
- malformed calls and missing expressions;
- source-range mapping after lowering;
- differential execution for desugarings when a small HIR interpreter/test evaluator is available;
- property: formatting or semantically irrelevant whitespace does not alter semantic identities/lowering apart from source positions.

## 12. Review questions

1. What duplicated semantic interpretation does this HIR node eliminate?
2. Is the source distinction being removed truly semantically irrelevant?
3. Are operand evaluation order and abrupt completion explicit?
4. Does the node retain selector/lookup/access context?
5. Are captures and return targets resolved once?
6. Can malformed source be represented without inventing valid semantics?
7. Can diagnostics map back precisely?
8. Would adding this HIR force formatter/refactoring to reverse-engineer source syntax? If yes, keep source representation alongside it.
