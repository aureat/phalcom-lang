# 06 — Memory and Garbage Collection

Who may point at what, and when it is safe to look. The through-line: *a collector is a
contract between the mutator and the runtime about what the machine's state means, and
every GC feature is a clause in that contract that someone has to honour on every
instruction.*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — Reference counting versus tracing, honestly

Swift and Objective-C use ARC and ship no tracing collector at all. Java, Go, and BEAM
trace and do no reference counting. Both camps are shipping serious systems.

1. State the throughput trade in terms of *what each algorithm's cost is proportional to*.
   The answer is not "RC is slower"; it is a statement about which quantity drives the bill.
2. "RC has no pauses" is the standard claim. Construct a program where ARC produces a pause
   comparable to a stop-the-world collection, and say what mechanism a language adds to
   mitigate it.
3. RC's cost per pointer operation gets dramatically worse under multithreading in a way
   tracing's does not. Explain the hardware mechanism, and name the two production
   mitigations.

### Q2 — Why CPython still runs a cycle collector

CPython reference-counts, and *also* runs a generational cycle detector.

1. Explain why the cycle collector is not optional in CPython specifically — name the
   ordinary language constructs that produce cycles whether the programmer wants them or
   not.
2. Deferred reference counting (Deutsch–Bobrow) and coalesced reference counting
   (Levanoni–Petrank) both cut RC's per-operation cost. Describe what each one stops
   counting, and what each one gives up in exchange.
3. CPython made `None`, `True`, and small integers *immortal* — their refcounts are never
   updated. That is a memory-model change, not a memory-saving one. Explain what problem it
   solves and why it became urgent recently.

### Q3 — Three ways to reclaim, three ways to fail

Mark-sweep, mark-compact, and copying (semispace) collection.

1. For each, state the cost proportional to *live* data and the cost proportional to *dead*
   data. That pair explains most of when each is chosen.
2. A long-running service reports `OutOfMemoryError` while a heap dump shows 40% of the heap
   free. Explain how, name which of the three collectors makes this possible, and name a
   real JVM collector where a specific object class makes it much more likely.
3. Copying collection halves usable heap and touches every live object. Given that, explain
   precisely why it is nonetheless the right algorithm for a nursery, and why the same
   argument does not extend to the old generation.

### Q4 — The barrier that pays for the nursery

A generational collector must find old→young pointers without scanning the old generation.

1. Compare **card marking** and **remembered sets** as the mechanism: what the barrier
   executes on each pointer store, what the collector must do at collection time, and where
   the precision/cost line falls.
2. HotSpot's card-marking barrier is *conditional* — it reads the card byte and only writes
   if it is clean. An unconditional store is one fewer instruction. Explain why the extra
   read is a performance win, and name the hardware effect.
3. Generational collection's premise is the weak generational hypothesis. Name a real
   workload where it is false, describe what happens to the collector, and say what tuning
   knob is the wrong response.

### Q5 — The object that vanished

A concurrent marker is running. The mutator does `black.field = white_obj;` and then
`grey_obj.field = null;`. The white object is now unreachable from anything the marker will
visit, but it is live.

1. State the two conditions that must both hold for an object to be lost, and state the
   strong and weak tri-colour invariants a barrier can maintain.
2. **SATB** (Yuasa, deletion barrier) and **incremental update** (Dijkstra, insertion
   barrier) each break one condition. Say which each breaks, what each records, and which
   one produces floating garbage.
3. Go 1.8 replaced its write barrier with a *hybrid* of both. What specific stop-the-world
   phase did that eliminate, and why did needing both barriers follow from Go's stack
   scanning strategy?

### Q6 — You can only stop where you can describe the machine

A thread must be stopped for the collector to scan its stack.

1. Explain why a thread cannot be stopped at an arbitrary instruction, in terms of what the
   collector needs to know and what a register holds mid-computation.
2. HotSpot polls for safepoints by *loading from a page that gets `mprotect`ed*. Explain why
   that is cheaper than a conditional branch on a flag, and what it costs when the safepoint
   actually fires.
3. A production JVM shows GC pauses of 1 ms in the logs, but the application observes 200 ms
   stalls. Name the quantity that is missing from the GC log, give a concrete code shape
   that produces it, and name the profiling artifact this same phenomenon causes.

### Q7 — Conservative roots and the door they close

Boehm's collector, and JavaScriptCore's scan of the machine stack, treat any stack word that
looks like a heap address as a reference.

1. Explain, mechanically, why conservative *stack* scanning forecloses moving collection.
   The answer is about writes, not reads.
2. Conservative scanning also causes false retention. Give the shape of a case where a
   *single* dead integer retains an unbounded amount of memory, and say why it is worse in a
   long-lived thread than a short one.
3. Blink's Oilpan is conservative on the stack and precise on the heap, and it still moves
   some objects. Explain how that is possible, and name the concept that reconciles the two.

### Q8 — The pointer that outlived its object

```c
PyObject *item = PyList_GetItem(list, 0);   /* borrowed reference */
PyObject_CallObject(user_callback, NULL);   /* may run arbitrary Python */
Py_ssize_t n = PyBytes_Size(item);          /* boom */
```

Every runtime with a native extension API has this bug in its FAQ.

1. Name the invariant that was violated, and be precise about which of the two possible
   disasters occurs in a non-moving collector versus a moving one.
2. V8 requires `Local<T>` inside a `HandleScope`; JNI has local references; Lua makes you
   keep values on the Lua stack; CRuby has `RB_GC_GUARD`. These are four different answers to
   the same question. Group them by mechanism and say what each demands of the extension
   author.
3. Now the harder version: a native routine allocates a result, then runs a *cleanup* path
   (an `ensure`/`finally` handler) before returning, holding the pending result in a local
   variable. Explain why this is the same bug wearing a disguise, and why an audit that
   greps for "pointer held across an allocating call" can miss it.

### Q9 — Interior and derived pointers

Go permits `&s.Field` and `&arr[i]`. Java does not. C# permits `ref` locals but historically
forbade `ref` fields on the heap.

1. What must a collector be able to do, given a bare address in the middle of an object,
   that it does not need for a base pointer? Name the data structure that answers it.
2. A JIT hoists `&arr[i]` out of a loop into a register, and then a GC happens mid-loop.
   Explain the *derived pointer* problem and what the compiler must have recorded to let the
   collector relocate `arr`.
3. C# allows managed pointers on the stack but restricted them from living on the heap for
   most of the language's life. Explain the reason — what a heap-resident `ref` would demand
   of the collector — and what `ref struct` is doing about it.

### Q10 — Read barriers buy relocation

ZGC and Shenandoah move objects while the application runs. Both use a read (load) barrier.
G1 does not, and must stop the world to evacuate.

1. Explain why concurrent *relocation* requires intercepting loads, when concurrent
   *marking* only needs write barriers. The argument turns on a counting fact about program
   behaviour and a correctness fact about stale references.
2. Shenandoah originally used a Brooks forwarding pointer — an extra word in every object
   header holding either self or the new location — and later switched to a load-reference
   barrier. Describe what each does on a read and what motivated the change.
3. ZGC puts metadata bits *in the pointer* and maps the same physical memory at several
   virtual addresses. Explain what the coloured bits encode, what "self-healing" means, and
   what this design costs that a G1-style collector does not pay.

