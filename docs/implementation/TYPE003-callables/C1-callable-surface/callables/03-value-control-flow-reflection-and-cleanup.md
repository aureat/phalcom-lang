# Task Set 4 — Value Semantics, Local Return, Loop Values, Reflection Migration, and Callable Cleanup

**Project:** Phalcom
**Repository:** `aureat/phalcom-lang`
**Status:** Implementation-ready coding task-set
**Execution:** Coding task 3 of 3; requires Task Sets 2 and 3 complete
**Primary areas:** Unit/no-result semantics, final-expression compilation, constructor initializer rules, local return, NLR deletion, loop values, direct `while`, Block-object deletion, reflection moves, public protocol cleanup, exhaustive tests
**Goal:** complete the callable redesign and remove all transitional legacy machinery

---

## 1. Mission

Finish the semantic migration after the new callable hierarchy and invocation runtime are stable.

This task-set implements the ratified decisions:

```text
D1   every executable construct has a value, placement may remain contextual
D2   assignment → ()
D3   declaration → ()
D4   if/else → selected branch; one-armed if → ()
D5   loop value comes only from break value; normal completion → ()
D6   return is local to current Method/Closure; bare return = return ()
D7   brace-delimited code blocks establish lexical scope
D8   lexical bindings beat implicit-self resolution
D9   no BoundMethod rebinding API
D10  Closure labels rejected for now
D11  callWith(pack) = self(***pack)
D12  needed/discarded compiler value context
D13–D21 reflection/object-surface decisions
D22  @constructor generation + ordinary initializer Method semantics
D31  explicit Unit result semantics
D32  retire Function arity/name base reflection
D33  break value
D34  language while is direct lexical control flow
D37  legacy callable machinery is fully superseded
```

This task-set deletes the old non-local-Block architecture rather than renaming it.

---

## 2. Unit becomes the canonical no-result value

The runtime already has surface Unit `()`.

Add/use an explicit bytecode representation:

```text
Bytecode::Unit
```

(or an exactly equivalent direct opcode if naming conventions require another name).

Canonical distinction:

```text
Bytecode::Nil / None
    absence only

Bytecode::Unit / ()
    successful no-result / empty product
```

Do not use `None` for fallthrough, mutation success, empty control-flow results, or bare return.

---

## 3. Return bytecode invariant

Strengthen compiler/VM contract:

> Every ordinary `Return` executes with one explicit result value already on the stack.

Examples:

```phalcom
return
```

lowers conceptually to:

```text
Unit
Return
```

Empty Method/Closure body:

```text
Unit
Return
```

Final ordinary expression:

```text
compile expression
Return
```

The VM must not invent `None` when `Return` finds no value.

A stack underflow at `Return` is an internal compiler/VM invariant failure, not surface absence.

---

## 4. Needed/discarded value compilation

Generalize the existing `want_value` idea into a deliberate compiler context:

```text
ValueContext::Needed
ValueContext::Discarded
```

or equivalent.

The semantic rule is independent of physical stack materialization.

### 4.1 Assignment

Language assignment:

```phalcom
x = expr
obj.field = expr
obj.property = expr
```

evaluates to:

```phalcom
()
```

The assignment target operation must still evaluate `expr` exactly once and perform all side effects.

If the underlying setter Method returns another value, assignment syntax discards that result and produces Unit.

Internal `SetLocal`/`SetField` stack behavior may remain optimized; compiler expression semantics must hide any implementation return value.

### 4.2 Declarations

```phalcom
let x = expr
const x = expr
class X { ... }
```

when executable in a value-observable position produce Unit unless a separately specified declaration form intentionally produces a value.

Do not leak initializer values as declaration results.

### 4.3 Empty block/body

Produces Unit.

### 4.4 Optimization

In `Discarded` context, do not push Unit merely to pop it immediately.

In `Needed` context, produce Unit exactly once.

---

## 5. Final-expression semantics

Ordinary Method and Closure bodies return their final executable construct's semantic value.

Examples:

```phalcom
foo {
    42
}
```

returns `42`.

```phalcom
foo {
    const x = 42
}
```

returns `()`.

```phalcom
foo {
    x = 42
}
```

returns `()`.

Closure follows the same final-expression rule.

Retire any old specification/compiler branch in which brace-bodied named Methods discard the final expression.

---

## 6. `if` value semantics

Ratified:

```text
if condition { A } else { B }
    → selected branch value

if condition { A }
    → ()
```

The one-armed form produces Unit whether or not the branch executes.

