# `universe.math` — Aggregate Implementation Audit, Corrective Architecture, and Package Design

**Repository:** `aureat/phalcom-lang`
**Audited branch:** `stdlib-random-math`
**Audited head:** `9742d085580a98fee8822e1c66026d179ddd772e`
**Base:** `main@615f5887f262f8adcb666f4cb4cb431e1a828e11`
**Primary implementation commit:** `3ab77b6e6b33551c68f2524aa2bfeb04405f015f` (`feat: add initial universe math implementation`)
**Scope of this report:** `universe.math` only. `universe.random` is intentionally excluded from design/planning in this turn.
**Status ruling:** **PARTIAL / NOT MERGE-READY / ARCHITECTURALLY RECOVERABLE**

---

## 1. Executive decision

The initial `universe.math` implementation is directionally strong. Its most important public-language decision is correct: mathematical operations are exported as **captured Telos method families**, not as forwarding closures or a synthetic public `Math` namespace class.

The current public shape:

```telos
let sqrt = &FloatMathKernel.sqrt(_)
let atan2 = &FloatMathKernel.atan2(_, _)
let gcd = &IntegerMath.gcd(_, _)
```

should be retained.

The source-authored exact integer algorithms (`gcd`, `lcm`, `isqrt`) are also fundamentally sound, and the native boundary for floating-point transcendental functions is appropriately narrow.

The implementation is nevertheless **not ready to merge**. The blocking issue is not mathematical computation itself; it is the general runtime handling of exported `const` values. The math package exposed an existing mismatch between Telos's semantic model—where `const` has no runtime global slot—and the module runtime/export machinery, which had assumed exported values were slot-backed. The branch begins a general repair, but the repair is unfinished and currently fails a focused runtime regression.

The package structure also needs correction. `FloatMathKernel` should not live directly in `math/package.ph`. The correct organization is a **facade package plus hidden sibling implementation modules**. Telos relative-import semantics already permit the package to import hidden children without exposing those child paths publicly, so there is no need to abuse `expose` for internal package organization.

The resulting architecture should be:

```text
universe.math
│
├── package.ph              public facade only
│   ├── public Float constants
│   ├── exact Family captures
│   └── public exports
│
├── float_kernel.ph         hidden child module
│   └── source-owned native FloatMathKernel
│
└── integer.ph              hidden child module
    └── source-authored IntegerMath
```

No `expose .float_kernel` and no `expose .integer` should appear in `math/package.ph`.

The user has explicitly accepted the current consequence that a captured `Family` can reveal its stored receiver and therefore make the kernel class value reachable through the Family's receiver getter. This report treats that as **accepted technical debt**, not as a blocker and not as justification to distort callable-reference semantics.

---

## 2. Evidence boundary and audit reconciliation note

The repository branch contains its own implementation handoff and auditor handoff, including the known failing static-constant discriminator and unresolved architectural questions. I independently audited the live branch and reconciled those records with the actual source and accepted language/module specifications.

The user also referred to a separate independent-auditor report. No distinct report body or attachment was available on the conversation/project file surface at the time this artifact was produced, so this document does **not** fabricate a line-by-line attribution to an unavailable report. The aggregate findings below are the union of:

1. the branch's recorded audit/implementation handoffs;
2. direct inspection of the branch code;
3. direct inspection of the accepted Telos numeric, module, callable-family, and static-evaluation architecture;
4. the prior architectural requirements established for this stdlib slice.

If the separate auditor artifact is supplied later, its delta should be reconciled before the patch-grade implementation plan is finalized.

---

# Part I — Current implementation audit

## 3. Branch state

The audited branch is four commits ahead of `main` and zero commits behind at the inspected state:

```text
3369adf1  docs: add stdlib random and math guidance
eaf95877  docs: record stdlib math random lifecycle handoff
3ab77b6e  feat: add initial universe math implementation
9742d085  docs: sync math implementation lifecycle
```

The math implementation adds or modifies, among other files:

```text
phalcom-core/core/universe/src/math/package.ph
phalcom-core/core/universe/src/math/integer.ph
phalcom-core/src/primitive/math.rs
phalcom-core/src/modules/static_constant.rs
phalcom-core/src/modules/materialize.rs
phalcom-core/src/modules/linkage.rs
phalcom-core/src/modules/context.rs
phalcom-core/src/heap/module.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/vm/bootstrap.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/primitive/reflection.rs
phalcom-native-meta/src/universe.rs
phalcom-native-surface/src/generated.rs
phalcom-modules/src/builtin.rs
phalcom-core/tests/fixtures/language/math/math_foundation.ph
```

The branch's own handoff records the slice as `IN_PROGRESS / PARTIAL / UNVERIFIED`. That status is accurate.

---

## 4. Findings summary

