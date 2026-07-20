# 03 — Bytecode and Interpreter Design

The shape of the loop, and what the instruction set costs. The through-line: *every
decision here is paid once per instruction, forever, in the hottest code in the system.*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — Stack versus register, per instruction

`local c = a + b`, where `a` and `b` are already locals.

```
; stack machine (JVM-shaped)          ; register machine (Lua-shaped)
ILOAD  a                              ADD  c, a, b
ILOAD  b
IADD
ISTORE c
```

1. Four instructions become one. Name precisely what per-instruction cost you removed
   and what you added in exchange — "it's fewer instructions" is not an answer.
2. The register form forces the compiler to own register allocation. Say concretely what
   Lua's compiler must now do that a stack compiler does not, and where it hits a hard
   wall that surfaces to the user.
3. CLR IL and JVM bytecode are stack machines despite being consumed almost exclusively
   by JIT compilers, never interpreted at speed. Argue that this is the right choice for
   them and the wrong choice for Lua.

### Q2 — Three dispatch loops

```c
/* A */  for(;;) { switch (*ip++) { case OP_ADD: ...; break; ... } }
/* B */  #define NEXT goto *table[*ip++];
         OP_ADD: ...; NEXT;
/* C */  static void op_add(Ctx c) { ...; MUSTTAIL return table[*c.ip++](c); }
```

1. B is not merely "A without the bounds check". Explain the mechanism by which
   replicating the dispatch changes anything, and name the hardware structure involved.
2. It has been argued that this win largely evaporated on modern cores. What changed in
   the hardware, and what part of the argument for B survives anyway?
3. C is the LLVM `musttail` design used by Google's protobuf parser and by CPython's
   tail-call interpreter build. What does it buy that B does not, and what does it cost?

### Q3 — Fusing two instructions into one

You fuse `LOAD_CONST k; ADD` into a single `ADD_CONST k`.

1. The work performed is byte-for-byte identical. Enumerate every component of the win.
2. You keep going and add ninety more fused opcodes. Describe the mechanism by which the
   win reverses.
3. CPython's `LOAD_ATTR_INSTANCE_VALUE` looks like a fused opcode and is not one.
   Distinguish it from `ADD_CONST`, and say what goes wrong if you conflate the two
   categories in your instruction-set design.

### Q4 — What a frame stores about the stack

Two frame layouts:

```rust
struct Frame { slots: *mut Value, /* absolute */ ip: usize, .. }
struct Frame { base:  usize,      /* index    */ ip: usize, .. }
```

1. The pointer form saves an addition on every local access. Name the single capability
   it makes impossible.
2. Lua's C API makes you wrap anything that can grow the stack in `savestack` /
   `restorestack`. What are those for, and what is the *complete* list of things that
   must be fixed up when a value stack is reallocated?
3. Suppose you keep the pointer form and simply never relocate the stack. State what you
   have just decided about deep recursion and about cheap coroutines, and name the error
   the user eventually sees.

### Q5 — Hoisting the instruction pointer

```rust
// A: ip lives in the frame          // B: ip hoisted into a local at loop entry
loop { let op = chunk[f.ip]; f.ip += 1; ... }
loop { let op = chunk[ip];   ip   += 1; ... }   // saved back on call/return
```

1. The frame field is in L1 cache and the load is a couple of cycles. Say why hoisting is
   worth doing anyway — the answer is not "it saves a load".
2. Enumerate every event that invalidates a hoisted `ip`. Getting this list short is the
   bug.
3. A VM caches some per-execution state and guards the cache with the identity of the
   currently executing *closure*. Give the program that breaks it, and state the general
   rule for which cached quantities that guard is sound for and which it is not.

### Q6 — The operand did not fit

Your call instruction is `CALL <fn> <argc:u8>`.

1. What does capping `argc` at 255 actually forbid? And explain why `f(*args)` with a
   300-element list is *not* the same question, even though users will report it as one.
2. Three encodings: fixed 32-bit words (Lua), variable-width bytes (JVM, CPython before
   3.6), fixed 16-bit wordcode with an escape prefix (CPython 3.6+). Give each one's
   decode cost and each one's structural cost.
3. Three ways to carry an operand that outgrew its field: an `EXTENDED_ARG`-style prefix,
   a `wide` variant opcode, or an indirection through the constant pool. Say which for
   which kind of operand and why the other two are wrong there.

### Q7 — The constant pool

```
for i = 1, n do print("hello") end
```

1. Why must `"hello"` be a pool entry rather than an inline immediate, and what does that
   force about the relationship between code objects and the garbage collector?
2. Interning selectors/symbols to small integers: state what it buys *at the dispatch
   site specifically*, and what it costs at compile time, at runtime, and at teardown.
3. One pool per function versus one per compilation unit. Give the trade, and name a
   retention bug that only the second design can have.

### Q8 — Patching a forward jump

A single-pass compiler emits `JUMP_IF_FALSE <placeholder>`, compiles the body, then
writes the real offset back.

1. Why must the placeholder's *width* be chosen before the distance is known? Name the
   two ways out and what each costs.
2. Real compilers ship an error for this — javac's "code too large", and the 16-bit
   offset errors in Wren and in Crafting Interpreters' Lox. Why does the error exist
   instead of the compiler simply doing the right thing?
