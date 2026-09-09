# Retained concurrency-control evidence

Audited working source at `b84da68fededc4b9e5b6e4841f0e1cb74ffcf7ad`, 2026-09-09 local date. Probes diagnose current behavior; they are not regression tests asserting desired semantics. No production code was modified.

## Execution inventory

| Probe | Coverage | Observed result |
| --- | --- | --- |
| [01-coroutine](probes/01-coroutine.ph) | A/B/C dynamic resumers, first/resume arguments, None/Error data, late Try capture, root predicate | Exit 0; correct data transfer; fresh non-root reports isRoot true |
| [02-failure](probes/02-failure.ph) | Call cascade, Try boundaries, Error identity, escaped ancestor capture, abort, surrounding catch/ensure | Exit 0; linked failures, capture reads 77; abort caught; guarded child resume rejected before entry |
| [03-async](probes/03-async.ph) | Three awaits and late uncaught failure | Exit 0; premature Some(None) remains after terminal return/failure |
| [04-waiter-generator](probes/04-waiter-generator.ph) | Ordinary waiter and generator containing await | Exit 0; generator's 42 is delivered to pump and absent from consumer stdout |
| [05-wrong-authority](probes/05-wrong-authority.ph) | Manual call/try/schedule while pending; stale waiters crossing subsequent suspensions | Exit 0; pending awaits return None and execution advances through wrong waits |
| [06-duplicate-queue](probes/06-duplicate-queue.ph) | Manual ready entry plus Future wake entry | Exit 70; finished-Fiber refusal prevents healthy later task |
| [07-native](probes/07-native.ph) | For loop, ordinary each yield/await, genuinely native handler path, dead waiter filtering | Exit 0; flat callbacks suspend; native-boundary error captured, host survives settlement |
| [08-continuations](probes/08-continuations.ph) | Pending then/map/catch callbacks await | Exit 0; six Some(None) observations across before/after wake |
| [09-guard-only](probes/09-guard-only.ph) | Proposed isDone-only guard in separate library-equivalent driver | Exit 0; driver Done, action later Done, outer permanently pending |
| [10-state-inspection](probes/10-state-inspection.rs) | Native read-only status/buffer assertions inside child | Exit 0; two Running statuses, ancestor state parked |
| [11-root-drive-stale](probes/11-root-drive-stale.ph) | Duplicate queue entry in automatic exit pump | Exit 70; healthy later root-drive task never runs |
| [12-resume-validation](probes/12-resume-validation.ph) | Wrong first arity recovery, call/try after Done/Failed, self/ancestor resume, scheduling current | Exit 0; invalid resumes refused, current Fiber can enqueue itself |

Every probe has `.stdout`, `.stderr`, and `.exit` siblings. Probes 03/04/09 retain additional `--trace=fibers` output in stderr. Timestamps are UTC; raw traces show September 8 while local date is September 9. The runner compares stdout and exit status, not timestamped traces.

The [focused corpus log](concurrency-corpus.log) records **2 aggregate tests passed**, 63 filtered out: positive and negative concurrency fixture groups. It includes existing settle-once, self/finished/root refusals, arity, native boundary, and both failure-upvalue regressions. This is not a broad release run.

## Reproduction

From repository root:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build -p phalcom-core --bin phalcom
python3 docs/audit/concurrency-control-2026-09-09/evidence/run-probes.py --check
python3 docs/audit/concurrency-control-2026-09-09/evidence/run-probes.py --state --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus concurrency -- --nocapture
```

Run a bounded subset with `--only 03-async 09-guard-only`. Run a source directly with `target/debug/phalcom --plain <probe.ph>`; add `--trace=fibers` to inspect switches and terminal events. Commands are serial; the runner uses a 60-second timeout per process and never overwrites retained evidence.

The Rust probe compiles against the newest local phalcom-core rlib using `nightly-2026-07-10`. Rebuild first if a newer incompatible artifact exists. It patches a probe-only Inspector method to inspect heap fields; no production method or file changes. Its assertions deliberately establish the observed defective Running status, not a desired acceptance rule.

## Limits

There is no implementation repair, baseline predecessor comparison, workspace certification, or general GC/exception-system proof. Audited core sources were unchanged. Unrelated documentation/semantic work appearing during the audit was preserved. The linked report separates executed consequences, source-established paths, and unresolved semantics.
