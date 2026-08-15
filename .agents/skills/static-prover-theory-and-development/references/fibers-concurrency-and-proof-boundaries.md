# Fibers, Concurrency, and Proof Boundaries

## Sequential reasoning boundary

If Phalcom fibers are single-threaded/cooperative, code between yield points may be reasoned about sequentially with respect to other fibers—subject to callbacks/native concurrency.

Proof model must know which operations may yield.

## Suspension

At `yield`/`await`, other fibers may mutate shared module/object state. Facts about shared mutable state cannot automatically survive suspension.

Options:

- havoc shared mutable state according to interference summary;
- require ownership/immutability;
- use rely/guarantee reasoning later.

## Fiber-local state

Facts about truly fiber-local lexical state can survive suspension if no shared aliases/native mutation can reach it.

## Futures

Await result contract comes from future-producing operation. Exceptional completion adds throw path.

## Cancellation

If cancellation is introduced, every suspension point may have cancellation edge and cleanup obligations.

## Parallelism

If runtime later supports OS-thread parallelism, cooperative assumptions are invalid. A memory model and synchronization semantics become prerequisites for sound proofs over shared state.

## Advanced logics

Rely-guarantee, concurrent separation logic and temporal logic are appropriate only when Phalcom actually needs concurrent shared-memory proof. Do not implement them preemptively.

## Blocking FFI

Blocking does not create interleaving on a single-threaded scheduler, but it harms liveness. A liveness prover/diagnostic needs `mayBlockThread` effects distinct from safety assertions.

---

## Deep treatment: suspension as an interference cut

### Sequential segment model

For cooperative fibers, partition execution into segments between suspension points:

```text
segment_0 ; yield ; segment_1 ; await ; segment_2
```

Within a segment, shared-state interference from other fibers is absent only if native callbacks/parallel runtime behavior cannot interleave. At suspension, apply an interference transformer:

```text
H_after ∈ Interfere(H_before, scheduler/shared-state summary)
```

A simple sound first model havocs shared mutable regions and preserves proven fiber-local/immutable regions.

### Stable facts across await

A fact `P` survives suspension if its read footprint is disjoint from possible interference writes:

```text
Reads(P) ∩ InterferenceWrites = ∅
```

or if synchronization/ownership guarantees preservation.

### Future memory model boundary

If Phalcom adds true parallel shared-memory execution, the prover cannot merely “havoc more.” Data-race semantics, atomics, synchronization, and memory ordering become part of the language model. Safety proofs may require rely/guarantee or separation-based concurrency logic. Record cooperative assumptions explicitly so future runtime changes invalidate incompatible proof modes.
