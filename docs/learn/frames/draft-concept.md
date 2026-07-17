# Call frames / activation records

Every procedure call needs somewhere to put the state that belongs to *this* invocation and not to
any other: where to resume the caller when the callee is done, which values are `this`/`self` for
the duration, the box of local variables the callee is free to mutate, and a working area for
intermediate values the callee's own expression evaluation produces. That bundle is the
**activation record** — "frame" is the informal synonym, and the two terms are used interchangeably
in the literature and in this document. The question this document is about is not *whether* a
running program needs this bundle — it obviously does, for any procedural or object-oriented
language with calls and returns — but a much narrower and more consequential one: **when the state
exists, what kind of thing IS it, in memory?** Is it an object the program can hold a reference to?
A row in an array the runtime owns and nobody else can see? Or nothing at all — just wherever the
host CPU happened to put its own stack frame? Three different answers exist, real systems occupy
each one, and the choice is not free: it forecloses or unlocks entire categories of language
feature (reflection, coroutines, tail calls, cheap green threads) before a single line of the
language's *syntax* has been designed.

## What an activation needs

Strip a procedure activation to the state that is genuinely per-call (not per-procedure, not
global):

- **A return address / continuation.** Where does control go, and with what value, when this
  activation finishes? In a machine-code implementation this is literally an instruction address;
  in a bytecode VM it is a saved instruction pointer plus the frame to resume; in a
  continuation-passing implementation it is reified as a function value.
- **A receiver / environment anchor**, if the language has one — `self`/`this` in an
  object-oriented call, or nothing at all in a pure top-level function call.
- **A window of locals and temporaries** — the arguments, the named local variables, and whatever
  slots the evaluator needs to hold intermediate results while it computes an expression. This is
  usually the largest part of the record and the part whose *size* is knowable statically, because
  a given procedure's body has a fixed maximum number of live temporaries — this is exactly the
  fact that later makes the array representation (branch (b), below) possible at all.
- **A link back to whoever is waiting on this activation to finish** — the caller. This is the part
  where the three branches genuinely diverge, and it's the part this document spends the most time
  on, because "how is the caller found" turns out to be almost the entire design question.

A *stack* of these records is the obvious data structure for the ordinary case, because calls and
returns nest: if A calls B calls C, C must finish before B can resume, and B must finish before A
can resume. That's a LIFO discipline, and a stack is the LIFO data structure. Everything in this
document assumes that ordinary discipline holds (it's why a bare array can stand in for a linked
structure at all, in branch (b)) and flags explicitly where a language breaks it (generators,
continuations, coroutines — anywhere an activation needs to outlive being "the top of the stack").

## Why static allocation broke: Algol, recursion, and the stack

It wasn't always a stack. Early Fortran had no recursion, so a compiler could assign every
variable of every subroutine a single, fixed, compile-time address — a local variable was really
just a differently-named global, allocated once for the life of the program. This is *static*
allocation: the storage for `f`'s locals exists whether or not `f` is currently running, and there
is exactly one copy.

Algol 60 broke this by allowing a procedure to call itself, directly or through a cycle of other
procedures. **[flagged — high confidence on the general history, medium confidence on precise
dates/attribution]** the standard account (repeated in most compiler textbooks, and traced to the
early implementation work around the Algol 60 report, notably Randell and Russell's account of
building an Algol 60 compiler in the mid-1960s) is that recursion is precisely what makes static
allocation *unsound*, not just wasteful: if `f` calls `f` before the first call has returned, there
are now two simultaneously-live sets of `f`'s locals. A single static address per variable can hold
only one of them — the inner call's writes clobber the outer call's still-needed values. The fix
that Algol 60 forced on implementers was to allocate a *fresh* activation record for every call,
at run time, and destroy it on return — dynamic allocation of activation records, organized as a
stack because calls nest. This is the historical origin of "the call stack" as a mechanism distinct
from "an array of global variable slots," and it is worth stating plainly because it is easy to
take the stack for granted: **the stack exists because recursion made a single static slot per
variable incorrect, not merely because a stack happens to be a convenient data structure.**

