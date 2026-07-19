# Remembering the Answer to a Question You Keep Asking

A send resolves its selector by walking the receiver's class chain: hash the selector, probe the
receiver's class for a matching method, and if it isn't there, climb to the superclass and probe
again, until either a method is found or the chain runs out. That walk is correct — it is *the*
definition of late-bound dispatch — and it is also work. Every single send re-derives an answer
from scratch, as if the question had never been asked before.

Put a send inside a loop that runs a million times against objects of the same class, and the VM
performs the same hash, the same probe, the same chain-climb, one million times, and gets the same
answer one million times. The walk is a *pure function* of (receiver's class, selector) as long as
nothing changes the method tables in between — and in the overwhelming majority of real programs,
nothing does, for long stretches. The observation that turns this into an optimization has a name
worth stating precisely, because everything else in this document is a variation on it:

> **The binding at a given call site is stable across calls, far more often than it is not.**

Not "the receiver's class never varies" — that is a stronger and, for many call sites, false claim.
The weaker and almost-always-true claim is: *the class this `Invoke` instruction saw last time is
overwhelmingly likely to be the class it sees this time.* A call site — a specific `Invoke` (or
`send`, or `CALL_METHOD`) instruction at a fixed program point — is a much better unit of memory
than "the method" or "the selector" in the abstract, because a single call site in a `for`-loop body
almost always sees one shape of the world, even though the *selector* it sends might be polymorphic
across the whole program (many other call sites send the same selector to many other classes without
disturbing this one). Caching, in this domain, is caching *per call site*, not per selector and not
globally.

## The origin, and why it is called *inline*

This is not a modern idea bolted onto an old design; it is one of the oldest optimizations in
dynamic-object-model runtimes, and it comes with a name and a date: L. Peter Deutsch and Allan M.
Schiffman, "Efficient Implementation of the Smalltalk-80 System," POPL 1984. Smalltalk-80's message
send was exactly the walk described above — hash the selector, probe the class, climb — and it was,
predictably, the dominant cost of running Smalltalk programs on the hardware of the era. Deutsch and
Schiffman's fix was not a smarter data structure for the walk; it was to stop doing the walk at all
on the common path, by **rewriting the call site itself**.

The mechanism, concretely: the first time a `send` executes at a given point in the compiled code,
it does the full lookup, finds the target method, and then — this is the part worth sitting with —
*patches the machine code at the call site in place*, so that the call instruction now jumps directly
to the compiled method it found, preceded by a short guard that checks the receiver's class against
the one that was looked up. The *next* time control reaches that point, there is no lookup at all:
the guard check-and-branch is directly *inline* in the instruction stream where the generic send used
to be. That is the origin of the term **inline cache** — the cache is not a separate table the send
instruction consults, it *is* the instruction, rewritten. If the guard passes, execution falls
straight into the cached method. If it fails — a different class showed up — the patched code traps
back into the general lookup routine, which resolves the new binding and repatches the site again.

This is worth being precise about because later systems (including every one discussed below) keep
the *idea* — a call site remembers a binding and pays only for a cheap check on the fast path — while
mostly abandoning the *literal* self-modifying-code implementation. A bytecode interpreter without a
native-code call site to patch instead keeps a small side-record associated with the instruction (a
cache *slot*, addressed by instruction position rather than embedded in the opcode's operand bytes);
the check-and-branch becomes "compare receiver's class to the slot's remembered class," and the
"jump directly to the method" becomes "skip straight to invoking the cached `Method` handle." The
representation changes; the shape — *remember one class, guard on it, fall through on match* — does
not. This is the moment Deutsch and Schiffman's paper reports something like a two-to-threefold
speedup on send-heavy code **[flagged — exact factor recalled with only moderate confidence; the
paper's headline number is in this range but I would not stake precision on the exact multiplier
without the source in hand]**, which is why the technique propagated into essentially every
dynamic-object-model runtime built since.

## How much can a call site afford to remember

Once you accept "cache the last binding at this site," a design space opens immediately: *how much*
does a site remember, and what happens when there is more than one answer to remember? This whole
axis is **call-site specialization** — the general move of tailoring a specific program point's
fast path to what has actually shown up there at runtime, as opposed to keeping every call site
equally generic. Four points on this axis are all real, occupied designs, not a straight-line
progression from worse to better — each is the *right* answer for a different distribution of what
call sites actually see at runtime.

**No cache — walk the chain on every send.** This is the honest baseline, and it is not a strawman:
naive Smalltalk-80 before Deutsch and Schiffman, and every tree-walking interpreter that resolves a
method name by literal dictionary lookup at each evaluation, runs this way, and it is *always
correct* — there is nothing to invalidate, because nothing is remembered. Its case is genuinely
strong for a first implementation: zero bookkeeping, zero soundness risk, a smaller and easier
interpreter core. The bill is that the walk is not a small cost sitting off to the side — for a
dynamically-typed, message-send-heavy language, it *is* the interpreter's dominant cost, often
dwarfing the actual work the method body does for anything but the heaviest primitives. A no-cache
interpreter spends more time deciding *what to run* than running it.

**Monomorphic inline cache — one slot per call site: (class, method, stamp).** This is the
Deutsch–Schiffman answer, and its assumption is explicit: *this site sees one receiver class,
consistently.* That assumption is empirically true at a striking fraction of real call sites — loop
bodies iterating a homogeneous collection, a getter called on instances of one class, an operator
applied to values of one type — because most polymorphism in practice is *inter-site*, not
*intra-site*: different call sites see different classes, but any one call site tends to see the same
class over and over. When the assumption holds, a hit costs one identity comparison and one stamp
check, full stop — no hash, no chain climb. When it doesn't hold — a site that legitimately receives
two or three different classes in rotation — a single-slot cache **thrashes**: every call finds the
slot holding the *other* class, invalidates and refills it, and the site pays the walk *and* the
failed comparison on every single call, which is strictly worse than having no cache at all for that
site. The monomorphic cache is a bet, and the bet is that call-site-level polymorphism is rare enough
that eating the occasional thrash is cheaper than the bookkeeping of guarding against it everywhere.

**Polymorphic inline cache (PIC) — a handful of slots per site, scanned linearly.** This is where
SELF enters, and it deserves the weight the source material gives it, because it is not a minor
refinement of the monomorphic idea — it is a different bet about what "the common case" is. Urs
Hölzle, Craig Chambers, and David Ungar, "Optimizing Dynamically-Typed Object-Oriented Languages With
Polymorphic Inline Caches," ECOOP 1991, observed that a meaningful fraction of call sites are not
monomorphic but **stably, boundedly polymorphic** — a site that, over the life of the program, sees
exactly two or three receiver classes, in some interleaving, and keeps seeing only those two or
three. A monomorphic cache thrashes forever on such a site; the fix is to stop treating "more than
one class" as failure and instead give the site *N* slots (small — SELF's implementations used
single-digit limits in practice), each an independent (class, method) pair, checked in sequence on a
miss before falling back to the full lookup. A hit anywhere in the small list is still far cheaper
than a walk; a genuine miss (a *new*, fourth class) appends a new entry rather than evicting one, up
to some limit. The cost is real: a bigger cache record per site, a linear scan instead of one compare
(cheap while *N* is small, which is precisely the design's premise), and a policy decision for what
happens when a site blows past the slot limit — which is exactly the megamorphic branch below.

The PIC bought SELF something beyond raw dispatch speed, and this is the detail worth being precise
about: because a PIC *records every distinct class it has ever seen at a site*, it is not just a
cache, it is a *log* — a runtime-observed histogram of the types that actually flow through that
program point. This is **type feedback**: information a static analysis could never obtain (the
language has no static types to analyze) but that falls out for free as a byproduct of caching. In a
system with a just-in-time compiler, type feedback is enormously valuable — a JIT can read a warm
site's PIC, see it has only ever held `Point` and `Vector3`, and compile a specialized version of the
loop with a runtime guard for exactly those two classes and a direct, uninlined call as the fallback,
turning a polymorphic dispatch into what is effectively an inlined `if`. **In a pure bytecode
interpreter with no JIT, a PIC still buys the interpreter faster dispatch on the fast path, but the
type-feedback half of its value goes unspent — there is no downstream compiler to hand the histogram
to, so the accumulated class list is doing real work as a cache and none as a signal.** That
distinction matters enough to state once, plainly, and not revisit.

**Megamorphic fallback — give up caching at this site.** Past some number of distinct classes (the
PIC's slot limit, or a monomorphic cache's repeated-thrash detector), continuing to try to cache
per-site stops paying for itself: the site is not "occasionally polymorphic," it is fundamentally
unpredictable — think a generic `toString`-style call reached from deeply heterogeneous collection
processing, or a dispatch dispatched through so much indirection that its receiver class is
effectively random from the cache's point of view. The honest move here, taken by V8, HotSpot, and
(in spirit) the original Smalltalk-80 system, is to stop trying to specialize *that call site* and
fall back to something coarser but *shared*: a single, program-wide lookup table keyed on
(class, selector) rather than on instruction position — sometimes called a **global method cache** —
that every megamorphic site probes instead of maintaining its own slots. This is worth naming
explicitly because it is, structurally, a return to a *pre-inline-cache* technique: Smalltalk-80 had
a shared, hashed method-lookup cache *before* Deutsch and Schiffman added per-site inline caches on
top of it, and the megamorphic fallback is exactly that older, coarser mechanism, kept alive as the
floor beneath the newer one. It bounds the worst case — a wildly polymorphic site degrades to roughly
a hash-table probe, not back to a full superclass-chain walk — but it is honest to say what it does
*not* do: for that specific site, caching bought nothing. Every slot spent getting there was spent
finding out the site doesn't fit the assumption the cache was built for.

## The hard half: telling a cache the world has moved

Everything above assumes a cached binding stays true until something explicitly disturbs it. In a
language with a genuinely dynamic object model — one where a class's method table can be mutated
*after* the program has already started running and already cached sends against it (open classes,
monkey-patching, `class_eval`-style redefinition, or simply loading more code at runtime) — that
assumption is not automatically true, and getting it wrong is not a performance bug, it is a
**correctness bug**. The canonical failure: a call site caches "receiver is class `Foo`, call
method `Foo>>bar` at address X." The program later redefines `Foo>>bar` — or removes it, or `Foo`
gains a new superclass that shadows it — and the *cached slot has no way to know*. The next call
still matches on class identity (still a `Foo`), still finds its guard passing, and confidently
invokes a method that is no longer the right one, or that no longer exists. Silent, wrong output, not
a crash — the worst kind of bug, because the cache did exactly what it was built to do and the thing
it was built to do turned out to be unsound.

So a cache is only as correct as its answer to one question: **how does a slot learn the world
changed?** There are two structurally different answers, and neither is a strawman — both are real,
shipped, occupied points in the design space, and the honest position is that neither dominates the
other; the deciding variable is *how often the program mutates class structure once it is past
warmup*.

**Per-class (or per-site) epoch — a version number that travels with the thing that can go stale.**
Give every class its own counter. Any mutation that could invalidate a cached binding through that
class — defining a method, removing one, changing the superclass link — bumps that class's counter,
and, critically, the counters of every class *beneath* it in the inheritance chain (because a method
change on a superclass can shadow or unshadow something a subclass's sends resolve through). A cache
slot stores the epoch value it was filled at; a probe compares the *current* epoch of the relevant
class against the stored one, and only trusts the slot if they match. This is **fine-grained**: a
redefinition of a method on `Foo` invalidates exactly the caches that actually resolved through
`Foo`'s changed method table — a call site that has only ever dispatched to `Bar`, an unrelated class
with no ancestry relationship to `Foo`, never notices, never refills, never pays a cent for `Foo`'s
mutation. The bill is bookkeeping, and it is not a small one: every mutation site in the runtime —
method definition, method removal, superclass reassignment, and anything else that can change what a
lookup finds — must correctly identify *every* class whose epoch needs bumping, including walking
*down* the subclass tree from the mutation point, and missing even one class on one mutation path is
not a crash, it is a *silent, load-bearing staleness bug* that will not show up until exactly the
program that exercises that one missed path runs, at which point it reproduces as "a cache served a
method that should have been overridden," indistinguishable from the original correctness bug this
whole apparatus exists to prevent.

**Single global version counter — one number for the whole runtime.** Instead of per-class
bookkeeping, keep exactly one counter, somewhere the whole VM can see it. *Any* method install,
anywhere, on any class, increments it once. Every cache slot — regardless of which class or site it
belongs to — stores the counter's value at the moment it was filled, and a probe is sound if and only
if that stored value still equals the current global value. This is **trivially correct**: there is
no subtree to walk, no "did I bump the right classes" question to get wrong, because there is only
one thing to bump and it is unconditionally right every time — a method install *anywhere* is, by
definition, a change to "the world," and the counter tracks exactly that, no more and no less. The
correctness argument is a two-line proof instead of an invariant that has to hold across every
mutation call site in the codebase forever. The bill is coarseness, and it is a real one: a method
defined on `Foo` bumps the *same* counter that a completely unrelated call site sending to `Bar` is
watching, so that `Bar` site's perfectly-still-valid cache is discarded anyway — not because anything
about `Bar` changed, but because the counter it happened to be comparing against moved. Each
discarded slot notices lazily, on its own next probe, and pays one refill (one walk) to re-warm —
this is not a crash or even a correctness risk, only a lost cache hit. Whether that is expensive
depends entirely on program shape: a workload that defines its classes during a startup/load phase
and then runs a long, mutation-free steady state pays this cost exactly once, at the boundary, and
the global counter is then indistinguishable in cost from the per-class version for the rest of the
run. A workload that keeps redefining methods *inside* its hot loop — pathological, but real in
sufficiently dynamic metaprogramming-heavy code — pays a full program-wide re-warm on every single
mutation, which the per-class scheme would have contained to one class's worth of sites. Neither
branch is "the correct one" in the abstract; one is a bet that mutation is front-loaded, the other is
insurance against mutation that isn't.

### The closest living relative: version tags in a bytecode interpreter with no JIT

Of the systems worth naming here, CPython's adaptive interpreter — landed for Python 3.11 under
PEP 659, "Specializing Adaptive Interpreter" (Mark Shannon) — is the one that sits closest to this
exact problem, because it is solving it in exactly this setting: a bytecode interpreter, not a JIT,
caching *at the level of a single instruction*, with no native code to patch. The mechanism, at the
level of shape rather than exact field names (which have shifted release to release):
**[flagged — I am recalling the general architecture with reasonable confidence but specific opcode
identifiers and struct field names below should be read as illustrative of the mechanism, not quoted
as exact source]**

A generic instruction — the two most commonly cited are `LOAD_ATTR` (attribute access) and
`LOAD_GLOBAL` (global-variable lookup) — starts out unspecialized. After it has executed some small
threshold number of times, the interpreter inspects the *concrete* situation it just handled: is this
attribute stored in the instance's own `__dict__` at a fixed offset? Is it a slot descriptor? Is the
global actually a builtin that has never been shadowed? Based on that answer, it rewrites — this is
CPython's own use of the term **quickening** — that specific bytecode slot from the generic opcode
into one of several specialized variants (illustratively, something like an "attribute is at a known
dict offset" variant versus an "attribute is a slot" variant), each of which embeds a small **inline
cache** directly adjacent to the instruction in the code object: a cached offset or descriptor, and a
**version tag** — for a type, this is the type's own version counter (bumped whenever *that type's*
attribute set changes), and for a global lookup, a similar version tag on the relevant dict's keys.
Each specialized instruction's fast path is: compare the live object's current version tag to the one
baked into the cache; on a match, use the cached offset directly with no attribute-lookup machinery
at all; on a mismatch, fall back to the generic, fully general behavior for *that one execution*, and
count the miss. Enough consecutive misses **de-specializes** the site back to the fully generic
opcode, rather than leaving a permanently-failing specialization sitting there paying its guard cost
for nothing.

This is worth holding up against the two-pole invalidation story above because it is neither pole in
its pure form and is more informative for that reason: CPython's version tags are **per-type** (and
per-dict-keys-object), which is the fine-grained pole's bet, but the bookkeeping burden that makes
fine-grained invalidation hard elsewhere is tamed here by a narrower scope of what can be mutated
through this path — a type's version tag only needs to change when *that type's* attributes change,
and CPython already had to track that centrally for other reasons (the type doesn't have a subclass
tree to walk in the general case relevant to `LOAD_ATTR`, unlike a full method-table epoch scheme
resolving through inheritance). The instructive point is that "per-class epoch" is not a single
design — it has its own sub-space of exactly how granular the tracked entity is, and CPython's answer
threads a smaller needle than the fully general per-class-and-subtree epoch described above, in
exchange for solving a narrower problem (attribute and global-name caching, not full polymorphic
method dispatch with inheritance-based shadowing).

## Two programs that separate the model from the intuition

The reader's intuition after the above is roughly right but underspecified until it is pinned to
concrete programs. Two pairs do the pinning.

**Monomorphic versus megamorphic, same call site shape.** A loop that sends `area` to a million
`Circle` instances in sequence is the monomorphic cache's ideal case: the first send walks the chain,
finds `Circle>>area`, fills the slot; every subsequent send is a class-identity compare and a direct
invoke, no walk, for 999,999 iterations. Now take the *identical* call site — same line of code, same
`Invoke` instruction — but feed it a list that interleaves `Circle`, `Square`, and `Triangle`
instances in rotation. A monomorphic cache at that site now *misses every single time*: each call's
receiver class differs from whatever the slot last held, so every call pays the full walk *plus* a
wasted comparison, which is strictly worse than never having cached at all. A PIC with three or more
slots handles this specific rotation cleanly (all three classes fit); a PIC with only two slots would
itself thrash on a three-class rotation, which is the concrete version of "megamorphic" — not an
abstract threshold, but *more distinct classes at this site than this site's cache has room for*.

**Redefinition mid-loop.** Take a loop sending the same selector to the same receiver class a million
times — the monomorphic-friendly case — but have the program redefine the method on that receiver's
class partway through, at, say, iteration 500,000. The predictive question a reader should be able to
answer without being told the answer: *does the loop's remaining 500,000 iterations see the old
method or the new one, and how would the VM know which to serve?* The chain-walk-every-time design
(no cache) gets this right automatically and uninterestingly — it re-resolves every time, so it picks
up the redefinition on the very next send, iteration 500,001. Any of the cached designs get it right
*only if* the redefinition act itself participates in invalidation — the method-install operation
must be the thing that bumps whatever the cache is watching (a global counter, the receiver class's
own epoch, or both). If it does, the call site's stale slot fails its next stamp check, falls through
to the general lookup, finds the new method, refills, and the remaining iterations run the new
version — the *cache misses exactly once* at the point of change and is silent about the fact that
anything happened. If some mutation path in the runtime does *not* participate — a method-install
routine that forgets to bump the counter, or a per-class scheme that bumps the wrong subtree — the
loop keeps calling the old method forever, and this is indistinguishable, from the outside, between
"working as designed" (the language chose not to make this construct visible to caches, rare and
usually explicit) and "a bug in the invalidation bookkeeping." That indistinguishability is exactly
why invalidation is the hard half: a cache-shape mistake degrades performance; an invalidation
mistake silently degrades *correctness*, and looks, from a passing glance at the loop's output, like
nothing at all happened.

```mermaid
sequenceDiagram
    participant Loop
    participant Site as Call site (Invoke @ ip)
    participant Slot as Cache slot
    participant Lookup as lookup_method (chain walk)
    participant World as Method table / version stamp

    Loop->>Site: send #1
    Site->>Slot: probe (empty)
    Slot-->>Site: miss
    Site->>Lookup: walk chain
    Lookup-->>Site: (class, method)
    Site->>Slot: fill (class, method, stamp=S0)
    Site-->>Loop: invoke method

    Loop->>Site: send #2 .. #N
    Site->>Slot: probe (class match, stamp==S0?)
    Slot-->>Site: hit
    Site-->>Loop: invoke cached method (no walk)

    Note over World: method redefined — stamp bumps to S1
    Loop->>Site: send #N+1
    Site->>Slot: probe (stamp S0 != current S1)
    Slot-->>Site: miss — stale
    Site->>Lookup: walk chain (again)
    Lookup-->>Site: (class, new method)
    Site->>Slot: refill (class, new method, stamp=S1)
    Site-->>Loop: invoke NEW method
```

## A different lever entirely: deleting the dispatch, not the lookup

Everything above makes a *lookup* cheap. A completely different technique attacks a different cost:
the overhead of the interpreter's fetch–decode–execute turn itself — the cost of landing on an
instruction, decoding which opcode it is, and jumping to the code that handles it, independent of
what that code then does. Call this **dispatch overhead** to distinguish it sharply from *work*: the
work of, say, a hash-map insert is the hashing and the insert; the dispatch is the machinery that got
the interpreter's instruction pointer to the bytecode that requested the insert in the first place.

A bytecode compiler can often see, statically, that certain opcode *pairs* occur adjacent to each
other constantly — "load a local variable, then invoke a method on it" is an enormously common
two-instruction sequence, because that's what `x.foo()` compiles to. **Superinstruction fusion** is
the peephole transformation that recognizes such a pair at compile time and replaces the two
instructions with *one* new opcode that does both jobs — `(LOAD_LOCAL, INVOKE)` collapses into a
single `INVOKE_LOCAL`. The interpreter's main loop then pays exactly one fetch-decode-execute turn
for what used to be two, for every occurrence of that pattern the compiler found.

This is a **peephole optimization** in the classical sense — a small, fixed-size window of adjacent
instructions is inspected and locally rewritten, with no need to understand anything about the
program beyond that window — and the *specific* technique of composing primitive operations into
larger units with less per-unit dispatch overhead has real ancestry worth naming. Forth, dating to
around 1970, is built entirely on **threaded code**: a Forth "word" compiles not to a sequence of
instructions in the usual bytecode sense but to a sequence of *addresses* of other words, and a tiny
shared epilogue (traditionally called `NEXT`) at the end of each primitive's code simply jumps to
whatever address comes next in the thread. The literature distinguishes several flavors of this by
what exactly gets threaded — *direct* threading (each slot holds a code address) and *indirect*
threading (each slot holds a pointer to a pointer, one more hop, to allow a shared header) are the
common ones; **token threading** — the thread is a sequence of small integer opcode tokens, dispatched
through a table rather than jumped to directly — is the variant that reads most like an ordinary
bytecode interpreter's opcode stream, and is the one worth holding next to superinstruction fusion,
since both are ultimately about how many dispatch turns a given unit of composed behavior costs.
Composing primitives into a word in Forth is, in spirit,
exactly the same move as fusing two opcodes into one: you are trading "one dispatch per primitive
operation" for "one dispatch per composite," at the cost of needing a distinct piece of code (a
distinct word, or a distinct fused opcode) for every composite you decided was common enough to
special-case. **[flagged — moderate confidence: Forth's threaded-code dispatch and modern
bytecode-VM superinstruction fusion are close cousins in motivation and mechanism, but I would not
claim a direct citation lineage from Forth's design to the specific "peephole-fuse-adjacent-opcodes"
technique; the more direct academic ancestor for that specific move, to the best of my recollection,
is Todd Proebsting's work on combining common opcode sequences into single "superoperators" in
bytecode interpreters (Proebsting, "Optimizing an ANSI C Interpreter with Superoperators," early-to-
mid 1990s — I recall this as POPL 1995 but hold that venue/year with only moderate confidence), with
further development in the interpreter-dispatch literature by Ertl, Gregg, and collaborators
(including Casey, Ertl, and Gregg's work on combining superinstructions with other dispatch
techniques such as stack caching, roughly mid-2000s) — exact titles and years not verified here.]**

A related but distinct technique from the same lineage is **quickening**: rather than fusing two
*different* opcodes discovered at compile time, quickening rewrites a single *generic* opcode into a
cheaper, more specific one *after* it has actually executed once and revealed which specific case
applies — CPython's PEP 659 mechanism described above is a modern, fully worked example of exactly
this, and the term itself has currency in the interpreter-optimization literature going back at least
to work on efficient bytecode interpretation in the 2000s and 2010s (Stefan Brunthaler's work on
quickening for Python-like interpreters is a name worth attaching to this **[flagged — I recall
Brunthaler's "efficient interpretation using quickening" line of work with moderate confidence on
substance, low confidence on exact title/venue/year]**). Fusion and quickening are siblings, not
identical: fusion decides *at compile time*, from static adjacency, which pairs to merge; quickening
decides *at run time*, from what a single instruction actually saw, which specialized replacement to
install. Both remove dispatch overhead; neither, by itself, removes work.

### Why removing instructions is not the same as removing time

This is the point at which the two levers — caching and fusion — must be kept apart in the reader's
head, because they are easy to conflate and their payoffs are not interchangeable. A cache removes a
*lookup* (a hash and a chain walk) and replaces it with a *check* (a compare); the check still costs
something, but far less than what it replaced, and that saving scales with however expensive the
lookup was. Fusion removes an entire *dispatch turn* — but a dispatch turn was never the expensive
part of *most* instructions to begin with.

Concretely: suppose a fetch–decode–execute turn in a reasonably tight interpreter loop costs some
small, roughly fixed number of nanoseconds — the exact figure depends on the host CPU, the branch
predictor's success rate on the dispatch's indirect jump, and the interpreter's specific dispatch
technique (a `match`/switch, computed goto, or direct/indirect threading), but it is, illustratively,
on the order of one to a few nanoseconds for a well-predicted dispatch, and meaningfully more when
the branch predictor mispredicts (which it does more often exactly when the *mix* of opcodes seen at
a dispatch site is unpredictable — a nice, if ironic, echo of the megamorphic problem one level down
in the stack) **[flagged — illustrative order-of-magnitude only; this is not a measured claim about
any specific runtime, it is the kind of number interpreter-dispatch literature reports for well-
predicted indirect branches on modern hardware]**. Fusing two instructions saves *one such turn* —
call it the fixed dispatch cost, D. If the fused-away instruction's actual *body* (the work it does
once dispatched to — say, a local-variable load, which is close to free) costs on the order of D
itself, fusion roughly halves the cost of that pair, a real and visible win. But if the fused-away
instruction's body is a **method invocation** — the very thing this document's first half spent so
much effort making cheap-but-not-free — its cost, even on a cache hit, includes a class check, a
frame push, and whatever the callee's body does, easily an order of magnitude or more above D. In
that case, fusing it into its neighbor removes one dispatch turn's worth of overhead from a total
cost that is mostly *not* dispatch overhead, and the saving is proportionally tiny — noise against
the invocation's own cost.

