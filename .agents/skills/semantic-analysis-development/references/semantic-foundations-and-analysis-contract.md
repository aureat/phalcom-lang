# Semantic Foundations and the Analysis Contract

This reference owns the foundational model for Phalcom semantic analysis: what an analysis result means, what it is allowed to claim, and how a production implementation connects concrete language behavior to finite static facts. It is deliberately distinct from the future language type system. A `ValueShape`, flow fact, proof fact, optimizer fact, and runtime value may describe the same program point while inhabiting different semantic domains.

## 1. The semantic contract

For every analysis, write down five things before implementation:

1. **Concrete question.** What property of actual Phalcom executions is being approximated?
2. **Abstract domain.** What finite representation answers that question?
3. **Soundness direction.** Is the fact a may-property, must-property, exact fact, heuristic, or proof result?
4. **Transfer semantics.** How does each relevant language operation transform the fact?
5. **Invalidation boundary.** Which source/runtime assumptions make the result valid?

A useful review sentence is:

> For program point `p`, fact `a` safely represents concrete states `γ(a)` under assumptions `A`, and every supported concrete transition from those states is represented by the abstract transfer unless the result is explicitly marked advisory/heuristic.

If that sentence cannot be made precise, the implementation is not ready to become shared semantic infrastructure.

## 2. Concrete and abstract states

Let `Σ` be the concrete machine-state domain needed for the question. For a local value-shape analysis, a concrete state might contain an environment and heap:

```text
σ = (ρ, H)

ρ : RuntimeBinding -> RuntimeValue
H : ObjectIdentity -> ObjectState
```

The complete VM state is often unnecessary. The point is to define enough concrete meaning to know what the static abstraction is approximating.

Let `A` be an abstract domain. An abstraction function `α` maps sets of concrete states into abstract facts, while a concretization function `γ` describes which concrete states an abstract fact represents:

```text
P(Σ) --α--> A
P(Σ) <--γ-- A
```

The implementation normally does not compute `α` or `γ`; they are the semantic contract behind `join`, transfer functions, widening, and tests.

For a may-analysis, the abstract fact must not omit possible concrete behavior. Informally:

```text
reachable_concrete_states(p) ⊆ γ(analysis_fact(p))
```

For a must-analysis, the order is interpreted differently: a claimed property must hold for every represented concrete execution.

## 3. Soundness, precision, completeness, usefulness

These terms must not be conflated.

- **Sound** for a may-analysis means no modeled possible behavior is excluded.
- **Precise** means the abstraction excludes many impossible behaviors.
- **Complete** means the abstraction loses no information for the property, which is rarely achievable for a dynamic language.
- **Useful** means the consumer can act on the result.

An editor heuristic may intentionally be useful without being sound. That is acceptable only when the fact is represented and consumed as heuristic. The same fact must not later be interpreted as checker proof.

A common failure is:

```text
no observed call site passes String
therefore parameter is not String
```

This is not sound in an open-world dynamic program. At best, observed call-site evidence says what analyzed callers have passed.

## 4. Information order versus language subtyping

An abstract-domain order `⊑` is not automatically Phalcom subtyping `<:`.

For an information domain, `a ⊑ b` typically means “`b` is at least as conservative / represents at least the concrete possibilities represented by `a`.” In a possible-shape domain:

```text
Instance(Integer) ⊑ Integer | String ⊑ Unknown
```

That ordering describes analysis precision. It does **not** assert that `Integer <: Integer | String` is a language rule, even if a future type system happens to have an analogous rule.

Review every relation by name. Never use one generic function called `compatible` to stand in for:

```text
abstract precision
language subtyping
assignability
runtime instance-of
selector dispatch compatibility
gradual consistency
proof implication
```

## 5. Exactness is relative to a proposition

`Exact` should mean exact with respect to a named proposition, not “omniscient.” A literal `1` can provide an exact fact that its evaluated value is an instance of the integer class, while saying nothing exact about object identity, future mutation, integer range, or effect behavior.

