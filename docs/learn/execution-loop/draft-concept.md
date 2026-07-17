# The bytecode interpreter execution loop

## A software CPU, and the two pointers worth tracking

A compiler's real output, whatever the compiler's language, has a shape most people
underrate: it is a *list*. Not a tree, not a graph — those existed earlier in the
pipeline and got flattened. By the time you have "bytecode," you have a linear
sequence of primitive operations, plus operand slots wired up so values can move
between them. Everything downstream of that fact is implementation detail, but it
is implementation detail with teeth, because the flattening is what makes the next
step possible at all: you can walk a list with a single moving pointer. You cannot,
without extra bookkeeping, walk a tree that way.

The loop that does the walking has a name older than any software VM: the
**fetch-decode-execute cycle**, borrowed wholesale from hardware CPU design (the
"instruction cycle" of a von Neumann machine). That borrowing is not a metaphor —
it is the precise correct description of what a bytecode interpreter is. It is a
CPU, implemented in software, whose instruction set nobody has ever built silicon
for. **Fetch**: read the instruction at the current instruction pointer. **Decode**:
figure out which operation that instruction names. **Execute**: perform the
operation, which mutates the machine's state — and as part of executing, the
instruction pointer moves, either by a fixed default (fall through to "next") or
because the instruction itself was a jump. Every bytecode VM's main loop is a
restatement of this cycle; the design choices in this document are all choices
about *how* to restate it.

The earliest fully worked example most historians point to is **Smalltalk-80**, as
specified in Goldberg and Robson's 1983 "Blue Book" (*Smalltalk-80: The Language
and its Implementation*). **[flagged — chapter/part numbering from memory, not
verified]** The book's virtual-machine specification is, notably, not an
afterthought or an implementation note: it *is* the spec, given as literal
algorithmic pseudocode precise enough that multiple independent groups ported
Smalltalk-80 to different hardware from the book alone. Whether that pseudocode
was ever literally rendered as a C `switch` in the original Xerox PARC system is
a separate, murkier question — the earliest Smalltalk-80 systems ran partly on
custom microcode, and portable C implementations came later.
**[flagged — moderate confidence; implementation-language lineage of early
Smalltalk-80 systems is not something I can state precisely]**. What is not in
doubt is the *shape* of the algorithm it specifies: read a bytecode, branch on
its value to the right case, perform that case's effect, advance, repeat. That
shape — later crystallized in C as the now-idiomatic

```c
while (true) {
    switch (*ip++) {
        case OP_ADD: ...
        case OP_JUMP: ...
    }
}
```

— is not any one language's invention so much as the natural translation of
"software CPU" into whatever structured-programming primitives the implementation
language offers. It shows up independently, convergently, in nearly every
bytecode VM built since, because it is close to the only sane way to write "walk
a flat list, branch on what you find" in a language with structured control flow.

Here is the grip this document hands you, and the notation to hold it with. At
every point below, exactly two pieces of state matter: **where the instruction
pointer (`ip`) is pointing**, and **what shape the operand/frame stack is in**.
Every design axis that follows — stack vs. register, switch vs. threaded dispatch,
end-of-array halt vs. frame-drain halt — is a different answer to "what happens to
those two things after one trip through the loop." Hold onto `ip` and "the stack"
as your two variables and nothing below should ever spiral past what you can track
in your head. Traces in this document are written as small tables — `ip`,
instruction, and stack-after, stack drawn left-to-right with the rightmost element
as top-of-stack (TOS) — reused with the same convention throughout.

## The machine model: where do operands live between instructions

The first design axis is not about the loop's control flow at all — it's about
the *data format* the loop consumes. When one instruction produces a value and a
later instruction needs it, where does that value sit in the meantime? There are,
broadly, two real answers, plus a third option that isn't really an answer — it's
declining to compile to bytecode in the first place.

### Stack machines

A stack machine keeps operands on an implicit **operand stack**. Instructions
don't name their inputs or outputs at all — `Add` doesn't say "add what to what
and put it where"; it says only "pop two, push their sum." The instruction is,
in the JVM/CPython/Wren sense of the word, a zero-operand instruction, even
though it clearly has operands in the mathematical sense — they're just not
*encoded in the instruction*, they're wherever the top of the stack happens to
be when the instruction executes. Loading a value still needs an instruction
(`Load a` has to say *which* variable), but combining values doesn't.

