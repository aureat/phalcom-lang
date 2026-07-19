# Naming a call-stack activation that may already be dead

*(Theory-only draft. No implementation is assumed or consulted. Every claim below is meant to hold
for call-stack-based runtimes in general — stack machines, register VMs, tree-walkers with an
explicit frame stack, whatever. Where a specific system is named, it is named because it took a
real, attributable position in the design space, not because it is the intended answer.)*

## The shape of the problem

A closure that performs a **non-local return** does not merely capture data. An ordinary closure
captures *values* — the bindings that were live in its enclosing scope. A closure with non-local
return additionally captures a **destination**: "when I say return, make some *other*, already-running
activation — the one that was executing when I was created — be the one that produces a result and
unwinds." That destination is not a value you can copy and forget about. It is a claim about the
future: that a particular activation will still be there, still mid-flight, when the closure is
finally invoked.

Activations, in any implementation that isn't allocating a fresh heap object per call, live in a
structure that **recycles**. A stack of frames, a `Vec` of frame records, a fixed-size ring of
register windows — the entire performance argument for using such a structure instead of one heap
allocation per call is that slots get reused the moment they're free. That reuse is exactly what
makes naming an activation hard: the name a closure captured was, at the moment of capture, a
perfectly good name for *this* activation. Time passes. The activation returns, its slot is freed,
a new activation — unrelated, for a different call, possibly a different function entirely — is
placed in the same slot. The old name, dereferenced now, finds *something*. It finds an activation.
It is simply the wrong one, and nothing about the name's shape tells you that.

This is not a bug waiting to be fixed by "being more careful." It is a structural consequence of
choosing recyclable storage for something that first-class control values want to name durably.
Two bodies of theory converge on it from different directions, and it is worth being precise about
both, because most treatments blur them into one vague worry about "closures and stack frames."

### The ABA problem, imported

The pattern — read a name, have the underlying thing change *and change back* to something
bit-identical while you weren't looking, then have your stale name accepted as valid because
identity is being represented by nothing more than a bit pattern — is the **ABA problem**, and the
name comes from lock-free concurrent data structures, not from language runtimes.

