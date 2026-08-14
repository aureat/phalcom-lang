# SMT, SAT, and Decision Procedures

## SAT

SAT decides satisfiability of propositional boolean formulas.

Useful after bit-blasting/boolean abstraction, but program properties usually require richer theories.

## SMT

Satisfiability Modulo Theories combines boolean structure with theories such as:

- equality with uninterpreted functions (EUF);
- linear integer arithmetic (LIA);
- linear real arithmetic (LRA);
- bit-vectors;
- arrays;
- strings/sequences;
- algebraic datatypes;
- floating point;
- quantifiers (incomplete/heuristic in general).

## Choose semantic theory deliberately

Phalcom `Int` is specified as exact/unbounded at the surface direction, so mathematical integers are a natural proof theory for `Int`, not fixed-width bit-vectors—unless proving a particular VM representation operation.

`Float` requires IEEE-754 floating-point theory if exact semantics matter. Treating floats as reals is unsound for NaN, infinities, rounding and signed zero.

## Decidable fragments

Quantifier-free linear arithmetic is much easier than nonlinear integer arithmetic or arbitrary quantifiers. Design contracts to stay in tractable fragments where possible.

## Uninterpreted functions

For pure functions without a full definition, model as uninterpreted plus known axioms/contracts. Too many axioms can make solver behavior unstable or inconsistent if contracts are wrong.

## Algebraic datatypes

Option/Result/sealed ADTs map naturally to SMT datatype constructors/testers/selectors if solver support is used.

## Arrays/maps

SMT arrays model total functional maps, not necessarily Phalcom mutable `Map` semantics directly. Heap/map mutation needs store/select encoding plus alias/effect model.

## Strings

SMT string theories can solve many length/concat/contains constraints but may return unknown/slow on complex regex/Unicode properties. Phalcom `String` is UTF-8 runtime text; prove semantic Unicode-level claims only with an encoding that matches API semantics.

## Solver result

Always handle:

```text
sat
unsat
unknown
error/timeout/resource
```

Treat `unknown` as `Unknown`, never `Proven`.

---

## Deep treatment: choosing theories by semantic obligation

### Solver capability is not one dimension

“SMT supports integers/strings/floats” does not imply every formula in those domains is tractable. The agent should classify obligations by fragment:

```text
QF_LIA   quantifier-free linear integer arithmetic
QF_LRA   quantifier-free linear real arithmetic
QF_BV    bit-vectors
QF_FP    floating point
QF_UF    uninterpreted functions/equality
arrays   extensional functional arrays
ADTs     algebraic datatypes
strings  solver-specific string/sequence fragments
NIA      nonlinear integer arithmetic
quantified combinations
```

Quantifier-free linear arithmetic is robust; quantified nonlinear heap/string formulas can be unstable or undecidable/incomplete in practice.

### Language-level versus representation-level proof

The same Phalcom operation can require different theories depending on the question:

```text
Question: does language Int addition preserve x+1>x for x>=0?
Model: mathematical Int (if surface Int is exact/unbounded)

Question: does tagged VM small-int fast path overflow correctly into big-int representation?
Model: machine bit-vectors + tag/overflow + bignum correspondence
```

Do not let representation details leak into language proof or mathematical semantics hide VM bugs.

### Floating point

IEEE-754 violates many real-number intuitions:

```text
NaN != NaN
x + y may round
+0 and -0 compare equal but can affect reciprocals/sign-sensitive ops
infinities exist
associativity fails
```

Therefore a proof of exact runtime behavior must use FP theory or a verified abstraction whose theorem covers the property. Mapping `Float` to `Real` may be acceptable for an explicitly heuristic analysis that cannot issue `Proven` for FP-sensitive obligations.

### Uninterpreted functions

EUF is useful when only congruence matters:

```text
x = y => f(x) = f(y)
```

But using an uninterpreted function for a Phalcom method silently assumes purity/determinism and ignores heap/effects unless the function is parameterized by state:

```text
f(H, receiver, args) -> (result, H')
```

Usually verified contracts are clearer than universal function axioms.

### Quantifiers

Axioms such as:

```text
∀x. length(append(x,e)) = length(x) + 1
```

may require triggers/instantiation heuristics. Poor triggers cause missed proofs or explosion. Prefer specialized lemmas instantiated at relevant program terms or solver-native ADTs/sequences where possible.

Unknown due to quantifier incompleteness is expected and must be surfaced as such.

### Datatypes

Closed ADTs like `Option<T>` or `Result<T,E>` are a strong fit for SMT datatypes when their language representation is semantically sealed. Useful facts:

```text
Some(v) != None
isSome(Some(v))
value(Some(v)) = v
```

If Phalcom's object model allows reflective construction that violates a sealed ADT abstraction, the solver model must instead encode the actual invariant or rely on a trusted abstraction boundary.

### Strings and Unicode

Runtime UTF-8 storage is representation; public String operations may be defined over Unicode scalar values, grapheme clusters, bytes, or code units depending on API. Solver string theory commonly reasons about Unicode code points or implementation-defined units, not necessarily Phalcom's exact API semantics. Each modeled operation needs a semantic contract matching its unit.

For example, proving byte offsets with a character-length theory is a category error.

### Solver portfolio and preprocessing

A staged engine can route obligations:

```text
constant simplifier
interval/congruence/ADT decision logic
specialized arithmetic procedure
SMT backend
optional secondary backend for cross-check/debug
```

Using multiple solvers can improve robustness but expands integration complexity. Disagreement must never be arbitrated by “pick Proven”; treat it as a soundness alarm.

### Resource limits

Configure:

```text
timeout
memory/rlimit where available
random seed when reproducibility matters
quantifier/resource limits
model production only when needed
```

A timeout is semantically `Unknown`, but operationally it should include theory/size metrics so engineers can improve preprocessing or contracts.

### Review questions

- What exact semantic domain is this term modeling?
- Is the fragment decidable/robust or heuristic?
- Does any approximation allow false `Proven`?
- Are quantifiers avoidable through summaries or instantiation?
- Are String units aligned with Phalcom semantics?
- Are solver failures isolated from compiler correctness?