This avoids implicit `T | Unit` behavior.

Compiler/inliner fast paths and deopt fallbacks must agree.

Any old one-armed sacred path producing `Some`/`None` as its direct language-level `if` result must be updated to the new semantics.

Do not globally change explicit `Bool#ifTrue` library messages unless required by the language-control-flow desugar; distinguish language `if` from callable library protocol where necessary.

---

## 7. Loop value semantics

Ratified:

```text
normal termination → ()
break               → ()
break expression    → expression value
```

Per-iteration body values are discarded.

### 7.1 AST

Change:

```text
Break { range }
```

to a structured optional operand, conceptually:

```rust
Break {
    value: Option<Expr>,
    range: SourceRange,
}
```

Parser:

```phalcom
break
break expression
```

Bare `break` is semantically `break ()`.

### 7.2 Compiler strategy

Use loop result storage only when required.

A straightforward implementation:

```text
initialize hidden loop-result = ()
...
break expr:
    evaluate expr
    store result
    jump exit
...
normal exit:
    leave default ()
...
exit:
    load result if Needed
```

In `Discarded` context:

- `break expr` must still evaluate `expr` for side effects;
- result storage may be optimized away.

### 7.3 Control-flow typing model

`break`, `continue`, `return`, and `throw` are terminating at their source position (conceptually `Never`) even though the enclosing loop/Method may produce a normal value elsewhere.

No public `Never` runtime class is required by this task.

### 7.4 Closure boundary

A `break` inside a real Closure cannot target a loop in an outer Method/Closure frame.

It is an out-of-loop compile error in that Closure's function context.

Compiler-generated synthetic artifacts must preserve source lexical loop control and must never turn lexical `break` into runtime cross-Closure control.

---

## 8. Language `while` becomes direct lexical compiler control flow

Retire language semantics defined as:

```text
condition Closure .whileTrue(body Closure)
```

Source:

```phalcom
while condition {
    body
}
```

must lower directly to loop bytecode/control-flow in the current executable body.

This is required for local:

```phalcom
break value
continue
```

without recreating non-local control transfer.

### 8.1 Higher-order `whileTrue`

If retained as a library combinator:

```phalcom
predicate.whileTrue(body)
```

move it to `Function` when its semantics depend only on invoking complete callables, not lexical Closure representation.

It is not the semantic definition of source `while`.

### 8.2 GuardBlock

Audit `GuardBlock` and Block override-epoch machinery.

Do not mechanically rename it `GuardClosure`.

Remove it if it only exists for old source-control-flow lowering.

Keep an optimization only if its semantic guard remains necessary for explicit Function/Closure library calls.

---

## 9. Constructor generation and initializer return semantics

Canonical conceptual transformation:

```text
@constructor source method
    → generated class-side factory Method
    → generated/hidden ordinary instance initializer Method
```

Factory:

```text
allocate/reuse instance
invoke initializer
discard initializer result
return instance
```

Initializer is an ordinary Method and therefore, purely at runtime, would return its final expression or explicit return value like any other Method.

### 9.1 Compiler restriction from `@constructor`

Because source marked with `@constructor` is being used as initialization code for a generated factory, reject:

```phalcom
return expression
```

inside that source constructor body.

This is a compile-time semantic restriction, not a special return opcode.

Allow:

```phalcom
return
```

which means:

```phalcom
return ()
```

from the generated initializer Method.

The factory ignores `()` and returns the allocated instance.

### 9.2 Final initializer expression

The initializer's final expression may evaluate to any value under ordinary Method semantics; the generated factory discards it.

This matters conceptually and for reflective exact invocation of the generated initializer if such internal reflection is ever permitted.

Do not hard-code “initializer always returns self” into ordinary Method execution.

---

## 10. Remove non-local return completely

All source `return` inside a Closure returns from that Closure activation.

Compile Closure return with ordinary:

```text
Bytecode::Return
```

Delete:

```text
Bytecode::ReturnNonLocal
```

and all compiler branching based on “is block” solely for return target selection.

### 10.1 Remove home-frame machinery

Delete or retire all machinery used only for non-local Block return:

```text
BlockObject.home_frame_token
CallFrame.home_frame_token
FrameToken generation checks used only by NLR
DeadFrameError NLR behavior
NLR stack-unwind repair
native call frame-count repair specific to NLR
```

Audit `FrameToken` / frame generation before deleting globally; if another independent feature uses it, preserve only that independent use.

### 10.2 Error/catch code

