# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `STDL002`
- Checkpoint: `STDL002.C2`
- Plan: `STDL002.C2.P1`
- Task/Gate: `T6` / `G4` precondition
- Repository: `aureat/phalcom-lang`
- Branch: `main`
- Starting revision: `3766f55bc31ff17c1388cf94366942b0869c13b9`
- Current revision: `3766f55bc31ff17c1388cf94366942b0869c13b9` at incident capture; this report is the only subsequent work unit

## 2. Escalation trigger

Shared-header STOP/CONSULT trigger 16 fired: the accepted plan requires an
evidence-based public `UtcOffset` range, but no accepted constraint currently
selects that range. At least two plausible public choices have materially
different invalid-state and P2-interoperability consequences. The local T6
rule explicitly says not to guess in this situation.

## 3. Required invariant

`UtcOffset` must remain a transparent validated `data` value containing a
fixed signed displacement in seconds. It must preserve second-resolution
historical offsets within the selected range, provide canonical equality,
ordering, hashing, negation, and construction helpers, and must not acquire
named-timezone, DST, or rule-set semantics.

The selected bound must be recorded with its evidence and must not be an
arbitrary convenience restriction. This preserves `P1-INV-20` (`UtcOffset` is
not `TimeZone`) and the T6 coverage contract `P1-CV-33`.

## 4. Expected architecture

T6 is expected to add an ordinary Phalcom `data UtcOffset(seconds: Int)` in
the time standard-library surface. Validation belongs in the general
validated-data construction model and the public factory path, not in a
compiler name check or temporal VM primitive. The bound must be chosen from
an already accepted semantic or backend-interoperability constraint, then
consumed by P2 at its conversion boundary.

P1 must complete T6 and pass G4 before later P1 publication/closure work is
treated as complete. P2 must consume the accepted P1 bound rather than
inventing a second range.

## 5. Observed behavior

The live plan/checkpoint set contains the `UtcOffset` shape and the explicit
VERIFY-FIRST procedure, but no accepted inclusive seconds bound. The P2 plan
still lists the P1 bound as an input and does not establish one. The guidance
mentions backend verification as a future step, not as an approved backend
contract.

Consequently, `UtcOffset` cannot be finalized without either guessing a public
API rule or making an architectural decision locally. T6's `UtcOffset` slice
and G4 remain incomplete; the already implemented non-offset civil layer is
not invalidated.

## 6. Minimal reproduction

Exact repository-state check:

```sh
git status --short
git rev-parse HEAD
git rev-parse origin/main
```

Observed output at incident capture:

```text
3766f55bc31ff17c1388cf94366942b0869c13b9
3766f55bc31ff17c1388cf94366942b0869c13b9
```

The empty status output confirms a clean, synchronized implementation
baseline.

Exact authority search:

```sh
rg -n -i "UtcOffset|offset.*bound|bound.*offset|fixed.?offset.*range|supported.*range" \
  docs/decisions docs/implementation docs/specs
```

Observed result: the matches describe `UtcOffset` as fixed signed seconds,
require second-resolution historical support, identify the bound as
VERIFY-FIRST, and require consultation when no accepted constraint exists;
no accepted inclusive `min_seconds`/`max_seconds` (or equivalent) decision is
present.

Expected result: an accepted public bound and evidence identifying whether it
is semantic, historical-data-driven, or an approved backend-interoperability
constraint.

## 7. Relevant code path

```text
STDL002.C2.P1 T6
  -> UtcOffset transparent data(seconds: Int)
  -> validated public factories and bound rejection
  -> time / time.calendar publication
  -> G4 civil/calendar stabilization
  -> P1 later publication and P2 backend conversion
```

The current source path implements the non-offset civil modules and their
canonical package publication. The `UtcOffset` source and its bound-dependent
tests are intentionally not added while the required decision is unresolved.

Relevant authority files:

- `docs/implementation/STDL002-standard-library-buildout/C2-time-complete/STDL002.C2.P1-plan-00-shared-header.md`
- `docs/implementation/STDL002-standard-library-buildout/C2-time-complete/STDL002.C2.P1-plan-01-body.md`
- `docs/implementation/STDL002-standard-library-buildout/C2-time-complete/STDL002.C2.P2-plan.md`
- `docs/implementation/STDL002-standard-library-buildout/C2-time-complete/STDL002.C2-CHECKPOINT.md`

## 8. Scoped implementation diff

At incident capture there was no source diff: the previous amendment was
already committed and pushed at `3766f55`. This report records the decision
blocker only; it does not add a temporal workaround, choose a bound, or
change the canonical module graph.