This is a genuinely good deal, not a strawman to be knocked down: the compiler
that emits stack bytecode from an expression tree is almost embarrassingly
simple — a straightforward post-order tree walk emits exactly the instruction
sequence you'd write by hand, with zero bookkeeping about where anything lives,
because "where it lives" is always the same answer: "on top." Instruction
encoding is correspondingly tiny and dense — many stack VMs get away with 1-byte
opcodes and no operand fields at all for arithmetic. This is the branch **the
JVM**, **CPython**, **Smalltalk-80**, and **Wren** all occupy, across four very
different eras and use cases, which should tell you the model is not a beginner's
mistake — it's a legitimate, load-bearing choice for a large fraction of the
field.

The bill comes due in instruction *count*. Every intermediate value that would
sit in a register on real hardware instead costs a `Load`/push and, later, a
pop-as-part-of-something-else — and each of those is a full trip through the
loop. A stack machine trades compiler simplicity and code density for *more
dispatches per unit of source-level work*. Whether that trade is bad depends
entirely on how expensive a dispatch is, which is exactly what Axis 2, below, is
about.

### Register machines

A register machine instead names its operands explicitly, by virtual-register
index, right in the instruction: `Add r3, r1, r2` means, unambiguously, "r3 :=
r1 + r2," full stop, no implicit state consulted. Instructions get fatter — an
add needs three operand fields instead of zero — but each instruction now does
the *entire* job of computing one intermediate value, including naming where
that value already lives and where the result goes. There is no separate "load"
step for values that are already resident in a register; the arithmetic
instruction just reads them from wherever the compiler decided to put them.

