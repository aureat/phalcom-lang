# 02 — Closures and Control Flow

Variables that outlive frames; jumps that leave frames. The through-line: *a frame is a
lifetime, and both halves of this file are about violating it on purpose.*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — Open and closed upvalues

```lua
local function counter()
  local n = 0
  return function() n = n + 1; return n end
end
```

While `counter` is running, `n` is a stack slot. After it returns, `n` must survive.

1. Describe what physically happens to `n` at the moment `counter` returns, and say where a
   write through the closure lands before and after that moment.
2. Lua keeps a per-coroutine list of open upvalues sorted by stack level and *searches* it
   when creating a closure. Why is a search required — why can the compiler not simply name
   the upvalue object to use?
3. The alternative is to heap-allocate every local that is ever captured, at declaration.
   Name what that costs and what it buys.

### Q2 — The loop variable, three languages

```js
for (var i = 0; i < 3; i++) fns.push(() => i);   // 3, 3, 3
for (let i = 0; i < 3; i++) fns.push(() => i);   // 0, 1, 2
```
```python
fns = [lambda: i for i in range(3)]              # 2, 2, 2
```

Go behaved like `var` until 1.22.

1. What must `let` do per iteration that `var` does not? Describe the codegen, and be
   specific about what happens to the update expression `i++`.
2. Python's fix is `lambda i=i: i`. Explain why a default argument fixes it, and what that
   tells you about Python's capture model.
3. Go's change is a semantic break in a language with a compatibility promise. How did they
   ship it, and why was that mechanism available to Go and not to JS?

### Q3 — Flat closures and the chain walk

Two representations. **(A)** Each closure points at its enclosing environment; a variable is
(depth, index) and access walks `depth` links. **(B)** Each closure holds a flat array of
upvalue pointers; a variable is one index and access is one indexed load. Lua and Wren use
(B).

1. Give the cost each representation pays and say where in the program's lifecycle it lands.
2. Under (B), an inner function capturing a variable three levels out forces the intermediate
   functions to do something. What, and what is the pathological case?
3. Which does an optimizing compiler prefer, and why does the answer flip when closures are
   created constantly but called once?

### Q4 — Funargs, up and down

Downward: `sort(xs, |a, b| a.key < b.key)` — the closure dies with the call. Upward:
`return |x| x + n` — it does not.

1. Which one permits stack allocation of the environment, and what analysis is required to
   prove it safe?
2. Rust encodes the distinction in the type system instead of analyzing it. Say how, and what
   that buys over an analysis.
3. A language permitting only downward funargs gets a real implementation benefit. Name what
   it forecloses, and name the bug class it invites in a language that keeps the
   representation but drops the restriction.

### Q5 — Returning from a frame that is gone

```smalltalk
maker
    ^ [ :x | ^x ]     "a block containing a non-local return"
```

Call `maker`, keep the block, evaluate it later. Smalltalk raises a `cannotReturn`-family
error; Ruby's proc equivalent raises `LocalJumpError`.

1. What does `^` inside a block mean, and what must the block carry to implement it?
2. Describe the check that produces the error. Why must it be a runtime check rather than a
   compile-time rejection?
3. Ruby ships both `proc` (returns from the enclosing method) and `lambda` (returns from
   itself). Argue why a language would want both, then say which you would cut and what
   breaks.

### Q6 — Break through a native frame

```ruby
items.each { |x| return x if x.matches? }
```

`each` is implemented in C and calls back into the interpreter.

1. `return` must transfer control past a native frame that is mid-loop. Say why "return a
   special value and let `each` check it" fails as a general answer.
2. Two implementations: unwind with an exception-like mechanism (MRI uses setjmp-based tags),
   or push a frame and let one flat interpreter loop observe the signal. Give the cost of
   each.
3. `break` and `return` from a block need different targets. Say what each targets, and why
   conflating them is a correctness bug rather than a nicety.

### Q7 — What proper tail calls actually promise

```
def even(n) = n == 0 ? true  : odd(n - 1)
def odd(n)  = n == 0 ? false : even(n - 1)
even(10_000_000)
```

Scheme guarantees this runs in constant space. Lua does. Java does not.

1. State the guarantee precisely — what is it a guarantee *about*, and what is it explicitly
   not?
2. The JVM has never shipped proper tail calls despite repeated proposals. Give two concrete
   reasons rooted in the platform rather than in effort.
3. Distinguish proper tail calls from "the compiler turns self-recursion into a loop". Give a
   program where the second helps and the first is required.

### Q8 — Trampolines and CPS

You must run mutual recursion to unbounded depth on a host with no tail calls — the JVM, or a
tree-walking interpreter that recurses natively for each user-level call.

1. Describe a trampoline and state its per-call cost honestly.
2. CPS conversion makes every call a tail call. Explain how that helps, and say exactly what
   it moves onto the heap.
3. CPS and garbage collection interact in a specific way. Name it, and name a runtime whose
   GC design makes CPS-shaped code affordable.

### Q9 — Blocks as objects, lambdas as values

Smalltalk: a block is an object, invoked by sending `value`, `value:`, `value:value:` —
ordinary sends, arity-discriminated by selector. Ruby: a block is not an object until you
capture it with `&`. ML/JS/Rust: functions are values with call syntax.

1. What does blocks-as-objects buy at the language level? Name the consequence for
   control-flow constructs specifically.
2. What does it cost at the implementation level, and what optimization is mandatory to make
   it viable at all?
3. Ruby's blocks are deliberately *not* values in the common case. What does that
   non-uniformity buy, and what does it cost?

### Q10 — Inlining a message someone might override

