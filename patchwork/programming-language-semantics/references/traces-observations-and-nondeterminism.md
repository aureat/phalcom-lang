# Traces, Observations, and Nondeterminism

Semantics should define what program behavior is observable before defining equivalence, optimization correctness, or concurrency properties.

## 1. Behavior is more than a value

A complete program can terminate with a value/status, throw uncaught error, produce IO events, mutate external state, diverge, interact forever, or schedule fibers in multiple allowed orders.

Represent behavior as outcome plus trace or as an event-labeled transition system.

## 2. Labeled transitions

```text
C --α--> C'
```

where `α` may be:

```text
τ                         internal/unobservable step
print(text)
read(resource, result)
spawn(fiber)
yield(fiber)
moduleInit(M)
nativeCall(name, result)
```

Internal implementation steps can vary without changing observable trace.

## 3. Trace projection

Not every event is user-visible. Define projection:

```text
observe(trace) -> observableTrace
```

Compiler correctness may permit different internal traces as long as projected behavior matches.

## 4. Nondeterminism

A configuration can have multiple valid successors because of scheduler choice, external input, clock/randomness, intentionally unspecified order, or native/environment outcomes.

Then a program denotes a **set** of behaviors, not one behavior.

## 5. Deterministic core plus nondeterministic environment

Useful factoring:

```text
language core step is deterministic given explicit external response/scheduler choice
```

This supports clearer proofs/tests than implicit external uncertainty.

## 6. Termination, divergence, and stuckness

Use behaviors such as:

```text
Terminates(outcome, trace)
Diverges(possibly infinite trace)
GoesWrong(internal semantic stuck)
```

A user-level exception is a defined outcome, not `GoesWrong`.

## 7. Safety and liveness trace properties

Safety says bad event never occurs:

```text
no write after close
no private invocation without authority
module initialization at most once
```

Liveness says good event eventually occurs under assumptions:

```text
runnable fiber eventually scheduled
future waiter eventually resumed after completion
```

Fairness assumptions matter for liveness.

## 8. Refinement

For optimization, target may have fewer nondeterministic behaviors than source only if removed behaviors are unobservable, undefined, or explicitly permitted by preservation policy.

Do not use "same output" when concurrency or external calls exist.

## 9. Trace equivalence versus state equivalence

Two internal machine states may differ while all future observations match. Conversely, identical current returned values can hide different pending IO or scheduler behavior.

Choose relation based on observable semantics, not structural equality of implementation states.

## 10. Scheduler traces

Possible events:

```text
yield(f)
suspend(f, reason)
wake(f)
resume(f)
complete(f, outcome)
cancel(f)
```

Tests can assert ordering constraints without freezing entire scheduler implementation.

## 11. External resource traces

For IO/process APIs, model abstract operations rather than host syscalls when possible:

```text
open(path) -> handle | error
read(handle,n) -> bytes | eof | error
spawnProcess(spec) -> process | error
```

This keeps semantics platform-independent while preserving observable outcomes.

## 12. Testing nondeterministic semantics

Assert properties/allowed sets:

- outcome belongs to permitted set;
- event A always precedes B;
- module init occurs once;
- every completed future wakes registered waiters under fairness assumptions;
- no event occurs after terminal cancellation if semantics promises that.

Avoid brittle tests freezing incidental scheduler order unless normative.

## 13. Competency checks

1. Why does concurrent program denote set of traces rather than one trace?
2. What is purpose of internal `τ` transitions?
3. When is exception observable outcome rather than stuck state?
4. Why can compiler correctness compare projected traces?
5. How should test assert nondeterministic but constrained scheduling?
