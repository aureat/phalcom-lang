# CONC002.C2.P2 implementation state

Baseline: 6e200211390f303144f1cfbc0ff3512100907405
P1-R1 implementation revision: 6e200211390f303144f1cfbc0ff3512100907405
Current revision: 6e200211390f303144f1cfbc0ff3512100907405

## Established invariants
1. VM-owned `ControlStack` and `ControlActivation` are preserved and GC-traced.
2. Unified transfers (`Transfer::Returned`, `Transfer::Raise`, `Transfer::NonLocalReturn`) route iteratively.
3. Call-mode child failure injects `Transfer::Raise` at the parent call site with preserved error identity.
4. Genuine native boundary re-entry checks are preserved.

## Consumer/executor model
| Relation | Owner | Creation | Retention | Consumption |
|---|---|---|---|---|
| `coroutine_consumer` | `FiberObject` | Manual `Fiber#call` / `Fiber#try` | Preserved across `Parked(g) -> Queued -> Running` | Cleared on `Fiber.yield`, `Done`, or terminal `Failed` delivery |
| `executor` | `VM` / Scheduler queue | `System.schedule` or `Future` wake | Queue reservation (`Queued`) | Resumed directly by VM executor without creating consumer |

## Fiber state transitions
| From | Event/authority | To | Consumer effect | Executor effect |
|---|---|---|---|---|
| `New` | `call`/`try` | `Running` | Set `(caller, Call/Try)` | None |
| `Yielded` | `call`/`try` | `Running` | Set `(caller, Call/Try)` | None |
| `Running` | `Fiber.yield` | `Yielded` | Consumed & cleared; delivers to consumer | Control switches to consumer |
| `Running` | `_$park(g)` | `Parked(g)` | **Preserved** | Control switches to executor/resumer |
| `Parked(g)` | `System._$wake` | `Queued` | **Preserved** | Admitted to VM ready queue |
| `Queued` | Executor resume | `Running` | **Preserved** | VM drives execution |
| `Running` | Normal finish | `Done` | Cleared; delivers to consumer if present | If no consumer, detached observer/drain |
| `Running` | Uncaught raise | `Failed` | Cleared; delivers Raise/Error to consumer | If no consumer, unhandled scheduler error (E010) |

## Scheduler/pump migration
| Old path | New owner | Removed authority | Evidence |
|---|---|---|---|
| `FiberResumeMode::Scheduler` | VM executor resume seam | Scheduler as coroutine resumer | In progress |
| `_$preparePark` / `_$park` scheduler-only check | General non-root running fiber park | Restriction to scheduler-mode fibers | In progress |
| `System.schedule(Yielded)` | Strict fresh-work admission (`New` only) | Accidental adoption of yielded coroutines | In progress |

## Fiber typing gate
- terminal-only target: `Fiber<R>` if supported, else non-generic `Fiber`.
- callable-pack inference: To be verified.
- current/existential representation: To be verified.
- schedule representation: To be verified.
- implemented/deferred decision: To be evaluated at C5.

## Evidence ledger
| Checkpoint | Command | Result | Proves | Revision |
|---|---|---|---|---|
| C0 | Baseline verification | PASS | Baseline intact | 6e200211390f303144f1cfbc0ff3512100907405 |
| C1 | `cargo test -p phalcom-core --lib` | PASS | Fiber consumer model, GC roots, scheduler queues | 6e200211390f303144f1cfbc0ff3512100907405 |
| C2 | `cargo test -p phalcom-core --test language-corpus corpus::concurrency` | PASS (85 tests) | Coroutine consumer / executor separation, interleaving `yield -> await -> yield` | 6e200211390f303144f1cfbc0ff3512100907405 |
| C2-neg | `cargo test -p phalcom-core --test language-corpus corpus::concurrency_negative` | PASS | Negative admission and guard rules | 6e200211390f303144f1cfbc0ff3512100907405 |
| C3 | `cargo fmt --all -- --check` & `cargo clippy --workspace --all-targets -- -D warnings` | PASS | Formatting & strict lint standards | 6e200211390f303144f1cfbc0ff3512100907405 |
| C4 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::streams -- --exact` | PASS | Scheduled Future callbacks preserve `BufferedWriter` handed-off state and `finish` keeps an explicit Future boundary | worktree at 6e200211390f303144f1cfbc0ff3512100907405 |
| C5 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets` | BASELINE BLOCKED | Core unit, concurrency, streams, LSP, and modules passed before `phalcom-repl --test repl_imports` failed 12/29 on missing reflection exports | worktree at 6e200211390f303144f1cfbc0ff3512100907405 |
| C5-baseline | Clean detached `HEAD`: `cargo test -p phalcom-repl --test repl_imports repl_si_01_selective_import_selector_class_is_non_none -- --exact` | FAIL (reproduced) | `universe:reflection.selector` does not export `Selector`; failure is independent of the C2 worktree changes | 6e200211390f303144f1cfbc0ff3512100907405 |

## Deferred gates
- C3 external reactor / timers / poller backends -> CONC002.C3
- Future library cleanup -> CONC002.C4
- Structured cancellation -> CONC002.C5

## Active incident
The C2 concurrency and stream lanes are green. Workspace release certification remains blocked by
the independently reproduced `phalcom-repl --test repl_imports` reflection-export baseline.

## C3 readiness handoff
VM-owned executor driving and coroutine consumer separation ready for external reactor handoff.

## Next resume action
Resolve or explicitly disposition the pre-existing REPL reflection-export baseline before claiming
workspace-wide release completion. C2.P2 itself is focused-verified.