A compiler inlines `cond ifTrue: [...] ifFalse: [...]` into a conditional jump whenever both
arguments are literal blocks. Then a user defines `ifTrue:ifFalse:` on their own class and
passes an instance as `cond`.

1. What has the compiler assumed, and what does the runtime need so the assumption is not a
   soundness bug?
2. Smalltalk sends `mustBeBoolean` when a jump instruction meets a non-Boolean. Say what that
   message *is* in optimization vocabulary.
3. In an open-world language the same problem recurs at every inlined call. Describe the
   general mechanism and the two pieces of metadata it requires.

### Q11 — Cleanup versus early exit

```java
try { return compute(); } finally { return 0; }   // returns 0; a pending exception vanishes
```
```go
for _, f := range files { defer f.Close() }        // nothing closes until the function returns
```

1. Explain how a `return` in `finally` can discard a pending exception, and what that tells
   you about the unwinder's internal state.
2. Java originally compiled `finally` with a bytecode subroutine (`jsr`/`ret`) and later
   switched to duplicating the handler. Why the switch, and what does duplication cost?
3. Go's `defer` is function-scoped, not block-scoped. Give the argument for that choice and
   the bug it causes; then say what Go did to make the common case cheap.

### Q12 — Labelled break is not a goto, until it is

```java
outer:
for (...) { for (...) { if (found) break outer; } }
```

Java bans `goto` but ships labelled `break` and `continue`.

1. In the simple case, what does labelled break compile to? Now put a `try`/`finally` between
   the label and the break. What does it compile to *then*, and what changed conceptually?
2. Why is labelled break acceptable to designers who banned `goto`? Give the structural
   property, not the aesthetic one.
3. A verifier or a structured IR must accept labelled break and reject arbitrary goto. What
   graph property separates them, and where does this matter outside Java?

### Q13 — Arity, partial application, and the unknown callee

Haskell: `f :: Int -> Int -> Int`, and `map (f 1) xs`. `f 1` is a value.

1. What object represents `f 1` at runtime, and what must the call sequence do when the
   number of arguments supplied does not match the callee's arity?
2. GHC moved from "push/enter" to "eval/apply". State each in a sentence, and say which side
   pays for the arity mismatch.
3. JS `Function.prototype.bind` and Python's `functools.partial` address the same need
   dynamically. Name a cost that shows up in the *engine*, not in the user's code.

### Q14 — Shared cells and the capture-by-value alternative

```js
function make() { let n = 0; return [() => n++, () => n]; }
```

Two closures, one variable. Java would reject capturing a mutable local outright.

1. What must the representation guarantee, and what does that force about where `n` lives
   once *either* closure is created?
2. Java requires captured locals to be final or effectively final and captures by value. Say
   what that forecloses and what it buys — include the concurrency argument.
3. Capture by reference keeps an environment alive. Name the leak pattern this causes and the
   mitigations real implementations use.

### Q15 — The closure that outlived its stack

A VM supports lightweight tasks, each with its own value stack. A task fails with an uncaught
error; the runtime tears the task down and releases its stack. A block created inside that
task had already been stored in a global. Calling it later reads garbage or crashes.

1. Name the invariant that was violated, and say why the *normal* return path never violates
   it.
2. Why is this bug in the same family as "a `finally` that does not run during an abnormal
   exit"?
3. Give a fix that does not require finding every teardown path, and say what it costs.

### Q16 — Dynamic scope, and why it is not a closure

Common Lisp special variables, Emacs Lisp's default binding, Ruby's `$~`, thread-locals,
Python's `contextvars`.

1. Why can dynamic binding not be implemented as lexical capture? Say what the lookup is a
   function of.
2. Two implementations: deep binding (search a chain of active bindings) and shallow binding
   (one cell per name, saved and restored around each scope). Give the cost profile of each,
   and name the operation that decides between them.
3. Thread-locals broke when coroutines arrived, and Python introduced `contextvars`. Explain
   the failure precisely, and say what a correct implementation must do at a suspension point.

### Q17 — Recursion depth as a language-level error

CPython raises `RecursionError`. A naive tree-walking interpreter written in C or Rust
segfaults instead.

1. Why does an interpreter that re-enters its own eval loop per user-level call have trouble
   turning deep recursion into a catchable error?
2. CPython historically enforced a *counter*, not a measurement. Name two ways a counter is
   wrong in opposite directions, and say what recent CPython changed.
3. A flat interpreter loop with a heap-allocated frame stack has an easier time. State what
   its limit becomes, and the new failure mode it introduces.

---

## Answers

### A1 — Open and closed upvalues

**1.** Before the return, the closure holds an **open upvalue**: a small heap object whose
pointer field points *at the live stack slot* holding `n`. Reads and writes through the
closure go through that pointer to the stack slot, so the running function's `n` and the
closure's `n` are literally the same storage — nothing needs synchronizing. At return, the
frame's slots are about to be reused, so the runtime **closes** the upvalue: it copies the
value out of the stack slot into a field inside the upvalue object itself and repoints the
pointer at that internal field. The closure's access code is byte-identical before and after
— one indirection either way. That uniformity is the entire trick: `GETUPVAL` never has to
know or check which state it is in.

**2.** Because upvalue objects must be **shared between closures capturing the same variable**.
Two closures created in the same scope over the same `n` must observe each other's writes, so
the second one must find the *existing* open upvalue rather than allocate a fresh one. The
compiler knows which stack slot is involved, but not whether an upvalue object for that slot
already exists — that depends on runtime control flow, such as a loop that creates a closure
per iteration or a branch that created one earlier. Hence: search the open list by stack
address, reuse on a hit, create on a miss. Keeping the list sorted by stack level makes the
search early-exit, and makes closing a frame a prefix operation — "close everything at or
above this level".