### Q11 — Finalizers, and why everyone regrets them

Java deprecated `finalize` for removal. C# steers you to `IDisposable`. Go documents that
`SetFinalizer` may never run. Python's `__del__` has a chapter of caveats.

1. List the structural problems with running user code at collection time. Cover: the thread
   it runs on, resurrection, ordering among mutually-referencing finalizable objects, and the
   number of GC cycles an object survives.
2. Resurrection specifically: describe the state the collector is in when it must run a
   finalizer, why that makes the object's liveness a self-referential question, and what
   Java's "finalize runs at most once" rule is really protecting.
3. Every modern answer — `try-with-resources`/`AutoCloseable`, `using`/`IDisposable`,
   `with`/context managers, Rust's `Drop` — moves cleanup to a *lexical* mechanism. Say what
   that buys, and name the case it cannot cover, which is why `Cleaner`/`PhantomReference`
   still exist.

### Q12 — What an ephemeron solves

```java
WeakHashMap<Key, Value> map = new WeakHashMap<>();
map.put(k, new Value(k));   // the value holds its own key
```

The Javadoc warns about this. `WeakMap` in JavaScript and `ConditionalWeakTable` in .NET do
not have the problem.

1. Explain the leak precisely: trace which reference keeps which object alive, and say why
   the entry is never removed even though the program has dropped every reference to `k`.
2. State the ephemeron's marking rule, and explain why implementing it requires something a
   single marking pass does not provide.
3. Weak-key maps are the standard way to attach metadata to objects you do not own. Given
   (1), what is the discipline a programmer must follow with a non-ephemeron weak map, and
   what does it cost at every lookup?

### Q13 — Allocation speed is a garbage collection decision

`new` in Java is often a pointer bump and a bounds check. `malloc` is a size-class lookup
and a free-list pop. These are not competing implementations of the same idea.

1. Explain what a collector must guarantee for bump allocation to be possible, and therefore
   why a non-moving collector cannot offer it.
2. Describe TLABs, and explain what they eliminate that would otherwise dominate multi-
   threaded allocation cost. Then name the fragmentation they introduce.
3. Immix-style mark-region collectors allocate into recycled *lines* within blocks rather
   than bump-allocating a fresh region. Explain what that buys over both a free list and a
   pure copying nursery, and what it gives up.

### Q14 — Arenas next to a moving collector

You want region/arena allocation — allocate many objects, free the whole region at once —
inside a language with a tracing, moving collector.

1. Name the two properties an arena assumes about its objects that a moving generational
   collector actively violates.
2. BEAM gives every process its own heap, which is effectively an arena freed wholesale on
   process death. Name the two things this buys the collector, and the invariant that makes
   it sound.
3. You add an arena for a parser's AST nodes to cut GC pressure. AST nodes point at interned
   strings on the GC heap, and GC-heap objects point back at AST nodes. Enumerate what the
   runtime now has to do that it did not before.

### Q15 — GC and the foreign function interface

A native library needs a stable `char*` into a Java `byte[]` for the duration of a call.

1. Explain what the runtime must do, and name the two strategies a JVM can pick between for
   `GetPrimitiveArrayCritical`. Say what each does to the collector while the region is held.
2. Pinning objects in a compacting heap creates a specific pathology. Describe it, and name
   the .NET feature added to contain it.
3. A thread blocks for 30 seconds inside a native call. Explain why this does *not* prevent
   a GC, what the runtime records at the transition, and what the thread must do on the way
   back before it may touch a single object reference.

### Q16 — Allocation rate and live set pull in opposite directions

Two services. Service A allocates 2 GB/s with a 50 MB live set. Service B allocates
50 MB/s with an 8 GB live set.

1. For each, say which GC cost dominates and why — one of them is dominated by a quantity
   the other barely pays.
2. Give the tuning you would apply to each, and explain why applying A's tuning to B is
   ineffective and B's to A is wasteful. Include the general relationship between heap
   headroom and GC cost per byte reclaimed.
3. Go's `GOGC` sets the heap target as a multiple of the live set, and Go later added
   `GOMEMLIMIT`. Explain what failure mode `GOGC` alone cannot prevent, and connect it to
   the A/B distinction above.

### Q17 — Your object pool made it slower

A team pools request objects to reduce GC pressure. Allocation rate drops. p99 latency gets
worse.

1. Explain the mechanism. Cover where pooled objects end up in a generational heap, what
   happens on every write of a fresh object into a pooled one, and what that does to
   young-generation collection cost.
2. Go charges concurrent GC work to the allocating goroutine via mutator assists. Explain
   why that makes "time spent in GC" a misleading metric, and where the cost shows up
   instead.
3. State the general principle this illustrates about optimizing against a generational
   collector, and name the one case where pooling genuinely is the right answer.

---

## Answers

### A1 — Reference counting versus tracing, honestly

**1.** **Tracing's cost is proportional to the live set** — a collection visits every
reachable object and nothing else, so dead objects are free. **Reference counting's cost is
proportional to the number of pointer operations** the program performs, which is roughly
proportional to the *work* the program does, not to how much memory is live or dead. That is
the whole trade. A program with a huge live set and low mutation rate favours RC; a program
that allocates furiously and keeps almost nothing favours tracing, because tracing pays
nothing at all for the 99% that died. It also explains the pathological corners: tracing
degrades as the live set approaches the heap size (each collection reclaims little for the
same marking cost), and RC degrades when a program shuffles pointers a lot without
allocating.

**2.** Drop the head of a 10-million-node linked list, or the root of a large tree. The
release cascades: freeing the head decrements the next node to zero, which frees it,
decrementing the next, and so on — a single `release` becomes a **synchronous, unbounded,
recursive deallocation** at a point of the program's choosing but not the programmer's
awareness. It is a stop-the-world pause in everything but name, and it is *worse* than a
tracing pause in one respect: it is proportional to the dead set, which nothing bounds. The
standard mitigation is a **deferred free list / lazy release** — push the doomed subgraph
onto a queue and drain it incrementally, which is what several RC runtimes do, at the cost
of surrendering the "deterministic destruction" property that was RC's main selling point in
the first place.

**3.** The refcount is a **mutable word inside the object header**, so every retain/release
dirties that cache line. Two threads merely *reading* the same object — passing it around,
holding it in local variables — now ping-pong an exclusive cache line between cores, and the
operation must be an **atomic read-modify-write** with the associated memory ordering,
costing tens of cycles rather than one. Tracing does not have this because a read is a read;
nothing is written to shared objects during normal execution. Mitigations: (a)
**immortalization** — never count objects that will never die (`None`, `True`, small ints,
class objects), removing them from contention entirely; (b) **biased reference counting** —
give each object an owner thread with a non-atomic fast count and a separate atomic count
for everyone else, so the common single-threaded case pays no atomics. Both appear in
CPython's free-threaded work; Swift instead relies on the optimizer removing redundant
retain/release pairs, plus `unowned`/`borrowing` conventions that avoid counting altogether.

**Trap.** "Reference counting is simpler." It is simpler to *start*. A production RC system
ends up with deferred counts, biased counts, immortal objects, a cycle collector or a
`weak`/`unowned` discipline, and a deferred free list — at which point it is not simpler than
a generational tracer, it is differently complicated.

### A2 — Why CPython still runs a cycle collector