## 9. Evidence

Facts observed directly from the repository:

- The P1 body orders T6 before G4 and makes the UtcOffset bound a T6
  precondition.
- The T6 body requires fixed signed-second state and second-resolution
  historical offsets within the selected range.
- The T6 body instructs the implementer to inspect accepted P2/backend
  constraints, record the selected bound and evidence, and consult rather
  than guess when no accepted constraint exists.
- The shared header marks `UtcOffset` as fixed signed seconds and explicitly
  separates it from `TimeZone`.
- The shared header lists the unresolved range choice as STOP/CONSULT trigger
  16.
- The P2 plan consumes an accepted P1 UtcOffset bound; it does not approve
  one.
- The checkpoint records the non-offset T6 civil layer as implemented and
  focused-tested, while retaining the UtcOffset bound as VERIFY-FIRST.
- The implementation baseline is clean and synchronized with `origin/main`.

Focused evidence already established for the completed adjacent work:

- Core Universe module tests: 20/20 passed.
- Direct time and negative-time language corpus tests passed.
- Semantic time capability tests: 2/2 passed.
- Standalone calendar package-publication regression passed.
- Reflection/package exposure census passed.
- Focused `cargo check` for `phalcom-core`, `phalcom-semantic`, and
  `phalcom-modules` passed.

## 10. Implementer's causal hypothesis

The blocker is intentional planning incompleteness, not a discovered
compiler, semantic, module-resolution, or runtime defect. P1 deliberately
left the public range VERIFY-FIRST so that historical second precision and
P2 conversion interoperability would be decided from accepted evidence.

## 11. Alternative hypotheses

- A narrow convenience range, such as a conventional `±18:00`, could be
  adopted. This would simplify validation but would make some otherwise
  representable fixed offsets invalid and could constrain P2 conversions.
- A broader historical-derived range could be adopted. This would preserve
  more fixed offsets but would define a different public invalid-state
  boundary and require explicit backend conversion behavior.
- `UtcOffset` could remain unbounded at the Phalcom `Int` level. That would
  avoid an immediate rejection rule but would contradict the T6 requirement
  for evidence-based bound rejection and defer ambiguity into P2.

These choices are not mechanically equivalent and therefore are not selected
by this report.

## 12. Attempts already made

- Searched the live decisions, implementation plans, checkpoint, and specs
  for an accepted UtcOffset bound. Prediction: an existing decision would
  allow T6 to proceed. Result: no accepted inclusive bound was found.
- Inspected the P1 T6 contract and P2 inputs. Prediction: one plan would
  establish the bound or name an approved backend constraint. Result: P1
  requires the decision and P2 consumes it; neither establishes it.
- Confirmed the completed standalone-universe and non-offset civil work
  remains focused-tested and the tree is clean. Prediction: the blocker is
  isolated to the unresolved range decision. Result: adjacent work remains
  valid and no implementation defect explains the stop.
- Did not add a guessed bound or a temporary backend-specific workaround,
  because the active plan explicitly forbids that response.

## 13. Why local authority is insufficient

The choice changes the public validation contract, the set of constructible
values, second-resolution historical coverage, and the conversion contract
P2 must honor. It also determines whether the bound is a language semantic
rule or merely an approved backend interoperability boundary. Choosing among
those alternatives would alter accepted behavior and cross-plan ownership;
the implementer may not resolve it as a mechanical detail.

## 14. Decision requested

Approve one inclusive `UtcOffset.seconds` bound and identify its authority:

1. Is the bound a language-level semantic range, a historical-data coverage
   range, or an approved backend-interoperability range?
2. What are the exact inclusive minimum and maximum seconds?
3. Which accepted backend/decision evidence must P2 consume, if any?

Until that decision is recorded, keep P1 paused at T6's UtcOffset
VERIFY-FIRST step. After approval, implement the ordinary validated data
surface, add bound/precision/ordering coverage, record the evidence in the P1
walkthrough and handoff, and resume G4.

## Resolution

The advisor approved and resolved this incident with the following authority:

```text
Authority: approved P1/P2 backend-interoperability constraint
Public semantic range: -93_599..=93_599 seconds, inclusive
Equivalent range: -25:59:59..=+25:59:59
Historical-data claim: none
P2 obligation: represent the entire P1 domain exactly, with second precision
```

The symmetric Jiff-compatible envelope makes negation total at both extrema
while retaining the approved POSIX/TZif-oriented interoperability coverage.
It is not a claim about the maximum historically observed civil offset. The
incident is resolved by architectural decision; implementation resumes at T6.
