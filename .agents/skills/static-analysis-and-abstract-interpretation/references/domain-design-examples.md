# Abstract Domain Design Examples

An abstract domain is not “an enum of guesses.” It defines a mathematical approximation: an order, join/meet where needed, top/bottom, transfer functions, widening if chains are unbounded, semantic equality, provenance policy, and a soundness contract. This reference develops small domains that can be reused as building blocks in Phalcom analysis. They are examples, not a mandate to combine everything into one mega-domain.

For each domain, ask first: what concrete property does it approximate, and which consumer needs it?

## 1. Constant domain

### Concrete property

Exact runtime scalar value, when statically known.

### Abstract elements

```text
Const#(V) = ⊥ | Const(v) | ⊤
```

Order:

```text
⊥ ⊑ Const(v) ⊑ ⊤
```

Different constants are incomparable except through `⊤`.

Join:

```text
⊥ ⊔ a = a
Const(1) ⊔ Const(1) = Const(1)
Const(1) ⊔ Const(2) = ⊤
a ⊔ ⊤ = ⊤
```

### Transfer

```text
Const(2) + Const(3) = Const(5)        # if primitive semantics trusted
Const(2) + ⊤        = ⊤
```

For Phalcom, arithmetic is message dispatch. Constant folding is sound only when the invoked operation is semantically fixed/pure or the optimizer inserts guards. Do not import primitive-integer semantics into arbitrary overloaded sends.

### Tests

Join algebra, exact equal constants, branch join, operation fallback, dispatch override/reflection assumptions.

## 2. Finite runtime class/shape domain

### Concrete property

Possible runtime receiver categories/classes, not language types.

```text
Shape# = ⊥ | Finite({ShapeAtom...}) | ⊤
```

For bounded advisory analysis:

```text
if |set| > K => ⊤
```

The current LSP `ValueShape` is richer and uses `Unknown` plus bounded `Union`, with `MAX_SHAPE_UNION = 8` at the inspected baseline. It also structurally joins collection shapes. Treat that as a concrete CURRENT implementation, not as the universal formal domain for checker typing.

### Join example

```text
Instance(String) ⊔ Instance(Int)
    = Union[String, Int]
```

If a third analysis path already widened to `Unknown`:

```text
Union[String, Int] ⊔ Unknown = Unknown
```

under the current advisory convention.

### Hazard

A correctness checker cannot necessarily reuse “oversized set => accept as unknown.” Its fallback must preserve the checker's soundness direction.

## 3. Flat nominal-class domain versus powerset domain

A flat class domain:

```text
⊥ | Class(C) | ⊤
```

loses all information as soon as two classes meet.

A finite powerset domain:

```text
⊥ | Classes({C...}) | ⊤
```

keeps bounded alternatives but costs set operations. This is a classic precision/cost choice.

Use product structure for class-side distinction:

```text
ReceiverAtom = Instance(C) | ClassObject(C)
```

rather than inferring side from `ClassId`.

## 4. Sign domain

Concrete integers abstract to signs:

```text
Sign# = P({Neg, Zero, Pos})
```

Examples:

```text
{Pos} + {Pos} = {Pos}
{Neg} * {Neg} = {Pos}
{Neg,Zero} * {Pos} = {Neg,Zero}
```

This finite powerset has no widening requirement. It is cheaper but less precise than intervals.

Use cases: obvious divide-by-zero checks, monotonic loop facts, contract linting. For Phalcom numbers, only apply primitive arithmetic rules where dispatch semantics justify them.

## 5. Interval domain

### Abstract elements

```text
Interval# = ⊥ | [l, u]
where l,u ∈ Z ∪ {-∞,+∞}, l <= u
```

Order by set inclusion:

```text
[l1,u1] ⊑ [l2,u2]
iff l2 <= l1 and u1 <= u2
```

Join:

```text
[l1,u1] ⊔ [l2,u2]
    = [min(l1,l2), max(u1,u2)]
```

Meet/intersection:

```text
[l1,u1] ⊓ [l2,u2]
    = [max(l1,l2), min(u1,u2)]
```

