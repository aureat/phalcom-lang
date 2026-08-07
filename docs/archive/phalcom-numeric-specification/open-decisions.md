# Numeric Open-Decision Register

> **Status:** Open items remaining after the 2026-07-29 numeric architecture ratification.
>
> **Rule:** These items do not reopen the public tower or the ratified semantic architecture. Each decision must be resolved explicitly; host behavior, provisional implementation, and examples are not implicit ratification.

## 1. Summary

| ID | Decision | Class | Blocks |
|---|---|---|---|
| OD-NUM-001 | Public names for exact Float-to-Int conversion and total comparison | Surface naming | Float protocol publication |
| OD-NUM-002 | Exact total-order sequence | Public semantics | Total-order implementation |
| OD-NUM-003 | Complete Float `%` and `**` special-case tables | Public semantics | Float protocol and power shipping |
| OD-NUM-004 | Numeric hash reduction, constants, width, and seed policy | Runtime semantics/implementation | Map/Set conformance |
| OD-NUM-005 | Resource-limit defaults and configuration API | Runtime/compiler policy | Numeric limits shipping |
| OD-NUM-006 | Native-versus-derived bitwise selector composition | VM architecture/floor | U-BITWISE |
| OD-NUM-007 | Fast-path invalidation mechanism | VM architecture | Numeric optimization |
| OD-NUM-008 | Large-constant binary encoding and sharing | Compiler/runtime architecture | Serialized constants |
| OD-NUM-009 | Decimal parser and shortest-render algorithm | Implementation selection | Canonical Float text fixtures |
| OD-NUM-010 | Compatibility and release strategy | Release policy | Numeric update release |
| OD-NUM-011 | Future public numeric-extension protocol | Deferred language architecture | Future numeric types only |

Blocking for the first complete numeric release: **OD-NUM-001 through OD-NUM-006, OD-NUM-008 through OD-NUM-010**. OD-NUM-007 blocks optimized paths but not a correct generic implementation. OD-NUM-011 is deliberately deferred.

---

## OD-NUM-001 — Public selector names

### Question

Choose source spellings for:

1. exact integral Float-to-Int conversion;
2. total numeric comparison.

### Exact-conversion options

| Option | Example | Strength | Cost |
|---|---|---|---|
| A | `x.toIntExact` | Explicit target and failure condition; familiar | `to` may imply conversion rather than query |
| B | `x.integerExact` | Compact, noun-like | Less conventional English |
| C | `x.asIntExact` | Signals checked view/conversion | `as` often suggests non-allocating reinterpretation |
| D | `x.exactInt` | Very compact | Reads like a property rather than failing operation |

### Total-comparison options

| Option | Example | Strength | Cost |
|---|---|---|---|
| A | `a.totalCompare(b)` | Clear and direct | Verb order differs from IEEE naming |
| B | `a.compareTotal(b)` | Close to IEEE `totalOrder` vocabulary | Slightly less natural at call site |
| C | `a.compare(b, total: true)` | Avoids selector growth | Mode flag weakens semantic clarity and dispatch identity |
| D | `a.totalOrder(b)` | Closest to IEEE terminology | Sounds Boolean rather than three-way comparison |

### Constraints

- Exact conversion must not be confused with `truncated` or value-dependent construction.
- Total comparison returns an ordering value or conventional negative/zero/positive Int; that result type must be specified with the name.
- Names must compose with selector literals, reflection, override, and super-send syntax.

### Non-ratified editorial default

`toIntExact` and `totalCompare(_)` are the clearest pair.

### Closure artifact

A PDR specifying exact selector signatures, return type, errors, reflection, and examples.

---

## OD-NUM-002 — Exact total-order sequence

### Question

Define a deterministic total order over Int and Float, including NaN, signed zero, infinities, and representationally distinct but numerically equal values.

### Options

#### A. IEEE-754 `totalOrder` adapted to Int

- Order Float values by IEEE totalOrder.
- Insert each Int at its mathematical position.
- Define a tie-break between equal Int and Float representations.

Strength: established treatment of NaN signs/payloads and signed zero.