3. Backward jumps need no patching at all. Name what is nonetheless harder about them.

### Q9 — Stopping the loop

```lua
while true do x = x + 1 end
```

The collector wants to stop the world. The scheduler wants to preempt.

1. Why can the collector not simply stop the thread wherever it happens to be?
2. Where do the checks go, and why is "check on every instruction" wrong even though it
   is correct?
3. HotSpot polls a page that gets `mprotect`ed away to signal a safepoint, instead of
   testing a flag. What does that actually save, what does it cost, and why is it usually
   the wrong trade inside a bytecode interpreter specifically?

### Q10 — Instrumentation that is switched off

```c
for(;;) {
    if (UNLIKELY(vm->trace_enabled)) trace(op, ip, sp);
    ...
}
```

`trace_enabled` is false in every production build.

1. Enumerate the costs when the flag is false. There are at least four and the branch is
   the cheapest of them.
2. CPython's PEP 669 replaced per-event checks in the loop with something structurally
   different. What, and why is it strictly better than a perfectly predicted branch?
3. You want opcode-frequency counts for a performance study. Explain why "add counters to
   the loop and measure" is a methodologically broken plan, and what to do instead.

### Q11 — Peephole optimization on a flat array

Windows like `LOAD_CONST k; POP` → nothing, or `JUMP L; L: JUMP M` → `JUMP M`.

1. Why is a sliding-window pass over a flat bytecode array structurally more dangerous
   than the same pass over an IR with basic blocks?
2. CPython moved its peephole optimizer off the emitted bytecode and onto the compiler's
   own CFG. Name what that bought, beyond "it's cleaner".
3. Give one rewrite that is genuinely safe on flat bytecode, and one that looks safe and
   is not.

### Q12 — Bytecode you did not produce

The JVM verifies class files. Dalvik verifies dex. WASM validates modules. CPython
documents that a hand-built code object may crash the interpreter and declines to check.

1. What is a verifier actually proving, and why is the *stack machine* form substantially
   easier to verify than a register form?
2. Java 6 introduced `StackMapTable` and moved work from the verifier to the compiler.
   What cost was being avoided, who pays now, and what broke in the ecosystem?
3. Defend CPython's refusal as a design decision, then name precisely what it forecloses
   and why it cannot be bolted on later.

### Q13 — Bytecode as a private ABI

CPython bumps a magic number every release and refuses stale `.pyc` files. JVM class
files from the 1990s still load today.

1. What has the JVM's compatibility promise cost it, concretely, in the evolution of the
   instruction set? Give specifics.
2. Why is versioning your bytecode "just in case" a trap rather than cheap insurance?
3. What event forces you to commit to stability, and what changes about the whole design
   the day it happens?

### Q14 — Locals array, operand stack, accumulator

The JVM frame has a locals array *and* an operand stack. Lua has one register window and
no separate operand stack. V8's Ignition has registers *and* an accumulator.

1. What does splitting locals from the operand stack buy?
2. Merging them, as Lua does, moves an obligation onto the compiler. State the obligation
   precisely, and name the failure mode when it is not met — it is not a crash.
3. Why does Ignition have an accumulator at all, given it already has registers? Name
   what the accumulator trades away.

### Q15 — Your refactor made it slower

You extracted the body of the `CALL` handler into a helper function for readability.
Nothing else changed. Benchmarks regress a few percent across the board — including on
programs that never execute a single `CALL`.

1. Give the two most likely mechanisms, and explain why one of them affects programs that
   never hit the changed code.
2. How do you determine which one it is, without guessing?
3. Give the clean fix, and say why `#[inline(always)]` / `__attribute__((always_inline))`
   is usually not it.

### Q16 — How big is the frame

Before running a function you must reserve its stack window.

1. How is the maximum stack depth computed, and why is "walk the instructions summing the
   net stack effect" wrong?
2. What happens to that number for an instruction whose stack effect depends on an
   operand, and what happens for one whose effect is genuinely dynamic?
3. A VM that grows the stack on demand instead — what does it gain, and what does it now
   owe? Connect the answer to Q4.

---

## Answers

### A1 — Stack versus register, per instruction

**1.** You removed three *dispatches* and three *decodes*. In an interpreter, the fixed
cost of getting to a handler — the indirect branch, its prediction, the operand extraction
— is a large fraction of the cost of a simple instruction like an integer add. Collapsing
four instructions to one collapses four dispatches to one, and that is close to the whole
win. What you added: instruction *width*. The register form must name three operands, so
each instruction carries more bits, so the same program occupies more bytes of icache and
the decode itself does more shifting and masking. The Lua 5.0 implementation paper reports
roughly a third fewer executed instructions with a comparable-magnitude increase in code
size — that is the shape of the trade, and it is a good one only because dispatch is
expensive relative to decode.

**2.** The Lua compiler must assign every local and every temporary to a numbered slot in
the frame's register window, maintain a free-register high-water mark, restore that mark at
each statement boundary, and guarantee that at every control-flow join the same value lives
in the same register — because there is no dynamic stack depth to absorb a disagreement. A
stack compiler gets all of that for free: "push" means "wherever the top is". The hard wall
is the width of the `A` field: Lua caps a function at roughly 250 registers, and a function
with too many live locals or a sufficiently nested expression is *rejected at compile
time*. That is a format constant surfacing to the user as "your function is too complex",
and no amount of compiler cleverness removes it — only a format change does.

