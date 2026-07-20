# 04 — Inline Caches and Speculation

Being fast by being wrong rarely, and surviving being wrong. The through-line: *every
cache is a bet on a world that is allowed to change, so the design question is never "is
it fast" but "what does it cost to be wrong, and who pays on the write side."*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — What the guard actually checks

```js
function dist(p) { return p.x * p.x + p.y * p.y }   // called a million times
```

The `p.x` site caches a shape and an offset, and guards with one pointer compare.

1. What must be true about the object representation for a single pointer compare to be a
   *sufficient* check? Enumerate everything the cached answer depends on.
2. Why can the guard not be "walk this object's fields and check that `x` is at offset 4"?
   The answer is not "that's slow" — say what makes it structurally unusable.
3. Deutsch and Schiffman's Smalltalk system patched the call instruction itself. V8's
   Ignition stores the cache in a side table. Both are monomorphic inline caches. Name
   exactly what differs about the guard.

### Q2 — Two constructors, one field set

```js
function A() { this.x = 1; this.y = 2 }
function B() { this.y = 2; this.x = 1 }
```

1. `new A()` and `new B()` have identical field sets and different shapes. Explain why,
   and argue that it is not a bug — say what the alternative would break.
2. Describe the transition tree: what makes it a tree rather than a DAG, and what happens
   to a program that adds fields in a data-dependent order?
3. `delete o.x`. What do engines actually do, and why is the answer usually "stop using
   shapes for this object at all"?

### Q3 — One shape, then four, then forty

A property access site is monomorphic, becomes polymorphic, then megamorphic.

1. Why is a polymorphic inline cache a *linear chain of compares* rather than a hash
   lookup, and at what point does that reasoning stop holding?
2. The megamorphic fallback in V8 and JSC is a cache shared by the whole program, keyed
   on shape and name. Argue why global is right here and a per-site cache is wrong.
3. A site is megamorphic. Name two things still worth doing that beat a full lookup, and
   one thing you must actively stop doing.

### Q4 — Epoch counter versus dependency list

Two invalidation designs:

- **A** — one global counter, bumped on any method definition. Every cache slot stores the
  counter value it was filled at and compares before use.
- **B** — each cache site registers on a per-class dependency list. A definition walks the
  list and invalidates just the affected sites.

1. Give the read-side and write-side cost of each, and name the workload that makes A
   collapse.
2. One of these cannot be used by an optimizing JIT that has already *inlined* the
   method. Which, and why — be precise about what is missing.
3. Name the third design, where it is actually deployed, and the platform constraint that
   has made it less attractive over the last decade.

### Q5 — The world is open

```ruby
class String
  def upcase; "nope"; end
end
```

...executed an hour into a hot process.

1. Beyond "a method was redefined", enumerate the distinct open-world events a runtime
   must treat as invalidating. Two of these are missed by almost everyone.
2. Java mostly forbids this, yet HotSpot still needs class-hierarchy-analysis
   invalidation. Why?
3. A runtime "seals" classes after startup to close the world. What have you actually
   bought, and what is the failure mode of the *sealing rule itself*?

### Q6 — Caching a global's slot

`print(x)` in a hot loop, where `print` is a module-level binding. You cache the slot
index at the call site.

1. Why is caching the *slot* dramatically better than caching the *value* — and name the
   case where caching the value is worth far more.
2. Shadowing is the hazard. Construct the failure precisely, and say what makes it a
   silent wrong answer rather than a crash.
3. Cache in the instruction stream versus in a per-function side array. Pick one, then
   explain why CPython does the opposite and is right to.

### Q7 — Field caches and call caches are different problems

1. State what a field IC can do that a call IC structurally cannot, and vice versa.
2. The property is found on a prototype three links up. Guarding on the receiver's shape
   is not sufficient. What else must be guarded, and what is the trick that keeps the
   guard O(1) regardless of chain depth?
3. Which of the two is worth more in a plain bytecode interpreter with no JIT? The answer
   is usually the opposite of what people assume — say why.

### Q8 — Who collects the feedback

Interpreter → baseline → optimizing.

1. Why must the *first* tier collect type feedback, and what goes wrong if you make tier
   zero too fast to profile?
2. The feedback the first tier gives you is systematically biased. In which direction,
   and what is the concrete pathology that results?
3. A baseline JIT is neither the fastest to start nor the fastest to run. Give two
   independent reasons it exists anyway — one of them is not about speed at all.

### Q9 — Materializing what the optimizer erased

Optimized code has scalar-replaced an object, kept a value only in a register, and
inlined three call frames. A guard fails.

1. Enumerate what must be reconstructed, and say why "just re-run the function from the
   top" is not available.
2. Why does deopt metadata dominate the complexity — and often the memory — of an
   optimizing tier?
3. Name what the existence of deopt *forbids* the optimizer from doing.

### Q10 — On-stack replacement, both directions

1. OSR-in solves a problem ordinary tiering cannot. Name it, and say what makes the
   compilation itself unusual.
2. Deopt is OSR-out. Name the asymmetry that makes one direction fundamentally harder
   than the other.
3. A hot loop sits in a function called exactly once. Without OSR, what does the system
   do — and what widely-repeated benchmarking advice is a direct consequence?