| ID | Severity | Finding | Ruling |
|---|---|---|---|
| MATH-AUD-01 | **BLOCKER** | Module-qualified exported `const` getter still attempts to call the materialized Float | Must fix before math can pass its own facade fixture |
| MATH-AUD-02 | **HIGH** | `RuntimeExportRef::StaticConstant(Value)` discards canonical static-constant provenance too early | Preserve canonical static identity through runtime materialization/re-export |
| MATH-AUD-03 | **HIGH** | Static constants can be independently materialized for exports and linked reads | Prefer one canonical VM materialization or prove duplication is semantically harmless while retaining identity |
| MATH-AUD-04 | **HIGH** | Package structure puts `FloatMathKernel` in `package.ph`; current `expose .integer` leaks implementation topology | Move implementations to hidden sibling modules; use relative imports; remove `expose` |
| MATH-AUD-05 | **ACCEPTED DEBT** | Family receiver introspection can reveal `FloatMathKernel` | Explicitly accepted by user; do not modify Family semantics in this slice |
| MATH-AUD-06 | **HIGH** | Original universal `≤1 ULP` accuracy guidance is stronger than the approved Rust `f64` backend guarantees for most transcendental methods | Replace with backend-accurate precision contract; `sqrt` is exceptional and correctly rounded |
| MATH-AUD-07 | **HIGH** | Current tests cover only a tiny subset of 23 Float kernels and do not establish family identity, type rejection, IEEE edges, or BigInt laws | Build a full math acceptance matrix |
| MATH-AUD-08 | **MEDIUM** | Runtime constant materialization contains an `expect(...)` panic path inside a fallible boundary | Replace with structured invariant/runtime error |
| MATH-AUD-09 | **MEDIUM** | Regenerated native surface includes unrelated previously-stale `Record` rows | Split or explicitly record generator-baseline synchronization |
| MATH-AUD-10 | **MEDIUM** | `prepare_universe_static_constants` names an operation it does not perform; it attaches lowering metadata | Rename or document precisely |
| MATH-AUD-11 | **MEDIUM** | Public math surface is missing at least arbitrary-base `log` and source-level angle conversion conveniences if this is to be a complete elementary math package | Add deliberately, with Float-only semantics |
| MATH-AUD-12 | **LOW/MEDIUM** | Constants are only tested by decimal rendering | Freeze exact binary64 bit patterns |

---

# Part II — What is correct and should be preserved

## 5. Method-family capture is the correct public abstraction

The branch correctly rejected closure wrappers such as:

```telos
let sqrt = |value: Float| {
  FloatMathKernel.sqrt(value)
}
```

and instead implements:

```telos
let sqrt = &FloatMathKernel.sqrt(_)
let cbrt = &FloatMathKernel.cbrt(_)
let atan2 = &FloatMathKernel.atan2(_, _)
```

This is not stylistic sugar. It preserves Telos's callable ontology:

```text
FloatMathKernel.sqrt(Float)
        │
        │ exact bound callable-family capture
        ▼
math.sqrt : Family
```

That preserves selector identity, callable reflection, live family activation semantics, and future overload isolation. `&T.f(_)` deliberately captures the selected operation shape; the package is not accidentally opting into all future overloads through a whole-family capture.

**Ruling:** retain exact captures. Do not replace them with wrappers to work around access/runtime issues.

---

## 6. Float-only transcendental signatures are correct

The current native declarations accept `Float` explicitly:

```telos
@class native sqrt(_ value: Float) -> Float
@class native exp(_ value: Float) -> Float
@class native sin(_ value: Float) -> Float
```

This matches the accepted numeric architecture:

```text
Int      exact, arbitrary precision
Float    IEEE-754 binary64
Number   shared numeric behavioral superclass
```

Transcendental APIs must not silently turn exact `Int` values into binary64 values:

```telos
math.sqrt(4.0)          // valid
math.sqrt(Float.new(4)) // valid explicit conversion
math.sqrt(4)            // reject
```

The package should not add `sqrt(Number) -> Float` or `sqrt(Int) -> Float` for convenience. Future `Decimal`, `Complex`, `Rational`, or arbitrary-precision floating domains may add deliberate families later.

---

## 7. The native boundary is appropriately narrow

`phalcom-core/src/primitive/math.rs` maps source-owned native declarations onto Rust `f64` methods. That is architecturally preferable to creating VM opcodes or new numeric heap forms.

The package requires no:

- parser syntax;
- new VM instruction;
- new `Value` tag;
- new heap object;
- ad hoc compiler recognition of `math`.

The source declaration remains authoritative and the Rust primitive remains an implementation attachment.

---

## 8. Exact integer algorithms are correctly source-authored

The current `IntegerMath` implementation uses Telos itself for:

```text
gcd
lcm
isqrt
```

This is the correct ownership boundary.

### 8.1 `gcd`

The implementation normalizes signs before Euclid's loop:

```telos
let mut x = if a < 0 { -a } else { a }
let mut y = if b < 0 { -b } else { b }
```

so `%` sign behavior cannot invalidate the algorithm. Required laws are satisfied by design:

```text
gcd(0, 0) = 0
gcd(a, 0) = |a|
gcd(0, b) = |b|
gcd(a, b) >= 0
```

### 8.2 `lcm`

The implementation correctly divides before multiplying:

```telos
(a ~/ gcd(a, b)) * b
```

which limits intermediate BigInt growth compared with `a * b ~/ gcd(a,b)`.

### 8.3 `isqrt`

The Newton iteration remains entirely in arbitrary-precision `Int`, beginning from a bit-length-derived upper estimate:

```telos
1 << ((n.bitLength + 1) ~/ 2)
```

and converging downward. The required postcondition is:

```text
r*r <= n < (r + 1)*(r + 1)
```

The algorithm itself appears sound; what is missing is adequate verification.

---

# Part III — Blocking and high-priority corrective architecture

## 9. BLOCKER: exported `const` getter dispatch is still incorrect

The branch correctly diagnosed the original failure: Telos `const` declarations have no runtime global slot, but module export materialization previously assumed exported values could be represented as slot-backed bindings.

Normative Telos semantics say:

```text
const initializer
    -> semantic static evaluation
    -> StaticValue
    -> MaterializationDescriptor
    -> runtime value/materialization
```

The initializer does not run at runtime and no ordinary global storage cell is required.

The branch added a `RuntimeExportRef::StaticConstant(...)` path and can now materialize a real Float, but one environment-aware module send path still does:

```rust
RuntimeExportRef::StaticConstant(value) => value
```

and then continues into callable dispatch. Therefore:

```telos
import .constants as constants
let x = constants.pi
```

