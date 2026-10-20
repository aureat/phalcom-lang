# PERF003.C2.P7 certification findings

## Current ruling — 2026-10-19

Independent reconciliation accepts **R.G6 and R.G7**. P7 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`; P7 certification is `ACCEPTED / RECERTIFIED`; P8 takeover is unblocked. The exact evidence is summarized in [REPORT.md](REPORT.md) and enumerated in [R.G6-R.G7-CERTIFICATION.md](R.G6-R.G7-CERTIFICATION.md).

## Findings that changed the ruling

1. **The earlier audit bundle was stale.** It named revision `e4503086`, old artifact digests and sizes, old allocation totals, and a runtime `ProductKind` failure that the branch later resolved. Those statements are superseded by the measurement at `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2`.
2. **The original warm sample process was invalid for isolated RSS.** Commit `317992000e358bc54ef0ba2446df3ffdc0be0abd` moved source and artifact scenarios into separate child processes. The regenerated rows show two distinct PIDs and 20 successful repetitions per mode.
3. **The current artifact needs a larger decoded-section limit.** Commit `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2` raised the decoded-section cap to 320 MiB while retaining the 512 MiB total transaction limit. Whole-load N is 373,069,634 bytes and reconciles with independent allocator requests.
4. **Runtime errors remain explicit.** The 20 runtime rows are all `RUNTIME_ERROR`. The harness leaves their error-detail field blank, so no root-cause or runtime-performance claim is made.
5. **Some matrix cases had no direct discriminator.** Commit `c1524361b9606dc537671997fe5069f2aa66035b` added tests for omitted required sections, container digest mismatch, invalid/duplicate module/import identities, duplicate type identities, and duplicate static pack labels.
6. **Workspace integration compilation remains a baseline failure.** Both broad commands stop at 412 stale semantic fixture/API errors. The same error family is recorded at clean predecessor `08e2b2f5`; the focused P7 matrix passes.

## Historical observations

Earlier findings about `ProductKind` bootstrap, source/artifact package failures, and the 15.7 MiB artifact belong to intermediate revisions. The `ProductKind` dispatch issue and SlotMap runtime follow-up were corrected before the measured revision. The prior audit figures are retained in Git history; they are not current certification evidence.