The canonical setting is a lock-free stack built on compare-and-swap (CAS): a thread reads the head
pointer (value `A`), intends to pop it, and prepares a CAS that says "if head is still `A`, swap it
for `A`'s successor." Between the read and the CAS, another thread can pop `A`, push some other node
`B`, pop `B`, and then — because the allocator freed and reused `A`'s memory — push a *new* node that
happens to occupy the exact same address as the original `A`. Head is now, once again, bit-for-bit
`A`. The first thread's CAS succeeds. It believes nothing happened. Something happened: the world
was mutated out from under it and mutated back to a state indistinguishable, at the granularity the
CAS can see, from the state it started in. The name of the pattern literally narrates the value
sequence: A, then B, then A again. **[flagged, moderate confidence]** the term is usually traced to
practitioner folklore around compare-and-swap primitives on architectures like the IBM System/370 in
the 1970s–80s and is very widely attested in lock-free-data-structure literature from the 1990s
onward (Treiber's lock-free stack is the textbook example used to teach it); I do not have a single
citable first-use source and would not present one as fact.

The transplant to activation identity is exact, not merely analogical. A raw frame index (or a raw
pointer) is a bit pattern used as a proxy for "the same logical activation is still there," in
precisely the way a raw head pointer is a bit pattern used as a proxy for "the same logical node is
still there." Both proxies are cheap specifically *because* they don't ask any question beyond
equality-of-bits. Both fail for the identical reason: the storage under the name is reused faster
than the thing using the name can be told about it, and the recycled occupant can be — by sheer
bad luck or, in an adversarial or high-churn setting, quite reliably — indistinguishable from the
original at the only granularity being checked.

### The upward funarg problem, and why it has two halves that don't reduce to each other

The classical PL name for "closures that outlive the frame they were created in" is the **upward
funarg problem**, from the older Lisp/Algol-family vocabulary distinguishing *downward* funargs
(passing a function *into* a callee — trivially safe under stack discipline, since the callee's
activation is nested inside the caller's) from *upward* funargs (a function escaping *upward*, out
of its creating scope — into a return value, a stored field, a callback registered for later —
outliving the very activation that built it). Stack-discipline implementations that pop and reuse
frames on return cannot support upward funargs without doing something extra, because the thing
being escaped *from* is about to stop existing in the ordinary sense.

Almost every treatment of "the funarg problem" you'll find stops at the **data half**: a closure
closes over *variables*, those variables' storage was a stack frame, and if the frame is popped or
reused the variables' storage is gone or corrupted — unless something intervenes. The interventions
are well known and largely solved territory: heap-allocate the environment outright (SECD-machine
style, or any closure-conversion pass that heap-boxes every captured variable), or start variables on
the stack and **close** them — copy them out to independently-owned heap storage — at the moment
their frame is about to die (the Lua-style upvalue-closing discipline). Either way, the answer is
"give the *data* a lifetime independent of the frame," and once you've done that, reading a captured
variable through a closure is safe forever, no matter how long ago its frame died.

The **control half** is a different question and does not get solved by any of the above. Non-local
return is not "read a captured variable's current value." It is "resume execution *at a specific
point in a specific, already-existing computation*, unwinding everything between here and there."
That is a claim about **control state** — is there still a live computation, on some call stack
somewhere, that this jump would resume and unwind toward — not a claim about **data state** — does
this storage still hold a coherent value. The two can and do diverge completely: heap-allocating the
environment makes the *variables* immortal (an ordinary garbage-collected object, reachable for as
long as anything points at it, readable correctly forever) while doing absolutely nothing about
whether the *activation* — the specific, unique episode of "this function being called, from this
caller, at this point in the program's execution" — is still around to be returned to. An
environment can be perfectly, permanently alive while its owning activation is unrecoverably,
permanently gone. Closing over a variable and closing over a return destination are not the same
kind of capture, and a design that solves the first (which almost every closure-supporting language
does, because you cannot have closures at all otherwise) has told you nothing about whether it has
solved the second (which many languages don't even attempt — see the JS discussion below). This is
the precise reason "naming a call-stack activation that may already be dead" is its own problem, not
a restatement of ordinary closure-environment capture: it is the control half of upward-funarg, and
the control half has no data-half solution to borrow.

## The distinguishing program

The clean way to see that these are two different features wearing similar syntax is a language that
gives them two different keywords — or two different block forms — and lets you write both, side by
side, and watch them diverge only once the closure is asked to escape.

**Ruby** is the canonical pair, because `lambda` and `proc` differ in *exactly* — and only — this
one respect:

```ruby
def call_it_now(callable)
  result = callable.call(1)
  puts "callable returned #{result} to me"
  result
end

call_it_now(lambda { |x| return x * 10 })
# => "callable returned 10 to me"
# lambda's `return` is an ordinary function return: it produces a value
# to whoever called the lambda, and control comes right back to call_it_now.

call_it_now(proc { |x| return x * 10 })
# => the value 10 becomes call_it_now's OWN return value; the line
# "callable returned..." never prints. `return` inside a proc means
# "make the method that is *lexically* enclosing me — call_it_now itself
# in the program text where this proc literal appears, not wherever
# it's invoked from — return, right now, unwinding through the call
# to `callable.call`."
```

**Smalltalk** draws the same line more starkly, because there is no "plain return from the block" at
all — a block's *value* is just whatever its last expression evaluates to, full stop, no keyword
needed — and `^` inside a block is reserved entirely for the non-local case:

```smalltalk
Object subclass: #Example.

Example >> findFirstOver: threshold in: aCollection
    aCollection do: [:each |
        each > threshold ifTrue: [^each]].
    ^nil
```

`^each` does not make the `do:` block "return `each`" the way a lambda would — `do:` never asked its
argument block for a return value in the first place. `^` targets `findFirstOver:in:`, the method
whose text lexically contains the block, and makes *that method* return, unwinding through both the
`ifTrue:` block and the enclosing `do:` iteration in one motion. This is the same feature as Ruby's
`proc`, expressed as the *only* return mechanism a block has, rather than as an alternative to a
plain one.

Both examples above are the easy case: the closure is invoked *while its enclosing method is still
on the stack*. Now the escape variant — the one this entire document is about:

```ruby
def make_escaping_proc
  proc { |x| return x }   # `return` here means "return from make_escaping_proc"
end

p = make_escaping_proc     # make_escaping_proc has already returned by the time
                           # this line finishes executing — its activation is gone.
p.call(42)
# => LocalJumpError: unexpected return
```

```smalltalk
Example >> makeEscapingBlock
    ^[:x | ^x]

| b |
b := Example new makeEscapingBlock.  "makeEscapingBlock has already returned"
b value: 42.
"=> a BlockContext error, historically reported along the lines of
   'return from a dead method context' / BlockCannotReturn.
   [flagged, moderate-low confidence on the exact class/selector spelling —
   see the Smalltalk-80 section below.]"
```

In both cases the closure is invoked correctly, with the right arguments, by code that has every
right to call it — and it still fails, because the destination it was told to jump to no longer
exists as a live computation. That failure, and only that failure, is what the rest of this document
is about producing *deliberately and detectably* rather than as memory corruption.

## The design space: how do you name something that may already be gone?

Restated precisely: you need a **name** for an activation such that (1) creating the name is cheap
enough to do on every call that might be captured, (2) using the name to jump back is cheap enough
to do on every non-local return, and (3) using a name whose target has been recycled is
*detectable*, not silently wrong. Six branches answer this differently. The first three are the real
architectural fork and get walked in full; the rest earn a paragraph or a sentence each, in
proportion to how much of a genuinely separate answer they are rather than a variation.

### (a) Raw index / raw frame pointer

The tempting case first: this is *free*. One word. No comparison, no metadata, no second field to
keep in sync, no allocation. If activations live in an array, "the fourteenth slot" is a `usize` and
capturing "my enclosing activation" costs exactly the storage of one integer, and dereferencing it
on non-local return costs exactly one array index — no different from any other pointer-chasing
operation the runtime already does constantly. This is what a naive tree-walking interpreter, or a
VM whose author has not yet been bitten, reaches for first, and it is what C's `setjmp`/`longjmp`
idiom effectively gives you too: a `jmp_buf` is, at bottom, a saved machine/stack state you jump back
to by address, with the standard library offering no guarantee whatsoever about whether the frame
that called `setjmp` is still on the stack when you `longjmp` to it (using it after that frame has
returned is undefined behavior, not a checked error — C simply does not offer the check).

What it forecloses is exactly the ABA problem above, transplanted verbatim: a recycled slot accepts
a stale index with no way to tell the difference, because nothing distinguishes "the activation this
index used to mean" from "the activation this index means right now" other than the bits of the
index itself, which are identical in both cases by construction (that's what recycling *is*). This
branch is the one every other branch in this table exists to fix; it is also the one every actual
system that has thought about the problem, in any of the traditions surveyed below, has moved away
from once first-class escaping closures (or fibers, or coroutines, or anything else that lets a name
outlive its target) entered the picture.

### (b) Location + generation serial

The second-cheapest thing that actually works: keep the raw index (or pointer) — call it the
**location**, the "where to look" half, still fiber-local or array-local, still one word, still
O(1), still involving no search — but pair it with a second word, a **generation** counter that
lives *with the slot itself* (not with any individual occupant) and is bumped every time the slot is
reused. A name captured at creation time copies the slot's current generation into itself alongside
the location. Validating the name later is one comparison: does the token's stored generation match
the generation currently sitting in that slot? If yes, the slot has not been reused since this name
was minted, and the location half can be trusted. If no, something else occupies that address now,
and the name is stale — detectably, cheaply, in constant time, with no search over any list of live
activations.

This is exactly the **generational index** pattern (the name is not mine to invent — see the
comparative section below), and it buys back almost everything branch (a) offered — a `Copy`-able
pair of words, one comparison, no garbage-collector participation whatsoever — while adding the one
thing (a) was missing: a monotonically increasing witness that makes bit-pattern equality of the
location half actually mean something again. The cost is that **validity is checked, never
guaranteed by the type system**: nothing stops you from holding a stale token, passing it around,
storing it in a data structure, for arbitrarily long — the type checker (if there is one) sees a
plain pair of integers and has no opinion about whether they currently denote anything. The other
cost, examined in full below, is that the counter is a fixed-width integer and fixed-width integers
wrap.

### (c) Strong or weak reference to a heap-allocated activation

The other genuinely different answer, and the one worth taking as seriously as (b), is to stop
representing activations as slots in a reusable array at all and instead make each one a real,
individually heap-allocated object — the way Smalltalk-80's `BlockContext`/`MethodContext` pair works,
and the way CPython's frame objects work. Once an activation is an ordinary heap object, "is it still
live" stops being a bespoke problem this feature needs its own mechanism for, and becomes an instance
of a question the memory manager already answers for everything: is this object still reachable /
does it still represent something the runtime considers "on a stack." A **strong reference** to such
an object is a normal GC root or GC edge — holding one keeps the activation, and everything reachable
from it, alive for as long as the reference exists, which means liveness is not merely checkable but
can be made a *real, always-answerable query*: the object is either still marked "active on some
call stack" or it's not, and either way you have an actual object in hand to ask. A **weak**
reference to the same object drops the keep-alive obligation — the activation can still be collected
once nothing else needs it — at the cost of needing the collector to know about the reference as a
distinct kind of edge that gets cleared (or reports "gone") at collection time rather than traced
normally.

What this buys is real: activations become reifiable, inspectable, first-class values in their own
right (you can hold one, ask questions about it, in Smalltalk's case even resume it), and "is this
name still good" is answered by the same machinery that answers "is this object still alive" for
anything else in the heap — no separate generation-counter apparatus, no separate side table, no
separate invented mechanism at all. What it forecloses is precisely the thing that makes branch (b)
attractive in performance-sensitive designs: activations can no longer be `Copy`-able values living
by-value in a flat, reusable array — every single call now costs a heap allocation (and, in a traced
collector, ongoing scan/mark traffic for as long as any reference to that activation survives), and
every non-local return or ordinary return now interacts with reference-counting or tracing machinery
that a flat by-value frame array simply never touches. This is not a paragraph-sized cost; it is a
representation-level fork that touches every call in the program, which is exactly why it earns being
walked in full rather than dismissed — a designer choosing a `Vec`/array-of-frames representation for
speed has, by that choice alone, already ruled this branch out, not as a hypothetical foreclosure but
as a direct consequence of the representation decision.

### (d) Side liveness table

One paragraph, because it is a real point in the space but not a structurally distinct one from (b):
keep frames themselves completely dumb — no generation field, no metadata at all embedded in the
slot — and maintain a *separate* structure (a bitset, a hash set, a side array) recording which
locations are currently live. Validating a name becomes a lookup in that table instead of a compare
against an embedded field. This buys a frame representation untouched by the whole scheme — useful
if frames are laid out somewhere that genuinely cannot spare the extra word — at the cost of a second
source of truth that must be kept in lockstep with *every* place frames are created, recycled, or
bulk-truncated; a missed update site in the side table is a silent soundness hole (the table says
"live" when the slot has actually been reused, or vice versa) rather than the loud, structural
guarantee that an embedded, atomically-bumped generation field gives essentially for free by living
right next to the data whose lifecycle it tracks. It also trades a same-cache-line field compare for
a second-structure lookup — another allocation, another cache line, another place for drift.

### (e) Static escape prevention

One sentence, as weighted: prove at compile time — via linear types, regions, or a borrow-checker-style
lifetime discipline — that no reference to an activation can outlive the activation, which removes
the runtime check (and the runtime failure mode) entirely but requires a type system capable of
expressing and checking that proof, and tends to foreclose or heavily constrain first-class closures
that escape their creating scope in the more dynamic, unannotated ways scripting-style languages take
for granted.

### (f) Unforgeable capability / nonce

One sentence, as weighted, and it earns dismissal for answering a different question rather than for
being a bad answer to this one: a capability or nonce scheme guarantees a name cannot be
**counterfeited** by a party that was never legitimately given one — an authentication/forgery
property — which is orthogonal to whether a name that *was* legitimately handed out still points at
something alive; a capability system still needs one of (b), (c), or (d) underneath it if it also
wants to answer the liveness question, because unforgeability and liveness are independent axes
entirely.

## Branch (b) in mechanical detail

### The pair and where the serial comes from

A token in this scheme is two fixed-width integers: **location** (an index into a recyclable
structure — array offset, register-window number, whatever the storage is) and **generation** (a
counter associated with *that slot*, not with any particular occupant of it). The generation is
incremented exactly once per reuse-event of the slot — typically at the moment a new activation is
placed into it, sometimes at the moment the previous occupant is retired; the two are equivalent as
long as it happens exactly once per cycle and happens before the new occupant's name could be
captured. A token is minted by copying the slot's *current* generation value at the moment the name
is created (i.e., at the moment a closure captures "my enclosing activation"). Validating a token
later is: read the slot's current generation, compare to the token's stored generation; equal means
"nothing has reused this slot since I was minted," unequal means stale.

There are two genuinely different places the counter can be scoped, and this is not a detail —
where it's shared changes what the token can and cannot detect. A **per-slot** counter (each array
position keeps its own generation, bumped independently) is what generational-arena and slotmap-style
libraries typically use: cheap to increment (no contention with unrelated slots, small counter width
per slot), but each slot wraps *independently*, on its own schedule, based purely on how often that
one position gets recycled. A **global/epoch** counter (one counter shared across the whole
structure, bumped on *any* reuse anywhere in it) gives a single total order across every activation
the structure has ever held — any two tokens with different generations can be compared for
"happened before," not just "same or different" — at the cost of every reuse anywhere contending for
the same counter (a real cost under concurrency; a non-issue single-threaded).

**Predict, before reading on:** suppose a captured token doesn't merely get invoked *later in the
same pool* — suppose it escapes to an entirely different execution context that happens to use
structurally identical storage: a different worker's frame array, a different fiber's stack, a
freshly spawned unit of concurrency that numbers its slots starting from zero just like the one the
token came from. The location half is still, syntactically, a perfectly in-range slot number in
whatever pool it's now being checked against — very possibly one with a live activation sitting at
that exact index. What, if anything, stops the token from validating as good against that unrelated,
live-but-wrong activation?

The honest answer is: nothing does, *if* the generation counter's scope stops at the boundary the
token just crossed. A per-pool generation counter can only ever promise "unique within this pool" —
it has no way to know, and no way to be asked, about a second pool's numbering. A token that crosses
that boundary is being validated against a counter that was never in a position to detect the exact
collision it's now facing, because from that counter's point of view the incoming token's generation
is just some number, indistinguishable from a legitimately-minted-here one, drawn from a numbering
scheme it has no relationship to. Closing this hole requires either a single generation source
shared across every context a token could conceivably travel between (a genuinely global counter, not
merely a global-within-one-pool one), or a third component in the token identifying which context it
belongs to at all, checked before the location/generation pair is even considered. Which of those a
given design chooses — and whether "should tokens be able to cross that boundary in the first place"
is answered by forbidding the crossing rather than by widening the counter — is a live design
question in its own right, not something this branch answers by default; it is precisely the
asymmetry to interrogate in any concrete design, and it should never be assumed to be handled just
because the single-pool case is.

### What monotonicity buys, and what happens at exhaustion

Monotonicity — the counter only ever increases, values are never reused — is the entire reason a bit
pattern comparison can be trusted again: it restores, for one specific pair of words, the "has
anything happened since I looked" question that CAS-on-a-raw-pointer could not ask and ABA exploits.
As long as the counter genuinely never repeats a value for a given slot, generation equality really
does mean "same allocation episode," not merely "same bits by coincidence."

But the counter is a fixed-width integer, and fixed-width integers wrap. Honesty about exactly when
this matters requires being specific about width, because the answer is not uniform:

- **8-bit or 16-bit generation counters** — the kind found in tightly packed entity-ID schemes that
  squeeze index and generation into a single 32-bit or 64-bit word to save memory in high-churn
  systems — wrap after 256 or 65,536 reuses of a *single slot*, respectively. In a system that spawns
  and despawns entities every frame (a common ECS pattern), 256 reuses of one slot is not a
  theoretical concern; it is reachable within seconds. This width is a real, load-bearing engineering
  tradeoff, not a rounding error.
- **32-bit generation counters** wrapping requires roughly four billion reuses of one specific slot.
  For most call-stack-activation workloads (frame recycling driven by function-call rates) this is
  not reachable within a program's actual lifetime for the vast majority of programs — but a
  long-running server process, hammering one hot code path for months, is not obviously exempt
  either; treating 32 bits as "safe" is a judgment call about the workload, not a proof.
- **64-bit generation counters** push the reuse count required to wrap into territory that is not
  reachable at any plausible operation rate within any plausible process lifetime — the standard
  move once a design wants to stop thinking about this axis at all.

What real systems do about it, once they've decided the risk is non-negligible for their width: (1)
**nothing** — accept the (already vanishingly small, at sufficient width) residual risk, document the
width and the assumption, and move on; this is the majority answer once the width is 32 bits or
more, and essentially universal at 64. (2) **Tag bits / a reserved tombstone value** — reserve the
maximum generation value as a permanent marker meaning "this slot is retired, never to be reused
again," so a slot that reaches max generation stops participating in recycling instead of wrapping;
this converts a wraparound *unsoundness* into a (rare, bounded) resource *leak* — a defensible trade,
because a leak is at least observable and recoverable by restarting the process, whereas silent
aliasing is neither. (3) **Epochs** — a coarser-grained counter that isn't bumped per-slot-reuse but
per bulk event (the whole arena was cleared and rebuilt, the whole session rolled over), used when
slots are recycled in batches rather than individually, trading fine-grained detection for a counter
that increments far less often and therefore has far more effective headroom before it would need to
wrap at all.

### Failure atomicity: check before you touch anything

The validation compare must happen **before any mutation of machine state that the jump would
otherwise perform** — before any frame is popped, before any register or variable is clobbered,
before any side effect of "beginning the unwind" takes place. This ordering discipline has a name:
**failure atomicity**, more commonly discussed under the **strong exception guarantee**. The
taxonomy — **basic guarantee** (invariants hold and nothing leaks, but visible state may have
partially changed), **strong guarantee** (an operation either completes in full or has no visible
effect at all — commit-or-rollback), and **nothrow guarantee** (the operation is certified never to
fail) — is due to **David Abrahams**, developed in the mid-1990s in the context of exception safety
for the C++ Standard Template Library. **[flagged, moderate confidence]** I recall this as
originating in Abrahams' STL exception-safety writing / an early "Exception Safety in STL Containers"
paper and being widely propagated afterward via the Boost documentation, but I do not have the exact
original venue and date pinned down precisely enough to cite as fact.

Applied here: offering the strong guarantee for a non-local-return attempt means the validity check
must be the *first* thing that happens, before a single frame is popped or a single piece of unwind
state is touched. Suppose instead an implementation began unwinding optimistically — popping frames,
running cleanup, mutating whatever bookkeeping an unwind mutates — and only performed the generation
check partway through, or worse, only noticed the mismatch after the unwind had already progressed.
A stale token would then leave the machine in a state that is neither "as if the jump never
happened" nor "a coherent state reachable by any jump that actually completes" — some frames popped,
some cleanup run, and then a failure with no state the recovery code can trust. That is a **torn**
machine: an error raised out of a world that no longer corresponds to anything the program's own
semantics describe. Even an implementation of this shape that happens not to visibly crash today is
still wrong, in the sense that matters: whether the caught error is safe to recover from becomes a
property of *how much unwinding happened to occur before the mismatch was noticed* — an accident of
implementation order — rather than a designed, checkable invariant. That accidental-safety is the
textbook definition of failing to provide the strong guarantee, regardless of whether any particular
run of the program happens to survive it.

## Where the branches actually pull against each other

**Speed vs. trust.** The location half is the *entire* reason a non-local return can be O(1) — no
search over a list of live activations, no walk up a chain of contexts asking "are you still there."
The generation half is the *entire* reason that O(1) index can be believed rather than merely hoped.
Neither is optional and they pull in opposite directions on the same design: every bit spent on the
generation is a bit not spent shrinking the token, and every attempt to shrink the token by trusting
the location alone is a reversion to branch (a) and everything it forecloses. There is no version of
this pair that gets the speed without also paying for the trust.

**Detection vs. prevention.** A runtime check (raise on invalid generation, or on a failed side-table
lookup) is a *dynamic* answer: it costs a branch on every attempted non-local return, it needs an
error/recovery story for the failure case, but it needs no new static machinery and works uniformly
regardless of how dynamic the escape pattern is. Static escape prevention (branch (e)) removes the
runtime check and its failure mode *entirely* — no program that would trigger it can even compile —
at the cost of expressiveness: the technique is only as permissive as the type system's ability to
prove escape-freedom, and genuinely dynamic patterns (storing a block in a heterogeneous container,
handing it to an API that doesn't know at compile time whether or when it will be invoked, letting it
cross a scheduling boundary decided at runtime) tend to force either heavy annotation burden or
outright rejection of the pattern. Detection buys generality; prevention buys the absence of a
failure mode, for the subset of programs the type system can actually see through.

**Liveness-preserving vs. non-owning identity — the sharpest tension.** A reference to a
heap-resident activation (branch (c)) is something the collector is a *participant* in. A **strong**
reference is an ordinary GC root/edge: holding the token keeps the activation, and everything
reachable from it, alive for as long as the token itself is reachable — which is a genuine leak risk
in exactly the failure case this whole mechanism exists to handle: an escaped, never-invoked-again
closure that captured a strong reference to its dead home activation keeps that activation's entire
frame — every local, every further captured upvalue — pinned in memory for as long as the closure
itself survives, purely so that the *eventual* attempt to use it can report an error. A **weak**
reference avoids the leak but does not avoid collector involvement — something must still walk weak
references at collection time and clear or tombstone them when their referent goes away; that
clearing pass is real, implemented machinery (a weak table, a phantom-reference queue, whatever
vocabulary a given collector uses for it), and the collector has to know this category of reference
exists at all.

A (location, generation) token is invisible to the collector in a stronger sense than "weak" — it
isn't a reference of any kind that the collector's vocabulary contains. It is two integers, sitting
in whatever structure captured them, exactly as inert to a tracing or reference-counting collector as
a hash code or a timestamp would be. Nothing walks it at collection time; nothing clears it; nothing
needs to know it exists. The consequence cuts both ways, and it is important to hold both halves at
once: **zero collector overhead is attributable to this token** — no edge to trace, no weak-table
slot, no clearing pass, no participation of any kind — but also **zero keep-alive effect**, in either
direction. Holding the token does not delay the slot's reuse for even one allocation; the token
cannot "resurrect" or protect its target by existing. All it can ever do is be compared, after the
fact, against whatever currently occupies the slot it names, and report agreement or disagreement.
This is the precise sense in which such a token is not a GC edge: not merely "a weak one" but simply
not a kind of edge at all, which is exactly what makes it free to the collector and exactly what
means it can only ever detect staleness, never delay or prevent the event that caused it.

Two more terms belong here because the distinction they draw is easy to blur and load-bearing once
drawn: a **dangling** reference (a raw pointer into memory that has been freed and possibly reused)
is unsafe to dereference at all — there is no check available, only undefined behavior, because
nothing about the pointer's representation carries enough information to validate it. A **stale**
token is different in kind: the underlying storage is still perfectly safe to read (you can look at
whatever is in the slot right now, compare its generation, get a well-defined "no" instead of
undefined behavior). The entire value of the generational-token pattern is converting what would
otherwise be a dangling-pointer hazard into a stale-token detectable-failure — turning an unanswerable
question into an answerable one, even though the answer, in the failure case, is "no." A
**tombstone** — a slot permanently marked "never valid again" rather than merely "reused since you
last checked" — is a third, stronger state some designs reserve for exactly the wraparound mitigation
described above: not "this generation doesn't match," but "this location will never validate against
anything again, by policy," which is a stronger and more permanent statement than ordinary staleness.

## The comparative cases that earn their place

Four, not more, and each is here for a specific, named reason rather than because it is a famous
language.

### Smalltalk-80: the ancestor, and the reason its version of the problem is structurally different

Smalltalk-80's activation records are not slots in an array; they are ordinary objects —
`MethodContext` for a method activation, `BlockContext` for a block activation — living in the same
object memory as everything else, individually heap-allocated, individually garbage-collected,
inspectable and even (famously) resumable like any other object. `^` inside a block compiles to a
non-local return that targets the block's **home context**: the `MethodContext` of the method
lexically enclosing the block, recorded in the `BlockContext` at creation time. Sending `^` asks,
in effect, whether that home context is still part of some process's active call chain. If it is
not — the method has already returned, or the process that would have contained it is gone — the
attempt fails, historically reported as an error along the lines of `BlockContext>>badReturnError`
producing a "cannot return" condition. **[flagged, moderate-low confidence on the exact
class/selector spelling]** — different Smalltalk-80-descended dialects (Squeak, Pharo, VisualWorks)
may spell the exact class and message slightly differently, and I do not want to assert one precise
identifier as universally correct; the mechanism and its existence I hold with much higher
confidence than the exact name.

Here is the paragraph worth isolating: a language whose contexts are real, individually
heap-allocated objects has a **structurally different** version of this problem than one whose
activations are slots in a reused array, and the difference is not merely cosmetic. Because a
`MethodContext` is an ordinary object, "is my home context still live" is not a bespoke question this
one feature needs its own machinery to answer — it is an instance of a question the object memory
already knows how to answer for *everything*: is this object still part of the reachable, active
graph. Liveness, in this world, can be a *real query with a real answer at any time*, because there
is an actual, individually-identified object to ask — a returned context doesn't cease to exist the
instant control passes it; it is simply no longer on any process's active chain, and becomes ordinary
garbage, collected in the ordinary way, only once nothing references it anymore, exactly like any
other object nobody needs. Contrast an implementation where activations are slots in a reused array
or reused stack memory: there, "the slot" has no persisting identity as an object at all. Reuse is
not a state transition experienced by a continuing object — it is a literal overwrite of storage
that used to represent one thing and now, bit-for-bit, represents an unrelated thing, with nothing
in between. The generational-token trick (branch (b)) exists to *manufacture*, cheaply and without
per-call heap allocation, an approximation of the enduring identity that a real heap object gets for
free simply by being a real heap object. Smalltalk-80 pays for the real version of that identity with
per-activation heap allocation and full collector participation on every single call — contexts are
notoriously among the more expensive things to allocate in a naive Smalltalk-80 implementation — which
is exactly why implementations that wanted call/return to be cheap moved activation records back onto
flat, reusable storage, and in doing so had to reinvent, out of smaller and cheaper parts, an identity
guarantee that heap objects had simply never given up.

### Generational arenas / ECS entity IDs: the pattern with a name

This is the single highest-value comparison, by the filter's own test 3: it is the one place the
mechanism under discussion is not merely used but explicitly **named** in circulation — "generation,"
"generational index" — in vocabulary a reader can carry directly back into any other codebase they
encounter. Game engines' entity-component systems assign each entity a handle that packs an index and
a generation/version number together (documented explicitly this way in, e.g., Bevy's `Entity` type
and discussed this way in Unity ECS material), precisely because entities are spawned and despawned
at high, irregular rates and a stale handle held by, say, a UI element referencing a despawned
character must fail cleanly rather than silently pointing at whatever new entity now occupies that
slot. General-purpose Rust libraries — `generational-arena` and `slotmap` are the two most cited —
implement exactly this data structure as a reusable primitive, and both describe their purpose
explicitly in terms of preventing ABA on handles into a slab. **[flagged, low-moderate confidence]**
on priority-of-coinage for the term "generational index" itself — it reads as converged, independently
arrived-at vocabulary across game-engine and Rust-library communities rather than as traceable to one
first coiner, and I would not want to assert a single origin.

### Rust: the same question, answered twice, at two different phases

Rust is worth walking because it gives an unusually clean before/after: it answers *this exact
question* — does a name still denote something live — twice, using two completely different
mechanisms, at two different phases of the program's existence, and the two are the same problem
seen through different lenses rather than competitors.

Statically, the borrow checker's lifetime system is, at bottom, a compile-time proof obligation that
any reference tagged with a given lifetime does not survive past the scope that lifetime is tied to
— branch (e), static escape prevention, realized directly in a mainstream, widely-used language
rather than confined to research type theory. This is a large part of why the naive "closure that
captures a way to jump back to a specific caller" pattern isn't something Rust programs reach for the
way Ruby or Smalltalk programs do: the borrow checker either proves the escape can't happen (an early
return via `?`, a labeled `break` out of nested loops — both scoped tightly enough to the enclosing
function that the compiler can see the whole story) or refuses to compile the attempt.

Dynamically, wherever a Rust program *does* want a recyclable-slot handle whose target might already
be gone — which is precisely the frame-identity problem, generalized to "any handle into any
runtime-managed recyclable collection" — the ecosystem reaches for the generational-index crates
above: branch (b), not branch (e), because the borrow checker's static machinery simply cannot see
into a runtime-managed arena or ECS world whose entities are created and destroyed at rates and in
patterns the compiler has no way to know ahead of time (that would require something considerably
more powerful than the current type system — closer to dependent or session types). The two
mechanisms are not in tension with each other; they partition the same underlying question by whether
the escape pattern in question is or is not statically analyzable, and seeing them side by side is
the clearest available illustration that "static prevention" and "dynamic detection" are two answers
to one question, not two different questions.

### Ruby / JS: the bill for having, or not having, the feature at all

Ruby's `lambda`/`proc` distinction is walked above as the distinguishing program; what belongs here
is the *bill*, made concrete by JavaScript's total absence of the feature. A JavaScript function
passed as a callback — to `Array.prototype.forEach`, most famously — has no syntactic way to reach
back into its caller's control flow at all. `return` inside a `forEach` callback ends only that one
invocation of the callback; iteration continues regardless. This produces one of the most commonly
hit frustrations in the language, precisely because the syntax looks exactly like a `break`:

```js
arr.forEach(x => {
  if (x === target) return; // does NOT stop the loop — silently continues
});
```

There is no error here. That is the point, and it is worse than an error: the mistake fails
*silently*, continuing to iterate past the point the author clearly intended to stop, with nothing
signaling that anything went wrong. The idiomatic workarounds exist only because the language offers
no control-transfer mechanism to reach for instead: `Array.prototype.some` (whose callback's return
value is defined to stop iteration early on a truthy result — repurposing an ordinary, local return
value as a signal *to the iterating method*, not as a non-local return to anywhere), `every`
(short-circuiting on a falsy result, same trick inverted), or simply abandoning the higher-order
callback style for a plain `for...of` loop where a real `break` is available. Every one of these
workarounds is a way of asking the *iteration itself* to interpret an ordinary, local, value-only
return as a stop signal, because the language provides no mechanism for the callback to affect its
caller's control flow directly. That is the cost, made concrete, of not having this feature at all:
no `LocalJumpError`, no `BlockCannotReturn`, no dead-frame failure of any kind can ever occur — because
the capability that would produce one was never granted in the first place. It is the zero-cost
degenerate case of "detection vs. prevention": prevention not by proof, but by omission.

### Cuts, and why

**Java**: lambdas and anonymous classes have no non-local return capability at all — `return` inside
a lambda body returns from the lambda itself, full stop, with no syntax to reach further outward.
This is the same shape as JavaScript's omission (nothing to compare against the feature this document
is about), so it is cut to avoid a second, redundant "doesn't have it" entry once JS has already made
that case with a concrete, well-known bill attached.

**Go**: closures capturing a per-iteration loop variable — famously, for years, the same variable
across every iteration until the language changed the semantics — is Go's actual scar in this
territory, but it is a **data-half** bug (which iteration's *binding* a closure sees), not a
**control-half** activation-identity bug. Including it here would blur exactly the distinction this
document spent its opening section drawing carefully; it belongs with the data half, not this one.

**Lua**: upvalues that start on the stack and get **closed** — copied off to independently-owned heap
storage — the moment their owning frame is about to end, is Lua's answer to the data half, and a
well-trodden one. Lua's `error`/`pcall` mechanism is a general, unrelated exception-unwind facility,
not an activation-naming or identity mechanism. Both are adjacent to this document's subject, not on
it, so both are cut.

**C#**: closures over captured variables (data half, with its own well-documented loop-variable
capture history predating a language-semantics change), and no non-local-return-versus-ordinary-return
ambiguity of the Ruby/Smalltalk kind — a lambda's `return` in C# just returns from the lambda. Same
shape as the Java cut: nothing here to illustrate the control half specifically.

That is four survivors, as the filter's own expected count anticipates, each earning its place on a
different one of the four tests rather than by being independently notable.