The occupants here are **Lua** (from version 5.0 onward), **Dalvik** (the
original Android bytecode VM, pre-ART), and **LuaJIT**'s bytecode (which
inherited Lua 5.0's register design as its interpreter-mode ISA even before the
tracing JIT gets involved). The bill: fewer instructions, therefore fewer
dispatches, for the same source-level expression — but the compiler now has to
actually solve a *register allocation* problem: which virtual register holds
which live value, when do two values' lifetimes conflict and force different
registers, when can a register be reused. That problem is real work — it is,
scaled down, the same problem native compiler backends solve for physical
registers — and it forecloses the "the compiler is just a tree walk" simplicity
that stack machines get for free. It also complicates any single instruction that
needs to name more live operands than a fixed instruction word has room for,
which pushes real register VMs toward either wider instruction encodings or
occasional multi-instruction escape sequences for the rare wide case (a cost this
document returns to when it covers the JVM's `wide` prefix — a different
encoding tax, but the same species of problem: fixed-width operand fields are a
bet that most instructions won't need more room than that).

The historically load-bearing data point here is **Lua's switch from a
stack-based virtual machine to a register-based one at version 5.0**
**[flagged — I recall Lua 5.0 released in 2003 and that Lua 3.x/4.0 were
stack-based; I do not have high confidence in the exact version boundary or
release year and would want it checked against the changelog rather than taken
on my word]**. This is not a case of a language designed register-first from a
clean sheet, the way Dalvik was. Lua *lived* with a stack VM for several major
versions, in production, then rewrote its virtual machine specifically because
profiling and instruction-count analysis showed the register model would win —
the implementers (Roberto Ierusalimschy and collaborators) documented the
rationale directly: fewer instructions executed per source-level operation,
because temporaries no longer need explicit load/store traffic through a stack.
That makes Lua's switch the single most citable piece of *empirical* evidence in
this whole design space, as opposed to a priori taste. It is worth treating as
a scar, not a footnote: something a real, widely embedded language paid a
compiler-complexity cost to obtain, after already having built and shipped the
simpler alternative.

### The pole you leave: tree-walking

One more point on this axis, worth exactly one clause of attention because it's
where you'd be if you hadn't compiled to bytecode at all: a **tree-walking / AST
interpreter** has no linear instruction stream and no `ip` — evaluation recurses
directly over the parsed tree, re-discovering at every visit what kind of node
this is (an implicit "decode" baked into every recursive call rather than
factored out into a loop), which is precisely why flattening to bytecode is
worth doing in the first place: it turns tree-shaped control flow into a walk
over a flat, indexable array, and gives you an `ip` that is just an integer
instead of a call stack of tree-visitor activations.

## Same expression, two machines: a countable difference

Take `a + b * c`, respecting the usual precedence (multiply first). Compiled to
stack bytecode:

| ip | instruction | stack after (left→right, TOS rightmost) |
|----|-------------|-------------------------------------------|
| 0  | `Load a`    | `[a]` |
| 1  | `Load b`    | `[a, b]` |
| 2  | `Load c`    | `[a, b, c]` |
| 3  | `Mul`       | `[a, b*c]` |
| 4  | `Add`       | `[a + b*c]` |

Five instructions. Five trips through fetch-decode-execute. `Mul` and `Add` carry
no operand fields at all — they just consult "top two of the stack" — but that
economy is bought by three separate `Load` instructions whose only job is
getting values *onto* the stack so the arithmetic can find them.

Compiled to register bytecode, assuming `a`, `b`, `c` are already resident in
virtual registers `ra`, `rb`, `rc` (the compiler's register allocator decided
that, as part of setting up the enclosing function's calling convention — no
runtime "load" is needed to make a local variable available, because "available"
just means "sitting in a register the whole time"):

| ip | instruction | effect |
|----|-------------|--------|
| 0  | `Mul r3, rb, rc` | `r3 := b * c` |
| 1  | `Add r4, ra, r3` | `r4 := a + r3` |

Two instructions. Two trips through the loop. Both are fatter than the
stack machine's `Mul`/`Add` — each names a destination and two sources — but
neither needed a `Load` counterpart, because the instruction format already had
room to say where its inputs live.

Same semantic content, 5 dispatches versus 2. That ratio is the entire
"instruction count / dispatch count" argument in countable form, and it is the
number Lua's implementers were looking at when they decided the register
rewrite was worth a harder compiler. Note what the comparison does *not* show:
it says nothing yet about how expensive one dispatch *is* — a stack machine with
a cheap dispatch and a register machine with an expensive one could still come
out even, or lose. That's Axis 2.

## The dispatch technique: how fetch-decode reaches a handler

Axis 1 was about instruction format. Axis 2 is orthogonal to it — it's about the
mechanics of the loop itself: once you have an opcode value in hand, by what
physical mechanism does control reach the code that implements it? **Dispatch**
is the name for that step, independent of which technique performs it — the
word names "opcode value has arrived, handler is now running," full stop,
regardless of mechanism.

### `switch` / `match`

The obvious implementation: one `switch` (or, in a language with pattern
matching, one `match`) inside the loop body, one `case` per opcode. Portable —
it's expressible in any structured language, no compiler extensions required —
and simplest to read, since every handler lives as a case arm of one visible
statement.

Its cost is subtler than "a comparison per case," because a decent compiler
turns a dense `switch` into a jump table, not a chain of comparisons — so the
*asymptotic* cost is already close to optimal. The real cost is **branch
predictor pressure**: there is exactly one indirect-branch instruction in the
entire loop (the jump the compiled `switch` uses to reach whichever case),
and it is asked, on every single iteration, to predict a *different* target
depending on which opcode comes next in the stream. Modern CPUs' indirect
branch predictors key their predictions substantially off the *address of the
branch instruction itself*; a single physical branch site trying to model "what
comes after an `Add`" and "what comes after a `Jump`" and "what comes after a
`Load`" all at once is being asked to hold many, often uncorrelated,
target distributions behind one prediction slot, and it mispredicts far more
than a branch that only ever needs to predict one thing would.
**[flagged — the mechanism (predictor keyed by branch-site address, single site
serving all opcodes) is the standard explanation I've seen given for this and I
hold it with reasonable confidence; I would not vouch for any specific
misprediction-rate number without a citation]**

### Direct and token threading

**Direct threading** removes the shared branch entirely: instead of a compact
opcode value, each slot in the compiled "code" array holds the actual machine
address of the handler that implements it. Fetch and decode collapse into one
step — there is nothing to decode, the fetched value already *is* the address
to jump to — and instead of falling through to one shared dispatch point, each
handler ends by jumping directly to whatever address sits in the *next* code
slot. There is no longer one branch site serving every opcode; there is one
branch site *per handler*, each of which tends to see a narrower, more
predictable distribution of "what comes next," because in real programs certain
opcodes really do tend to follow certain others. The cost: a code slot must now
be wide enough to hold a full pointer, not a single byte, which is a real hit to
code density and cache footprint compared to a byte-opcode stream.

**Token threading** (sometimes called *indirect* threading in other taxonomies —
terminology here is genuinely inconsistent across sources, flagged accordingly)
keeps the compact small-integer encoding but restores one level of indirection:
fetch a small token, use it to index a table of handler addresses, jump through
what that table entry holds. It buys back the code-density loss of direct
threading, at the cost of one extra memory load per dispatch (the table lookup)
versus direct threading's "the fetched value is already the address." It still
gives you the same multiple-branch-site benefit over a shared `switch`.

### Computed goto: the fast case, and its precondition

**Computed goto**, a.k.a. labels-as-values, is GCC's (and Clang's) extension —
not standard C, not C++, not a feature most other systems languages expose —
letting code take the address of a label with `&&label` and jump to a value with
`goto *target`. Applied to dispatch, it produces exactly the token-threading
shape *inside a single function*: fetch an opcode, index into a table of `&&`
label addresses built once at startup, `goto *table[opcode]` straight into the
matching case, with the case's own code ending in another such `goto` rather
than falling back to a shared `switch`. **CPython gates this behind a build-time
flag, `USE_COMPUTED_GOTOS`**, auto-enabled when the compiler supports the
extension, and the CPython project's own measurements found it a real,
non-trivial win over plain `switch` dispatch in `ceval.c`. **[flagged — I
recall this optimization and gating mechanism with reasonable confidence; I do
not have a reliable specific speedup percentage or introduction version to cite
and would not want a number invented here]**

