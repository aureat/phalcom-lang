# STDL002.C2.P1.A1 Walkthrough — Standalone Universe Closure and T6 Resume

This record captures the completed architect amendment for the standalone
Universe dependency-closure blocker, canonical Period identity check, and
focused T6 civil-module resume.

## 1. Result

```text
Amendment: STDL002.C2.P1.A1
Checkpoint: STDL002.C2
Status: COMPLETE
Completion: IMPLEMENTED
Verification: FOCUSED_TESTED
```

The standalone compiler now discovers the complete resolver-backed closure of
accepted canonical Universe dependencies. Standalone entry admission,
entry-only runtime ownership, strict linker behavior, and sibling/project
import rejection remain intact. Existing T6 civil source is published through
the normal Universe graph and is executable through the language-corpus path.
`UtcOffset` remains outside this amendment pending its separate P1
VERIFY-FIRST bound decision.

## 2. Implemented slices

### 2.1 Generic standalone dependency closure

`ProgramAnalyzer` now shares `discover_reachable_dependencies` across project,
package, and standalone discovery. The helper uses the canonical resolver and
a worklist, so a standalone source importing `Timestamp` or the `time` package
receives transitive `Duration`/`Clock` support without widening standalone
authority. `ModuleLinker` was not weakened and no time-name branch was added.

### 2.2 Period identity and source contracts

The direct Period diagnostic was an authored `Period.isZero` return-contract
mismatch, not duplicate declaration identity. The canonical owner,
declaration form, constructor result, and factory returns share one semantic
identity; the body now uses an equivalent typed conditional. The core
identity regression preserves this invariant.

### 2.3 T6 civil publication and verification

The builtin catalog/source provider now publishes `Date`, `DateOverflow`,
`Time`, `DateTime`, `YearMonth`, and `MonthDay`; package sources expose the
existing declarations at calendar and time roots. Live source corrections are
mechanical and local: missing `Weekday` import, unsupported `|>`/`!==`
spellings, typed Boolean conditionals, `DateTime.==`, and the concurrent
`Date.advance` overflow naming are aligned with the current language surface.
Month number validation retains the supported `raise()` form; literal `match`
patterns are intentionally deferred as a separate language enhancement.

The added corpus covers valid civil boundaries, epoch/weekday/leap anchors,
DateTime projections, Period month/day application, overflow constrain/reject,
YearMonth movement, MonthDay, Time bounds, invalid constructors, and the
currently unavailable Period ordering operation.

## 3. Verification ledger

All commands used cleared local compiler flags and ran serially.

| Scope | Result |
|---|---|
| `cargo test -p phalcom-core --test core modules_universe -- --nocapture` | PASS — 20/20 |
| `cargo test -p phalcom-core --test language-corpus corpus::time -- --exact --nocapture` | PASS — 1/1 |
| `cargo test -p phalcom-core --test language-corpus corpus::time_negative -- --exact --nocapture` | PASS — 1/1 |
| `cargo test -p phalcom-semantic --test semantic capabilities::time -- --nocapture` | PASS — 2/2 |
| `cargo test -p phalcom-core --test core modules_universe::standalone_universe_calendar_package_exports_civil_types -- --exact --nocapture` | PASS — 1/1 |
| `cargo test -p phalcom-core --test core reflection_census::physical_universe_corpus_matches_catalog -- --exact --nocapture` | PASS |
| `cargo test -p phalcom-core --test core reflection_census::package_exposures_match_catalog_children -- --exact --nocapture` | PASS |
| `cargo check -p phalcom-core` | PASS |
| `cargo check -p phalcom-semantic` | PASS |
| `cargo check -p phalcom-modules` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `git diff --check` | PASS |

## 4. Deferred and baseline findings

- `UtcOffset` bound selection remains P1 VERIFY-FIRST; no source or guessed
  range was added.
- The focused `phalcom-modules --test builtin_catalog` run had one unrelated
  D-class baseline failure: `bcat_08_root_alias_does_not_create_synthetic_declaration`
  could not resolve `universe.collections.iterator_laws`. Eight of nine tests
  passed; this is deferred to collections/catalog ownership.
- Workspace/release certification was not run. This amendment requires focused
  evidence, not C2 release completion.

## 5. Handoff

Resume the existing `STDL002.C2.P1` body at the remaining T6 contract:
perform the approved `UtcOffset` bound decision when available, then continue
P1's ordinary-package/T7 closure. Do not add a time-specific resolver or
duplicate temporal declaration path.
