---
id: LANG007.C4.algebraic-literals-follow-on-handoff
kind: handoff
checkpoint: LANG007.C4
status: BLOCKED
completion: PARTIAL
verification: UNVERIFIED
created: 2026-10-08
updated: 2026-10-11
---

# Contextual algebraic literals — handoff

## Current state

- Worktree: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/complex-math`
- Branch: `feat/complex-math`
- Remote: `origin/feat/complex-math`
- Starting published revision: `43a4f466`
- Continuation base revision: `fdbc2c41`; this branch delivery includes the
  incident packet and matching lifecycle updates.
- Delivery anchor: `f36f84fa` (`feat: checkpoint algebraic literal and Quaternion foundations`),
  pushed to `origin/feat/complex-math`. Delivery publishes partial implementation.
- This continuation edits only the managed `complex-math` worktree.

## Current stop — 2026-10-11

The bounded semantic stack repair is implemented and focused-tested: shared
realization carriers pass the 760-byte budget, and binary dispatch avoids large
formal/advisory expression-match frames. The complete 64-coefficient Float
Horner analysis passes on 2 MiB with Float type, no diagnostics, and no
contextual realization products.

Do not resume package acceptance until the dedicated minimal-project bootstrap
gate passes. Its current failure is a distinct `AssociatedInvokeNotLoweredYet`
while compiling `Function.attempt`'s `Result::Ok(self.call())`, range 782..805;
the compiler has lowering semantics with an empty associated-site map. The
source and compiler associated-invoke implementation match clean-main
`c2dd635e`; investigate canonical semantic attachment/projection rather than
adding a lowering fallback. See the
[incident report](../../STDL001-numeric-library/stack-overflow-incident-2026-10-11.md), §16.

## Architect ruling — 2026-10-11

Applied the ruling's contextual-candidate gate, iterative arithmetic summary,
explicit generic dispatch side, and conforming-subject `Self` binding. Focused
semantic tests pass for `T.zero`/`T.one`, wrong-side rejection, ordinary `Int`
candidate rejection, arithmetic facts, and the 50,000-node region. The
dependency-free compile-in-process and semantic-baseline probes pass. The
follow-up architect response was received and applied: preserve canonical
realization evidence behind `Arc` and separate ordinary/algebraic binary
frames. The earlier wrapper-relocation failure is historical. Current stack
measurements and the remaining bootstrap error are recorded in incident §16.

## T3 architecture ruling and current implementation

The user chose **Add class-side traits** on 2026-10-10. `@class` behavioral
requirements now retain `DispatchSide::Class` through trait-surface creation,
default-body analysis, conformance witness formation, and semantic selection.
Conformance remains ordinary `impl Trait for Target` on the nominal target;
there is no metatype conformance. Enum class-side trait conformances and
class-side index requirements are excluded.

`docs/specs/objects/traits.md` records the contract. `universe.numbers` now
declares `Zero`, `One`, `RealEmbeddable`, and `BasisRealizable`, with direct
Complex/Quaternion realizations. Data components are prevented from witnessing
class-side getters.

The Rust check for `phalcom-semantic` and `phalcom-core` passed. Changed Rust
files pass `rustfmt --check`, and `git diff --check` is clean. No source-language
acceptance probes or tests were run, so T3 remains `PARTIAL / UNVERIFIED`.

## T4 semantic realization product — partial

Explicit expected types now realize a typed `Basis<I/J/K>` value or a native
imaginary suffix through exact `BasisRealizable<Basis<Axis>>` evidence. Bare
basis units also retain the selected `One` capability for their coefficient.
`ExpressionAnalysis` publishes canonical source/target types, axis, coefficient
source, class-side selection evidence, and the evaluation-order contract;
semantic dependencies and callable-body fingerprints include the product.

Directly evidenced products and assumed generic class-side selections now
project into `ModuleLoweringSemantics`. The compiler lowers them through
selected or active-requirement trait calls and the canonical `Basis<Axis>`
constructor. Direct class receivers use canonical declaration imports;
assumed type-parameter receivers resolve from the active runtime type
environment.

Bounded arithmetic-region selection now uses canonical basis identities and
known scalar types: I-only regions choose Complex, J/K or mixed-axis regions
choose Quaternion, explicit canonical algebra expectations take precedence,
and completed bindings are not rescanned. Known scalar domains remain distinct;
unresolved scalar types do not select a default. Real-valued terms use a
separate checked `RealEmbeddable` operation; basis coefficients use direct
component realization.

Compiler lowering accepts expression coefficients in both multiplication
orders. When the basis is the left operand, lowering stages that value in a
scratch local, evaluates the coefficient next, then supplies the basis while
retaining source evaluation order. Real embeddings project through the selected
`RealEmbeddable.fromReal` class-side requirement.

No Phalcom source-language probes or tests were run. The focused Rust check for
`phalcom-semantic` and `phalcom-core` passed after these changes. T4 and T5
remain `PARTIAL / UNVERIFIED` pending source acceptance, semantic/compiler
coverage, and incremental parity.

`LANG007.C4.P3` is already COMPLETE / IMPLEMENTED / FOCUSED_TESTED for the
separate Pack/call-rest plan. Do not reopen or relabel P3 for this follow-on.
If P3 means Phase 3 of `cliteral.md`, contextual realization is unfinished.
Task 3 of the execution plan is partial as well.

## Completed prerequisites

1. Imaginary suffix lexer/parser support exists and has two focused AST tests.
2. All known exhaustive AST consumers compile.
3. `universe.numbers` exports source-owned `Basis`, `I/J/K`, canonical basis
   bindings, and `Quaternion`.
4. Universe provider registration includes `numbers.algebra` and
   `numbers.quaternion`.
5. Exact Quaternion arithmetic, noncommutativity, conjugation, norm, large Int
   arithmetic, basis aliases, qualified access, and generic identity retention
   probes are present; the focused basis and Quaternion law probes pass.

The capability-unavailability test passed in the aggregate runtime run. The
aggregate result was three passes and two failures, not feature acceptance.

## Stable interfaces to preserve

- `ImaginaryAxis` and `ImaginaryCoefficient` live in `phalcom-ast::token`.
- `Expr::Imaginary { coefficient, axis, range }` is the canonical AST shape.
- `Basis<Axis>` is nominal and axis identity is represented by `I`, `J`, or `K`.
- `universe.numbers` owns `Complex`, `Quaternion`, basis declarations, and
  numeric capabilities.
- Basis bindings are currently runtime immutable `let` values because structural
  data constants are outside the existing static materialization contract.
- Quaternion operation applicability remains capability-gated. Unsupported
  operations use ordinary dispatch failure; no Dynamic arithmetic fallback is
  permitted.
- Explicitly expected imaginary literals now retain a checked realization fact
  in `ExpressionAnalysis`; bounded arithmetic regions can also select a
  canonical target. Directly evidenced products and real scalar embeddings are
  projected and lowered. Expressions without checked products still fail
  closed in the compiler.

## Resolved generic reification incident

Read `algebraic-literals-generic-reification-incident.md` before editing.

The exact issue was generic runtime reification: a checked
`Quaternion<Int>.complexPart` call carries a `T = Int` call-entry recipe, but
the source-owned `Complex<T>` construction inside the generic method reaches
runtime as `Complex<Dynamic>`. Components remain `(1, 2)`, while exact applied
data descriptor identity differs from direct `Complex<Int>` construction.

The owning-layer repair is now present in `phalcom-core/src/modules/compile.rs`:
`close_covering_body_bindings` consumes the published semantic
`EffectiveCallableDefinition.inherent_body_context.covering` relation and adds
both declaration-owned and impl-owned identities to the executable call-entry
recipe. It fails closed on inconsistent values, does not mutate semantic call
specializations, and leaves VM Dynamic fallback and descriptor identity
unchanged.

Do not replace this with separate `Quaternion<Int>` and `Quaternion<Float>`
source methods, name/index matching, or VM-side substitution.

## Next implementation order

1. **Complete:** preserve the exact `Complex<Int>` identity regression and add
   neutral generic product, nested-call, outer-forwarding, independent-binder,
   and declaration-canonical semantic regressions.
2. Complete source-level T3 acceptance for scalar identities and explicit
   basis-realization/real-embedding capabilities. Preserve exact conformance and
   callable identity; class-side source declarations are currently unverified.
3. **Partial:** add semantic acceptance coverage for the new durable product
   and extend direct realization to any remaining basis/suffix cases that need
   a concrete expected type. The product already preserves canonical source
   identities, target type, axis, scalar domain, capability evidence, and
   evaluation order.
4. **Partially implemented:** project directly evidenced products into
   compiler lowering. The compiler consumes the product, invokes the selected
   class-side capabilities, and constructs source-owned basis values; it must
   not infer from `i`/`j`/`k` spelling. Add direct semantic/compiler acceptance
   coverage and runtime tests.
5. **Implemented, source acceptance pending:** assumed generic class-side
   receivers resolve from the current frame's stable type-parameter binding.
   The conformance slot still carries the method and type/conformance
   environments only. Missing or non-nominal bindings fail closed. Next add
   contextual arithmetic-region propagation for `i + j`, `2*i`, and mixed
   axes while preserving ordinary selector identity and source evaluation
   order.
6. Add semantic negative cases: `j` into Complex, scalar-domain mismatch,
   ambiguous generic contexts, and unsupported capability operations.
7. Run the focused runtime probe, semantic tests, compiler projection tests,
   forced-GC tests, and large exact-coefficient tests.
8. Only then promote the follow-on lifecycle record from IN_PROGRESS / PARTIAL.

## Must-read and verification boundaries

- `LANG007.C4-CHECKPOINT.md` — shared follow-on lifecycle and active next action.
- `algebraic-literals-follow-on-plan.md` — tracked task/resume contract.
- `algebraic-literals-follow-on-walkthrough.md` — executed evidence and omissions.
- `algebraic-literals-generic-reification-incident.md` — resolved repair record.
- `algebraic-literals-t3-capability-boundary-incident.md` — resolved class-side
  trait ruling and implementation boundary.
- `cliteral.md`, `docs/specs/complex-numbers.md`,
  `docs/specs/static-evaluation.md`, and `docs/specs/objects/traits.md` — design
  proposal and existing normative boundaries.
- `phalcom-core/tests/core/audit_probes/algebraic_literals.rs` — passing slice
  tests and retained red acceptance reproducers.

The original local plan under `docs/superpowers/plans/` is ignored by Git.
Use the tracked follow-on plan on a fresh checkout. Its P12 filename suggestion
was administrative drift; P12 is already reserved for String–Char work.

All four new AST tests passed in the larger AST run. That run also had a
separately classified Record-pattern fixture failure (D by scoped inspection,
not predecessor-confirmed). Existing rustfmt drift remains outside changed
blocks. No workspace certification or semantic/incremental realization tests
were performed. Run Cargo commands serially and keep their selected-test counts.

## Recommended first commands

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast -p phalcom-semantic -p phalcom-core -p phalcom-lsp --message-format=short
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test integration imaginary_literal -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::algebraic_literals -- --nocapture
```

The last command remains red only for the unfinished typed realization probe;
the generic reification blocker is resolved and its focused regressions pass.