### Q11 — Deopt storms

A site deopts. The method is recompiled. It deopts again.

1. Why is naive "recompile on deopt" *catastrophic* rather than merely slow?
2. What is the minimum state needed to prevent it, and — the part people get wrong —
   where must that state live?
3. HotSpot, LuaJIT, and CPython's specializing interpreter each have a version of this
   mitigation. Name each, and state the shape they share.

### Q12 — Inlining is not about call overhead

You inline a two-line accessor and the benchmark improves far more than a call costs.

1. If call overhead were the point, name the cheaper fix. Then say what inlining actually
   buys.
2. Why is "inline if the callee is small" insufficient as a heuristic, and what do real
   systems use instead?
3. Speculative inlining at a polymorphic site: what does the emitted code look like, and
   how many targets is it worth doing for before the trade inverts?

### Q13 — When the cache is a net loss

A site's receiver alternates between two shapes on every iteration. The site has a
one-entry monomorphic cache.

1. Enumerate the per-miss costs, and compare honestly against having no cache at all.
2. Why can a 95/5 polymorphic site be perfectly healthy while a 50/50 site is
   pathological, even when the arithmetic "hit rate" is not that different?
3. Give your fix options and the cost of each. One of them is not a VM change.

### Q14 — Patching code versus writing data

1. Give the read-side cost difference precisely — down to the instructions.
2. Name three constraints that have pushed modern engines toward the data side.
3. Name the case where patching still wins decisively, and why.

### Q15 — The number says 94%

You instrument every inline cache in the VM and report a 94% hit rate.

1. Why is that number nearly meaningless as stated? Name the two independent weightings
   it is missing.
2. Construct a site with a 100% hit rate that you should delete.
3. What measurement actually establishes that an IC is earning its place?

### Q16 — Speculating on something other than a type

```js
for (let i = 0; i < a.length; i++) sum += a[i];
```

1. Name three distinct speculations beyond "the receiver's type" that an optimizing tier
   makes here, and the guard for each.
2. V8 speculates that arithmetic stays in the small-integer range. What does *that* guard
   cost, and what makes this speculation different in kind from a shape guard?
3. "This global still holds the same function" is among the highest-leverage speculations
   in any dynamic language. Say why, and say what makes its fast-path cost zero.

---

## Answers

### A1 — What the guard actually checks

**1.** The shape pointer must be a complete summary of *everything the cached answer
depends on*: that the field exists, its offset, whether it lives in the object's inline
slots or in an out-of-object backing store, that it is a plain data property and not an
accessor or a proxy trap, and — for anything reached through inheritance — that the
lookup path is unchanged. If any of those can vary while the shape pointer stays equal, the
guard is unsound. This is why a shape is not "the set of field names": it is "everything a
lookup would have consulted". V8's maps encode the prototype, the elements kind, and
assorted flags for exactly this reason. Where something genuinely cannot be folded into the
receiver's own shape — a *prototype's* shape changing — you need a second guard, which is
Q7.

**2.** Because that is a **search**, and the search is precisely the thing you were
caching. A guard that costs the same order as the operation it guards is not a cache, it is
overhead with extra steps. But the structural objection is sharper than cost: a field scan
is a loop with a data-dependent trip count, i.e. an unpredictable branch in the single
hottest position in the program. The identity compare is O(1), branch-predictable when the
site is stable, and requires no loads beyond the object header you were going to touch
anyway. "Identity-cheap" means exactly this: constant time, one comparison, no new memory
dependency.

**3.** The comparison is identical; what differs is where the compared value comes from and
who may write it. Patched code encodes the cached shape as an **immediate in the
instruction stream**, so the guard is a compare against a literal — no load at all — and
the miss path calls a stub that rewrites the site. A side table holds the cached shape in a
feedback slot indexed by site, so the guard is a *load then compare* — an extra dependent
load with its own cache behaviour. In exchange, the code stays shared across every closure
of that function, it can be read concurrently without patching protocol, it works where
writable-executable memory is restricted, and the accumulated feedback survives the
compiled code being thrown away.

**Trap.** "The guard checks the type." It checks *identity of a summary*, and the summary
must cover every input to the cached decision — including things that are not the
receiver's type at all, such as whether some prototype four links away was mutated last
Tuesday. Say "type" and you will build a guard that is silently insufficient.

### A2 — Two constructors, one field set

**1.** Because a shape *is a path*. Shapes are produced by transitions from a parent shape,
one added field at a time, and offsets are assigned in insertion order. Two insertion
orders produce two offset assignments, hence two shapes. Not a bug: the entire product of a
shape is that it fixes offsets, and offsets can only be fixed by fixing an order. The
alternative — canonicalize by sorting names — means adding a field to an object could
*move* fields that were already there, which invalidates every cached offset for that shape
and requires re-laying-out live objects at runtime. You would trade a shape explosion for a
much worse problem.