Here is the precondition the technique quietly assumes, and the reason it does
not transplant everywhere: computed goto needs the "code" to be a raw,
indexable address space — a byte or word stream you can walk by adding a known
width to a pointer, decode by reading a fixed-size cell, and jump into via a
value you compute at runtime. That is exactly what a byte-opcode array is. It is
*not* what you have if your compiled program is instead a native array of a
tagged/variant type — say, a language-level enum where each variant may carry
different, variant-specific payload data. In that representation there is no
raw byte offset to compute a jump target from in the first place; the "decode"
has already happened by construction (the enum's discriminant already tells you
which variant this is, the moment you have a value of that type in hand), and
whatever dispatches on that discriminant — a `match`/`switch` over the
enum — is up to the host language's own compiler to turn into a jump table or
not, as an optimization decision the source code cannot directly control the way
`&&label`/`goto *x` lets C control it. Put differently: computed goto is a tool
for skipping past a *decode step you would otherwise need*; a representation
that has already done that decoding for you (a materialized, self-describing
value with a discriminant, rather than an offset into an undifferentiated byte
stream) has nothing left for the technique to buy, and typically no
language-level primitive that would let you use it even if it did.

### Subroutine threading, and the rest, briefly

**Subroutine threading** compiles each handler as an actual callable
subroutine and dispatches via a real `call` instruction per opcode rather than
a `jmp` — costing a stack-frame push/pop per dispatch that pure threading avoids,
in exchange for working on any target that has ordinary function calls and
nothing fancier, which made it the portable fallback on architectures or
compilers lacking anything like computed goto. Beyond these four, the
literature (see especially Ertl and Gregg's empirical survey of dispatch
techniques) catalogs finer variants — replicated/inline threading that
duplicates handler code per call site to further help branch prediction at the
cost of code size, and "context threading," which restructures dispatch around
real hardware call/return pairs specifically to exploit return-address branch
predictors — that this document names without unpacking; they refine the same
four ideas above rather than introducing a new one.