**3.** Costs: one allocation per captured local *per entry to the scope*, even when the
closure never escapes and even when the function returns immediately; plus an indirection on
every access to that local from the owning function itself, not merely from closures. A
function that captures a variable it then reads a million times in a loop has just turned its
hottest local into a heap object. Buys: no open/closed distinction, no open list, no
close-on-return work, and — the underrated one — no interaction with any code path that
abandons a frame abnormally, which is a genuine correctness benefit (see Q15). It is the
design most compilers choose when they have escape analysis available to undo the cost in the
cases that matter.

### A2 — The loop variable, three languages

**1.** Per iteration, `let` creates a **fresh binding** — a new environment record for the
body — and the loop must copy the current value *in* at the top and copy the possibly-modified
value back *out* before running the update expression. Concretely: allocate a new binding,
initialize it from the previous iteration's value, run the body (closures capture *this*
binding), then copy back and run `i++` against the next one. The copy-out is the part people
miss: without it, `i++` would increment a binding no closure ever sees, and the loop would
either not terminate or would not observe body mutations of `i`. `var` has a single
function-scoped binding, so every closure shares it and all of them read the final value.

**2.** Because default arguments are **evaluated at function-definition time and stored in the
function object**, so `i=i` snapshots the current value into the closure's own parameter
storage, bypassing capture altogether. That it works tells you Python closures capture the
*variable* — a cell — rather than the value, and that the cell is per-scope with no
per-iteration freshness. Python 3 did give comprehensions their own function scope, which
stopped `i` leaking into the enclosing scope, but that is a different fix: there is still one
cell shared across all iterations, so late binding survived.

**3.** Go gated the change on the **language version declared in `go.mod`**: a module
declaring `go 1.22` gets per-iteration variables, one declaring `go 1.21` keeps the old
behaviour, and the toolchain can apply it per file. That works because Go compiles from
source, the build system knows every module's declared version, and there is one blessed
toolchain. JS has no such handle — the unit of delivery is a script fetched by a browser with
no manifest, versioning by pragma was tried once and `"use strict"` is the lone survivor
(opt-in, per function), and the web's compatibility rule is absolute. So JS could only add
`let` as *new syntax* beside `var`, leaving `var` broken permanently. The general lesson
generalizes past loops: **whether you can fix a semantic mistake depends on whether your
distribution model has somewhere to write a version number.**

**Trap.** "`let` just moves the declaration inside the loop body." If it did, the update
expression `i++` — which lives in the loop *header*, outside the body — could not see the
variable at all, and the loop would not advance. The copy-in/copy-out dance exists precisely
to reconcile a per-iteration binding with a header that must read and write across
iterations, and a candidate who has not noticed that has not implemented it.

### A3 — Flat closures and the chain walk

**1.** (A) pays at **access**: cost proportional to nesting depth, and each link is a
dependent load — a pointer chase the hardware cannot prefetch or overlap. It pays almost
nothing at creation, one pointer. (B) pays at **creation**: building the array copies one
pointer per captured variable, so creation is O(captured), and it pays nothing at access —
always one load at a known index. Lua's `GETUPVAL` being a single indexed load is exactly this
choice cashed out.

**2.** The intermediate functions must **capture the variable as well**, purely to relay it:
the compiler adds an upvalue to every function on the path from the definition to the use,
even though those functions never mention the variable. The pathological case is a deep chain
where each level creates its closure inside a loop — the transitively-captured set propagates
outward, inflating the upvalue array of every enclosing closure, and the per-creation copying
gets multiplied by the loop trip count. The uncomfortable consequence is action at a distance:
adding one reference to an outer variable in a deeply nested helper silently changes the
allocation profile of code far above it, with nothing at the edit site to suggest it.

**3.** Optimizing compilers generally prefer flat closures. Flat-versus-linked is a choice
*within* closure conversion — the neighbouring transformation, lambda lifting, avoids the
record entirely by turning free variables into extra parameters and rewriting every call
site. A flat record makes the environment a plain struct
with known offsets — which is what enables scalar replacement, unboxing, and treating captured
immutable values as constants for constant folding. The answer flips when creation dominates:
a callback allocated in a hot loop and invoked once pays (B)'s copying on every iteration and
would have paid (A)'s chain walk exactly once. That is the honest trade — **flat closures
optimize the call, linked closures optimize the creation** — and it is why systems where
allocation is a pointer bump (GHC) reason about this differently from systems where allocation
is expensive.

### A4 — Funargs, up and down

**1.** Downward. The requirement is **escape analysis**: proving no reference to the
environment outlives the frame — the closure is not returned, not stored into a heap object,
not captured by another escaping closure, not passed to a callee you cannot see. That last
clause is what makes it hard in practice: any call to an unknown function forces conservative
escape unless you inline it or have an interprocedural summary. This is why escape analysis
and inlining are co-dependent, and why HotSpot's scalar replacement mostly fires *after*
inlining has opened up the scope rather than before.

**2.** Rust makes captures part of the closure's type, and lifetimes turn escape into a *type
error* rather than a missed optimization. A closure borrowing a local has a lifetime bounded
by that local, so returning it does not compile; a `move` closure takes ownership and may
escape; `Box<dyn Fn>` is an explicit opt-in to heap allocation. What that buys over an
analysis: the answer is guaranteed, local, and stable across compiler versions. You never get
a performance regression because an analysis was confused by an unrelated refactor, and you
never silently get a heap allocation in a context that forbids one. What it costs is that the
programmer performs the analysis by hand, and that some correct programs are rejected because
the lifetime system cannot see why they are safe.