**3.** IL and class files are *distribution* formats consumed by a JIT. Stack form is the
right choice there for three reasons. It is compact, which matters when the artifact is
shipped and loaded. It is trivially verifiable, because the operand stack *is* the def-use
chain — an operand's producer is structurally determined, so type propagation is a
straight-line simulation. And it is target-independent: you do not want to bake a register
count into a format that will be compiled to machines with wildly different register files,
and the JIT reconstructs SSA from a stack form almost mechanically. Lua has the opposite
constraints: the artifact is produced and consumed by the same process, is never verified,
and is *interpreted* — so per-instruction dispatch cost is the entire game and every one of
the stack form's advantages is worth nothing to it.

**Trap.** "Register machines are faster because they avoid pushing and popping memory."
The operand stack's top few slots are in L1 and often kept in a register by a
top-of-stack cache; the memory traffic is close to free. The win is the count of
dispatches, and if you say "memory traffic" you will also predict that a register VM helps
a JIT-backed system, which it does not.

### A2 — Three dispatch loops

**1.** A has exactly one indirect branch, shared by every opcode. The processor's indirect
branch predictor sees a single site whose target history interleaves every opcode
transition in the program — so it is being asked to predict the next opcode from a stream
that mixes all of them. B replicates the dispatch at the tail of each handler, giving one
indirect branch *per opcode*. Real bytecode has a strongly skewed bigram distribution
(after `LOAD_LOCAL` you very often get `LOAD_LOCAL` or an arithmetic op), so a per-opcode
branch site can learn "what usually follows *me*" and predict it. The structure is the
indirect branch predictor / branch target buffer. Ertl and Gregg's work on efficient
interpreters is the canonical measurement of this effect.

**2.** What changed: modern indirect predictors (the ITTAGE family, on Intel from roughly
the Haswell generation onward, and on contemporary AMD and ARM cores) use long global
history, so a single shared indirect branch can be predicted nearly as well as replicated
ones — Rohou et al.'s "don't trust folklore" result. What survives: (a) on in-order and
embedded cores with weak predictors the classic win is entirely intact; (b) threading still
removes the range check and lets the dispatch be scheduled *with* the handler's work rather
than after a join; and (c) the durable one — a single giant `switch` is one enormous
function, and its register allocator must satisfy every handler at once, so the worst
handler's register pressure taxes all the others, and hot values get spilled at the loop
header where all paths merge. Replicated dispatch removes that merge point.

**3.** C buys **register allocation you can actually control**. Each handler is a separate
function with an identical signature, so `ip`, `sp`, the frame pointer and the dispatch
table are passed in registers fixed by the calling convention — the compiler *cannot* spill
them, because they are parameters, and the tail call means no frame is built. You have
converted "please keep these in registers across a 6,000-line function" into a hard ABI
guarantee. It also lets the optimizer compile each handler in isolation, which keeps
compile times sane and makes one bad handler's complexity local. Costs: you need a compiler
with a real `musttail` guarantee (this is not portable C); everything not in the parameter
set must be reloaded through the context on every handler, so you must choose the register
set very carefully and anything that does not fit gets slower; profiles and debuggers see a
forest of tiny functions instead of one loop; and you have tied your interpreter's
performance to one compiler's behaviour.

**Trap.** "Computed goto is faster because it skips the bounds check." The bounds check is
a compare and a well-predicted branch; it is nearly free. If you name it as the mechanism
you will also be unable to explain why the technique helps less than it used to, or why the
tail-call form helps *more* than computed goto on the same hardware.

### A3 — Fusing two instructions into one

**1.** Four things. (a) One dispatch instead of two — the indirect branch, its prediction,
and its BTB slot. (b) One decode instead of two. (c) The intermediate value never
round-trips through a stack slot: it is produced and consumed inside one handler, so it
lives in a register, and you delete a store, a load, and the store-to-load forwarding
between them. (d) Shorter bytecode, so more of the program fits in icache and the fetch
stream is denser. (c) is the one people forget and it is frequently the largest.

**2.** Opcode space is finite — typically eight bits — so more fused forms mean either
squeezing out room for real instructions or widening the opcode field for everyone. More
handlers means a larger interpreter *text* footprint, which pushes the loop out of L1i and
spreads the genuinely hot handlers apart; it also consumes branch-target predictor
capacity, so the per-opcode prediction that motivated threading in the first place starts
to degrade. And in the `switch` form each new case is more pressure on one register
allocator. There is a crossover where the icache misses you bought exceed the dispatches
you saved. Both YARV's instruction unification and Ertl's dynamic superinstructions
converge on the same conclusion: fuse the *measured* hot bigrams of a representative
corpus, not everything you can think of — and accept that the corpus drifts.

**3.** `ADD_CONST` is a **fusion**: it is unconditionally correct, it can never be wrong,
it needs no guard and no fallback, and its payoff is a fixed constant per occurrence.
`LOAD_ATTR_INSTANCE_VALUE` is a **specialization**: it does *less work* by betting that the
receiver has a particular type and layout, so it requires a guard, a deoptimization path
back to the generic instruction, and a policy for when to re-specialize. Conflating them
produces two specific bugs: you ship a "fused" opcode that is actually a bet and has no
fallback (silent wrong answers), or you attach a guard and a deopt counter to something
that is unconditionally correct (pure overhead, plus a slot in your feedback structures for
nothing).