## The loop's moving parts, held still

### The instruction pointer and who moves it

`ip` (or `pc`) points at the next instruction to fetch. The default action, on
every ordinary instruction, is "advance past what was just consumed" — by one
opcode-width for a fixed-width encoding, by opcode-plus-however-many-operand-
bytes for a variable-width one. A jump instruction is nothing more than an
instruction whose *execute* step assigns `ip` a value other than the default
next-position — there is no separate "control flow" subsystem; branching is just
one more instruction whose job happens to be "write into `ip` instead of into a
value slot." Everything from `if`/`else` to loops to function calls compiles down
to some pattern of default-advance versus explicit-overwrite of this one
variable.

### The operand stack's push/pop discipline

On a stack machine, the loop's other piece of state is a top-of-stack index
(or pointer) into a preallocated array. Every instruction's contract is stated
purely in terms of how many cells it pops and how many it pushes — `Add` is
"pop 2, push 1," `Dup` is "pop 0, push 1 (a copy of what's now on top)," and so
on — which is exactly why the trace-table notation above works: you can predict
the stack's shape after any instruction purely from that instruction's
pop/push arity, without knowing anything about what came before it beyond the
stack's current depth.

### Decode, and why it can cost almost nothing

Under `switch`/`match` dispatch, decode is a genuine, separate step: given a
fetched opcode value, determine which case applies (compiled, typically, to a
jump-table lookup, so "genuine" here still means "cheap," just not *free*).
Under direct threading or computed goto, decode and fetch collapse into the same
memory access — the value fetched *is* (or directly indexes) the address
executed next, so there is no separate "which case is this" computation left to
perform. This is the sense in which decode cost approaches zero: not that
nothing happens, but that the thing which happens is folded entirely into an
operation (a fetch, a jump) you were going to pay for regardless.

### The halt condition: two honest answers

How does the loop know to stop? Two designs answer this differently, and the
difference is not cosmetic.

**(a) Run off the end of the code array.** The simplest possible answer: `ip`
is compared against the code length (or a sentinel value) at the top of every
iteration, and the loop exits when it's exhausted the array. This is a natural
fit for "compile one script, run it top to bottom" — there is exactly one
code array, exactly one `ip` walking it, and "done" means "walked all of it."

**(b) Drain to a frame floor.** Introduce a second stack — a stack of
*activation records* (call frames), each holding at least a saved `ip` and a
saved stack base to restore on return — separate from (or, as covered below,
overlapping with) the operand stack. A `Call` instruction pushes a new frame and
retargets `ip` into the callee's code; a `Return` instruction pops the current
frame, restores the caller's saved `ip`, and — critically — the loop does not
check "is the code array exhausted" *at all*. Instead, the loop was entered with
a remembered frame-stack depth, a **floor**, and it keeps stepping until a
`Return` drops the frame stack back down to exactly that floor, at which point
control leaves the loop and returns to whatever invoked it — which may be the
top-level host program, or may itself be another activation of the very same
loop, called re-entrantly.

That last clause is the whole point of model (b): because the loop's stopping
condition is stated relative to a floor supplied at entry rather than relative
to "the end of the code," the *identical* loop can run an entire top-level
program (floor = 0, run until the frame stack drains back to empty) or service
a single nested call — a native/primitive function calling back into
interpreted code, a comparator invoked mid-sort, an overloaded-operator
callback triggered from inside a primitive — by entering the loop again with
the *current* depth as the new floor, and returning control the moment that
one activation finishes, without disturbing whatever the outer invocation of
the loop was doing. Model (a) has no equivalent move available: "the end of the
array" is a property of a single top-level program, not a property you can
parameterize per nested call, so re-entrant invocation under model (a) needs
either a wholly separate mechanism or a recursive call into the *host*
language's own call stack rather than a re-entry of the same bytecode loop.
Both designs are real, shipped choices, not a "wrong way" and a "right way" —
model (a) is simpler wherever re-entrant interpreted calls from native code are
rare or nonexistent; model (b) is what a runtime reaches for the moment native
code needs to call back into interpreted code as a matter of course.

## Two things the loop must also answer