**3.** It forecloses the entire family of "produce a behaviour" idioms — factories, partial
application, callbacks registered for later, iterators expressed as closures, memoizing
wrappers — which amounts to functions no longer being values in the useful sense; they become
almost-macros. The bug class is **dangling environments**: a C++ lambda capturing by reference
and stored in a `std::function` that outlives the frame is exactly this, and the failure is a
use-after-free with no diagnostic and no reliable reproduction. The instructive detail is
*why* Pascal was safe with the same representation: it forbade the upward case syntactically.
C++ kept the cheap representation and dropped the restriction, and the bug class is the
difference.

### A5 — Returning from a frame that is gone

**1.** `^` inside a block is a **non-local return**: it returns from the *home method* — the
method activation in which the block was created — not from the block. So the block must carry
a reference to that home activation (Smalltalk's outer/home context; Ruby's captured frame).
Evaluating `^` means unwinding every frame between the current one and the home frame, running
any intervening `ensure`/`finally` handlers in order, and then completing the home frame's
return with the supplied value. Mechanically it is an unwind *with a target*, which is
precisely an exception carrying a private, unforgeable tag — and plenty of VMs implement it as
literally that.

**2.** The check is whether the home activation is **still live** — still on some stack, not
already returned. Implementations record a token: a frame identity, typically a stack index
paired with a monotonically increasing generation number, or a pointer to a heap-allocated
context with a dead flag set on return. At `^`, compare the token against the live frames and
raise if the home frame is gone. It must be a runtime check because liveness is a dynamic
property: the same block at the same source location is legal when invoked within the home
method's dynamic extent — `coll do: [:x | ^x]`, the standard early-exit idiom and the entire
reason the feature exists — and illegal when invoked afterwards. Only the escaping case is an
error, and escape is not decidable in general. One detail that is not optional: the token must
be *frame identity*, not merely a stack index, because recursion reuses indices, and a stale
index will match a different live frame and return from *it* — silent corruption instead of an
error.

**3.** For both: they are genuinely different things wearing one syntax. A block passed to
`each` is *a piece of the enclosing method*, so `return` inside it should mean what `return`
means in that method, or early-exit from an iteration becomes impossible. A lambda passed as a
callback is *a function value*, so `return` inside it should mean "produce my result", or
every callback becomes a control-flow hazard for whoever invokes it. Which to cut: cut proc
return semantics for anything that is a first-class value, and keep non-local return only for
the syntactic block-argument form — effectively Smalltalk's position. What breaks is the
ability to take a `&block`, store it, and have it still behave like inline code, which
underwrites a large amount of Ruby DSL machinery. Ruby's actual sin is not shipping two, it is
that the two axes — return semantics and arity strictness — are welded together, so you cannot
ask for lenient arity with self-return or the reverse.

**Trap.** "You can catch this at compile time by rejecting `^` inside a block that gets
returned." You catch the syntactically obvious case and miss everything real: a block stored
into a collection, passed to a method that stores it, or captured in another closure. And the
identical construct is *legal and idiomatic* when the block is consumed within the home
method's extent, so a static rule strict enough to be sound would reject the feature's primary
use.

### A6 — Break through a native frame

**1.** Because it requires *every* native combinator to check the value, know what to do with
it, and propagate it through its own cleanup — and any one that forgets silently swallows the
transfer, turning `return` into "keep iterating". The sentinel must also be a value the block
could never legitimately produce, which in a dynamic language it can be. It does not compose:
with `each` called from `map` called from user code, every level must forward faithfully, and
the failure mode of one missed forward is a wrong answer rather than a crash. And it cannot
run intervening `ensure` blocks, which `return` is obliged to do — a flag has no way to
trigger cleanup on frames it is passing through.

**2.** Unwind: every native frame must be exception-safe, so any resource it holds is released
by a handler — explicit setjmp-guarded cleanup in C, destructors in C++/Rust. MRI's tag
mechanism is exactly this, and it is the reason writing a correct C extension that yields to a
block is genuinely difficult. The upside is that it works regardless of interpreter structure
and costs nothing on the non-breaking path. Flat loop: the native combinator can no longer be
a native loop; it must become a resumable state object that the interpreter drives, so every
combinator is rewritten and each element costs a trip through interpreter dispatch instead of
a direct call. The upside is that a transfer becomes an ordinary frame-stack operation —
visible, inspectable, no host-language unwinding — and it composes with coroutines, which the
unwinding design does not.

**3.** `break` targets **the method invocation that was given the block** — `each` itself —
and makes that call return. `return` targets **the method that lexically contains the block**.
In `def find; items.each { break 1 }; more_work; end`, `break` makes `each` evaluate to 1 and
`more_work` still runs; `return 1` skips `more_work` entirely. Conflating them means either
`break` skips code it should not or `return` fails to skip code it should — both silent
wrong-answer bugs, and both invisible in the common case where the block is the last statement
in the method, which is exactly where people test. Ruby additionally distinguishes `next`
(return from the block itself), so there are three targets behind what looks like one feature.

### A7 — What proper tail calls actually promise

**1.** It is a guarantee about **space**: a call in tail position must not grow the stack, so a
program whose recursion is entirely in tail position runs in a constant number of frames. It
is *not* a speed guarantee — a tail call still happens and is not automatically cheaper than a
normal call — and, decisively, it is not an optimization the implementation may skip when
inconvenient. It is a semantic requirement, which is the whole point: programs are permitted
to *depend* on it, so "we optimize this when we can" is a different and much weaker feature.
Scheme's reports are explicit that this is what makes iteration expressible as recursion
without a separate looping construct.