**Trap.** "Superinstructions are the interpreter's version of inlining." Inlining's payoff
is almost entirely second-order — it exposes the callee to the caller's optimizer. Fusion
has no downstream: the win is a bounded constant proportional to your dispatch cost, and
when you have driven dispatch down (by threading, by tail calls) the value of fusion drops
with it.

### A4 — What a frame stores about the stack

**1.** It makes the value stack **immovable**. Any reallocation invalidates every stored
pointer at once, and the index form survives because it is relative to a base you re-read
after the move.

**2.** They convert a raw pointer into a stack *index* across an operation that might
`realloc` the stack, and back again afterwards. The complete fix-up list on a move is
longer than people expect: every call-info's base / top / function pointers; the caller's
saved top; every **open upvalue**, which is a heap object holding a pointer *into* the
stack and is the one most often missed; any iterator or cursor the runtime holds into the
stack; any pointer a native extension is holding across a call that can grow the stack —
which is exactly why the C API forces you to do it manually, because the VM cannot see the
C function's locals. Miss any one and you get a use-after-free that only manifests past a
particular growth threshold, i.e. it reproduces on inputs of a certain size and looks like
a data-dependent miracle.

**3.** You have decided that the stack is sized once, so either you reserve a large window
up front — which sets your per-coroutine memory floor and rules out "millions of cheap
fibers" — or you accept a hard recursion depth. The user sees a fixed-depth stack overflow:
CPython's `RecursionError` at a configurable recursion limit is precisely this design
choice surfacing, and the fact that the limit is a tunable *number* rather than "until
memory runs out" is the tell. Foreclosed: cheap, growable, numerous coroutine stacks.

**Trap.** "I'll use a `Vec<Value>` and hand out `&mut` slices; the borrow checker keeps me
honest." It keeps you honest by refusing to compile the interpreter, and the standard
escape is raw indices — at which point you have chosen the index design for borrow-checker
reasons and never noticed that the real question was relocation. Decide relocation first;
the aliasing story follows from it.

### A5 — Hoisting the instruction pointer

**1.** Because the frame field is *memory the compiler cannot prove is unaliased*. Every
call through a function pointer, every store through a `Value*`, every helper that takes
`&mut Vm` forces a reload of `f.ip` afterwards, because the compiler must assume it may
have been written. Hoisting does not make the load cheaper; it makes the value **stop being
memory**, so it lives in a machine register across the entire handler and the compiler can
schedule around it freely. The same reasoning applies to the stack pointer and to the
current chunk's base pointer, and it is why the tail-call design in Q2 is so effective —
it makes "these live in registers" an ABI fact rather than a hope.

**2.** Every event that changes the current activation or the code being executed: a call;
a return; a throw or unwind; a coroutine or fiber switch; OSR entry; a debugger installing
a breakpoint by mutating the code array; a garbage collector that relocates the code object;
any lazy quickening or specialization pass that *reallocates* the instruction array
(writing in place is fine, moving is not); and any re-entrant call from a native primitive
back into the interpreter. The failure mode is the worst class of VM bug: execution
continues *correctly* with a wrong `ip`, so the symptom appears arbitrarily far from the
cause.

**3.** Recursion breaks it. Two simultaneously live frames of the *same* closure have the
same closure identity and different `ip`, different `base`, different locals — so the guard
passes and the cached value is wrong. The general rule: a closure-identity guard is sound
for quantities that are a function of the **code** (the constant pool pointer, the chunk's
base, the arity, the upvalue count — anything derivable from the compiled artifact) and
unsound for quantities that are a function of the **activation** (`ip`, `base`, slot
addresses, anything per-call). If you want to cache activation-derived state you need a
frame-identity guard, and frame identity is not stable across a stack move either — see Q4.

### A6 — The operand did not fit

**1.** It forbids a *syntactic* call site with more than 255 arguments. Humans do not write
those; generated code does — macro expansion, generated dispatch tables, a literal
structure lowered into a constructor call, a code generator emitting one call per column of
a wide schema. The splat case is a different question because there the argument count is
**not in the encoding at all**: `f(*args)` compiles to an instruction that takes an array,
and the count is discovered at runtime. What bites there is the frame's slot capacity and
the calling convention, not an operand field. The two limits are independent, and the
classic error is to widen the operand field in response to a splat bug report, ship it, and
have the same crash.

**2.** *Fixed 32-bit:* decode is a couple of shifts and masks with no dependency on the
opcode — you know where the next instruction starts before you know what this one is, which
helps prefetch and is essentially required by the tail-call form. Cost: wasted bits on
zero-operand opcodes, and hard ceilings permanently baked into every field width (Lua's
~250 registers, its jump range). *Variable-width bytes:* densest, so the best icache
behaviour, but the next `ip` depends on the current opcode, which serializes decode; and
you **cannot decode backwards**, which complicates disassembly, peepholes, exception-table
scanning, and any tool that wants to find instruction boundaries. *Wordcode with an escape
prefix:* fixed two bytes gets most of the decode simplicity and most of the density; the
prefix is what makes an 8-bit operand field survivable at all. Its structural cost is that
"the operand is large" becomes "there are extra instructions", so every jump offset,
every line-table entry, and every stack-depth computation must count the prefixes.

