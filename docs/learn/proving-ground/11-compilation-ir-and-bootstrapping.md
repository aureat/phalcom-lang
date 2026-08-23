# 11 — Compilation, IR, and Bootstrapping

Lowering, and building the thing that builds the thing. The through-line: *every phase
throws information away, and the whole art is deciding what to destroy and in what order.*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — Where a feature belongs

You are adding `for x in xs { body }`. Three places it could be handled:

- **A** — the parser emits the desugared while-loop directly; no `For` node exists.
- **B** — an AST `For` node survives type checking and is lowered on the way into the IR.
- **C** — the IR itself has a loop-with-iterator construct and the backend handles it.

1. Desugaring in the parser is the cheapest option and is almost always wrong. Name the two
   concrete artifacts that degrade, and say exactly how the user notices.
2. GHC desugars an enormous surface language into a tiny Core. Rust keeps a large HIR and
   lowers separately to MIR. Reconstruct why each is right for that language.
3. Give an example of an optimization that must **not** run before a particular lowering,
   and say what breaks if it does.

### Q2 — The honest ladder

Four implementations of one language: AST walker, bytecode VM, tiered JIT, AOT compiler.

1. AST walker to bytecode VM is usually a large speedup. It is *not* because "bytecode is
   closer to the machine." Name the three real sources, and rank them.
2. Given identical information, name the two things a JIT can do that a bytecode
   interpreter fundamentally cannot — "fundamentally", not "in practice".
3. For each rung of the ladder, name the one piece of machinery that becomes an order of
   magnitude harder, and say why it is the same piece of machinery each time.

### Q3 — What a phi node is for

```
      entry
     /     \
 x1 = 1   x2 = 2
     \     /
     x3 = phi(x1, x2)
     y  = x3 + 1
```

1. `phi` is not an executable instruction. Say what it is a notation for, and where the
   real operation lives.
2. SSA makes constant propagation *sparse*. Explain what "sparse" means here by describing
   what the pre-SSA analysis was forced to do instead.
3. Phi placement is computed from dominance frontiers. Give the intuition for why the
   frontier is exactly the right set — not the definition, the reason.

### Q4 — Getting out of SSA

Naively, destructing SSA means replacing each phi with copies at the end of each
predecessor block. Consider a loop whose header contains:

```
L:  x1 = phi(a, y1)
    y1 = phi(b, x1)
    ...
    goto L
```

1. Sequentialize those two copies in either order and the program is wrong. Show it, and
   name the fix.
2. Describe the lost-copy problem — the setup that produces it and why splitting critical
   edges is part of the answer.
3. Some compilers allocate registers *before* destructing SSA — Go's backend does — while
   LLVM, GCC, and HotSpot C2 all destruct first. State the argument for the SSA-first
   ordering, and name the problem it relocates rather than solves.

### Q5 — CPS, ANF, and SSA

Appel's claim: SSA is functional programming.

1. Give the correspondence precisely. Four pairs: basic block ↔ ?, phi node ↔ ?, jump ↔ ?,
   dominance ↔ ?
2. Name something CPS makes easy that SSA does not, and name a language whose feature set
   forces the choice.
3. ANF was proposed as CPS's cheaper cousin. Name what ANF gives up, and name the
   construct GHC had to add to Core to get part of it back.

### Q6 — Loops need a tree

```
for (i = 0; i < n; i++)
    sum += a[i] * k;      // k is loop-invariant
```

1. Hoisting `a[i] * k`'s invariant part out of the loop requires several facts. Name them,
   and say which one comes specifically from the dominator tree.
2. Natural loops are found via back edges — an edge `t → h` where `h` dominates `t`. Why the
   dominance condition? Construct a CFG with a cycle that this definition refuses to call a
   loop, and say what such CFGs are called.
3. Induction-variable strength reduction turns `&a[i]` into a running add. Name the analysis
   it requires and name something downstream it can make *worse*.

### Q7 — Two register allocators

HotSpot's C1 uses linear scan. Its C2 uses graph colouring. Both are correct; they ship in
the same VM.

1. Why is graph colouring affordable AOT and in a top tier, but not in a first tier? Be
   specific about what is superlinear.
2. Spilling is a separate problem from colourability. Explain why, and say what live-range
   splitting buys that spilling alone does not.
3. In a deopt-capable JIT, the allocator takes on an obligation an AOT allocator does not.
   Name it, and say what live-range splitting does to its cost.

### Q8 — Phase ordering

Two pipelines: inline-then-constant-propagate, and constant-propagate-then-inline.

1. Give a program where each order strictly wins.
2. LLVM ships a fixed pass pipeline with several passes appearing multiple times. Why not
   a fixpoint loop over all passes until nothing changes?
3. State the property that makes phase ordering unsolvable rather than merely expensive.

### Q9 — Inlining as the enabling optimization

A small callee, a hot call site.

1. Inlining is called the enabling optimization. Name four downstream optimizations it
   enables and, for each, say what specifically it needed inlining to supply.
2. Name the four costs of inlining, and say which is hardest to measure and why.
3. A JIT will inline a virtual call that an AOT compiler cannot. What makes that legal, and
   name the two pieces of infrastructure that must exist first.

### Q10 — Escape analysis

```java
Point p = new Point(x, y);
return p.x + p.y;
```

1. Scalar replacement and stack allocation are the two payoffs. HotSpot ships one and not
   the other. Say which, and give the reason it is the tractable one — the reason is about
   the *collector*, not about the stack.
2. Escape analysis dies at a call the compiler did not inline. Explain the mechanism, and
   state the consequence for pass ordering.
3. An object was scalar-replaced. Execution then deoptimizes, and the interpreter expects
   a real object. What must the compiler have recorded, and what does the runtime do at
   deopt time?

### Q11 — Knowing it early, and getting it wrong

Three things called "computing at compile time": constant folding, constant propagation
(SCCP), and partial evaluation.

1. SCCP is strictly stronger than iterating "constant-propagate, then delete dead code" to
   a fixpoint. Explain why, in terms of how the two lattices are initialized.
2. A peephole rewrites `x / 2` to `x >> 1` for a signed 32-bit `x`. It is a miscompilation.
   Say what is wrong, give the correct sequence, and give a second peephole that is legal
   in one language and a miscompilation in another.
3. State the first Futamura projection, then name the production system that is literally
   an implementation of it, and say what mechanism makes its specializations revocable.

### Q12 — What separate compilation gave away

Two translation units, a hot cross-module call.

1. Name what separate compilation forecloses. "Cross-module inlining" is one item on a list
   of at least five.
2. LTO recovers it by changing what an object file contains. Say what is in there instead,
   and name four things that get worse.
3. ThinLTO is the deployed answer at scale. What is in the summary, and what does it trade
   against full LTO?

### Q13 — Tiering, warmup, and on-stack replacement