**2.** (a) **Stack-inspection security.** The historical Java security model made access
decisions by walking the call stack (`AccessController.doPrivileged` and its relatives); a
tail call that erases the caller's frame erases the evidence those checks consulted. (b)
**Stack traces and observability.** Java's exception model and its entire
debugging/profiling ecosystem assume frames persist, and proper tail calls delete them — a
tail-recursive loop would show one frame with the actual path gone. Lua is honest about
exactly this cost: its tracebacks print `(...tail calls...)` where frames were elided. A third
real reason: specifying a general tail call across differing frame layouts and access-control
contexts, in a way the verifier can check, is hard to *specify*, not merely hard to build.

**3.** Self-tail-call-to-loop is a local rewrite — `f` calls `f` in tail position, so replace
the call with assignments to the parameters and a jump to the top — and it requires knowing
statically that the callee is `f`. Proper tail calls are required for **mutual** recursion
(the `even`/`odd` pair, where neither function is self-recursive so no local rewrite exists),
for tail calls through a variable or an interface (a state machine dispatching to
`next_state(input)` where the state is a function value), and for CPS-compiled code where
every call is a tail call to an unknown continuation. The program in the setup is precisely the
witness: it is the smallest thing the loop optimization cannot touch.

**Trap.** "TCO is just an optimization, so a language either has it or is a bit slower." The
difference is that with a guarantee, `even`/`odd` at ten million is a *correct program*, and
without it that program is a crash. A best-effort optimization cannot be relied on across
compiler versions or optimization levels, so no library can be written in the style that needs
it — which is why "we do TCO in release builds" is not the same feature at all.

### A8 — Trampolines and CPS

**1.** Instead of calling, a function *returns a thunk describing the next call*, and a driver
loop repeatedly forces thunks until one yields a final value. Space is constant because the
driver's frame is the only frame. The honest cost: one heap allocation per logical call for
the thunk, a megamorphic indirect call per step (the driver invokes an unknown closure every
time), no inlining across the boundary because the callee is a value rather than a site, and
total loss of the hardware's return-address prediction. You have traded a hardware-accelerated
call/return pair for an allocation plus an unpredictable indirect jump — typically an order of
magnitude — which is why trampolining is a correctness fallback rather than a strategy.

**2.** In CPS a function never returns; it calls its continuation. Since every call is then in
tail position, no call needs a return address and the whole program can run on a fixed frame,
which is what makes it a legal shape on a host without proper tail calls — combined with a
trampoline, since the host stack must still not grow. What moves to the heap is **the return
address together with the live locals**, reified as continuation closures: what used to be a
stack frame becomes an allocated object per call, chained by capture. That is the trade in one
line — CPS converts the stack into a heap-allocated linked structure you can inspect, capture,
and resume, which is exactly why it is the natural implementation for first-class
continuations.

**3.** CPS makes closure allocation the dominant cost, and those closures are overwhelmingly
**short-lived and dead on arrival** — the generational hypothesis in its purest form. So CPS
is affordable precisely when allocation is a pointer bump and a minor collection costs
proportional to *survivors* rather than to garbage. GHC is the canonical case: nursery bump
allocation, copying minor collection, and a compiler willing to allocate freely because
garbage that dies before the next collection is effectively free to reclaim. On a runtime with
malloc-based allocation or reference counting — CPython — the same style is unaffordable,
which is a clean illustration that *which styles are idiomatic in a language is downstream of
its GC design*, not of its syntax.

### A9 — Blocks as objects, lambdas as values

**1.** It buys **control flow as a library**. If a block is an object and invoking one is a
message send, then `ifTrue:ifFalse:`, `whileTrue:`, `timesRepeat:`, `do:`, `ensure:`, and
`on:do:` are ordinary methods taking block arguments — there is no privileged conditional or
loop syntax, and a user-defined control structure is indistinguishable from a built-in one.
That is the whole Smalltalk argument and the payoff is real: the grammar is tiny and
extensibility is unbounded. The related consequence is that blocks, being objects, can be
stored, inspected, and handed to a debugger or a scheduler — which is what lets Smalltalk
express exception handling and its process model inside the language rather than in the VM.

**2.** It costs an **allocation per block creation** and a **full message send per
invocation**, on paths that in another language are a jump. Naively, `1 to: n do: [:i | ...]`
is: allocate a block, send `to:do:`, then send `value:` once per iteration — three dispatches
and an allocation where a `for` loop is a compare and a jump. The mandatory optimization is
**compile-time inlining of the control-flow-shaped sends**: the compiler recognizes
`ifTrue:ifFalse:`, `and:`, `or:`, `whileTrue:`, and `to:do:` with *literal block* arguments
and emits jumps, never allocating the blocks at all. Every serious Smalltalk does this, and it
is not an optional nicety — it is the difference between a usable language and a curiosity,
and it is what sets up the soundness problem in Q10.

**3.** It buys **not allocating**. A block passed with the implicit `do |x|` form has exactly
one call site and one lifetime, so the implementation can carry it as part of the frame rather
than as a heap object, and `yield` becomes a direct call rather than a dispatch through a
`Proc`. You pay for objecthood only when you write `&blk` and demand a value. What it costs is
uniformity: there is now block-that-is-not-a-value, `Proc`, `lambda`, and `Method`, with
differing arity rules, differing `return` semantics, and conversion syntax between them — a
permanent source of confusion that a uniform design simply does not have. Ruby chose speed and
idiom over uniformity, and the block/proc/lambda tangle is the invoice.

### A10 — Inlining a message someone might override

**1.** It has assumed the receiver is one of the two Booleans — that the send would have
resolved to `True>>ifTrue:ifFalse:` or the `False` counterpart. That is a speculation, and it
is unsound unless the emitted code **checks it**. So the jump instruction is really "if the
receiver is `true`, jump; if `false`, fall through; otherwise bail out", and the bail-out path
must reconstruct a send that was never emitted: materialize the block objects that were never
allocated, assemble the arguments, and perform the real dispatch. If the bail-out cannot do
that reconstruction, you have a fast path that silently misbehaves, and *that* — not the
inlining — is the soundness bug.