**3.** **Prefix** for operands whose large case is rare and can afford an extra dispatch —
local indices, jump offsets, most counts. **A `wide` variant opcode** when the large case
is hot enough that you refuse the extra dispatch and you can spend the opcode space; the
JVM has both, a general `wide` prefix and a dedicated `goto_w`, because branches were worth
the opcode and locals were not. **Constant-pool indirection** when the operand is not a
number at all — a name, a signature, a big immediate, a nested function — because then the
instruction carries one index and the payload is unbounded. Using a prefix for something
hot costs you a dispatch in the inner loop; using a pool entry for something you need
without a memory load costs you a dependent load in the inner loop. Both are common
mistakes and both are invisible until you profile.

### A7 — The constant pool

**1.** Because the operand field is a handful of bits and a string is arbitrarily long, and
because you want *one* string object rather than one per execution of the loop. The
consequence is that a code object **owns references to heap objects**, so it is itself a
traceable GC object: the bytecode array and the constants must be traced together, and the
code object's liveness pins every constant it names. That is why unloading code is a
garbage-collection question in every VM that supports it — JVM class unloading is tied to
classloader reachability for exactly this reason, and any system that wants to discard
compiled code has to answer "what did it keep alive" first.

**2.** At the dispatch site, a selector becomes a small integer or a pointer with identity,
so method lookup is a pointer compare or an **array index** rather than a string hash and
compare. That is what makes an inline cache guard cheap enough to be worth having, and it
is what makes a flat per-class method array possible at all — Wren indexes a class's method
buffer directly by symbol id. Costs: a global intern table, which is a mutation point (a
lock, or a per-thread cache, or a promise of single-threadedness); it never shrinks unless
you make it weak, and weak interning drags the symbol table into GC and into finalization
ordering; and symbol ids are VM-global, so they leak across module boundaries and are a
hazard the moment you try to serialize anything.

**3.** Per-function pools give simple lifetimes and good locality — a function's constants
sit together — at the cost of duplicating constants across functions. Per-unit pools
deduplicate and give one place to intern, at the cost of one more indirection and of
*coupling lifetimes*. The retention bug only the second design can have: a single
long-lived closure from a large module retains the module's entire constant table, so
memory that "should" have been freed when the module fell out of use is pinned by one
callback. The JVM chose per-class, CPython chose per-code-object with names interned
separately — both are deliberate positions on this trade.

**Trap.** "Interning is just a memory optimization." Its real product is *identity*: it
converts an equality test on names into a pointer compare, which is what every downstream
cache, method table, and shape mechanism is built on. Treating it as memory dedup means
you will happily make it weak, or lazy, or per-module, and quietly break the identity
guarantee everything else depends on.

### A8 — Patching a forward jump

**1.** Because a single-pass compiler emits bytes into an array as it goes, so every byte
after the placeholder already has a fixed position relative to it. Widening the placeholder
later shifts everything downstream, which invalidates every offset already written — other
jumps, line tables, exception tables, debug info. Way out (a): always reserve the maximum
width. Lua's fixed-width instruction with a large signed-offset field makes this a
non-issue in practice; javac reserves 16 bits and keeps `goto_w` for the rest. You pay
bytes on every jump forever. Way out (b): stop being single-pass — emit basic blocks with
symbolic targets and run a **branch relaxation** pass that iterates to a fixpoint assigning
widths. It must iterate because widening one branch can push another past its limit. This
is precisely the assembler's problem, and it costs you an IR and a pass.

**2.** Because "the right thing" is an IR plus a relaxation fixpoint, and single-pass
compilers exist specifically to avoid both — they exist for compile speed and for
simplicity, and those are real goals. But the deeper reason is that the limit is usually
the *format's*, not the compiler's: javac's "code too large" is the class file's 65535-byte
method limit, and no compiler cleverness can fix a limit that the artifact format forbids
exceeding. The honest reading of such an error message is: *a format constant has surfaced
to the user*, and the fix is a format change with all the compatibility consequences of
Q13 — which is why nobody fixes it.

**3.** A backward jump is not just a jump; it is a designated **policy point**. The
interrupt/safepoint check goes on the back-edge (Q9). The hotness counter goes on the
back-edge. OSR entry happens at the back-edge. So any pass that merges, duplicates, or
reorders blocks must preserve the back-edge's identity or you silently lose preemption or
tiering. And in a language with `ensure`/`finally`, a `continue` compiles to something that
is not a jump at all — it must run pending cleanups first, so the "backward jump" is
actually a small unwinding protocol wearing a jump's clothes.

### A9 — Stopping the loop

**1.** Because stopping requires a **description** of the machine's state — which registers
and stack slots currently hold references — and that description exists only at points the
compiler chose to record it. Between recorded points a reference may live only in a
register the map does not mention, an object may be half-initialized, or a raw interior
pointer may be live. You could record a map at every instruction, but the metadata would
dwarf the code and constrain every optimization. So the set of stoppable points is exactly
the set of described points, and that is what "safepoint" names.

