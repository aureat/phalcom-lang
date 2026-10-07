# 11. Gaps, rejected alternatives, V1 boundary, and future phases

## 11.1 Standard-library and language-gap ledger

| Finding | Classification | Why it matters | Recommendation |
|---|---|---|---|
| No Rational type or normalized quotient value | Library gap | Exact fractions are needed by probability, symbolic-adjacent numeric code, and exact algorithms | Add `Rational` as a later `universe.numbers` data type; does not block current Float math |
| No Complex type or branch-cut contract | Library gap plus specification work | Complex scalar support is required by some linear algebra/scientific consumers | Design now; implement after `Complex<Float>` semantics and backend are approved |
| Current `Number` is too broad for generic laws | Typing/library gap | `Number` does not say whether operations are exact, ordered, closed, or field-like | Add small numeric capability traits and use explicit conformance; no immediate compiler change required |
| No generic result-type/promotion mechanism for numeric operations | Typing gap | `Complex<T> / Complex<T>` and Rational/Float operations have different result policies | Initially use explicit families/conversions. Consider a finite associated-result or promotion relation only after concrete use cases |
| No generic constant syntax proven by current tests | Ergonomic/typing gap | `pi` cannot safely adapt to arbitrary precision without a representation contract | Keep typed `Float` constants; expose future precision-specific constants through explicit APIs |
| Operator dispatch has bilateral reflected fallback | Existing compiler facility | Third-party numeric types can offer left/right cooperation without type-directed selector lookup | Reuse current mechanism; do not add type-based overload dispatch |
| `Ordering` currently models numeric compare | Runtime/library gap | Float NaN creates unordered cases while Complex has no order | Add or clarify a partial numeric comparison result before generic ordered algorithms depend on it |
| Float classification lacks a complete public taxonomy | Library gap | Statistics/probability need finite/NaN/infinite/subnormal checks | Add predicates first; add `FloatClass` only if users need exhaustive classification |
| Backend precision is platform-sensitive | Compiler/runtime contract gap | Native Rust `f64` functions cannot promise universal ULP/reproducibility | Document backend-level guarantees; add deterministic backend only as a separately funded decision |
| Numeric policy/resource limits are not a generic numeric extension interface | Compiler/runtime gap | Rational/Complex can create large intermediate values | Make future types route through the same policy or explicitly declare a separate policy budget |
| Generic algorithms depend on current semantic evidence products | No gap found | Traits, associated types, guarded evidence, and generic calls already exist | Keep formal authority in semantic layer; consumers must not reconstruct facts |

## 11.2 Alternatives rejected

### A universal implicit promotion lattice

Rejected for V1. It creates ambiguity between exactness and convenience, makes
third-party participation global, and interacts poorly with Telos's exact selector
identity. `Int + Float` already has a deliberate built-in meaning; that does not
justify silently promoting Rational, Decimal, Complex, or arbitrary user types.

### No mixed arithmetic at all

Rejected as too hostile for ordinary code. Built-in Int/Float mixed arithmetic
already has an accepted contract, and Complex real-scalar ergonomics matter.
The compromise is a small built-in matrix plus explicit library conversions.

### Full Python numeric ABC tower

Rejected as a direct model. Python's `Number`/`Complex`/`Real`/`Rational`/
`Integral` hierarchy is useful documentation, but its extension protocol relies
on reverse methods and boilerplate. Telos's nominal traits/evidence system offers
a smaller capability design with stronger coherence.

### Full Julia-style promotion and parametric numeric families

Rejected for the first phase. Julia demonstrates that parametric `Rational{T}`,
`Complex{T}`, and promotion can compose, but it depends on a mature multiple-
dispatch and conversion ecosystem. Telos has exact selector dispatch and a
different trait model; copying the surface would import complexity before the
semantic contract is ready.

### Making `Real`, `Exact`, or `Approximate` runtime classes

Rejected. These concepts describe laws/properties and can cut across concrete
representations. Runtime classes would encourage type tests instead of capability
constraints and would make future Decimal/BigFloat choices harder.

