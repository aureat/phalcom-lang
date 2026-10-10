---
id: LANG007.C4.algebraic-literals-follow-on-walkthrough
kind: walkthrough
checkpoint: LANG007.C4
status: BLOCKED
completion: PARTIAL
verification: UNVERIFIED
created: 2026-10-08
updated: 2026-10-11
---

# Contextual algebraic literals — walkthrough

## Current stop — 2026-10-11

The source-acceptance continuation is blocked by a shared CLI stack overflow.
A dependency-free project with only an exported integer binding reproduces
the abort, so the current evidence does not attribute it to algebraic literals
or class-side traits. The failing phase remains unknown. See the
[incident report](../../STDL001-numeric-library/stack-overflow-incident-2026-10-11.md).

This record covers the user-directed algebraic-literals follow-on performed in
the `feat/complex-math` worktree. It is separate from `LANG007.C4.P3`, whose
Pack and call/rest work remains complete.

Delivered at `f36f84fa` on `origin/feat/complex-math`.

## 1. Objective and outcome

The intended objective was to implement the proposal in
`docs/implementation/LANG007/LANG007.C4/cliteral.md`:

- native `i`/`j`/`k` suffix literals;
- source-owned `Basis<Axis>` values and canonical axes;
- source-owned `Quaternion<T>` arithmetic;
- semantic contextual realization into `Complex<T>` and `Quaternion<T>`;
- compiler lowering from checked realization facts.

The work stopped at a user-directed checkpoint after syntax integration and a
source-owned basis/Quaternion slice. Contextual semantic realization and native
literal execution remain incomplete.

At the initial stop, T3 lacked realization and scalar-identity capabilities and
the generic `complexPart` identity regression was failing. The latter was
resolved on 2026-10-09; the T3 boundary was decided on 2026-10-10. Phase 3 of
`cliteral.md` (contextual realization) remains incomplete.

## 2. Architecture implemented

- `Token::Imaginary` preserves an integer/float coefficient and an explicit
  `ImaginaryAxis::{I,J,K}`.
- `Expr::Imaginary` is a dedicated AST node. Suffixes are not rewritten into
  identifier references.
- Lowercase contiguous suffixes are recognized atomically. Whitespace keeps
  `2 i` and `2*i` separate; invalid suffixes such as `2iq` produce one numeric
  lexical error.
- `universe.numbers` now exposes source-owned `Basis`, `I`, `J`, `K`, `i`, `j`,
  `k`, and `Quaternion` declarations. `universe.numbers` remains the canonical
  owner of `Complex` and the new algebra types.
- Canonical basis bindings use runtime immutable `let` bindings because the
  current static-materialization contract does not materialize structural data
  constants. No arbitrary runtime method was added to static evaluation.
- `Quaternion<T>` owns capability-gated addition, subtraction, negation,
  conjugation, norm-squared, and Hamilton multiplication.
- Universe module registration and all known AST consumers were updated.

At the initial stop, the semantic checker returned
`Unknown(UncheckedExpression)` for `Expr::Imaginary` and the compiler rejected
it. Temporary direct-construction/denotation prototypes from that attempt were
removed. The current continuation publishes canonical realization evidence
and consumes directly evidenced products in compiler lowering; expressions
without a product still fail closed.

## 3. Files changed

- AST: `phalcom-ast/src/{ast.rs,lexer.rs,parser.rs,token.rs}` and focused
  lexer/parser tests.
- Universe: `numbers/algebra.ph`, `numbers/quaternion.ph`,
  `numbers/package.ph`.
- Universe provider: `phalcom-modules/src/builtin.rs`.
- Exhaustive consumers: compiler attributes/product planning, LSP semantic
  tokens, source index, advisory analyzer, and semantic/compiler expression
  consumers.
- Runtime probes: `phalcom-core/tests/core/audit_probes/algebraic_literals.rs`.
- Design proposal: `docs/implementation/LANG007/LANG007.C4/cliteral.md`.

## 4. Verification actually run

Passed:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast -p phalcom-semantic -p phalcom-core -p phalcom-lsp --message-format=short
  passed

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test integration imaginary_literal -- --nocapture
  2 passed

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::algebraic_literals::basis_values_preserve_nominal_axis_identity_through_aliases -- --nocapture
  1 passed

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::algebraic_literals::quaternion_constructor_arithmetic_is_exact_and_order_sensitive -- --nocapture
  1 passed
```

The unsupported-capability probe also passed in the five-test runtime group:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::algebraic_literals -- --nocapture
  3 passed, 2 failed
```

