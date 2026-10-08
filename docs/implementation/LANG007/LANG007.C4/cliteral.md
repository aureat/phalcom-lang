# Telos: Unified Algebraic Units, Complex Numbers, and Quaternions
## Repository-consolidated language and standard-library design

**Design status:** Architectural proposal

**Repository:** `aureat/phalcom-lang`

**Inspected branch:** `main`

**Revision:** `61c8734f752c8ffc252b6d4d4ba378c2bfe2b2b1`

**Scope:** Numeric ontology, algebraic units, contextual inference, compiler semantics, standard-library architecture, and implementation requirements.

## 1. Executive architectural decision

I recommend that Telos adopt **contextually realizable algebraic constants** as a narrowly defined language capability, with `i`, `j`, and `k` as its first standard-library applications.

The fundamental design is:

> `i`, `j`, and `k` are three canonical, immutable mathematical basis values. Their identities are independent of `Complex<T>` and `Quaternion<T>`. Context determines their numeric realization, but does not change their identity or redefine their mathematical meaning.

This separates three things that we have previously been treating as one:

| Concept | Semantic responsibility | Example |
|---|---|---|
| Algebraic identity | Identifies a mathematical basis element | `i`, `j`, `k` |
| Contextual realization | Determines which numeric representation can contain it | `Complex<Float>`, `Quaternion<Int>` |
| Numeric representation | Determines the concrete value, operations, and storage | `Complex<T>`, `Quaternion<T>` |

The resulting developer experience should be:

```python
from math import Complex, Quaternion, i, j, k

// Contextual realization
let z: Complex<Float> = i
let q: Quaternion<Float> = i

// Arithmetic inference
let a = 3 + 4*i
// Complex<Int>

let b = 1 + 2*i + 3*j + 4*k
// Quaternion<Int>

// Native numeric notation
let c = 3 + 4i
// Complex<Int>

let d = 1 + 2i + 3j + 4k
// Quaternion<Int>

// Mathematical laws
let e = i*j
// Quaternion<Int>(0, 0, 0, 1)

let f = j*i
// Quaternion<Int>(0, 0, 0, -1)
```

These examples describe **the proposed final semantics**, not functionality already verified in Phalcom.

### The decisions I would establish

| Question | Decision |
|---|---|
| Preserve separate `Complex<T>` and `Quaternion<T>`? | Yes |
| Share physical representation? | No |
| Define a unified `math.i`? | Yes |
| Give `i` a fixed `Complex<Int>` type? | No |
| Make `i` a reserved compiler keyword? | No |
| Make `i`, `j`, `k` ordinary imported names? | Yes |
| Allow contextual realization of their values? | Yes, under restricted rules |
| Allow user-defined algebraic basis systems eventually? | Yes, without promising arbitrary conversion |
| Introduce automatic numeric promotion? | No |
| Add symbolic algebra evaluation to the compiler? | No |
| Delegate multiplication laws to standard-library implementations? | Yes |
| Preserve Telos's nominal type identity and explicit capability evidence? | Yes |

The significant new language capability is contextual realization of a **resolved binding**. Native literal suffixes are a related but distinct syntax feature.

These should use compatible inference mechanisms without being incorrectly collapsed into the same lexical or semantic category.

---

## 2. Repository constraints and architectural authority

The existing Phalcom repository already establishes several design principles that this proposal must respect.

The repository's numeric architecture records identify the following characteristics:

- `Int` and `Float` are distinct numeric types with specific existing arithmetic and conversion semantics.
- Generic immutable `data` declarations are part of the language's established model.
- Traits and explicit conformance evidence provide the preferred mechanism for generic numerical behavior.
- Runtime types do not determine selector identity; arithmetic cannot simply introduce conventional runtime-type-based overloading.
- `const` values have established static-evaluation and binding semantics.
- The `math` package uses exported method-family captures and implementation modules rather than a universal mathematical object.
- The existing numerical design explicitly discourages unrestricted implicit numeric promotion.