**2.** It is a **deoptimization**. Specialized code detected that its guard failed and
transferred to a general path, and `mustBeBoolean` is the language-visible name of that
transfer. What makes it a genuinely elegant design is that it is a real message send to the
offending object, so a user class can implement `mustBeBoolean` and participate — the escape
hatch from the compiler's speculation is an ordinary part of the object protocol rather than a
hidden VM mechanism.

**3.** **Guarded inlining with deoptimization.** At the inline site emit a guard — receiver's
class equals the assumed class, or "no loaded subclass overrides this method" — and on failure
transfer to code that continues execution as though the inline never happened. Two pieces of
metadata are required. (a) A **deoptimization map**: for each guard, how to reconstruct
interpreter-level state at that program point — frames, locals, expression stack, and any
objects that were scalar-replaced and must now be materialized on the heap. (b) A **dependency
record**: what the compiled code assumed, registered against the classes and methods involved,
so a later definition or class load can find and invalidate it. Without (a) you cannot bail
safely; without (b) you never learn that you must. That pair is the whole HotSpot/V8
architecture in two bullets, and it is the standing price of speculating in a language where
the class graph can change.

**Trap.** "The compiler can just check at compile time whether anyone overrode `ifTrue:`." In
an open world, "nobody has overridden it" is a fact with an expiry date — a class defined
later, a file loaded later, a REPL line typed later all falsify it. The check must therefore
be a *runtime guard plus a registered dependency*, and a candidate who proposes the static
check has skipped the reason deoptimization exists at all.

### A11 — Cleanup versus early exit

**1.** Because `finally` runs *during* unwinding with the pending exception held as in-flight
state, and an abrupt completion of the `finally` block **replaces** the pending completion.
The unwinder's order is: record the in-flight completion (a value to return, or an exception
to propagate), run the handler, and if the handler itself completes abruptly, the handler's
completion wins. That is why every style guide bans `return`, `break`, and `throw` in
`finally`: the exception is not caught, it is *overwritten*, so no trace of it exists anywhere
— no log, no stack trace, nothing. It also tells you the unwinder is a state machine over a
"pending completion" value rather than a simple longjmp; a longjmp has no way to express "and
now my destination has changed".

**2.** `jsr`/`ret` pushed a *return address* as a value onto the operand stack, which made the
type of a stack slot depend on which path reached the instruction. That is very hard to
verify, and the JVM's move to a stack-map-based verifier — single-pass, with declared frame
types at merge points — is fundamentally incompatible with a slot whose type varies per
predecessor. So `jsr` was deprecated and then rejected outright in newer class file versions.
Duplication costs **code size**: the `finally` body is emitted once per exit path — normal
completion, the exceptional path, and each `break`, `continue`, or `return` leaving the block
— so a `finally` doing real work in a method with several exits bloats measurably and can push
the method past an inlining threshold, which is a real second-order performance cost. Verifier
simplicity, paid for in bytes.

**3.** The argument for function scope: `defer` exists to **release what the function
acquired**, and its dual is the function's return, so it should pair with `return` rather than
with a block. Function scope also makes `defer`'s interaction with named return values
coherent — a deferred closure can modify the value being returned — which block scoping would
muddle. The bug: `defer` inside a loop accumulates, so opening ten thousand files in a loop
holds ten thousand descriptors until the function returns, and the idiomatic fix (wrap the body
in an immediately-invoked function literal) is ugly enough that people skip it. Go made the
common case cheap with **open-coded defers**: when the set of deferred calls in a function is
statically known and not inside a loop, the compiler emits the calls inline at each exit path
with a bitmask tracking which ones are live, instead of pushing entries onto a runtime-managed
list. Note where that lands — it is exactly Java's handler duplication from part (2), arrived
at from the opposite direction.

**Trap.** "`defer` in a loop is a footgun but at least it is obvious." It is invisible in the
common case, because the function usually returns quickly during development and the resource
pressure only appears under a large input in production. The general shape is worth naming:
any cleanup mechanism scoped to a unit *larger* than the acquisition site converts a bug into
a load-dependent bug.

### A12 — Labelled break is not a goto, until it is

**1.** Simple case: a single unconditional jump to the instruction following the labelled
loop. With an intervening `try`/`finally`, it becomes an **unwind** — the jump cannot be
direct, because every `finally` body between the break and the target must run, innermost
first. So the compiler either emits those finally bodies inline before the jump (Java's
duplication, with the `break` counting as one more exit path) or compiles the construct as a
runtime unwind to a target label. Conceptually it stopped being a *jump* and became a
*structured exit with cleanup*, which puts it in the same category as `return` and exception
propagation. The general rule worth stating: **any transfer crossing a scope boundary that
carries cleanup is an unwind wearing a jump's syntax.**

**2.** **The target is always a statically enclosing construct, and control always moves
forward and outward.** You can never jump *into* the middle of a loop or a block, so every
loop retains a single entry point, every block still dominates its own contents, and scoping
plus definite-assignment analysis remain structural walks over the AST. Arbitrary `goto`
permits entering a scope at an arbitrary point, which breaks single-entry loops, breaks the
guarantee that a variable's declaration executed before its use, and makes cleanup insertion
undecidable by inspection. Labelled break is `goto` restricted to the edges the syntax tree
already contains.