**2.** It is a tree rooted at the empty shape, one edge per "add field *f* with these
attributes". Tree rather than DAG because a shape is defined by its path and the two paths
of part 1 must remain distinguishable — merging them *is* the canonicalization you just
rejected. The tree is shared structure: every object built the same way traverses the same
nodes, which is what makes shapes memory-cheap in the first place. The hazard: field
addition in data-dependent order — an object built by iterating a hash, one built from JSON
keys in arrival order, `if (opt) this.z = ...` — fans the tree out combinatorially. No two
objects share a shape, every downstream IC goes megamorphic, and you have paid the tree's
memory with none of its benefit. The advice "initialize all your fields in the constructor,
in a fixed order" is a direct *consequence* of the transition tree, not folklore.

**3.** Either a transition to a shape that records a hole — preserving the offsets of
everything after the deleted field, at the cost of a dead slot and yet another tree node —
or, far more commonly, converting the object to **dictionary mode**: a genuine hash map
with no fixed offsets, where every IC on that object goes megamorphic permanently. Engines
choose dictionary mode because the alternatives are shapes-with-holes (which multiply the
tree and leak slots) or re-layout (which invalidates caches and moves live data). Dictionary
mode is the runtime's admission that this object is being used as a *map*, not as a
*record*, and the shape machinery has nothing to offer a map. V8 has a path back — it can
re-normalize a dictionary object into a shape — but it is a deliberate, slow transition,
not something that happens under you.

### A3 — One shape, then four, then forty

**1.** Because at small N, a sequence of compares against immediates beats a hash on every
axis: no hash computation, no table load, and — when the site is *biased* toward one shape
— excellent branch prediction, so the expected cost is close to one compare. A hash costs a
hash, a dependent load, and an unpredictable branch before you learn anything. The reasoning
stops holding at a handful of entries; V8 caps polymorphic sites at four shape/handler
pairs, and the caps elsewhere are the same order. The right way to see it: a PIC is a
*decision list tuned for a skewed distribution*, and going megamorphic is the admission
that the distribution is not skewed.

**2.** It keys on (shape, name) and is shared by every site in the program. Global is right
because megamorphic sites are usually megamorphic for a shared reason — a generic utility,
a framework's property access, a serializer — and the same (shape, name) pairs recur across
many such sites, so a shared table gets *more* hits than the sum of isolated ones. A
per-site cache at a megamorphic site would thrash on its own traffic and would multiply
memory across thousands of sites for no hit-rate gain at all. V8's and JSC's stub caches
are exactly this: a small, direct-mapped or low-associativity global hash table. The cost of
sharing is that one megamorphic site's traffic evicts another's, so the table's behaviour
is a whole-program property and hard to reason about locally.

**3.** Still worth doing: (a) probe the global stub cache, which is enormously cheaper than
a prototype-chain walk ending in a dictionary lookup; (b) cache the *shape of the
operation* even when you cannot cache its answer — "this site has only ever seen plain data
properties on the receiver itself, never an accessor, never a proxy, never an index-named
property" lets you keep a fast lookup routine even though the target varies. Must stop:
**updating the cache**. A cache that rewrites its entry on every miss at a megamorphic site
is pure cost — you pay the store, dirty a cache line, possibly generate coherence traffic,
and never hit. Every serious engine has an explicit terminal "stop caching here" state, and
forgetting to implement it is a common self-inflicted slowdown that looks like a mystery.

### A4 — Epoch counter versus dependency list

**1.** *A (epoch).* Read side: one extra load and compare per cache use — cheap, and the
counter is a read-mostly shared line, which is fine. Write side: O(1), bump the counter.
The hidden cost is blast radius: **any** definition anywhere invalidates **every** cache in
the program. The collapsing workload is code that defines methods at runtime in a loop —
defining singleton methods per object, metaprogramming that generates accessors, a REPL
reopening classes, lazily loading libraries — where the process spends its life flushing
and re-warming every cache it has. Ruby lived this with a single global method-cache serial
and moved to finer-grained per-class serials for exactly this reason.
*B (dependency lists).* Read side: **free** — the cache stores no version; if it is there,
it is valid. Write side: expensive and fiddly. You maintain lists whose memory is
proportional to sites times dependencies, an invalidating definition must find every
dependent (for a class-hierarchy dependency that means walking subclasses), the lists must
be correct under concurrency, and they must be collectible when the dependent code dies.

**2.** **A** cannot be used once the method has been inlined. After inlining there is no
cache slot left to check — *the code is the cache*. A version check would have to be
re-inserted at the inline site, which defeats the entire purpose (you inlined to expose the
body to the optimizer; a check per call reinstates the guard you removed) and does not even
have a well-defined home after the optimizer has hoisted, merged, and specialized the
inlined code. So an optimizing tier requires an invalidation mechanism that reaches
*compiled code*: HotSpot records a class-hierarchy-analysis dependency on the nmethod, and
when a class is loaded that violates it the nmethod is made not-entrant and executing
frames are deoptimized. Graal's `Assumption` objects are the same mechanism, made explicit
and first-class. Zero read-side cost is the only acceptable cost inside optimized code, and
only B and C deliver it.