These observations are documented in [the repository's numerical evidence ledger](https://github.com/aureat/phalcom-lang/blob/main/docs/math-proposals/02-repository-findings.md).

Two existing design documents are particularly relevant.

**[Complex-number architecture](https://github.com/aureat/phalcom-lang/blob/main/docs/math-proposals/06-complex-numbers.md)** recommends keeping `Complex<T>` as a distinct nominal type and imposing operation-specific scalar requirements. It also identifies complex division, branch cuts, IEEE behavior, and exact arithmetic as separate contracts.

**[Numerical package architecture](https://github.com/aureat/phalcom-lang/blob/main/docs/math-proposals/08-package-architecture.md)** recommends keeping numerical representations and capabilities separate from higher-level mathematical algorithms, preserving the existing `universe.math` facade.

I agree with both directions.

The design below extends them by introducing quaternions and contextual algebraic constants. It does not replace the existing numerical foundation or propose a competing numeric type system.

One important constraint follows immediately:

**A conventional `const i = ...` declaration cannot acquire multiple unrelated concrete types merely because its use occurs in different contexts.**

Supporting that behavior requires a precisely specified extension to the semantics of the exported value and its use sites.

The implementation must not quietly weaken `const` typing or introduce arbitrary implicit conversions to achieve the desired notation.

---

# Part I — Mathematical and Type-System Design

## 3. The numeric ontology

Telos should distinguish the mathematical relationship between numeric algebras from the type relationships between their concrete representations.

Mathematically:

\[
\mathbb Z \hookrightarrow \mathbb C \hookrightarrow \mathbb H
\]

The first embedding sends an integer to a complex number with zero imaginary component. The second sends a complex number into the selected \(1,i\) plane of the quaternions.

This does not establish nominal subtyping.

```text
                    Mathematical structures

                Integer arithmetic
                        │
                        │ canonical embedding
                        ▼
                Complex algebra
                        │
                        │ canonical i-plane embedding
                        ▼
                Quaternion algebra


                       Telos types

             Int          Complex<T>
                              │
                              │ explicit embedding
                              ▼
                         Quaternion<T>
```

The second diagram does not imply a type hierarchy. Its arrow denotes an operation that constructs a value of another nominal type.

In particular:

```python
Complex<Int> <: Quaternion<Int>   // False
```

The classifier identities remain different.

Both structures may participate in generic algorithms through shared capabilities, but their semantics must reflect their algebraic differences.

### 3.1 `Complex<T>`

The fundamental representation remains:

```python
data Complex<T>(
  _ real: T,
  _ imag: T
)
```

The mathematical interpretation is:

\[
(a,b)\mapsto a+bi
\]

For core arithmetic:

\[
(a+bi)(c+di)=(ac-bd)+(ad+bc)i
\]

The initial useful component domains are `Int` and `Float`, but construction should not intrinsically require every arithmetic capability.

For example, constructing `Complex<T>` and accessing its components requires fewer capabilities than multiplication or division.

### 3.2 `Quaternion<T>`

The representation remains:

```python
data Quaternion<T>(
  _ real: T,
  _ i: T,
  _ j: T,
  _ k: T
)
```

Its mathematical interpretation is:

\[
(a,b,c,d)\mapsto a+bi+cj+dk
\]

The multiplication rules are:

\[
i^2=j^2=k^2=-1
\]

\[
ij=k,\quad jk=i,\quad ki=j
\]

\[
ji=-k,\quad kj=-i,\quad ik=-j
\]

Crucially, quaternion multiplication is associative but not commutative.

These are ordinary Telos data values. Neither needs to inherit from the existing runtime `Number` class merely to qualify as a mathematical number.

That point matters because [the current `Number` surface](https://github.com/aureat/phalcom-lang/blob/main/phalcom-core/core/universe/src/scalar/number.ph) includes ordering operations and a particular division contract. Those assumptions do not uniformly describe complex numbers and quaternions.

**Recommendation:** Do not force either data family into the existing `Number` inheritance hierarchy. Define their mathematical capabilities explicitly.

### 3.3 Preserve efficient storage

The physical representation should be free to specialize independently.

For binary64 coefficients:

```text
Complex<Float>
    real: Float
    imag: Float

    Two scalar components


Quaternion<Float>
    real: Float
    i: Float
    j: Float
    k: Float

    Four scalar components
```

A compact implementation could eventually use unboxed scalar aggregates, but the current data specification does not guarantee such storage.

The repository's [data-object specification](https://github.com/aureat/phalcom-lang/blob/main/docs/specs/objects/data-objects.md) explicitly separates the semantic components of a data object from its physical representation.

Therefore, I would establish a **semantic storage objective**, not a new ABI guarantee:

- A complex number has two observable scalar components.
- A quaternion has four observable scalar components.
- Numeric operations should not require intermediate heap objects where the compiler can safely eliminate them.
- Any future unboxed representation must preserve nominal identity, reflection, data semantics, and observable numeric behavior.

Actual representation performance must be benchmarked against the current VM. Four declared data components do not, by themselves, establish a 32-byte runtime object.

---

## 4. A first-class algebraic unit

This is the central addition.

I would slightly revise our previous `@contextual const` proposal.

**Do not initially introduce an `@contextual` annotation.** It would create a new source-level declaration category before we have established whether Telos needs one.

Instead, represent the units as ordinary immutable data values and make contextual realization an explicit semantic capability.

### 4.1 The canonical unit representation

Conceptual standard-library declarations:

```python
// universe.math.algebra

/// An immutable algebraic basis element identified by its axis.
data Basis<Axis>()
```

Define the three axes:

```python
// universe.math.quaternion

data I()
data J()
data K()
```

Then define the canonical bindings:

```python
// universe.math

const i = Basis<I>()
const j = Basis<J>()
const k = Basis<K>()
```

The declaration uses familiar Telos machinery:

- A nominal generic data family.
- Three zero-component nominal axis types.
- Three statically constructed immutable values.
- Ordinary exported bindings.

The `Basis<Axis>` declaration is a proposed standard-library addition, not an existing repository abstraction.

It is important that the axis itself is part of the type.

Thus:

```python
Basis<I>
Basis<J>
Basis<K>
```

are distinct applied types.

There is no requirement for a runtime `axis` field.

### 4.2 Why a type parameter rather than an integer axis?

We could write:

```python
data Basis(_ axis: Int)
```

But that would make the distinguishing information primarily a runtime value.

Instead, with:

```python
data Basis<Axis>()
```

the difference is represented by nominal type identity.

This allows:

```python
Basis<I>
Basis<J>
Basis<K>
```

to remain distinct even before runtime evaluation.

It also avoids introducing a special enumeration into the compiler's numeric value representation.

This design aligns with Telos's existing emphasis on nominal identity and exact applied types.

### 4.3 What is the type of bare `i`?

This is an important refinement of our previous discussion.

I recommend:

```python
let unit = i
// Basis<I>
```

Not:

```python
let unit = i
// Complex<Int>
```

Why?

Because immediately converting an unconstrained `i` into `Complex<Int>` defeats part of the purpose of defining an independent algebraic unit.

Consider:

```python
from math import i

let imaginary = i

let z: Complex<Float> = imaginary
let q: Quaternion<Float> = imaginary
```

Both assignments should be legal.

The intermediate binding remains:

```python
imaginary: Basis<I>
```

It has not committed to either numeric representation.

By contrast:

```python
let z = 2*i
// Complex<Int>
```

is a completed arithmetic expression with an inferred concrete numeric result.

This distinction gives us a particularly coherent rule:

> **An unconstrained algebraic unit retains its own type. A completed numeric operation realizes the unit into an algebra selected by its context and applicable arithmetic contract.**

Consequently:

| Expression | Inferred type |
|---|---|
| `i` | `Basis<I>` |
| `j` | `Basis<J>` |
| `k` | `Basis<K>` |
| `let x = i` | `Basis<I>` |
| `i + i` | `Complex<Int>` |
| `i * j` | `Quaternion<Int>` |
| `2 * i` | `Complex<Int>` |
| `2 * j` | `Quaternion<Int>` |

This is a deliberate improvement over defaulting a bare `i` to `Complex<Int>`.

It preserves more information without adding runtime complexity to realized complex or quaternion values.

---

## 5. Contextual realization as a typed relation

We now need to define the operation that connects an algebraic unit to a numeric representation.

Conceptually, define:

\[
\operatorname{Realize}(B,A)
\]

where:

- \(B\) is a particular basis value or basis type.
- \(A\) is the intended numeric representation.
- A valid realization produces a value of \(A\).
- The relationship must be explicitly established.

The permitted canonical realizations are:

\[
\operatorname{Realize}(i,\mathbb C_T)=(0_T,1_T)
\]

\[
\operatorname{Realize}(i,\mathbb H_T)=(0_T,1_T,0_T,0_T)
\]

\[
\operatorname{Realize}(j,\mathbb H_T)=(0_T,0_T,1_T,0_T)
\]

\[
\operatorname{Realize}(k,\mathbb H_T)=(0_T,0_T,0_T,1_T)
\]

There is deliberately no realization:

\[
\operatorname{Realize}(j,\mathbb C_T).
\]

### 5.1 A standard capability contract

I would introduce one capability whose conceptual role is:

```python
trait BasisRealizable<B> {
  @class realize(_ basis: B) -> Self
}
```

The intended instances are:

```text
Complex<T>:
    BasisRealizable<Basis<I>>

Quaternion<T>:
    BasisRealizable<Basis<I>>
    BasisRealizable<Basis<J>>
    BasisRealizable<Basis<K>>
```

This is the abstraction that tells the type checker which target types can represent particular basis values.

The operations remain owned by the standard library.

A concrete implementation is equivalent to:

```python
// Illustrative implementation.
// Exact generic constraint syntax and identity providers
// must be established against the active capability package.

impl<T> BasisRealizable<Basis<I>> for Complex<T>
  where T |= AdditiveIdentity, T |= MultiplicativeIdentity
{
  @class
  realize(_ basis: Basis<I>) -> Complex<T> {
    Complex(T.zero, T.one)
  }
}
```

And for quaternions:

```python
impl<T> BasisRealizable<Basis<J>> for Quaternion<T>
  where T |= AdditiveIdentity, T |= MultiplicativeIdentity
{
  @class
  realize(_ basis: Basis<J>) -> Quaternion<T> {
    Quaternion(T.zero, T.zero, T.one, T.zero)
  }
}
```

These declarations are **design-level Telos sketches**. In particular, the numeric identity traits and this realization capability are proposed APIs; their implementation is not verified by the current compiler.

### 5.2 Why target-side realization?

The alternative would be to place knowledge of complex and quaternion numbers inside `Basis`.

That would be architecturally backward.

A basis element should not need to know every algebra capable of containing it.

Instead:

```text
Basis<I>
    │
    ├── Complex<Float> declares realization
    ├── Complex<Int> declares realization
    ├── Quaternion<Float> declares realization
    └── Quaternion<Int> declares realization
```

Future user-defined numeric types can implement the capability for their own targets without modifying the mathematical unit.

This also respects Telos's conformance ownership model: a package that owns a numeric target can provide the corresponding realization behavior, subject to ordinary coherence rules.

### 5.3 Realization is not subtyping

Formally:

\[
\operatorname{Realize}(B,A)
\not\Rightarrow B<:A
\]

A valid realization does not establish a subtype relation.

For example:

```python
let unit = i
// Basis<I>

let z: Complex<Float> = unit
// Contextual realization

let q: Quaternion<Float> = unit
// Different contextual realization
```

The type of `unit` is unchanged.

Only the value required at the particular use site is realized.

This is distinct from ordinary subtype assignment, and the semantic checker must record the distinction.

### 5.4 The essential safety boundary

I would restrict this behavior to **explicitly recognized basis-value realizations**, initially those associated with the canonical `Basis<Axis>` abstraction.

I would not implement a general mechanism resembling:

```text
AnyType → AnyOtherType
    whenever some conversion method exists
```

That would quickly become an implicit conversion system, something the existing mathematical design has explicitly avoided.

Instead:

```text
Basis<Axis> → Target
```

is permitted only when the target provides the correct realization evidence.

Ordinary concrete values remain ordinary concrete values.

For example:

```python
let z = Complex(1, 2)

// Not an algebraic basis value.
let q: Quaternion<Int> = z
```

This should still require the declared complex-to-quaternion embedding.

The basis realization mechanism is not involved.

---

## 6. The algebra selection rules

Contextual realization requires a precise, predictable selection procedure.

I recommend a deliberately bounded system.

### Rule 1 — Explicit expected types have priority

```python
let z: Complex<Float> = i
let q: Quaternion<Float> = i
```

The expected types determine the realization target.

The compiler verifies the applicable capability evidence and materializes the result.

For:

```python
let z: Complex<Float> = j
```

the operation is rejected because `Complex<Float>` does not realize `Basis<J>`.

### Rule 2 — Existing concrete operands provide type evidence

Consider:

```python
let z = Complex(2.0, 3.0)

let result = z + i
```

The existing `Complex<Float>` operand establishes a concrete arithmetic context.

The right-hand `i` is realized as `Complex<Float>`.

Similarly:

```python
let q = Quaternion(1.0, 2.0, 3.0, 4.0)

let result = q * j
```

Here `j` is realized as `Quaternion<Float>`.

The scalar component type is determined by the existing value, not guessed from the basis.

### Rule 3 — Unconstrained arithmetic selects a canonical algebra

For a numeric expression containing basis elements but no concrete enclosing algebra, Telos needs deterministic defaults.

I recommend:

\[
\{i\}\rightarrow\mathbb C
\]

\[
\{j\}\rightarrow\mathbb H
\]

\[
\{k\}\rightarrow\mathbb H
\]

If an arithmetic expression contains `j` or `k`, the smallest canonical supported algebra is quaternionic.

Otherwise, a numeric expression involving `i` defaults to the complex algebra.

Examples:

```python
let a = i + i
// Complex<Int>

let b = i * i
// Complex<Int>

let c = i + j
// Quaternion<Int>

let d = j * k
// Quaternion<Int>
```

The scalar default is `Int` for exact integral expressions.

These defaults apply to the **standard canonical algebra family**. They must not arise from searching all imported third-party algebra types and choosing an arbitrary candidate.

For an external algebra, explicit expected type evidence or an already-typed operand should establish the target.

### Rule 4 — Generic contexts do not guess concrete algebras

Consider a generic identity operation:

```python
class Identity {
  @class
  of<T>(_ value: T) -> T {
    value
  }
}
```

Then:

```python
let x = Identity.of(i)
```

The natural result is:

```python
Basis<I>
```

The compiler should not decide that the generic type parameter must be `Complex<Int>`.

There is already valid, value-producing evidence for `T = Basis<I>`.

For generic algorithms that actually require a numeric interpretation, the appropriate realization capability must be expressed in the contract or supplied by a concrete expected type.

**Generic inference should not fabricate a type argument merely because a basis value could be realized into many types.**

### Rule 5 — Ambiguity remains observable

Suppose contextual constraints admit two unrelated targets.

Telos must not select one through declaration order, import order, or hash-map iteration order.

It should report an ambiguity.

A canonical default may be applied only in the explicitly specified defaulting situation.

### Rule 6 — Ordinary aliases preserve the basis type

```python
from math import i as imaginary

let unit = imaginary
// Basis<I>

let z: Complex<Float> = unit
let q: Quaternion<Float> = unit
```

The imported name and local alias preserve the same value semantics.

This includes qualified access:

```python
import math

let z: Complex<Float> = math.i
```

The compiler resolves the actual exported value and its type.

It must never recognize contextual basis values merely by matching their source identifier text.

---

# Part II — Resolving the Existing Compiler Architecture

## 7. The most important repository finding: binary arithmetic is currently synthesized too early

The inspected implementation provides a very specific architectural constraint.

In:

[`phalcom-semantic/src/checker/expression.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-semantic/src/checker/expression.rs)

the current `synthesize_binary_expr` follows this general sequence:

```rust
let left_typed =
    analyze_expression(ctx, &binary.left, &ExpectedType::None);

let right_typed =
    analyze_expression(ctx, &binary.right, &ExpectedType::None);

apply_binary_operation_from_typed(
    ctx,
    &binary.left,
    &left_typed,
    binary.op.clone(),
    &binary.right,
    &right_typed,
    expected,
    binary.range,
)
```

The important behavior is that **both operands are analyzed without an expected type before the operation is resolved**.

That is reasonable for many existing operations.

It is insufficient for the contextual algebraic inference we want.

### 7.1 The failing conceptual scenario

Consider:

```python
let q: Quaternion<Float> = i + j
```

The current synthesis strategy would first analyze:

```text
i → Basis<I>
j → Basis<J>
```

It would then attempt ordinary operator dispatch on the left operand.

But `Basis<I>` does not have to own quaternion multiplication or addition methods.

Even more importantly:

```python
let q = 1 + 2*i + 3*j + 4*k
```

is a larger, left-associated arithmetic expression.

If the first operation commits to a concrete complex result, then the subsequent quaternion context must either support mixed arithmetic or trigger a new elaboration decision.

This is not solved by changing the lexer.

Nor is it solved by defining:

```python
const i = Basis<I>()
```

The compiler needs a mechanism for gathering relevant contextual constraints before committing to a numeric representation.

### 7.2 Do not change Telos's selector model

The repository's [selector specification](https://github.com/aureat/phalcom-lang/blob/main/docs/specs/callables/selectors.md) defines selector identity structurally, independently of argument runtime types.

The current semantic analyzer also implements bilateral reflected arithmetic in:

```text
phalcom-semantic/src/checker/expression.rs

apply_binary_operation_from_typed
reflected_binary_selector
should_use_reflected_binary_target
```

This is useful existing machinery.

It is not, however, equivalent to unrestricted type-based operator overloading.

For example, I would **not** implement the basis feature by adding a second `*(_)` operation on `Int`, distinguished only by accepting `Basis<I>` rather than `Number`.

That conflicts with the existing selector philosophy.

Similarly, introducing a special dispatch table that says:

```text
Int * Basis<I> → Complex<Int>
Int * Basis<J> → Quaternion<Int>
```

would solve only the immediate examples, while establishing a parallel numeric operator system.

I recommend neither solution.

### 7.3 Proposed solution: contextual operand elaboration

Introduce a semantic stage that can resolve eligible basis-bearing arithmetic expressions before committing their constituent operations to concrete targets.

The stage should operate on normal AST expressions and produce canonical typed elaboration facts.

Conceptually:

```text
Parsed ordinary expression
          │
          ▼
Resolve identifiers and ordinary operand evidence
          │
          ▼
Discover contextual basis requirements
          │
          ▼
Determine admissible algebra and scalar type
          │
          ▼
Prove realization and arithmetic capabilities
          │
          ▼
Record concrete operand realizations and operator targets
          │
          ▼
Ordinary compiler lowering
```

The central distinction is that **the semantic analyzer owns the interpretation; the compiler owns its execution**.

The bytecode compiler should consume the final checked result.

It should not independently inspect `i`, infer quaternions, or rebuild trait evidence.

This is consistent with the repository's `AGENTS.md` guidance requiring formal semantic authority to remain in the semantic layer.

---

## 8. A bounded contextual arithmetic solver

I would not introduce a general expression-wide symbolic algebra engine.

Instead, introduce a bounded **algebraic inference region**.

A region is a connected part of an ordinary arithmetic expression whose evaluation requires unresolved algebraic basis realizations.

For example:

```python
1 + 2*i + 3*j + 4*k
```

can initially be analyzed as one relevant region.

The parser continues to produce ordinary binary expressions.

The semantic analyzer gathers evidence for the region.

### 8.1 Constraints

For each basis occurrence:

\[
\operatorname{Realizable}(B_i,A)
\]

For each scalar coefficient:

\[
\operatorname{ScalarAdmissible}(S,A)
\]

For each operator:

\[
\operatorname{OperatorApplicable}(A,\circ)
\]

And, where a result type is specified:

\[
\operatorname{Result}(E)=A_{\mathrm{expected}}
\]

The solver chooses a concrete algebra only when the constraints permit it.

For the initial canonical built-ins:

```text
Basis<I>:
    Complex<T>
    Quaternion<T>

Basis<J>:
    Quaternion<T>

Basis<K>:
    Quaternion<T>
```

This candidate set is intentionally small.

The solver is not permitted to search arbitrary runtime classes for methods that might happen to accept the operands.

### 8.2 A concrete example

Input:

```python
let q = 1 + 2*i + 3*j + 4*k
```

Step one: resolve source identities.

```text
i → math.i → Basis<I>
j → math.j → Basis<J>
k → math.k → Basis<K>
```

Step two: determine the scalar type.

```text
1 → Int
2 → Int
3 → Int
4 → Int
```

Step three: collect algebra requirements.

```text
I requires Complex or Quaternion
J requires Quaternion
K requires Quaternion
```

Step four: intersect with applicable arithmetic contracts.

```text
Required algebra: Quaternion<Int>
```

Step five: produce a checked elaboration equivalent to:

```python
Quaternion(1, 0, 0, 0)
  + Quaternion(0, 2, 0, 0)
  + Quaternion(0, 0, 3, 0)
  + Quaternion(0, 0, 0, 4)
```

The optimizer may subsequently reduce this to:

```python
Quaternion(1, 2, 3, 4)
```

The first construction is a semantic explanation. It need not imply four runtime allocations.

### 8.3 Preserve explicit expression boundaries

Not every expression should be recursively absorbed into an algebraic region.

For example:

```python
let z = 2*i
let q = z + 3*j
```

The first statement produces a concrete `Complex<Int>`.

The second statement must be resolved using the declared mixed complex/quaternion arithmetic semantics.

It must not retroactively change the type of `z`.

Likewise, method calls, stored values, function results, mutable reads, and effectful subexpressions should retain their declared semantics.

Parentheses alone should not freeze inference:

```python
let q = (2*i) + 3*j
```

But a completed binding does.

This is an important invariant for predictable typing.

### 8.4 Preserve evaluation order

The solver may select an algebra before the operands execute.

It may not reorder source operations.

For example:

```python
computeA() * i + computeB() * j
```

must still evaluate `computeA()` before `computeB()`.

Each function must execute exactly once.

If the elaborated operations involve potentially observable floating-point arithmetic, transformations must also respect the language's numeric evaluation contract.

This means literal specialization and general arithmetic reassociation are separate optimization questions.

### 8.5 No universal promotion lattice

The compiler should not infer:

```text
Int < Complex<Int> < Quaternion<Int>
```

as a universal subtyping or conversion chain.

Instead, the required scalar or complex embedding is an explicitly declared part of the applicable arithmetic operation.

This keeps contextual basis realization separate from general numeric promotion.

The distinction should be visible in the semantic evidence.

---

## 9. Two realization operations are necessary

There is a subtle mathematical and numerical issue with treating every scalar-basis expression as ordinary multiplication.

Consider:

```python
let x = 2.0*i
```

It is tempting to elaborate this to:

```python
Complex(2.0, 0.0) * Complex(0.0, 1.0)
```

For finite values, this produces the expected result.

But for infinity, NaN, and signed zero, ordinary floating-point multiplication can produce observable artifacts.

For example, multiplying a zero component by infinity produces NaN under IEEE arithmetic.

A basis coefficient should instead be placed directly into the appropriate component.

Therefore, I recommend distinguishing **real embedding** from **basis-component realization**.

### 9.1 Real embedding

A real scalar \(x\) embeds as:

\[
\operatorname{Real}_{\mathbb C}(x)=(x,0)
\]

\[
\operatorname{Real}_{\mathbb H}(x)=(x,0,0,0)
\]

A possible standard capability:

```python
// Proposed interface

trait RealEmbeddable {
  type Scalar

  @class fromReal(_ value: Self::Scalar) -> Self
}
```

This should be implemented by `Complex<T>` and `Quaternion<T>` for their scalar type.

### 9.2 Basis-component realization

A coefficient \(x\) on an imaginary basis becomes:

\[
\operatorname{Component}_{\mathbb C}(x,i)=(0,x)
\]

For quaternions:

\[
\operatorname{Component}_{\mathbb H}(x,i)=(0,x,0,0)
\]

\[
\operatorname{Component}_{\mathbb H}(x,j)=(0,0,x,0)
\]

\[
\operatorname{Component}_{\mathbb H}(x,k)=(0,0,0,x)
\]

I would refine the proposed capability accordingly:

```python
// Proposed interface, not currently implemented.

trait BasisRealizable<B> {
  type Scalar

  @class along(
    _ coefficient: Self::Scalar,
    _ basis: B
  ) -> Self
}
```

This captures the actual semantic operation required for both bare units and coefficients.

Bare `i` is equivalent to realizing a coefficient of one:

```python
let z: Complex<Float> = i

// Conceptually:
Complex.along(1.0, i)
```

The expression:

```python
let z = 2.0*i
```

uses the coefficient directly:

```python
Complex.along(2.0, i)
```

The latter is **not defined as multiplication by a quaternion or complex value containing an implicit one**.

Instead, basis realization places the coefficient in the designated component.

For an explicitly constructed `Complex` multiplication, ordinary complex arithmetic still applies.

This distinction makes the semantics cleaner and avoids unnecessary exceptional-value propagation.

### 9.3 Scalar-left and scalar-right notation

Both expressions should be supported:

```python
2*i
i*2
```

They should realize the same basis component for the canonical scalar domains.

But this symmetry is specific to scalar action on basis elements.

It does not imply commutative quaternion multiplication.

For example:

```python
i*j
j*i
```

must use the Hamilton product and produce different results.

The semantic analyzer should distinguish:

| Operation | Interpretation |
|---|---|
| `2*i` | Coefficient realization |
| `i*2` | Coefficient realization |
| `2*j` | Coefficient realization |
| `i*j` | Quaternion multiplication |
| `j*i` | Quaternion multiplication |
| `q*i` | Ordinary quaternion multiplication |
| `i*q` | Ordinary quaternion multiplication |

The last two cannot be treated as equivalent.

This distinction can be implemented through a typed elaboration result without changing ordinary operator syntax.

---

## 10. Native imaginary literals

The same mathematical realization contract should serve native literals.

The lexer should recognize:

```python
2i
3j
4k
```

as numeric literal forms with attached basis information.

It should not rewrite them into ordinary identifier references.

### 10.1 Recommended AST

A suitable conceptual representation:

```rust
enum ImaginaryAxis {
    I,
    J,
    K,
}

struct ImaginaryLiteral {
    coefficient: NumericLiteral,
    axis: ImaginaryAxis,
    range: SourceRange,
}
```

The actual Rust representation should be selected according to existing AST conventions.

The relevant implementation owners are:

- [`phalcom-ast/src/token.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-ast/src/token.rs)
- [`phalcom-ast/src/lexer.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-ast/src/lexer.rs)
- [`phalcom-ast/src/parser.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-ast/src/parser.rs)

The lexer currently distinguishes `Token::Int` from `Token::Float`, while the parser and semantic analyzer preserve that distinction.

The accepted [TDR-0070 numeric-literal decision](https://github.com/aureat/phalcom-lang/blob/main/docs/decisions/accepted/0070-numeric-literals.md) explicitly defers complex and imaginary suffixes.

Therefore, adding them is a new language decision rather than an interpretation of already-approved syntax.

### 10.2 Literal syntax rules

I would initially support the existing accepted integer and floating-point coefficient grammars followed immediately by a lowercase basis suffix.

Examples:

```python
2i
3j
4k

2.5i
3.125j
4.0k

1e3i
2.5e-4j

0xFFi
0b101j
```

The suffix is part of the numeric token.

Whitespace separates expressions:

```python
2i     // Imaginary literal

2*i    // Multiplication using an identifier

2 i    // Not an imaginary literal
```

Bare identifiers are unaffected:

```python
let i = 42

let a = i
// Int

let b = 2i
// Imaginary literal
```

Importing or shadowing `math.i` must have no effect on the lexical interpretation of `2i`.

Invalid contiguous suffixes should be diagnosed atomically, following the existing numeric lexer policy.

For example:

```python
2iq
3jvalue
4k2
```

These should not silently tokenize as a valid imaginary literal followed by unrelated identifier text.

### 10.3 Unary signs and precedence

A negative imaginary literal should follow Telos's ordinary unary-sign rules:

```python
-2i
+3j
-4k
```

The sign is not intrinsically part of the unsigned coefficient token.

Exponentiation should retain its established operator precedence.

For example:

```python
-2i ** 2
```

must follow the language's specified unary/exponentiation precedence; imaginary syntax must not introduce a special exception.

The same applies to subtraction:

```python
1 - 2i
```

This remains an ordinary subtraction expression whose right operand is an imaginary literal.

### 10.4 Literal typing

Native imaginary literals should have a contextual realization set.

| Literal | Default result | Other permitted target |
|---|---|---|
| `2i` | `Complex<Int>` | `Quaternion<Int>` |
| `2.0i` | `Complex<Float>` | `Quaternion<Float>` |
| `3j` | `Quaternion<Int>` | None in the initial standard family |
| `3.0j` | `Quaternion<Float>` | None in the initial standard family |
| `4k` | `Quaternion<Int>` | None in the initial standard family |

The coefficient type is determined by the existing numeric literal grammar.

This raises one important compatibility question:

```python
let z: Complex<Float> = 2i
```

Should `2` be implicitly interpreted as `Float`?

**For the first implementation, I recommend no.**

Current Phalcom assigns an exact `Int` type to integer literals and explicitly avoids general implicit `Int`-to-`Float` conversion.

Therefore:

```python
let z: Complex<Float> = 2.0i
```

is the unambiguous initial spelling.

By contrast:

```python
let z: Complex<Float> = i
```

is valid because realizing a mathematical basis unit into a target algebra is a different operation from converting an existing `Int` value into `Float`.

A future language-wide contextual numeric-literal policy could permit `2i` in a `Complex<Float>` context, but that decision should be made consistently with scalar literals elsewhere.

I would not introduce it as an undocumented exception.

### 10.5 Relationship between `2i` and `2*i`

The two expressions should produce equivalent mathematical results in matching scalar domains.

However, they do not share a lexical identity:

```text
2i
    Numeric literal with basis suffix

2*i
    Binary multiplication expression
    with a resolved basis-valued operand
```

Both eventually use the same basis-component realization contract.

The compiler should not implement two independent mathematical interpretations.

---

# Part III — Standard-Library Architecture

## 11. Recommended package organization

The live package currently resides under:

```text
phalcom-core/core/universe/src/math/
```

Its `package.ph` exports constants and captured mathematical Families, backed by source-owned implementation modules.

The current contents include the Float kernel, integer helpers, reductions, and special functions.

There is no `universe.numbers` directory in the inspected `main` tree, although the existing numerical proposals describe one as a possible future foundation.

For this implementation, I recommend retaining `math` as the public API owner and adding separate source-owned components.

```text
phalcom-core/core/universe/src/math/
│
├── package.ph
│
├── algebra.ph
│   ├── Basis<Axis>
│   ├── scalar identity capabilities
│   ├── RealEmbeddable
│   ├── BasisRealizable
│   └── shared conjugation/norm capabilities
│
├── complex.ph
│   ├── Complex<T>
│   ├── complex constructors
│   ├── arithmetic implementations
│   ├── basis realization
│   └── complex embedding helpers
│
├── quaternion.ph
│   ├── Quaternion<T>
│   ├── quaternion constructors
│   ├── Hamilton multiplication
│   ├── basis realization
│   ├── complex embeddings
│   └── quaternion decomposition
│
├── float_kernel.ph
├── integer.ph
├── reduction.ph
├── numerical_errors.ph
│
└── special/
```

This is a **recommended target layout**, not a claim that these files already exist.

The future architecture may still move foundational numeric declarations into a dedicated `universe.numbers` package before their public identity is stabilized.

What matters is that the semantic owner of `Complex` and `Quaternion` be fixed before public release.

Moving a nominal data declaration to a different module later is not necessarily a harmless path refactor: its canonical declaration identity can change.

Therefore, the implementation should select one durable owner and re-export from the public `math` facade, rather than create duplicate declarations in multiple packages.

### 11.1 Public root exports

The intended root interface is:

```python
from math import (
  Complex,
  Quaternion,
  i, j, k
)
```

The algebraic capabilities should also have a stable import path for third-party packages.

For example:

```python
from math.algebra import (
  Basis,
  BasisRealizable,
  RealEmbeddable,
  Conjugatable,
  NormSquared
)
```

Export policy needs to distinguish the stable public vocabulary from implementation details.

The axis marker types and private arithmetic helpers need not all become prominent root-level API members.

### 11.2 Preserve the existing facade

I would not replace:

```python
let sqrt = &FloatMathKernel.sqrt(_)
```

with an unrelated generic numeric dispatcher as part of this project.

`math.sqrt` currently has a specific Float-oriented contract.

Introducing complex and quaternion numbers should not silently change it.

Future APIs may include:

```python
math.complex.sqrt(z)
math.quaternion.exp(q)
```

But complex branches, quaternion exponential functions, and logarithms require their own mathematical and numerical specifications.

Those are separate from implementing the core structures and unit syntax.

---

## 12. `Complex<T>` API

I recommend a compact initial interface.

```python
// Conceptual public API.

data Complex<T>(
  _ real: T,
  _ imag: T
)
```

Construction:

```python
let a = Complex(3, 4)

let b = 3 + 4i

let c: Complex<Float> = i
```

Component access:

```python
a.real
a.imag
```

Core operations:

```python
a + b
a - b
a * b

a.conjugate
a.normSquared
```

For supported scalar domains, additional operations include:

```python
a / b
a.abs
a.arg
```

These should be conditionally available according to the actual component capabilities.

### 12.1 Capability bounds

I would distinguish the requirements as follows:

| Operation | Required scalar properties |
|---|---|
| Construction | Valid component values |
| Access | None beyond construction |
| Addition | Closed addition |
| Subtraction | Additive inverses |
| Multiplication | Commutative scalar multiplication and distributivity |
| Conjugation | Negation of imaginary component |
| Squared norm | Suitable multiplication and addition |
| Division | An appropriate division-capable scalar domain |
| Magnitude | Nonnegative real norm and square-root capability |
| Transcendentals | Explicit approximate numerical backend |

These are mathematical requirements, not automatically proven properties of a nominal trait implementation.

For example, `Multipliable` alone does not prove distributivity or commutativity.

Telos should be careful not to let weak behavioral traits imply stronger algebraic laws.

### 12.2 Integer complex numbers

```python
let z = 3 + 4i
// Complex<Int>
```

This is a valid exact Gaussian integer value.

Addition and multiplication should remain exact, subject to the existing integer resource policy.

General division should not automatically produce `Complex<Int>`:

\[
\frac{1+i}{2}=\frac12+\frac12i
\]

The result is not a Gaussian integer.

This suggests division being unavailable for the integer specialization unless a separately specified result type is provided.

### 12.3 No ordering

Neither complex nor quaternion values should implement ordinary magnitude-based `<` or `>` comparisons.

For example:

```python
let z = 3 + 4i

z < 5
```

should not acquire an arbitrary ordering rule.

Magnitude comparison is meaningful when explicitly requested, but it is not a total order on the complex algebra.

---

## 13. `Quaternion<T>` API

The proposed core type:

```python
data Quaternion<T>(
  _ real: T,
  _ i: T,
  _ j: T,
  _ k: T
)
```

The initial developer interface:

```python
let q = 1 + 2i + 3j + 4k

q.real
q.i
q.j
q.k

q.conjugate
q.normSquared
```

The quaternion conjugate is:

\[
\overline q=a-bi-cj-dk.
\]

The squared norm is:

\[
N(q)=a^2+b^2+c^2+d^2.
\]

For nonzero quaternions over a suitable real field:

\[
q^{-1}=\frac{\overline q}{N(q)}.
\]

### 13.1 Quaternion multiplication

For:

\[
p=a+bi+cj+dk
\]

\[
q=e+fi+gj+hk
\]

the product is:

\[
\begin{aligned}
pq={}&(ae-bf-cg-dh)\\
&+(af+be+ch-dg)i\\
&+(ag-bh+ce+df)j\\
&+(ah+bg-cf+de)k.
\end{aligned}
\]

This is the required Hamilton product.

The implementation must preserve operand order.

Core verification examples:

```python
let qi: Quaternion<Int> = i
let qj: Quaternion<Int> = j
let qk: Quaternion<Int> = k

Test.expectEqual(qi*qj, qk)
Test.expectEqual(qj*qi, -qk)

Test.expectEqual(qj*qk, qi)
Test.expectEqual(qk*qj, -qi)

Test.expectEqual(qk*qi, qj)
Test.expectEqual(qi*qk, -qj)
```

The use of typed quaternion values here is intentional: it tests the actual arithmetic implementation independently of contextual-basis inference.

### 13.2 Scalar-domain restrictions

`Quaternion<Int>` supports exact addition and multiplication.

General division should not be provided as a closed `Quaternion<Int>` operation.

`Quaternion<Float>` can support division and inversion under a specified floating-point policy.

The implementation should distinguish:

```python
q.normSquared
q.norm
q.inverse
```

These operations do not all require the same scalar capabilities.

Moreover, a floating-point quaternion norm should use a numerically robust scaled algorithm rather than naively computing four squares before taking the square root.

### 13.3 Separate rotation semantics

I would preserve:

```python
Quaternion<Float>
```

as a general algebraic value.

A future:

```python
UnitQuaternion<Float>
```

can represent a validated unit-norm quaternion for applications such as 3D rotations.

The latter has different invariants and should not be introduced as a mere alias.

Quaternion literals should always produce the general quaternion type.

---

## 14. Complex/quaternion interoperability

The canonical embedding is:

\[
\phi(a+bi)=a+bi+0j+0k.
\]

I would define a public constructor:

```python
// Proposed @constructor call overload.

impl<T> Quaternion<T> {
  @constructor
  call(_ value: Complex<T>) -> Quaternion<T> {
    Quaternion(value.real, value.imag, T.zero, T.zero)
  }
}
```

The exact eligibility of a generic data `@constructor` member alongside the primary constructor needs validation against the active constructor specification and tests. The intended constructor semantics are unambiguous.

Usage:

```python
let z = Complex(3, 4)

let q = Quaternion(z)
// Quaternion(3, 4, 0, 0)
```

For the reverse direction, provide two explicitly different operations.

A projection:

```python
let z = q.complexPart
// Complex(q.real, q.i)
```

And a checked reconstruction:

```python
let z = q.asComplex
// Option<Complex<T>>
```

`asComplex` succeeds only when the quaternion has zero `j` and `k` components in the selected scalar domain.

For floating-point coefficients, the behavior with NaNs and signed zeros must be specified, rather than inferred accidentally from an implementation's equality checks.

### 14.1 Mixed arithmetic

I recommend a bounded mixed-operation matrix.

| Left | Right | Result |
|---|---|---|
| `Complex<T>` | `T` | `Complex<T>` |
| `T` | `Complex<T>` | `Complex<T>` |
| `Quaternion<T>` | `T` | `Quaternion<T>` |
| `T` | `Quaternion<T>` | `Quaternion<T>` |
| `Complex<T>` | `Quaternion<T>` | `Quaternion<T>` |
| `Quaternion<T>` | `Complex<T>` | `Quaternion<T>` |

This is an arithmetic operation contract, not a conversion hierarchy.

For example:

```python
let z = Complex(1, 2)
let q = Quaternion(3, 4, 5, 6)

let result = z + q
// Quaternion(4, 6, 5, 6)
```

The standard library can implement this using the appropriate component operations.

For multiplication, the two operand orders must have separate correct implementations.

### 14.2 Use Telos's existing reflected operations

The current checker already recognizes reflected binary selectors.

Therefore, scalar-left operations can potentially be provided through the existing mechanism rather than by adding conflicting methods to `Int` or `Float`.

For example, a complex type may provide a compatible reflected arithmetic method where the current dispatch rules permit it.

This must be validated against the exact current selector signatures and conformance behavior.

**Do not assume that two methods accepting different parameter types can coexist as independent runtime overloads under the same selector.**

That would contradict the current selector identity contract.

For the initial feature, direct basis-oriented contextual elaboration can be implemented first, with ordinary mixed-type arithmetic verified separately.

---

## 15. Generic numerical capabilities

I would introduce only capabilities that have concrete consumers in this implementation.

| Capability | Purpose |
|---|---|
| Additive identity | Construct scalar zero |
| Multiplicative identity | Construct scalar one |
| Conjugation | Produce algebraic conjugate |
| Squared norm | Return the norm squared in an associated scalar domain |
| Real embedding | Construct an algebra value from its real scalar |
| Basis realization | Construct an algebra value from a basis coefficient |
| Existing arithmetic capabilities | Express supported addition, subtraction, and multiplication |

For example:

```python
// Proposed signatures.

trait Conjugatable {
  conjugate -> Self
}

trait NormSquared {
  type Scalar

  normSquared -> Self::Scalar
}
```

The associated result type matters because the squared norm of a quaternion is a scalar, not another quaternion.

For example:

```python
let q = 1 + 2i + 3j + 4k

q.normSquared
// Int: 30
```

The generic result should not be forced into `Quaternion<Int>` merely to satisfy an artificial universal numeric interface.

### 15.1 Capability composition

Telos should preserve its existing independent-conformance design.

For instance:

```python
impl<T> Conjugatable for Complex<T>
  where T |= Negatable
{
  ...
}
```

and:

```python
impl<T> Conjugatable for Quaternion<T>
  where T |= Negatable
{
  ...
}
```

would provide independent conformance evidence.

I would not introduce an inheritance chain such as:

```text
Number
  └── Hypercomplex
       └── Quaternion
```

It would add more conceptual machinery than this problem needs.

### 15.2 Algebraic law documentation

Capabilities should document mathematical expectations such as:

\[
x+0=x
\]

\[
x(y+z)=xy+xz
\]

\[
(xy)z=x(yz)
\]

But a Telos implementation of a trait should not automatically constitute a formal proof that those laws hold.

This matters particularly for floating-point arithmetic, where exact algebraic identities can fail as machine-level equalities because of rounding, signed zero, infinities, and NaNs.

The compiler must not introduce law-based reassociation solely because a type claims an algebraic capability.

---

# Part IV — Compiler Integration and Semantic Correctness

## 16. Contextual realization must be recorded as a semantic fact

The compiler cannot reliably reconstruct contextual realizations from source syntax alone.

This is especially important for qualified and imported names:

```python
from math import i as imaginary

let x = imaginary

let z: Complex<Float> = x
```

The final occurrence of `x` has type `Basis<I>`, but its spelling carries no mathematical significance.

The type checker must recognize the available realization through canonical type and conformance evidence.

### 16.1 Proposed semantic product

I recommend a semantic product conceptually resembling:

```rust
struct ContextualRealization {
    source_expression: ExpressionId,
    source_type: TypeId,
    target_type: TypeId,
    realization: RealizationKind,
    evidence: ResolvedRealizationEvidence,
}
```

Possible realization categories:

```rust
enum RealizationKind {
    BasisUnit,
    BasisCoefficient,
    RealEmbedding,
}
```

These structures are **proposed**, not existing Phalcom declarations.

The important information is:

- Which expression is realized?
- What is its source type?
- What exact target type was established?
- Which declared realization operation applies?
- Which conformance and callable evidence justified it?
- Which source locations should be used in diagnostics?

The solver's temporary inference variables should not appear in published semantic products.

This follows the repository's established separation between query-local inference and durable semantic metadata.

### 16.2 Required implementation ownership

The relevant existing components include:

| Repository component | Proposed responsibility |
|---|---|
| `phalcom-semantic/src/checker/expression.rs` | Discover and type-check contextual algebraic expressions |
| `checker/expected.rs` | Supply expected target types |
| `checker/inference.rs` | Solve relevant generic and associated-type constraints |
| `checker/typed_expr.rs` | Carry local checked realization information |
| `checker/analysis.rs` | Publish durable expression analysis |
| `checker/call.rs` | Apply exact callable and generic realization evidence |
| `types/denotation.rs` | Represent any necessary distinction between ordinary values and contextual interpretations |
| `module_bindings.rs` | Preserve imported binding identity and checked value facts |
| `static_constant.rs` | Preserve canonical constant identity and static materialization |
| `phalcom-core/src/compiler/lib/expr.rs` | Lower validated semantic realizations into ordinary executable operations |

Not every file necessarily needs modification. These are the relevant ownership boundaries to investigate during implementation planning.

**The implementation should extend the canonical semantic analysis pipeline, not introduce an alternative mathematical expression checker.**

### 16.3 Formal typing distinction

Ordinary expression synthesis can be written conceptually as:

\[
\Gamma\vdash e\Rightarrow T
\]

Contextual checking is:

\[
\Gamma\vdash e\Leftarrow T
\]

A contextual realization adds a distinct operation:

\[
\Gamma\vdash e\rightsquigarrow_T e'
\]

where \(e'\) is the checked elaboration of \(e\) into a value of type \(T\).

For example:

\[
\Gamma\vdash i\Rightarrow\operatorname{Basis}\langle I\rangle
\]

but:

\[
\Gamma\vdash i\rightsquigarrow_{\operatorname{Complex}\langle Float\rangle}
\operatorname{Complex}(0.0,1.0)
\]

The second judgment is valid only when the required realization evidence is established.

This prevents a subtle but serious mistake: treating the contextual operation as proof that the source type and destination type are equivalent.

They are not.

### 16.4 Static versus runtime values

The existing static-evaluation specification is explicit: static evaluation is not arbitrary runtime method execution.

Therefore:

```python
const i = Basis<I>()
```

should be treated as an ordinary closed static data construction.

But:

```python
const z: Complex<Float> = i
```

requires a statically eligible realization.

The implementation must establish that eligibility explicitly.

It cannot invoke an arbitrary `BasisRealizable` method during static evaluation simply because that method exists at runtime.

I would establish two phases of support:

**Initial runtime realization**

```python
const i = Basis<I>()

let z: Complex<Float> = i
```

The compiler lowers the runtime realization using established semantic evidence.

**Static realization**

```python
const z: Complex<Float> = i
const q = 1 + 2i + 3j + 4k
```

These become supported when the static evaluator can reduce the required data construction and approved operations according to its normative rules.

The final language feature should support both. However, static compatibility must be proven rather than assumed.

---

## 17. Type inference must remain coherent across generic boundaries

Contextual units create several important generic inference cases.

### 17.1 Concrete expected result

```python
class Example {
  @class
  complexUnit() -> Complex<Float> {
    i
  }
}
```

The result annotation supplies the target.

This is unambiguous.

### 17.2 Generic identity

```python
class Example {
  @class
  identity<T>(_ value: T) -> T {
    value
  }
}
```

Then:

```python
let x = Example.identity(i)
```

should infer:

```python
Basis<I>
```

The compiler already has sufficient evidence.

It should not search for every type into which `i` can be realized.

### 17.3 Generic functions with numeric constraints

Suppose we have a generic algorithm accepting a numeric algebra `A`.

Conceptually:

```python
class AlgebraTools {
  @class
  someOperation<A>(_ value: A) -> A
    where A |= SomeAlgebraicCapability
  {
    ...
  }
}
```

Calling it with a bare basis value must not automatically select an arbitrary `A`.

The compiler needs either:

- Concrete argument evidence.
- An expected result type that determines `A`.
- An explicit specialization.
- Another valid constraint that uniquely determines `A`.

If no unique result exists, the correct diagnostic is an underconstrained or ambiguous inference result.

The basis system should not introduce hidden preference rules into general generic inference.

### 17.4 Preserve symbolic constraints

An unresolved generic target is not necessarily invalid.

For example, a generic method may have a type parameter `A` satisfying the appropriate basis-realization capability.

The compiler should preserve the unresolved realization obligation in its symbolic inference representation until sufficient evidence becomes available.

This is especially relevant because Phalcom already has sophisticated machinery for associated types, conformance evidence, and generic specialization.

The implementation should reuse that machinery instead of inventing an algebra-specific generic solver.

---

## 18. Avoid new runtime machinery

Contextual basis realization should ordinarily disappear during compilation.

For example:

```python
let z: Complex<Float> = i
```

can lower into the ordinary data construction equivalent to:

```python
Complex(0.0, 1.0)
```

The resulting value is a normal `Complex<Float>`.

It should not contain:

```text
A pointer to math.i
A symbolic basis descriptor
An algebra registry handle
A runtime conversion callback
A dynamically selected realization table
```

These are unnecessary for the selected representation.

The source-level `Basis<I>` value remains useful when it is intentionally retained:

```python
let unit = i
```

In that case, `unit` is an actual nominal basis value.

Since `Basis<I>` has no declared runtime components, it should be a strong candidate for a zero-payload representation, subject to the runtime's data-object model.

### 18.1 Optimization boundaries

The optimizer may specialize:

```python
let q = 1 + 2i + 3j + 4k
```

into one quaternion construction.

However, it must first establish semantic equivalence under the applicable scalar and arithmetic operations.

In particular:

- Integer arithmetic must remain exact.
- Floating-point signed-zero behavior must be preserved where observable.
- NaN behavior must follow the numeric contract.
- Effectful operands must be evaluated once and in order.
- Quaternion multiplication must never be reordered.
- Ordinary expression reassociation must not be assumed legal.

I would implement correctness-first lowering before adding literal-fusion optimization.

---

# Part V — Diagnostics, Edge Cases, and Language Boundaries

## 19. Required diagnostic behavior

A new contextual inference mechanism is only useful if failures are understandable.

I recommend introducing specific diagnostic cases rather than mapping every failure to a generic type mismatch.

The following names are illustrative proposed categories.

| Diagnostic | Example | Explanation |
|---|---|---|
| Basis realization unavailable | `let z: Complex<Float> = j` | Target algebra cannot represent the basis |
| Contextual algebra ambiguous | Multiple unrelated realizations satisfy the context | No unique target |
| Scalar representation conflict | Integer coefficient required as Float without permitted conversion | Scalar types are incompatible |
| Algebraic operation unavailable | Division on an exact integer algebra without an applicable operation | Requested operation lacks a valid contract |
| Generic realization underconstrained | Generic result type cannot be determined | Insufficient evidence to select a target |
| Invalid imaginary literal | `2ijk` | Invalid numeric suffix |
| Static realization unavailable | A `const` initializer requires non-static realization | Operation is not eligible for static evaluation |

The diagnostics should identify both the original expression and the contextual requirement.

For example:

```python
let z: Complex<Float> = 3j
```

A useful diagnostic would explain:

```text
Cannot realize imaginary basis j as Complex<Float>.

Basis j requires an algebra supporting the quaternion j-axis.
The expected target Complex<Float> has no applicable realization.
```

This is substantially more useful than reporting that some method named `+` or `*` could not be found.

### 19.1 `Dynamic` must not hide invalid semantics

Suppose an operation passes through `Dynamic`.

The compiler should not treat that as proof of an arbitrary basis-realization capability.

Existing dynamic-boundary rules should continue to apply.

Any dynamic conversion or dispatch must remain explicitly justified and runtime-safe.

### 19.2 Reflection

Reflection must preserve the distinction between a basis value and its realization.

```python
let a = i
// Basis<I>

let b: Complex<Float> = i
// Complex<Float>
```

Their reflected concrete types must differ.

The contextual relationship does not establish classifier equality.

This is consistent with Telos's separation of semantic domains, canonical type identities, and static descriptions.

### 19.3 Equality

There is one additional design boundary I would make explicit.

These statements should be distinguished:

```python
let qi: Quaternion<Int> = i
let qj: Quaternion<Int> = j

Test.expectEqual(qi*qj, Quaternion(0, 0, 0, 1))
```

versus:

```python
i*j == k
```

The first is straightforward typed quaternion equality.

The second requires the contextual system to realize `k` consistently with the result of `i*j`.

That is reasonable to support, but it should be implemented through the same contextual elaboration contract as assignments and arithmetic.

Similarly:

```python
i*i == -1
```

requires either a declared scalar/complex equality operation or contextual realization of the right operand.

I would not silently add universal cross-type numeric equality as part of the basis feature.

For the initial implementation, tests should compare concrete values of the same selected algebra.

---

## 20. Third-party algebraic extensions

The proposed `Basis<Axis>` representation has an important advantage: it provides a possible foundation for user-defined algebraic units.

Consider a future geometric algebra package.

It might define:

```python
data E1()
data E2()
data E3()

const e1 = Basis<E1>()
const e2 = Basis<E2>()
const e3 = Basis<E3>()
```

A concrete algebra type could then declare how those units are represented.

Conceptually:

```python
data Multivector<T>(...)
```

And:

```python
impl<T> BasisRealizable<Basis<E1>> for Multivector<T> {
  ...
}
```

The developer could write:

```python
let v: Multivector<Float> = e1
```

This follows the same realization model as the built-in imaginary units.

However, the compiler should not automatically infer:

```python
let v = 2.0*e1
```

as `Multivector<Float>` merely because that happens to be one available candidate.

An external algebra needs a concrete contextual target unless Telos eventually establishes a separately specified canonical-default mechanism for it.

That restriction prevents imported packages from unpredictably changing the inferred types of otherwise unchanged expressions.

### 20.1 An important limit on extensibility

This proposal does **not** automatically provide generalized geometric algebra syntax.

For example:

```python
e1 * e2
```

requires the multiplication law of the selected algebra.

Different geometric algebras have different signatures and multiplication contracts.

The type `Basis<E1>` by itself cannot determine that product universally.

Therefore, the initial generic abstraction should support contextual basis realization, not a global symbolic multiplication table.

This is the correct boundary between reusable language semantics and domain-specific mathematics.

---

# Part VI — Implementation Program

## 21. Recommended implementation sequence

I would organize the implementation as a dedicated language-and-library program with five bounded phases.

Each phase should have its own acceptance gate.

This is an architectural sequence rather than a patch-level plan.

### Phase 1 — Numerical structures and algebraic capabilities

**Goal:** Establish correct `Complex<T>` and `Quaternion<T>` implementations independently of new syntax.

Implement:

- Nominal data declarations.
- Constructors and component access.
- Addition, subtraction, negation, multiplication.
- Conjugation and squared norm.
- Scalar identities.
- Explicit real embedding.
- Explicit complex-to-quaternion embedding.
- Checked complex projection.
- Appropriate equality behavior.
- Operation-specific capability bounds.

Verify:

```python
let z = Complex(3, 4)
let q = Quaternion(1, 2, 3, 4)
```

and all required algebraic identities.

**Acceptance condition:** Mathematical structures are correct using ordinary constructor syntax, without contextual basis inference.

This isolates standard-library correctness from compiler changes.

### Phase 2 — First-class algebraic units

**Goal:** Establish the canonical basis values and their stable identity.

Implement:

```python
data Basis<Axis>()

data I()
data J()
data K()

const i = Basis<I>()
const j = Basis<J>()
const k = Basis<K>()
```

Then establish the explicit realization capability and its concrete implementations.

Validate:

- Static construction.
- Exported constant identity.
- Module re-exports.
- Alias and shadowing behavior.
- Generic data identity.
- Realization conformance coherence.
- Correct scalar zero and one construction.

**Acceptance condition:** The basis values exist as independent ordinary Telos values, and their explicit realization operations work correctly.

No contextual assignment behavior is required yet.

### Phase 3 — Contextual basis realization

**Goal:** Support contextual target realization without introducing native imaginary literals.

Required examples:

```python
from math import i, j, k

let z: Complex<Float> = i
let q: Quaternion<Float> = i

let a = 2*i
let b = 3*j
let c = 4*k

let d = i*j
let e = j*i
```

This phase owns the main semantic architecture.

It must establish:

1. Contextual realization evidence.
2. Expected-type propagation.
3. Basis-bearing operator inference.
4. Deterministic canonical defaults.
5. Generic inference boundaries.
6. Stable semantic products.
7. Compiler lowering.
8. Precise diagnostics.
9. Cross-module behavior.

**Acceptance condition:** Every required expression has a unique, justified type and correctly executes using standard-library operations.

No special runtime algebra objects are introduced.

### Phase 4 — Native imaginary literal syntax

**Goal:** Support `i`, `j`, and `k` suffixes using the same underlying numerical realization model.

Implement:

```python
2i
3j
4k

1 + 2i
1 + 2i + 3j + 4k
```

This phase modifies lexical recognition, parsing, semantic interpretation, and lowering.

It must reuse Phase 3's canonical algebra selection and realization machinery.

**Acceptance condition:** Suffix literals and equivalent basis-coefficient expressions agree for supported matching scalar domains.

The implementation must additionally prove lexical compatibility and precedence correctness.

### Phase 5 — Static realization, optimization, and complete integration

**Goal:** Complete the language feature's compile-time and performance contracts.

Implement and verify:

- Static basis realization where statically eligible.
- Composite static complex/quaternion construction.
- Cross-module constant materialization.
- Canonical semantic serialization and incremental products.
- Literal-fusion optimization where semantics permit.
- Relevant LSP type presentation and diagnostics.
- Performance and allocation benchmarks.
- Final package API stabilization.

**Acceptance condition:** The implementation has no outstanding correctness gaps in the approved language contract, and optimization does not change observable semantics.

---

## 22. Comprehensive verification requirements

I would require several complementary test suites.

### 22.1 Mathematical property tests

For suitable scalar domains:

| Property | Required verification |
|---|---|
| Complex addition | Componentwise result |
| Complex multiplication | Rectangular multiplication formula |
| Complex conjugation | Negation of imaginary component |
| Quaternion addition | Componentwise result |
| Quaternion multiplication | Hamilton product |
| Quaternion associativity | `(p*q)*r == p*(q*r)` for exact scalar arithmetic |
| Noncommutativity | `i*j != j*i` |
| Quaternion norm identity | `q*conjugate(q)` gives scalar squared norm |
| Complex embedding | Preserves addition and multiplication |
| Quaternion conjugation | Reverses multiplication order |
| Exact integer arithmetic | No silent rounding |
| Floating-point behavior | NaN, infinity, signed zero, overflow, underflow |

For exact integer quaternions:

\[
\overline{pq}=\overline q\,\overline p
\]

\[
N(pq)=N(p)N(q).
\]

These are excellent property-based testing targets.

For floating-point arithmetic, mathematical identities should be tested under carefully selected tolerances and exceptional-value policies, not blindly asserted as bitwise equalities.

### 22.2 Literal and inference matrix

At minimum:

```python
let a = i
// Basis<I>

let b = j
// Basis<J>

let c = 2*i
// Complex<Int>

let d = 2.0*i
// Complex<Float>

let e = 2*j
// Quaternion<Int>

let f = i*j
// Quaternion<Int>

let g: Complex<Float> = i
// Complex<Float>

let h: Quaternion<Float> = i
// Quaternion<Float>

let q = 1 + 2i + 3j + 4k
// Quaternion<Int>
```

Negative cases must include incompatible algebra targets and scalar-type mismatches.

### 22.3 Algebraic context propagation

Test:

```python
let q = i + j
let q = j + i

let q = (i + i) + j
let q = i + (i + j)

let q = 2*i + 3*j
let q = 2*i + 3*j + 4*k

let q: Quaternion<Float> = i + j
```

And test named values separately:

```python
let x = i
let y = j

let q = x + y
```

This must work from the types of the basis values, not merely from recognizing the original imported identifiers.

### 22.4 Module and identity tests

Test importing the units directly:

```python
from math import i
```

Through aliases:

```python
from math import i as imaginary
```

Through re-exports, qualified module access, and intermediate bindings.

Test lexical shadowing:

```python
from math import i

let i = 42
```

And verify that:

```python
2i
```

remains an imaginary literal.

### 22.5 Generic inference tests

Verify:

- Generic identity preserves `Basis<I>`.
- An expected concrete algebra permits realization.
- Conformance evidence is required.
- Generic return types are not guessed from unrelated candidates.
- Missing evidence is rejected.
- Ambiguous evidence is diagnosed.
- Generic specialization retains the selected callable identity.
- Cross-module generic calls produce the same result as local calls.

These tests are particularly important given the repository's existing complex generic inference infrastructure.

### 22.6 Static and incremental tests

Verify:

- Static `Basis<I>` construction.
- Static constant export.
- Static realization of approved operations.
- Stable canonical constant identity.
- No runtime evaluation of a supposedly static initializer.
- Correct dependency invalidation.
- Cold analysis and incremental reanalysis agreement.
- No inference-local variables escaping into durable metadata.

### 22.7 Compiler and runtime tests

Verify:

- Operand evaluation order.
- Single evaluation of effectful coefficients.
- Correct method-family selection.
- Correct reflected-operation behavior.
- No unexpected dynamic fallback.
- Ordinary data reflection.
- Correct behavior across garbage collection.
- No value-identity corruption through optimization.
- Correct behavior for very large `Int` coefficients.

### 22.8 Performance tests

I would benchmark:

```python
Complex<Float>
Quaternion<Float>
```

using:

- Construction.
- Addition.
- Multiplication.
- Conjugation.
- Norm calculation.
- Sequential accumulation.
- Large collections of values.
- Literal-heavy constant expressions.

Benchmark both time and allocation behavior.

An important comparison is between:

```python
let z = Complex(3.0, 4.0)
```

and:

```python
let z = 3.0 + 4.0i
```

Similarly:

```python
let q = Quaternion(1.0, 2.0, 3.0, 4.0)
```

versus:

```python
let q = 1.0 + 2.0i + 3.0j + 4.0k
```

Native notation should not create avoidable allocation overhead compared with ordinary construction.

However, I would not require an arbitrary zero-allocation benchmark result before understanding the current VM's representation model.

---

# Part VII — Specification and Repository Consolidation

## 23. Required specification updates

The feature crosses multiple semantic domains. It should not be specified only in a math implementation record.

The repository's specification authority rules require each semantic concept to be owned by its canonical specification.

I recommend:

| Specification | Required change |
|---|---|
| `docs/specs/README.md` | Register the new language feature and its normative owner |
| Numeric lexical specification / TDR-0070 successor | Define `i`, `j`, `k` suffix grammar |
| `docs/specs/typing/` | Define contextual realization and related typing judgments |
| `docs/specs/generic-constraints.md` | Specify realization obligations across generic contexts |
| `docs/specs/objects/traits.md` | Clarify realization conformance coherence |
| `docs/specs/bindings-and-fields.md` | Clarify basis-valued bindings versus realized assignments |
| `docs/specs/static-evaluation.md` | Define static realization eligibility |
| `docs/specs/callables/dispatch.md` | Clarify interaction between contextual elaboration and normal operator dispatch |
| `docs/specs/universe.md` | Establish ownership of canonical basis declarations and numeric types |
| `docs/math-proposals/` | Reconcile the older complex proposal with the new approved design |

I would also require a dedicated normative specification covering:

**Contextual algebraic values and their realization.**

That document should be the single semantic authority for:

- Basis-value identity.
- Admissible realization targets.
- Expected-type influence.
- Arithmetic context inference.
- Defaulting.
- Ambiguity.
- Interaction with generic constraints.
- Static versus runtime realization.
- Semantic preservation during lowering.

The standard-library documentation would separately own the actual complex and quaternion algebra.

This avoids duplicating inference rules in math-package documentation.

---

## 24. Identified repository tensions

The investigation reveals several concrete architectural tensions.

| Tension | Current repository evidence | Required resolution |
|---|---|---|
| Numeric literal rigidity | Integer literals synthesize `Int`; Float literals synthesize `Float` | Keep coefficient types explicit initially |
| Bottom-up binary checking | `synthesize_binary_expr` analyzes both operands with no expected type | Add bounded contextual operator inference |
| Exact selector dispatch | Selector identity excludes runtime parameter types | Do not add type-only operator overloads |
| Existing `Number` contract | Runtime class includes ordering and Float-oriented division | Keep complex/quaternion as distinct data types |
| Static `const` semantics | Static evaluator does not execute arbitrary runtime methods | Prove statically eligible realization |
| Generic underconstraint | Existing inference distinguishes insufficient and conflicting evidence | Preserve underconstraint rather than guessing targets |
| Cross-module constants | Constants and exports have stable canonical identity | Reuse binding and static-constant products |
| Existing math facade | Public APIs are captured Families and constants | Preserve facade and hidden implementation topology |
| No general promotion policy | Numerical architecture rejects unrestricted implicit promotion | Restrict realization to explicitly declared basis operations |

These are not reasons to reject the design.

They determine where the implementation must be carried out and what must not be changed.

The most significant engineering task is **the semantic elaboration of basis-bearing operator expressions**.

That task should be completed and verified before native literal syntax is introduced.

---

## 25. Explicitly rejected designs

I would reject the following alternatives.

### A. Make every complex number a quaternion

This unnecessarily expands ordinary complex storage and arithmetic. It also erases a meaningful difference in algebraic laws.

### B. Export separate complex and quaternion `i` constants

This forces users to select a representation before mathematical context requires one.

It also creates undesirable namespace duplication.

### C. Give `math.i` a fixed `Complex<Int>` type

This preserves a simple constant implementation but makes contextual Float and quaternion realization require additional conversion rules.

It does not satisfy the desired unified semantics.

### D. Make the compiler recognize identifiers named `i`, `j`, and `k`

This breaks lexical independence and creates problems with imports, aliases, shadowing, and user-defined names.

Only numeric suffixes should have lexical recognition.

### E. Introduce general implicit conversion

A universal conversion mechanism would be much more powerful than necessary and would interact unpredictably with Telos's exact selector and generic inference models.

### F. Add a generic `Hypercomplex<T,N>` runtime representation

This models component counts while failing to capture the different mathematical laws of complex numbers, quaternions, and octonions.

Generic algebraic construction may be useful internally, but it should not replace their distinct public types.

### G. Implement symbolic quaternion multiplication in the compiler

The compiler should understand contextual typing and realization, not duplicate the mathematical operations implemented by the standard library.

### H. Introduce `@contextual const` immediately

The proposed ordinary `Basis<Axis>` values already provide stable identity and aliasing.

Contextual realization should first be implemented as a semantic operation on explicitly admissible values.

A new declaration annotation should only be considered if concrete third-party use cases demonstrate that the ordinary data-and-capability model is insufficient.

---

# Part VIII — Consolidated Final Design

## 26. The proposed public Telos experience

The following is the complete intended developer-facing model.

### Ordinary complex numbers

```python
from math import Complex, i

let z = 3 + 4*i
// Complex<Int>

let w = 3 + 4i
// Complex<Int>

let f = 3.0 + 4.0i
// Complex<Float>
```

### Quaternion numbers

```python
from math import Quaternion, i, j, k

let q = 1 + 2*i + 3*j + 4*k
// Quaternion<Int>

let r = 1 + 2i + 3j + 4k
// Quaternion<Int>
```

### Contextual unit realization

```python
from math import i, j, k

let unit = i
// Basis<I>

let a: Complex<Float> = unit
// Complex<Float>(0.0, 1.0)

let b: Quaternion<Float> = unit
// Quaternion<Float>(0.0, 1.0, 0.0, 0.0)

let c: Quaternion<Int> = j
// Quaternion<Int>(0, 0, 1, 0)
```

### Arithmetic inference

```python
let a = i*i
// Complex<Int>(-1, 0)

let b = i*j
// Quaternion<Int>(0, 0, 0, 1)

let c = j*i
// Quaternion<Int>(0, 0, 0, -1)

let d = i + j + k
// Quaternion<Int>(0, 1, 1, 1)
```

### Typed arithmetic

```python
let qi: Quaternion<Int> = i
let qj: Quaternion<Int> = j
let qk: Quaternion<Int> = k

Test.expectEqual(qi*qj, qk)
Test.expectEqual(qj*qi, -qk)
```

### Generic basis identity

```python
from math import i

class Example {
  @class
  identity<T>(_ value: T) -> T {
    value
  }
}

let x = Example.identity(i)
// Basis<I>

let y: Complex<Float> = x
// Complex<Float>
```

### Mixed complex/quaternion arithmetic

```python
let z = 3 + 4i
// Complex<Int>

let q = 1 + 2i + 3j + 4k
// Quaternion<Int>

let result = z + q
// Quaternion<Int>(4, 6, 3, 4)
```

### Explicit algebraic embedding

```python
let z = Complex(3, 4)

let q = Quaternion(z)
// Quaternion(3, 4, 0, 0)
```

These examples collectively exercise the intended language contract.

They should eventually become normative positive tests.

---

## 27. Final architectural invariants

I would establish the following as the non-negotiable design contract.

| ID | Invariant |
|---|---|
| **ALG-01** | `Complex<T>` and `Quaternion<T>` are distinct nominal data types. |
| **ALG-02** | `Basis<Axis>` identifies a basis element independently of its numeric realization. |
| **ALG-03** | `math.i`, `math.j`, and `math.k` are canonical exported values, not reserved identifiers. |
| **ALG-04** | A basis value's type and identity remain stable across aliases and imports. |
| **ALG-05** | Contextual realization requires explicit, valid target evidence. |
| **ALG-06** | Realization does not establish subtyping or type equivalence. |
| **ALG-07** | Unconstrained standalone basis values retain their basis types. |
| **ALG-08** | Canonical arithmetic defaults are deterministic and independent of unrelated imports. |
| **ALG-09** | Generic inference cannot invent concrete algebra types without sufficient evidence. |
| **ALG-10** | Native imaginary literals and basis-coefficient expressions share one realization contract. |
| **ALG-11** | Ordinary selector identity and dispatch semantics remain unchanged. |
| **ALG-12** | Complex/quaternion arithmetic is owned by the standard library. |
| **ALG-13** | Complex-to-quaternion embedding is not a general implicit conversion. |
| **ALG-14** | Quaternion multiplication remains order-sensitive. |
| **ALG-15** | Scalar precision and exactness rules are preserved. |
| **ALG-16** | Compiler lowering consumes canonical checked semantic evidence. |
| **ALG-17** | Static realization never executes arbitrary runtime methods. |
| **ALG-18** | Numeric optimizations preserve observable language semantics. |
| **ALG-19** | No general symbolic algebra engine or conversion lattice is introduced. |
| **ALG-20** | Third-party algebraic types participate through explicit capabilities without destabilizing canonical defaults. |

---

## 28. Repository verification status

This design is based on targeted inspection of the actual GitHub repository at `main`, revision `61c8734f`.

I inspected the existing mathematical design records, Universe math package, scalar declarations, lexer/parser structures, expression checker, current binary dispatch implementation, binding and static constant machinery, and the relevant normative specifications.

**Important qualification:** The inspected `main` tree does not yet expose an implemented `Complex<T>` or `Quaternion<T>` standard-library data declaration. The user's ongoing complex-number implementation may exist on another branch or in local work not represented by this revision. Consequently, the proposed source APIs and compiler modifications are not claimed to be already implemented or tested.

No compiler build or test suite was executed as part of this read-only design analysis.

The design should be reconciled with the ongoing complex-number branch before implementation begins, particularly its existing capability declarations and constructor conventions.

---

# Final Recommendation

I recommend approving this design with one central architectural commitment:

**Telos should treat algebraic basis values as first-class, representation-independent values that admit restricted contextual realization into explicitly supported mathematical structures.**

The resulting model is:

\[
\boxed{
\text{Basis identity}
\xrightarrow{\text{contextual realization}}
\text{Concrete algebraic value}
}
\]

With:

\[
\operatorname{Basis}\langle I\rangle
\longrightarrow
\begin{cases}
\operatorname{Complex}\langle T\rangle\\
\operatorname{Quaternion}\langle T\rangle
\end{cases}
\]

And:

\[
\operatorname{Basis}\langle J\rangle,\operatorname{Basis}\langle K\rangle
\longrightarrow
\operatorname{Quaternion}\langle T\rangle
\]

for the initial canonical numeric family.

This achieves the originally desired properties simultaneously:

**A unified mathematical vocabulary:**

```python
from math import i, j, k
```

**Efficient specialized representations:**

```python
Complex<T>
Quaternion<T>
```

**Natural mathematical notation:**

```python
3 + 4i
1 + 2i + 3j + 4k
```

**Contextual polymorphism without general implicit conversion:**

```python
let z: Complex<Float> = i
let q: Quaternion<Float> = i
```

**And a reusable semantic foundation for future algebraic types.**

The most consequential decision is not whether Telos supports the suffix `j` or the constructor `Quaternion(...)`.

It is whether Telos can distinguish **a mathematical value's identity from the concrete representation selected for one use of that value**, while retaining precise typing, explicit evidence, ordinary binding semantics, and predictable execution.

I believe the proposed bounded realization model is the right way to make that distinction.

**Implementation priority:** Establish the ordinary algebraic data and realization contracts first, then repair the contextual expression inference boundary, and only afterward add native literal syntax. That sequence minimizes architectural risk and prevents the numeric feature from becoming a collection of isolated compiler special cases.