**2.** On loop back-edges, at call sites, and at allocation sites. Back-edges because they
are the only way to bound time-to-safepoint inside a loop; calls and allocations because
state is already coherent there and a map already exists. "Every instruction" is not
incorrect, it is unaffordable: a branch in the innermost loop of the entire system, plus a
map per instruction. Go's history is the instructive counterexample from the other side —
before 1.14, a tight non-allocating loop was genuinely non-preemptible because the
cooperative checks were in function prologues, and the fix required signal-based
asynchronous preemption *plus* the metadata to describe an arbitrary interrupted point.

**3.** The polling page is a load from a page that is normally present; arming a safepoint
`mprotect`s it away so every poll faults. It saves the **branch**: no compare, no
predictor slot, just a load that always hits cache. It costs a fault handler that must
recover exact machine state at the faulting instruction (so you still need the map), a page
protection syscall plus TLB shootdown to arm, and a debugging environment where a stray
memory fault is normal operation. Inside a bytecode interpreter it is usually the wrong
trade because you *already* have a dispatch branch every instruction — you can fold the
check into machinery you are already paying for, by swapping the dispatch table for one
whose handlers all trap, or by decrementing a counter that shares a cache line with
something you already load. Buying a page fault to save a branch you were taking anyway is
a bad deal.

**Trap.** "A safepoint is a check on the back-edge." A safepoint is two things: the check
*and* the map. Everyone implements the check. Without the map, when the check fires there
is nothing the collector can legally do, and you discover this only when you try to make
the collector precise or moving.

### A10 — Instrumentation that is switched off

**1.** (a) The flag must be loaded (or held) in the hottest loop, which is register
pressure; in a monolithic `switch` that pressure is global, so the spill it causes may land
in a handler you never looked at. (b) Code size: the untaken path and the call sequence are
still in the function, pushing hot handlers apart in icache, unless you explicitly outline
them. (c) It is an **optimization barrier**: a call to a non-inlined function means every
VM field must be assumed clobbered, so hoisted `ip` and `sp` get spilled before it and
reloaded after — and if the compiler concludes those values escape, it may stop keeping
them in registers anywhere in the function. (d) It constrains the whole function's frame
and calling-convention decisions. The branch itself is the cheapest item on this list,
which is why "it's predicted, it's free" is precisely the wrong analysis.

**2.** It moves the decision from runtime into the **code**: when monitoring is enabled for
a code object, CPython rewrites that object's instructions to instrumented variants. When
monitoring is off there is no check at all — not a cheap check, *no* check. That is
strictly better than a predicted branch because it removes (a) through (d) as well, and
because the cost now scales with the code you actually instrumented rather than with all
code in the process. The price is a mechanism for mutating live code, with everything that
implies: an `ip` pointing into an array being rewritten, concurrency, and a reliable way to
restore the original.

**3.** Because the counting build is a *different program*: different register allocation,
different code size, different icache behaviour, and the increments themselves occupy
dispatch slots. You cannot subtract that out, and the distortion is not uniform across
opcodes — it is worst exactly where the opcodes are hottest. The sound method is two
builds: an instrumented one used *only* to obtain counts, and a clean one used *only* to
obtain times, with no number ever crossing between them. Better still, take timing from the
untouched binary with hardware counters or a sampling profiler, which is the entire reason
those tools exist.

### A11 — Peephole optimization on a flat array

**1.** Because a flat bytecode array has **implicit incoming edges you cannot see
locally**. Any offset in the array may be a jump target, an exception handler entry, a line
table boundary, or a generator/coroutine resume point. A window rewrite is sound only if
nothing targets an offset inside the window and nothing stores an offset past it — and
answering "does anything target this" on a flat array requires scanning every jump and
every side table in the function. That is not a peephole, it is a whole-function analysis
wearing a peephole's costume. In an IR with basic blocks and symbolic labels, incoming
edges are explicit and the question is answered by looking at the block header.

**2.** Explicit predecessors, so block leaders are known rather than inferred. Symbolic
jump targets, so offsets and widths are *derived output* recomputed after every
transformation rather than invariants the optimizer must preserve by hand. Real dataflow —
dead block elimination, jump threading, redundant load removal — none of which a sliding
window can express. And, underrated: it made the line-number and exception tables derived
artifacts, so they stopped being things the optimizer had to patch in lockstep with the
code, which is where this class of pass historically produced its worst bugs.

**3.** Genuinely safe: rewriting a jump whose target is another unconditional jump to point
at the final target — *provided you keep both instructions in place and only change the
operand*. A length-preserving rewrite cannot break any offset anywhere, which is the whole
reason it is safe. Looks safe and is not: deleting a `POP` that follows a `LOAD_CONST`. If
that `POP`'s offset is an exception handler's entry, a line-table boundary the debugger
maps a breakpoint to, or a generator resume point, you have deleted a target and the
failure will present as a jump into the middle of an instruction, far from the pass that
caused it.

**Trap.** "A peephole is safe as long as I only make the code shorter." Shorter is the
dangerous direction, precisely because it relocates every offset after the window. The safe
direction is same-length rewrites; anything that shrinks the array is a whole-function
transformation and should be done where whole-function facts are available.

### A12 — Bytecode you did not produce