```java
public static void main(String[] a) {
    long s = 0;
    for (int i = 0; i < 2_000_000_000; i++) s += f(i);
    System.out.println(s);
}
```

The loop is entered exactly once, before anything is compiled.

1. Explain why this program needs OSR, and name the hard part of implementing it — it is
   not "jump into the middle of compiled code."
2. Modern tiering has more than two levels (V8: interpreter, baseline, mid-tier, optimizing;
   HotSpot: interpreter, C1, C2). Say what each additional level exists to buy.
3. For a process that lives 200ms, the profile never gets good. Name three escapes the
   industry deployed, and what each gives up.

### Q14 — AOT with PGO versus a JIT

Same program, two toolchains: a JIT with online profiling, and an AOT compiler with a
profile from a representative training run.

1. Name what the JIT knows that AOT+PGO cannot know, even with a perfect training run. The
   deepest item on the list is not about profiles at all.
2. Name what AOT can do that a JIT cannot afford, and say why the constraint is structural.
3. GraalVM's native-image gets AOT plus a closed-world assumption. Name what closed world
   forbids, and what the ecosystem did in response.

### Q15 — You must bootstrap this

You have a compiler for language L, written in L. You have no L compiler.

1. Give the staged order. Then state what the standard three-stage comparison actually
   checks, and why the comparison is stage 2 against stage 3 rather than stage 1 against
   stage 2.
2. Go and Rust took different routes out of this problem. Describe each and name the trade.
3. You need a language change that the previous compiler cannot even parse. State the
   discipline, and name the mechanism a real compiler uses to implement it.

### Q16 — Trusting trust

Thompson's compiler recognizes when it is compiling the login program and inserts a
backdoor; it also recognizes when it is compiling *itself* and reinserts both behaviours.
The backdoor then persists with no trace in any source.

1. Say precisely why auditing the compiler's source cannot find it, and name the property
   the malicious code must have for the trick to survive.
2. Describe diverse double-compiling. State exactly what it proves and exactly what it
   assumes.
3. Reproducible builds and full-source bootstrap chains each contribute something different
   to this. Say what each contributes, and name the residual trusted base after both.

### Q17 — The bootstrap floor

Your runtime loads a core library written in the language itself. But in that file,
`class Point { ... }` is itself a message send to a class object, and `Object`'s class is
`Object class`, whose class is `Metaclass`, whose class is `Metaclass class`, whose class is
`Metaclass`.

1. Enumerate what must exist natively before the first line of that file can execute, and
   explain why the metaclass knot cannot be built by any ordered construction.
2. An error occurs during bootstrap — but the exception classes are defined in the file
   being loaded. State the discipline, including the failure mode you get if you ignore it.
3. Give the minimal primitive set that lets the rest be written in the language, and name
   the test that tells you the line was drawn in the wrong place.

### Q18 — Debugging what the optimizer produced

```
$ gdb ./a.out
(gdb) print x
$1 = <optimized out>
```

1. Name four distinct optimizations that damage debug information, and say what each one
   damages specifically.
2. Building with `-g` must not change the generated code. State why that is a hard
   requirement rather than a nicety, and name the class of compiler bug it creates.
3. A deopt-capable runtime can present a fully unoptimized view of a program running
   optimized code. Say what mechanism buys that, and what it costs the optimizer.

---

## Answers

### A1 — Where a feature belongs

**1.** **Diagnostics** and **debug information**. If the parser emits a while-loop, then
every later error is reported in terms of a program the user did not write: a type error on
the iteration variable points at a synthetic `.next()` call, a borrow error mentions a
temporary with no name in the source, and an exhaustiveness warning fires on a desugared
match arm. Spans can be forged onto the desugared nodes, but the *shape* is wrong, so
messages talk about the wrong construct. Second: the debugger and profiler map addresses
back to lines through the same lowering, so stepping over a `for` steps through invisible
code, and a breakpoint on the loop header lands somewhere the user cannot explain. The rule
that falls out: **desugar after every phase that reports to a human.**

**2.** GHC's surface language is enormous (do-notation, list comprehensions, guards, class
methods, deriving, arrows) but its *semantics* are uniform — everything means a Core term,
and Core is System FC with about a dozen constructors. A tiny Core means every optimization,
every analysis, and the type-checker for the intermediate language are written once, and
the surface can grow indefinitely for free. That is only affordable because GHC type-checks
the *surface* first, so the error messages are already produced before desugaring. Rust's
constraint is different: borrow checking, drop elaboration, and move analysis need an
explicit control-flow graph with explicit drops and explicit temporaries — facts that do not
exist in the surface language at all — but those analyses must report errors in *source*
terms. So Rust checks types on a high-level HIR that still resembles the source, then lowers
to MIR to acquire a CFG, and MIR carries span information back so borrowck can talk about
the user's variables. Two different answers because the analysis that needs the low-level
form is the one that must produce the errors.

**3.** Borrow checking must run on **unoptimized** MIR. If you optimize first, a
dead-store elimination or a copy propagation can remove the very borrow that made the
program illegal, so an incorrect program passes. More generally: any check whose job is to
reject programs must run before any transformation that could make an illegal program look
legal. Another shape of the same rule: constant folding of arithmetic must not run before
the language's overflow semantics are pinned down in the IR, or you will fold
`i32::MAX + 1` to a value in a language where it should trap.

### A2 — The honest ladder

**1.** Ranked: (a) **Operands are resolved at compile time.** In a naive AST walker, reading
a local variable is a name lookup in an environment — a hash lookup or a chain walk — on
every access. A bytecode compiler resolves it to a slot index once, and the runtime cost
becomes an array index. This is usually the single biggest factor and it is the one people
forget, because it is not about the instruction encoding at all. (b) **Dispatch is
amortized differently.** An AST walker pays an indirect call plus a pointer dereference per
*node*; a bytecode loop pays one dispatch per *instruction*, and instructions are chosen to
be coarser than nodes. (c) **Locality.** Bytecode is a contiguous byte array walked forward;
an AST is a pointer graph allocated at parse time in whatever order, so every step is a
potential cache miss. Note that (a) and (c) are properties of the *compilation*, not of the
*representation* — an AST walker that pre-resolves variables and flattens its nodes captures
much of the win, which is exactly why "closure compilation" AST interpreters are
competitive.

**Trap.** "Bytecode is faster because it's closer to the machine." Bytecode is not closer to
any machine — it is a different abstract machine, usually a stack machine no hardware
implements. Saying this reveals that you have never measured where an AST walker's time goes,
which is variable lookup and pointer chasing, both of which a better-compiled AST fixes
without any bytecode at all.

**2.** (a) **Speculate on facts that are true now but not provable** — this receiver has
always been a `String`, this field has always held a small integer, this class has no
loaded subclasses — and install a guard plus a deoptimization path. An interpreter can
observe those facts but cannot *depend* on them, because it has no mechanism to generate
code that would be wrong if they changed and no way to unwind it if they do. (b) **Remove
the dispatch entirely**, along with everything it forced: with the operation known, values
stay in registers across instruction boundaries, the operand stack disappears, and
arithmetic on a known type becomes one machine instruction. An interpreter always pays at
least one dispatch and one stack traffic round-trip per operation, no matter how good the
threading is; that floor is what a compiler removes.