## Displays, and the two links

Once activation records are dynamically allocated and chained, a new problem appears that static
allocation never had: how does a nested procedure find the locals of the procedure that lexically
encloses it (as opposed to whichever procedure happened to call it)? Algol-family languages allow
procedures to be declared nested inside other procedures and to freely reference the enclosing
procedure's locals by name. Two runtime pointers, per activation, are the classic answer, and the
literature is careful to keep them terminologically separate because they answer *different*
questions:

- The **dynamic link** points to the activation record of *whoever called this one* — it answers
  "who resumes when I return?" and is what makes the stack a stack at all.
- The **static link** points to the activation record of *the most recent activation of the
  lexically enclosing procedure* — it answers "where do I find the locals of the scope I'm nested
  inside?", which in general is a completely different activation than the one that called this
  procedure (a deeply nested procedure can be invoked from anywhere its enclosing procedure passed
  it as a value, or where it's visible, not just from directly inside its lexical parent's body).

Walking a chain of static links to find a variable declared *k* lexical levels up costs O(*k*)
pointer chases per access. **[flagged — medium confidence on precise origin]** the classic
optimization, associated with early Algol 60 implementations, is the **display**: instead of
walking a chain, maintain a small array indexed by lexical nesting depth, where slot *i* always
holds a pointer to the currently-active activation of the procedure at lexical level *i*. Looking
up a non-local variable becomes one array index plus one offset, at the cost of updating the
display's entries on every call and restoring them on every return. Whether a language pays this
cost with a chain, a display, or avoids the problem altogether by *not* having runtime-visible
lexical nesting of procedures (many modern languages instead give a nested function a **closure**
that captures the specific variables it needs, by reference or by value, at the moment the closure
is created — sidestepping the need for a runtime static-link mechanism at call time in exchange for
a capture mechanism at closure-creation time) is its own design fork, and it's a deliberately
separate one from the fork this document is about. It's worth naming here only because "static
link" and "dynamic link" are the vocabulary from which the *modern* term "caller pointer" descends,
and because it clarifies a common conflation: the dynamic link (who called me) is the concept this
document's central fork is actually about. The static link (my lexical parent) is a different
relation this document does not otherwise discuss.

## The fork: how do you represent an activation record?

Given that a fresh record needs to exist per call, in some dynamically-managed collection organized
as a stack, the design question is: **what kind of thing is a record, mechanically?** Three answers
exist, and each is a coherent, defensible engineering position — not a strawman on the way to a
"correct" answer. Take each on its own terms before totting up its bill.

### (a) A heap-allocated frame object, linked to its caller by a pointer

The tempting case: if an activation record is just an ordinary object — allocated on the heap,
managed by the same garbage collector that manages everything else, reachable through an ordinary
object reference — then a huge amount of machinery falls out for free, because "ordinary object"
already comes with identity, arbitrary lifetime, and reflectability built in. A frame object can
outlive the call that created it simply by someone holding a reference to it, the same way any
other object outlives its creator. A debugger doesn't need a special stack-walking protocol; it
just follows object references the way it follows any other reference. A program can ask "what is
executing right now, and what called it, and what called *that*?" as an ordinary query over
ordinary data, because the answer *is* ordinary data: walk the parent pointer. Coroutines and
generators — anything that needs to suspend an activation and resume it later, possibly from a
completely different point in the call graph — become almost trivial to express, because
"suspending an activation" is just "stop touching this object for a while," which requires no
special support beyond what the GC already does for every other object.