**3.** **Code patching / watchpoints**: the cached value is an immediate in machine code,
and invalidation rewrites the instruction — or flips a branch to jump to a bailout stub.
JSC's watchpoint sets do this: firing a watchpoint jettisons or repatches the dependent
code. Read cost is zero, like B. Write side must handle other threads currently executing
the instruction being patched, hence atomic naturally-aligned writes, icache invalidation,
and often a safepoint. The constraint that has degraded its appeal is **W^X**: on iOS,
Apple Silicon, and hardened server configurations, writable-executable memory is restricted
or requires per-thread mode switching, so every patch costs more than it used to. That is a
significant part of why data-side feedback became the default design.

**Trap.** "Epoch counters are just dependency lists at a coarser granularity." They are not
points on one axis — they are different mechanisms with different preconditions. An epoch
works only where a *check* is possible, and the entire value of an optimizing tier is the
removal of checks. The moment you inline, epochs are not a coarse option; they are not an
option.

### A5 — The world is open

**1.** Method definition, redefinition, and removal; class reopening; **inserting a
superclass, module, or mixin into an ancestry**, which re-linearizes lookup for an entire
subtree in one operation; **creating a singleton / eigenclass for an individual object**,
which changes that object's lookup without touching its class at all; a `method_missing` /
`__getattr__` fallback becoming defined, which changes the meaning of *absence*; a global
or module binding being reassigned; a prototype object being mutated; the prototype *link*
being reassigned (`__proto__`, `Object.setPrototypeOf`); replacing a data property with an
accessor; freezing or sealing, which is a transition that can *enable* caching and so must
also be observed; and simply loading new code. The two that get missed are the ancestry
insertion and the singleton-class creation, because neither is a "method definition" and
both change lookup results immediately.

**2.** Because Java's world is open in one specific dimension: **class loading**. C2 will
devirtualize a call when class-hierarchy analysis says there is exactly one implementor —
but a class loaded a minute later can add a second. So the devirtualization is a
speculation with a recorded dependency, and the JVM must be able to invalidate the compiled
code and deoptimize live frames when it breaks. This is the entire performance argument for
`final` and for sealed hierarchies: they convert a bet into a fact, so the optimizer needs
no dependency and no deopt path.

**3.** Bought: guards you can **delete** rather than merely make cheap, and inlining that
needs no dependency list or watchpoint at all. Failure modes of the rule: it must be
checkable and it must be honest. If sealing is "after `main` starts", then anything that
loads lazily — a plugin, a `require` inside a function, a deserializer that synthesizes
classes, a test harness — either breaks or forces an unseal, and unsealing is a global
deoptimization event that can cost more than the sealing ever earned. If sealing is
per-class and opt-in, the ecosystem will not use it: Java's `final` is the natural
experiment — it is free, it is correct, and libraries still avoid it, because marking a
class final is an *API commitment* about extensibility, not a performance annotation.
Treating sealing as a performance switch when it is really a compatibility decision is the
general trap.

### A6 — Caching a global's slot

**1.** Because the binding's *storage location* is stable while its value is not. Cache the
slot and every read is a load with no lookup and no invalidation on assignment; cache the
value and you must invalidate on every write, which in a language with mutable globals is
the common case. The exception, and it is a big one: when the language or the runtime can
establish that the binding is effectively constant after initialization — a `const`, or a
module-level function nobody reassigns — then caching the *value* lets the optimizer
**constant-fold** it, which is worth vastly more than saving one load, because a constant
callee unlocks inlining and everything downstream. V8's script-context constness tracking
and the general family of "this cell still holds what it held" mechanisms exist to buy
exactly that, paid for with an invalidation dependency.

**2.** The failure: a name resolves to a particular slot at compile time and later resolves
to a *different binding* at the same site. Concretely — a global table that is rehashed or
compacted, so indices move; a language where a binding can be introduced dynamically
(`eval` injecting a name, a `with` scope, a module later defining a name that shadows an
import, a class body reopening a name); or two compilation units whose global tables are
keyed the same way and collide. It is silent because the cached index is *valid*: it points
at a real slot holding a real value. There is no fault, no type error, no bounds violation
— just the wrong variable's value flowing into your program. That is the worst failure mode
a cache can have, and it is why the structural fixes matter: make the cache hold a pointer
to a **stable heap cell** so compaction cannot move it, and make compile-time resolution a
*fact* by forbidding after-the-fact shadowing. If the language permits dynamic shadowing,
the cache needs a guard on the *resolution*, which usually means an epoch on the scope.

**3.** The side array, generally: writing into the instruction stream makes the code
non-shareable across closures and threads, requires the code to be writable in a running
process, and invalidates icache. A per-function feedback array is ordinary data, the code
stays shared, and the update is a plain store. CPython does the opposite — its inline caches
live literally in the code array, as `CACHE` entries interleaved with instructions — and it
is right to, because its constraints differ: a code object's bytecode is already
per-code-object rather than shared across closures, the array is data rather than executable
memory (so no W^X, no icache concern), and execution is serialized. When your constraints
remove all three objections, the "wrong" answer becomes the right one — and it buys the
cache being adjacent to the instruction that reads it, which is excellent locality.

### A7 — Field caches and call caches are different problems