### The GC safepoint

A garbage collector that wants to trace live objects needs a moment where the
VM's roots — the operand stack, the frame stack, anything else the collector
walks — are **coherent**: every live value the program cares about is sitting
in a place the collector knows how to find (a stack slot, a frame field), with
nothing live parked somewhere the collector doesn't look. Mid-instruction is
frequently *not* such a moment. Consider `Add` implemented as "pop the right
operand into a host-language local, pop the left operand into another
host-language local, compute, push the result": for the handful of instructions
between the two pops and the final push, one or both operands have left the
stack the collector knows how to scan and are sitting in ordinary host-language
variables the collector has no visibility into at all. If a collection were
triggered at that exact instant, those values could be missed as roots (freed
out from under the still-running instruction) or, depending on how the
collector's write barriers and allocation are wired up, double-counted or
otherwise mishandled — either way, wrong.

The one point in the loop where this is guaranteed *not* to be a problem is the
loop's own **back-edge** — top of the next iteration, `ip` freshly advanced (or
freshly overwritten by whatever jump/call/return just ran), no instruction
mid-flight, every value the program currently cares about fully committed to
stack slots or frame fields and nowhere else. This point has a name:
**safepoint**. A collector that only ever runs (or only ever checks whether it
needs to run) at the back-edge of the dispatch loop can trust the stack's shape
completely, because the back-edge is, by construction, the one moment nothing
is half-done.

### One stack, or two: operands and locals

A separate design question, adjacent to but distinct from the machine-model
axis above: are a frame's **local variables** stored in their own array, or are
they a *window* into the same physical array the operand stack already uses —
local `i` of the currently executing frame living at `stack[base + i]`, where
`base` is a per-frame offset recorded when the frame was created, and the
frame's actual operand-stack traffic (pushes and pops for evaluating
expressions) happens *above* `base + local_count`, in the same contiguous
block? The windowed design has an elegant consequence for calling convention:
if a caller has already pushed a callee's arguments onto the top of the shared
stack, those pushed values simply *become* the callee's first N locals for
free — no copy, the callee's frame `base` is just set to point at where the
arguments already are. It also means a single contiguous array is the entirety
of what a GC root-scan needs to walk for every active frame at once, rather
than one walk per separately allocated locals block. The consequence runs the
other way too: every call site's stack shape is now load-bearing for a whole
frame's addressing scheme, which constrains things like variadic calls and
stack-shape changes across the call boundary far more tightly than a design
where each frame owns an independently allocated locals array and the operand
stack is a wholly separate structure. This is a real fork with real
consequences on both sides, and it's as deep as this document goes — the
combinatorics of calling-convention design that follow from picking one side or
the other belong to a document about calls and frames specifically, not to the
execution loop in general.

## Four specimens, and what they cost to build

### Lua: the register machine's scar

