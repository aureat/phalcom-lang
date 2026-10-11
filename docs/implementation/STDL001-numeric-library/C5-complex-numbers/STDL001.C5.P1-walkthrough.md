# STDL001.C5.P1 walkthrough

State: COMPLETE / IMPLEMENTED / FOCUSED_TESTED.

## Outcome

The checkpoint implements immutable `Complex<T>`, capability-selected exact
arithmetic, Float division/magnitude/phase/polar conversion, integer powers,
and the principal-branch elementary facade at `universe.math.complex`.
The normative contract is `docs/specs/complex-numbers.md`.

Implementation commit `a72d1f79` is published on `origin/feat/complex-math`, based
on `ad1db231`. The isolated `.worktrees/complex-math` worktree is retained.
Execution was inline; the user prohibited subagents and approved significant
decisions, then authorized commit/push after focused completion.

## Architecture and files

- `numbers/capabilities.ph` declares independent closed operations and Int/Float
  conformances. `numbers/complex.ph` owns the data family and behavior, including
  exact IntegerPower witnesses for Int and Float applications.
- `numbers/float_complex.ph` supplies scalar-pair normalization, scaled division,
  sign handling and polar construction without importing the Complex owner.
- `math/complex/kernel.ph` implements sqrt, exp, log, log1p, trigonometric and
  hyperbolic functions, inverse functions and principal complex powers.
  `package.ph` exports captured Families. Universe source/provider registration
  follows the existing optional-child architecture.
- Compiler operators consume canonical conditional/trait proof products and
  preserve bilateral reflected decline. Rigid assumed-trait Self is specialized
  to the proven subject by the semantic layer.
- Runtime type graph comparison uses complete stable identity and arguments.
  Exact data descriptors alias equivalent metadata/overlay types to their first
  representative and retain issued identity and physical layout.
- Materialization relocates conditional invocation and captured Family recipes.
  Receiver emission distinguishes a method's own receiver from lexical block
  captures even during reentrant detached-method compilation.
- `VM::semantic_same` compares Int content across independent LargeInt allocations
  and immediate/heap realizations. Float exactness stays bit-sensitive and
  Int/Float equality does not create exact sameness.

## Amendments and numerical corrections

D1–D3 fixed generic capability scope, IEEE principal branches and the pinned
CPython source adaptation. D4 authorized canonical operator evidence transport;
D5 authorized exact generic data identity at its runtime owner. D6 clarified the
negative oracle: absent selected capability plus ordinary runtime unavailability.
D7 separated integer and principal complex powers and defined zero endpoints.
D8 selected existing conformance identities for the two IntegerPower witnesses.
D9 authorized the general Int exactness correction exposed by huge exact powers.

Incident documents retain causes and approved decisions. The implementation
adds no Complex runtime tag, native numerical primitive or promotion lattice.

Focused probes corrected two omitted nonfinite inverse/hyperbolic cases,
large-exponent half-factor rounding in sinh/cosh, and zero-sign loss in log1p.
The CPython 3.13.9 adaptation retains the PSF license and commit provenance.
Finite references come from independent mpmath 1.4.1 calculations of exact
binary64 inputs at 100–700 decimal digits.

## Verification

Final acceptance commands executed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::complex -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::math::every_float_family_matches_backend_across_ieee_and_accuracy_tiers -- --exact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test builtin_provider
```

Results: 12 Complex tests, one adjacent Float matrix and four provider tests
passed. The unavailable-operation test intentionally executes and checks ordinary
runtime errors. The Complex group includes 72 independent finite/scaled fixtures,
signed cut sides, branch endpoints, subnormals, infinities, NaNs, zero signs,
third-party capabilities, exact Int arithmetic, powers, Family capture and GC.

Owning regressions executed during the final powers slice:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::numeric_values::integer_exactness_is_independent_of_heap_representation_without_float_coercion -- --exact --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::inherent_impl::reentrant_conditional_method_uses_own_receiver_and_nested_closures_capture_it -- --exact --nocapture
```

Both passed and both were observed failing with the relevant correction absent.
The receiver regression covers direct self reads and nested lexical capture.

Earlier checkpoint evidence, retained in CHECKPOINT.md: five type graph identity
unit tests, three general exact data identity tests, two conditional operator
tests, one rigid trait requirement semantic test and six adjacent reflection
tests passed. Those established interfaces were inherited rather than re-audited.

Scoped format validation used `rustfmt --check --config skip_children=true` on
all 19 touched/new Rust files. It and `git diff --check` passed. The scoped source,
tests and new files were reviewed; temporary VM diagnostics were removed.

## Deferred verification and residuals

Workspace build/test/clippy and release certification were deliberately deferred.
No global correctly-rounded or performance guarantee was measured.

The earlier data subsystem run had 27 passes and five failures. The patch's
layout guard failure was corrected and verified. At P1 closure, four unrelated
pre-runtime fixtures were deferred: two mutated immutable `let` bindings and two
lacked static-const lowering projections. The subsequent fixture-only follow-up
resolved all four and passed the complete `language::data_e2e` gate (32/32).
CHECKPOINT.md and `deferred-data-fixtures-walkthrough.md` record that evidence.

## Takeover

No active Complex task remains in this checkpoint. The completion handoff
identifies established interfaces. The separate four-fixture repair is complete;
`deferred-data-fixtures-handoff.md` is marked resolved.