The two failures are active unfinished acceptance: typed realization/literal
execution and generic `complexPart` exact identity. Subsequent exact runs of
the expanded basis and Quaternion-law tests each passed. The group was not
rerun after its failing projection assertion was simplified to remove temporary
type-graph instrumentation; the same component/descriptor identity oracle is
retained.

The full AST integration target was also run. It discovered 306 tests, with 304
passing, one existing parser failure in
`parser::record_pattern_remainder_is_terminal_and_map_patterns_are_explicit`,
and one ignored test. All four new syntax tests passed. The Record-pattern
failure is classified D by scoped source inspection: its fixture contains no
numeric literals and fails in the unchanged Record-key grammar. It was not
confirmed against a clean predecessor and was not modified.

## 5. Known failures at the initial stop

The combined native literal/runtime probe fails semantic checking first on the
typed basis assignments. Imaginary nodes remain unchecked and the compiler
explicitly rejects them without a semantic realization product. A passing
lexer/parser test therefore establishes syntax only.

The typed basis probe reports `BindingInitializerMismatch` for:

```phalcom
let typedComplex: Complex<Float> = i
let typedQuaternion: Quaternion<Float> = j
```

The generic `Quaternion<T>.complexPart -> Complex<T>` probe reaches runtime
with correct components but produces `Complex<Dynamic>` metadata, while direct
`Complex(1, 2)` produces `Complex<Int>`. The exact applied data identity
contract is therefore violated somewhere between checked generic call-entry
evidence and runtime recipe instantiation. The exact loss point is not proven.

The generic identity defect is recorded in
`algebraic-literals-generic-reification-incident.md`; its owning-layer repair is
recorded in the corresponding walkthrough and handoff. The typed realization
failure remains T4 work.

## 6. Deliberate scope exclusions

- No static `const` realization was broadened.
- No runtime algebra registry or source-name-based literal recognition was added.
- No implicit Int-to-Float promotion was added.
- The original slice preserved ordinary selector identity and dispatch. The
  continuation adds side-aware trait requirements using the existing
  `DispatchSide` identity.
- No broad workspace suite, clippy, or release certification was run.

No adviser decision was obtained during the initial slice. The user's later
class-side trait ruling is recorded in the T3 incident and checkpoint. Runtime
immutable basis bindings remain an implementation adaptation under the
approved runtime-first scope; static declaration eligibility/materialization
remains deferred. The tracked `algebraic-literals-follow-on-plan.md` captures
the execution contract without the occupied P12 identifier suggested by the
ignored local plan.

## 7. Delivery hygiene

Scoped `rustfmt --check --config skip_children=true` exposed existing formatting
drift in the lexer, parser, AST tests, and semantic expression checker. Formatting
of newly added blocks was corrected locally; unrelated drift was preserved.
Temporary production/test logging and realization prototypes have been removed.
The supplied proposal's four Markdown hard-break spaces were normalized for
`git diff --check` without changing its design text.

## 8. Initial stop classification

At the initial 2026-10-08 stop this slice was `BLOCKED / PARTIAL /
FOCUSED_TESTED`. The continuation below supersedes that blocker state.

The AST and source-owned Quaternion/basis slice is implemented and focused
tested. The proposal’s acceptance objective was not complete.

## 9. Continuation after class-side trait decision — 2026-10-10

The user chose **Add class-side traits**. The normative trait specification now
defines side-specific behavioral requirements under the existing nominal
conformance model, without adding metatype conformance. Enum class-side trait
conformances and class-side index requirements remain excluded.

Implemented in this continuation:

- Trait method, getter, setter, and property requirements preserve
  `DispatchSide::Class` in their callable and requirement identities.
- Trait default body analysis and conformance witness type formation preserve
  the requirement side. Class-side index requirements are rejected.
- Data components can witness instance-side getters only; enum class-side trait
  conformances are rejected.
- `universe.numbers` now defines `Zero`, `One`, `RealEmbeddable`, and
  `BasisRealizable`. Complex and Quaternion expose direct class-side identity,
  real-embedding, and I/J/K component-realization implementations.

