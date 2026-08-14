# Abstract Machines, Continuations, and Frames

An abstract machine gives an executable mental model between high-level operational rules and Phalcom's stack bytecode VM. It is especially useful for closures, exceptions, `super`, non-local return, and fibers.

## 1. CEK and CESK intuition

A CEK machine state:

```text
⟨control, environment, continuation⟩
```

A CESK machine adds mutable storage:

```text
⟨control, environment, store, continuation⟩
```

For Phalcom, extend conceptually with modules and scheduler:

```text
⟨control, env, store, kont, modules, scheduler⟩
```

This is a semantic model, not a proposal to replace the bytecode VM.

## 2. Why continuations matter

A continuation records the rest of computation. For:

```phalcom
f(a(), b())
```

continuation frames can encode:

```text
1. after receiver/function, evaluate a
2. after a, evaluate b
3. after b, perform call/send
4. after call, continue caller
```

No hidden host recursion is required in semantics.

## 3. Useful continuation frames

```text
Halt
Sequence(restStatements, env, next)
SendReceiver(selectorSyntax, args, env, next)
SendArg(receiver, selector, doneArgs, remainingArgs, env, next)
InvokeReturn(callerFrameId, next)
Assign(location, next)
IfBranches(thenBlock, elseBlock, env, next)
Loop(loopId, condition, body, env, next)
Handler(handlerSpec, env, next)
Ensure(cleanup, env, pendingOutcome, next)
```

These frames expose exact evaluation and unwinding order.

## 4. Method activations

```text
Activation {
  id,
  method,
  receiver,
  environment,
  lexicalClass,
  fiber,
  returnContinuation,
  alive
}
```

`lexicalClass` supports `super`; `receiver` supports `self`; `id/alive` supports non-local return.

## 5. `super` in a machine

No superclass receiver is pushed. Send metadata records a different lookup start:

```text
SendInfo {
  receiver = self,
  lookupStart = superclass(currentLexicalClass),
  selector = s
}
```

This makes the semantic distinction explicit.

## 6. Exception unwinding

On `Throw(v)`:

```text
while top continuation is not applicable Handler:
    if top is Ensure:
        execute cleanup with pending Throw(v)
    else:
        discard frame
consume Handler
```

Exact precedence depends on cleanup policy, but machine structure makes it inspectable.

## 7. Non-local return

A block carries `homeFrameId`. On non-local return:

```text
if homeFrame dead -> DeadFrameError
if homeFrame belongs to prohibited fiber -> control error
else unwind until homeFrame return boundary
```

A physical stack pointer is not semantic target identity.

## 8. Fibers

Each fiber owns a suspended machine configuration or continuation stack:

```text
FiberState = Running(machine)
           | Suspended(machine, reason)
           | Completed(outcome)
```

`yield` stores current continuation and selects another runnable fiber. Resumption reinstates it.

Suspension is not termination: suspended home frames remain live; completed ones are dead.

## 9. Stackful versus stackless implementation

Phalcom can present stackful fiber semantics even if a future implementation transforms control into heap state machines. Semantic equivalence depends on observable control behavior, not physical stack representation.

## 10. Bytecode mapping

Typical correspondence:

```text
semantic continuation     VM mechanism
----------------------    -------------------------
Sequence                  instruction pointer
method return             call frame + return IP
lexical environment       locals/upvalues
store                     slots/heap cells
handler                    handler metadata
fiber suspension           saved VM stack/frame state
```

When VM frame layout changes, verify correspondence rather than assuming it.

## 11. Garbage collection roots

Abstract-machine state identifies semantic roots:

- receiver/locals in frames;
- captured cells in closures;
- values stored in continuations/argument assembly;
- suspended fiber stacks;
- pending exceptions/control values;
- module namespaces.

A GC omitting suspended continuations can collect semantically live values.

## 12. Native calls

Native primitives can be modeled:

```text
NativeCall(receiver, args, world) -> Outcome × world'
```

If native code can callback into Phalcom, throw, allocate, or yield, VM must preserve roots and control invariants across boundary.

## 13. Continuation marks and source metadata

If stack traces/debugging expose frames, compiler-generated continuations should retain source-level metadata sufficient to reconstruct meaningful source frames. This is separate from machine control state but constrains lowering.

## 14. Tail calls

Tail-call optimization can reuse/eliminate caller frame only when no semantic feature requires its continued identity. Potential blockers include:

- non-local-return home frame references;
- reflective stack inspection;
- cleanup/handler boundaries;
- debugging guarantees.

If tail-call semantics are not observable, optimization may collapse frames; if frame identity is observable, guards/restrictions are required.

## 15. Why not use VM as semantics directly?

VM contains administrative details such as hidden locals, temporary stack values, dispatch caches, opcode fusion, and GC bookkeeping. Those should evolve freely. An abstract machine captures semantic control structure without freezing opcode design.

## 16. Competency checks

1. Which activation fields are needed for lexical `super`?
2. Why is suspended home frame live while completed frame is dead?
3. How can exception cleanup be represented without host exceptions?
4. Which continuation values must be GC roots?
5. Why can a stackless implementation implement stackful semantics?
6. When could tail-call elimination become semantically observable?