or `⊥` if empty.

### Transfer

```text
[a,b] + [c,d] = [a+c, b+d]
[a,b] - [c,d] = [a-d, b-c]
```

Multiplication requires min/max of endpoint products and careful infinity handling.

### Loop

```text
x = 0
while cond { x = x + 1 }
```

naive joins may produce:

```text
[0,0] -> [0,1] -> [0,2] -> ...
```

so widening is required.

### Phalcom caution

Do not apply interval transfer to a user-overridable `+` send unless target semantics are trusted or guarded.

## 6. Congruence domain

Track modular properties:

```text
x ≡ a (mod n)
```

Examples:

```text
x ≡ 0 mod 2   # even
x ≡ 1 mod 2   # odd
```

Useful for alignment/stride reasoning. A reduced product with intervals can derive stronger facts:

```text
interval:   x ∈ [0, 1]
congruence: x ≡ 0 mod 2
=> x = 0
```

Only add reduction if a consumer benefits.

## 7. Presence / Option-state domain

For runtime/control reasoning independent of the full language type:

```text
Presence# = ⊥ | NoneOnly | SomeOnly | Maybe
```

or with payload:

```text
OptionFact#(A) =
    ⊥
  | NoneOnly
  | Some(A)
  | Maybe(A)
```

Join:

```text
NoneOnly ⊔ Some(T) = Maybe(T)
Some(A) ⊔ Some(B)  = Some(A ⊔ B)
```

A trusted presence test refines edges. The checker should represent formal `Option<T>` in its own type domain; the path/presence domain may bridge to it.

## 8. Boolean truth domain

The current LSP carries `known_boolean: Option<bool>` alongside shape. A more algebraically explicit domain is:

```text
Bool# = ⊥ | False | True | Either
```

This makes `⊥` distinguishable from “unknown boolean.” It can drive branch reachability.

Truth tables can be abstracted exactly for built-in boolean operations where semantics are fixed.

## 9. String domain

Possible product abstraction:

```text
String# =
    ExactLiteral(short?)
  × LengthInterval
  × PrefixSet
  × SuffixSet
  × EncodingValidity
```

Applications:

- exact reflective selectors;
- static path/security lints;
- bounded label analysis;
- format-string checks.

Unbounded exact strings are a memory hazard. Keep literal length/set caps explicit. Automata/regex domains are substantially more expensive and should have a concrete consumer.

## 10. Selector domain

Phalcom-specific domain:

```text
Selector# =
    ⊥
  | Exact(SelectorId)
  | Finite(Set<SelectorId>)
  | Family { base, arity_range?, known_labels? }
  | Dynamic
```

Useful for reflective perform, dynamic packs, method families, signature help, and call graphs.

Join examples:

```text
Exact(foo(_)) ⊔ Exact(foo(_:bar:))
    -> Family(base=foo, ...)
```

or a finite set, depending on consumer and representation.

Do not place type annotations in selector identity.

## 11. Collection domain

A practical abstraction:

```text
Collection# = {
    kind,
    length: Interval#,
    element: Value#,
    mutability/escape: auxiliary facts,
}
```

For list literals:

```text
[1, "x"]
=> length = [2,2]
   element = Int ⊔ String
```

Tuple literals should not be collapsed to homogeneous element joins if positional lanes matter:

```text
Tuple#([A, B, C])
```

Records preserve labels:

```text
Record#({name: String, age: Int})
```

The tuple/record distinction is semantically relevant even if both are product-like mathematical structures.

## 12. Map domain

A two-tier map abstraction can retain exact small keys plus summary values:

```text
Map# {
    known: Map<KeyConst, Value#>,
    other_key: Key#,
    other_value: Value#,
    size: Interval#,
}
```

For a dynamic key write:

```text
m[k] = v
```

if `k` may alias several known keys, weak-update those entries and summary. A naive strong update to one guessed key is unsound.

## 13. Tuple and record product domains

Products compose lane domains:

```text
Tuple#(A1 × A2 × ... × An)
Record#({l1:A1, ..., ln:An})
```

Join position-wise/label-wise only when structures are compatible under domain policy. Otherwise widen to a coarser product/shape alternative.

For records, canonicalize label order so semantic equality does not depend on source insertion order unless Phalcom's record semantics make order observable.

## 14. Effect domain

A may-effect product:

```text
Effects# =
    Throws# × Reads# × Writes# × IO# × Yield# × Reflection# × Native#
```

Join is component-wise union/OR. This domain should live separately from value/type facts so a precise return value can coexist with broad effects.

## 15. Escape domain

Simple flags:

```text
Escape# = Local | EscapesCaller | EscapesFiber | EscapesGlobal | Unknown
```

If scopes are not totally ordered, use a powerset of escape destinations instead of forcing a false hierarchy.

Escape transfer is monotone: once an object may escape to a boundary, later analysis cannot remove that possibility unless a more context-specific analysis reruns from scratch.

## 16. Provenance/trust is metadata, not necessarily lattice payload

It is tempting to define:

```text
ValueFact = Shape × Confidence × Provenance
```

but ask whether confidence/provenance participates in semantic ordering or only explanation/equality/invalidation. The current `InferredValue` carries shape, known boolean, confidence, and bounded provenance. That is practical for advisory tooling.

A future correctness domain may separate:

```text
semantic abstract value
trust/source evidence
precision-loss reason
```

so provenance changes do not accidentally perturb fixed-point semantics.

## 17. Product domains

Compose independent facts:

```text
Value# = Shape# × Constant# × Presence# × Interval#
```

Join component-wise:

```text
(a1,a2,a3) ⊔ (b1,b2,b3)
    = (a1⊔b1, a2⊔b2, a3⊔b3)
```

This avoids a giant enum such as:

```text
IntConstantPositiveSomeExactString...  # bad design
```

But unconstrained products may contain impossible combinations. A reduced product can normalize them.

## 18. Reduced products

Suppose:

```text
Shape#    = Int
Constant# = Const(0)
Sign#     = Positive
```

This combination is inconsistent because zero is not positive. Reduction can produce `⊥` or fix the sign depending on trusted relation.

Reduction function:

```text
ρ : A × B -> A × B
```

should be sound and ideally idempotent:

```text
ρ(ρ(x)) = ρ(x)
```

Do not add cross-domain reductions casually; they can create solver cycles and hidden cost.

## 19. Sum/variant/tag domain

For future algebraic or pattern-like variants:

```text
Variant# = Finite(Set<Tag × Payload#>) | ⊤
```

Branch on tag filters alternatives. This is useful for `Option`/`Result`-like values, but language type representation and runtime representation may differ. Keep the bridge explicit.

## 20. Taint domain

Security analyses often use:

```text
Taint# = P(TaintKind)
```

with sources, propagation, sanitizers, sinks. Sanitization is semantic: do not remove taint because a function name looks like `sanitize`.

For high-stakes security checks, heuristic silence is not evidence of safety.

## 21. Domain selection checklist

For every proposed domain, write:

```text
Concrete question:
Abstract elements:
Meaning γ(a):
Order ⊑:
Bottom:
Top:
Join/meet:
Transfer functions:
Termination/widening:
Canonical equality:
Precision-loss reasons:
Consumer/trust contract:
Incremental dependencies:
Tests:
```

If these are missing, the “domain” is still an intuition.

## 22. Review exercises

1. Why is `Unknown` not bottom in the current shape-style may analysis?
2. When does one-element points-to set still require weak update?
3. Why can `Const(1) + Const(2) = Const(3)` be unsound in a message-dispatch language?
4. What makes an Option presence domain different from the formal `Option<T>` type?
5. When should selector alternatives widen to a family rather than generic dynamic?
6. Which product-domain inconsistencies need a reduction?
7. What consumer justifies adding a relational numeric domain?
8. How does the domain terminate under loops/recursion?
9. Is confidence part of semantic ordering or explanatory metadata?
10. What property test checks the join algebra?
