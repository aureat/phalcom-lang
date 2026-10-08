---
id: STDL001.C5
program: STDL001
kind: checkpoint-record
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# STDL001.C5: complex numbers and complex elementary mathematics

## Objective

Ship a generic, immutable `Complex<T>` with capability-dependent arithmetic,
exact Int operations, and a principal-branch binary64 elementary backend.
The public owners are `universe.numbers` and `universe.math.complex`.

Work is inline in `.worktrees/complex-math`, branch `feat/complex-math`,
starting at `ad1db231`. The user prohibits subagents and requires approval
before significant decisions. After focused completion, the user authorized
commit/push; the implementation is published as `a72d1f79` on `origin/feat/complex-math`.

## Decision ledger

| Decision | State | Contract |
| --- | --- | --- |
| D1 | Approved by user | Generic data family; small operation-specific capabilities; exact Int addition/subtraction/multiplication/conjugation/squared norm; full Float arithmetic and elementary math |
| D2 | Approved by user | Principal branches; signed-zero cut sides; IEEE-style results instead of new arithmetic exceptions; backend-dependent final bits; separate complex facade |
| D3 | Approved by user | Source-authored numerical backend, adapted from CPython's scaled algorithms; precise per-function cuts; log1p included |
| D4 | Approved by user | Extend operator lowering using existing canonical conditional callable/evidence products and InvokeConditional; no Dynamic fallback |
| D5 | Approved by user | Canonical generic data identity across closed metadata and instantiated runtime overlays; general owning-layer correction with exact recursive comparison and stable declaration identity |
| D6 | Approved by user | Preserve gradual missing-message acceptance; verify proof absence plus ordinary runtime unavailability for unsupported operations |
| D7 | Approved by user | Integer ** and Float facade powInt use squaring; separate principal complex pow. Zero exponent is one. Int negative exponents raise ArgumentError; Float negatives use reciprocal. Complex zero base: positive real exponent gives zero, negative real gives +inf+0i, nonzero imaginary/NaN exponent gives NaNs |
| D8 | Approved by user | IntegerPower capability with exact Complex<Int>/Complex<Float> conformance witnesses owns integer **; distinct canonical witness identities preserve the approved API without conditional inherent selector collisions |
| D9 | Approved by user | VM-owned Int exactness compares integer content across independent LargeInt allocations and immediate/heap representations, without Float coercion; retain recursive data exactness and Float bit-sensitive exactness |

The design companion records a correction to D2's shorthand: inverse functions
have their own cuts, and signed-zero boundary values include both -pi and +pi.
The user explicitly confirmed this correction in D3.

## Established boundaries

- Canonical source declarations own types, capabilities, and callable Families.
- Existing semantic products own generic applicability and evidence.
- Complex data uses the existing data representation and exact applied identity.
- Component conversions are explicit. Float real math retains its current domain.
- Main contains ongoing unrelated work; this worktree contains its committed base.
- Rational, probability, and Iterable inference incidents are predecessor context.
  They have no accepted implementation task in this checkpoint.

## Plan ledger

| Plan | Scope | Status | Completion | Verification |
| --- | --- | --- | --- | --- |
| STDL001.C5.P1 | Generic Complex, exact arithmetic, Float kernels, powers and owning integration corrections | COMPLETE | IMPLEMENTED | FOCUSED_TESTED |

## Baseline evidence