reaches a real Float and then attempts `Float.call`, producing the recorded error:

```text
3.141592653589793 does not understand 'call'
```

### Correct immediate fix

For a Getter-shaped module export send, a static constant is already the result:

```text
module getter
    ↓
RuntimeExportRef::StaticConstant
    ↓
return materialized value
```

It must follow the same early-return rule already implemented in the dynamic export-send path.

This is a mechanical repair, not a reason to redesign module dispatch.

---

## 10. HIGH: retain canonical static-constant identity through runtime projection

The more important issue is structural.

Current code introduces:

```rust
RuntimeExportRef::StaticConstant(Value)
```

while a linked read stores:

```rust
RuntimeLinkedRead::StaticConstant {
    module: ObjRef,
    value: Value,
}
```

The semantic layer already has the stable identity:

```rust
StaticConstantId(SymbolId)
```

where `SymbolId` is canonical `(module, name)` identity.

A re-export does not create a new semantic constant merely because it has a new public alias. Therefore the runtime projection must not lose:

```text
canonical owner
canonical StaticConstantId / SymbolId provenance
```

and later reconstruct it from:

```text
current facade module + public export alias
```

That reconstruction is wrong for:

```text
source.const x
      ↓ re-export as y
facade.y
      ↓ selective import
consumer.y
```

The canonical declaration remains `source::x`.

### Required invariant

> Every runtime reference to an exported/imported static constant must retain or be able to recover the original `StaticConstantId`, independent of re-export aliases.

### Recommended representation direction

Do not over-constrain the implementation to one concrete Rust layout in this design report, but the runtime should have an explicit canonical static-constant reference, conceptually:

```rust
RuntimeStaticConstantRef
    -> canonical StaticConstantId
    -> owner module
    -> materialized Value
```

Then:

```rust
RuntimeLinkedRead::StaticConstant(RuntimeStaticConstantRef)
RuntimeExportRef::StaticConstant(RuntimeStaticConstantRef)
```

are two projections of the same canonical entity.

If preserving `Copy` on the runtime enums is important, use a VM-local compact handle/index into a static-constant table rather than discarding identity.

---

## 11. HIGH: materialize static constants through one owning path

The branch currently may materialize the same semantic constant separately while building exports and linked reads. For immediate values this is cheap, but heap-backed primitive constants such as `String` and `LargeInt` can allocate repeatedly.

Normative static semantics permit physical representations to vary and do not make heap object identity user-visible. Therefore duplicate materialization is not automatically semantically invalid. It is nevertheless a poor runtime ownership model because it complicates:

- GC rooting;
- reflection provenance;
- re-export consistency;
- allocation behavior;
- debugging and runtime identity tracking.

### Preferred runtime law

> A canonical `StaticConstantId` should normally have one VM-local materialized runtime value, reused by exports and imports.

A VM-local materialization cache/table keyed by `StaticConstantId` is the cleanest direction. If the implementation intentionally permits repeated materialization, it still must preserve canonical identity separately and prove that heap identity cannot leak into semantics.

---

## 12. HIGH: static-constant re-export semantics must preserve the source target

`ModuleExecutionContext` currently creates a `LinkedReadSpec::Binding(SymbolId { module: current_target_module, name: item.name })` for a static export and sets the canonical `symbol` field to `None`.

That throws away the remote source target after a facade/re-export boundary.

The fix should be driven by the linked interface's canonical target, not by reverse-engineering runtime export tables. A selected import must carry the original source `SymbolId`, which then maps to the source `StaticConstantId`.

Acceptance cases must include:

```text
direct selected import
renamed selected import
direct re-export
renamed re-export
two-hop re-export
module-qualified access
reflection Export.value
```

---

## 13. MEDIUM: fail with structured runtime errors, never `expect(...)`

The new materialization path contains an invariant lookup implemented with:

```rust
.expect("static constant owner registered")
```

inside code that otherwise returns `PhResult`.

A malformed or inconsistent module product should become an explicit runtime/internal invariant error. It must not panic the host process.

Replace panic-based assumptions with contextual errors containing the missing canonical module/constant identity.

---

## 14. GC requirement for heap-backed static constants

Because the branch now places materialized values directly into runtime linked-read/export structures, heap-backed values must remain GC-reachable.

Required coverage:

```telos
const text = "persistent"
const huge = 10 ** 100
```

Then:

1. import/export them;
2. force GC;
3. read them again;
4. verify content and provenance remain correct.

The branch has started adding trace edges. That needs a focused proof test before the generic constant fix is declared complete.

---

# Part IV — Package organization redesign

## 15. Current organization is unnecessarily coupled

Current `math/package.ph` contains:

- package documentation;
- `FloatMathKernel` class source declaration;
- six public constants;
- every public family capture;
- integer helper import;
- public exports.

That makes the facade the implementation home and forces future math expansion into one file.

The branch also currently uses:

```telos
expose .integer
from universe.math.integer import IntegerMath
```

which makes the implementation child part of the public module traversal surface.

That is unnecessary.

---

## 16. Telos already supports the correct hidden-child model

The module resolver distinguishes:

- **relative imports inside a project/package**, which resolve package-internally; and
- **external absolute traversal**, which validates `expose` edges.

Therefore `universe.math` can contain registered child modules without publicly exposing them.

The correct facade can use:

```telos
from .float_kernel import FloatMathKernel
from .integer import IntegerMath
```

without:

```telos
expose .float_kernel
expose .integer
```

External code then cannot ordinarily import:

```telos
import universe.math.float_kernel
import universe.math.integer
```

because those paths are not exposed by `universe.math`.

This is exactly the distinction `expose` was designed to encode.

---

## 17. Final package structure

### `phalcom-core/core/universe/src/math/package.ph`

