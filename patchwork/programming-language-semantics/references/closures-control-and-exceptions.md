# Closures, Non-local Control, and Exceptions

Blocks and abrupt control are where informal semantics most often breaks. Phalcom's `Block` object can capture lexical state and a home activation, so it is richer than a plain function pointer.

## 1. Closure semantic value

```text
Closure = {
    code,
    capturedEnvironment,
    homeFrameId?,
    homeFiberId?,
    metadata
}
```

A VM may store upvalue handles, frame indices, heap cells, or copied immutable values. The semantic requirement is preservation of lexical binding and control behavior.

## 2. Construction versus invocation

Block literal evaluation creates closure/captures. It does **not** execute body.

```phalcom
let x = 0
const b = || { x = 1 }
// x remains 0
b()
// x is 1
```

Analyzer summaries should distinguish latent block effects from immediate construction effects.

## 3. Invocation

Calling a block:

1. evaluates call arguments;
2. creates parameter bindings;
3. reuses captured lexical storage for free variables;
4. executes body;
5. returns ordinary block result or propagates abrupt control.

Dynamic caller does not replace lexical environment.

## 4. Local return

A local return targets currently executing callable activation:

```text
Return(frameId, value)
```

The method invocation consumes a return targeted at itself and converts it to its call result.

## 5. Non-local return

A block non-local return targets home method activation:

```text
NonLocalReturn(homeFrameId, value)
```

Control unwinds through intervening block/method calls until target frame.

This must be explicit because same block may be invoked by many dynamic callers while retaining one home activation.

## 6. Escaping blocks and dead frames

If a block escapes and is invoked after home frame completed:

```text
homeFrame.liveness = Dead
```

then non-local return cannot return through that frame. Phalcom's intended behavior is an explicit language error such as `DeadFrameError`, not host panic and not conversion to local return.

## 7. Fiber boundary

A home frame belongs to a fiber. If block is invoked on another fiber, define whether non-local return is forbidden immediately, raises only when attempted, is impossible because blocks cannot migrate, or follows another rule.

Never permit control to jump into another fiber's stack by implementation accident.

## 8. Throw

Throw is an abrupt outcome:

```text
Throw(errorValue)
```

It unwinds until a matching handler. Without handler it becomes an uncaught language error at fiber/program boundary according to runtime policy.

## 9. Handler frames

In abstract machine:

```text
HandlerFrame(pattern/type?, handlerBlock, savedEnv, nextKont)
```

On `Throw(v)`, unwind until applicable handler, executing cleanup as required.

## 10. Ensure/finally cleanup

If Phalcom provides cleanup constructs, define outcome composition. Example policy shape:

```text
body outcome      cleanup outcome     final outcome
Normal(v)         Normal(_)           Normal(v)
Throw(e)          Normal(_)           Throw(e)
Return(t,v)       Normal(_)           Return(t,v)
anything          Throw(e2)           Throw(e2)      // if cleanup throw overrides
```

If cleanup return overrides earlier throw, state it explicitly. Do not let Rust RAII define language policy implicitly.

## 11. Break and continue

Represent targets:

```text
Break(loopId, value?)
Continue(loopId)
```

Then nested loops/blocks can be reasoned about without guessing host-language targets.

If break/continue cannot cross closure-call boundaries, reject statically or define runtime error.

## 12. Cancellation

Cancellation is not automatically throw. Decide whether it:

- runs cleanup;
- is catchable;
- has dedicated outcome;
- propagates across future/join relationships;
- can interrupt only at yield points or arbitrary safepoints.

## 13. Control effects

Useful analyzer effects:

```text
mayReturnNormally
mayThrow
mayNonLocalReturn(target)
mayBreak(loop)
mayYield
mayCancel
```

These are not ordinary value types. A block can return `Int` and independently may non-locally return from its home method.

## 14. Tail expressions and Unit/self conventions

If methods use tail-expression results and methods with no meaningful value produce Unit/receiver convention, define it at method-body completion. Do not infer it from bytecode stack cleanup.

## 15. Continuation view

Abrupt control can be understood as continuation manipulation:

```text
return           discard frames until target call boundary
throw            discard until handler
break/continue   discard until target loop continuation
non-local return discard until home activation boundary
```

This clarifies interactions difficult to state with nested Rust calls.

## 16. Higher-order methods

If a method receives a block parameter and invokes it, caller effects include the block's latent effects. A summary may record:

```text
invokesParameter(index)
```

so interprocedural analysis can propagate block throws, captured writes, yields, and non-local returns only when invocation is reachable.

## 17. Static-analysis traps

- executing block body effects at construction;
- joining non-local return value into block's ordinary return type;
- forgetting higher-order invocation;
- assuming block with no construction effects is pure;
- ignoring dead-home-frame behavior in escape analysis.

## 18. Conformance scenarios

Test shared captured mutation, escaping block normal call after home return, escaping block non-local return after home return, nested non-local return, throw during argument evaluation, cleanup during return/throw, and cross-fiber block behavior when supported.

## 19. Competency checks

1. Why must non-local return contain target identity?
2. What state must block retain for lexical mutation versus non-local return?
3. Why is `mayNonLocalReturn` not block return type?
4. What happens if cleanup itself throws during earlier return?
5. Which semantics determines whether an escaping block can cross fibers?