Before feature edits:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::math::every_float_family_matches_backend_across_ieee_and_accuracy_tiers -- --exact
```

PASS: one selected test, zero failures. This establishes existing Float Family
transport and IEEE behavior on the worktree's pinned toolchain.

## D4 focused evidence

- `cargo test -p phalcom-core --test core language::inherent_impl::conditional_operator`: two tests passed, covering trait-only component slots, unary/getter conditional dispatch, operand order, and Unsupported reflected fallback.
- `cargo test -p phalcom-semantic --test semantic conditional_component_operators_publish_rigid_trait_requirement_evidence`: one test passed after failing exact rigid-subject/result equality before correction.
- Complex component equality/hash/signed-zero and third-party arithmetic tests passed.
- D5 subsequently resolved the factory/direct exact identity blocker and the
  conditional recipe relocation defect; Int arithmetic now passes exact assertions.

These results establish D4's focused correction, not completed Complex acceptance.

## D5 established interface and evidence

- `typing::identity::equivalent` compares complete metadata/overlay graphs using
  stable declarations and every static argument. Pool-local numbering and
  fingerprints do not decide equality. Iterative graph traversal handles deep
  exact metadata; unions compare normalized member sets.
- `RuntimeDataRegistry::intern_exact_with_component_slots` retains the first
  descriptor and its physical layout as the representative. Incoming raw type
  handles become aliases; already-issued descriptor identity remains stable.
- The existing materializer relocates conditional calls, conditional recipe maps,
  and captured conditional Family recipes to their assigned metadata pool.
- Five `typing::identity::tests` pass, including deliberate fingerprint collisions,
  structural labels/presence/callable shape, union order, depth, and scoped lambdas.
- Three `language::data_e2e::data_canonical_identity` regressions pass: both
  construction orders, nested applications, phantom/row arguments, same-spelled
  module owners/arguments, equality/hash, and post-GC exact sameness.
- Two `language::inherent_impl::conditional_operator` tests pass with added exact
  result identity for operators, getter, ordinary call, and captured Family.
- Three `audit_probes::complex` tests pass, including exact Int arithmetic.
- Six `reflection::spec03` tests pass as adjacent type-equivalence consumers.
- Scoped Rust formatting and `git diff --check` pass.

Commands use `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core`, the
`--lib typing::identity::tests` or named `--test core` filters, and `-- --nocapture`.

## Deferred fixture failures: resolved follow-up

An earlier owning `language::data_e2e` run had 27 passes and five failures.
One failure was the patch's overly strict physical-layout guard, corrected
and covered by the passing direct-first Bool discriminator. Four failures were
classified D (unrelated pre-runtime fixtures):

- `data_record_evaluates_arguments_in_source_order` and
  `data_source_order_with_exception_proves_evaluation_semantics` mutated immutable
  `let` bindings; current semantics require `let mut`.
- `p4_empty_layout_row_specializations_keep_exact_identity_through_generic_callable`
  and `p4_shape_generic_construction_through_generic_callable_keeps_behavior_class_identity`
  used static `const` declarations whose successful lowering projections were absent.

These failures occurred before runtime typing/data interning and did not block D5.
The user authorized the fixture follow-up by continuing the published handoff
at `0076ad54`. Each failure reproduced individually before editing.

The follow-up changes only eight source declarations in `data_e2e.rs`: `n`,
`order` and `threw` use `let mut`; the five typed Factory results use runtime
`let`. The row tests exercise generic runtime construction and descriptor
identity, as confirmed by their assertions and the predecessor's T5 runtime
specialization records. Static evaluation does not execute arbitrary runtime
methods. Every existing assertion is preserved; no production path changes.

All four exact selectors passed after correction. The handoff's acceptance gate
also passed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::data_e2e -- --nocapture --test-threads=1
# PASS: 32 selected tests, zero failures
```

Scoped `rustfmt --check --config skip_children=true` and `git diff --check` pass.
The four-fixture follow-up is COMPLETE / IMPLEMENTED / FOCUSED_TESTED, with no
remaining failure in this gate. `deferred-data-fixtures-walkthrough.md` records
the causes, preserved assertions and verification; its handoff is marked resolved.

## D6 focused evidence

`audit_probes::complex::complex_unsupported_operations_have_no_proven_capability_and_fail_at_runtime`
passes (one selected test). Bool addition and Int division publish no selected
conditional/trait edges and raise the ordinary bilateral `UnsupportedOperation`;
missing compare publishes no capability and raises ordinary doesNotUnderstand.
T1's construction, capabilities, exact arithmetic, and unavailability gate is closed.