**1.** It proves that the interpreter's own preconditions cannot be violated, so the
interpreter needs no runtime checks for them: that at every program point the operand stack
has a fixed depth and a fixed type signature regardless of which path reached it; that
every local read is definitely assigned; that every jump target is an instruction boundary
and within the method; that each instruction's operands have the types it demands. The
stack form is easier because the stack *is* the def-use structure — each operand's producer
is structurally determined, so verification is a straight-line abstract simulation with
merges only at branch targets. A register form has no such structure: any register may be
written from anywhere, so you need a genuine dataflow fixpoint over the CFG with a type
lattice, plus per-register definite-assignment. Dalvik's verifier is meaningfully more
machinery than the JVM's for exactly this reason.

**2.** Avoided: the **iterative dataflow fixpoint** at class-load time — superlinear,
unbounded in practice, and squarely on the startup critical path, which was the driver
(constrained devices, application startup). Now the compiler emits the types at every merge
point in a `StackMapTable`, and the verifier makes one linear pass checking the claim.
The compiler pays: it must compute and serialize those frames, and class files get bigger.
The ecosystem broke in a specific and famous way: every bytecode-manipulation library —
every agent, every framework doing runtime instrumentation — was producing correct bytecode
with no frames, and suddenly produced *rejected* bytecode. All of them had to learn to
compute stack map frames, which is a nontrivial analysis, and that migration was painful
for years. That is the general lesson about moving obligations to the producer: you move
them to *every* producer, including ones you have never heard of.

**3.** Defence: `.pyc` is not a distribution format for untrusted code. The trust boundary
is the source file and the process, and inside that boundary a caller who can construct a
code object can already reach arbitrary memory through `ctypes` — a verifier would be a
locked door in a wall that is not there. Verification also costs import time, and import
time is a live user complaint. Forecloses: ever using bytecode as a *security boundary* —
no sandbox, no browser-style execution of a module from an untrusted party, no accepting
precompiled artifacts you did not produce. And it cannot be bolted on, because
verifiability is a **format property**, not a pass you add: WASM is single-pass validatable
because its control flow is structured, so jump targets are well-formed by construction and
types are checkable without a fixpoint. Retrofitting that onto an instruction set designed
for an interpreter's convenience means changing the instruction set.

### A13 — Bytecode as a private ABI

**1.** Every opcode ever shipped works forever. The old `jsr`/`ret` subroutine instructions
for `finally` were a verification nightmare, were deprecated, and the verifier still had to
handle them for old class files. Existing invocation instructions could not have their
semantics changed, so new capability had to arrive as an *entirely new instruction* —
`invokedynamic` — with a bootstrap-method indirection, rather than as an extension of what
was there. Structurally, this pushes evolution out of the instruction set and into the
library and the linkage layer, which is why so much JVM progress happens through
`invokedynamic` bootstraps and `MethodHandle`s. And you can never remove a mistake: the
two-slot representation of `long` and `double`, the constant-pool structure, the
descriptor grammar are permanent.

**2.** Because a version number is a promise you have not specified. It implies you will
accept old input, which means you now need a compatibility path per version, a test matrix
across versions, and a policy answering "does old bytecode observe new semantics?" — and,
decisively, you can no longer change the *meaning* of an existing instruction, only add new
ones. Meanwhile, if nobody actually ships bytecode across a version boundary, you have paid
for all of that and used none of it. CPython's magic number is the honest cheap version: it
is a **refusal**, not a compatibility mechanism. It says "this artifact is stale, regenerate
it", and regeneration is free because the source is right there. Detect-and-rebuild is
almost always the right first design.

**3.** The forcing event is someone other than your compiler producing bytecode you must
consume, or someone caching artifacts across an upgrade you do not control. What changes:
you now consume input you did not produce, so a **verifier becomes mandatory** (Q12) and
the format must be designed to be verifiable. Every field needs an explicit size and there
must be real extension points. You must separate "the format" from "the interpreter's
convenience" — no more fields whose documented meaning is "whatever the compiler happened
to put there". And instruction semantics become *specification text*, including every error
case, rather than implementation behaviour. Organizationally you go from one program to two
programs with a contract, and the contract has to be tested from both sides independently.

**Trap.** "We'll design the format to be versionable now so we can promise stability
later." Versionability is the easy part. The hard part is that stability requires a written
specification of every instruction's behaviour including its failure modes, and you cannot
retrofit a specification onto an instruction set whose current semantics is "whatever the
interpreter does" — because the interpreter does things you have not noticed, and users
will depend on them.

### A14 — Locals array, operand stack, accumulator

**1.** Splitting separates a scratch area of dynamic depth from named storage of fixed,
compiler-known index and frame-long lifetime. Consequences: a local's address is a constant
offset regardless of the current stack depth, so local access never has to know what the
expression evaluator is doing; each instruction's stack effect can be stated independently
of how many locals exist, which is what makes depth verification a simple simulation; and
"locals are definitely assigned" becomes a separate, simpler analysis from "the stack is
well-typed". The cost is that moving between them takes explicit instructions — the
`ILOAD`/`ISTORE` traffic of A1.