Update `on`/`ensure`/error-handling primitives and comments that contain special NLR branches.

A local Closure `return` is an ordinary successful return from the nested callable, not an unwind through an enclosing native combinator.

No escaped Closure can fail because its old home Method frame is dead.

Remove DeadFrameError tests tied solely to this behavior.

---

## 11. Collapse transitional Block representation

After NLR removal, delete the transitional public/internal Block wrapper.

Target surface/runtime relationship:

```text
Object::Closure(ClosureObject)
    → surface Closure
```

Remove:

```text
Object::Block
BlockObject
heap/block.rs
block_class
```

unless `block_class` was already removed in Task Set 2.

`ClosureObject` remains a valid internal compiled-code representation for bytecode Methods as well; documentation/comments must distinguish:

```text
MethodKind::Closure
```

meaning “Method implemented by compiled closure bytecode” from the surface class `Closure`.

A purely internal rename such as `CompiledClosure` is optional and must not be mixed into this task unless it clearly reduces confusion without expanding risk.

---

## 12. Function/Closure callable-protocol cleanup

Reclassify old `primitive/block.rs` selectors by semantics.

### 12.1 Function-level operations

Operations that require only “invoke this complete callable” should live on Function.

Candidates include:

```text
call
callWith
whileTrue
on
ensure
```

subject to exact existing semantics.

### 12.2 Closure-only operations

Keep on Closure only operations that truly depend on lexical Closure representation/capture.

Do not keep a Closure-only API merely because the old implementation file was named `block.rs`.

### 12.3 Obsolete operations

Remove APIs tied to non-local-return/home-frame behavior.

### 12.4 `Function#arity` and `Function#name`

Ratified: remove them from the base Function protocol rather than inventing misleading scalar semantics.

Do not replace them with new reflection in this task.

Callable reflection is a separate later design.

Update tests that relied on scalar arity/name.

---

## 13. Reflection surface migration

Implement the ratified boundary.

### 13.1 Move to Behavior

Remove from Object and install on Behavior:

```text
methodFor
respondsTo
```

`Behavior#methodFor(selector)`:

- performs normal inherited lookup;
- exact lookup remains authoritative;
- rest fallback follows the common resolver where applicable;
- returns the exact reified Method or absence according to existing reflection conventions;
- does not invoke dNU.

`Behavior#respondsTo(selector)`:

- tests the same normal lookup/resolution model;
- does not count dNU as responding.

Do not expand the unresolved reflection API beyond these ratified moves.

### 13.2 Keep on Behavior

Keep:

```text
name
superclass
methods
```

`methods` remains direct-dictionary-only.

### 13.3 Retire from Object

Remove:

```text
Object#name
Object#methodFor
Object#respondsTo
Object#class=(put)
Object#isA
```

### 13.4 Keep on Object

Keep:

```text
class
is
isExactly
perform
doesNotUnderstand
```

plus ordinary universal protocol such as equality/hash/string conversion.

`perform` remains receiver-specific dynamic execution and must use Task Set 3's common shape-aware activation.

`doesNotUnderstand` remains the receiver miss hook.

### 13.5 Freedom of domain selector `name`

Add a regression proving a user class may define ordinary `name` without shadowing a reflective class-name method inherited from Object.

Class-name reflection is:

```phalcom
obj.class.name
```

---

## 14. Lexical binding precedence

Preserve/verify ratified resolution:

```text
local/parameter
captured lexical binding
module/global binding as applicable
then implicit self message
```

A callable lexical binding named `helper` wins over an implicit `self.helper` selector.

Add regressions inside Methods and nested Closures.

Do not introduce runtime lookup to resolve this; it is compiler name resolution.

---

## 15. BoundMethod rebinding

Do not add:

```text
BoundMethod#bind
BoundMethod#rebind
```

as an initial public API.

A BoundMethod remains:

```text
exact Method + one receiver
```

If reflection exposes the underlying Method in the future, rebinding may be expressed through that Method; do not invent it here.

---

## 16. None/Unit semantic audit

Search native/core code for returns of `None` that mean “successful operation completed with no meaningful payload”.

Classify every occurrence.

Keep `None` only where semantics mean absence, such as:

```text
lookup miss
optional missing value
iterator exhaustion marker where explicitly specified
```

Change to Unit where semantics mean:

```text
mutation completed
side effect completed
empty executable result
normal loop completion
bare return
```

Do not globally replace `none_value()`.

Iterator protocols may legitimately use None as an exhaustion sentinel; preserve those semantics.