## G1 Float arithmetic interface

`numbers/float_complex.ph` supplies source scalar-pair division with separately
scaled products, signed-zero sign helpers, and polar construction. Float-only
Complex members supply division, abs/arg, and classifications; Int supplies explicit
toFloat conversion. Three `complex_float_` tests pass, including prior equality,
large/minimum-subnormal/unbalanced division, zeros/poles/infinity, phase cut sides,
polar roundtrip, and post-GC exact component identity. No native primitive was added.

## T3 nonfinite discriminator

Observed: the expanded signed-cut/nonfinite probe reports two omitted CPython
table cases: tanh(-0+NaN*i) real -0, and acosh(-0+NaN*i) imaginary pi/2.
Hypothesis/evidence: the generic NaN returns shadowed those table-specific zero
exceptions in the source adaptation. Correction: handle those two exact cases
before generic NaNs. Predicted effect: retain the specified known component while
the indeterminate component remains NaN. Discriminator: the exact
`complex_nonfinite_and_signed_cut_cases_follow_the_backend_contract` test.

That discriminator now passes. The separate high-precision scaling probe exposed
5.5e-14 relative error in cosh(710+1.5i): subtracting ln2 from the large exponent
rounded before exp. Moving the half factor into the trig multiplier retained the
exact exponent and corrected the rounding loss. The discriminator
`complex_scaled_elementary_functions_match_high_precision_references` passes.

## G2 powers and exactness

D9's kernel-tier Int exactness test failed on independent LargeInt content
before correction and passes afterward. The unchanged Complex exact-power
discriminator also passes: exact arbitrary-precision results, negative Int
rejection, Float reciprocal powers, captured Families, principal reference,
signed cuts and zero endpoints. A general reentrant conditional-method regression
reproduced the GetUpvalue panic under the old stack-depth test and passes with the
body-kind correction. Methods use their own receiver; nested block literals
continue to capture it. D9 is recorded in `complex-power-exactness-incident.md`.

## G3 focused acceptance

The first expanded Complex run found `log1p(-0-0i)` real +0 instead of -0:
the finite formula adds `x*(2+x)` (-0) to `y*y` (+0), discarding the sign before
ln1p. Returning zero input unchanged corrected the case; the signed-zero matrix
and the complete final Complex group pass.

Final commands and results:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::complex -- --nocapture --test-threads=1
# PASS: 12 selected tests, zero failures
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::math::every_float_family_matches_backend_across_ieee_and_accuracy_tiers -- --exact
# PASS: 1 selected test, zero failures
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test builtin_provider
# PASS: 4 selected tests, zero failures
```

The independent matrix contains 16 common-input references, 17 scaled/branch-point
references and 39 finite cut/small-input references from mpmath 1.4.1 at 100–700
decimal digits. Special-value/sign assertions cover all 16 elementary Families.
Powers additionally cover arbitrary-size Int exponents, exact huge components,
zero endpoints, captured Families and post-GC results.

Scoped Rust formatting passes for all 19 touched/new Rust files with
`rustfmt --check --config skip_children=true`; `git diff --check` passes.
The scoped diff was reviewed. No VM diagnostic instrumentation remains.

## Delivery and next action

`STDL001.C5.P1` is COMPLETE / IMPLEMENTED / FOCUSED_TESTED.
`docs/specs/complex-numbers.md` owns the accepted public numeric contract and is
indexed by `docs/specs/README.md`. The value specification records D9 Int
exactness. Adaptation/license notices remain beside the source kernels.

The walkthrough and handoff record stable interfaces, amendments and exact
verification. Implementation commit `a72d1f79` is published on
`origin/feat/complex-math`; the isolated worktree is retained for review.
No active Complex implementation task remains. The separate four-fixture
follow-up is complete; `deferred-data-fixtures-walkthrough.md` records its closure.
Workspace build/test/clippy, broad crate suites and release certification were
deliberately deferred. The owning `language::data_e2e` gate is now green (32/32).
