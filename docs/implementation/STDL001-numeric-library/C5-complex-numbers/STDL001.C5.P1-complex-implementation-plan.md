---
id: STDL001.C5.P1
program: STDL001
checkpoint: STDL001.C5
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# Complex Numbers Implementation Plan

Execution: inline using the executing-plans workflow. The user prohibits
subagents and requires approval of significant choices. Commit/push are outside
the current authorization.

Goal: generic Complex data and arithmetic, with the approved principal-branch
Float elementary surface.

Architecture: source-owned data and nominal operation capabilities; conditional
inherent members selected by canonical semantic evidence; source-authored scaled
Float algorithms over the existing scalar math kernel. Public math functions
are captured Families. Existing data identity and GC machinery realize values.

Tech stack: Phalcom source, existing Rust Universe embedding and integration
test infrastructure, independent Python/mpmath reference generation for fixtures.

## Files and responsibilities

- `core/universe/src/numbers/package.ph`: public type/capability exports.
- `numbers/capabilities.ph`: independent closed arithmetic capabilities and
  scalar conformances; no trait inheritance.
- `numbers/complex.ph`: data owner, generic arithmetic, conditional Float
  arithmetic, identities, construction, explicit real-component operations.
- `numbers/float_complex.ph`: scaled component-pair arithmetic helpers. It
  takes/returns scalar components and imports no Complex type, avoiding cycles.
- `math/complex/package.ph`: public captured Families.
- `math/complex/kernel.ph`: elementary numerical kernels consuming Complex.
- `math/complex/CPYTHON-LICENSE.txt` and `README.md`: adaptation notices and
  numerical implementation/reference contract.
- Universe root, math facade, `phalcom-modules/src/builtin.rs`: canonical graph
  registration and embedded source, following existing optional-child patterns.
- `tests/core/audit_probes/complex.rs`: runtime/numerical regressions.
- `tests/reference/math_complex_reference.rs`: independent finite references.
- `docs/specs/complex-numbers.md`: normative approved public semantics.

Core source/test paths above are under `phalcom-core/`.

## Fixed invariants

Use the approved D1-D3 design. Exact applied identity and trait evidence stay
canonical. No Complex runtime tag, Dynamic arithmetic fallback, type-based
overload dispatch, global promotion, compiler workaround, or native floor
addition. Int division is unavailable; Int negative powers cannot manufacture
Float components. Float zero division and poles return defined IEEE components.
Components preserve signed zero; NaN payload/sign bits are not API.

Mechanically flexible: private decomposition, trait/helper names, imports,
test plumbing. Verify first: generic operator witness compatibility, covering
impl equality guards, class-side identity witnesses, captured Family evidence,
inherited ordering behavior, mixed-constructor inference.

## Acceptance and coverage

Core: rectangular construction/access; exact arbitrary-precision Int arithmetic;
third-party component capability; signed-zero numeric equality versus exact
sameness; NaN inequality; equal hashes; unavailable unsupported operations;
explicit Float conversion; captured Family and GC preservation.

Float: division with large/subnormal operands; magnitude/phase; polar roundtrip;
integer and complex powers; all elementary functions at independent finite
references; signed cut sides, branch endpoints, zeros, infinities, NaNs, poles;
scaled sqrt/log/exp; near-zero log1p; inverse functions near branch points.

## Tasks and gates

### T1: core capabilities and data

- [x] Add construction/exact arithmetic and unsupported-component regressions.
- [x] Run the exact core regression and confirm missing-library failure.
- [x] Add capability declarations/scalar conformances, generic Complex data,
  conditional arithmetic, and canonical Universe registration.
- [x] Run the core regression. Run one exact conditional-evidence discriminator
  only if runtime evidence remains uncertain.

D4 amendment: operator lowering must consume existing conditional and trait
invocation products, including unary getters and the trait-bound component
operators inside generic arithmetic bodies. Preserve bilateral preference and
Unsupported decline routes. Publish compiled conditional methods by the existing
canonical detached-callable registry for imported calls. Add compiler regressions
for witness-only component operators, reflected fallback, once-only evaluation,
and cross-module conditional calls. No new semantic fact domain is authorized.