**3.** **Reducibility** — every cycle in the control-flow graph has a single entry point, or
equivalently every back edge targets a dominator. Reducible graphs make natural-loop
identification, loop-invariant code motion, and structural analyses straightforward;
irreducible ones force node splitting or a general analysis that is both slower and weaker.
Where else it matters: WebAssembly's control flow is **structured by design** —
`block`/`loop`/`br` to a label depth, with no arbitrary jumps — for precisely this reason,
because it makes validation single-pass and lets a consumer generate code without first
building a general CFG. It is also why compiling arbitrary C `goto` to wasm requires either
relooping or a dispatch-loop-plus-state-variable, which is the trampoline from A8 wearing yet
another costume.

### A13 — Arity, partial application, and the unknown callee

**1.** A **partial application object** — GHC calls it a PAP — holding the function together
with the arguments supplied so far. When further arguments arrive, it is either completed
(arity now satisfied, so make the real call) or extended (build another PAP). The call
sequence therefore has three cases: exactly saturated (fast path, direct call), under-saturated
(allocate a PAP), and over-saturated (call with the first *n*, then apply the result to the
rest — which requires the result to itself be a function, a fact known only at runtime). That
third case is why a curried language cannot compile every call site to a fixed-arity jump.

**2.** **Push/enter**: the caller pushes arguments and jumps to the function, and the *callee*
inspects how many arguments are available and deals with any mismatch. **Eval/apply**: the
caller evaluates the function first, inspects its arity, and the *caller* decides how to call
— a direct call when saturated, a generic apply otherwise. Eval/apply puts the cost on the
caller, which turns out to be correct, because at most call sites the arity is statically known
and the check vanishes entirely at compile time. Push/enter forces every function to begin with
a stack-depth check it usually does not need, and it makes the calling convention harder to map
onto a machine's registers, since the callee must be prepared for a variable stack shape.

**3.** **It defeats the engine's fast call path and splits its inline caches.** A bound function
in JS is an exotic object whose call behaviour prepends stored arguments and forwards, so a
call site seeing bound functions sees a *different callable identity per `bind`* — it cannot
inline through the wrapper without special-casing it, and the underlying target's own caches
are fragmented across wrappers. It also allocates: `bind` called in a loop or per render
produces a fresh function object every time, which is why "do not bind in render" is a
standard React performance rule and why engines added specific machinery to see through simple
bound functions. `functools.partial` has the same shape: an extra layer of frame construction
and argument-tuple building per call, which CPython's specializer handles worse than a plain
function.

**Trap.** "Partial application is just a closure, so it costs what a closure costs." A closure
has a known arity and a known code pointer; a PAP or a bound function has neither at the call
site, so the call goes through a generic apply that cannot be specialized. The cost is not the
allocation — it is that the callee's identity became dynamic.

### A14 — Shared cells and the capture-by-value alternative

**1.** Both closures must reach the *same* storage — a shared cell — so a write through one is
visible through the other. That forces `n` out of any per-closure copy and into a single object
referenced by both: an open upvalue while the frame lives and a closed one afterwards in the
Lua design, or a heap cell from the start in a boxing design. The consequence worth naming is
that **creating a closure retroactively changes a variable's storage class**, so a compiler
must determine capture-ness *before* assigning stack slots — which is why closure conversion
runs before register allocation rather than after, and why a late-discovered capture can force
re-running earlier passes.

**2.** It forecloses the counter idiom above, and generally any accumulator or state machine
expressed as closures over a mutable local — the state must be lifted into a field, an array
element, or an `AtomicInteger`, which is why `int[] count = {0}` is a well-known Java
workaround. What it buys: (a) capture is a copy, so a lambda is a self-contained value with no
link to a frame — no open/closed distinction, no lifetime coupling to the defining method, no
retention of the enclosing environment; and (b) the **concurrency argument**, which is the real
one: a captured variable that cannot be mutated cannot be involved in a data race, so a lambda
handed to another thread is safe with respect to its captures by construction. Java bought a
whole class of thread safety at the price of one idiom, at a moment when it was adding
`parallelStream`.

**3.** **Environment retention.** The closure holds the cell, and under a linked-environment
representation it holds the entire enclosing environment record, so one small callback can keep
a large object graph alive — a listener capturing a scope that contains a big buffer keeps the
buffer alive for as long as the listener is registered. This is one of the leading causes of
memory retention in long-running JS applications. Mitigations: flat closures with per-variable
capture, so only what is actually referenced is retained (one of the strongest practical
arguments for the flat representation); compilers that null out captured slots proven dead; and
in engines, being careful that debugger support — which forces retaining the *whole* scope so
it can be inspected — is enabled only when a debugger is actually attached. That last one is a
clean illustration that observability and retention are in direct conflict here, and that the
resolution is a mode switch rather than a compromise.

### A15 — The closure that outlived its stack

**1.** The invariant is: **no stack slot may be released while an open upvalue still points at
it** — equivalently, every path that abandons a frame must close that frame's upvalues first.
The normal return path never violates it because closing is wired into the return sequence:
returning pops the frame *through* a routine that first closes every upvalue at or above the
frame's base. Abnormal teardown — an uncaught error, a killed task, a suspended coroutine
collected as garbage — is a *second* way to abandon frames, and if it releases the stack
directly instead of going through that routine, the upvalues remain open, pointing into freed
or reused memory. The bug is not in the closure machinery at all; it is that there are two ways
to leave a frame and only one of them was taught the rule.

**2.** Both are instances of **abnormal exit paths failing to discharge obligations that normal
exit discharges**. Closing upvalues, running `ensure`/`defer`, releasing locks, popping handler
stacks, restoring dynamic bindings — all of these are per-frame obligations, and every teardown
path owes all of them. Each such bug looks unrelated to the others until you notice the shared
cause: the obligations were attached to the *return instruction* rather than to *the act of
destroying a frame*. That is the design error, and it reproduces itself exactly once per
obligation, which is why these bugs arrive in a slow trickle over years rather than all at
once.

