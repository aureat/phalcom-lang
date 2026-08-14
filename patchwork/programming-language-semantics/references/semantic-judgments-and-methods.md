# Semantic Judgments and Methods

This reference explains the main mathematical forms used to specify programming languages and how to choose among them. The goal is not formal notation for its own sake; the goal is to expose assumptions that otherwise become accidental VM behavior.

## 1. A judgment is a claim under explicit assumptions

A judgment has a subject and a context. Examples:

```text
Γ ⊢ x ↦ b               name x resolves to binding b
Γ ⊢ e : T               expression e has type T
ρ; σ ⊢ e ⇓ v; σ'        e evaluates to v, changing store σ to σ'
C → C'                   machine configuration C takes one step
{P} c {Q}               c establishes Q when started in P
```

Read the turnstile as "under these assumptions, the following assertion holds." Keep different contexts distinct:

```text
Γ   static binding/type environment
ρ   runtime environment
σ   mutable store/heap abstraction
κ   continuation/control stack
μ   module state
χ   scheduler/fiber state
```

Using one undifferentiated `Env` for all of these hides important distinctions.

## 2. Static semantics

Static semantics covers rules that do not require ordinary program execution. In Phalcom this can include lexical declaration/shadowing, selector formation, declaration well-formedness, access context, optional type checking, protocol conformance, definite assignment, and exhaustiveness in checker modes.

Example name-resolution rule:

```text
lookupScope(Γ, x, p) = b
------------------------- NAME
Γ; p ⊢ x ↦ b
```

`p` matters when declarations become visible only after their source position or when recovery permits incomplete declarations.

Static semantics must not silently answer a dynamic question. Resolving `x` to a lexical binding does not prove the runtime value's class.

## 3. Dynamic semantics

Dynamic semantics specifies execution. Three useful styles are common.

### Big-step / natural semantics

```text
ρ; σ ⊢ e ⇓ o; σ'
```

This states the final outcome of evaluating `e`. It is concise for ordinary deterministic evaluation.

Use an explicit `Outcome` rather than returning only a value:

```text
Outcome = Normal(Value)
        | Return(Target, Value)
        | Throw(Value)
        | Break(Target)
        | Continue(Target)
        | Yield(Value)
        | Cancel(Value)
```

A big-step semantics that returns only `Value` usually hides exactly the control behavior that becomes difficult in blocks, exceptions, and fibers.

### Small-step / transition semantics

```text
⟨e, ρ, σ, κ⟩ → ⟨e', ρ', σ', κ'⟩
```

Small-step is better when evaluation order matters, divergence must be represented naturally, fibers interleave, exceptions/non-local control move through frames, or compiler correctness is phrased as simulation.

### Abstract-machine semantics

Instead of rewriting syntax directly, model the runtime's conceptual state:

```text
State = Control × Environment × Store × Continuation
```

A CESK-like machine is often closer to an implementation while remaining representation-independent. For Phalcom, frames, block home activations, send continuations, and fiber stacks make this especially useful.

## 4. Axiomatic semantics

Hoare logic describes state relations rather than execution steps:

```text
{P} c {Q}
```

This is excellent for contracts and static proving, but poor as the primary definition of message dispatch or scheduler behavior.

For a method:

```text
requires P(self, args)
ensures  Q(self, args, result, oldState, newState)
```

A prover can generate verification conditions from this, but the meaning of method calls, exceptions, mutation, and reflection still comes from the dynamic semantics.

## 5. Trace semantics

When external effects matter, values alone are insufficient. Use traces:

```text
C --print("x")--> C'
C --read(fd, bytes)--> C'
C --yield(fiber)--> C'
```

Then an execution produces:

```text
τ = [event₁, event₂, ...]
```

Compiler correctness can compare observable traces rather than internal state.

## 6. Denotational perspective

A denotational semantics maps syntax to mathematical meanings compositionally:

```text
⟦e⟧ : Env → Store → Result
```

It is powerful for equivalence and compositionality, but a full denotational model for Phalcom's mutable objects, reflection, exceptions, modules, and fibers would be expensive. Use denotational reasoning locally when useful rather than requiring it as the first formalization.

## 7. Inference rules versus algorithms

A semantic rule is declarative:

```text
Γ ⊢ e₁ ⇓ v₁
Γ ⊢ e₂ ⇓ v₂
applyPlus(v₁, v₂) ⇓ v
------------------------
Γ ⊢ e₁ + e₂ ⇓ v
```

An implementation algorithm is procedural:

```text
v1 = eval(e1)
v2 = eval(e2)
return send(v1, #+(v2))
```

The algorithm is correct only if it implements the rule and all side conditions. Do not let an efficient algorithm become the normative rule accidentally.

## 8. Derived forms

A derived form is syntax whose semantics is defined by translation into a core language:

```text
surface construct D[e]
      desugars to
core construct C[e]
```

A valid derivation requires:

```text
Obs(D[e]) = Obs(C[e])
```

for the observations the language exposes. Translation must preserve evaluation count/order, source-level control, access context, reflection expectations, and error behavior.

## 9. Determinism and nondeterminism

For a relation `→`, deterministic means:

```text
C → C₁ and C → C₂  implies  C₁ = C₂
```

Single-fiber Phalcom evaluation may be deterministic given a fixed external world. IO, clocks, randomness, reflective mutation by other fibers, and scheduler choice introduce nondeterminism. Model those inputs/events explicitly rather than vaguely calling the entire language nondeterministic.

## 10. Partial versus total semantics

Distinguish:

- terminates normally;
- terminates abruptly;
- diverges;
- gets stuck because semantics omitted a case;
- is dynamically erroneous by an explicit language rule.

A `MessageNotUnderstood` exception is not stuck if the language defines it. A VM panic due to an impossible tag is not a language exception.

## 11. Phalcom working pattern

For each feature, write at least these judgments or equivalent pseudocode:

```text
resolve(name, lexicalContext) -> SemanticTarget
value/eval(expression, machineState) -> Outcome
lookup(receiverClass, selector, accessContext, lookupStart) -> Method | Miss
invoke(method, receiver, args, machineState) -> Outcome
```

Then state how compiler and analyzer correspond to them.

## 12. Common errors

### Mixing representation with semantics

Bad: a closure is defined as a particular Rust struct.

Better: a closure denotes executable code paired with captured lexical storage and required control metadata; VM representation may vary.

### Hiding abrupt completion

Bad:

```text
eval(statement) -> Value
```

with host-language exceptions used internally for `return` and `throw`.

Better:

```text
eval(statement) -> Outcome
```

or a machine whose continuation frames make transfer explicit.

### Treating static approximation as execution

`ValueShape::Instance(Point)` is evidence used by tooling. It does not define that evaluating the expression produces a `Point` in every run unless the analysis has a corresponding sound guarantee.

## 13. Competency checks

1. Why is big-step semantics awkward for cooperative scheduling?
2. Why does `return` require an outcome or continuation target rather than a plain value?
3. When can a surface construct be specified solely as syntactic sugar?
4. Which observations are required before claiming two lowerings equivalent?
5. What is the difference between a dynamic error and a stuck configuration?