**3.** Bytecode VM: **precise GC roots** become harder — you must be able to describe the
live set at any allocation point, which is easy while all values live in a known operand
stack and hard the moment they do not. JIT: **deoptimization**, which requires a full map
from optimized machine state back to interpreter state at every safepoint. AOT: **debug
information and stack unwinding** through inlined, scheduled, register-allocated code. It is
the same piece of machinery each time — the ability to **describe the abstract machine's
state at an arbitrary program point** — because that is what GC, deopt, debugging,
exceptions, and profiling all separately require. Each rung makes the mapping from concrete
state to abstract state less direct, and every one of those five features is a customer of
that mapping.

### A3 — What a phi node is for

**1.** It is notation for a **parallel copy performed on the incoming control-flow edge**.
The real operation lives at the end of (or on) each predecessor block: "when control leaves
predecessor *i*, move that predecessor's operand into `x3`'s location." All phis at the top
of a block execute *simultaneously* — they read all their operands before writing any
result — which is why they are drawn as a block of instructions with no ordering among them,
and why destruction is not a simple textual substitution (A4).

**2.** Pre-SSA constant propagation is a **dense** dataflow analysis: it computes a map from
*every variable* to a lattice value at *every program point*, and iterates over the CFG to a
fixpoint. Cost and memory scale with (variables × program points). Under SSA, a variable has
exactly one definition, so "what is `x3`'s value" is a property of the *definition*, not of a
point — there is one lattice cell per SSA name, and information flows along def-use edges
rather than along the CFG. That is what sparse means: the analysis walks the def-use graph,
touching only the places where the value actually appears, instead of every point in the
program. SSA does not make the analysis smarter; it removes the need to re-derive at every
point what was already known.

**3.** A phi is needed for `x` in block *B* exactly when two definitions of `x` from
different paths first become simultaneously reachable at *B*. "Definition in *D* stops being
the only one in force" happens precisely at the first block *B* that is reachable from *D*
but **not dominated by** *D* — because if *B* were dominated by *D*, every path to *B* goes
through *D* and *D*'s definition is the only one that can arrive. The set of such *B* is the
dominance frontier of *D*. So the frontier is not a clever heuristic; it is the literal
definition of "where *D*'s exclusivity ends," and phi placement is its transitive closure
because each inserted phi is itself a new definition with its own frontier.

**Trap.** "A phi picks the value based on which block you came from, so it's a runtime
select." It has no runtime existence and it does not test anything — there is no branch, no
condition, and nothing to select on. If a phi were a select, SSA would have made the program
slower rather than making the analyses cheaper. The value arrives because the predecessor
*put it there*; the phi only records that fact.

### A4 — Getting out of SSA

**1.** The phis are parallel: on the back edge, `x1` must receive the old `y1` and `y1` must
receive the old `x1` — a swap. Sequentializing as `x1 = y1; y1 = x1` leaves both holding the
old `y1`. The other order leaves both holding the old `x1`. This is the **swap problem**,
and the fix is to treat the phi group as a **parallel copy** and sequentialize it properly:
build the dependency graph among the copies, emit the copies that form a chain in dependency
order, and break each cycle with a **temporary** (or, in registers, an exchange/rotate). The
general routine is due to the Briggs et al. work on practical SSA construction and
destruction.

**2.** The **lost-copy problem** appears after copy propagation has extended a live range
past the point where the destruction copy wants to go. The setup: a loop whose phi operand
is a variable that is *also* live out of the loop (used after the loop ends). Destruction
wants to insert `x = y` at the end of the predecessor — but that predecessor block is also
on the path to the loop exit, so the copy overwrites the value the exit is going to read,
and the value that should have escaped the loop is lost. Splitting **critical edges** —
edges from a block with multiple successors to a block with multiple predecessors — gives
each phi operand its own block to put the copy in, so the copy executes only on the path it
belongs to and never on the exit path. Without the split there is simply nowhere correct to
put it.

**3.** The argument: the interference graph of a program **in SSA form is chordal**, and
chordal graphs are optimally colourable in polynomial time — so the NP-hardness of general
graph colouring (Chaitin's result) does not apply, and you get an allocator with a
principled optimality story instead of a heuristic one. What it relocates: once colours are
assigned, destruction is no longer "insert copies between virtual registers", it is "insert
copies and swaps between *physical* registers", so the parallel-copy sequentialization of
part 1 now has to be done with a limited supply of scratch registers — a cycle among three
physical registers with none free needs a memory temporary or a rotate. It also relocates
spilling: spill decisions must be made before or during colouring, since a spilled value
changes the interference structure. The elegance is real and the bookkeeping just moves.

### A5 — CPS, ANF, and SSA

**1.** Basic block ↔ **a (tail-recursive) local function / continuation**. Phi node ↔ **that
function's parameter**. Jump ↔ **a tail call** to it. Dominance ↔ **lexical scoping**: a
definition dominates a use exactly when, in the functional rendering, the binding is in
scope at the use — and the dominator tree is the nesting structure of the local function
definitions. This is Appel's point, and its practical value is that every SSA
transformation has a functional-program reading, so you can check a proposed SSA
transformation by asking whether the corresponding rewrite on the functional program is
sound.

**2.** **First-class control**: `call/cc`, delimited continuations, effect handlers,
generators, and guaranteed tail calls all become ordinary manipulations of a value, because
CPS has *already* reified the continuation. Exceptions become an extra continuation
argument instead of a special mechanism. A language whose feature set forces the choice is
Scheme — you cannot compile `call/cc` faithfully in an IR that assumes a control stack you
cannot name — and the same argument now applies to any language with effect handlers or
multi-shot continuations. SSA assumes a conventional stack discipline; that assumption is
free until the language stops obeying it.

**3.** ANF gives up **closure under transformation**. CPS is stable: beta-reduction of a CPS
term yields a CPS term. ANF is not — inline a function into an ANF program and the result
generally violates ANF's own grammar (a let-bound computation ends up in an argument
position), so you must re-normalize after every inlining, and re-normalization can duplicate
code. It also does not reify the continuation, so control operators remain special. The
construct GHC added to Core is **join points** — a marked binder for a continuation that is
only ever tail-called. Without them, the shared "what to do after this `case`" had to become
an ordinary function, which meant a heap-allocated closure on a path where a jump would do;
`join` lets Core name a continuation without paying for one. That is the compiler
re-acquiring, explicitly, one thing CPS had for free.

