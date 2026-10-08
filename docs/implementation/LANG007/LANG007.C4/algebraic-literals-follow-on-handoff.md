---
id: LANG007.C4.algebraic-literals-follow-on-handoff
kind: handoff
checkpoint: LANG007.C4
status: BLOCKED
completion: PARTIAL
verification: FOCUSED_TESTED
created: 2026-10-08
---

# Contextual algebraic literals — handoff

## Current state

- Worktree: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/complex-math`
- Branch: `feat/complex-math`
- Remote: `origin/feat/complex-math`
- Starting published revision: `43a4f466`
- Delivery anchor: the commit containing this handoff, whose subject is
  `feat: checkpoint algebraic literal and Quaternion foundations`.
- Find that commit with `git log --oneline -5`; the commit hash is also reported
  inline after the requested push. Delivery publishes partial implementation.
- Main worktree is clean and must remain untouched.

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
- The checker returns `Unknown(UncheckedExpression)` for imaginary literals.
  Compiler lowering explicitly rejects the node. No checked realization fact,
  compiler realization descriptor, or temporary direct-construction prototype
  remains in the delivered tree.

## Blocking incident

Read `algebraic-literals-generic-reification-incident.md` before editing.

The exact issue is generic runtime reification: a checked
`Quaternion<Int>.complexPart` call carries a `T = Int` call-entry recipe, but
the source-owned `Complex<T>` construction inside the generic method reaches
runtime as `Complex<Dynamic>`. Components remain `(1, 2)`, while exact applied
data descriptor identity differs from direct `Complex<Int>` construction.

Do not fix this by adding separate `Quaternion<Int>` and `Quaternion<Float>`
source methods. That would introduce a workaround and leave the generic
call-entry/type-recipe owner inconsistent.

## Next implementation order

1. Obtain an architectural ruling for generic source-body type reification:
   determine how covering-impl/body-coordinate substitution and runtime call
   environment recipes must propagate through nested generic data construction.
2. Add a semantic regression for exact `Complex<Int>` identity through the
   generic `complexPart` body before changing production code.
3. Correct the owning generic reification/materialization seam and rerun the
   exact probe.
4. Complete T3's scalar identity and explicit basis-realization/real-embedding
   source capabilities. Preserve exact conformance and callable identity.
5. Add a durable semantic realization product for basis expressions and native
   suffix literals. It must preserve canonical source identities, target type,
   axis, scalar domain, and realization evidence.
6. Project that product into compiler lowering. The compiler must consume the
   product and construct ordinary data values; it must not infer from `i`/`j`/`k`
   spelling.
7. Add contextual arithmetic-region propagation for `i + j`, `2*i`, and mixed
   axes. Preserve ordinary selector identity and source evaluation order.
8. Add semantic negative cases: `j` into Complex, scalar-domain mismatch,
   ambiguous generic contexts, and unsupported capability operations.
9. Run the focused runtime probe, semantic tests, compiler projection tests,
   forced-GC tests, and large exact-coefficient tests.
10. Only then promote the follow-on lifecycle record from BLOCKED / PARTIAL.

## Must-read and verification boundaries

- `LANG007.C4-CHECKPOINT.md` — shared follow-on lifecycle and active next action.
- `algebraic-literals-follow-on-plan.md` — tracked task/resume contract.
- `algebraic-literals-follow-on-walkthrough.md` — executed evidence and omissions.
- `algebraic-literals-generic-reification-incident.md` — pending decision.
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

The last command is expected to remain red until semantic realization and the
generic reification blocker are resolved.