Verification for this continuation:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic -p phalcom-core --message-format=short — passed
rustfmt --edition 2024 --check phalcom-semantic/src/traits.rs phalcom-semantic/src/session.rs phalcom-semantic/src/types/annotation.rs — passed
git diff --check — passed
```

No source-language acceptance probes or tests were run. T3 remains
`PARTIAL / UNVERIFIED`; T4 canonical contextual realization and T5 compiler
projection/lowering remain unfinished. Current follow-on state is
`IN_PROGRESS / PARTIAL / UNVERIFIED` as of 2026-10-10; that state is superseded
by the 2026-10-11 stack-overflow stop above.

## 10. Continuation: canonical realization product — 2026-10-10

After the user selected class-side traits, T4 now records a directly expected
basis or suffix realization in the canonical expression analysis product.
`checker/algebraic_realization.rs` resolves the exact `BasisRealizable` and,
for bare basis units, `One` requirement through the existing generic
conformance proof authority. It accepts only a unique class-side requirement,
checks the selected parameter/return types and exact scalar domain, and leaves
missing or unresolved evidence unelaborated. It does not infer an algebra for
an expression without a concrete expected type.

The product stores expression/source/target identities, axis, exact suffix
coefficient or selected scalar identity, evidence-backed callable selection,
and source-order evaluation obligations. `ExpressionAnalysis` publication,
incremental product fingerprints, and callable semantic dependencies include
the realization fact. Compiler projection and runtime lowering were not yet
implemented at this point in the continuation; source-level conformance and
semantic acceptance were also unverified.

Verification performed in this continuation:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic -p phalcom-core --message-format=short — passed
rustfmt --edition 2024 --check --config skip_children=true on the new realization module and touched semantic files other than expression.rs — passed
git diff --check — passed
```

`expression.rs` retains unrelated baseline rustfmt drift; the new call site was
reviewed in the scoped diff. No tests were added or run. The follow-on remains
`IN_PROGRESS / PARTIAL / UNVERIFIED`. At that point, the next implementation
seam was semantic acceptance plus compiler projection/lowering, followed by
bounded arithmetic-region inference.

## 11. Continuation: compiler projection and lowering — 2026-10-10

Directly evidenced realization products now project from canonical semantic
analysis into `ModuleLoweringSemantics`. The projection retains the selected
`BasisRealizable` and `One` trait calls, exact target declarations, basis data
construction metadata, coefficient payload, and evaluation contract. Runtime
trait environment bindings are attached from the selected conformance
evidence. Compiler lowering loads class receivers through canonical imports,
invokes those selected class-side capabilities, preserves a source basis read
once, and materializes suffix basis values through the canonical data
constructor. It does not derive targets from source spellings.

This slice currently handles directly evidenced class receivers. Assumed
generic receivers do not yet have a runtime type-object projection and fail
closed during semantic-to-lowering projection. Arithmetic-region inference,
expression coefficients, and source-language acceptance remain incomplete.

Verification for this slice:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic -p phalcom-core --message-format=short — passed
rustfmt --edition 2024 --config skip_children=true on touched compiler/lowering Rust files — passed
git diff --check — passed
```

No tests were added or run. T5 is `PARTIAL / UNVERIFIED`; the follow-on remains
`IN_PROGRESS / PARTIAL / UNVERIFIED`.

## 12. Continuation: assumed generic class-side receivers — 2026-10-10

The user selected the active runtime type environment as the source of an
assumed generic receiver. Semantic lowering now represents a contextual class
receiver as either an exact declaration or a stable type parameter. An assumed
class-side capability projects its existing generic evidence slot; it does not
create a new conformance search or add the receiver to `RuntimeConformanceSlot`.

The compiler emits `LoadTypeParameterClass` for a generic receiver. The VM
requires the current frame's type environment to contain the stable parameter,
then resolves a nominal class or applied nominal origin. Missing bindings and
non-nominal types fail closed. `InvokeTraitRequirement` continues to use the
existing method and type/conformance environments. The executable semantic
pool owns the stable parameter references, and the bytecode specification now
documents the instruction.

Verification performed:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --message-format=short — passed
rustfmt on the six touched core Rust files — passed
git diff --check — passed
```

No tests were added or run. Generic receiver behavior remains source-unverified;
T5 and the follow-on remain `PARTIAL / UNVERIFIED`. Next work is focused source
acceptance and the bounded arithmetic-region lowering/inference slice.

## 13. Continuation: bounded arithmetic regions and real embedding — 2026-10-10

The semantic layer now selects a bounded Complex/Quaternion target from
canonical basis axes and known scalar types. An explicit canonical algebra
expectation takes precedence. I-only regions select Complex; J/K or mixed axes
select Quaternion. Scalar domains must agree, completed bindings remain
boundaries, and unresolved scalar types do not create a default target.

`ContextualRealization` now records either a basis-component operation or a
separate `RealEmbeddable.fromReal` operation. Both carry canonical target and
selected class-side capability evidence through expression publication,
incremental fingerprints, semantic lowering, runtime roots, and compiler
lowering. Scalar-by-basis expressions work in either operand order; when the
basis is first, lowering stages it so both source operands still execute once
in source order.

The compiler and semantic crates pass the focused `cargo check`. No Phalcom
source compilation or tests were run. Source conformance, semantic acceptance,
runtime behavior, and incremental parity remain unverified. The follow-on
remains `IN_PROGRESS / PARTIAL / UNVERIFIED`.