The bill: every call now does a heap allocation, and every return makes that allocation garbage.
For a language where calls are the single most frequent runtime event — which is essentially every
language — this is allocation pressure at the highest possible frequency in the system, and it
scales with the GC's cost model directly: more allocations, more collections, and (if the collector
is generational or tracing) more objects to trace on every minor collection for however long the
frame stays reachable. The parent-pointer chain is also, structurally, a **self-referential linked
list that the runtime itself is walking while mutating** — every call adds a node, every return
removes one, and the "current frame" pointer is constantly being repointed. In a language whose
type system tracks ownership and aliasing statically (a borrow checker, in the Rust sense), this
shape is close to the textbook case that such systems are worst at expressing directly: a node that
is owned by whoever holds it *and* points back at the thing that (transitively) owns it, with the
runtime wanting to mutate both ends. It's not that it can't be done — `Rc<RefCell<Frame>>` and
friends exist precisely for this — but every one of those escape hatches re-adds at runtime exactly
the bookkeeping (reference counts, borrow panics, weak vs. strong pointers to avoid the cycle) that
a flat array quietly doesn't need.

### (b) A flat array of value-records — stack windows carved from one shared array

The tempting case: a call's frame size is knowable *before* the call happens — it's a static
property of the procedure being called (argument count, local count, and however deep the
procedure's own expression evaluation needs to nest, all fixed by the procedure's own code). If the
size is known statically, there is no reason to ask a general-purpose allocator for a fresh block
of memory on every call: the runtime can pre-reserve one big contiguous array up front, and treat
each call as simply "carve out the next `N` slots of the *same* array." A call becomes writing into
already-owned memory and bumping a cursor; a return becomes moving the cursor back. No allocator is
consulted at all on the hot path. Because everything lives in one contiguous region, calling deep
into a chain of activations touches memory that is physically close together and was recently
written — exactly the access pattern a CPU cache rewards. And because "who called this activation"
is now simply "whatever is stored one slot below it in the same array," the runtime never needs to
store a caller pointer at all — array position *is* the caller relationship, for free, as a
direct consequence of the LIFO discipline established earlier: calls and returns nest, so the
record directly below the current one, in the same array, is definitionally the record that made
the currently-executing call.

The bill: a frame, under this representation, is not a first-class value the *language* can hand
around. It is not "an object" in the object-model sense — it has no identity that survives being
overwritten by the next call to reuse the same slots, and the runtime typically does not expose an
operation that returns "a reference to frame N" as ordinary data, because there is no safe way to
let a language-level value alias a location the VM is about to reuse for something else. If the
language wants *any* form of introspection — a stack trace, a `caller()` builtin, a debugger
attaching mid-execution — that introspection has to be built as a **side channel that copies out**
whatever information is being asked for (see "the distinguishing program," below) rather than
handing back a live handle into the array. And because slots get reused, a value that legitimately
wants to refer to "that specific past activation, even after it's popped" (a captured continuation,
a generator that's paused, a stack trace someone's still holding onto after the frames it describes
have long since returned) needs an entirely separate identity mechanism layered on top — some way
to tell "the record currently sitting in slot 12" apart from "a *different* record that used to sit
in slot 12 three calls ago and has since been overwritten." That mechanism — commonly a generation
counter or token stamped into each freshly (re)used slot — is real, necessary machinery this branch
owes and branch (a) gets for free from the identity ordinary objects already have. This document
only names that the problem exists; the mechanism belongs to whatever piece of the system is
responsible for frame identity, not to the representation question this document is about.

### (c) The native machine stack — no bookkeeping at all

