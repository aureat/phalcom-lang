# Operational Semantics

Operational semantics describes programs by the computation steps they perform or by their final outcomes. For Phalcom it is the most practical foundation because message dispatch, mutable state, non-local control, modules, and fibers all have operational character.

## 1. Big-step semantics

A store-aware big-step judgment can be written:

```text
ρ; σ ⊢ e ⇓ o; σ'
```

where `o` is an explicit outcome.

For a literal:

```text
------------------------------ LIT
ρ; σ ⊢ 42 ⇓ Normal(42); σ
```

For a lexical variable:

```text
ρ(x) = ℓ      σ(ℓ) = v
-------------------------- VAR
ρ; σ ⊢ x ⇓ Normal(v); σ
```

For assignment:

```text
ρ; σ ⊢ e ⇓ Normal(v); σ₁
ρ(x) = ℓ
σ₂ = σ₁[ℓ ↦ v]
-------------------------------- ASSIGN
ρ; σ ⊢ x = e ⇓ Normal(v); σ₂
```

If evaluating `e` returns an abrupt outcome, assignment propagates it without mutating `x`.

## 2. Sequencing and abrupt propagation

```text
ρ; σ ⊢ s₁ ⇓ Normal(v₁); σ₁
ρ; σ₁ ⊢ s₂ ⇓ o₂; σ₂
-------------------------------- SEQ-NORMAL
ρ; σ ⊢ s₁; s₂ ⇓ o₂; σ₂
```

and:

```text
ρ; σ ⊢ s₁ ⇓ Abrupt(a); σ₁
-------------------------------- SEQ-ABRUPT
ρ; σ ⊢ s₁; s₂ ⇓ Abrupt(a); σ₁
```

This pattern scales to returns, throws, break/continue, and cancellation.

## 3. Message-send decomposition

Do not define send as "call VM dispatch." Expose semantic stages.

For:

```phalcom
receiver.foo(a, label: b)
```

conceptually:

1. evaluate `receiver`;
2. evaluate arguments in specified lexical order;
3. construct canonical selector from call shape;
4. choose dispatch side and lookup start;
5. resolve method under access context;
6. create invocation environment with preserved receiver and parameter bindings;
7. evaluate method body;
8. return/propagate its outcome.

A schematic rule:

```text
ρ; σ  ⊢ r     ⇓ Normal(vr); σ₁
ρ; σ₁ ⊢ args  ⇓ Normal(vs); σ₂
selector(callShape) = s
lookup(classOf(vr), s, access, start=classOf(vr)) = m
invoke(m, receiver=vr, vs, σ₂) ⇓ o; σ₃
------------------------------------------------------- SEND
ρ; σ ⊢ r.s(args) ⇓ o; σ₃
```

Every premise has observable consequences. A compiler may fuse them, but may not change meaning.

## 4. Argument evaluation relation

It is often useful to define argument-list evaluation separately:

```text
ρ; σ ⊢ [] ⇓ Normal([]); σ
```

```text
ρ; σ  ⊢ e      ⇓ Normal(v); σ₁
ρ; σ₁ ⊢ rest   ⇓ Normal(vs); σ₂
-------------------------------- ARG-CONS
ρ; σ ⊢ e,rest ⇓ Normal(v::vs); σ₂
```

Abrupt completion of `e` prevents evaluation of `rest`. Dynamic packs can extend this relation with cursor creation/iteration while preserving lexical sequencing.

## 5. Miss handling

If lookup fails, specify the language-level path:

```text
lookup(...) = miss
message = reifyMessage(receiver, selector, arguments, sourceContext)
invoke(doesNotUnderstand, receiver, [message], σ) ⇓ o; σ'
---------------------------------------------------------- SEND-MISS
...
```

The exact Phalcom dNU contract should be substituted here. The critical point is that miss is an explicit semantic branch, not a Rust `None` accidentally converted into a generic VM error.

## 6. Access denial

Lookup miss and access denial are semantically distinct unless the language intentionally merges them.

Possible result space:

```text
LookupResult = Found(Method)
             | Miss
             | AccessDenied(Method)
```

Define whether an inaccessible method blocks superclass lookup, raises immediately, or is hidden as if absent. Then make direct, cached, and reflective paths obey the same rule.

## 7. `super` send

`super` does not evaluate to the superclass object.

For current receiver `self = v` and lexical defining class `C`:

```text
lookupStart = superclass(C)
receiver    = v
```

Then:

```text
lookup(lookupStart, selector, access, ...) = m
invoke(m, receiver=v, args, ...) ...
```

Compiling `super.foo()` as `Superclass.foo()` is wrong because it changes receiver identity and likely dispatch side.

## 8. Class-side send

A class object is a runtime value. If `v = C` is the class object:

```text
classOf(v) = metaclassOf(C)
```

Ordinary lookup begins there. Class-side inheritance therefore uses ordinary lookup machinery over the metaclass hierarchy.

## 9. Small-step semantics

A transition system is better for ordering and suspension:

```text
C = ⟨control, env, store, kont, modules, scheduler⟩
C → C'
```

Example left-to-right binary evaluation:

```text
⟨e₁ op e₂, ρ, σ, κ⟩
→
⟨e₁, ρ, σ, BinLeft(op, e₂, ρ, κ)⟩
```

When `e₁` becomes `v₁`:

```text
⟨v₁, ρ, σ, BinLeft(op, e₂, ρ₂, κ)⟩
→
⟨e₂, ρ₂, σ, BinRight(op, v₁, κ)⟩
```

This makes order structural rather than relying on prose.

## 10. Divergence

Big-step rules primarily describe terminating evaluations. For divergence use small-step infinite sequences, coinductive big-step semantics, or a separate divergence judgment.

Do not define loops only as "repeat until false" and then treat nontermination as outside semantics. Nontermination is a program behavior.

## 11. Exceptions

With explicit outcomes:

```text
Throw(v)
```

propagates until a handler consumes it. In a machine semantics, handler frames can perform the same role.

Specify:

- what expressions have already executed before throw;
- cleanup during unwinding;
- handler matching;
- source/stack trace timing;
- interaction with non-local return and cancellation.

## 12. Allocation

A semantic allocation rule needs fresh identity:

```text
o ∉ dom(H)
H' = H[o ↦ Object(class=C, fields=defaults)]
-------------------------------------------- ALLOC
H ⊢ new C ⇓ o; H'
```

Fresh means distinct from every live/observable object identity, not necessarily monotonically increasing.

## 13. Field access

Receiver-local field syntax should be specified separately if it is not a message send:

```text
fieldLoc(selfObject, FieldId(C,_x)) = ℓ
σ(ℓ)=v
------------------------------- FIELD
...
```

That distinction matters for access, reflection, optimization, and analyzer modeling.

## 14. Closures

Block construction:

```text
closure = ⟨code, capture(ρ), homeFrame, homeFiber⟩
------------------------------------------------ BLOCK
ρ; σ ⊢ |x| { body } ⇓ Normal(closure); σ
```

No body execution occurs at construction. Invocation creates parameter bindings and executes with captured environment/storage.

## 15. Loops

Specify condition evaluation frequency and control targets:

```text
condition evaluated before each iteration
continue targets next condition check
break consumes loop target
return/throw/non-local return propagate past loop unless caught
```

If surface control is message/block based, distinguish user-visible send semantics from a possible core control form used to explain or optimize it.

## 16. Scheduler transitions

A yield transition can be modeled:

```text
⟨Yield(v), fiber=f, scheduler=χ⟩
   --yield(f,v)-->
⟨resumeValue?, fiber=f', scheduler=rotate(χ,f)⟩
```

Exact resumed values and queue policy depend on ratified fiber semantics.

## 17. Native transition

Model native code as a semantic boundary:

```text
native(name, receiver, args, world) -> outcome, world'
```

Its declared effects determine whether it may allocate, throw, callback, block, or yield. A Rust function signature alone does not define those language effects.

## 18. Implementation relation

The VM need not literally interpret these rules. It may use stack bytecode, inline caches, native primitives, and optimized paths. Correctness requirement is observational correspondence under permitted nondeterminism.

## 19. Common unsound shortcuts

- evaluating arguments in selector-storage order instead of lexical source order;
- converting `super` to a superclass receiver;
- executing block effects during block creation;
- representing method miss as implementation panic;
- skipping access checks in cached/reflective dispatch;
- treating recovery AST as executable value;
- assuming native primitive cannot yield/throw because its Rust signature does not encode that.

## 20. Competency checks

1. Write a send rule where the second argument throws. Which later stages must not occur?
2. Explain why a `super` call can invoke a superclass implementation while `self.class` still names the subclass.
3. Give an example where changing argument evaluation order changes output.
4. What freshness property must allocation satisfy if VM handles are recycled?
5. Why is an infinite small-step sequence preferable to "no derivation" when reasoning about divergence?