Already introduced above as the Axis 1 anchor, worth returning to for how the
choice actually got built. Lua's register instructions are fixed-width — a
uniform instruction word carrying an opcode field plus a small number of
operand fields (register indices, or a wider immediate field for constants and
jump offsets) — which keeps decode simple despite the richer operand naming: no
variable-length instruction parsing is needed, only field extraction from a
known-width word. **[flagged — I recall a fixed 32-bit instruction word with
roughly this field layout (commonly described as A/B/C/Bx/sBx fields in
discussions of Lua's bytecode) but would not vouch for the exact bit widths
without checking the source]**. The register allocator itself is notably
*not* a general graph-coloring allocator of the kind a native compiler backend
would use — it's a simple, single-pass, stack-discipline allocator that tracks
which registers are "in use" for the expression currently being compiled and
frees them as expressions finish, which keeps the compiler close to the
stack-machine compiler's simplicity while still emitting register-addressed
instructions. That detail matters for the axis-1 discussion above: "harder
compiler" does not have to mean "as hard as a native optimizing backend" — Lua
is the existence proof that a lightweight, expression-scoped allocator gets you
most of the dispatch-count win at a fraction of a full allocator's complexity.

### CPython: the switch that learned to jump

CPython's `ceval.c` is the reference specimen for a `switch`-dispatched stack
machine at real-world scale: one large loop, one `switch` over the current
opcode, one case per opcode, operating on a stack machine whose bytecode is a
flat, indexable stream — since Python 3.6, a **uniform fixed-width (2-byte:
1-byte opcode, 1-byte argument) "wordcode" format**, replacing an earlier
variable-length encoding. **[flagged — the 3.6 wordcode change and 2-byte
width I hold with moderate-to-good confidence; exact version number worth
double-checking]**. That fixed width is precisely the byte-stream property the
computed-goto discussion above depends on, and CPython indeed offers computed
goto as a build-time opt-in (`USE_COMPUTED_GOTOS`) layered over the same
`switch`-shaped case bodies, letting the *same* handler code be reached either
by falling into a shared `switch` or by an `&&label`/`goto *table[op]` chain
depending on what the host compiler supports — a clean illustration that
dispatch technique (Axis 2) can be swapped without touching machine model or
instruction format (Axis 1) at all. Later CPython versions layered further
dispatch-adjacent machinery on top — per-instruction specialization/"adaptive"
opcodes that rewrite themselves based on observed operand types — which is a
related but distinct idea (closer to inline caching than to threading) that
this document flags but does not chase further; it belongs with the
superinstruction/fusion material below. **[flagged — CPython 3.11+
specialization is real (PEP 659) but I'm summarizing from general awareness,
not verified detail]**

### Smalltalk-80: the ancestor where message send *is* the opcode

Smalltalk-80's interpreter loop and its object model are, more than in any
system since, the same artifact. The dominant bytecode is not "call" bound to a
fixed code address — it is **send**, carrying a message selector, which must
perform a runtime method lookup (walk the receiver's class, then its
superclass chain, searching each class's method dictionary for the selector)
*before* the interpreter even knows what code address to jump to. Even
arithmetic goes through this path conceptually — `3 + 4` is, semantically, "send
the message `+` to the object `3`, with argument `4`" — with a set of "special
selector" bytecodes carved out purely as a performance escape hatch, letting the
interpreter fast-path common arithmetic and comparison sends on small integers
without paying full dictionary-lookup cost on every one, while still preserving
the illusion (and, when the fast path doesn't apply, the reality) that it's an
ordinary message send underneath. This is why later Smalltalk-lineage systems —
Self, and Smalltalk's own modern descendants — get described as
**"send-centric"**: the interpreter loop's central, most-executed activity
*is* dynamic dispatch, not a side concern bolted onto an otherwise
address-jumping loop. It's a genuinely different center of gravity from a VM
where "call" is cheap and "virtual dispatch" is one somewhat-more-expensive
special case of it.

### JVM: what a byte-operand encoding costs (the `wide` prefix)

A sharp, contained illustration of the tax a compact fixed-width-operand
encoding pays in its rare case. Several JVM instructions — `iload`, `istore`,
`aload`, `astore`, `ret`, and `iinc` among them — encode their local-variable
index as a single byte, which is dense and fast for the overwhelming majority
of methods (few methods have more than 255 locals) but structurally cannot
address a local-variable slot past index 255. The escape hatch is the `wide`
opcode: emitted immediately *before* the instruction it modifies, its entire
job is "the next instruction's index operand is 2 bytes, not 1" (and, for
`iinc` specifically, that its immediate constant is likewise widened). The
interpreter pays for this rare case with an extra fetch-decode-execute cycle —
`wide` is dispatched exactly like any other opcode, it just does nothing but
flag the mode for the instruction immediately following it — which is the
general shape of what a fixed-width operand field always costs when reality
exceeds the field's range: not incorrectness, but an escape sequence that
doubles the dispatch count for that one logical operation.

### Wren: the small stack VM as a reference point

Worth naming specifically because it's a clean, small, modern instance of the
stack-machine branch, built for embedding (game scripting) rather than for a
general-purpose ecosystem the way CPython or the JVM are, and because it's the
direct production-grade sibling of "clox," the teaching VM built across Bob
Nystrom's *Crafting Interpreters* — the same author built both, and the toy
book VM is, in effect, a simplified rendering of lessons Wren's real
implementation already embodied. As a reference point, Wren is useful precisely
*because* it's small enough to read start to finish and still see every piece
this document has named — the operand stack, the `ip`, the dispatch loop — with
nothing else in the way. **[flagged — Wren's original release year I'd place
loosely in the early-to-mid 2010s but do not hold with confidence]**

### What's cut, and why

**V8, and tracing/method JITs generally** (LuaJIT's trace-compiling mode,
HotSpot's C1/C2, PyPy's meta-tracing) are cut deliberately, not for lack of
relevance to performance but because they answer a different question. A JIT's
defining move is to stop running the interpreter loop for hot code and instead
compile that code to native machine instructions ahead of running them again —
which means the fetch-decode-execute loop this document describes is, for
JIT-compiled paths, no longer the thing executing at all. That's a second
machine bolted onto (or eventually replacing) the first, with its own entirely
separate design space (trace selection, deoptimization, guard placement, on-stack
replacement) — worth a document of its own, not a variant reading of this one.
Any implementation committed to being a pure interpreter, with no compile-to-
native tier, simply never encounters that space, which is exactly why it's cut
here rather than folded in as a third axis.

Two more, named briefly and dismissed: **Ruby's YARV** is architecturally a
stack machine in the same family as CPython and Wren, with a larger, more
specialized opcode set — it doesn't open a design axis the above don't already
cover, so treating it as a fourth full specimen would be repetition dressed as
breadth. The **CLR** (.NET) is likewise a byte-operand stack machine sharing
the JVM's basic shape (including its own analogue of a wide-operand escape for
rare cases), and in practice ships with JIT compilation on by default closely
enough that discussing its pure-interpretation behavior in isolation would be
somewhat artificial — it is closer to the V8 cut than to a new specimen.

## Where the tensions actually sit

**Dispatch overhead ⊗ instruction density.** This is Axis 1 and Axis 2 in
tension with each other, not two independent knobs that happen to sit near each
other: a stack machine's instruction-count cost (more dispatches per
expression) matters *more* the more expensive each dispatch is, which is
exactly why the same design choice — go register, cut dispatch count — and the
same fix for dispatch cost itself — thread it, or gate a computed goto behind
it — are both responses to the same underlying quantity, just attacking it from
opposite ends: fewer trips through the loop, versus cheaper trips through the
loop. A VM that has already made dispatch nearly free (direct threading, say)
has correspondingly less to gain from paying a harder compiler for a register
model than one still paying full `switch`-dispatch cost per instruction — the
two axes are not truly orthogonal in their *payoff*, even though they are
orthogonal in their *implementation*.

**The halt condition ⊗ re-entrancy.** The frame-drain model (b) isn't merely a
different way to detect "done" — it's what makes "one loop, many callers"
possible at all. Any runtime that needs native code to call back into
interpreted code as an ordinary, non-exceptional occurrence (not just at
program start) needs *something* that plays the role of a re-enterable loop
with a parameterized stopping point; model (a)'s "walk off the end of the
array" has no natural generalization to a nested, temporary re-entry, because
"the end of the array" isn't a per-call-site concept. The choice here is
downstream of a question this document hasn't asked — how often, and how
casually, does native code need to call back into interpreted code? — and it
quietly forecloses or unlocks that capability rather than being a free
stylistic pick.

**The loop ⊗ GC coherence.** The safepoint argument above is really a
statement about *where in the loop's structure a collector is allowed to look*.
Placing that permission at the back-edge is cheap precisely because the loop
already visits that point on every single iteration regardless of GC — no new
control-flow needs to be introduced, only a check (or, in simpler designs, an
implicit guarantee) hung off a point that was going to exist anyway. Any design
that instead wants to collect *mid-instruction* — say, to interrupt a very long
running primitive — has to manufacture coherence somewhere that doesn't
naturally have it, which is real, nontrivial machinery (explicit root
registration, stack maps, or the instruction being written to make its own
partial state safe to observe) that the back-edge placement gets for free by
construction. The execution loop's basic shape is, in this sense, already half
of a GC design decision, whether or not the person writing the loop was
thinking about garbage collection at all.