**Trap.** "CPS is obsolete; everyone uses SSA now." The correspondence in part 1 means they
are the same structure viewed from two sides, and the choice is driven entirely by the source
language: a compiler for a language with first-class control or effect handlers still reaches
for a continuation-based IR — SML/NJ is the standing example. But the correspondence cuts
both ways: MLton compiles Standard ML through **SSA**, and OCaml's Flambda is ANF, not CPS.
"Obsolete" here means "not what LLVM does," which is a statement about LLVM's input language.

### A6 — Loops need a tree

**1.** Facts required: the expression is **loop-invariant** (its operands are defined
outside the loop or are themselves invariant); it is **safe to speculate** (no trap, no
side effect, no possible fault — this is why `a[i]` cannot be hoisted but `k * 2` can); and
the target of the hoist — the **preheader** — must **dominate every use** so the hoisted
value is defined on every path that reads it. That last one is the dominator-tree fact. It
is also why LICM's first act is often to *create* a preheader block: the natural loop may
have several predecessors, and none of them individually dominates the body.

**2.** Because dominance is what guarantees a **single entry point**. A loop with one entry
has a header that every path into the body passes through, which is what makes "outside the
loop" and "before the loop" well-defined — and every loop transformation (LICM's preheader,
unrolling, rotation, induction-variable analysis) needs a place that is unambiguously before
all iterations. A cycle without such a header:

```
        entry
       /     \
      A  <->  B          (entry can jump to either A or B)
```

`A` and `B` each reach the other, but neither dominates the other, so neither edge is a back
edge. This is an **irreducible** CFG. It arises from `goto`, from some
`switch`-in-a-loop patterns, and routinely from decompiled or machine-generated code. The
standard responses are **node splitting** (duplicate one of the entries to create a single
header, at a code-size cost) or simply bailing out of loop optimizations for that region —
which is what many production compilers do, so irreducible control flow is a real, if
invisible, performance cliff.

**3.** It requires **induction-variable / scalar-evolution analysis**: recognizing that `i`
advances by a constant per iteration, and that `&a[i]` is therefore an affine function of
the iteration count, so it can be replaced by a pointer initialized before the loop and
incremented by the stride. What it can make worse: it **creates an extra live value across
the whole loop**, raising register pressure, which can turn into a spill in a loop with many
such candidates — the classic case where an "optimization" is a pessimization on a
register-poor target. It can also **defeat later pattern matching**: an address computed as
`base + i*4` is recognizable to an addressing-mode selector or a bounds-check-elimination
pass that reasons about `i`, and once it becomes an opaque running pointer, the relationship
to `i` is gone and the later pass loses. This is a phase-ordering instance (Q8): the
strength reducer destroyed the form a later pass needed.

### A7 — Two register allocators

**1.** Building the **interference graph** is the problem: it is quadratic in the number of
simultaneously live values in the worst case, both in time and in memory, and the
simplify/spill/select loop iterates over it, potentially re-building after each spill round.
Linear scan sorts live intervals once and sweeps, so it is O(n log n) with a small constant
and a predictable memory footprint. In a top tier or AOT, compile time is amortized over
every subsequent execution of the code (or over the whole product lifetime), so spending it
is obviously right. In a first tier, the entire point is to produce *acceptable* code
*immediately* — the tier exists to bridge the warmup gap, and an allocator whose cost is
quadratic in a large method defeats its own purpose. The choice is not about allocator
quality; it is about who pays and when.

**2.** Colourability asks a yes/no question about the whole graph; **spilling asks where in
the program to insert loads and stores**, which is a placement problem with a cost model
(frequency of the block, loop depth, whether the value is redefined) that the graph does not
contain. A graph can be uncolourable because of one long-lived value that conflicts with
everything, and spilling it wholesale is enormously wasteful if it is hot in one small
region. **Live-range splitting** breaks that value into several shorter ranges connected by
copies, so it can live in a register where it is hot and in memory where it is not, and the
interference graph gets easier at the cost of a few copies. Spilling alone is all-or-nothing
per value; splitting makes the granularity per-region.

**3.** The obligation is **location maps at every deopt point / safepoint**: for each such
point, a description of where every value the interpreter would need actually lives — this
register, that stack slot, or "it is the constant 7", or "it does not exist, rematerialize
it". AOT needs something similar for unwinding and debug info, but a JIT needs it to be
*executable*, because deopt reconstructs interpreter frames from it and resumes. Live-range
splitting makes this expensive: a value's location is no longer a property of the function
but a property of the program point, so the map is per-PC and grows with the number of
safepoints times the number of live values. This metadata is a substantial fraction of a
JIT's memory footprint, and it is why some values get pinned to a single location — the
allocator is trading code quality for metadata size.

**Trap.** "Linear scan is the fast-and-worse allocator; graph colouring is the good one."
The gap is much smaller than the reputation, because modern linear-scan variants split live
ranges and use spill costs, and because on SSA form the colouring problem was never the hard
part (A4). The real differentiator between allocators in production is **spill placement and
splitting policy**, not the colouring algorithm — and an answer that ranks them by algorithm
name suggests reading about allocators rather than profiling one.

### A8 — Phase ordering

**1.** Inline-then-propagate wins when the constant is at the *call site*:

```c
int scale(int x, int f) { if (f == 1) return x; return x * f; }
... scale(y, 1) ...
```

Only after inlining does `f == 1` become foldable, the branch resolvable, and the multiply
removable. Propagate-then-inline wins when the constant is what makes the callee small
enough to be worth inlining, or when propagation kills the call entirely:

```c
if (DEBUG) log_expensive(build_giant_message());   // DEBUG is a known-0 global
```

Propagating `DEBUG = 0` first deletes the whole branch; inlining first would first inline
`build_giant_message` and `log_expensive`, blowing the size budget and possibly causing the
inliner to *decline* to inline something that mattered elsewhere.

**2.** Because passes are not **monotone** and not **confluent**. A fixpoint loop presumes
that applying more transformations never makes things worse and that the order does not
change the final state; neither holds. Reassociation can destroy a form the idiom recognizer
matched; vectorization can defeat a later scalar simplification; an unroll can push a loop
over the inlining budget. Two passes can also *undo each other* (a sinking pass and a
hoisting pass with different cost models), so the loop may not terminate at all. And there
is a hard budget: a fixpoint over the full pass list on a large module would take
unacceptable compile time for a marginal gain. LLVM's repeated passes are the pragmatic
compromise — run the cheap simplifiers again where experience says a preceding pass tends to
create new opportunities.

**3.** The transformations do not form a **lattice with a join**: there is no "best" result
you can converge on, because two transformations can each be individually profitable and
mutually exclusive. Choosing an order is therefore a search over a combinatorial space
whose objective function (actual runtime on actual inputs) is only measurable by running the
program. That makes it a genuine optimization problem, not a scheduling problem — which is
why the literature on it is empirical (iterative compilation, autotuning, learned pass
orderings) rather than analytic, and why every production compiler's pipeline is a
hand-tuned artifact with historical accidents in it.

### A9 — Inlining as the enabling optimization