**2.** The obligation is *static register allocation with agreement at every join*: the
compiler must know at every instruction exactly which slot holds which value, and must
guarantee that all predecessors of a merge point leave the same value in the same slot,
because there is no dynamic depth to absorb a disagreement. Every construct whose arms
could differ — a conditional expression, a short-circuiting `or`, a `break` out of the
middle of an expression, an exception landing pad — must be normalized to a single
destination register by the compiler. The failure mode is not a crash and not a verifier
error: it is **reading the wrong register**, which yields a plausible value of the wrong
variable and corrupts silently. That is why register-machine compilers are religious about
a free-register high-water mark restored at every statement boundary — the discipline is
load-bearing, not stylistic.

**3.** The accumulator is an implicit operand: `Add r3` means `acc = acc + r3`, so a linear
expression tree needs one register operand per instruction instead of three (destination,
source, source). That shrinks the *average* instruction substantially, which matters
enormously for a bytecode that is both interpreted and used as a JIT's input — less to
store, less to decode, less to parse in the compiler front end. It also names the value
about to be consumed, which is a free hint. What it trades away: a serial dependency
through one location, and explicit `Ldar`/`Star` moves whenever a value must persist across
another computation. You have converted operand width into move instructions, and the
compiler now makes that trade per expression — good for the common linear shape, worse for
expressions with several live intermediates.

### A15 — Your refactor made it slower

**1.** (a) **Aliasing and clobbering.** The call means the compiler must assume the helper
may write through any pointer reachable from `&mut Vm`, so hoisted `ip`, `sp`, and any
cached VM fields must be spilled before it and reloaded after. Worse, if taking the address
of the VM state makes the compiler treat those values as escaping, it may stop keeping them
in registers *anywhere in the function* — which is exactly how a change to the `CALL`
handler slows down programs that never call anything. (b) **Code layout.** Extracting a
chunk changes the function's size and moves every handler, so hot handlers that shared an
icache line or sat on a favourable alignment boundary no longer do. A few percent from
layout on an unrelated change is the normal weather of interpreter work, not an anomaly.

**2.** Not by reading the diff. Compare the two builds at the machine level: disassemble
the dispatch loop in both and count spills and reloads of the hoisted values — mechanism
(a) shows up as literal extra stores and loads around the region. Take hardware counters:
frontend-bound stalls and L1 instruction-cache misses point at layout, load/store stalls at
the reload sites point at spilling. And run the decisive experiment for (b): perturb the
layout without changing the semantics — pad, realign, reorder handlers — and see whether
the regression tracks layout. If a no-op alignment change recovers it, the helper was never
the cause.

**3.** The clean fix for (a) is to make the helper's relationship to the hoisted state
explicit: pass `ip` and `sp` in by value and return the updated values, so nothing about
them escapes and the compiler can prove the helper cannot touch them. That is structurally
the same discipline the tail-call design in Q2 enforces by construction, and it is one of
that design's underrated benefits. `always_inline` is usually not the fix because it puts
the code back where it was — including the register pressure you were trying to relieve —
and because it does not address the aliasing claim at all: an inlined helper that still
writes through `&mut self` still forces reloads. The question is what the compiler can
*prove about memory*, not where the instructions physically live.

**Trap.** "A few percent is noise, and I only refactored." On an interpreter benchmark a
few percent is not noise, layout effects are not noise, and "I only refactored" describes
precisely the situation where the regression is real, structural, and will compound with
the next three refactors.

### A16 — How big is the frame

**1.** It is a **maximum over paths**, not a sum along the instruction array. At every
control-flow join the depth from all predecessors must agree — that is the same invariant a
verifier checks — and the answer is the max over the whole CFG. A linear sum is wrong
because array order is not execution order: an `if`/`else` whose arms each push one value
has linear net effect +2 and true depth +1. The correct method is an abstract
interpretation: propagate depth along edges, assert agreement at merges, take the maximum.
Single-pass compilers approximate it by tracking a running depth and a high-water mark
*and* by only ever emitting shapes whose arms balance by construction. Exception handlers
deliberately violate the join invariant: a handler's entry depth is defined by the format
— typically "discard the stack, push the exception" — not by its predecessors, which is
why handler entries are special-cased in every verifier and every depth computation.

**2.** An operand-dependent effect is fine: `CALL n` pops n+1 and pushes 1, and `n` is
right there in the instruction, so the abstract interpreter reads it. A genuinely dynamic
effect is not: if a splat call's argument count is unknown until runtime, no static maximum
exists. The two responses are (a) make the stack effect constant by materializing the
arguments into a heap object first, so the instruction takes one array operand — which is
why splat calls are structurally a *different opcode* in most VMs, not a variant of the
normal one — or (b) accept a growable stack with a runtime check. Almost everyone does (a),
and the reason is exactly this analysis.

**3.** Gains: no static maximum to compute, no max-depth field in the format, recursion
depth bounded by memory rather than by a per-frame reservation, and cheap fibers that start
tiny and grow. Owes: everything in A4 — every reference into the stack becomes an index or
must be fixed up on relocation, including open upvalues and any pointer a native extension
holds. And growth must be *checked*, which means either a check per push (unaffordable) or
one check per call against a statically computed maximum — at which point you need the
static maximum after all. The realistic design is both: compute the max statically, check
once on entry, grow if needed, and pay the fix-up cost when growth actually happens.