**1.** A field IC's cached answer is an **offset** — a small integer that reduces the whole
operation to a compare and a load. There is nothing left over. A call IC's cached answer is
a **target**, and the call still happens; the cache removes the lookup, not the call. So a
field IC's best case is two instructions while a call IC's best case still pays a call.
Conversely, the call IC enables something the field IC cannot: once a target is known and
stable, an optimizing tier can *inline the body*, and that unlocks constant propagation,
escape analysis, and the entire second-order payoff of Q12. Field caches are the bigger
interpreter win; call caches are the bigger JIT win.

**2.** You must also guard that nothing along the path from receiver to holder changed —
that no intervening prototype gained a shadowing property and no prototype link was
reassigned. Walking the chain to verify defeats the cache. The trick is a **validity cell**
(V8's prototype validity cell; JSC's structure chains and watchpoints are the same idea):
each prototype's lineage owns a cell, the IC caches (receiver shape, validity cell, offset),
and any mutation to any prototype in that lineage invalidates the cell. The guard is
therefore two pointer compares regardless of how deep the chain is. The general principle
is the one that recurs throughout this file: **turn an O(depth) read-side check into an
O(1) one by making the rare write side do the work.**

**3.** Field caches, usually — for two reasons. First, in an interpreter there is no
inlining to unlock, so a call IC saves you a method lookup and literally nothing else.
Second, in many dynamic languages the method lookup is *already* cheap: if selectors are
interned to integers and each class carries a flattened method array (Wren indexes a
class's method buffer by symbol id; the Smalltalk lineage flattens method dictionaries),
dispatch is one array index and a call IC's headroom is tiny. Meanwhile property access may
still be a hash probe on a string key, where the headroom is enormous. Measure both lookups
before assuming the call site is the interesting one.

**Trap.** "Inline caches are for method dispatch." Historically true — Deutsch and
Schiffman were caching sends — but in a modern dynamic language the property-access caches
carry most of the win, which is why V8's IC engineering is dominated by load and store ICs
rather than call ICs. Repeating the historical framing will lead you to optimize the site
that was already fast.

### A8 — Who collects the feedback

**1.** Because the optimizer's speculations are only as good as the observed distribution,
and the only place to observe is wherever the code runs first. A tier with no feedback
forces the optimizer either to speculate blind — guessing from static types, or compiling
fully generic code — or to insert its own profiling and recompile, which is
profile-guided optimization with no profile, i.e. an extra tier you did not admit to
building. The Dart VM states this most cleanly: the unoptimized code's `ICData` *is* the
type feedback, and the optimizer reads it directly. Make tier zero too fast to profile and
you have bought a faster warmup and lost the input to every later tier.

**2.** It is biased toward the **warmup phase**: initialization code, the first few inputs,
the monomorphic startup path of a site that becomes polymorphic in the steady state. The
pathology: a site that sees one shape during setup and many afterwards gets optimized
monomorphic and then deopts forever — Q11's storm, arriving by way of an honest
measurement. Compounding it, counters are attributed per site but *phases are not
recorded*, so you cannot distinguish "was monomorphic, is now polymorphic" from "is
polymorphic, happened to see one shape first". Mitigations: a warm-up threshold before
feedback is trusted at all, sticky bits recording "this site was ever polymorphic" so the
history is not erasable, and re-profiling after a deopt rather than reusing the feedback
that already failed.

**3.** First, **compile-time economics**: an optimizing compile is expensive and you do not
want to spend it on code that will run a thousand times; a baseline JIT compiles almost
instantly with essentially no analysis and buys a large constant factor over the
interpreter. Second, and this is the one that is not about speed: the baseline tier
provides a **stable, well-specified frame layout and IC infrastructure to deoptimize
into**. Deopt has to land somewhere, and landing in a baseline frame with a known layout is
far simpler than reconstructing an interpreter's state. JSC's Baseline, V8's Sparkplug, and
HotSpot's C1 all serve this double duty, and the deopt-target role is why they survive even
as interpreters get faster.

### A9 — Materializing what the optimizer erased

**1.** You must reconstruct the **stack of logical frames** the language semantics says
exists — the inliner erased three physical frames, but the source-level program still has
three activations, and the exception handlers, stack traces, and debugger all depend on
them. For each frame: the locals, read out of machine registers and stack slots according
to the deopt map; any object that escape analysis **scalar-replaced**, which must be
genuinely allocated now and have its fields stored back and every reference to it fixed up;
any value the optimizer chose to keep unboxed, re-boxed into the representation the
interpreter expects; and the correct resume point — a *valid* bytecode index, not merely
some instruction. Re-running from the top is unavailable because the code has already had
**effects**: output written, stores performed, objects allocated that escaped. Deopt must
resume at a point, not restart from one.

**2.** Because the metadata must exist at *every* point a deopt can occur — every guard —
and must describe, at each such point, the full source-level state of every inlined frame in
terms of the current physical machine state. It therefore scales as (guards) × (inline
depth) × (live values). Worse, every optimization that moves, rematerializes, or deletes a
value must **update every deopt map that names it**. That is the real complexity: deopt maps
are a second consumer of every value in the IR, and they rot silently if they are a side
table. Which is exactly why optimizing compilers with deopt keep the deopt state as explicit
IR nodes participating in the dataflow — HotSpot's `JVMState` on safepoint nodes, V8's and
Graal's `FrameState` — rather than as annotations. Memory-wise, the OOP maps and debug info
attached to compiled code are a well-known nontrivial fraction of its footprint.

**3.** It forbids any transformation that makes a source-level state unrecoverable at a
deopt point. You cannot delete a computation whose result some deopt map names — you can
only *sink* it into the map as an instruction to recompute on the slow path. You cannot
reorder an observable effect across a guard. You cannot let a guard float above the effect
it was meant to protect. And structurally: values are kept alive by deopt maps, which
extends live ranges and consumes registers. "Deopt metadata causes register pressure in code
that never deopts" is real, routinely surprising, and the concrete answer to why deopt is
not free.

**Trap.** "Deopt is the slow path, so its cost doesn't matter." The *metadata* is charged to
the fast path's budget: it constrains code motion, extends live ranges, and occupies memory
proportional to the optimized code. Deopt is expensive whether or not it ever fires — which
is also why "just add one more speculation" is never a local decision.

### A10 — On-stack replacement, both directions

**1.** OSR-in solves the function that is **entered once and loops a million times**.
Ordinary tiering swaps implementations at call boundaries, so a function that is never
called again never gets its optimized version — the classic `main`-with-one-big-loop shape.
OSR swaps mid-execution, at a loop back-edge. The compilation is unusual because the entry
point is *in the middle of the CFG*: the compiler must treat the OSR entry as a block with
an externally supplied set of live-ins, matched to the incoming frame's layout, and the code
preceding the loop was not executed in this compilation, so much of the context an ordinary
compile would have (constants established before the loop, types narrowed by earlier
branches) is simply absent. OSR-compiled code is often measurably worse than the same
function compiled normally, for exactly this reason.

**2.** Going **out** lowers to a *canonical, fully specified* state — the interpreter's —
and every interpreter state is representable, so deopt is always possible in principle.
Going **in** must *raise* a canonical state into an optimizer-chosen representation, and the
optimizer may have chosen representations the incoming frame cannot supply: values it wanted
unboxed whose provenance is unrecorded, objects it scalar-replaced that already exist as
real objects in the incoming frame, invariants it established from code that did not run
this time. Lowering to a standard form is total; raising to a specialized form is partial.
That asymmetry is why OSR-in is restricted to designated points — loop headers with an
explicitly recorded live-in set — while deopt can be placed at any guard.

**3.** It runs that loop in the baseline or interpreter forever. That is the honest answer,
and the consequence is the widely-repeated advice to **put your benchmark inside a function
and call it many times** — advice which is real, is a direct artifact of tiering
granularity, and which people repeat without knowing why. The other mitigation is lowering
the compile threshold, which buys this case and costs compile time everywhere else.

### A11 — Deopt storms

**1.** Because the recompilation consumes the same feedback that produced the failing
speculation, so it produces the same code, which fails the same guard. That is an unbounded
loop paying a *full optimizing compile* plus a deopt on every iteration. The program is not
merely slow, it is spending its life in the compiler — saturating a background compiler
thread, or stalling the mutator if compilation is synchronous — and the pathology is
**stable**: nothing about it decays, so it will not fix itself given more time. This is the
difference between a slow program and a wedged one.

**2.** Minimum state: a record keyed by **the deopt point and the reason** — this bytecode
index failed this kind of speculation, this many times — plus a policy that consults it
before re-speculating. Where it must live is the part people get wrong: **outside the
compiled code**, attached to the method's persistent profile, because the compiled code is
precisely what gets discarded on deopt. HotSpot puts per-bci trap reasons and counts in the
method data object for exactly this. Store the record in the code and you have built an
amnesiac that rediscovers the same failure forever.

**3.** *HotSpot:* per-bci trap reason and count in the MDO; past a threshold it recompiles
*without that particular speculation*, and a per-method recompilation cutoff eventually
stops optimizing the method at all. *LuaJIT:* penalizes trace roots whose traces abort or
whose guards fail repeatedly, and eventually **blacklists** the bytecode as a trace entry
point so it stops trying. *CPython's specializing interpreter:* an adaptive backoff counter
per site — after a specialization fails, the site waits an exponentially growing number of
executions before re-attempting, and may simply remain generic. Shared shape: **remember the
failure at the site, back off on retry, and be willing to conclude permanently that this
site is not speculable, degrading to correct-but-generic.** A system that cannot give up is
a system that can wedge.

**Trap.** "Add a cap on recompiles per method." Necessary, insufficient. One method can have
several independently-failing sites, so a per-method cap either fires early and kills
optimization of the healthy sites, or fires late and lets the storm run. The state has to be
per-site *and* per-reason, or the policy cannot distinguish "this one guard is hopeless"
from "this method is hopeless".

### A12 — Inlining is not about call overhead

**1.** If call overhead were the point, the cheaper fix is a better calling convention — a
direct call with arguments in registers is a handful of cycles and near-perfectly predicted.
What inlining actually buys is **context**. The callee's body enters the caller's
optimization scope, so: known-constant arguments flow in and fold; the receiver's type is
known, so the callee's own ICs collapse to direct loads; escape analysis can see that an
object does not outlive the caller and scalar-replace it; loads across the call become
eliminable because the compiler now knows exactly what the callee writes; and branches on
arguments the caller knows disappear. Essentially the whole win is second-order, which is
why inlining a two-line accessor pays out far beyond the call's cost — it turns a field into
a register.

**2.** Because size is a proxy for *compile cost*, not for *value*. A tiny function called
once is worthless to inline; a medium function on the hottest path with a constant argument
is worth a great deal. Real systems combine: hotness of the **call site** (not of the
callee); whether arguments are constant; whether the receiver is monomorphic; whether
inlining would enable escape analysis on a specific allocation; and a **cumulative bytecode
budget for the whole compilation**, so depth is bounded by a global spend rather than a
fixed number of levels — which is the right shape, because a chain of trivial wrappers
should be inlinable to depth ten while one fat callee should not be inlinable at all.
HotSpot's split between "always inline if tiny" and "inline if hot and moderately sized" is
this distinction in its simplest form; Graal scores candidates by estimated benefit against
budget.

**3.** It looks like: a guard on the receiver's shape or class, the inlined body for that
case, and a fall-through to either a real call or a deopt when the guard fails. For two or
three targets you emit a short guard chain with each body inlined — HotSpot does bimorphic
inlining, and engines will inline a couple of PIC entries. It stops paying quickly, for
three compounding reasons: each body multiplies code size and dilutes the instruction cache;
each guard is a branch whose *misprediction* cost lands on the hot path; and at a genuinely
balanced polymorphic site the expected guard-chain cost approaches the dispatch you were
trying to avoid. Past two or three, a direct indirect call is better, and past that,
megamorphic dispatch is better still.

### A13 — When the cache is a net loss

**1.** Per miss you pay: the guard's compare, *plus its branch misprediction* — and a
data-dependent alternation is exactly the pattern predictors handle worst once the sequence
is not short and regular; plus the full lookup you would have paid anyway; plus the **cache
update**, a store into the feedback slot which dirties a cache line, and if that line is
shared across cores (shared code with shared feedback, or a global stub cache) generates
coherence traffic. Against no cache at all: a lookup, and nothing else — no guard, no
mispredict, no store, no dirty line. So a thrashing monomorphic site is *strictly worse*
than no cache, by more than people expect, and the store is the term everyone omits.

**2.** Because the cost is dominated by branch prediction and write traffic, and both are
functions of the **pattern**, not of the aggregate ratio. A 95/5 site has a strongly
predicted guard and updates rarely. A 50/50 alternating site mispredicts constantly and
stores on every other execution. Worse, two sites with identical hit-rate *fractions* can
differ by an order of magnitude if one's misses are clustered — cold for a while, then hot
and stable, a phase change — and the other's are interleaved. A scalar hit rate cannot
distinguish these. This is precisely why engines track a *state machine*
(uninitialized/monomorphic/polymorphic/megamorphic) rather than a ratio: the state encodes
the thing that matters, which is "does this site thrash".

**3.** (a) **Widen to a PIC** — the standard fix, correct for the two-shape case, costs a
longer guard chain and more per-site memory. (b) **Declare it megamorphic** and use the
shared stub cache — right when the shape set is genuinely large. (c) **Stop updating after
N misses** while keeping the existing entry — the cheap, targeted fix that removes the store
traffic without any new machinery, and the one most often missing. (d) **Fix the program**,
which is not a VM change: two shapes at one site almost always means two object layouts that
should have been one — two constructors with different field order, or an optional field
initialized conditionally. That fix has the largest payoff and the VM cannot make it for
you, which is exactly why engines expose shape and IC state to developer tooling instead of
only trying to compensate internally.

### A14 — Patching code versus writing data

**1.** Patched: the cached shape is an immediate encoded in the instruction, so the guard is
`cmp reg, imm` followed by a branch — zero loads beyond the object header you already
needed. The subsequent access can also use a *literal* offset, so the whole fast path is a
compare and a fixed-displacement load. Data side: the guard is `load feedback[i]; cmp; jne`
— one additional dependent load, plus the address computation from the feedback vector base
and the site index — and the access typically loads the offset and does an indexed load
rather than a displacement load. In a tight monomorphic loop the difference is small because
the feedback line stays hot; across a large program with thousands of lukewarm sites it is
real additional L1 and L2 pressure.

**2.** (a) **Concurrency** — another thread may be executing the exact instruction you are
rewriting, so patching demands atomic naturally-aligned writes, icache invalidation, and
frequently a safepoint; a data store is just a store. (b) **W^X and code signing** — iOS,
Apple Silicon, and hardened server configurations either forbid writable-executable memory
or require per-thread mode toggling, so each patch carries a syscall-ish cost that a store
does not. (c) **Sharing and lifecycle** — if the cache lives in the code, then code carrying
different feedback cannot be shared: not across closures of the same function, not across
isolates, not across processes via a mapped code cache; and discarding compiled code
discards everything you learned. Separating bytecode (shared, immutable) from feedback
(per-instantiation, mutable) buys all of that back, which is a large part of why V8 moved
that way.

**3.** When the code is already private to one compilation, already writable, and the read
sits in the innermost guard — that is, inside a JIT tier that owns its code lifecycle and can
afford a patching protocol. JSC's baseline repatching and HotSpot's compiled inline caches
(which patch a call site to a direct call and repatch to a megamorphic stub when it goes
polymorphic) both win there, because they remove a load from the hottest guard and the
machinery for code lifecycle already exists. And there is one case where patching is not an
optimization but a requirement: when the cached value must be an **immediate** so that a
downstream optimization can treat it as a compile-time constant. A value in a feedback slot
is a load; a value in the instruction is a constant, and constants are what unlock folding.

### A15 — The number says 94%

**1.** Because it is unweighted along two independent axes. (a) **By execution count** — a
site executed twice counts the same as one executed a billion times, so the number is
dominated by the long tail of cold sites and tells you nothing about the hot ones. You want
hits and misses weighted by dynamic execution, which typically reveals that a handful of
sites are the entire budget. (b) **By cost** — a miss whose slow path is a prototype-chain
walk ending in a dictionary probe costs orders of magnitude more than a miss whose slow path
is one array index, and a *hit* that saves a cheap lookup saves nothing. The quantity you
actually want is time saved: (misses avoided × slow-path cost) − (guard cost × executions) −
(update cost × misses) − memory. A percentage contains none of those terms.

**2.** A site inside a function called once at startup, iterating three elements, on a
monomorphic receiver. After the first iteration it hits every time: 100% hit rate, three
executions, two lookups saved. You paid for a feedback slot's memory, a cache line touched
during startup, and the guard's code. Multiply by the tens of thousands of such sites in a
real program and you have a measurable memory and *startup-time* regression with no
throughput return at all — which is exactly why V8 allocates feedback vectors lazily, only
after a function has been called enough times, and why "profile everything from the first
instruction" is a known startup anti-pattern.

**3.** A controlled A/B on the whole system: the same build shape with the cache enabled
versus disabled or capped, on a representative workload, with the methodology you would
demand of any other performance change — repeated runs, a quiet machine, an interval rather
than a point estimate. Supplement with *per-site dynamic counts* to locate where the win
comes from. If the A/B shows nothing, the IC does not work on that workload regardless of
its hit rate. Hit rate is a diagnostic that explains a result; it is never evidence that
there is one.

**Trap.** "Hit rate went from 80% to 95%, so we got faster." You may have made the remaining
misses more expensive by lengthening a guard chain, improved sites nobody executes, or paid
for the improvement in memory and startup time. Hit rate is an input to an explanation, not
an outcome, and reporting it as an outcome is the single most common way IC work gets
justified without ever being validated.

### A16 — Speculating on something other than a type

**1.** (a) **Element kind and array representation** — that `a` is a packed fast-elements
array of a uniform kind, so `a[i]` is a bounds-checked machine load rather than a generic
property lookup that might hit a hole, an index-named property, or a prototype. The guard is
the array's map, since element kind is encoded there, plus packedness. (b) **Bounds** — that
`i < a.length` proves the load is in range, so the per-iteration check can be hoisted out of
the loop by induction-variable analysis, guarded once in the preheader against the length,
with a deopt if it fails. Note the shape of that: the *guard moves*, which is legal only
because deopt can resume mid-loop — speculation and OSR-out are what make loop optimization
possible at all in a dynamic language. (c) **That `a.length` is loop-invariant**, i.e.
nothing in the body can resize `a`; the guard is whatever effect analysis plus map guards
establish that no reachable write reaches the array. (d, equally valid) **That `sum` remains
in the represented numeric range.**

**2.** The guard is close to free and is not a comparison at all: the machine's `add` sets an
overflow flag, so the check is a jump-on-overflow to a deopt stub. That is what makes it
different in kind — a shape guard summarizes an *identity* with a pointer and can be
invalidated by someone else's write; this is a speculation about a **value's range**, it
must be checked on the *result* rather than the input, and there is nothing to store and
nothing to invalidate. No cache, no write side, no dependency: it either fails or it does
not. Its interesting consequence is representational — succeeding lets the value live
unboxed in a machine register, which means the deopt path has to *create* a boxed object,
tying this back to materialization in A9.

**3.** Because in a dynamic language every reference to a top-level function goes through a
mutable binding, so without speculation every library call is an indirect call through a
cell and *nothing downstream is knowable* — no inlining, no constant folding, no escape
analysis across the call. Win the bet and the callee becomes a **compile-time constant**,
which unlocks the entire chain. It is cheap because the guard is on the *binding*, not on
the call: register one dependency on one cell, invalidated only by assignment to that cell,
and the fast-path check cost inside optimized code is **zero** — there is no check emitted at
all, exactly as in A4's dependency-list design and A7's validity cell. Same trick, third
appearance: move the cost to the rare write side and the read side becomes free.

**Trap.** "Speculation means guessing types." Types are merely the easiest thing to summarize
with a pointer compare. The highest-value speculations in a mature engine are about
**stability** — this binding has not been reassigned, this prototype has not been mutated,
this array has not been reshaped, this class has no second implementor — and those are
enforced by dependency rather than by comparison, which is precisely why their fast-path cost
is nothing at all.