**1.** (a) **Constant propagation into the callee** — needs the *actual argument values* at
the call site, which only exist once the bodies are merged. (b) **Escape analysis** — needs
to see every use of the allocated object; an un-inlined callee that receives the pointer is
an unknown use (A10). (c) **Devirtualization of the callee's own calls** — needs the
receiver's concrete type, which frequently comes from an allocation in the *caller*.
(d) **Redundancy elimination / CSE and load-store forwarding across the boundary** — needs
the caller's and callee's memory operations in one region so alias analysis can relate them;
a call is otherwise an opaque memory clobber. Add to those: dead-argument elimination, and
loop optimizations on a loop whose body contains a call.

**2.** Costs: **code size** (and therefore instruction-cache pressure and TLB pressure, which
can make the program slower with no other change), **compile time** (superlinear, because
each inlined body is optimized again in each new context), **register pressure** (a bigger
region has more simultaneously live values, so more spills), and **debug/profile fidelity**
(inlined frames must be reconstructed, and the profile attribution becomes ambiguous). The
hardest to measure is **I-cache pressure**, because its cost is not local to the inlined
site — it appears as a slowdown in *unrelated* code that got evicted, so a microbenchmark of
the inlined function shows an improvement while the whole application regresses. That
non-locality is exactly why inlining heuristics are tuned empirically on whole applications
and why "just raise the threshold" reliably fails.