D5 amendment (approved by user): canonicalize equivalent closed metadata and
instantiated overlay exact types when interning data descriptors. Runtime typing
owns recursive type comparison, using stable declaration identities and complete
static arguments; the data registry owns descriptor representatives. Preserve
nested applications, phantom arguments, owner modules, structural shapes, and
already-issued descriptor identity. No display-name or hash-only equality.
Add direct/generic construction, nested, phantom, and cross-module discriminators;
run the directly affected data identity suite and Complex tests at this gate.

D6 amendment (approved by user): unsupported operations retain ordinary gradual
missing-message acceptance. Verify no selected conditional/trait capability is
published and execution fails via ordinary terminal message miss. Static rejection
is not part of this numeric acceptance oracle.

### T2: Float arithmetic

- [x] Add independent division, phase/polar, equality/hash and sign regressions.
- [x] Implement scaled division/component helpers in source and conditional
  Float members in the data owner. Define pole/zero classifications explicitly.
- [x] Run directly affected exact tests. Record G1 core interface evidence.

### T3: elementary kernels

- [x] Add finite reference and branch/IEEE regressions before kernels.
- [x] Retain pinned CPython adaptation notices. Implement scaled sqrt/log/log1p,
  exp, stable hyperbolic functions, rotation-derived trig functions, and scaled
  inverse functions with documented per-function special-value cases.
- [x] Export captured Families in the optional complex math facade.
- [x] Run the numerical discriminator; no broad BUILD suites.

### T4: powers and composition

D7 amendment (approved by user): integer `**` and `powInt(z, n)` use squaring;
the Float facade's separate `pow(z, w)` uses principal `exp(w*log(z))`. Zero
exponent returns one, including 0^0. Int negative exponent raises ArgumentError.
Float negatives use reciprocal division. For complex exponents, zero base with
positive real exponent returns zero, negative real exponent returns +inf+0i, and
nonzero imaginary or NaN exponent returns NaN components.

D8 amendment (approved by user): integer power is the `IntegerPower` capability,
with exact conformances for Complex<Int> and Complex<Float>. Existing conformance
callable identities distinguish these witnesses; no new inherent specialization
identity or precedence rule is introduced. Principal complex pow stays separate.

D9 amendment (approved by user): correct general Int exactness at
`VM::semantic_same` for equal independently allocated LargeInts and equivalent
immediate/heap Int representations. Preserve Float bit-sensitive exactness and
forbid Int/Float coercion. Add a kernel-tier scalar regression and retain the
Complex exact-power discriminator unchanged.

- [x] Add exact nonnegative and Float negative integer-power regressions,
  complex-exponent references, zero endpoints, and captured Family composition.
- [x] Implement exponentiation by squaring and complex powers under D2.
- [x] Run exact power/composition tests. Record G2 numerical interfaces.

### T5: stabilize and document

- [x] Complete the independent reference matrix and edge cases.
- [x] Run `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::complex`.
- [x] Run the existing Float IEEE/backend matrix as the adjacent regression.
- [x] Run the builtin provider tests because Universe topology changed.
- [x] Scoped Rust format check and `git diff --check`; review the entire scoped
  diff for accidental edits. Update normative spec and its index.

### T6: delivery

- [x] Record completed G3 results and unresolved baseline failures truthfully.
- [x] Update checkpoint and plan completion/verification separately.
- [x] Write walkthrough and handoff with exact commands and outcomes.
- [x] Retain the isolated branch/worktree for the user.

## Execution budget and stops

Run Cargo serially with pinned nightly and cleared local flags. Keep coherent
source slices between tests. Workspace build/test/clippy and release
certification are deferred. An unchanged PASS is acquired evidence.

Allow up to three concrete mechanical corrections. Before one serious
behavioral correction record observed result, causal hypothesis, evidence,
predicted effect, and a discriminating test. If that correction fails, stop.
Stop and ask the user immediately if generic evidence, canonical identity,
native boundary, ownership, representation, accepted semantics, or public
contract needs redesign. Do not fix predecessor probability/Iterable incidents.

## Inline plan review

The reviewed dependency graph keeps component helpers independent of Complex,
so `numbers` does not import its `math.complex` consumer. Numerical tests use
independent values; CPython adapted formulas are not their finite oracle.
Covering guards follow the owning impl specification. Native scalar behavior
and resource policy remain the same. Significant endpoint choices uncovered
during implementation require another approval before that slice proceeds.
