# Effects, Closures, and Concurrency Analysis

Value analysis answers “what might this expression produce?” Effect analysis answers “what might evaluating it do?” Phalcom needs both. A call whose return is precisely known may still mutate fields, invoke a supplied block, return non-locally through that block, throw, yield a fiber, block the VM thread, perform IO, cross native code, or mutate reflective state. Treating all of that as a single `impure` bit loses information needed by the checker, optimizer, prover, diagnostics, and concurrency tooling.

This reference defines an abstract-analysis model for effects, closures, higher-order calls, and cooperative concurrency. It does not define normative fiber scheduling or exception semantics; those belong to the corresponding language/runtime skills. Load those specifications before making CURRENT claims.

## 1. Effects are abstract sets of observable behaviors

Let a concrete execution of expression `e` from state `σ` produce value/state/trace:

```text
⟨e, σ⟩ ⇓ ⟨v, σ', τ⟩
```

where `τ` records observable or semantically relevant events. An effect analysis abstracts possible traces:

```text
Effects#(e) = α({ τ | e can execute with trace τ })
```

For a may-effect domain, ordering is set-like inclusion:

```text
E1 ⊑ E2  iff  γ(E1) ⊆ γ(E2)
E1 ⊔ E2  covers either effect set
```

A basic domain can be a product:

```text
Effect# =
    Throws#
  × Reads#
  × Writes#
  × Allocation#
  × IO#
  × Scheduling#
  × Reflection#
  × Native#
  × HigherOrder#
```

Keeping components orthogonal avoids a combinatorial enum.

## 2. A practical Phalcom effect summary

A future internal representation might resemble:

```rust
struct EffectSummary {
    throws: ThrowSet,
    reads: AccessSet,
    writes: AccessSet,
    allocates: bool,
    io: IoEffect,
    scheduling: SchedulingEffect,
    reflection: ReflectionEffect,
    native: NativeEffect,
    higher_order: HigherOrderEffect,
    abrupt: AbruptEffect,
    trust: FactTrust,
    provenance: EffectProvenance,
}
```

This is a design pattern, not a requirement to expose user-facing effect types.

For early implementation, components can be coarse:

```text
Reads  = None | Known(Set<LocationClass>) | AnyReachable
Writes = None | Known(Set<LocationClass>) | AnyReachable
Throws = No | MayThrow
IO     = No | MayIO
Scheduling = NoYield | MayYield | BlocksThread | Spawns
Reflection = None | ReadsDispatch | MutatesDispatch | DynamicInvoke
```

Precision should increase only when a consumer needs it.

## 3. Join and composition

Branch merge joins may-effects component-wise:

```text
Effects(if c then a else b)
    = Effects(c) ⊔ Effects(a) ⊔ Effects(b)
```

Sequential composition is not always identical to join because order can matter. At minimum, the aggregate may-effect set is:

```text
Effects(a ; b) = Effects(a) ⊔ Effects(b)
```

but consumers performing reordering need conflict information:

```text
CanReorder(A, B) only if
    Write(A) ∩ (Read(B) ∪ Write(B)) = ∅
and Write(B) ∩ Read(A) = ∅
and neither has ordering-sensitive effects
```

IO, synchronization, exceptions, yielding, allocation identity, reflection, and user-observable finalization can all create additional ordering constraints.

## 4. Pure/impure is not enough

A boolean `impure` cannot distinguish:

```text
x + y                # maybe only dispatch/user code
readFile(path)       # IO
fiber.yield()        # scheduling
obj._field = v       # local heap mutation
perform(selector)    # dynamic invocation
Class.install(...)   # dispatch mutation, if supported
nativeHash(bytes)    # native call, maybe pure under contract
```

Different consumers need different cuts:

- optimizer cares about reads/writes/throws/yields and observable allocation;
- prover cares about mutation footprint, exceptions, calls, and trusted contracts;
- LSP may only need “invokes argument block” and “dynamic send” for inference;
- lints may care about blocking inside a fiber or ignored future/task result.

Therefore define the smallest structured domain that serves actual consumers, but do not collapse semantically distinct effects merely to simplify storage.

## 5. Expression evaluation and sends

A send effect includes all evaluation steps:

```text
Effects(receiver.send(args...)) =
    Effects(receiver)
  ; Effects(args in lexical order)
  ; Effects(dispatch lookup, if observable)
  ; Effects(selected target)
```

This matters because a later argument can observe mutations from an earlier argument, and argument evaluation can itself throw/yield/return abruptly.

A compiler optimization cannot move receiver or argument evaluation merely because it ultimately calls the same selector.

## 6. Closure creation versus closure invocation

A block/closure has two different effect surfaces.

### Construction effects

Creating a closure may:

- allocate a closure object;
- capture bindings/upvalue cells;
- extend captured storage lifetime;
- possibly change escape status.

It does **not** automatically execute the body.

### Latent invocation effects

The body has latent effects:

```text
ClosureValue# = {
    captures,
    latent_effects,
    nonlocal_control,
    return_fact,
    invocation_contract?
}
```

Example:

```text
let f = || { counter = counter + 1 }
```

The assignment to `counter` is not an immediate write at closure construction. It becomes possible when `f` is invoked.

This distinction is already reflected in the current LSP structured flow: `BlockEffects` retains captured writes, non-local returns, invoked parameters, and dynamic-send information separately from ordinary flow, and caller analysis propagates block effects when a callee summary says the corresponding parameter is invoked.

## 7. Captures turn lexical variables into shared cells

A captured mutable variable is no longer analyzable as if each closure held an independent copied value.

Conceptually:

```text
let x = 0
let inc = || { x = x + 1 }
let read = || { x }
```

Both closures access the same storage cell. A useful abstract location is:

```text
CapturedCell(BindingId(x), HomeCallableId)
```

If the closure escapes, this cell outlives the home frame. If the closure can run later or on another fiber, the value of `x` at a subsequent local read may require interference reasoning.

The semantic identity of a binding and the runtime representation of its captured cell are related but not identical. Do not use stack-slot position as a durable semantic identity.

## 8. Higher-order invocation contracts

For a callable parameter `p`, a summary can record whether the callee invokes it. “Invokes” itself can be refined along several dimensions:

```text
InvocationSummary(p) = {
    cardinality: Never | AtMostOnce | ExactlyOnce | Many | Unknown,
    timing: Synchronous | Deferred | Unknown,
    control: PropagatesNonLocalReturn | CapturesIt | Unknown,
    ordering: before/after selected effects if known,
}
```

The current LSP records the set of parameter positions invoked on reachable source paths. That is a useful **CURRENT** foundation, but not yet the full contract above.

Why cardinality matters:

```text
repeatTwice(|| { x = x + 1 })
```

If the analyzer assumes “invoked” means once, it computes the wrong state.

Why timing matters:

```text
schedule(|| { x = 1 })
use(x)
```

A deferred callback does not justify updating `x` before `use(x)`.

## 9. Non-local control from blocks

Smalltalk-inspired block semantics may include non-local return. When normative Phalcom semantics allow it, block invocation can terminate a home callable rather than returning normally to the immediate caller.

Model this as an abrupt edge/effect, not a normal return value:

```text
BlockFlow = {
    normal,
    local_returns,
    nonlocal_returns(target_home),
    throws,
    breaks/continues where legal,
}
```

A higher-order method that invokes a block must propagate those abrupt outcomes according to the language's exact control semantics. An optimizer cannot assume a callback returns normally merely because its ordinary return shape is known.

The current LSP already records `nonlocal_returns` in `BlockEffects`; preserve that distinction as the analysis grows.

## 10. Effect polymorphism for higher-order helpers

Many library operations have effects parameterized by a callback:

```text
map(collection, block[E])
    effects = collection_read ⊔ allocation ⊔ E

withResource(block[E])
    effects = acquire ⊔ E ⊔ release
```

An internal effect variable can model this without user-facing syntax:

```text
Summary(map) = BaseEffects ⊔ InvokeEffect(param_block)
```

At a call site, substitute the supplied block's latent effects. This is analogous to summary parameterization, not necessarily a formal effect type system.

## 11. Cooperative fibers: `may_yield` is a semantic boundary

In a cooperative scheduler, code executes without interleaving until it reaches an operation that can yield/suspend. This makes `may_yield` unusually valuable.

Suppose analysis establishes a fact about shared mutable state `S`:

```text
assert S.flag == false
call f()
use S.flag
```

If `f` is guaranteed not to yield and no invoked user code mutates `S`, the fact may survive. If `f` may yield, another runnable fiber may mutate shared reachable state before control resumes. Therefore suspension can require interference havoc.

A conservative transfer is:

```text
post = transfer_call(pre, f)
if may_yield(f):
    post = InterferenceHavoc(post, SharedReachableState)
```

The havoc can be narrowed by ownership/isolation analysis later.

## 12. `blocks_thread` is not `may_yield`

These effects are nearly opposites operationally:

```text
may_yield
    current fiber suspends; scheduler may run others

blocks_thread
    underlying VM/runtime thread cannot schedule other fibers while blocked
```

A native blocking syscall can freeze cooperative scheduling even though it never “yields.” Tooling and lints should keep the distinction. A future async-aware standard library may offload blocking operations, but the analysis summary should describe actual behavior of the selected implementation/profile.

## 13. Spawn and deferred execution

Spawning/scheduling a closure changes both escape and concurrency properties:

```text
spawn(block)
```

can imply:

- `block` escapes the current call;
- captured mutable cells escape to another fiber/task;
- the block's latent effects are not immediate sequential effects;
- its effects become possible interference on shared state;
- cancellation/lifetime rules may add abrupt cleanup effects.

Do not simply join the block's writes into the current post-state as though it ran synchronously.

## 14. Shared-state interference domain

A scalable first concurrency analysis can classify locations:

```text
LocalOnly
EscapesCallable
SharedAcrossFibers
Global
UnknownReachability
```

Then yielding invalidates only locations that may be concurrently reachable:

```text
YieldHavoc(state):
    preserve LocalOnly facts
    preserve deeply immutable facts
    forget or join SharedAcrossFibers / Global mutable facts
    conservatively treat UnknownReachability
```

More precise happens-before or actor/isolation reasoning belongs in a future concurrency-specific skill if Phalcom introduces stronger concurrency primitives.

## 15. Cancellation and cleanup

If/when Phalcom fibers support cancellation, static analysis must distinguish:

```text
normal completion
throw/error
non-local return
cancellation
scheduler shutdown/abort if observable
```

Resource correctness may require cleanup on every abrupt path. A future effect summary should identify cancellation points. Do not claim `{P} C {Q}` over all executions if cancellation can interrupt `C` and the proof model ignored it.

## 16. FFI callbacks and retained closures

Rust FFI can turn a lexical closure into a long-lived callback. A native signature needs to state whether it:

```text
invokes callback synchronously
retains callback after return
invokes callback zero/one/many times
may invoke from another runtime thread/fiber
may mutate objects passed by handle
may re-enter Phalcom
```

Without these facts, escape and effect analysis must assume the more conservative behavior allowed by the FFI contract.

GC/rooting is a runtime concern, but analysis decisions that depend on callback lifetime must match the actual rooting/ownership behavior.

## 17. Effect trust levels

Consumers should distinguish:

```text
ExactLanguageInvariant
SoundSourceSummary
DeclaredTrustedNativeSummary
ProvenContractSummary
HeuristicObservedSummary
UnknownDynamicBoundary
```

A source summary extracted from incomplete editor syntax may be useful for hover but not safe for code motion. A native declaration may be accepted as trusted for checker/prover only under explicit FFI trust policy.

## 18. Current Phalcom mapping

At the inspected 2026-08-14 repository baseline, the current semantic engine has:

```text
SummaryEffects {
    dynamic_send: bool,
    invokes_parameters: BTreeSet<usize>,
}
```

and block analysis retains:

```text
nonlocal_returns
captured_writes
invokes_parameters
dynamic_send
```

These are **CURRENT** and valuable. They should be generalized rather than duplicated when new consumers need throws, yield/blocking, global/field writes, IO, reflection, FFI, escape, or callback timing. Do not create a separate checker-only effect engine with different callable identities and propagation rules.

## 19. Rust representation principles

Prefer IDs and interned sets over long-lived references:

```text
EffectLocation::Field(FieldId)
EffectLocation::Global(GlobalId)
EffectLocation::Captured(BindingId)
EffectLocation::Region(AbstractRegionId)
```

Use canonical sorted/interned small sets for deterministic equality. If summaries are cached, separate semantically relevant equality from provenance/generation metadata so harmless provenance changes do not force whole-world invalidation.

For a cache, specify:

```text
key: CallableId + body/surface dependency revisions + analysis mode
value: canonical EffectSummary
dependencies: callees, native summaries, relevant class/module surfaces
invalidation: dependency semantic change
concurrency: immutable published generation
memory bound: per-generation/reachable summary retention
```

## 20. Failure modes

- Applying block body effects at closure construction time.
- Assuming an invoked block runs exactly once.
- Treating deferred callback effects as synchronous state updates.
- Treating `may_yield` and `blocks_thread` as one boolean.
- Preserving shared mutable facts across a yield with no isolation proof.
- Ignoring non-local return because ordinary return inference is precise.
- Calling every effect “impure” and then trying to recover reordering information later.
- Trusting native code as effect-free because source is unavailable.
- Treating dynamic call as “unknown return only.”
- Using heuristic editor effects for optimizer transformations.

## 21. Testing obligations

Build targeted tests for:

1. closure construction does not apply latent captured write;
2. synchronous known callback invocation does apply the callback transfer;
3. zero/one/many invocation cardinalities produce different states;
4. deferred callback does not update immediate post-state;
5. non-local return reaches the correct home callable flow;
6. nested closures share the correct captured cell;
7. dynamic callback target widens effects conservatively;
8. `may_yield` invalidates shared mutable facts but preserves local immutable facts;
9. blocking native effect is distinguishable from yielding;
10. native callback retention marks closure/captures escaped;
11. changed callee effect summary invalidates callers;
12. incremental effect facts equal clean rebuild results;
13. optimizer refuses reordering when read/write sets conflict;
14. effect summary equality ignores provenance-only revision fields where intended.

A useful metamorphic property:

```text
extract an inline block body into a helper whose trusted summary is equivalent

=> effect result at the call site should remain semantically equivalent
```

## 22. Review questions

1. Is this effect immediate or latent until callback invocation?
2. Can the operation throw, return non-locally, yield, block, or spawn?
3. Which mutable locations can it read/write?
4. Does a closure escape? Who can invoke it later?
5. How often and when can callback parameters execute?
6. Does yielding permit interference with facts currently held?
7. What guarantees a native summary matches the Rust implementation?
8. Is this effect fact trusted enough for the consumer using it?
9. What dependency change invalidates the summary?
10. Are value facts and effect facts being solved coherently over the same callable graph?

An agent that cannot answer these should not approve an effect-sensitive optimization or proof.