**1.** Because cycles are not an exotic data structure in CPython, they are the *default
shape of the runtime*. An instance holds `__dict__`; the values in it can point back. A class
holds its methods, and each function holds `__globals__`, which is the module dict, which
holds the class — every module-level class is in a cycle with its own methods on definition.
An exception holds a traceback, which holds frames, which hold the exception. A closure over
a variable holding the closure. Any doubly-linked list, any parent-pointer tree, any observer
registration. So without a cycle collector, ordinary correct Python code leaks steadily, and
the leak is unfixable by the programmer because they did not construct the cycle — the object
model did.

**2.** **Deferred RC** stops counting **references from the stack and registers** — the
highest-frequency, shortest-lived references, which is where most of the traffic is. In
exchange, a zero count no longer proves death (a stack reference might exist), so the
collector must periodically stop and scan the stacks to confirm which zero-count objects are
really dead. It trades per-operation cost for a periodic scanning phase, and it gives up
immediate reclamation — the thing RC was for.

**Coalesced RC** stops counting **intermediate values of a field**. Between two epochs, if a
field went `a → b → c → d`, only the initial `a` and the final `d` matter; the increments and
decrements for `b` and `c` cancel. It logs the field's original value on first write in an
epoch (a write barrier, so RC has acquired one of tracing's costs) and reconciles at epoch
end. It gives up precision in time — you learn about deaths only at epoch boundaries — and it
requires the log, so memory-write-heavy code pays barrier cost.

**3.** It removes the **cache-line contention and atomicity requirement** for the objects
that every thread touches constantly. `None` is referenced by essentially every operation in
every thread; if its refcount is a shared atomic counter, it becomes a hardware-level global
lock. Under the GIL this did not matter — only one thread ran Python at a time, so refcount
updates could be plain non-atomic increments. The moment the GIL is removed, every refcount
update must be atomic, and the singletons become the single hottest contended cache lines in
the process. Immortality means those objects' counts are simply never touched: the
increment/decrement path checks for the immortal marker and returns. It became urgent because
free-threaded CPython made a previously free operation cost a contended atomic.

### A3 — Three ways to reclaim, three ways to fail

**1.**
- **Mark-sweep**: marking is O(live); sweeping is O(heap) — it walks all of memory, dead
  included, to build free lists. So it pays for dead objects, but cheaply and linearly, and
  the sweep can be done lazily/incrementally during allocation.
- **Mark-compact**: marking O(live), then compaction O(live) again (move each live object and
  fix up every reference), plus a pass to compute forwarding addresses. Nothing proportional
  to dead data, but a high constant on live data — typically the slowest per-live-byte of the
  three.
- **Copying**: O(live) and *only* O(live). Dead objects are never touched, never swept,
  never even looked at. The entire from-space is reclaimed by resetting a pointer.

That pair explains the choices: where most objects die, you want the algorithm that charges
nothing for death (copying). Where most objects live, you want the one that charges least for
life (mark-sweep, non-moving, no per-object copy).

**2.** **Fragmentation.** A non-moving mark-sweep collector reclaims memory into a free list
of holes; if the free bytes are scattered as thousands of small gaps and the request needs a
large contiguous run, the allocation fails despite ample free memory. Only the non-moving
collector makes this possible — mark-compact and copying both produce a single contiguous
free region by construction. The JVM case: **G1's humongous objects**. An object larger than
half a region is allocated into a sequence of contiguous regions; a heap with humongous
allocations interleaved among ordinary regions can fail to find enough *adjacent* free
regions, producing humongous allocation failures and forcing full GCs, with plenty of free
heap on the report. Large `byte[]` buffers in a service with a modest region size is the
classic trigger.

**3.** Because in a nursery the **live fraction is tiny** — typically a few percent under the
generational hypothesis. Copying costs O(live), so if 3% survives, a nursery collection
touches 3% of the nursery and reclaims 97% by resetting a pointer. The "halves the heap"
objection also mostly evaporates: you do not need a full semispace, you need survivor space
proportional to the *expected survivors*, which is why HotSpot's young generation is Eden
plus two small survivor spaces rather than two equal halves. The argument does not extend to
the old generation because there the live fraction is near 1: copying the old generation
means copying essentially all of it, at which point O(live) is O(heap), you have paid a full
relocation and a full reference fixup, and you have given up half your address space to
reclaim a few percent. Hence the near-universal hybrid: copying young, mark-sweep (with
occasional or incremental compaction) old.

### A4 — The barrier that pays for the nursery

**1.** **Card marking**: the old generation is divided into fixed-size cards (512 bytes is
typical) with a byte-per-card table. The barrier on `obj.field = ref` is: compute
`card_table[addr >> 9]` and store a dirty marker. That is a shift, an add, and a store —
unconditional, no branch on whether the target is young, no data structure. At collection
time, the collector must **scan every dirty card in full**, examining every reference-typed
slot in those 512 bytes, most of which are irrelevant. So: cheapest possible barrier, most
expensive possible collection-time scan, precision limited to a card.

**Remembered sets**: the barrier filters (is the target young? is this a cross-region store?)
and, if so, records the *precise slot address* into a per-region set. At collection time the
collector iterates exactly the recorded slots — no scanning of irrelevant memory. So:
expensive barrier (a branch, possibly a hash-set insert, possibly a lock or a per-thread
buffer), cheap and precise collection. G1 uses both — a card table feeding per-region
remembered sets built by concurrent refinement threads — and remembered-set maintenance is
historically G1's largest non-pause overhead, which is exactly the price of the precision.

The line falls on **how much you collect at once**. If you collect the whole young
generation and it is small, a card scan is fine. If you collect a *subset* of regions chosen
independently (G1's whole premise), you need precise per-region incoming references, because
a card table tells you "somewhere in this 512 bytes" and you would have to scan the entire
old generation's dirty cards for every partial collection.

**2.** Because of **cache-line false sharing on the card table**. The card table is dense —
one byte per 512 heap bytes — so a 64-byte cache line covers 32 KB of heap. Many threads
writing to old-generation objects in nearby memory all write to the same card-table cache
line, and each unconditional store forces exclusive ownership, ping-ponging the line between
cores. Since a card is almost always *already dirty* (dirtying is sticky until the next
collection), the read-then-conditionally-write version turns almost all of those writes into
shared-state reads, which cores can hold concurrently. HotSpot exposes this as
`-XX:+UseCondCardMark` and it is a large win on multi-socket machines and a small loss
single-threaded — a textbook case of an extra instruction being faster because the machine's
cost model is about coherence traffic, not instruction count.

**3.** **Large caches and object pools** — anything that deliberately makes objects long-lived
— and, differently, **large batch pipelines** that build a big intermediate structure that
survives the young collection wholesale. What happens: survival rate in the nursery climbs,
so each young collection copies most of Eden into survivor space, then copies it *again* on
the next collection, then promotes it. You pay the copying cost repeatedly for objects that
were always going to be promoted, plus increased card/remembered-set traffic once they are
old and being written to. Symptom: rising young-GC pause times with unchanged allocation
rate. The **wrong response is to enlarge the young generation** — that makes each collection
copy an even larger surviving set, so pauses get worse, and the intuition "bigger young gen
means fewer collections" fails because the cost per collection is what is growing. The right
responses are to lower the tenuring threshold so doomed-to-survive objects get promoted
immediately instead of being copied several times, or to stop creating the long-lived
garbage.

### A5 — The object that vanished

**1.** Both must hold: **(a)** a black object (already scanned; the marker will not revisit
it) is made to point at a white object, and **(b)** every path from a grey object (scanned
but with unscanned children pending) to that white object is destroyed. Either alone is
harmless — a black→white pointer is fine if some grey object still reaches the white object,
and destroying grey paths is fine if nothing black points at it.

- **Strong tri-colour invariant**: no black object points to a white object.
- **Weak tri-colour invariant**: any white object pointed to by a black object is also
  reachable from some grey object (i.e. condition (b) never completes).

A barrier maintains one or the other; you do not need both.

**2.** **Incremental update / Dijkstra (insertion barrier)** breaks condition **(a)**: on
`black.field = white`, it records or shades the *new* target, pushing it grey. It maintains
the strong invariant. **SATB / Yuasa (deletion barrier)** breaks condition **(b)**: on
overwriting any reference, it records the *old* value, guaranteeing the marker still sees the
graph as it existed at the start of the cycle. It maintains the weak invariant. SATB is the
one that produces **floating garbage**: it marks objects that were reachable at snapshot time
even if the program dropped them milliseconds later, so they survive this cycle and are
collected in the next. G1 and Shenandoah use SATB; that floating garbage is the reason both
need heap headroom and can suffer allocation failure if the concurrent cycle does not finish
in time.

**3.** It eliminated the **stop-the-world stack re-scan** at the end of the mark phase. Go
scans goroutine stacks *without* write barriers on stack writes — stack slots are written
constantly and a barrier there would be ruinous — so a scanned (black) stack could acquire a
pointer to a white object with no barrier firing. The pre-1.8 fix was to re-scan all stacks
in a STW phase at mark termination, and that phase grew with the number of goroutines,
producing the multi-millisecond pauses Go was trying to eliminate. The hybrid barrier
(Yuasa-style deletion on the overwritten slot, plus Dijkstra-style shading of the new value)
makes the invariant hold for objects reachable from an already-scanned stack without needing
to look at that stack again — so a stack, once scanned, is permanently black. Needing both
follows directly from the decision to leave stacks barrier-free: neither barrier alone covers
the "black stack, no barrier" hole, but together they ensure any object a black stack can
reach is either already marked or was logged at the moment the heap reference to it changed.

**Trap.** "SATB is conservative, so it is the safe/simple choice." SATB's conservatism is not
free safety — it is retained garbage that must be paid for with heap headroom, and it is
precisely why a concurrent collector can fail *by running out of memory during a cycle* and
degrade to a stop-the-world full collection. Both barriers are correct; they fail in
different directions.

### A6 — You can only stop where you can describe the machine

**1.** Because the collector must be able to answer, for every word in every register and
every stack slot, **"is this a reference, and to what object?"** Mid-computation, a register
may hold an untagged integer, a raw address obtained by arithmetic on an object pointer (a
derived pointer with no base), a half-initialized object whose header is written but whose
fields are garbage, or a value in the middle of a multi-instruction sequence that
temporarily breaks the object model. The compiler emits **stack maps** (HotSpot's OopMaps)
describing the reference locations, but only at specific program points — emitting them at
every instruction would be enormous and would over-constrain the register allocator. So the
set of stoppable points is exactly the set of points with a stack map, and that set is a
compiler decision.

**2.** Because the poll is a single instruction — a load from a known page — with **no
branch**, so it costs almost nothing in the common case and, crucially, does not consume
branch predictor resources or create a control-flow join that inhibits optimization. When the
VM wants a safepoint, it `mprotect`s the page; the next poll traps with SIGSEGV, and the
signal handler parks the thread. The cost when it fires is a **page fault and a signal
delivery** — hundreds to thousands of cycles, far more than a branch would have cost — but it
fires once per safepoint request rather than millions of times per second. It is the standard
"make the common case free and the rare case expensive" trade, implemented with the MMU.

**3.** The missing quantity is **time-to-safepoint (TTSP)**: the interval between the VM
requesting a safepoint and the last thread actually reaching one. GC logs report the pause
from the moment all threads are stopped; the application experiences the stall from the
moment the *first* thread stopped and started waiting. A code shape that produces it: a long
**counted `int` loop** with no calls and no allocation. HotSpot historically omitted
safepoint polls from counted loops on the theory that they terminate quickly, so an
`for (int i = 0; i < Integer.MAX_VALUE; i++)` doing arithmetic is uninterruptible for its
whole duration. Also: a large array copy or fill inside a single intrinsic, and JNI critical
sections. The profiling artifact is **safepoint bias**: any profiler that samples stacks by
requesting a safepoint (the classic `jstack`-loop or `AsyncGetCallTrace`-less samplers) can
only ever observe threads *at safepoints*, so it systematically fails to attribute time to
the exact code that has no polls — the hot loops. That is why async-profiler, which samples
via signals and walks stacks outside safepoints, reports different and more truthful
profiles.

### A7 — Conservative roots and the door they close

**1.** Because a conservative scan produces a set of **maybe-references**, and moving an
object requires **rewriting every reference to it**. If a stack word is a genuine reference,
you must rewrite it; if it is an integer that coincidentally equals a heap address, rewriting
it corrupts the program's data silently. The collector cannot distinguish the two — that is
the definition of conservative — so it must not write. Reading is fine: treating a possible
reference as live merely retains something. Writing is unsound. Hence conservative
stack-scanned objects can be *retained* but never *relocated*, which forecloses compaction,
copying nurseries, and bump allocation for anything reachable that way.

**2.** A dead stack slot — a spilled register from an earlier computation, or a caller's
uninitialized local — holds a bit pattern that happens to fall inside the heap. The
collector retains the object at that address; that object is the head of a large graph (a
cache, a document tree, a list of buffers), and the entire graph is retained. One 8-byte
dead integer retains hundreds of megabytes. It is worse in a long-lived thread because the
stale slot is never overwritten: a deep call that ran once at startup leaves its residue in
stack memory that the thread's subsequent shallower calls never touch, so the false root
persists for the process's lifetime. In a short-lived thread the stack goes away and the
retention with it. This is why conservative collectors often *zero* dead stack regions or
limit scanning depth — mitigations, not fixes.

**3.** Because the restriction is per-object, not global: **objects that no conservative root
points at can still be moved**. Oilpan scans the stack conservatively, and every object thus
discovered is **pinned** for that collection; everything else is precisely traced through the
heap (where Oilpan has exact type information from its tracing methods) and may be compacted.
The reconciling concept is **pinning** — a per-object, per-collection "do not relocate" flag
derived from how the object was found. It costs you a fragmentation source (pinned objects
punch holes in the compacted region) and it makes compaction opportunistic rather than
guaranteed, which is why such collectors compact selected arenas rather than the whole heap.

**Trap.** "Conservative collection is unsound / can free live objects." It cannot — it errs
in the retaining direction. The unsoundness is entirely on the write side. Getting this
backwards means you cannot explain why Boehm is usable in production at all.

### A8 — The pointer that outlived its object

**1.** The invariant: **a raw object pointer held by native code across any operation that
can trigger a collection must be visible to the collector as a root.** `PyList_GetItem`
returns a borrowed reference — no refcount increment — so the list is the only thing keeping
the item alive. The callback runs arbitrary Python that can clear or reassign the list, the
item's refcount drops to zero, and the memory is freed. In a **non-moving** collector, the
disaster is **use-after-free**: `item` points at freed memory, which may be recycled into an
unrelated object, so `PyBytes_Size` reads a bogus header and returns nonsense or segfaults —
and, worse, may succeed and silently corrupt. In a **moving** collector the disaster is
different and arguably more insidious: the object may still be alive but **relocated**, so
`item` points at the *old* address, which now contains stale bytes or a forwarding record;
you get a wrong answer from a live object, and the corruption's blast radius is whatever you
write through it.

**2.** Two mechanism groups.

- **Explicit handle/root registration** — V8's `HandleScope`/`Local<T>`, JNI local
  references, Lua's stack. The native code registers its references in a structure the
  collector traces, and dereferences go through the handle (which the collector can update on
  relocation). This *supports moving collection* and demands that the author never hold a raw
  pointer at all — only handles — and that scope lifetimes be managed correctly (a loop
  creating handles without an inner scope exhausts the handle table, a classic JNI local
  reference leak).
- **Keep-alive assertions against a conservative scan** — CRuby's `RB_GC_GUARD`, which does
  not register anything; it exists to defeat the *compiler*, forcing the value to remain in a
  stack slot where CRuby's conservative stack scan will find it. This demands that the author
  reason about optimizer behaviour, which is a much worse contract, and it is only available
  because CRuby is non-moving.

The dividing question is: does the runtime find native roots by being *told*, or by *looking*?
Being told buys relocation and costs API ergonomics; looking is ergonomic and forecloses
relocation (A7).

**3.** It is the same bug because the cleanup path **can allocate and can therefore collect**
— running a user-defined `ensure` block executes arbitrary code; even a runtime-internal
cleanup usually allocates something — while the pending result is held only in a native local
variable that was never rooted. The result is live by the program's semantics and invisible
to the collector, so it is freed or moved, and the routine then returns a dangling reference
that will be used far away from here.

An audit that greps for "pointer held across an allocating call" misses it for two reasons.
First, the call in the source is not obviously allocating — it is `run_cleanup()` or
`unwind()`, whose allocation happens several frames down inside user code. Second, and more
fundamentally, the local is not held *across a call site* in the shape the pattern expects:
it is held across a **control-flow region** (the whole cleanup path, including exceptional
exits), and the dangerous path may be the one that only executes when something has gone
wrong. The correct predicate is not "is a pointer live across an allocating call" but
"**is any unrooted reference live across any point that can reach user code or the
allocator**", including error and unwind paths — which is a reachability question over the
control-flow graph, not a textual one.

**Trap.** "We audited the native code for GC safety." An audit is only as good as its
predicate, and the predicates people actually write are syntactic ("pointer, then a call,
then use") while the bug is semantic. The durable fixes are structural: make the raw pointer
type unavailable to extension authors (handles only), or make rooting automatic via a scoped
type whose destructor unroots — i.e. move the invariant into the type system rather than into
a checklist.

### A9 — Interior and derived pointers

**1.** Given an address in the middle of an object, the collector must be able to compute the
**base address and the type of the containing object** — otherwise it cannot find the header,
cannot know the object's size, cannot trace its fields, and cannot mark it. The data
structure is a **page/span table mapping address ranges to allocation metadata**: Go's heap
is divided into spans, each with a known object size class and start offset, so
`base = span.start + (addr - span.start) / span.elemsize * span.elemsize` recovers the object
in constant time. Non-moving collectors also use per-page bitmaps of object starts. The cost
is that the metadata must exist for the whole heap and be consulted on every interior
pointer, which is why languages that forbid interior pointers (Java) can use a simpler and
denser heap layout.

**2.** A **derived pointer** is `base + offset` where the base is a live object reference that
the collector may relocate. Once the JIT hoists `&arr[i]` into a register, that register holds
an address inside `arr` — but if the collector moves `arr`, it will find and update the
register/slot holding `arr` itself, and leave the derived register pointing into the *old*
location. If the compiler has kept only the derived pointer and dropped the base (a common
strength-reduction outcome — the loop no longer needs `arr` at all), the collector cannot even
compute the correction, because it does not know which object the derived pointer belongs to
or what the offset was. The compiler must therefore record, in the stack map at every
safepoint in that loop, a **derived-pointer entry pairing the derived slot with the slot
holding its base**, so the collector can compute `offset = derived - base_old` before moving
and `derived_new = base_new + offset` after. HotSpot's C2 does exactly this; the practical
consequences are that the base must be kept alive (inhibiting an optimization that would have
killed it) and that safepoint metadata grows.

**3.** A heap-resident `ref` would demand that the collector **trace and update interior
pointers found anywhere in the object graph**, not just on stacks. That means every heap
object might contain a pointer into the middle of another object, so relocation requires
base-recovery (per (1)) for arbitrary heap slots, generational barriers must handle interior
targets, and — the killer — the *lifetime* question becomes unanswerable in the existing model:
a `ref` to a local variable stored on the heap would outlive the stack frame it points into,
so the CLR would need to either forbid stack targets or box them. Restricting managed pointers
to the stack makes the lifetime question trivially answerable (a `ref` never outlives the frame
that made it) and confines interior-pointer handling to the stack-scanning code, where it
already exists.

`ref struct` is the type-system machinery that enforces "this value may only live on the
stack": it cannot be boxed, cannot be a field of a class, cannot be captured by a lambda,
cannot be an array element. C# 11's `ref` fields relaxed the rule *inside* that box — a
`ref struct` may now hold a `ref` field, precisely because `ref struct` already guarantees the
whole thing is stack-confined — accompanied by `scoped` and an extended set of lifetime rules
so the compiler can prove no `ref` escapes. The language solved the GC problem by proving the
GC never has to see it.

### A10 — Read barriers buy relocation

**1.** The counting fact: **loads vastly outnumber stores**, and more importantly, a mutator
can *use* a reference — dereference it, compare it, pass it — without ever writing it. The
correctness fact: after an object has been relocated, every reference to it that the mutator
still holds is **stale**, and a stale reference is not merely unhelpful, it is a pointer to
memory that no longer holds the object. A write barrier only fires when the mutator *stores*
a reference; it cannot intercept the mutator *loading* a stale reference out of a
not-yet-updated field and dereferencing it. So with write barriers alone, you must guarantee
no stale reference is ever loaded — which means all references must be updated before the
mutator runs again — which means stopping the world to do the update pass. Concurrent
marking is different: marking never changes the meaning of a reference, it only records
liveness, so intercepting the (rarer) mutations that could hide an object from the marker is
sufficient.

**2.** **Brooks pointer**: every object carries an extra header word that points to itself
normally, and to the new copy after relocation. Every read of an object's field goes
`obj = *(obj + fwd_offset)` first, then reads the field — an unconditional extra load and
dependent memory access on *every* field access, plus one extra word per object of memory
overhead. It is dead simple and always correct.

**Load-reference barrier**: instead of indirecting on every field access, the barrier fires
when a *reference is loaded from the heap*, checks whether the referent is in a region being
evacuated (a cheap test), and if so resolves and returns the new address — and typically
**writes the corrected reference back into the slot it came from**. The motivation for the
change: the Brooks pointer paid on every access forever, including for objects nobody is
moving and during phases when nothing is being evacuated, and the extra header word was pure
memory overhead across the entire heap. The load barrier concentrates the cost on reference
loads and lets the fast path be a predictable, mostly-not-taken check, while the write-back
means each stale reference is fixed at most once.

**3.** ZGC stores metadata **in unused high bits of the pointer** — which colour phase the
reference was last seen in (`marked0`, `marked1`, `remapped`), and finalizable status. The
same physical page is mapped at multiple virtual addresses differing only in those bits, so a
"coloured" pointer dereferences correctly without masking — the hardware does the work.
**Self-healing** means the load barrier, on encountering a reference whose colour is stale,
does the work (mark it, or look up its forwarding entry) *and then stores the corrected,
recoloured reference back into the memory location it was loaded from*, so the next load of
that slot takes the fast path. The heap converges to the correct colour through ordinary
program execution rather than through a dedicated fix-up pass.

What it costs that G1 does not pay: **a barrier on every reference load** — a test and a
mostly-not-taken branch on the hottest operation in the machine — which is a real,
measurable throughput tax; substantial **virtual address space** consumption from
multi-mapping (harmless on 64-bit, but it forecloses running on 32-bit and interacts with
address-space-based tooling); and the fact that reference loads are no longer plain loads,
which constrains the JIT and complicates every piece of code that wants to look at raw
memory. ZGC trades throughput for pause times that do not scale with heap size — the correct
trade for a large-heap latency-sensitive service, and the wrong one for a batch job.

### A11 — Finalizers, and why everyone regrets them

**1.**
- **Thread**: finalizers run on a runtime-owned thread with no relationship to the code that
  created the object, so they see no thread-locals, no security context, no transaction, and
  no lock the creator held. If one finalizer blocks — on I/O, on a lock, on a deadlock — the
  *entire queue* stops and every subsequent object's memory is never reclaimed. A single bad
  finalizer is an unbounded memory leak with no stack trace pointing at it.
- **Resurrection**: the finalizer receives `this`, which is a live reference, and may store it
  anywhere. The object comes back from the dead.
- **Ordering**: if A and B are both finalizable and reference each other, there is no order in
  which each can safely assume the other is still valid. Java refuses to define one and simply
  runs both, so a finalizer may observe an object whose finalizer has already run. Any
  finalizer that touches another finalizable object is therefore unsound in principle.
- **Cycles**: an object with a finalizer must survive the collection that discovered it was
  garbage (so the finalizer can run), then be re-determined garbage in a later collection.
  So finalizable objects cost **at least two GC cycles**, hold their entire reachable subgraph
  alive for that duration, and are guaranteed to be promoted out of the nursery — turning
  cheap young garbage into expensive old garbage.

**2.** The collector has proven the object unreachable and is about to reclaim it, and now
must run code that takes the object as an argument — which means it must **make it reachable
again**, because you cannot pass a reference to something that does not exist. So the
collector's own action falsifies the conclusion that justified the action. Liveness becomes
self-referential: the object is dead, therefore we run the finalizer, therefore it is alive.
Every implementation resolves this by *revivifying* the object and its subgraph, running the
finalizer, and then re-testing reachability afterwards.

Java's "at most once" rule protects **termination**. Without it, a finalizer that resurrects
`this` into a static field, which is later cleared, makes the object garbage again — and it
would be finalized again, and could resurrect again, forever. The runtime would have an
object it can never reclaim and a finalizer it must keep running. Marking the object
"finalized" after the first run makes the process monotone: every object is finalized at most
once, so the queue drains. The cost is that a resurrected-then-re-dropped object is collected
*without* cleanup, so the resource it guarded leaks.

**3.** Lexical cleanup buys **determinism and locality**: the cleanup runs at a known point,
on the calling thread, in the caller's context, in a defined order (reverse of acquisition,
which is exactly the order that makes nested resources safe), and — critically — its
*failure* is reportable to the code that cared, as an exception at a place with a stack
trace. It also decouples resource lifetime from memory lifetime, which is the real insight:
a file descriptor is not memory and there is no reason a memory reclamation algorithm should
be scheduling its release. Rust's `Drop` is the strongest form because ownership makes the
lexical point *provable* rather than conventional.

What it cannot cover: **the case where no lexical scope owns the resource** — an object whose
lifetime is genuinely determined by reachability, handed to code that does not know it holds
a resource, or shared among consumers with no join point. For those, you still need a
reachability-triggered hook, which is what `Cleaner` and `PhantomReference` provide — but
notice how they are designed: the cleanup action **must not reference the object**, so
resurrection is impossible by construction, and the whole thing degrades to "a callback fires
when a phantom reference is enqueued" rather than "user code runs with the corpse in hand".
It is the same capability with the two dangerous powers (resurrection, and running arbitrary
code on the object) removed. Treat it as a **safety net for a leaked `close()`**, not as the
mechanism.

**Trap.** "Use a finalizer as a backstop in case the caller forgets to close." That is
exactly the reasonable-sounding advice that produced the deprecation. The backstop delays
release indefinitely, so under a workload that closes correctly it does nothing, and under a
workload that leaks it releases file descriptors far too late to prevent EMFILE — while
imposing the two-cycle cost on every instance. Use `Cleaner` if you must, and log loudly when
it fires, because it firing means you have a bug.

### A12 — What an ephemeron solves

**1.** The map's internal entry holds a **weak** reference to `k` and a **strong** reference
to the `Value`. The `Value` holds a **strong** reference back to `k`. So the chain is:
map (strongly reachable) → entry → strong ref to Value → strong ref to k. The key is
therefore **strongly reachable**, the weak reference to it is never cleared, the entry is
never enqueued for removal, and the Value is never dropped — so the whole triple is immortal
for the map's lifetime. The program dropping its own reference to `k` changes nothing,
because the map itself is supplying the strong path. The weak reference is doing no work at
all: it points at something the very same data structure keeps alive.

**2.** The ephemeron rule: **the value is marked only if the key is proven reachable by some
path that does not go through this ephemeron's value.** Operationally, marking becomes a
**fixed-point computation**: mark the roots normally, treating ephemerons as opaque; then for
every ephemeron whose key is now marked, mark its value and continue tracing; repeat until a
full pass adds nothing new. Any ephemeron whose key is still unmarked at the fixed point has
a dead key, so both key and value are unreachable and the entry is dropped.

A single marking pass cannot do this because the decision "should I mark this value?" depends
on the *final* answer to "is the key reachable?", which is not known until marking completes —
and marking the value might itself mark another ephemeron's key, cascading. It is inherently
iterative, which is the implementation cost: extra passes over a worklist of pending
ephemerons, and a concurrent collector must handle ephemerons whose keys become marked after
its pass has moved on. That cost is why `WeakHashMap` does not do it and JS `WeakMap`,
.NET's `ConditionalWeakTable`, and Lua's `__mode="k"` tables (ephemeron-correct since 5.2)
all do.

**3.** The discipline is: **the value must not, transitively, reference the key** — and since
you often cannot know what a value transitively references, the practical rule is to wrap the
value in a `WeakReference` yourself, or store only data that provably cannot reach the key
(primitives, immutable copies of the relevant fields). Wrapping costs you a **second
dereference and a null check on every lookup** — `map.get(k).get()` can return `null` because
the value was collected independently of the key, which is a state an ephemeron map never
produces — plus an extra object per entry, plus the need to handle "entry present, value
gone" everywhere. That "entry present, value gone" state is the real cost: it is a new
failure mode you have introduced into every call site, and it exists purely because the
collector would not do the fixed-point computation for you.

**Trap.** "A weak map is a map with weak keys, so it can't leak." It can leak trivially, and
the leak is invisible to a heap dump reader who does not know to look for the value→key edge,
because everything *looks* correctly weak. The Javadoc's warning is not a footnote; it is the
single most common misuse of the class.

### A13 — Allocation speed is a garbage collection decision

**1.** Bump allocation requires a **large contiguous region of free memory** and the ability
to hand out the next `n` bytes by incrementing a pointer. That means free memory must be
consolidated, which means the collector must be able to **move live objects out of the way** —
compaction or copying. A non-moving collector reclaims memory *in place*, so free space is
whatever pattern of holes the dead objects happened to occupy; there is no contiguous region
to bump through, and the allocator must search a free list or size-class bin for a hole that
fits. So "allocation is a pointer bump" is not an allocator achievement, it is a **dividend
paid by the collector's willingness to relocate** — and any decision that forecloses moving
(conservative roots, unmanaged interior pointers, pinning for FFI) also forecloses fast
allocation, several layers away from where it was made.

**2.** A **TLAB** (thread-local allocation buffer) is a chunk of the nursery handed to one
thread; that thread bump-allocates within it with no synchronization, and only takes a lock
(or a CAS) when it needs a new chunk. It eliminates the **atomic compare-and-swap on the
shared bump pointer** that would otherwise be required on *every single allocation*, which
in a heavily allocating multithreaded program is a contended cache line touched millions of
times per second — the single hottest point of contention in the runtime. The fragmentation
it introduces is **TLAB waste**: when a thread's remaining TLAB space is too small for the
next object, that tail is abandoned (filled with a dummy object so the heap remains
parseable). With large objects or badly sized TLABs this can waste a meaningful fraction of
Eden, which is why the JVM adaptively sizes TLABs per thread based on observed allocation
rate.

**3.** Immix divides the heap into blocks and blocks into small **lines** (128 bytes is the
canonical figure). Marking records which lines contain live objects; allocation then bumps
through *runs of free lines* within partially-occupied blocks. Over a free list, it buys
**bump allocation and cache locality** — you allocate sequentially into contiguous lines
rather than jumping around a size-class free list, so freshly allocated objects that are used
together are near each other. Over a pure copying nursery, it buys **not having to copy**:
most reclamation is by recycling lines in place, with **opportunistic evacuation** of only the
most fragmented blocks, so you get most of compaction's benefit for a fraction of its copying
cost, without reserving a semispace.

What it gives up: **precision**. A line is retained if any object in it is live, so a single
survivor holds a whole line, producing bounded internal fragmentation that a copying collector
would have eliminated. It also needs both marking and evacuation machinery, and its
performance depends on a defragmentation policy with tuning parameters — it is not the
"one simple rule" that semispace copying is. MMTk uses it as a default for good reason, but
the complexity is real.

### A14 — Arenas next to a moving collector

**1.** (a) **Objects stay at a fixed address for the arena's lifetime** — arena users hold raw
interior pointers freely, and the whole point is that no relocation or per-object bookkeeping
occurs. A moving generational collector relocates young objects on every nursery collection.
(b) **Objects are reclaimed collectively by lifetime, not individually by reachability** — the
arena asserts "all of these die together, at this point", which is a *lifetime* claim the
collector has no way to verify and would happily contradict by keeping one object alive
because something still points at it (or by reclaiming most of them earlier). The two models
disagree about who decides when an object dies, and about whether an object's address is
stable.

**2.** It buys: (a) **collection is per-process and independent**, so a collection is bounded
by one process's small heap and pauses only that process — no stop-the-world, no global
coordination, which is what makes BEAM's soft-real-time latency claims achievable at millions
of processes; (b) **no write barrier is needed for cross-heap references, because there are
none** — the two hardest parts of concurrent GC (inter-region reference tracking and global
synchronization) simply do not arise, and a dead process's heap is freed **wholesale with no
tracing at all**, which is why process death is cheap enough to be used as an error-handling
mechanism.

The invariant that makes it sound: **no pointer ever crosses a heap boundary**, enforced by
copying messages on send and by the language having no shared mutable state. (The real system
qualifies this with an off-heap shared binary space for large binaries, which is
reference-counted precisely because it is the one place the invariant is relaxed — and which
is, unsurprisingly, the source of BEAM's most notorious memory-leak class.)

**3.** You have created a **bidirectional reference between a traced heap and an untraced
region**, so the runtime must now:
- **Treat arena contents as roots** for the GC heap: every pointer from an AST node to an
  interned string must be found and marked, so the arena needs either exact layout metadata
  the collector can trace, or a registered root list — and it must be scanned on *every*
  collection, so a large arena makes every GC more expensive, which is the opposite of the
  goal.
- **Keep GC-heap references to arena objects from being followed** into memory the collector
  does not manage, and prevent the collector from relocating or reclaiming based on them.
- **Prevent relocation from breaking arena→heap pointers**: if the collector moves an interned
  string, every arena-held pointer to it must be updated, which requires those pointers to be
  precisely known and writable — i.e. the arena must participate fully in relocation.
- **Order destruction**: freeing the arena while GC objects still point into it produces
  dangling references, so the arena's lifetime must be provably outside every GC object's, or
  those back-pointers must be weak.

At which point the arena is a specially-managed GC region with extra rules, not a bump
allocator you dropped in. That is the general lesson: an arena is cheap exactly to the extent
that it is *closed*, and it stops being cheap the moment references cross its boundary in
either direction.

### A15 — GC and the foreign function interface

**1.** The runtime must guarantee the address is **stable and the bytes are the real array**
for the call's duration. `GetPrimitiveArrayCritical` is specified to permit two
implementations: **pin the array in place**, which means the collector may not move it, and
in a compacting collector is typically implemented by **disabling GC entirely** (or at least
disabling collection of that region) while any critical section is open; or **copy the array**
into native memory and copy back on release, which leaves the collector free but costs O(n)
twice and breaks any expectation that the native code sees concurrent mutations. HotSpot
historically took the GC-disabling route, which is why the documentation reads like a warning
label: holding a critical region across blocking I/O stalls every thread in the process
waiting to allocate.

**2.** Pinned objects **cannot be moved, so they punch immovable holes in a heap the collector
wants to compact**. The compactor must slide live objects around them, which fragments the
result; if pinned objects are scattered and long-lived, the heap accumulates unusable gaps and
the collector's ability to produce contiguous free space degrades — you get the A3 failure
(OOM with free memory) plus rising GC times as compaction becomes less effective. .NET's
answer is the **Pinned Object Heap** (POH), a separate segment where you allocate objects that
you *know* will be pinned, so the immovable objects are segregated from the compactable heap
and the main heap can compact freely. It is the same architectural move as a large-object
heap: when a subset of objects has an incompatible constraint, give them their own region
rather than letting them constrain everything.

**3.** Because the runtime marks the thread as **"in native"** at the transition. A thread in
native code, by construction, is not touching object references through the runtime's model —
it holds only handles/local references (A8), which the collector knows how to find and update
independently. So the collector treats an in-native thread as **already at a safepoint**: it
does not need to stop it, because it can already fully describe its state. The GC proceeds
without waiting; that is exactly the design that keeps one blocked FFI call from stalling the
whole VM. (Go does the same thing with `entersyscall`/`exitsyscall`, and CPython's
`Py_BEGIN_ALLOW_THREADS` releasing the GIL is the same idea in a different currency.)

On the way back, the thread must **check whether a safepoint/GC is in progress and block if
so** before it may dereference any reference. This is non-negotiable: while it was away,
objects may have moved, so every handle it holds must be re-resolved through the handle table
rather than through any raw address it cached before leaving. A thread returning from native
that touched a stale raw pointer is the A8 bug in its most direct form, and it is why the
transition back is a real synchronization point with a cost, not just a state flag flip.

### A16 — Allocation rate and live set pull in opposite directions

**1.** **Service A** (2 GB/s, 50 MB live): dominated by **collection frequency**. It fills any
reasonable nursery many times per second, so it runs a great many young collections — but each
one is cheap, because with a 50 MB live set the surviving fraction of each nursery is tiny and
copying cost is O(live). Total GC cost ≈ (collections per second) × (small constant). Marking
the old generation is nearly free and rarely needed.

**Service B** (50 MB/s, 8 GB live): dominated by **the cost of tracing the live set**. It
allocates slowly enough that young collections are infrequent and irrelevant; the bill is the
concurrent (or full) marking of 8 GB of live objects, which must complete before the heap
fills, plus any relocation of that live data, plus remembered-set/card-scanning work
proportional to how much of that 8 GB gets written to. Young collection cost — the quantity
that is A's entire bill — is a rounding error for B.

**2.** **A**: enlarge the young generation. Every doubling roughly halves the collection
count, and because survivors are few, the cost per collection barely rises — so total GC time
falls close to linearly. Throughput-oriented collector, generous Eden, do not bother with a
concurrent old-gen collector.

**B**: enlarging the young generation does approximately nothing, because young collections
were never the cost. What B needs is **heap headroom for the old generation** (so concurrent
marking has time to finish before the heap fills, avoiding the degradation to a stop-the-world
full GC), a **concurrent marking** collector, and attention to the **write barrier and
remembered-set volume**, since mutation of a large old generation is what drives its
incremental cost. If pauses matter, B is the case for a relocating concurrent collector
(A10), where the pause is bounded by root-set size rather than live-set size.

Applying A's tuning to B is ineffective because it optimizes a term that is already
negligible. Applying B's tuning to A is wasteful because you buy a large heap and pay a
concurrent collector's throughput tax to reduce a cost A does not have.

The general relationship: **GC cost per byte reclaimed falls as the ratio of heap size to
live-set size grows.** A tracing collection costs roughly O(live) and reclaims roughly
(heap − live), so cost per reclaimed byte ≈ live / (heap − live). Doubling the heap with a
fixed live set more than halves the amortized cost. This is the single most important
quantitative fact in GC tuning, and it is why "just give it more memory" is so often the
correct answer, and why it stops working precisely when the live set is what is large.

**3.** `GOGC` expresses the heap target as a *ratio* — collect when the heap reaches
(1 + GOGC/100) × live. That is exactly right for A-shaped workloads and it self-tunes as the
live set changes. What it cannot prevent is **absolute memory exhaustion**: if the live set
grows — a cache filling, a leak, a burst of retained data — the target grows proportionally,
so Go happily targets a heap that exceeds the container's memory limit, and the process is
OOM-killed rather than collecting harder. That is the B-shaped failure: a ratio-based policy
is blind to an absolute ceiling. `GOMEMLIMIT` adds the missing absolute bound — as the heap
approaches the limit, the collector runs more aggressively regardless of the ratio, trading
CPU for staying alive. The pair is the honest answer, because the two services need different
policies and no single knob expresses both: `GOGC` controls the throughput/memory trade in the
normal regime, `GOMEMLIMIT` handles the regime where the live set itself is the problem.

### A17 — Your object pool made it slower

**1.** Pooled objects are, by design, **long-lived**: they survive many collections, get
promoted, and end up in the old generation. Meanwhile the request handler still allocates
fresh short-lived objects — strings, lists, parsed values — and stores them **into** the
pooled object. Every one of those stores is an **old→young pointer**, so every one trips the
write barrier and dirties a card / adds a remembered-set entry (A4).

The consequence at the next young collection: the collector must treat all those recorded
slots as roots, so it **scans dirty cards or remembered-set entries proportional to the
pool's write traffic**, and — worse — every young object referenced from the pool is *live by
definition*, so it survives, gets copied into survivor space, and eventually gets promoted.
You have converted a population of objects that would have died in Eden for free into a
population that is copied at least once and often promoted. Young collections get longer and
promotion rate goes up, which increases old-generation pressure, which eventually forces the
expensive old-generation work you were trying to avoid. Allocation rate went down; GC cost
went up. Fewer, more expensive collections with a worse tail is exactly the p99 story.

**2.** Because Go's collector performs much of its work **on the allocating goroutine**: when a
goroutine allocates during a GC cycle, it is charged an amount of marking work proportional to
its allocation, and it performs that work inline before its allocation is granted. So GC time
does not appear as "the collector ran for N ms"; it appears as **allocation calls taking
longer**, which shows up in your application's own latency measurements, distributed across
every allocating call site. A dashboard reading "GC CPU: 5%, pauses: 200 µs" can coexist with
a service whose p99 doubled, because the cost was billed to the mutator. The place to look is
allocation stall / assist time and the distribution of latency for allocation-heavy handlers,
not the pause histogram. The general form of this trap: **any collector that shifts work onto
the mutator (assists, load barriers, write barriers, lazy sweeping) makes "time in GC" a
systematically understated metric**, and the understatement grows precisely as the collector
gets better at avoiding pauses.

**3.** The principle: **a generational collector is already optimized for short-lived
garbage — allocation is a pointer bump and death is free — so "reduce allocations" is often
optimizing the cheap half while making the expensive half worse.** The costs a generational
collector actually charges for are *survival*, *promotion*, and *mutation of old objects*.
Anything that makes objects live longer or that increases old→young writes is working against
the collector even if it reduces the allocation counter. Measure survivor/promotion volume and
barrier traffic, not allocation count.

The case where pooling genuinely wins: when the resource being pooled is **not memory** — a
connection, a thread, a file descriptor, a mapped buffer, an OS-level object with an expensive
construction cost — or when the objects are **large and long-lived enough that they would be
promoted or humongous-allocated anyway** (big byte buffers for I/O, where the alternative is
repeatedly allocating multi-megabyte arrays that go straight to the old generation or a
large-object region and force expensive collections). Pool things whose cost is *not*
reclaimable by the collector cheaply. Do not pool small objects to save the collector work it
was already doing for free.

**Trap.** "We reduced allocations by 90%, so GC pressure is down 90%." Allocation rate and GC
cost are related only through survival. A workload that allocates 10× less but promotes 2×
more has made the collector's job harder, and the metric that would have shown it — promotion
rate, or survivor-space occupancy after each young collection — is one almost nobody has on a
dashboard.