The tempting case: don't build a frame abstraction. A tree-walking interpreter, or a compiler that
targets native code directly, can simply let each language-level call *be* a call in the host
language or the host machine — a recursive call to an `eval` function in C, or a genuine `call`
instruction in generated machine code. The host language's or the host CPU's own calling
convention already does everything a frame needs: it already has a return address, it already has
local-variable storage (the callee's own stack frame), and it already unwinds correctly on return,
because that's what a function call *is* at the hardware level. This is the cheapest possible
option, in every sense — zero extra bookkeeping, zero extra memory beyond what the call would have
cost anyway, and it runs at the speed of whatever's underneath (native call/return instructions, or
a host compiler's already-optimized function-call sequence).

The bill: there is no data structure to inspect, because there is no data structure — the "frame"
is a region of the C or machine stack the runtime doesn't control and usually can't even describe
in a portable way. Reflection of any kind is essentially closed off; you cannot address "the
current frame" as a value because it was never reified as one. Worse, this representation actively
fights any feature that wants to suspend one logical thread of execution and resume a different one
on the same OS thread — green threads, fibers, cooperative coroutines — because doing that means
swapping out *the native stack itself*, which is either not possible portably (you'd need to save
and restore an entire, arbitrarily deep region of machine stack, plus retarget every saved return
address and frame pointer it contains) or requires dropping to platform-specific stack-switching
tricks (a technique real systems do use — swapping the stack pointer to a separately allocated
stack region, as green-threading runtimes and some fiber libraries do — but that both costs a
dedicated stack allocation per fiber and forfeits the "zero bookkeeping" that made this branch
attractive in the first place). And because the garbage collector has no frame abstraction to
consult, it has no reliable way to find out which heap references are currently alive in local
variables sitting on that native stack; it either has to *guess* (conservative stack scanning —
treat every word on the stack that looks like it could be a valid heap address as if it were one,
a technique real collectors such as the Boehm–Demers–Weiser collector use, at the cost of
occasionally retaining garbage that merely looked like a pointer), or the compiler has to emit
separate, precise metadata describing exactly which stack slots hold live references at each point
— which is exactly the kind of extra bookkeeping this branch was trying to avoid, just moved from
runtime into the compiler.

## The reification spectrum

The three branches above are not three isolated points; they're markers on a continuum, and that
continuum has a name worth having explicitly: **reification** — turning something that would
otherwise be implicit control-flow state into an addressable *value* the running program can hold,
inspect, and pass around. At one end (branch (c)), the frame isn't reified at all — it doesn't
exist as data, only as an effect (the machine's own bookkeeping, invisible to the running program).
In the middle (branch (b)), the *runtime* reifies frames, but only for its own internal bookkeeping
— frame data exists as values the VM's own code manipulates, but the running *program* has no
built-in way to obtain one as a first-class value of its own. At the far end (branch (a)), frames
are reified all the way up to the language's own object model — a frame is not merely data the
implementation happens to keep; it is an *instance*, of the same general kind as every other object
the program manipulates, and the language exposes operations that hand the program a live reference
to one. The useful thing about stating it as a spectrum rather than a trichotomy is that it makes
room for the systems that don't sit cleanly at one end — a VM that keeps branch-(b)-style cheap
internal records most of the time, but is willing to *manufacture* a genuine branch-(a) object on
demand, the moment (and only the moment) a program actually asks for one reflectively. That hybrid
point on the spectrum turns out to be exactly where one of the systems discussed below (CPython,
after a specific and datable change) now sits, and it's the key to one of this document's two
closing tensions.

## The distinguishing program: can you get a handle to *this* frame?

The cleanest way to tell which branch — or which point on the spectrum — a given runtime actually
occupies is not to read its source, but to ask what a small program written *in* the language can
observe. The sharpest version of that program is reflective stack introspection: **can code running
right now obtain a value that represents its own currently-executing activation, or one of its
callers, as data?**

In Python, this is a first-class, unremarkable operation:

```python
import sys

def inner():
    f = sys._getframe()       # the currently-executing frame, as an object
    print(f.f_back.f_code.co_name)   # walk to the caller's frame, read its name
    return f

def outer():
    return inner()

kept = outer()   # `kept` is a live reference to inner()'s frame, even after inner() has returned
```

This program only makes sense if a frame is an object with identity that the language's own
reference semantics apply to. `f.f_back` is a field read on an ordinary value; storing `kept` in a
variable that outlives the call is exactly as unremarkable as storing any other object reference
that outlives the scope that created it — and it has exactly the consequence you'd expect an
ordinary reference to have: as long as `kept` is reachable, the frame it points to, and *its*
`f_back`, and so on up the chain, are all kept alive by the ordinary reachability rules the garbage
collector already applies to everything else. (This is also a well-known way to accidentally leak
memory in Python — holding a traceback or a frame object alive holds its entire enclosing chain of
frames, and everything *those* frames' locals reference, alive with it.)

Smalltalk has the same operation, arguably more purely, because it's not even a library call — it's
a pseudo-variable, `thisContext`, bound at all times to an object representing the currently
executing context:

```smalltalk
inner
    Transcript showCr: thisContext sender printString.
    ^thisContext
```

`thisContext sender` walks to the caller's context exactly the way `f.f_back` does in Python, and a
context returned from a method is just as ordinary an object as anything else in the image —
storable, inspectable, and (this is Smalltalk's most consequential use of the idea) *resumable*:
because a context is a real object holding its own instruction pointer and its own link to where it
should resume, a saved context is most of what's needed to implement a general continuation or a
coroutine, entirely in terms of the language's own object model, with no separate mechanism bolted
on.

Contrast this with Lua, which occupies branch (b). Lua *does* offer a debug facility —
`debug.getinfo(level)` — that can report information about an activation N levels up the call
stack: its function, its current line, its local-variable names. But the crucial difference is what
comes back: a fresh table containing a **snapshot copy** of the requested fields, not a reference to
the activation itself. There is no `debug.getframe(level)` that hands back a value with the
identity of "activation N," because activation N is not a value with identity in Lua's own object
model — it's a window (a base/top pair) into the VM's internal register stack, and that window will
be overwritten the moment the call it belongs to returns. You can *ask questions about* a live
activation from outside it; you cannot *hold onto* one. That is the distinguishing behavior of a
non-reified-to-the-language representation exactly as sharply as `sys._getframe()` is the
distinguishing behavior of a reified one: one returns a handle with identity and independent
lifetime, the other returns a copy with neither.

And at the far, unreified end, branch (c) systems typically offer *nothing* built in — no debug
library, no snapshot, no handle — because there is no runtime-owned data structure to query in the
first place. Any answer to "who's my caller?" has to be threaded through explicitly, as an ordinary
extra function argument, by the language's own program, because the implementation has nothing to
consult on the program's behalf. That absence is itself diagnostic: a language that can't answer
"who called me?" without the programmer manually plumbing the answer through as a parameter is
telling you, as clearly as a stack trace could, which branch its implementation took.

## The mechanism, under the array branch

Branch (b) is worth walking through as a full lifecycle, because the mechanism is where the
representation's consequences stop being abstract. Assume a bytecode VM: one big, contiguous,
shared array of values — call it the **operand/value stack** — that holds *every* activation's
locals and temporaries at once, back to back. An activation record proper (the small, fixed-shape
piece of metadata: saved instruction pointer, receiver, and critically, a **stack offset** marking
where in that shared array this activation's own window begins) is itself stored in a second,
much smaller array — call it the frame stack. This two-array split is the standard shape of a
**register-window** design: "register window" because each activation gets a private, contiguous
range of slots in the big shared array that behaves, from that activation's point of view, exactly
like a private bank of local registers, even though physically it's just an interval of one array
everyone shares.

- **Construction, on call.** The callee's slot requirement is known statically from its own code
  (argument count + local count + max expression-evaluation depth). The VM computes a stack offset
  — simply "wherever the caller's window currently ends" — reserves that many slots by extending
  the shared value array, writes a new frame record (return instruction pointer, that stack offset,
  the receiver, whatever else an activation needs) into the *next* slot of the frame array, and
  transfers control to the callee's code. No allocator call happened; both "allocations" were
  cursor bumps into memory the VM already owned.
- **The window.** Every local-variable access the callee performs is `stack_offset + i` for some
  statically-known `i` — the compiler, not the runtime, already determined which local a name
  refers to and at what fixed offset within the activation's window it lives, back when it compiled
  the callee's body. The window is what makes "locals" and "the shared value stack" the same piece
  of memory without callees stepping on each other: they simply address disjoint, adjacent ranges.
- **Return.** Pop the top entry off the frame array. The instruction pointer it carried is where
  execution resumes; the shared value array is truncated back down to that frame's own
  `stack_offset` (discarding its locals and temporaries, keeping everything below, which belongs to
  the caller). The new top of the frame array is, automatically, the caller — there was never a
  pointer to follow, because "the record one slot down" *was* the caller pointer, structurally,
  from the moment the frame was pushed.
- **Unwind, on a non-local exit.** An exception, or any control-flow event that needs to discard
  several activations at once rather than one at a time (unwinding past two, five, or a hundred
  nested calls to reach a handler or a loop's exit point), is where this representation's *shape*
  — not just its per-call cost — pays off. Under branch (b), unwinding N frames is a single
  operation: truncate the frame array directly down to the target length, and truncate the value
  array directly down to the target frame's `stack_offset`. Every intervening activation vanishes
  in one bulk length update, because they were never independently-owned nodes that each needed
  individually unlinking — they were just entries in an array, indistinguishable in cost from one
  entry or a hundred. (This is not quite "free" if the values sitting in those discarded slots hold
  resources that need their own cleanup on the way out — a heap handle needing a reference-count
  decrement, say — but the *frame bookkeeping itself* collapses to one bulk operation regardless of
  how many activations it spans.) Contrast branch (a) directly: a heap-object-with-parent-pointer
  representation has no equivalent bulk move available. Unwinding N frames means walking from the
  current (deepest) frame up N `f_back`-style links, and at each one, doing whatever "this node is
  no longer reachable" requires — dropping a reference, decrementing a count, or simply leaving it
  for the collector to notice later — one node, one operation, at a time. The array representation
  turns an O(N) *sequence of individually-owned-node teardowns* into an O(1) *length update*
  (plus, unavoidably, whatever O(N) per-slot value cleanup the discarded data itself demands); the
  linked-object representation cannot collapse the bookkeeping the same way, because there is no
  single "length" for a set of separately allocated, separately owned nodes to be trimmed down to.

- **A GC root, regardless of branch.** One property holds no matter which branch a runtime chose,
  and it's worth stating as its own fact rather than folding it into any one branch's bill: whatever
  currently constitutes "the stack of active frames," in whatever representation, has to be treated
  by the garbage collector as part of the **root set** — the starting points a trace or mark pass
  begins from, because these are the only places the *running program itself* currently holds
  pointers into the heap that the collector has no other way to discover. Under branch (a) this is
  almost invisible as a special case, because frames are heap objects and get traced like any other
  object (the walk just has to *start* somewhere, typically the topmost context). Under branch (b),
  it is not invisible: a frame here is a plain value, not itself a node the collector's ordinary
  heap trace would ever visit, so root enumeration has to be an explicit extra step — walk the
  frame array directly (not the value array generically, but specifically the fields of each frame
  and, through it, the *portion* of the value array that frame's window covers) and hand the
  collector every heap reference found inside. Under branch (c), it's hardest of all: the "frame
  array" doesn't exist to walk, so the collector either has to scan the native stack conservatively
  (treating stack words that look like valid heap addresses as roots, accepting the risk of
  occasionally retaining something that merely looked like a pointer) or the compiler has to emit
  separate precise root-location metadata for every call site. The general point survives the
  branch: **being a plain value, rather than a heap object, does not exempt a frame from being a GC
  root — it only changes how the collector finds out that it is one.**

## Ancestors and scars

Three systems earn a close look because each *names* something in the design space above, not
because a longer list would be more thorough. (Cut, deliberately: the JVM, whose frame
representation is heavily implementation- and JIT-specific rather than a single clean point on this
spectrum and so doesn't cleanly name a branch the way the three below do; Ruby MRI, which sits close
enough to CPython's position on this particular axis that it would repeat the same bill without
adding a new idea; and tree-walking interpreters generically, already covered in full as branch
(c) itself rather than as a separate case study.)

**Lua — CallInfo, register windows, the array branch's clearest ancestor.** Lua's bytecode VM keeps
one contiguous array of values per Lua state (`TValue`-typed, in the reference implementation) as
the shared register file every activation draws its window from, exactly the register-window shape
described above; the reference implementation and design papers describing it (notably "The
Implementation of Lua 5.0," Ierusalimschy, de Figueiredo, and Celes) present locals and temporaries
as living in that one shared, contiguous store rather than in individually heap-allocated per-call
objects — this is high confidence and is the well-documented, defining feature of Lua's calling
convention. **[flagged — medium confidence]** the metadata records describing each active call
(`CallInfo` in the reference implementation) are, as I recall the source, themselves kept in a
growable array but additionally linked to their neighbors for fast traversal — I hold the precise
current-version detail of "array-backed, doubly-linked for O(1) walking" at only medium confidence
and would not want a design decision to rest on it; the high-confidence, load-bearing fact is the
one stated above: Lua's *locals* live in one flat shared array, addressed by a base/top window per
call, not in a per-call heap allocation. Who: Lua and languages descended from or inspired by its
VM design (Wren names itself explicitly in this lineage). What it buys: the full array-branch case
made above — no per-call allocator traffic, cache-local activation, unwinding by truncation. Bill:
`debug.getinfo` returning copies rather than handles is Lua's own instance of the distinguishing
program from above, and it's a deliberate, visible consequence of the branch, not an oversight.

**CPython — PyFrameObject, `f_back`, and a bill that has since been renegotiated.** CPython's
frames are, historically and by default in the mental model most Python programmers carry, ordinary
heap objects: a `PyFrameObject` per activation, linked to its caller through an `f_back` field, with
`sys._getframe()` and `inspect.stack()` handing the running program a live reference to exactly this
chain — the canonical instance of branch (a) and of the distinguishing program returning a real
handle rather than a snapshot. **[flagged — medium confidence]** historically, CPython did not pay
a full malloc/free cost on *every* call despite frame objects being heap objects — my recollection
is that the interpreter reused frame objects via a per-thread free list rather than going to the
general allocator on every single call — but the object was still a comparatively heavyweight,
individually-identified heap structure, embedding its own local-variable slot array
(`f_localsplus`) inside itself, and the *design*, independent of that internal reuse optimization,
is squarely branch (a). The famous consequence programmers actually hit is `RecursionError` —
**[flagged — medium confidence on the precise mechanism, high confidence that the guard itself is
real and well known]** — CPython's classic interpreter loop recursed, at the C level, once per
nested Python-level call (each Python call caused a nested native call into the evaluator), so deep
Python recursion became deep *native* C-stack recursion, risking an actual OS stack overflow and a
hard crash; `sys.getrecursionlimit()`/`setrecursionlimit()` and the catchable `RecursionError` exist
as a safety valve raised well before the real C stack is exhausted. That detail is worth pausing on
because it shows CPython straddling two branches at once on two different layers: the
*language-visible* call stack is fully reified, heap-object frames (branch (a)); the *control-flow
transfer underneath it*, at least in the classic design, rode the native C stack (branch (c)),
which is exactly why deep recursion was dangerous in the first place — the reified frame objects
were never the resource actually at risk of overflowing.
**[flagged — medium-high confidence, dated]** CPython 3.11 (released October 2022), as part of the
"Faster CPython" project, changed the *representation* underneath this semantics without changing
the semantics: internal activations became lighter-weight structures kept in a per-thread
contiguous frame stack — much closer in spirit to branch (b) — and the full, heavyweight
`PyFrameObject` the language exposes through `f_back`/`sys._getframe`/tracebacks is now
**lazily materialized** only at the point something actually asks for it reflectively, rather than
built unconditionally on every call. I'm confident this characterization is directionally correct
and the date is right; I would not stake an exact internal type name or every detail of the
materialization trigger on memory alone. The reason this matters here, rather than being a footnote
about one interpreter's release notes, is that it's a concrete, datable existence proof of the
reification-spectrum idea stated earlier: CPython moved a real, shipping system rightward along the
spectrum — from "reify every call unconditionally" toward "keep the cheap internal representation
by default, manufacture the expensive reified object only on demand" — while keeping the
*language-level contract* (`sys._getframe()` still works, `f_back` still works) completely
unchanged. Semantics and representation moved independently.

**Smalltalk — MethodContext, BlockContext, and `thisContext` as the extreme, and the ancestor of
the semantics everything above is measured against.** Smalltalk didn't just implement branch (a);
it's the system that established the *semantic contract* branch (a) is judged against — a context
(the Smalltalk term for an activation record) is an ordinary object in the image, `thisContext` is
a pseudo-variable bound to the currently executing one, and because a context carries its own
sender link and its own resumption point as ordinary object state, the same mechanism that gives
`thisContext sender` its answer is what makes general continuations and coroutines expressible
directly in terms of the object model, with nothing extra bolted on — hand someone a context, and
you've handed them something that can, in principle, be resumed. This is the "extreme" end of the
reification spectrum: not merely "frames are heap objects," but "frames are objects of exactly the
same *kind* as every other object in the system, subject to the same reflection, the same garbage
collection, and the same message-send protocol as a `String` or an `Array`." The scar is the one
the CPython story above already previewed by decades: naive per-send heap allocation of a
general object is expensive, and a message send is Smalltalk's single most frequent runtime event,
by the same argument made against branch (a) in general. **[flagged — medium confidence on the
specific paper and its exact techniques, high confidence that this class of optimization is real
and well documented]** the literature on efficient Smalltalk-80 implementation (I recall Deutsch
and Schiffman's widely-cited work on efficient Smalltalk-80 implementation, alongside inline
caching, as also addressing the cost of context management) describes techniques for keeping
contexts on something closer to a conventional, cheap stack internally and only fully
materializing/reifying a context as a first-class heap object when the program's own semantics
force it to exist as one — the same demand-driven materialization move CPython made explicit, and
dated, thirty-some years later. The throughline worth naming plainly: **Smalltalk is the ancestor
of the *semantics* — first-class, reflectable, resumable activations — far more than it is proof
that those semantics require paying the branch-(a) bill on every single call.** A system can keep
the Smalltalk-style contract while representing the common case far more cheaply, exactly as later
tension #1 states outright.

## Two tensions worth sitting with

**Semantics and representation are different axes, and conflating them is the single most common
mistake this design space invites.** It is tempting to treat "does this language let you reflect on
the call stack?" and "how does this runtime store the call stack in memory?" as one question with
one answer, because the systems most people learn from first — CPython pre-3.11, classic Smalltalk
— happen to answer both questions the same way at once (fully reified semantics, fully reified,
heap-object-per-call representation). But the CPython 3.11 story above is a direct, shipping
counterexample: the *language contract* — you can call `sys._getframe()`, you get back something
with `f_back`, tracebacks work, `thisContext`-style semantics if a language chose to expose them —
can be honored in full while the *default*, hot-path representation underneath is the cheap,
array-branch shape, with a real branch-(a) object manufactured only lazily, on demand, when (and
only when) the program actually exercises the reflective part of the contract. A language can be
"Smalltalk" in what it promises the program can observe and do with an activation, while being
"Lua" in how the overwhelming majority of calls — the ones nobody ever reflects on — are actually
represented in memory. Whether a given system pays branch (a)'s bill *at all*, or only pays it on
the rare call that gets reflected on, is a genuinely separate engineering decision from whether the
language *semantically permits* reflection in the first place — and mistaking the second question's
answer for a forced answer to the first is exactly the imported intuition worth actively dislodging.

**A value is not exempt from being a garbage-collection root just because it isn't itself a heap
object.** This was stated in passing during the mechanism walkthrough above, but it deserves to be
named as a tension in its own right, because it's easy to read "frames are plain values, not heap
objects" as implying "frames are outside the garbage collector's concern," and that implication is
false. A frame — under any of the three branches — typically holds at least one field that *is* a
reference into the heap the collector manages: a receiver, a closure or callable being executed, a
captured environment. That field makes the frame part of the root set the moment it exists,
regardless of whether the frame itself lives on a heap, in a flat array, or nowhere addressable at
all (the native-stack branch, where the collector's problem is hardest precisely because there's no
frame *value* to walk in the first place). Being cheap to represent and being irrelevant to the
collector are two different properties, and a design that gets the first one right still owes the
collector an honest answer to the second.
