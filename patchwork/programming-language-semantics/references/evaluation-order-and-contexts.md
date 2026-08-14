# Evaluation Order and Evaluation Contexts

Evaluation order becomes observable as soon as expressions can mutate state, throw, allocate identities, perform IO, invoke user code, reflect, or yield. A language that fails to specify order has still made a design decision: implementations are free to disagree.

## 1. Specify order at every composition boundary

For Phalcom, explicitly decide order for receiver versus arguments, positional versus labeled arguments, computed labels, pack expansions, collection elements, map keys/values, assignments, subscripts, interpolation fragments, constructor arguments/field initializers, executable attributes, and module top-level initialization.

Prefer one global principle such as lexical left-to-right unless a construct intentionally differs.

## 2. Evaluation contexts

Evaluation contexts define where the next reduction occurs.

```text
E ::= []
    | E.send(args)
    | v.send(v*, E, args*)
    | E + e
    | v + E
    | ...
```

Then:

```text
e → e'
------------- CTX
E[e] → E[e']
```

avoids duplicating congruence rules for every nested expression.

## 3. Contexts encode order

For:

```phalcom
makeReceiver().mix(first(), label: second())
```

left-to-right semantics establishes:

```text
makeReceiver
first
second
lookup/invoke
```

If selector construction includes computed labels or dynamic argument packs, specify when those label/pack expressions execute relative to ordinary values.

## 4. Exactly-once evaluation

Lowerings often accidentally duplicate expressions.

Bad transformation:

```text
x[i] += rhs
```

into:

```text
x[i] = x[i] + rhs
```

if `x` or `i` has effects.

Correct lowering conceptually introduces hidden temporaries:

```text
t_receiver = eval(x)
t_index    = eval(i)
t_old      = getIndex(t_receiver, t_index)
t_rhs      = eval(rhs)
t_new      = send(t_old, +, t_rhs)
setIndex(t_receiver, t_index, t_new)
```

while preserving exact order.

## 5. Short-circuit constructs

For `and`/`or` or sacred-selector inlining, the skipped branch is semantically significant:

```text
false and sideEffect()   // sideEffect is not evaluated
true  or  sideEffect()   // sideEffect is not evaluated
```

If surface semantics are message-based, intrinsic optimization must preserve the send's lazy behavior.

## 6. Conditional control and blocks

If conditionals are block-taking sends, distinguish:

- evaluation of condition/receiver;
- construction of block arguments;
- invocation of exactly selected block.

Both closures may be constructed even though only one body executes. Capture/allocation can be immediate; body effects remain latent.

## 7. Collection literals and spread

For:

```phalcom
[a(), *b(), c()]
```

specify evaluation of `a`, evaluation of spread source `b`, timing of iteration/exhaustion, and evaluation of `c`.

If spread is lazy at one layer and eager in literal construction, state where exhaustion occurs. Errors from unbounded sources are part of observable order.

## 8. Labeled argument storage is not evaluation order

A runtime may canonicalize arguments by labels for lookup. That must not reorder evaluation unless the language says so.

```text
evaluation sequence: lexical source order
argument association: selector/parameter mapping
storage order: implementation choice
```

## 9. Exception cut-off

If an earlier subexpression throws, later subexpressions do not execute.

```phalcom
receiver(arg1(), arg2())
```

If `arg1()` throws, `arg2()` must not run under ordinary sequential semantics.

## 10. Yield points

Yield does not change lexical order inside the fiber, but another fiber may mutate shared state before continuation resumes:

```text
read shared x
call mayYield()
read shared x again
```

The two reads need not match.

## 11. Optimizer constraints

Reordering is safe only if semantics/analysis proves sufficient non-interference. Typical blockers include state reads/writes, exceptions, allocation identity, IO, reflection, callbacks, yield/cancellation, and timing-sensitive external operations.

Same returned values are insufficient.

## 12. Evaluation-order test pattern

Use a trace builder in fixtures, then assert both result and exact trace. This catches order drift across compiler, VM, optimizer, and native paths.

## 13. Competency checks

1. Why can selector canonicalization reorder storage but not evaluation?
2. What hidden temporaries are required to lower compound subscript assignment safely?
3. How can closure construction be observable even when its body is not invoked?
4. Which optimizer proof is needed before swapping two calls?
5. Where should an exception stop an argument-evaluation sequence?
