# STDL002.C2.P1.A1 Handoff — Resume P1 T6/T7

## Current state

`STDL002.C2.P1.A1` is complete as `IMPLEMENTED / FOCUSED_TESTED`. The
standalone Universe dependency blocker is removed. The canonical resolver
worklist discovers transitive Universe dependencies for standalone entries,
while standalone authority remains restricted to the entry and accepted
Universe roots. Strict linking remains strict.

The direct Period identity seam is closed by evidence: canonical owner,
declaration form, constructor result, and factory return types agree. The only
Period source correction was an equivalent typed conditional in `isZero`.

The existing T6 civil source is registered and published through the canonical
builtin graph/provider. Direct and package imports of `Date`, `DateOverflow`,
`Time`, `DateTime`, `YearMonth`, and `MonthDay` compile and execute through
normal module/compiler semantics. Positive and negative civil corpus coverage
is green.

## Invariants to preserve

- Do not add a time-specific module-resolution/linker branch.
- Do not weaken `ModuleLinker` strictness.
- Keep Universe support modules semantic-only in standalone runtime ownership.
- Keep temporal source declarations canonical; do not add wrappers or duplicate
  data roots.
- Keep enum cases free of `@variant` annotations in authored enums.
- Use `raise()` for Phalcom error propagation and the supported inheritance
  test syntax (`value is not Type`) where the live parser/evaluator supports it.
- Do not implement literal `match` patterns in this lane; that is a separate
  language enhancement.
- Preserve the approved `UtcOffset` numeric domain: inclusive
  `-93_599..=93_599` signed seconds. P2 may not narrow it or lose second
  precision.

## Evidence already available

See:

```text
STDL002.C2-CHECKPOINT.md
STDL002.C2.P1.A1-blocker-remediation-walkthrough.md
```

The focused module aggregate is 21/21 green. The positive and negative time
corpus filters, semantic time capability suite, package publication regression,
reflection census checks, and crate checks are green. Formatting and diff
checks are green.

The T7 continuation is also complete: grouped `time`/`calendar` publication,
the exact `Duration`/`Timestamp` temporal prelude, ordinary-project runtime
ownership, canonical prelude identity, and explicit negative visibility
coverage are recorded in checkpoint §27. The ordinary `time_p1` fixture passes
through the normal project linker and full VM.

## Deferred issue

`phalcom-modules --test builtin_catalog` still has one unrelated D-class
failure for the collections `iterator_laws` import. Do not repair it as part
of temporal work.

## Approved UtcOffset continuation

The advisor resolved the incident recorded in
`STDL002.C2.P1-T6-utc-offset-bound-incident.md`:

```text
authority = approved P1/P2 backend-interoperability constraint
min_seconds = -93_599
max_seconds =  93_599
inclusive = true
historical-maximum claim = false
```

`UtcOffset` is now a transparent validated data value in
`phalcom-core/core/universe/src/time/offset.ph`, published through the
canonical `time.offset` module and `universe.time` package. Its factory and
boundary tests preserve exact signed-second semantics, total negation,
canonical ordering/equality/hash behavior, and fixed-offset-only meaning.

## Next action

Continue `STDL002.C2.P1` at T8 stabilization and final P1
walkthrough/handoff preparation. P2 must wait for the completed P1
walkthrough/handoff and P1 completion conditions.