Risk: exposes NaN payload ordering that ordinary Phalcom protocol otherwise hides.

#### B. Numeric-first canonical total order

Suggested coarse sequence:

```text
-Infinity
finite numeric values by exact mathematical value
+Infinity
NaN
```

Tie-break equal numeric values by canonical type/representation, and canonicalize all NaNs together.

Strength: simple, stable, aligned with public numeric concepts.

Risk: discards payload distinctions; must decide `-0.0`/`+0.0` and Int/Float ties.

#### C. Key-equivalence total preorder plus representative tie-break

First compare numeric-key equivalence classes, then compare representation only for deterministic sorting.

Strength: aligns sorted collections with Map/Set equivalence.

Risk: a “compare == 0” result may not mean public `==`, because NaNs are one key class.

### Required subdecisions

- Does total comparison return zero for two NaNs?
- Are NaN sign and payload observable in ordering?
- Is `-0.0` ordered before `+0.0` despite public equality?
- Does `1` compare-total equal `1.0`, or is one ordered first?
- What ordering value type is returned?

### Acceptance criteria

- total, antisymmetric, and transitive;
- deterministic across supported platforms;
- explicitly documented relation to `==` and numeric-key equality;
- suitable for sort and persistent index ordering;
- property-tested over raw Float bit patterns.

---

## OD-NUM-003 — Float `%` and `**` special-case tables

### Question

Complete the normative result/error matrix for non-finite and signed-zero cases.

### Float `%` cases requiring decisions

| Dividend | Divisor | Candidate choices |
|---|---|---|
| finite nonzero | `±0.0` | NaN (ratified direction) |
| `±0.0` | finite nonzero | signed zero matching divisor (ratified finite rule) |
| finite | `±Infinity` | finite dividend, signed zero, or NaN |
| `±Infinity` | finite | NaN or host-compatible rule |
| `±Infinity` | `±Infinity` | NaN |
| NaN | any | NaN |
| any | NaN | NaN |

The table must preserve the floor-remainder model where mathematically meaningful and avoid inheriting accidental host `fmod` behavior.

### Float `**` cases requiring decisions

At minimum:

```text
NaN ** 0
1 ** NaN
(-1) ** ±Infinity
±0.0 ** positive/negative exponent
negative zero with odd/even integral exponent
±Infinity ** finite exponent
finite base ** ±Infinity
negative finite base ** nonintegral Float
NaN propagation exceptions
```

### Options

A. Adopt a named platform-independent table closely matching a major established `pow` contract.

B. Adopt IEEE-754 recommended `pow` behavior where specified and define the remaining language cases.

C. Define a smaller Phalcom table optimized for conceptual regularity, even where it differs from common libraries.

### Constraints

- numeric zero to negative numeric exponent raises `#divideByZero` before table dispatch;
- ordinary finite real results remain within one ULP;
- exact Int exponent classification must not be lost through Float conversion;
- table results must pin signed zero and infinity signs;
- behavior must be platform-independent even if the underlying approximation differs within one ULP.

### Closure artifact

Two exhaustive tables plus bit-pattern conformance fixtures.

---

## OD-NUM-004 — Numeric hash algorithm

### Question

Choose the concrete canonical reduction and VM mixing model while preserving cross-type equality.

### Options

#### A. Modular rational hash

Represent every finite numeric value as exact rational `m/n`, reduce modulo a fixed prime, then apply sign and special tokens.

Strengths:

- natural extension to Rational and Decimal;
- mathematically coherent across Int/Float;
- arbitrary Int user hashes can use the same modular reduction.

Costs:

- modular inverse logic;
- careful denominator-divisible-by-modulus handling;
- exact constants become part of implementation compatibility policy.

#### B. Canonical numeric byte encoding plus keyed hash

Encode exact canonical numeric value, then feed bytes to the VM's keyed hash.

Strengths:

- straightforward per-run randomization;
- flexible internal width;
- one general mixing backend.

Costs:

- canonical rational byte encoding must make equal Int/integral Float byte-identical;
- potentially allocates or streams large values;
- future Decimal/Rational canonicalization still required.