Responsibilities only:

- package documentation;
- import hidden implementation owners;
- define public constants with canonical identity in `universe.math`;
- capture public method families;
- export public API.

It should contain **no native kernel class implementation** and no substantial integer algorithm body.

### `phalcom-core/core/universe/src/math/float_kernel.ph`

Responsibilities:

```telos
native class FloatMathKernel is Object {
  @class native sqrt(_ value: Float) -> Float
  ...
}

export FloatMathKernel
```

The export is needed so the parent package can selectively import it. The child path itself remains unexposed.

### `phalcom-core/core/universe/src/math/integer.ph`

Responsibilities:

```telos
class IntegerMath {
  @class gcd(...)
  @class lcm(...)
  @class isqrt(...)
}

export IntegerMath
```

Again, the child module can export the implementation owner to its parent without exposing the child module path publicly.

### Builtin topology

`phalcom-modules/src/builtin.rs` still needs physical/canonical nodes:

```text
math
math.float_kernel
math.integer
```

and `UniverseSourceProvider::source_text` mappings for both children.

But `math/package.ph` must not publish `expose` edges for them.

---

## 18. Native identity must move with the source declaration

Once `FloatMathKernel` moves from `universe.math` to hidden `universe.math.float_kernel`, its canonical `UniverseKey` source path must move too.

Current:

```text
UniverseKey::FloatMathKernel -> ["math"]
```

Required:

```text
UniverseKey::FloatMathKernel -> ["math", "float_kernel"]
```

All generated/native source-anchor metadata must agree with this new owner. There must be exactly one source-owned declaration identity.

Do not leave the native key anchored at `math` while the source class lives in the child module.

---

## 19. Accepted technical debt: kernel reachable through `Family.receiver`

A Telos `Family` stores its receiver/context. Therefore:

```telos
math.sqrt.receiver
```

can reveal the class object used to create the capture.

The user has explicitly ruled that this consequence is acceptable for now.

The math implementation must therefore **not**:

- weaken Family reflection;
- introduce a special opaque Family just for stdlib math;
- copy methods onto another fake public owner;
- synthesize closure wrappers to hide the receiver;
- alter ordinary access semantics.

Record this as a future callable-capability/privacy design issue. It is not part of this slice's acceptance gate.

---

# Part V — Floating-point backend contract

## 20. The approved Rust `f64` backend changes what math can promise

The branch was explicitly approved to use Rust `f64` methods.

This must change one part of the original guidance: a universal `≤1 ULP` acceptance target is not supportable as a normative guarantee for most Rust transcendental functions.

Current Rust documentation distinguishes two categories:

### Strongly specified example

`f64::sqrt` is documented as the rounded infinite-precision IEEE-754 `squareRoot` result and is guaranteed not to change.

### Unspecified-precision examples

Rust documents functions such as:

```text
exp
exp2
ln
log
log2
log10
sin/cos/tan
asin/acos/atan/atan2
cbrt
hypot
expm1
ln1p
sinh/cosh/tanh
asinh/acosh/atanh
```

with **unspecified/non-deterministic precision**, potentially varying by platform and Rust version.

Relevant official reference:

- <https://doc.rust-lang.org/stable/core/primitive.f64.html>

### Consequence

Telos must not claim:

```text
all transcendental math is <= 1 ULP
all finite results are bit-for-bit cross-platform deterministic
```

while directly delegating to those Rust methods.

---

## 21. Correct numerical contract for this backend

### 21.1 Representation

All `math` floating operations consume and produce Telos `Float`, which is IEEE-754 binary64.

### 21.2 Exactness boundary

No implicit `Int -> Float` conversion at the `math` API boundary.

### 21.3 Precision

For operations whose Rust backend documents exact/correct rounding (currently notably `sqrt`), Telos may inherit that guarantee.

For backend functions documented with unspecified precision, Telos should document:

> The operation uses the runtime's supported binary64 mathematical backend. Finite transcendental results may vary in their final bits across supported platforms or runtime/toolchain versions. The API does not promise bitwise reproducibility for these functions.

### 21.4 Special values

Document explicit behavior only where Telos deliberately guarantees it or the chosen backend contract clearly supplies it. Important examples include:

```text
sqrt(-finite) -> NaN
sqrt(-0.0)    -> -0.0
ln(0.0)       -> -Infinity
ln(negative)  -> NaN
ln1p(-1.0)    -> -Infinity
ln1p(< -1.0)  -> NaN
```

Do not specify NaN payload bits. Telos already treats payload bits as non-surface state.

### 21.5 Future deterministic backend option

If Telos later requires reproducible last-bit transcendental math, that is a backend-policy change requiring a controlled libm/correctly-rounded implementation. It must not be pretended into existence through stricter tests against Rust's current unspecified contract.

---

# Part VI — Final `math` package API design

## 22. Package philosophy

`universe.math` owns **mathematical operations that are not already naturally receiver-owned by `Int` or `Float`**.

It should not become a dumping ground for every numeric operation.

The ownership split should remain:

```text
Number / Int / Float
    arithmetic operators
    comparisons
    rounding/classification
    bit operations
    exact/inexact construction

universe.math
    transcendental functions
    trigonometry
    hyperbolic functions
    mathematically named constants
    exact integer algorithms whose meaning is inherently multi-argument/non-scalar
```

---

## 23. Public constants

Keep the six current constants:

```telos
const pi: Float
const tau: Float
const e: Float
const ln2: Float
const ln10: Float
const sqrt2: Float
```

Canonical binary64 bit requirements:

| Constant | Binary64 bits |
|---|---|
| `pi` | `0x400921fb54442d18` |
| `tau` | `0x401921fb54442d18` |
| `e` | `0x4005bf0a8b145769` |
| `ln2` | `0x3fe62e42fefa39ef` |
| `ln10` | `0x40026bb1bbb55516` |
| `sqrt2` | `0x3ff6a09e667f3bcd` |

Keep these declarations in `math/package.ph` so their canonical static identities are directly:

```text
universe.math::pi
universe.math::tau
...
```

rather than re-export aliases from an implementation child.

Do not compute them at package initialization.

---

## 24. Elementary Float families

Retain the current set:

```telos
sqrt(_ value: Float) -> Float
cbrt(_ value: Float) -> Float

exp(_ value: Float) -> Float
exp2(_ value: Float) -> Float
expm1(_ value: Float) -> Float

ln(_ value: Float) -> Float
ln1p(_ value: Float) -> Float
log2(_ value: Float) -> Float
log10(_ value: Float) -> Float

hypot(_ x: Float, _ y: Float) -> Float
```

### Additional required elementary operation: arbitrary-base logarithm

The package should add:

```telos
log(_ value: Float, base: Float) -> Float
```

backed directly by Rust `f64::log(base)` rather than implementing `ln(value) / ln(base)` in source. Rust explicitly provides a dedicated arbitrary-base operation and notes that `log2`/`log10` remain preferable when those exact bases are known.

The public family should be captured exactly:

```telos
let log = &FloatMathKernel.log(_, base:)
```

using the exact current selector syntax accepted by Telos for the labeled parameter.

---

## 25. Trigonometric families

Required:

```telos
sin(_ value: Float) -> Float
cos(_ value: Float) -> Float
tan(_ value: Float) -> Float

asin(_ value: Float) -> Float
acos(_ value: Float) -> Float
atan(_ value: Float) -> Float
atan2(_ y: Float, _ x: Float) -> Float
```

Angles are radians.

`atan2` argument order is explicitly `(y, x)` and must be documented/tested as such.

---

## 26. Hyperbolic families

Required:

```telos
sinh(_ value: Float) -> Float
cosh(_ value: Float) -> Float
tanh(_ value: Float) -> Float

asinh(_ value: Float) -> Float
acosh(_ value: Float) -> Float
atanh(_ value: Float) -> Float
```

---

## 27. Angle conversion helpers

A complete elementary math package should include explicit radian/degree conversion helpers:

```telos
degrees(_ radians: Float) -> Float
radians(_ degrees: Float) -> Float
```

These are source-expressible and require no new native kernel.

Semantics:

```text
degrees(x) = x * (180.0 / pi)
radians(x) = x * (pi / 180.0)
```

They should remain Float-only for the same explicit exact/inexact reason as the transcendental functions.

These methods may be ordinary source-owned public method families in the package facade if package-level methods are the canonical current source form. If package-level callable declaration is not supported/clean in the live grammar, put a tiny source implementation owner in a hidden child and capture its members. Do not introduce closure-valued exports.

---

## 28. Exact integer families

Required first complete slice:

```telos
gcd(_ a: Int, _ b: Int) -> Int
lcm(_ a: Int, _ b: Int) -> Int
isqrt(_ value: Int) -> Int
```

Semantics:

### `gcd`

```text
gcd(0, 0) = 0
gcd(a, 0) = |a|
gcd(0, b) = |b|
gcd(a, b) = gcd(b, a)
gcd(a, b) >= 0
```

### `lcm`

```text
lcm(a, 0) = 0
lcm(0, b) = 0
lcm(a, b) >= 0
```

For nonzero values:

```text
lcm(a,b) = abs((a ~/ gcd(a,b)) * b)
```

### `isqrt`

For `n >= 0`:

```text
r = isqrt(n)
r*r <= n < (r + 1)*(r + 1)
```

For negative `n`, raise `ArgumentError`.

All remain exact arbitrary-precision operations.

---

## 29. Deliberately deferred integer combinatorics

A "full" math ecosystem will eventually benefit from:

```text
factorial
comb
perm
```

but they should **not** be added to this implementation slice merely to imitate Python's `math` module.

Reason: their source-level iteration can consume unbounded CPU long before the existing integer bit limit rejects a result. A correct implementation needs a considered work/resource policy and efficient algorithms, not just a loop over arbitrary user input.

Design these in a follow-on after the foundational package and numeric resource model are proven.

Primality, modular exponentiation, number-factorization algorithms, and arbitrary-precision transcendental computation are likewise separate numerical-library concerns.

---

# Part VII — Operations that do NOT belong in `math`

## 30. Keep existing scalar ownership

Do not add aliases for operations already owned by `Float`/`Number`:

```text
abs
floor
ceil
truncated
rounded
isNaN
isFinite
isInfinite
pow / **
ordinary + - * / % ~/
```

For example, do not create:

```telos
math.abs(x)
math.floor(x)
math.pow(x, y)
```

when the language already has canonical receiver/operator semantics.

---

## 31. Floating representation utilities belong on `Float`, not `math`

The following may be valuable standard-library additions, but their natural owner is the binary64 scalar protocol:

```text
copySign
signBit
nextAfter / nextUp / nextDown
ulp
frexp-style decomposition
ldexp / scale-by-power-of-two
fused multiply-add
IEEE remainder / truncating remainder distinctions
```

Do not place them into `math` merely because C/Python historically do so.

If the project wants these capabilities, open a separate `Float` protocol extension decision.

---

## 32. Generic ordering utilities do not belong in `math`

Defer/locate elsewhere:

```text
min
max
clamp
```

These are generic ordered-value operations, not inherently floating mathematical functions.

---

## 33. Advanced special functions are a later backend decision

Defer:

```text
erf / erfc
gamma / log-gamma
beta
Bessel functions
statistical distributions
complex functions
```