**3.** Funnel every frame destruction through a **single primitive** — call it `pop_frame` —
that discharges all obligations in a fixed order, and make raw stack truncation private to it
(an inner module in Rust, a static function in C). Teardown paths then cannot forget, because
they cannot express forgetting. The cost: paths that only wanted to drop memory now pay the
full obligation walk, and error paths that used to be a truncation become a loop that may run
user code — an `ensure` block executing during error unwinding can itself raise — so the
primitive must be reentrancy-safe and failure-safe, which is genuinely fiddly and is exactly
why implementations skip it. The cheaper alternative fix is to eagerly box every captured
variable, which removes this bug entirely by decoupling closures from frames, at the allocation
cost from A1.

**Trap.** Diagnosing this as "the GC collected the stack too early" or "we need a write
barrier". It is not a collector bug and no barrier helps: the upvalue is a perfectly reachable,
perfectly traced object whose *interior pointer* was aimed at memory the runtime deliberately
released outside the collector's knowledge. Reaching for the GC here is the classic wrong turn,
because the collector is the one component behaving correctly.

### A16 — Dynamic scope, and why it is not a closure

**1.** Because the value depends on the **dynamic call chain at the moment of the read**, not
on lexical nesting at the point of the write — the same expression yields different values
depending on who called you, and a closure captures precisely the wrong thing (the environment
where the function was *defined*). The lookup is a function of the stack of currently active
bindings: of time, not of place. That is exactly why it survives as a distinct feature.
Everything else in a modern language is lexical, and dynamic scope is the deliberate exception
reserved for genuinely ambient things — the current output stream, the current transaction, the
current locale, the current request.

**2.** Deep binding: a read walks a chain of (name, value) bindings from most recent, so **reads
are O(depth)** while bind and unbind are O(1) push/pop. Shallow binding: the current value
lives in one cell per name, so **reads are O(1)** — a single load — and binding saves the old
value and restores it on scope exit, also O(1), but **stack switching becomes O(number of
bound variables)**. That switching operation is the decider. If you switch stacks, threads, or
coroutines frequently, shallow binding must swap out every bound variable at each switch or
report the wrong value, whereas deep binding merely changes which chain you are reading.
Single-threaded interpreters choose shallow because reads dominate overwhelmingly; systems with
many concurrent contexts drift toward deep binding or toward a per-context table.

**3.** Thread-locals key ambient state on the OS thread. With coroutines, many logical tasks
share a thread and interleave at suspension points, so a value set by task A becomes visible to
task B — and worse, a value you set before suspending may have been overwritten by another task
by the time you resume. The precise failure is that **the identity the state was keyed on (the
thread) is no longer the identity the state is about (the logical task)**. `contextvars`
re-keys onto a *context* that is captured and restored around task execution. A correct
implementation must snapshot the context when a task is created, install it when the task is
resumed, and restore the previous one when the task suspends — the context becomes part of the
task's saved state, exactly like the instruction pointer. The corollary people trip on: a
mutation made after a task copied the context is *not* visible to that task, which is a real
semantic difference from thread-locals and not merely a scoping refinement.

### A17 — Recursion depth as a language-level error

**1.** Because the resource being exhausted is the **host's machine stack**, and the interpreter
has no reliable, portable way to know how much of it remains. The host-stack consumption per
user-level call varies with the host function's frame size, which varies with the compiler,
optimization level, and which path through eval was taken. Guard pages fire as a signal at an
arbitrary instruction, where you cannot safely run a handler that allocates or unwinds, and
where recovering means longjmp'ing out of unknown host frames with no destructors run. So you
are left estimating — and an estimate that is too generous crashes hard, while one that is too
tight rejects valid programs.

**2.** Wrong permissively: a recursion consuming many host frames per user frame — a call
through `__getattr__`, into a C function, into a comparator, back into user code — exhausts the
real stack long before the counter fires, so you segfault with the counter at a fraction of its
limit. Wrong restrictively: deeply recursive but cheap frames, such as a plain recursive walk
over a linked list, hit the counter while gigabytes of stack remain, which is why every Python
programmer has reached for `setrecursionlimit` and why raising it is genuinely dangerous — you
are loosening the only guard standing between you and the real crash. Recent CPython
**decoupled the two resources**: Python-to-Python calls are inlined in the evaluation loop and
no longer consume a C frame, so pure-Python recursion is bounded by a heap-allocated frame
stack, while a separate C-recursion limit guards the paths that genuinely re-enter native code.
Two limits, because there were always two resources — which was the correct diagnosis all
along.

**3.** Its limit becomes **memory**: the frame stack is a growable heap array, so the bound is a
policy number you choose, and exceeding it is a clean, catchable error raised at a well-defined
point with the interpreter in a coherent state. You can even make it a soft limit and grow past
it. The new failure mode is that the frame stack **moves when it grows**, so any borrowed
pointer into it dangles after a reallocation — a cached pointer to the current frame, a slice of
the operand stack held across a call, a native primitive holding a reference to its arguments.
That bug is quieter and nastier than a stack overflow: it only manifests when a call crosses a
growth boundary, so it reproduces at specific depths and vanishes under a debugger that
preallocates differently. Implementations either store indices rather than pointers everywhere,
or reserve the maximum up front and never move.

**Trap.** "Just raise the recursion limit; it is only a safety valve." It is a safety valve on
a resource the interpreter cannot measure. Raising it in a program whose recursion passes
through native frames converts a catchable `RecursionError` into a segfault, and the segfault
appears at a different depth on a different platform, optimization level, or Python build —
which is why the limit is conservative and why the real fix was to stop consuming the host
stack, not to pick a bigger number.