#### C. Hybrid

Use mathematical reduction to a fixed-width canonical integer, then keyed-mix that integer for hash-table placement.

Strengths: separates equality coherence from collision hardening.

Costs: two stages and more constants.

### Required subdecisions

- internal width (`u64`, `usize`, or another fixed contract);
- per-run seed and mixing function;
- special tokens for NaN and infinities;
- arbitrary user-returned Int reduction;
- whether public `hash` exposes canonical pre-mix or VM-mixed result;
- collision behavior is ordinary and not confused with equality.

### Constraints

- all significant bits of arbitrary Int hash results matter;
- `numericKeyEqual` implies equal hash;
- hash is not persistent across VM runs unless separately promised;
- algorithm must be denial-of-service resistant for untrusted keys.

---

## OD-NUM-005 — Resource-limit defaults and configuration

### Question

Choose defaults, profiles, and configuration lifetime for numeric policy.

### Required controls

```text
maxSourceNumericDigits
maxTextConversionDigits
maxIntegerBits
maxNumericAllocationBytes
```

### Options

#### A. One universal default profile

Simple but cannot serve both trusted scientific workloads and untrusted embedding safely.

#### B. Trusted and sandbox profiles

Trusted profile uses generous/disabled arithmetic limits; sandbox uses finite deterministic limits.

This is the leading architecture.

#### C. No defaults; embedder must configure

Explicit but unsafe for command-line and novice use.

### Required subdecisions

- concrete defaults;
- whether source digit limit depends on radix;
- whether power-of-two radices receive higher/exempt linear-conversion limits;
- configuration at VM creation versus mutable at runtime;
- module cache key interaction;
- whether loaded bytecode is revalidated under a stricter VM policy;
- behavior for results whose exact size is expensive to predict;
- general execution/allocation budget integration.

### Constraints

- policy failure is deterministic;
- source uses `numeric.limit`, runtime uses `#numericLimit`;
- OOM is not misreported as policy failure;
- huge right shift/bitAt can short-circuit without allocation;
- tests can install very low limits cheaply.

---

## OD-NUM-006 — Bitwise primitive composition

### Question

Which ratified bitwise selectors are VM-blessed primitives and which are derived Phalcom methods?

### Options

#### A. All ten native

```text
& | ^ ~ << >> bitAt bitCount bitLength trailingZeros
```

Strength: best predictable performance.

Cost: largest frozen-floor increase and duplicated public bindings.

#### B. Operators native, queries derived

Native:

```text
& | ^ ~ << >>
```

Derived:

```text
bitAt bitCount bitLength trailingZeros
```

Strength: smaller floor.

Risk: poor asymptotics for large Int if derivation loops through bits.

#### C. Operators plus one introspection primitive

Native operators plus `bitLength`; derive remaining queries using internal arithmetic or private helpers.

Strength: potential balance.

Risk: `bitCount` and `trailingZeros` may still need efficient limb access unavailable to Phalcom code.

#### D. Private low-level limb/query primitive

Expose a smaller private VM hook and implement public selectors in core Phalcom code.

Strength: public floor may remain conceptually derived.

Risk: private primitive is still VM-blessed capability and must be audited; reflection/override behavior becomes more complex.

### Required evidence

- asymptotic analysis for LargeInt;
- benchmark on sparse/dense large values;
- primitive-floor derivability review;
- exact census amendment;
- no accidental public private-tier leakage.

---

## OD-NUM-007 — Fast-path invalidation

### Question

Choose the guard/invalidation mechanism for optimized numeric selectors and rendering.

### Options

A. Generic per-class/per-selector method version epochs.

B. Independent Int and Float pristine flags for each optimized selector family.

C. Global method-table generation.

D. No numeric fast paths initially.

### Constraints

- installing or replacing an Int method invalidates Int paths without depending on Number's row;
- Float overrides invalidate independently;
- subclass/closed-class rules are respected;
- generic and optimized errors/spans are identical;
- mechanism should generalize beyond numbers if practical.

### Non-ratified default