The current Rust stable `f64` backend does not provide a sufficiently broad stable special-function surface with an agreed precision contract. Adding an external numerical backend is a separate dependency and portability decision.

---

# Part VIII — Proposed facade source

## 34. Target `math/package.ph`

The exact syntax must be verified against the current parser/interface builder, but the intended architecture is:

```telos
@!documentation(
  "Elementary binary64 mathematics and exact integer algorithms. " +
  "Float functions do not implicitly convert Int values."
)

from .float_kernel import FloatMathKernel
from .integer import IntegerMath

const pi: Float = 3.141592653589793
const tau: Float = 6.283185307179586
const e: Float = 2.718281828459045
const ln2: Float = 0.6931471805599453
const ln10: Float = 2.302585092994046
const sqrt2: Float = 1.4142135623730951

let sqrt = &FloatMathKernel.sqrt(_)
let cbrt = &FloatMathKernel.cbrt(_)

let exp = &FloatMathKernel.exp(_)
let exp2 = &FloatMathKernel.exp2(_)
let expm1 = &FloatMathKernel.expm1(_)

let ln = &FloatMathKernel.ln(_)
let ln1p = &FloatMathKernel.ln1p(_)
let log = &FloatMathKernel.log(_, base:)
let log2 = &FloatMathKernel.log2(_)
let log10 = &FloatMathKernel.log10(_)

let sin = &FloatMathKernel.sin(_)
let cos = &FloatMathKernel.cos(_)
let tan = &FloatMathKernel.tan(_)
let asin = &FloatMathKernel.asin(_)
let acos = &FloatMathKernel.acos(_)
let atan = &FloatMathKernel.atan(_)
let atan2 = &FloatMathKernel.atan2(_, _)

let sinh = &FloatMathKernel.sinh(_)
let cosh = &FloatMathKernel.cosh(_)
let tanh = &FloatMathKernel.tanh(_)
let asinh = &FloatMathKernel.asinh(_)
let acosh = &FloatMathKernel.acosh(_)
let atanh = &FloatMathKernel.atanh(_)

let hypot = &FloatMathKernel.hypot(_, _)

let gcd = &IntegerMath.gcd(_, _)
let lcm = &IntegerMath.lcm(_, _)
let isqrt = &IntegerMath.isqrt(_)

// Either direct public source families if supported cleanly:
degrees(_ radians: Float) -> Float {
  radians * (180.0 / pi)
}

radians(_ degrees: Float) -> Float {
  degrees * (pi / 180.0)
}

export (
  pi, tau, e, ln2, ln10, sqrt2,

  sqrt, cbrt,
  exp, exp2, expm1,
  ln, ln1p, log, log2, log10,
  sin, cos, tan, asin, acos, atan, atan2,
  sinh, cosh, tanh, asinh, acosh, atanh,
  hypot,

  degrees, radians,

  gcd, lcm, isqrt
)
```

**Important:** do not add `expose .float_kernel` or `expose .integer`.

If package-level source methods are not valid/current for `degrees`/`radians`, use a hidden implementation owner and exact Family captures rather than closures.

---

# Part IX — Native kernel design

## 35. Target `float_kernel.ph`

```telos
/// Binary64 mathematical backend for the public universe.math facade.
/// The class itself is an implementation owner; users consume captured Families.
native class FloatMathKernel is Object {
  @class native sqrt(_ value: Float) -> Float
  @class native cbrt(_ value: Float) -> Float

  @class native exp(_ value: Float) -> Float
  @class native exp2(_ value: Float) -> Float
  @class native expm1(_ value: Float) -> Float

  @class native ln(_ value: Float) -> Float
  @class native ln1p(_ value: Float) -> Float
  @class native log(_ value: Float, base: Float) -> Float
  @class native log2(_ value: Float) -> Float
  @class native log10(_ value: Float) -> Float

  @class native sin(_ value: Float) -> Float
  @class native cos(_ value: Float) -> Float
  @class native tan(_ value: Float) -> Float

  @class native asin(_ value: Float) -> Float
  @class native acos(_ value: Float) -> Float
  @class native atan(_ value: Float) -> Float
  @class native atan2(_ y: Float, _ x: Float) -> Float

  @class native sinh(_ value: Float) -> Float
  @class native cosh(_ value: Float) -> Float
  @class native tanh(_ value: Float) -> Float

  @class native asinh(_ value: Float) -> Float
  @class native acosh(_ value: Float) -> Float
  @class native atanh(_ value: Float) -> Float

  @class native hypot(_ x: Float, _ y: Float) -> Float
}

export FloatMathKernel
```

The child export is an internal package dependency mechanism because the child path itself is not exposed.

---

## 36. Rust native implementation

Keep `phalcom-core/src/primitive/math.rs` as the native implementation module.

The current repeated primitive functions are acceptable because `phalcom-native-surface-gen` scans concrete `syn::Item::Fn` declarations and does not expand `macro_rules!` invocations carrying the primitive attribute.

A local helper for argument extraction/unary application is appropriate; generated primitive declarations themselves must remain explicit unless the generator architecture is separately upgraded.

Add the arbitrary-base log kernel using:

```rust
value.log(base)
```

No VM state mutation, allocation, or numeric coercion should occur inside Float kernels.

---

# Part X — Error semantics

## 37. Float domain errors follow binary64 result semantics

Do not introduce a general `MathDomainError` for Float functions.

Examples:

```text
sqrt(-1.0) -> NaN
ln(-1.0)   -> NaN
ln(0.0)    -> -Infinity
acos(2.0)  -> NaN
```

This is consistent with accepted Telos Float semantics.

---

## 38. Exact integer validation uses argument errors

