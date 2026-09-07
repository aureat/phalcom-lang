# AUDIT — L05: Raw-run reuse and capture lifetime

## Status and scope

**Unresolved; retained source-level concern, not a reproduced failure.**

This document records the next bounded investigation in the integrated runtime audit. It was requested after the user reported another cybersecurity warning preventing continuation. The warning's exact trigger and internal decision are unknown. No new runtime probe, source verification, fix or commit was performed while preparing this document. This record does not reclassify the work or bypass an approval restriction.

Scope: VM execution reset, escaped closures, open captures and error cleanup. Frontend work is unnecessary unless needed to establish the validity of a specific runtime example. Preserve unrelated checkout changes and existing runtime architecture.

## Retained evidence

The existing [grouped findings, section 4](AUD-RUNTIME-005-grouped-contract-and-performance-findings.md#4-public-fresh-run-reset-bypasses-close-before-truncate-discipline) and [L05 continuation lead](AUD-RUNTIME-S2-uninvestigated-leads-and-insights.md#l05--raw-execution-reuse-after-an-error) record:

- `interpret.rs::run_in_module` clears frames and stack directly.
- `vm/api.rs::unwind_cell` uses disciplined unwinding; `run_cell` uses the cell cleanup path.
- `vm/dispatch.rs::unwind_to` closes captures before truncating storage.
- `close_upvalues_from` assumes open capture indices still address live stack storage.
- `run_until` preserves uncaught frames for diagnostics. Preserving those frames is intentional and is not itself the suspected defect.

These are retained observations, not a fresh verification against the latest checkout. No failed-run/reuse sequence has been executed for L05. No wrong closure result, panic or ordinary-source failure has been established for this lead.

## Question to resolve

Can a supported caller reuse raw execution after an ordinary runtime error while retaining a closure whose capture still refers to the previous execution's stack?

The required invariant is that a surviving capture preserves its language value across teardown of its owning activation. Before storage is discarded or reused, an open capture must be closed or otherwise retain valid ownership of that storage.

The concern is a possible mismatch between diagnostic preservation after failure and cleanup before the next execution. A VM bounds check alone would not establish that a reused slot still belongs to the original capture.

## Investigation sequence

1. Refresh the named functions and their relevant diffs. Trace production callers of `run_in_module` and its documented preconditions. Determine whether reuse requires explicit cleanup or whether callers already guarantee it.
2. Trace creation, escape and ownership of a captured local across an ordinary error. Identify a supported root that keeps the closure alive after the activation fails.
3. If the reuse sequence is supported, prepare a bounded local diagnostic case: create and retain a capture, return an ordinary runtime error, begin a second raw execution, then inspect the retained closure's value.
4. Compare the same lifecycle through the disciplined cell path and through any documented explicit cleanup sequence. Keep closure liveness separate from stack-slot lifetime.
5. Record the exact execution boundary, expected value, observed result and cleanup state. Preserve source and output under `evidence/` if a probe is eventually run.

Do not recover from an arbitrary Rust panic to manufacture a supported lifecycle. Do not assume that direct mutation of VM internals represents a valid embedding contract. Do not repeat previous constant, super, GC or native-admission probes for this investigation.

## Classification criteria

| Outcome | Classification |
| --- | --- |
| Supported reuse discards storage while a surviving capture still refers to it, with an observed incorrect result or controlled diagnostic failure | Confirmed issue; document the root cause and reachability |
| All supported callers or enforced preconditions close captures before reset | Disproved for the examined supported path |
| Raw execution explicitly requires fresh state or caller cleanup, and callers satisfy that requirement | Intentional contract; record the boundary and any documentation gap |
| Capture escape, supported reuse or the final observed behavior cannot be established | Unresolved; retain the missing evidence |

A manually inconsistent internal state is not sufficient to claim an ordinary-source defect. A successful diagnostic process exit is not a passing regression test if it records incorrect behavior.

## Architecture to preserve

Preserve close-before-truncate cleanup, home-frame generation tokens, generational object handles, diagnostic capture before unwind and the separate disciplined cell API. Any future correction should belong to the execution lifecycle boundary rather than compensating inside individual closure reads.

## Checkpoint and next action

No additional L05 verification was completed. The integrated assessment remains provisional B and the audit remains incomplete. Resume with the production caller/precondition trace before designing an executable case. Keep fixes and commits outside the audit scope.

For the warning's prior context, see [S1](AUD-RUNTIME-S1-cybersecurity-warning-explanation.md). That document does not establish the trigger of the newly reported warning.
