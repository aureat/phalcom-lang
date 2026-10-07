# 2. Repository findings and evidence ledger

## 2.1 Naming and authority

**Observed:** The live repository is Phalcom, not Telos. `docs/specs/README.md`
names Phalcom as the current language and says `docs/specs/` is authoritative.
The repository contains `phalcom-ast`, `phalcom-semantic`, `phalcom-core`, and
related crates. This report assumes Telos is the requested language identity for
this project and cites the live Phalcom tree as its evidence base.

**Design consequence:** Proposed syntax and semantics must follow the current
Phalcom grammar and semantic conventions until a later decision explicitly changes
them.

## 2.2 Bounded evidence ledger

| Capability | Evidence | Observed behavior | Numerical consequence |
|---|---|---|---|
| Numeric runtime representation | `phalcom-core/src/value/repr.rs:17-33, 57-119`; `value/mod.rs:30-38, 130-172` | `Int` and `Float` are immediate tags; large integers are `Object::LargeInt` but report class `Int` | `Int` is already a two-tier exact surface type; do not add public `BigInt` merely for representation |
| Exact integer arithmetic | `phalcom-core/src/primitive/number.rs:117-155, 205-223, 372-458`; `tests/fixtures/language/math/math_integer_large.ph` | Checked small operations promote through `normalize_bigint`; large operands remain exact; bit limits are enforced | Rational can reuse canonical integer normalization and numeric policy |
| Numeric literal typing | `docs/decisions/accepted/0070-numeric-literals.md:10-24`; `phalcom-ast/src/parser.rs` literal branches; math fixtures | Integer literals are exact `Int`; fractional/exponent literals are `Float`; no `n` suffix | Do not design imaginary suffixes or public BigInt literal syntax in V1 |
| Division semantics | `docs/decisions/accepted/0022-numeric-surface-split-int-float-and-division.md:67-89`; `0069-numeric-tower-residue-rulings.md:32-70` | `/` is true division; `~/` is floor division returning exact `Int`; Float-to-Int narrowing is explicit | Rational division must not reuse `/`'s Float result; exact quotient APIs need distinct names/contracts |
| Mixed comparison | `phalcom-core/src/primitive/number.rs:58-86`; `tests/core/audit_probes/numeric_contracts.rs:98-109` | Int/Float comparison decomposes finite Float exactly instead of converting Int through f64 | Exact comparison is a first-class semantic kernel, not an incidental operator helper |
| Equality and hashing | `phalcom-core/src/value/mod.rs:287-475`; `primitive/number.rs:305-345`; `tests/fixtures/language/arithmetic/number_equality_cross_type.ph` | `==` compares numeric value across Int/Float; `===` is exact representation/sameness; hash handling is under active numeric contract work | Rational equality/hash must be value-canonical; Complex equality cannot be ordered and must define NaN behavior separately |
| Floating special values | `phalcom-core/src/primitive/float.rs:39-231`; `math_special_values.ph/.expected` | NaN, infinities, signed zero, finite checks, and backend domain results are observable | Every Float API needs an IEEE classification contract; `Real` must not mean “finite Float” |
| Current elementary math | `phalcom-core/src/primitive/math.rs:1-130`; `core/universe/src/math/{package,float_kernel,integer}.ph`; math fixtures | `universe.math` exposes captured Families for Float kernels, constants, angle conversions, `gcd`, `lcm`, `isqrt` | Preserve Family capture and facade architecture; extend by explicit families, not wrapper closures |
| Current math package contract | `docs/implementation/LANG007/.../stdlib-math-aggregate-audit-and-design.md:15-31, 165-207, 748-1065` | Float-only transcendental API; exact integer helpers; no Complex/Rational/special functions | Existing math is a coherent Float layer, not a general numeric tower |
| Method-family references | `docs/specs/callables/{references,families,arguments,dispatch}.md`; `tests/core/execution/family_runtime.rs:41-121, 456-509` | Exact callable references retain selector shape; Families are live/snapshot capabilities and GC roots | Public math functions should remain captured Families; generic algorithm parameters can accept callable capabilities |
| Selector identity | `docs/specs/callables/arguments.md:21-23, 378-546`; `dispatch.md:117-182`; `compiler/lib/expr.rs:2039-2167` | Runtime types do not participate in selector identity; operators use exact sends plus bilateral reflected fallback | Do not model numeric overloads as type-based selector dispatch; mixed behavior must be explicit in method bodies/capabilities |
| Traits and conformance | `docs/specs/objects/traits.md:1-66, 171-189, 418-453, 811-859`; `tests/core/language/generic_conformance.rs:13-136`; `semantic/tests/semantic/impls/queries.rs:205-279` | Nominal traits, explicit `impl`, associated types, `Self`, guarded evidence, coherence; no trait inheritance or trait objects | Use small algebraic/numeric capabilities with `where T |= ...`; do not construct supertraits |
| Data values | `docs/specs/objects/data-objects.md:1-68, 140-192, 692-753`; `tests/core/language/data_e2e.rs:184-205, 310-358` | Nominal, transparent, shallowly immutable, generic data; exact type identity matters; `impl` adds behavior but no state | Rational and Complex should be data families with canonical semantic state, not mutable classes |
| Static values/constants | `docs/specs/static-evaluation.md`; `semantic/tests/semantic/integration/static_definition_bodies.rs:20-105`; math audit §9-14 | `const` is statically evaluated/materialized and has stable semantic identity; exported constants exercised module machinery | Constants should be typed API members/static values, not initialization-time computations; generic constants need existing static-domain support or explicit future work |
| Packages and exports | `core/universe/src/math/package.ph`; math audit §15-19; module specs | A facade can import hidden child modules and export captured public values; hidden implementation topology need not be exposed | Keep one public `universe.math`; place future kernels in hidden children |
| Generic type machinery | `phalcom-semantic/tests/semantic/foundations/type_members.rs`; `tests/core/language/associated_traits.rs:84-204`; `type-meta/src/generic.rs` | Applied types, type parameters, associated types, static/value parameters, and evidence exist in the semantic layer | A generic `Complex<T>` is plausible, but operation result typing and promotion need explicit design before implementation |

## 2.3 Important contradictions and status notes

1. `docs/decisions/accepted/0022...` and `docs/decisions/accepted/0069...` describe
   the intended exact Int/Float tower. The older skill overlay says the split is
   “unbuilt,” but the live tree contains `LargeInt`, exact mixed comparison, and
   arbitrary-size math fixtures. The report treats live implementation as current
   behavior and the accepted records as intended semantic authority; the overlay
   status is stale for this question.
2. `docs/specs/README.md` describes Phalcom's older import convention as
   `import "./path" as Name`, while the current math package and many current
   tests use `from ... import ...`. This report uses syntax from current math and
   generic tests and marks future syntax sketches as proposed where needed.
3. The math implementation handoff says focused math tests pass while the plan is
   still partial because Float IEEE/accuracy and large-Int matrices remain. This
   report therefore treats the existing math surface as a current implementation,
   not as a complete numerical foundation.

## 2.4 Evidence confidence

- **High:** runtime representation, current math API, exact integer algorithms,
  Family behavior, traits/data/generic precedents.
- **Medium:** final public numeric specification because implementation records,
  accepted decisions, and the overlay have status drift.
- **Low / absent:** Rational, Complex, generic numeric capability declarations,
  arbitrary-precision Float, complex branch-cut behavior, and generic constant
  syntax. No current implementation precedent was found for these.