### Implementing Complex as an untyped pair only

Rejected. A pair cannot express division result laws, no-ordering, branch cuts,
component exactness, or IEEE special values. The pair is the representation idea,
not the complete semantic abstraction.

### Adding every traditional special function to V1

Rejected. `erf`, gamma, beta, Bessel functions, and distributions require backend
and accuracy decisions. Record their consumers and seams now; implement in a
later special-functions phase.

## 11.3 Recommended V1 boundary

### Implement now / preserve now

- `Int` exact arbitrary-precision surface semantics and numeric policy.
- `Float` binary64 semantics, current predicates, and honest special-value
  behavior.
- Current `universe.math` Float Families, constants, angle helpers, `gcd`, `lcm`,
  and `isqrt`.
- Exact comparison and numeric hash laws across current Int/Float values.
- `universe.math` facade/hidden-child topology and Family capture convention.
- Documentation of explicit Float boundaries and backend precision limits.

### Design now, implement after focused follow-up

- `Rational` normalized data value and exact operations.
- `Complex<T>` core data/operations, with `Complex<Float>` as first backend.
- Capability traits for additive, multiplicative, ordered, Euclidean, and
  conjugate/norm operations, with exact names selected from concrete algorithms.
- Explicit conversion matrix and future finite promotion relation requirements.
- Float classification and representation utilities (`fma`, `nextUp`, `ulp`,
  `copySign`) with ownership on `Float`.
- Complex branch-cut and special-value specification.

### Explicitly outside V1

- Arbitrary-precision `Real`/`BigFloat`.
- Symbolic algebra or exact algebraic-number roots.
- A global implicit promotion lattice.
- General special functions and distributions.
- Statistics, probability, simulation, linear algebra, tensors, autodiff,
  optimization, signal processing, units, and scientific I/O.

## 11.4 Open architectural questions

1. Should Telos's formal numeric capability traits use operator requirements,
   class-side constructors, or callable-family members as their canonical shape?
2. Should ordinary numeric comparison expose `Unordered` explicitly, or should
   `compare` retain the current `Ordering` plus a separate `totalCompare`?
3. Should numeric equality unify `Rational(1,2)` with a future exact decimal or
   only with Int/integral Float values?
4. Is a public `FloatClass` enum worth its surface, or are predicates enough?
5. Should `Complex<Float>` be a standard generic data value with generic syntax,
   or a specialized concrete type first to reduce type-system risk?
6. What backend and reproducibility level should complex elementary functions
   promise across platforms?
7. Does a future finite promotion relation belong in the semantic type system,
   compiler operator lowering, or a library-level explicit `promote` API?
8. Can static evaluation represent typed constants parametrically without
   making runtime representation or arbitrary user code part of static semantics?
9. Which numeric resource-policy fields should apply to Rational and Complex
   intermediate values, and how should composite cost be charged?

## 11.5 Future phases

### Phase 2 — Rational and exact conversion

Ratification of normalization, construction, arithmetic, equality/hash,
formatting, `toFloat`, `toIntExact`, policy accounting, and property tests.

### Phase 3 — Capability traits and generic scalar algorithms

Select the smallest trait set justified by Rational/Complex and one generic
algorithm family. Add semantic evidence, coherence tests, and no type-based
selector dispatch.

### Phase 4 — Complex core and `Complex<Float>` backend

Implement data representation, arithmetic, conjugate/norm, robust division,
formatting, equality, and real-scalar interop.

### Phase 5 — Complex elementary functions

Specify and implement principal branches, signed zero, NaN/infinity behavior,
accuracy, and backend portability. Add `math.complex` only after this contract.

### Phase 6 — Special functions and numerical algorithms

Evaluate `erf`, `erfc`, gamma/log-gamma, beta, stable summation, FMA, and
reproducible reduction as independent packages or bounded math extensions.

### Phase 7 — Consumer packages

Only after the foundation is stable: statistics/probability, linear algebra,
simulation, optimization, and scientific computing can select their scalar
requirements from the shared capabilities.