The general form of this, worth stating as a standalone claim because it is the single most
important intuition-correcting fact in this entire lever: **the fraction of instructions a fusion
pass removes from a program is not the fraction of execution time it saves, and the two can differ by
an order of magnitude or more in either direction of surprise.** A pass that eliminates, say, one
instruction in eleven from a program's instruction stream (an eminently plausible fusion yield for
`load-then-invoke`-shaped patterns) saves *time* in proportion to how much of the program's *actual
runtime* those eliminated instructions' dispatch overhead — not their work, their dispatch overhead
alone — represented. For a program dominated by cheap instructions (arithmetic, local loads, stack
shuffling — a "dispatch-bound" program, one where the interpreter spends more cycles deciding what to
do than doing it), that fraction can be close to the instruction-count fraction, because dispatch
*is* most of the cost. For a program dominated by heavy instructions — allocation, hashing, garbage
collection, or (again) a cache-missing megamorphic send — the same instruction-count reduction buys
almost nothing, because dispatch was never where the time was going. Fusion is a lever with a
domain of applicability, not a universal multiplier, and that domain is exactly "code whose time is
dispatch-bound," which is a property of the *workload*, not of the compiler pass.

## The two levers, examined together

**V8's ladder, as a single worked example spanning both levers.** V8's handling of JavaScript
property access is a useful place to see cache-shape design and its downstream consequences in one
system, because it runs the full monomorphic → polymorphic → megamorphic progression on real,
observable programs, and because it connects the cache-shape story to a related-but-distinct concept
this document has deliberately kept at arm's length: **deoptimization**. V8 gives objects with
identical property layouts a shared descriptor — historically termed **hidden classes** (V8's own
early terminology; the underlying concept, and a form of the term "map," originates with SELF's own
representation technique for exactly this purpose, and other engines use their own names, e.g.
"shapes") — so that a property access compiled against a known hidden class becomes a fixed-offset
load rather than a dictionary probe, which is the property-access analogue of a monomorphic inline
cache. A property-access site that observes more than one hidden class over its life climbs the same
ladder described above: monomorphic → polymorphic (a bounded, V8-internal PIC) → megamorphic (a
shared, program-wide fallback structure, this system's version of the global method cache). Where V8
adds a genuinely distinct idea is at the optimizing-compiler tier: when V8's JIT compiles a hot
function, it can bake in an assumption drawn from a site's observed feedback (for instance, "this
property access has only ever seen one hidden class, so compile it as a direct offset load with a
single guard, no polymorphism handling at all"). If that assumption is later violated by a genuinely
new object shape reaching the compiled code, the guard fails and the function **deoptimizes** —
execution abandons the optimized machine code mid-flight and falls back to the interpreter (or a
lower optimization tier), which can then handle the new case generally and, in some designs, feed the
new observation back in so a future recompilation can account for it. This is worth distinguishing
sharply from invalidation as covered above: **invalidation is a cache noticing a *stored binding* has
gone stale; deoptimization is a *compiled, speculatively-optimized function* noticing one of its
*baked-in assumptions* has gone false.** They rhyme — both are "a fast path's precondition broke, and
something has to notice and fall back" — but deoptimization only exists where there is compiled code
carrying speculative assumptions to begin with, which requires a JIT. A pure bytecode interpreter,
however cache-rich, has no optimized machine code to fall out of — its "fallback" is always just the
next-slower interpreter path, which is why this document treats deoptimization as V8's concept to
name, not a mechanism to build.

**Cut from this document, and why.** HotSpot (the JVM) runs a call-site specialization ladder with
essentially the same shape as V8's — monomorphic and bimorphic inline caches at call sites, a
megamorphic fallback through a virtual dispatch table — and does not earn a separate section here
because, for the specific axis this document is walking (cache shape and its bill), it would say the
same thing V8's treatment already says; a JVM-specific deep dive would be repetition wearing a
different name, not new information about the design space. Ruby's open-class model — where any code,
anywhere, can reopen and mutate any class's method table at any time, including built-in classes — is
real and relevant *motivation* for why invalidation is hard at all, but it is motivation, not new
mechanism: Ruby's caches face the identical soundness problem described in the invalidation section
above and are not meaningfully re-taught by walking through Ruby's specific implementation choices
here. And JIT compilation as a subject in its own right — tiering strategies, inlining heuristics,
speculative optimization beyond the one-paragraph connection type feedback and deoptimization
required above — is a large enough topic that folding it in here would dilute rather than sharpen the
two levers this document is actually about; a pure bytecode interpreter with no JIT can adopt every
design discussed above (cache shape, invalidation granularity, fusion) without ever needing a
compiler back end, and that is precisely the setting this document has been written for.

## Where the model strains

Two tensions are worth surfacing explicitly, because each is a place where the "cache = remembered
answer" mental model, comfortable everywhere else in this document, stops being simply true.

**Caching exists only because binding is late, and must be undone only because binding stays late
forever.** This is not incidental — it is the whole shape of the problem, worth stating as a single
claim: a language with a genuinely dynamic object model (methods resolved by name, against a table
that can be mutated after the program starts) is the *only* setting in which any of this machinery
has a reason to exist. A statically-typed, statically-bound language where `x.foo()` compiles, once
and for all, to a direct call at a known address (or, for virtual dispatch, a fixed vtable slot index
that is itself immutable after compilation) never needs an inline cache, because there was never a
per-call *decision* being remembered — the decision was made once, at compile time, and cannot go
stale because nothing in the language model allows the table it was drawn from to change underneath
it. Every mechanism in this document — the slot, the guard, the version tag, the megamorphic
fallback — is scaffolding built specifically to make a *late* decision behave, on the fast path, as
though it had been made *early*, while remaining honestly willing to re-decide the moment the world
proves it wrong. The cache is not an optimization bolted onto dynamic dispatch as an afterthought; it
is dynamic dispatch's own admission that "late" and "re-derived from scratch every time" are
different properties, and that a language can keep the first without paying for the second — but only
by accepting the entire invalidation apparatus as the permanent cost of keeping the second's
alternative honest. A cache in a dynamic-object-model language can never be "finished" in the way a
compiled call is finished; it is finished-until-proven-otherwise, forever, for as long as the program
runs, because the thing it is caching an answer to (the class-vs-method binding) is, by the language's
own design, always still an open question.

**Fusion's soundness depends on a fact about control flow the fusion pass itself did not establish.**
Fusing two adjacent instructions into one and rewriting the bytecode stream in place — dropping the
first opcode into a new fused form, and typically leaving the second instruction's *slot* either
removed or turned into an unreachable placeholder — is only sound if **no jump target in the entire
program lands on the instruction being absorbed.** If some other point in the code computes a branch
to what used to be the second instruction of the fused pair — a `goto`, a loop back-edge, an
exception handler's resume point, anything that can transfer control to an arbitrary instruction
offset rather than only ever falling through sequentially — that jump now lands either on dead code,
on the middle of a multi-byte fused instruction's operand bytes (semantic garbage), or past the end
of what the fusion pass assumed was one atomic unit, any of which is a correctness bug indistinguishable
in symptom from memory corruption. Guarding against this requires the fusion pass to have, in hand,
a complete and correct set of every instruction offset that is a legitimate jump target *before* it
decides what is safe to merge away — and this is not a fusion-specific problem, it is the general
form of the **peephole-safety problem** that any local, in-place bytecode transformation faces: any
optimization that changes instruction boundaries, reorders instructions, or deletes an instruction
outright is sound only relative to a correctly and completely computed set of control-flow entry
points into the transformed region, and an incomplete jump-target analysis is a silent, load-bearing
soundness gap in exactly the same shape as an incomplete invalidation subtree walk above — a rare
input (a jump that happens to target the fused-away offset) is required to expose it, which is
precisely the profile of bug that survives ordinary testing and shows up only later, on the input
nobody thought to construct.
