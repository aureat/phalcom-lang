---
id: LANG007.C4.algebraic-literals-follow-on-walkthrough
kind: walkthrough
checkpoint: LANG007.C4
status: BLOCKED
completion: PARTIAL
verification: FOCUSED_TESTED
created: 2026-10-08
---

# Contextual algebraic literals — walkthrough

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

Task 3 of the implementation plan is also partial: explicit realization and
scalar-identity capabilities are absent and the generic `complexPart` regression
is failing. Phase 3 of `cliteral.md` (contextual realization) is not complete.

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

The semantic checker currently returns `Unknown(UncheckedExpression)` for
`Expr::Imaginary`. The compiler has an explicit rejection arm. Temporary
semantic direct-construction/denotation/lowering prototypes were removed; none
of those prototypes is part of this delivery.

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

## 5. Known failures and architectural blocker

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

This is recorded in
`algebraic-literals-generic-reification-incident.md`. It requires an owning-layer
decision before continuing; an Int/Float-specific method workaround would create
a special-case source API and hide the generic reification defect.

## 6. Deliberate scope exclusions

- No static `const` realization was broadened.
- No runtime algebra registry or source-name-based literal recognition was added.
- No implicit Int-to-Float promotion was added.
- No ordinary selector identity or dispatch rule was changed.
- No broad workspace suite, clippy, or release certification was run.

No adviser decision was obtained. The runtime immutable basis bindings are an
implementation adaptation under the approved runtime-first scope; static
declaration eligibility/materialization remains deferred. The tracked
`algebraic-literals-follow-on-plan.md` captures the original execution contract
without the occupied P12 identifier suggested by the ignored local plan.

## 7. Delivery hygiene

Scoped `rustfmt --check --config skip_children=true` exposed existing formatting
drift in the lexer, parser, AST tests, and semantic expression checker. Formatting
of newly added blocks was corrected locally; unrelated drift was preserved.
Temporary production/test logging and realization prototypes have been removed.
The supplied proposal's four Markdown hard-break spaces were normalized for
`git diff --check` without changing its design text.

## 8. Final classification

`BLOCKED / PARTIAL / FOCUSED_TESTED`.

The AST and source-owned Quaternion/basis slice is implemented and focused
tested. The proposal’s acceptance objective is not complete.