`isqrt(-1)` is not a Float-domain NaN case; the function promises an exact `Int` result over a nonnegative integer domain.

Use the existing argument-validation error surface:

```telos
ArgumentError.new("isqrt requires a non-negative Int").raise()
```

Do not create a broad `MathError` hierarchy in this slice.

---

# Part XI — Testing and acceptance design

## 39. Package topology tests

Prove all of the following:

```text
universe.math imports successfully
universe.math.float_kernel is not externally traversable
universe.math.integer is not externally traversable
math.sqrt is exported
math.gcd is exported
FloatMathKernel is not directly exported by universe.math
IntegerMath is not directly exported by universe.math
```

Also verify the accepted debt:

```text
math.sqrt.receiver may reveal FloatMathKernel
```

This is not a failure in the current slice.

---

## 40. Family identity tests

For each representative shape:

```telos
math.sqrt
math.atan2
math.gcd
```

verify that the value is a `Family`, not a closure, and that the selected selector shape is exact.

Test at least:

```text
sqrt(_)      one positional Method selector
atan2(_,_)   two positional Method selector
gcd(_,_)     two positional Method selector
```

Future internal overloads must not automatically expand these public captures.

---

## 41. Type boundary tests

Positive:

```telos
sqrt(4.0)
ln(1.0)
gcd(12, 8)
isqrt(81)
```

Negative:

```telos
sqrt(4)          // no implicit Int -> Float
ln(1)            // no implicit Int -> Float
gcd(12.0, 8.0)  // requires Int
isqrt(81.0)      // requires Int
```

The failures should occur at semantic checking, not as late runtime type errors.

---

## 42. Constant tests

Assert exact binary64 bits for all six constants.

Also test both access forms:

```telos
from universe.math import pi
import universe.math as math

pi
math.pi
```

and reflection:

```text
Export.value
module export table inspection
```

No `pi` runtime global slot should be required merely because `pi` is exported.

---

## 43. Static-constant module regression matrix

Before math is considered stable, the general const repair must pass:

```text
selected import
qualified module getter
selected import alias
single re-export
authorized renamed re-export
two-hop re-export
user package -> user package
Universe -> user package
reflection Export.value
Module.dependencies canonical owner
GC survival for String/BigInt static constants
unsupported/deferred structural materialization fails closed
```

---

## 44. Float kernel binding tests

For every native function, integration tests should compare the Telos result against the **same Rust backend operation on the same platform**. This verifies native binding, argument order, and result transport without pretending to establish a stronger accuracy guarantee than the backend offers.

Examples:

```text
sqrt         vs f64::sqrt
cbrt         vs f64::cbrt
exp          vs f64::exp
ln1p         vs f64::ln_1p
atan2(y,x)   vs y.atan2(x)
hypot(x,y)   vs x.hypot(y)
log(x,base)  vs x.log(base)
```

For NaN, compare classification rather than NaN payload bits.

---

## 45. IEEE/special-value matrix

Each relevant function needs explicit tests for:

```text
+0.0
-0.0
+Infinity
-Infinity
NaN
representative subnormals
function domain boundaries
```

At minimum:

### `sqrt`

```text
-1
-0
+0
1
+Infinity
NaN
```

### logarithms

```text
negative
-0
+0
1
+Infinity
NaN
```

### inverse trig

```text
-1
+1
just outside [-1,1]
±Infinity
NaN
```

### `atan2`

Test signed-zero quadrant combinations and infinities against the selected backend contract.

---

## 46. Numerical-accuracy testing policy

Do **not** encode a universal one-ULP release gate for unspecified-precision Rust functions.

Use three tiers:

### Tier A — exact specified operation

For `sqrt`, assert the backend/Telos exact bit result for broad input coverage.

### Tier B — backend conformance

For unspecified-precision Rust methods, assert Telos exactly transports the same-platform Rust result.

### Tier C — high-precision numerical sanity

Optionally run a reference-oracle suite to detect catastrophic regressions, with function-specific tolerances that are **test engineering thresholds, not public API promises**.

This distinction is essential.

---

## 47. Exact integer property tests

### `gcd`

```text
g = gcd(a,b)
g >= 0
if g != 0: a % g == 0 and b % g == 0
gcd(a,b) == gcd(b,a)
```

Include:

```text
0/0
zero/nonzero
negative inputs
coprime inputs
inline Int boundaries
LargeInt values
very large powers/products
```

### `lcm`

```text
lcm(a,0) == 0
lcm(0,b) == 0
lcm(a,b) >= 0
```

For nonzero values:

```text
gcd(a,b) * lcm(a,b) == abs(a*b)
```

### `isqrt`

For many `n >= 0`:

```text
r = isqrt(n)
r*r <= n
n < (r+1)*(r+1)
```

Boundary families:

```text
k*k - 1
k*k
k*k + 1
```

for tiny, inline-boundary, LargeInt, and very large `k`.

Negative input must produce the intended `ArgumentError`.

---

## 48. Numeric resource policy

Math must not bypass existing VM numeric limits.

The exact integer algorithms should naturally route through current `Int` arithmetic, which already enforces bit-length limits for expanding results. Verify that:

```text
lcm with huge operands
isqrt of huge values
gcd of huge values
```

cannot trigger host panic/OOM through a library-specific backdoor.

Do not swallow `#numericLimit` and do not silently truncate.

---

# Part XII — Generated native-surface hygiene

## 49. Current generated file contains unrelated synchronization

Regenerating `phalcom-native-surface/src/generated.rs` on this branch increased the declaration count from `354` to `382`.

Math contributes 23 native declarations, while the remaining delta includes already-authored `Record` primitives that existed on `main` but were missing from the generated artifact.

That does not necessarily mean the generated output is wrong; it means the baseline was stale.

