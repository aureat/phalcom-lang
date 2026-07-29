# Phase 02 Core Model

Checkpoint 02 replaces the monolith's configuration and outcome string tags with typed values. It intentionally does not move search algorithms into `src/core`.

## Public core values

`Settings`, `Phase`, `ExampleStatus`, `PropertyResult`, `FailureOrigin`, `Failure`, and `Statistics` are descriptive values. They use `@data`; values whose fields are write-once use `@immutable`; closed state families use `@sealed` and `@variant`.

`Settings` retains the Checkpoint 01 defaults:

| Setting | Standard value |
|---|---:|
| `maxExamples` | 100 |
| `maxDiscards` | 1000 |
| `maxShrinks` | 1000 |
| `maxChoices` | 10000 |
| `seed` | `None` |
| `database` | `None` |
| `phases` | Explicit, Reuse, Generate, Shrink |
| `statefulSteps` | 50 |
| `deadline` | `None` |

Getter and update selectors share names. An update returns a new `Settings`; it does not mutate its receiver.

## Outcome boundaries

`ExampleStatus` has four cases:

- `Valid`: the property completed successfully.
- `Invalid`: the example was rejected by an assumption or strategy domain.
- `Overrun`: replay or choice-budget execution could not complete.
- `Interesting`: the example produced a reproducible candidate failure.

`PropertyResult` has four cases:

- `Passed`: the configured search completed without a counterexample.
- `Falsified`: a stable counterexample was found.
- `Inconclusive`: the search could not establish pass or falsification, such as discard exhaustion.
- `Errored`: execution or verification failed independently of a counterexample.

The compatibility reporter dispatches on these variants. It never prints an overrun, health-check failure, or flaky verification as a falsifying example.

## Failure identity

`FailureOrigin` includes error type, module, selector, line, column, and optional label. Equality and `sameSite` therefore distinguish two assertion sites that throw the same error class.

The compatibility engine currently lacks throwing-frame reflection, so adapter-created origins use an explicit unknown location. This limitation is recorded rather than pretending that error-class identity is source-aware. The complete property engine will populate real origins when the relevant reflection surface is available.

## Mutable workers

`_StatisticsCollector`, `_PropertyContext`, and `_PropertyContextStack` remain mutable private workers. A collector freezes its current counters into `Statistics`. Context installation is stack-disciplined and uses `ensure`, so normal return and thrown errors both restore the previous stack state.

The stack is process-local because the assumed toolchain does not yet expose `ContextLocal<T>`. Fiber-local installation remains required before parallel property execution.
