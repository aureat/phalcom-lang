# Effects, Abrupt Control, and Computation Types

## Purpose

Type theory often begins with pure expressions `A -> B`, but Phalcom execution includes throwing, non-local returns from blocks, mutable captures, fibers/yielding, process termination, IO, and dynamic reflection. These are **computation effects**, not ordinary result payloads.

This reference prevents value types such as `Result<T,E>` from being used as a dumping ground for control semantics.

## 1. Value type versus computation behavior

A method can have normal result type:

```text
String
```

while its computation may also:

```text
throw ParseError
mutate self
perform IO
yield
nonlocally return
never return
```

Conceptual typing can separate:

```text
Γ ⊢ e : T ! ε
```

where `T` is normal result and `ε` is an effect/control summary.

Phalcom need not expose effect syntax publicly to benefit from this separation internally.

## 2. `Never` describes no normal result, not every effect

If expression always throws:

```text
fail() : Never
```

then it has no normal value. The throw effect/cause may be represented separately.

Likewise process termination or non-local return can synthesize `Never` at that program point because control does not continue normally, while carrying distinct control effect.

Do not infer that all `Never` computations are interchangeable for proof/optimization.

## 3. `Result<T,E>` is a value sum

```text
Result<T,E> = Ok(T) | Err(E)
```

Both alternatives are ordinary returned values.

Throwing `E` differs:

```text
throw E
```

because it transfers control to a handler/unwinds frames.

A transformation from throwing API to `Result` API is a semantic transformation that changes calling/handling style; it is not mere type normalization.

## 4. Exceptions as effect rows/sets

A checked-exception-like effect can be modeled:

```text
T ! {throw E1, throw E2}
```

Effect subtyping often uses set inclusion:

```text
ε1 ⊆ ε2
```

A replacement function that throws fewer effects is safer than one that throws additional unchecked effects if effects are part of contract.

Phalcom may choose unchecked exceptions and not expose exception effects in callable type equality. Static prover/analysis can still track them as summaries.

## 5. Non-local return from blocks

Smalltalk-inspired blocks can have non-local return semantics depending on Phalcom design.

A block body:

```text
|| { return x }
```

may return from its home method, not from block invocation.

At return expression program point:

```text
normal result = Never
control effect = NonLocalReturn(home_frame, type_of_x)
```

Typing block merely as:

```text
() -> Never
```

loses which home frame/result contract is targeted.

If non-local return is legal only while home frame active, runtime/lifetime semantics also matter.

## 6. Method return checking with non-local returns

Suppose home method declares:

```text
foo() -> String
```

A nested block performs:

```text
return 42
```

Checker must validate non-local return payload against home method's expected `String`, not block's normal result type.

Therefore typing context needs a return target:

```text
ReturnContext { home_callable_id, expected_type }
```

Blocks inherit/capture that target according to dynamic semantics.

## 7. Yield/suspension effects

Fibers/cooperative concurrency introduce suspension points.

Effect:

```text
MayYield
```

matters because mutable/refinement assumptions can become stale across yield if other fibers can mutate shared state.

A type system need not reject yielding functions, but semantic analysis/prover should know suspension can invalidate:

- field refinements;
- lock/ownership assumptions;
- time-sensitive invariants.

Do not encode `MayYield` as union in result type.

## 8. Async/future result versus effect

If future Phalcom API returns `Future<T>`, that is a value type representing delayed result.

Creating/awaiting future may additionally have suspension effects.

```text
Future<T>        value-level abstraction
MaySuspend       computation effect
```

Do not conflate them.

## 9. IO and native effects

`readFile() -> Bytes` may have effects:

```text
IO
throw IOError
```

A pure function type system can ignore them for basic type safety, but static prover, optimizer, constant folder, and refactoring cannot.

For example, common-subexpression elimination of effectful call is unsound even if result types match.

Thus type theory should expose an extension point to effect summaries shared with semantic analysis/optimizer.

## 10. Effect polymorphism

Higher-order function:

```text
map(f, xs)
```