Before merge, choose one of two clean histories:

1. synchronize the stale generated native surface in a separate preparatory commit, then add math rows; or
2. retain the combined generated update but explicitly document that the extra non-math rows are generator-baseline repair.

Do not leave an unexplained cross-subsystem generated diff in a math feature commit.

---

# Part XIII — Documentation requirements

## 50. User-visible package contract

Package documentation should state clearly:

```text
- Float mathematical functions operate on IEEE-754 binary64 values.
- Integer arguments are not implicitly converted to Float.
- Trigonometric arguments/results are radians.
- degrees/radians provide explicit angle conversion.
- Most transcendental finite results follow the supported runtime backend and are not promised bit-for-bit across platforms/toolchain versions.
- sqrt inherits the stronger correctly-rounded backend guarantee.
- gcd/lcm/isqrt are exact arbitrary-precision integer operations.
- Float domain-invalid inputs generally produce NaN/infinity rather than MathDomainError.
```

Do not describe transcendental Float operations as mathematically exact.

---

# Part XIV — Final architecture invariants

## 51. Required invariants

The final `math` implementation should satisfy all of the following.

### API and identity

```text
MATH-INV-01  universe.math is the sole public package namespace.
MATH-INV-02  Public Float functions are exact Family captures, not closures.
MATH-INV-03  Public captures select explicit selector shapes, not whole overload families.
MATH-INV-04  No public Math class is introduced.
MATH-INV-05  FloatMathKernel is source-owned in hidden universe.math.float_kernel.
MATH-INV-06  IntegerMath is source-owned in hidden universe.math.integer.
MATH-INV-07  Hidden implementation children are not exposed by math/package.ph.
MATH-INV-08  Family.receiver kernel reachability is accepted technical debt in this slice.
```

### Numeric semantics

```text
MATH-INV-09  Float transcendental functions accept Float, not Number.
MATH-INV-10  No implicit Int -> Float conversion occurs at the math API boundary.
MATH-INV-11  gcd/lcm/isqrt operate exactly on arbitrary-precision Int.
MATH-INV-12  isqrt(n) satisfies r² <= n < (r+1)².
MATH-INV-13  Existing Float/Number scalar operations are not redundantly re-exported.
MATH-INV-14  Float domain-invalid results follow binary64/backend semantics rather than a broad MathDomainError.
```

### Static constants

```text
MATH-INV-15  math constants remain const, not runtime let workarounds.
MATH-INV-16  const has no synthetic runtime global slot requirement.
MATH-INV-17  StaticConstantId provenance survives imports and re-exports.
MATH-INV-18  Module-qualified const getter returns the value directly.
MATH-INV-19  Heap-backed constant materializations remain GC rooted.
MATH-INV-20  Unsupported materialization fails closed rather than yielding None/uninitialized slots.
```

### Native/runtime

```text
MATH-INV-21  No math-specific VM opcode/value tag/heap arm is added.
MATH-INV-22  Native kernel owner metadata matches the source declaration module.
MATH-INV-23  Rust f64 backend precision is not overspecified by Telos documentation/tests.
MATH-INV-24  Native surface generation is deterministic and its unrelated baseline changes are explained/split.
MATH-INV-25  Fallible materialization paths return structured errors rather than panicking.
```

---

# Part XV — Merge readiness criteria for the math slice

## 52. The math phase is complete only when all of these are true

1. `FloatMathKernel` has moved to a hidden child source module and canonical native ownership has moved with it.
2. `IntegerMath` is hidden behind a relative package import; `.integer` is no longer exposed publicly.
3. `package.ph` is a facade: constants, exact captures, small public helpers, exports—no kernel class implementation.
4. The general exported-static-constant runtime model is corrected rather than worked around with `let`.
5. Selected, qualified, aliased, and re-exported constants retain canonical static identity and return correctly.
6. Heap-backed exported constants survive forced GC.
7. `sqrt(4)` is statically rejected while `sqrt(4.0)` succeeds.
8. Public callable values are verified as `Family` captures.
9. All Float kernel bindings have focused integration tests.
10. Exact constant bit patterns are frozen.
11. IEEE/special-value cases are covered.
12. `gcd`, `lcm`, and `isqrt` have arbitrary-precision property/boundary coverage.
13. Rust backend precision is documented honestly: no blanket one-ULP or bit-reproducibility claim.
14. Generated native-surface drift is resolved/recorded.
15. Focused math and static-constant regressions pass before broader affected suites are run.

---

# Part XVI — Final design verdict

The initial implementation has found the correct **language-level shape** but reached an unfinished **module-runtime seam**.

The right response is not to simplify the math API into runtime `let` values, wrappers, or a monolithic native `Math` object. The right response is to finish the general Telos machinery that the design is correctly exercising:

```text
source-owned native behavior
+ first-class exact Family captures
+ compile-time const materialization
+ canonical module identity
+ source-authored exact algorithms
```

After the corrective work, `universe.math` should present a compact, coherent facade:

```telos
import universe.math as math

math.pi
math.sqrt(2.0)
math.log(8.0, base: 2.0)
math.sin(math.radians(45.0))
math.gcd(84, 30)
math.isqrt(10 ** 100)
```

while implementation details remain organized underneath:

```text
universe.math                public facade
    │
    ├── hidden float_kernel  native binary64 implementation owner
    └── hidden integer       exact Telos implementation owner
```

The accepted `Family.receiver` visibility consequence remains documented technical debt. It should be solved, if desired, by a future general callable-capability/privacy design—not by compromising this package's family semantics.

The next turn should convert this audit/design into a patch-grade implementation plan with exact files, symbols, dependency order, test gates, migration steps, and stop/approval points.