Use generic per-selector versions if already available or inexpensive; otherwise ship correct generic dispatch and defer fast paths rather than introduce a numeric-only invalidation architecture prematurely.

---

## OD-NUM-008 — Large constant encoding

### Question

Choose the serialized, heap-independent encoding of LargeInt constants.

### Options

A. Sign plus big-endian magnitude bytes.

B. Sign plus little-endian magnitude limbs.

C. Minimal two's-complement bytes.

D. Normalized source digits plus radix.

### Evaluation

- A is canonical, compact, language-neutral, and independent of host limb size.
- B is fast for one BigInt library but couples artifacts to library/host details unless normalized.
- C aligns with bitwise intuition but requires canonical sign-extension rules.
- D is simple for compiler handoff but inefficient for repeated loading and leaves parse cost in runtime.

### Required subdecisions

- endianness;
- zero representation;
- sign canonicalization;
- versioning;
- deduplication;
- per-module versus per-VM sharing;
- maximum encoded length under policy;
- hash/checksum for corrupted bytecode.

### Non-ratified default

Sign plus minimal big-endian magnitude bytes, with zero represented only by immediate `Int(0)`.

---

## OD-NUM-009 — Float parser and renderer implementation

### Question

Choose implementations that satisfy correctly rounded parsing and deterministic shortest rendering.

### Parser options

A. Standard-library parser after proving correct rounding and cross-platform consistency.

B. Dedicated correctly rounded decimal-to-binary64 algorithm/library.

C. Custom implementation.

### Renderer options

A. Ryu-family shortest formatter.

B. Schubfach/Dragonbox-family formatter.

C. Standard-library shortest formatting with a normative post-processing layer.

### Constraints

- parser rounds ties to even;
- renderer round-trips through the selected parser;
- exact notation thresholds and exponent spelling are enforced;
- signed zero and special spellings are canonical;
- output is identical across supported platforms;
- dependency license and maintenance are acceptable;
- boundary fixtures are generated from the normative algorithm, not host output.

### Closure artifact

Dependency/algorithm decision plus a generated corpus pinning every formatting boundary class.

---

## OD-NUM-010 — Release and compatibility strategy

### Question

How does the project deploy the breaking numeric semantics?

### Options

A. Immediate pre-1.0 break with migration notes.

B. One-release warning period for mechanically detectable old forms.

C. Language edition recorded per source module.

D. VM-wide compatibility mode.

### Constraints

- equality/hash/key semantics cannot vary inside one VM;
- Float `%` cannot safely have ambiguous semantics without explicit source mode;
- cached bytecode must record edition dependencies if editions are chosen;
- warnings require a fixed removal release;
- new documentation defaults to new semantics;
- runtime mode flags are strongly disfavored because library behavior becomes caller-dependent.

### Non-ratified default

If Phalcom remains pre-1.0 with a small ecosystem, choose immediate break. Otherwise use a narrowly scoped source edition, but keep key equality, hashing, and runtime representation VM-global.

---

## OD-NUM-011 — Future public numeric-extension protocol

### Question

May user-defined numeric classes participate in built-in mixed arithmetic, exact comparison, hashing, and numeric-key equality?

### Options

A. Kernel tower remains closed; user classes use ordinary methods without built-in cross-type integration.

B. Public conversion/coercion hooks.

C. Public canonical exact-value protocol.

D. Attribute/multimethod registration into the numeric semantic kernel.

### Risks

- asymmetric comparison and coercion;
- hash/equality disagreement;
- callback reentrancy during Map probing;
- performance and invalidation complexity;
- ambiguous result-type selection;
- security/resource behavior from user callbacks.

### Current disposition

Deferred. The internal canonical numeric-value layer may be designed for future extension, but no unrestricted user protocol ships with the Int/Float update.

---

## 2. Decision closure checklist

Every closed OD must update:

1. the affected normative document;
2. conformance tests;
3. implementation architecture where relevant;
4. migration notes if user-visible;
5. ADR/PDR and status records;
6. primitive census if VM bindings change;
7. this register, replacing **Open** with the final decision and date.