---

## 17. Tests — body/value semantics

Add tests for:

```phalcom
method { 42 }                       // 42
method { const x = 42 }             // ()
method { x = 42 }                   // ()
|| { 42 }                           // 42
|| { const x = 42 }                 // ()
|| {}                               // ()
```

Test bare/local return:

```phalcom
return
return value
```

Nested Closure return must not exit enclosing Method.

Escaped Closure containing return remains callable after creator returns.

No DeadFrameError.

---

## 18. Tests — loops

Test each source loop supported by the language:

```text
normal completion → ()
break → ()
break expression → value
```

Test:

- side effects in discarded break expressions;
- nested loops bind break to innermost loop;
- continue preserves loop result state;
- break inside nested real Closure is rejected;
- `if` inside loop preserves lexical break under inlined and deopt compiler paths;
- direct `while` supports Fiber yield according to ordinary bytecode execution, with no native Closure combinator boundary.

---

## 19. Tests — constructors

Add:

```text
factory always returns allocated instance
initializer final expression does not replace factory result
bare return ends initializer early and factory still returns instance
return value in @constructor body is compile error
ordinary non-constructor factory Method may return arbitrary final value
ordinary exact initializer Method semantics remain ordinary internally
```

Test superclass constructor chaining and instance reuse.

---

## 20. Tests — reflection

Verify:

```text
obj.methodFor        → absent
obj.respondsTo       → absent
obj.name             → user/domain selector if defined

obj.class.methodFor  → works
obj.class.respondsTo → works
obj.class.name       → class name
```

Verify inherited `methodFor`.

Verify `methods` remains direct-only.

Verify `class=(put)` absent.

Verify `is` and `isExactly` remain.

Verify `isA` removed.

Verify `perform` and dNU still work after moves.

---

## 21. Exhaustive cleanup searches

Run repository searches for stale live implementation references:

```text
Block
BlockObject
block_class
block_call
block_arity
block_name
home_frame_token
FrameToken
ReturnNonLocal
DeadFrameError
non-local return
isClosed
Method is Function
BoundMethod surface class Block
callWith List
Function arity
Function name
args...
```

Not every historical ADR occurrence should be edited.

Classify results:

```text
live code                    → update/remove
current normative spec       → Task Set 1/new folder is canonical; broader docs only if sanctioned
historical ADR/as-built      → preserve unless explicitly marked superseded
tests/fixtures               → update
comments describing live code→ update
```

`args...` search must distinguish unrelated ellipsis/range uses; only remove spread/rest claims.

---

## 22. Required verification

At minimum run:

- formatter;
- Rust unit tests;
- `phalcom-ast` parser/lexer tests;
- compiler tests;
- VM dispatch tests;
- F.1/F.2/F.3 argument-pack/rest suites;
- Fiber tests;
- inheritance/super tests;
- access-control tests;
- constructor tests;
- reflection tests;
- bootstrap/core module compilation;
- examples/benchmarks that are part of CI;
- full workspace test command used by the repository.

No completion claim until the full relevant suite is green.

---

## 23. Final architectural invariants

At completion, all of these must be true:

```text
Object
├── Method
└── Function
    ├── Closure
    ├── BoundMethod
    └── Family
```

```text
Method is not Function.
BoundMethod is exact Method + receiver.
Family does not use dNU as its call router.
Closure return is local.
There is no Block public class.
There is no non-local return.
There is no DeadFrameError semantics for escaped Closures.
Function call uses *** only as complete rest syntax.
args... is not spread/rest.
Closure rest is positional-only.
Unit means no-result.
None means absence.
Assignment and declaration evaluate to Unit.
One-armed if evaluates to Unit.
Loops produce Unit unless break supplies a value.
Language while is direct lexical control flow.
@constructor factories return allocated instances while generated initializer
Methods retain ordinary Method execution semantics.
Object reflection is narrowed; behavior reflection lives on Behavior.
```

---

## 24. Completion gate

Task Set 4 is complete when:

- Unit/no-result semantics are live end-to-end;
- final-expression Method/Closure semantics are correct;
- constructor compiler restriction/generator semantics are correct;
- `break value` and direct lexical `while` are implemented;
- ReturnNonLocal/home-frame/Block wrapper machinery is gone;
- Function protocol no longer exposes old scalar arity/name;
- reflection surface is migrated exactly as ratified;
- Block terminology is absent from live public/runtime callable semantics;
- no spread/rest syntax uses `...`;
- all relevant tests and bootstrap pass.