**3.** It is legal because the JIT does not inline *unconditionally*: it emits a **guard**
(a check on the receiver's class, or a check on a global assumption) and a **deoptimization
path** for when the guard fails. It can also use class-hierarchy analysis — if only one
implementation of the method is currently loaded, it can inline with *no* per-call guard at
all. The two pieces of infrastructure: (a) **deoptimization**, so a failed speculation has
somewhere to go; (b) **dependency tracking / code invalidation** — HotSpot records that a
compiled method depends on "class C has no loaded subclass overriding m", and the class
loader invalidates that compiled code when a violating class is loaded. AOT has neither, so
it can only devirtualize what it can *prove* — which is why closed-world AOT (native-image)
can devirtualize aggressively and open-world AOT essentially cannot.

**Trap.** "The JIT wins because it has a profile." The profile is the smaller half.
The bigger half is that a JIT may act on facts that are *currently* true, because it can
undo the action; AOT may only act on facts that are *always* true. That asymmetry is a
capability difference, not an information difference, and it survives even if you hand the
AOT compiler a perfect profile.

### A10 — Escape analysis

**1.** **Scalar replacement** — explode the object into its fields as ordinary SSA values,
and the allocation vanishes entirely. It is easier because a stack-allocated object is still
an *object*: it needs a header, it must be scannable by the collector (so it needs a stack
map entry describing it), references from it to heap objects must be traced, and its
lifetime must be tied to a frame that deoptimization and exceptions can unwind. Scalar
replacement asks nothing of the collector at all — there is no object, only values in
registers — so it needs no GC changes, no header, no stack map, and it composes with every
other SSA optimization for free. The collector is the reason, and the general rule it
illustrates: the cheapest way to make the GC handle something is to make the something stop
existing.

**2.** The mechanism: passing the reference to a callee the compiler cannot see means the
callee could store it in a static field, a heap object, or another thread's structure. With
no body to analyse, the conservative conclusion is "escapes globally," and the allocation
must be materialized. The consequence for ordering is direct: **escape analysis must run
after inlining**, and its effectiveness is bounded by the inliner's budget — an allocation
whose only escaping use is a call the inliner declined by two bytecodes stays on the heap.
This is a major reason inlining heuristics matter more than they look like they should, and
why "why didn't my object get scalar-replaced" is nearly always answered by "this call
wasn't inlined."

**3.** The compiler must have recorded, in the deopt metadata for that program point, a
**rematerialization descriptor**: the object's class, and for each field, where the value
currently lives (register, stack slot, or constant). At deopt time the runtime **reallocates
the object on the heap**, writes the fields from those locations, and installs the reference
into the reconstructed interpreter frame — HotSpot does exactly this, and it also has to
**re-acquire any locks** that were eliminated by lock elision on that object, in the right
order. Note what this implies: deopt can *allocate*, which means deopt can trigger a GC,
which means the deopt path itself must be GC-safe. The interaction between speculative
optimization and the collector is a place where the abstractions stop being separable.

### A11 — Knowing it early, and getting it wrong

**1.** SCCP (Wegman and Zadeck) initializes **optimistically**: every SSA value starts at
TOP ("assume constant until proven otherwise") and every basic block starts **unreachable
until proven reachable**, and the algorithm only ever lowers values. Iterated
constprop-then-DCE is **pessimistic**: values start at BOTTOM (unknown) and are raised only
when proven. The difference shows on **cyclic dependencies**, which is exactly what a loop
is:

```
i = 1
loop: if (i != 1) goto exit
      i = 1
      goto loop
```

`i`'s phi at the loop header takes an operand from the back edge. Pessimistically, that
operand is unknown on the first pass, so the phi is unknown, so the branch is not
resolvable, so the back edge stays live, and no later iteration can recover — the analysis
has already committed to "unknown". Optimistically, the back edge is initially *unreachable*,
so the phi sees only the entry operand and is 1; the branch folds; the back edge is never
marked reachable; and `i` is proven constant. The fixpoint you reach depends on which end you
start from, and only the optimistic one can conclude a value is constant *because* a branch
is dead *because* the value is constant. Iterating the pessimistic version any number of
times cannot recover this.

**2.** Arithmetic shift right rounds toward negative infinity; C, C++, Java, and Rust
integer division truncates **toward zero**. So `(-3) / 2` is `-1` but `(-3) >> 1` is `-2`.
The correct sequence adds a bias for negatives first — conceptually
`(x + (x >>> 31)) >> 1` for 32-bit, i.e. add 1 before shifting iff `x` is negative, using an
unsigned shift to extract the sign bit branchlessly. A second peephole that flips languages:
**`x + 1 > x` → `true`**. Legal in C and C++ for signed `int`, because signed overflow is
undefined behaviour and the compiler is entitled to assume it does not happen; a
miscompilation in Java or in Rust's release-mode wrapping arithmetic, where overflow is
*defined* to wrap and `INT_MAX + 1 > INT_MAX` is genuinely false. The lesson worth stating:
a peephole is not a fact about arithmetic, it is a theorem in a specific language's
semantics, and the reason it is a favourite home for miscompilations is that everyone
believes they already know the semantics.

**3.** **First Futamura projection**: specializing an interpreter with respect to a fixed
source program yields a compiled program — `spec(interp, src) ≡ compiled_src`. The
production system that is literally this is **Truffle/GraalVM**: you write an AST
interpreter, and Graal's partial evaluator specializes it with respect to a particular AST,
inlining the entire interpreter loop for that program's node graph and constant-folding away
every dispatch, producing machine code for the guest program from an interpreter you wrote
in Java. The "compile-time constant" being specialized on is **the AST itself plus values
marked as stable** (`@CompilationFinal`, `Assumption` objects, profile-informed node
specializations). Revocability comes from **deoptimization**: an `Assumption` can be
invalidated at runtime, which discards the compiled code and drops execution back into the
AST interpreter, where the node re-specializes and the whole thing is partially evaluated
again. That is why the technique works for dynamic languages at all — the specialization is
allowed to be wrong, because there is a mechanism for being wrong.

**Trap.** "Constant folding and constant propagation are the same thing at different scopes."
Folding is a *rewrite* on an expression whose operands are already literal; propagation is a
*dataflow analysis* that discovers which operands are literal in the first place. The
distinction is load-bearing: folding has no lattice, no fixpoint, and no notion of
reachability, which is exactly why it cannot discover the loop-carried constant in part 1 no
matter how many times you run it.

### A12 — What separate compilation gave away

**1.** Cross-module **inlining**; cross-module **devirtualization** (you cannot do class
hierarchy analysis on a hierarchy you cannot see); **whole-program alias analysis** (a call
to an external function is an unknown memory clobber, which sinks every load-store
optimization across it); **global dead code and dead field elimination** (you cannot prove a
public symbol is unused); **cross-module constant propagation and function specialization**;
and **layout decisions** — hot/cold function splitting, function ordering to improve I-cache
and TLB behaviour, and struct field reordering. It also *pins* an ABI at every boundary:
calling conventions, struct layouts, and vtable shapes become part of the contract, which
forecloses representation changes forever, not just optimization opportunities.

**2.** The object file contains **serialized IR** (LLVM bitcode) instead of, or alongside,
machine code, and code generation is deferred to link time when the whole program's IR is
available. Four things that get worse: **link time and link-time memory** (the linker now
runs an optimizer over the entire program, which is where the multi-gigabyte-RSS link
failures come from); **incrementality** (a one-line change can invalidate optimization
decisions across the program, so the "rebuild one file" model degrades); **debuggability and
reproducibility of performance** (a function's generated code now depends on the whole
program, so a benign change elsewhere can move it); and **parallelism** — the whole-program
analysis stage is inherently serial and needs all the IR resident at once. Codegen itself can
be partitioned (GCC's LTRANS stage under WHOPR, LLVM's `-flto-partitions`), so the honest
claim is that the *analysis* does not parallelize and the memory ceiling is what bites.

**3.** ThinLTO puts a compact **per-module summary** in the object file: the module's symbol
definitions and references, a call graph with rough profile/size information, and enough
metadata to decide what is worth importing. The linker merges these into a global index,
decides which functions each module should import from which other modules, and then each
module is optimized **in parallel**, pulling in only its imports. The trade: import
decisions are made from summaries rather than full IR, and each module sees only a slice of
the program, so it inlines and specializes less aggressively than monolithic LTO — but it is
parallel, memory-bounded, and incrementally cacheable, which is what makes it usable on
programs the size of a browser or a Rust release build. It is the standard "give up the last
few percent to make it scale" trade, and it is the right one.

### A13 — Tiering, warmup, and on-stack replacement

**1.** The loop is entered once and never re-entered, so the ordinary compilation trigger —
"this *method* has been called N times, compile it and use the compiled version on the next
call" — never fires usefully: by the time the counter trips, control is already inside the
interpreted frame and will not leave it for hours. OSR exists to replace the *running* frame.
The hard part is **state translation at a loop header**: you must take an interpreter frame
(operand stack, locals, monitors, `bci`) and construct the corresponding optimized frame
(values in registers and stack slots as the allocator assigned them), which means the
optimized code needs a **special entry point at that loop header** with a matching state
map, and it means the optimizer's freedom is constrained — it cannot have hoisted a
computation out of the loop into code that OSR entry skips over, unless the OSR entry stub
recomputes it. In practice compilers generate a *separate* OSR version of the method,
compiled with the loop header as the entry, precisely because the constraint is awkward
enough that reusing the normal compilation is worse.

**2.** The **interpreter** exists to start instantly and to be small enough to be correct;
it also collects the first profile. The **baseline** compiler (V8's Sparkplug, HotSpot's
C1 in its profiling mode) exists to remove interpreter dispatch overhead at near-zero
compile cost, for code that is warm but not hot — most code in a large application never
becomes hot enough to justify the optimizing compiler, and running it in the interpreter
forever is the actual cost. The **mid-tier** (Maglev, or C1 with full profiling) exists
because there is a large middle: code hot enough that dispatch overhead matters, but where
the optimizing compiler's compile time and memory would not repay themselves. The
**optimizing tier** exists for the small set of genuinely hot methods where unbounded compile
time is worth it. Each tier is answering a different point on the "how many more times will
this run" distribution, and the distribution is heavy-tailed, which is why one or two tiers
leave money on the table at both ends.

**3.** (a) **AOT compilation of the whole program** (GraalVM native-image, .NET
ReadyToRun/NativeAOT): zero warmup, gives up peak throughput and, in the closed-world case,
dynamic language features. (b) **Persisting work across runs** — class-data sharing
(AppCDS), cached profiles, or tiered-AOT where the AOT code is later replaced by JIT code:
gives up simplicity and requires a training/dump step that must be kept in sync with the
deployed artifact. (c) **Checkpoint/restore** (CRaC, CRIU-based approaches): snapshot the
process *after* warmup and restore it per request; gives up a lot — the snapshot embeds open
file descriptors, network state, random seeds, and time, so applications must be modified to
participate in checkpoint notifications, and the security story around snapshotted secrets
is its own problem. There is a fourth honest answer: **do not use a JIT runtime for 200ms
processes**, which is why so much serverless code is in languages that never had one.

**Trap.** "OSR is just deoptimization run backwards." They share the state-map machinery but
not the difficulty. Deopt goes from optimized to interpreted, and the interpreter can accept
*any* state you hand it — it is the general machine. OSR goes the other way, into code that
was compiled under assumptions the interpreted frame never established, and the optimized
frame will only accept states its entry point was compiled to expect. One direction targets
a universal receiver; the other targets a specialized one, and that asymmetry is the whole
implementation cost.

### A14 — AOT with PGO versus a JIT

**1.** (a) **This process's constants** — a configuration flag read from the environment at
startup, the set of classes actually loaded, a plugin that is or is not present. These are
constant for the run and unknowable at build time. (b) **Phase behaviour within a run** — a
call site that is monomorphic on type A during initialization and monomorphic on type B
during steady state looks polymorphic in an aggregated training profile and monomorphic to a
JIT that recompiles. (c) **Code that did not exist at build time** — dynamically loaded
classes, generated proxies, `eval`. The deepest item is none of these: it is that a JIT may
act on facts that are **merely true so far**, because deoptimization gives it a way to be
wrong. AOT may only act on facts that are **provably always true**. That is a difference in
the *kind* of premise each is allowed to use, and no quality of profile closes it.

**2.** **Analyses whose cost is unbounded**: full graph-colouring register allocation with
live-range splitting, deep interprocedural analysis over the whole program, aggressive
search-based instruction scheduling, superoptimization of hot sequences, and profile-guided
whole-program layout (function ordering, hot/cold splitting, BOLT-style binary rewriting).
The constraint is structural, not a matter of effort: a JIT's compile time is *on the
program's critical path* — every cycle spent compiling is a cycle the application is not
running, and it is spent again in every process. AOT's compile time is paid once, offline,
by a build machine, and amortized over every execution of the artifact for its whole
lifetime. Those are different economies, and they lead to different algorithms even when the
compiler is literally the same codebase.

**3.** Closed world forbids **arbitrary reflection** (any class/method/field reached only by
name at runtime), **dynamic class loading**, **dynamic proxies**, runtime **bytecode
generation**, most **JNI** patterns, and `MethodHandle` construction from runtime data —
because the whole point is that the reachability analysis must see every possible callee in
order to eliminate everything else and to devirtualize. The ecosystem's response was to move
the work to build time: **reachability metadata** (JSON configs listing reflectively accessed
elements), **tracing agents** that run the application to generate that config,
**build-time initialization** of static state, and — the significant one — **frameworks
rewriting themselves** so that the work they used to do by reflection at startup (Spring's
component scanning, Hibernate's proxies, Quarkus's whole model) happens during the build and
emits static code. That is the real story: closed-world AOT did not just change a compiler
flag, it forced a generation of frameworks to stop using reflection as their architecture.

### A15 — You must bootstrap this

**1.** Stage 0: a compiler for a *subset* of L, written in some other language — or an
existing binary of an older L compiler. Stage 1: compile the L-compiler's source with stage
0. Stage 2: compile the same source with stage 1. Stage 3: compile the same source with
stage 2. The comparison is **stage 2 against stage 3**, and they must be bit-identical.
The reason it is not 1 against 2: stage 1's *binary* was produced by the bootstrap compiler
(different codegen, different optimizations, possibly a different implementation entirely),
while stage 2's binary was produced by stage 1 — the new compiler. So stage 1 and stage 2
legitimately differ as *artifacts* even when everything is correct. But stage 2 and stage 3
are the same source compiled by compilers that are semantically identical (stage 1 and stage
2 are both "the new compiler", differing only in who built them), so their outputs must
match. A mismatch means the compiler's output depends on the compiler that built it — which
means either a miscompilation or a nondeterminism bug. GCC's `make bootstrap` performs
exactly this comparison and it is a standard way both classes of bug get caught.

**2.** **Go** made a clean break: at 1.5 they mechanically translated the C-written
compiler into Go, so the toolchain became self-hosted in one step. Releases then bootstrapped
from Go 1.4 — the last C-implemented release — for years, and the floor now advances roughly
annually (Go 1.24 and 1.25 require Go 1.22), so the chain is deliberately kept a few hops
long rather than growing per release. The trade: a large one-time engineering cost, initially
worse-quality generated code (the translation was faithful, not idiomatic), in exchange for
a short, comprehensible chain and no permanent C dependency. **Rust** built a chain: each
release is compiled by the previous release's binary, going back through many releases to
the original OCaml-written compiler. The trade: no flag day and continuous progress, but the
provenance chain is now long, so bootstrapping Rust from source alone means replaying a long
sequence of releases — which is why `mrustc` exists (a C++ implementation that compiles an
old `rustc`, cutting the chain short for distributions that require a source bootstrap). One
optimized for a clean history, the other for never stopping; both are defensible and both
consequences were foreseeable.

**3.** The discipline is the **N-1 rule**: the compiler's own source must always be
compilable by the *previous* release. So a language change ships in two steps — first a
release that **accepts and implements** the new construct without the compiler's own source
using it; then, one release later, the compiler's source may use it. The mechanism a real
compiler uses is a **bootstrap conditional**: `rustc` compiles its own source with
`#[cfg(bootstrap)]` / `#[cfg(not(bootstrap))]`, so a single source tree can be built either
by the stage0 compiler — which is the current *beta*, not the last stable — or by the in-tree compiler
(taking the new one), and the old arm is deleted in the following cycle. Without such a
mechanism your only options are to keep a binary blob or to freeze the language.

**Trap.** "Self-hosting proves the language is mature." It proves the language can express a
compiler, which is a statement about one workload — heavy on data structures and pattern
matching, light on numerics, concurrency, FFI, and I/O. Plenty of languages self-hosted early
and were nowhere near production. The interesting property self-hosting actually buys is that
the implementers become their own users on their largest codebase, and that feedback loop is
the real argument for it.

### A16 — Trusting trust

**1.** Because the malicious behaviour lives in the **compiler binary**, not in the compiler
source, and the binary is what compiles the next compiler. You can read the source line by
line and find nothing, because there is nothing there: the source is clean, and the binary
inserts the payload each time it compiles that clean source. The required property is
**self-reproduction under compilation** — the backdoor must recognize the compiler's own
source and emit a version of itself into the output, which is a quine-like construction: the
compiled compiler must contain the code to re-emit the code that emits it. Thompson's essay
builds exactly this in stages, and the point of the staging is to show it is engineering,
not magic.

**2.** **Diverse double-compiling** (Wheeler). Let `S` be the compiler's source, `A` the
suspect compiler binary, and `B` any *other* compiler for the same language, from an
independent lineage. Compute `stage1 = B(S)` — the compiler's source built by the trusted
outsider — then `stage2 = stage1(S)`. Now compare `stage2` against `A` itself, bit for bit.
The second compile is what makes the test meaningful: `stage2` was produced by a compiler
that was itself built from `S`, so if `A` is honest it must be the same artifact. What it
**proves**: the suspect binary `A` corresponds to its purported source `S` — nothing lives
in the binary that is absent from the source. Note the suspect binary must be *one side of
the equality*; comparing two derived compilers against each other proves nothing, because a
payload that backdoors `login` without reproducing itself into the compiler would leave both
sides clean. What it **assumes**: that the build is deterministic and reproducible (else the
comparison is meaningless), that `S`'s semantics are deterministic, that `B` is not
subverted in a way that happens to produce the *identical* payload (independent lineage is
the mitigation), and that the environment running the comparison is itself trustworthy.

**3.** **Reproducible builds** contribute the *precondition*: without bit-identical outputs
from identical inputs, no comparison — DDC's, stage-2-vs-stage-3, or a distribution's
rebuild check — can distinguish an attack from a timestamp. They make the equality test
meaningful. **Full-source bootstrap** (the Bootstrappable Builds work: `hex0`/`stage0`,
GNU Mes, TinyCC, then GCC — Guix's default bootstrap path since 2023, with an equivalent
effort still in progress in Nixpkgs) contributes the *reduction of the
seed*: instead of trusting a multi-hundred-megabyte compiler binary, the trusted binary is
a few hundred bytes of hex that a person can genuinely audit by hand, and everything above it
is built from source. Residual trusted base after both: the **hardware** (Thompson's own
closing observation — the attack works at any level, including microcode and silicon), the
**kernel and the loader** that run the seed, the **assembler of last resort** if one is
outside the chain, and the **human** who read the hex. You cannot get to zero; you can get
the trusted set small enough to be examinable, and that is the actual goal.

### A17 — The bootstrap floor

**1.** Before the first line: an **allocator** and a fixed **object header layout**; a
**class object representation** that can be allocated *before any class exists*; a **method
dictionary** structure and a **symbol interner** (selectors must exist before methods can be
looked up); the **primitive set** the file's literals require — small integers, strings,
arrays — because a literal in that file has to become an object of some class; the
**parser and compiler** themselves; and a **send mechanism** that can find a method without
consulting any user-installable lookup. The metaclass knot cannot be built by ordered
construction because the class-of relation has a **cycle**: `Metaclass class`'s class is
`Metaclass`, which is also (transitively) the class of `Metaclass class`. Any construction
order requires some object's class pointer to be filled with an object that does not yet
exist. The only way through is to separate allocation from initialization: **allocate the
core class objects raw**, with null or placeholder class pointers, then **patch the pointers
in a second pass** once all the objects have addresses. This is how Smalltalk images are
built and it is why the bootstrap is a distinct program rather than a prelude — it has to
write object fields directly, below the level at which the language's own semantics hold.

**2.** The discipline: there must be a **pre-language failure path** — a native abort that
formats a message with no allocation of language-level objects and no message send — and a
**defined switchover point** after which failures are reported through the language's own
exception machinery. Concretely: (a) nothing early in the core file may depend on a class
defined later in it, and that ordering constraint should be checked, not assumed; (b) the
loader records a flag ("core installed") that flips the error path; (c) any primitive that
can fail before the switchover must have a native fallback message. The failure mode if you
ignore this is **secondary failure hiding the primary**: an error during bootstrap tries to
raise an exception, exception construction requires a class that is not installed yet, that
lookup fails, which tries to raise an error, and you get either infinite regress or a crash
whose reported cause is "cannot find class `Error`" — which tells you nothing about the
actual bug fifty lines earlier. The rule generalizes: **never report a failure through the
machinery that failed.**

**3.** The minimal set is roughly: **allocation**, **identity comparison**, **class-of**,
**machine-integer arithmetic and comparison**, **string and symbol creation plus symbol
equality**, **indexed slot read/write** (arrays), and the **send** primitive itself. Almost
everything else — collections, control-flow protocols, printing, iteration, comparison
protocols — is expressible above that line. Two tests that the line was drawn wrong. First,
**circularity on a hot path**: if a method written in the language ends up recursively
invoking the primitive it is supposed to be implementing (an `Integer` addition in the
language that needs addition to compute an index), the boundary is inconsistent and it will
manifest as a stack overflow at startup or a subtle infinite regress. Second, and more
insidious, **census drift**: the set of primitives the library actually depends on and the
set the runtime actually provides are two lists that will diverge, silently, within months —
someone adds a primitive for convenience, someone else stops using one. If the only thing
holding them together is a comment or a document, the floor rots and nobody notices until a
port to a new backend fails. It has to be an enforced invariant check that runs in CI, and
the check must be *derived* from the code rather than maintained by hand, or it becomes the
next stale document.

**Trap.** "You can avoid the bootstrap problem by writing the core library in the host
language instead." You move it, you do not avoid it. The moment the core library can be
*extended* by user code — subclassing `Object`, adding a method to `Integer` — the same knot
reappears as a consistency requirement between the natively-built classes and the
language-built ones, and now you have two construction paths that must produce identical
object shapes. The bootstrap is not an artifact of self-hosting; it is what a reflective
object model costs.

### A18 — Debugging what the optimizer produced

**1.** (a) **Inlining** damages the *call stack*: a single PC now corresponds to several
nested source frames, so the debugger must be told the inline tree — DWARF's
`DW_TAG_inlined_subroutine` records exist for exactly this, and without them a backtrace
lies about who called whom. (b) **Register allocation and live-range splitting** damage
*variable locations*: a variable lives in different places at different PCs, so a single
"variable is at offset N" entry is wrong and you need location *lists* keyed by PC range.
(c) **Instruction scheduling and code motion** damage the *line table*: instructions from
several source lines interleave, so stepping jumps backwards and forwards, and a breakpoint
on a line may execute code from three lines. DWARF's `is_stmt` flag is a partial mitigation.
(d) **Dead-value and store elimination** damage *availability*: a variable that is never
loaded after a point has no location at all, which is what `<optimized out>` means — the
value genuinely does not exist anywhere in the machine.

**2.** Because if `-g` changes codegen, then the program you debugged is not the program you
shipped, and every heisenbug becomes unfalsifiable — you cannot reproduce a production
failure under a debugger, which is the entire use case. It also breaks build reproducibility
between debug-info and stripped builds. The class of bug it creates is **debug info
affecting optimization**: a pass that counts instructions, checks `hasOneUse()`, or walks a
basic block without skipping debug intrinsics will behave differently when debug records are
present, so the optimization silently stops firing in `-g` builds. LLVM treats this as a
correctness bug class and has tooling (`-debugify` and its variants) that inserts synthetic
debug info and verifies both that it survives the pipeline and that it did not perturb the
output. The dual bug is a pass that transforms code and *drops* the debug info instead of
updating it, which is how variables silently become `<optimized out>` in code where they are
obviously live.

**3.** The mechanism is that the runtime maintains, at every **safepoint**, a complete map
from optimized machine state back to the **abstract (interpreter) state** — the same
metadata deoptimization needs (A7, A10). Given that map, a debugger request that the
optimized view cannot answer is handled by **forcing a deoptimization**: the frame is
rewritten into interpreter frames, and from then on the method executes interpreted, where
every local exists, every frame is real, and stepping is exact. This is the SELF and HotSpot
answer — Hölzle, Chambers, and Ungar's "debugging optimized code with dynamic
deoptimization" — and it is why a JVM can offer full-speed execution and full-fidelity
debugging without a separate debug build, which a C toolchain fundamentally cannot.

What it costs the optimizer: **safepoints are barriers**. Values that are live in the
abstract state must be recoverable at every one, which constrains code motion across them
and inflates metadata (that metadata is a real fraction of a JIT's memory). The optimizer
may still delete and reorder aggressively *between* safepoints, but it must be able to
reconstruct the abstract state *at* them — including rematerializing scalar-replaced objects
and re-acquiring elided locks. The trade is precise and worth naming: you give up some
freedom at a bounded set of points, and in exchange the observable model of the program
never diverges from the source, no matter what the code generator did in between.