Thus:

```text
Exact(ValueShape::Instance(Integer))
```

should be read as “the runtime-shape proposition is exact at this site,” not “all semantic properties are known exactly.”

This becomes critical when product facts are introduced. A value can have exact shape but unknown aliasing and may-throw effects.

## 6. Product analyses

Real semantic facts are frequently products:

```text
A = Shape × BoolConst × Reachability × Effect × ProvenanceMeta
```

The order and join are componentwise only when the components are independent enough for that representation. If components constrain one another, reduction may improve precision:

```text
Shape = Bool, BoolConst = true        valid
Shape = String, BoolConst = true      inconsistent -> reduce/widen/error internally
```

Do not silently retain impossible products. Either define a reduction operation or keep the components separately with explicit invariants.

## 7. May and must polarity

Before adding a boolean flag, decide its polarity.

Examples:

```text
may_throw      : false ⊑ true       // learning that throwing is possible widens behavior
may_yield      : false ⊑ true
must_return    : true ⊑ false       // merge loses guarantee if one path may not return
is_initialized : must property at program point
```

A field named `pure: bool` is dangerous because absence of observed impurity is not proof of purity. Prefer `effects: MayEffects` plus a separate proven-purity fact if needed.

At a join point:

```text
MayEffect.join  = set union
MustFact.join   = logical intersection / meet of guarantees
```

The implementation representation may use bitsets, enums, or maps, but polarity must be documented.

## 8. Transfer functions must model evaluation, not syntax resemblance

Suppose Phalcom evaluates a send:

```phalcom
receiver foo: argument
```

A semantic transfer must respect actual dynamic evaluation order:

```text
1. evaluate receiver
2. evaluate argument(s) in language-defined order
3. form selector identity
4. perform lookup beginning at the correct dispatch origin
5. invoke target/fallback/dynamic path
6. apply call effects
7. produce value or abrupt completion
```

The analysis may approximate steps, but cannot reorder them merely because the AST visitor is convenient. If evaluating an argument may mutate receiver-visible state, the result after the call must reflect that possibility.

## 9. Abrupt completion is semantic state

A transfer should distinguish normal continuation from abrupt outcomes:

```text
TransferResult<A> = {
    normal: Option<A>,
    returns: A_return,
    throws: A_throw,
    breaks: Map<LoopTarget, A>,
    continues: Map<LoopTarget, A>,
    non_local_returns: ...
}
```

An implementation need not literally use this Rust shape. The invariant is that terminated paths do not contribute to later normal flow.

Tempting but wrong:

```text
analyze return expression
continue traversing following statements
join facts from unreachable statements
```

That creates impossible evidence and can corrupt return summaries, definite assignment, refinements, and diagnostics.

## 10. Dynamic operations create semantic barriers, not universal blindness

A reflective or unresolved operation should invalidate only facts that depend on assumptions the operation may break.

Examples:

- Unknown call return -> return fact may become `Unknown`.
- Unknown call that may mutate arbitrary fields -> field refinements depending on heap stability may be killed.
- Unknown call does not necessarily invalidate immutable lexical bindings whose values cannot be changed.
- Reflection that can mutate method dictionaries affects dispatch-related assumptions and optimizer caches, not necessarily local integer constant facts.

Design barrier effects explicitly. “Set everything to Unknown after any dynamic send” is sound but can destroy editor usefulness; “ignore dynamic sends” is precise-looking but unsound. The correct abstraction tracks which semantic dimensions are endangered.

## 11. Recovery facts are not language facts

Editor analysis operates over malformed programs. Preserve a three-way distinction:

```text
RecoveredSyntax     // parser produced a stable-enough recovery node
SemanticUnknown     // valid question, insufficient semantic knowledge
LanguageError       // completed construct violates language rules
```

Example while editing:

```phalcom
users map: |u| { u.
```