inherits effects of callback `f`.

Effect-polymorphic type concept:

```text
map : ((A -> B ! ε), List<A>) -> List<B> ! ε
```

This is powerful but adds inference complexity. Phalcom can initially keep effects as semantic summaries rather than user-visible type parameters.

## 11. Effect ordering

An effect domain can form a lattice/set:

```text
Pure ⊑ MayThrow ⊑ UnknownEffects
```

or powerset of atomic effects.

This is an abstract-analysis/effect order, not subtype order of value types.

If effects participate in callable subtyping, define a bridge:

```text
replacement effects ⊆ expected allowed effects
```

while value parameter/result rules remain usual variance.

## 12. Mutation effects and refinements

Effect summary can name mutation footprint:

```text
Writes(self.field_x)
Writes(AnyGlobal)
```

A refinement fact on `self.field_x` survives a call only if call summary proves no relevant write.

This connects type/refinement system to semantic effect analysis. Do not make all calls erase all refinements forever if precise summaries exist; do not preserve them optimistically without evidence.

## 13. Dynamic sends

A dynamic receiver send may have conservative effect:

```text
UnknownEffects
```

unless runtime/semantic summaries constrain target set.

Static checker can still type result as `Dynamic` according to gradual policy, but prover/optimizer must respect unknown effects.

## 14. Reflection and dispatch mutation

Reflection can:

- invoke arbitrary user code;
- access/install methods if API permits;
- inspect type descriptors;
- mutate class state.

An optimizer relying on stable dispatch/type facts must guard/invalidate around such effects.

Type metadata being immutable does not imply object model is closed.

## 15. Handlers and effect elimination

A handler can turn an effect into normal value/control.

Conceptually:

```text
try e catch E => h
```

removes handled `throw E` from outward effect summary if all paths handled, while result type joins normal `e` result and handler result.

This demonstrates two separate operations:

```text
value join: T_e ⊔ T_h
control effect subtraction/union: (ε_e - handled_E) ∪ ε_h
```

Do not use one lattice for both automatically.

## 16. Abrupt control in CFG

`return`, `throw`, non-local return, break/continue, process exit produce edges to different control destinations, not ordinary fallthrough.

Type result `Never` helps expression typing, but CFG must encode control target.

A feature spec should define:

- normal successor;
- abrupt successor/target;
- stack unwinding/cleanup;
- captured home frame semantics;
- fiber suspension/resumption.

## 17. Static proof boundary

A function with effect summary `UnknownEffects` weakens proofs about heap state. A prover should require frame conditions/verified contracts before assuming fields remain unchanged.

Type checker may still accept value types. Again:

```text
type-correct != effect-free != proven contract
```

## 18. Runtime contracts

Typed-runner checks can themselves throw/type-error. Decide whether contract violation is:

- ordinary exception effect;
- distinguished fatal checker violation;
- tool-mode diagnostic outside language semantics.

Do not let instrumentation silently change catchable behavior unless mode specification permits it.

## 19. Testing obligations

- thrown expression types as no-normal-result without becoming `Result`;
- non-local return checks against home callable result;
- block normal result distinct from non-local control;
- yield invalidates mutable refinements under shared-state semantics;
- handler removes handled throw from summary;
- dynamic send carries conservative effects;
- optimizer/prover does not assume purity from result type;
- typed-runner contract violation semantics match mode spec.

## 20. Failure modes

- Encoding exception effects as `T | Error`.
- Treating every `Never` computation as same effect.
- Non-local return validated against block result instead of home method.
- Yield ignored when preserving field refinements.
- Pure-looking return type used to justify CSE/constant folding.
- Effect summary and subtype lattice conflated.

## 21. Competency questions

1. Why is `Result<T,E>` different from `T` plus `throw E` effect?
2. What extra information must non-local return carry beyond `Never`?
3. How can `MayYield` affect flow refinements without changing value result type?
4. What relation should effect sets use in callable replacement if effects are part of contract?
5. Why must optimizer/prover care about effects even if basic checker does not expose them to users?