The parser may recover the block, parameter `u`, and member-access prefix. The semantic engine can still publish the binding identity and receiver fact for `u`; it must not pretend the incomplete selector is a valid send.

Downstream consumers should be able to ask whether a fact depends on recovery so diagnostics can avoid cascading noise.

## 12. Provenance is part of explanation, not necessarily lattice equality

A semantic value and its explanation have different convergence requirements.

```text
semantic equality: shape/effect/refinement changed?
explanation equality: evidence set changed?
```

If every newly discovered source path changes fixed-point equality because provenance grows, recursive analysis may fail to converge even when semantic facts stabilized long ago. Keep provenance bounded and, where needed, version it separately from semantic summary hashes.

## 13. Current Phalcom mapping

CURRENT repository semantics include an advisory `ValueShape` with `Unknown`, class/module/callable/container shapes, bounded unions, `InferredValue` confidence/provenance, local facts, field facts, parameter contributions, callable summaries, and immutable semantic snapshots. The source explicitly documents `ValueShape` as “deliberately not a language type.”

Treat that as an architectural boundary:

```text
runtime execution
      ↓ abstract observation
ValueShape / InferredValue          CURRENT advisory engine
      ↕ explicit bridge
TypeId / constraints / judgments   FUTURE typed language
      ↕ explicit bridge
ProofFact / VC result               FUTURE prover
```

A future checker may use exact shape facts as evidence, but it must translate them through a defined rule, for example:

```text
shape_to_type_fact(Instance(C)) = Nominal(C) only when the runtime-class/type correspondence is valid
shape_to_type_fact(Unknown)     = no derived type fact
```

Never implement the bridge as `Type = ValueShape`.

## 14. Review obligations for every new fact

For a proposed fact such as “receiver is definitely a `String` here,” demand answers to:

1. What concrete executions does the fact quantify over?
2. Is it may, must, exact, heuristic, or proven?
3. What invalidates it: local assignment, captured write, field mutation, unknown call, yield, reflection, module reload?
4. What is its join?
5. Is there bottom/unreachable?
6. Can chains/unions grow without bound?
7. Does provenance affect semantic equality?
8. Which consumer may rely on it for correctness?
9. Can malformed source produce it?
10. Can clean full analysis and incremental analysis disagree?

If any correctness consumer relies on the fact, the answers must be stronger than “it seems to work for completion.”

## 15. Competency pressure tests

An implementation agent using this reference should reject or qualify each of these:

### “We saw only integers at call sites, so the parameter type is Integer.”

Reject as a normative typing conclusion in an open world. Accept as bounded interprocedural runtime-shape evidence if the fact is marked with the correct provenance/confidence.

### “The solver timed out, so the contract is false.”

Reject. Timeout means `Unknown`/resource exhaustion, not refutation.

### “A dynamic call means all locals become Unknown.”

Reject as unnecessarily destructive. Invalidate dimensions affected by the call’s conservative effect model.

### “`Unknown` is the top type.”

Reject. Current `ValueShape::Unknown` is top-like in one analysis precision domain; it is not automatically the future language top type, `Any`, or dynamic type.

### “We can preserve a branch refinement across `fiber yield` because the local variable itself was not assigned.”

Only if the refined proposition is stable across suspension. A field/property reachable through shared mutable state may change while the fiber is suspended.

## 16. Implementation review questions

- Can the fact be stated as a proposition about concrete execution?
- What is the order `⊑` and why does iteration move monotonically?
- Is `join` a least upper bound or a conservative approximation to one?
- Does widening preserve the soundness direction?
- Does an unknown reason survive far enough to render a useful diagnostic?
- Is there an explicit conversion when a checker/prover consumes an advisory fact?
- Is any runtime representation detail being mistaken for a language semantic distinction?
- Could reflection or open-world mutation invalidate the assumption?
- Does cancellation discard unpublished partial work?
- Can the same result be recomputed deterministically from a clean snapshot?
