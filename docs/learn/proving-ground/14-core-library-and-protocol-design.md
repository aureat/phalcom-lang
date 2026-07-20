# 14 — Core Library and Protocol Design

The library decisions that are really language decisions. The through-line: *once a protocol
is in the core, it is part of the language whether or not it is in the grammar.*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — The bootstrap floor

You are bringing up a class-based language whose core library is written in the language
itself. The first line of that library is:

```
class Object { }
```

To execute it, the runtime must create a class object. Creating a class object means
allocating an instance of `Class`. `Class` is itself a class, so it is an instance of
something, and its methods live in a method dictionary, which is an object, whose class
must already exist.

1. Name the smallest set of things that must be built by the host — by hand, in Rust or C,
   writing fields directly — before a single line of the core library can be *evaluated*.
   Justify each entry by naming the specific circularity it breaks.
2. Smalltalk-80 does not solve this at startup; it ships an image. Python solves it by
   statically initialising `PyType_Type` and `PyBaseObject_Type` in C. Rust's `core` solves
   it with lang items. State what all three have in common as a *technique*.
3. A colleague proposes moving the floor lower: "let the host build only `Object` and
   `Class`, and write the metaclass wiring in the core library." Give the concrete reason
   this usually fails, in terms of what the compiler emits for `class Foo { }`.

### Q2 — Primitive or library

Two lists. Things you *could* write in the language: `Array>>map`, `String>>split`,
`Integer>>times:`. Things you *cannot*: `Object>>identityHash`, `Object>>class`,
`Array>>at:` on a raw slot, `Block>>value`.

1. Give the test that separates the two lists. It is not "is it fast enough" — state the
   structural criterion.
2. `Array>>map` is writable in the language, but nearly every real implementation ships it
   native anyway. Name the two costs of doing so, and note which of them is the one that
   actually bites (hint: see file 07).
3. Some operations sit in a third category: writable in the language, but only if the
   language exposes a capability it otherwise would not. Give an example and say what the
   capability leaks.

### Q3 — Internal versus external iteration

```
// internal
coll.each { |x| total = total + x }

// external
let c = coll.cursor
while c.hasNext { total = total + c.next }
```

1. Internal iteration lets the collection choose the traversal order and hold its own
   invariants across the whole loop. Name three concrete implementation freedoms that buys
   the collection, and one thing it makes structurally impossible for the *consumer*.
2. External iteration requires the collection's traversal state to be reified into a
   separate object. For a hash table this is easy; for a balanced tree it is not. Explain
   what the tree cursor has to materialise, and relate the cost to the same problem in file
   07's generators.
3. Ruby ships `each` (internal) and `Enumerator#next` (external) and implements the latter
   on top of the former. Name the machinery required, and the cost per element.

### Q4 — `break` in a block

`each` takes a block. The user writes:

```
items.each { |x|
    if x.isBad { break }
    process(x)
}
```

1. In a language where the block is an ordinary closure and `each` is an ordinary method,
   explain precisely what `break` must compile to. Name the mechanism, and say which frame
   is the target.
2. Now the same program with an external cursor and a real `while` loop. Why is `break`
   trivially expressible there and expensive in the block form? Be specific about what the
   runtime has to do at the `each` frame on the way out.
3. `continue` is easy in the block form and `break` is hard. Explain the asymmetry — it is
   a one-sentence answer once you see it — and say what that implies about which of the two
   a minimal language should ship first.

### Q5 — Four ways to end a loop

Four protocols for "give me the next element, or tell me there are none":

- **A** — `next()` returns a record `{value, done}` (JS).
- **B** — `next()` raises on exhaustion (Python's `StopIteration`).
- **C** — `hasNext()` then `next()` (Java).
- **D** — `next()` returns a private, unforgeable end sentinel; the loop compares against it.

1. Rank these by allocation per step and by *number of dynamic dispatches* per step. They
   are not the same ranking.
2. Exactly one of these forces the producer to compute an element before it is asked for.
   Identify it and give a case where that is observably wrong, not merely wasteful.
3. **D** is the cheapest and almost nobody exposes it as a public protocol. Give the two
   reasons, one about safety and one about composition.

### Q6 — The contract nobody enforces

```
class Point {
    var x
    hash  => x.hash
    ==(o) => o is Point and o.x == x
}

let p = Point(1)
let s = Set()
s.add(p)
p.x = 2
s.contains(p)   // ?
```

1. Say exactly what happens mechanically, and why the answer is "usually false but not
   reliably false".
2. Rust makes this a documented *logic error* rather than undefined behaviour; C++ makes
   an analogous violation for `std::unordered_set` undefined. Explain what Rust does
   structurally to earn "not UB", and what it costs.
3. A core library wants to make the failure impossible rather than documented. Name two
   designs that do so, and what each forbids.

### Q7 — Ordering, and the comparator that lies

```
sort(people, by: { |a, b| a.rank < b.rank ? -1 : 1 })
```

Note there is no zero case.

1. Name the property this comparator violates, and state the *minimum* set of properties a
   sort needs from a comparator. Do not say "it must be transitive" and stop.
2. `std::sort` with this comparator can read out of bounds and crash. Java's `Arrays.sort`
   throws `IllegalArgumentException: Comparison method violates its general contract!`.
   Rust's `sort_by` yields an unspecified order and may panic, but is guaranteed not to be
   unsafe. Explain what in each implementation produces its specific failure mode — this is
   a question about merge/partition internals, not about language philosophy.
3. `Comparable` (the object knows its own order) versus `Comparator` (the order is a
   separate value). Give the case that forces you to ship both, and name what goes wrong in
   a library that ships only the first.

### Q8 — Mutating while iterating

```
for x in list { if x.isDead { list.remove(x) } }
```

Three library policies: **fail-fast** (detect and raise), **snapshot** (iterate a copy or a
persistent version), **undefined** (document it as the user's problem).

1. Fail-fast is usually implemented with a modification counter compared on each step.
   Explain why Java documents `ConcurrentModificationException` as *best-effort* and must
   do so, and construct the case that slips through.
2. Argue that this choice is a **language** decision rather than a library one. Your
   argument must name something outside the collection class that the choice constrains.
3. Snapshot semantics are free in Clojure and expensive in Java. Explain why, and name the
   thing that is not free even in Clojure.

### Q9 — Persistent structures and the pair problem

Clojure's default map is a hash array mapped trie with 32-way branching; `assoc` returns a
new map sharing all untouched subtrees. Java ships `HashMap` and `Map.of`. Swift ships
`Array` with copy-on-write and no separate immutable type.

1. Explain what the branching factor is trading, and why 32 rather than 2. Your answer must
   mention both the depth and the copy cost of a single `assoc`.
2. Structural sharing has a cost that does not appear in asymptotic notation. Name it, and
   say why it makes a persistent vector lose badly to a flat array on a workload that is
   pure iteration.
3. Swift's copy-on-write gives value semantics with one mutable type. Name the two things
   it needs from the runtime to work, and the performance cliff a user can fall off without
   changing a line of their own code.

### Q10 — The string API minefield

```
s = "café"          // e followed by U+0301, decomposed
s.length            // ?
s[3]                // ?
s == "café"         // precomposed U+00E9
```

1. Python 3, JavaScript, Rust, Swift, and Go each answer `s.length` differently and each
   answer is defensible. Give each unit, and name the operation each language made cheap by
   choosing it.
2. Swift answers the `==` line `true` where Python and JavaScript answer `false`. State
   what Swift is doing, and give the concrete cost — one performance cost and one *semantic*
   cost that bites a library author.
3. A core library wants `s[i]` to be O(1) and Unicode-correct. Prove you cannot have both
   for grapheme clusters, then name the two escape hatches real libraries use.

### Q11 — The numeric tower

Four positions:

- Scheme: exact/inexact, integers promoting to rationals, a full tower.
- Python 3: arbitrary-precision `int`, `/` always produces `float`, `//` floors.
- JavaScript: one `Number` (double), plus `BigInt` — and `1n + 1` throws.
- Lua 5.3: `float` and `integer` subtypes of one type, `1 == 1.0`, `//` for integer division.

1. JavaScript could have made `1n + 1` coerce. Explain the specific correctness argument
   for throwing instead, using a concrete pair of values.
2. Python's `/` change in Python 3 was a breaking change to an operator. What class of bug
   did it fix, and what class did it create? Name both precisely.
3. Implicit promotion of `int` to `float` on overflow (rather than to bignum) is a design
   some languages take. Give the argument for it, then the argument that kills it — the
   killer is a single property of floating point.

### Q12 — What counts as false

- Lua and Ruby: only `nil`/`false` are false. `0` and `""` are true.
- Python: `0`, `0.0`, `""`, `[]`, `{}`, `None`, `False`, and anything whose `__bool__` or
  `__len__` says so.
- JavaScript: `false`, `0`, `-0`, `0n`, `""`, `null`, `undefined`, `NaN`.
- Smalltalk: nothing is false except `false`. `1 ifTrue: [...]` is a `doesNotUnderstand`.

1. Python's rule requires a *method call* to evaluate a condition. Name the two costs, one
   in the interpreter and one in the type system, and say what it lets a library author do
   that the Lua rule does not.
2. Smalltalk's `ifTrue:` is a message send to a Boolean, not a control-flow instruction.
   Given that, explain how a Smalltalk-lineage VM makes `if` fast anyway, and what it must
   check to stay correct.
3. The JavaScript list is the one everybody complains about. Isolate the *single* member
   that causes most real bugs, and say why it is worse than the others rather than merely
   longer.

### Q13 — The one-armed conditional

```
x = cond ifTrue: [ 42 ]     // cond is false. x = ?
```

1. Enumerate the possible answers a language can give, and say what each one costs at the
   use site. There are more than two.
2. If the answer is `nil`, the expression's type is "42 or nil" and the user must handle
   absence. If the answer is an `Option`, they must unwrap. Both are the same information.
   Explain what actually differs, in terms of what the compiler and the reader can check.
3. Now the block returns `nil` legitimately: `cond ifTrue: [ lookup(k) ]` where `lookup`
   can return `nil`. Show the ambiguity this creates in the `nil` design, name the standard
   fix, and say why the fix costs an allocation per evaluation unless you are careful.

### Q14 — `+` on strings

The core library is asked to define `"a" + "b"`.

1. Give the argument against, from the perspective of *error messages*. Construct the
   concrete expression whose diagnostic gets worse.
2. Give the argument against from the perspective of *inference*: what happens to a
   type-inferring compiler when `+` is overloaded across numbers and strings, and what
   machinery is needed to recover. Name a language that pays exactly this.
3. Java has `+` on `String` and nothing else overloaded. Name the operational trap that
   creates in a loop, and how the platform eventually dealt with it.

### Q15 — Three jobs, one method

```
print(x)             // for a user
log.debug("{}", x)   // for me, at 3am
"" + x               // for a URL I'm building
```

1. Rust splits this into `Display` and `Debug`; Python into `__str__` and `__repr__`; Ruby
   into `to_s` and `inspect`; Java into `toString` and nothing. Name the distinct
   *obligation* each of the two roles carries — one of them has a property that can be
   mechanically tested.
2. Argue that there is a third job, distinct from both, and name the failure that occurs
   when a library conflates it with the user-facing one.
3. Java's single `toString` is the cautionary case. Give the specific way it fails in
   production, and why "just be disciplined" does not work at the library boundary.

### Q16 — Raise or return

A core library's `Dictionary` needs a lookup. Ruby ships both `[]` (returns `nil`) and
`fetch` (raises). Python ships `[]` (raises) and `.get` (returns `None`). Rust ships `get`
returning `Option` and `[]` panicking. Go returns `(value, ok)`.

1. Give the rule for when a core library operation should raise rather than return a value
   representing absence. The rule cannot be "when it's exceptional" — make it operational.
2. A library that raises forces the caller to pay for the unwinding path even when absence
   is expected. Name the two costs, and say which one persists even if the exception is
   never thrown.
3. Go's `(value, ok)` and Rust's `Option` look equivalent. Name the thing Rust's version
   makes impossible that Go's does not, and the thing Go's version makes possible that
   Rust's does not.

### Q17 — The sealed kernel

A proposal: `Integer`, `Boolean`, `Nil`, `String`, `Block`, and `Class` may not be
subclassed, reopened, or have their methods redefined by user code.

1. Name three optimisations this unlocks that are unavailable — or available only
   speculatively — in an open-world design. For each, say what the open-world version has to
   do instead.
2. Smalltalk permits redefining `SmallInteger>>+`. Ruby permits reopening `Integer`. Both
   have production VMs that are nonetheless fast at arithmetic. Explain how, and name the
   safety mechanism.
3. State what sealing forecloses, with a real use case that becomes impossible, and say
   what a partial answer looks like.

### Q18 — Adding a method is a breaking change

Version 1.1 of your core library adds `Object>>flatten`.

1. In a language with open classes, explain the exact mechanism by which this breaks a
   working program that never mentioned your library's `flatten`. Name a real instance of
   this from Ruby or Objective-C.
2. Java hit a structurally similar wall and answered with default methods in interfaces.
   Explain what they fixed, and the new failure they introduced.
3. You must ship the method anyway. Name three mitigations, ranked by how much they cost
   the *user*, and say which one the design should have adopted before v1.

---

## Answers

### A1 — The bootstrap floor

**1.** The irreducible floor, and the circularity each entry breaks:

- **Raw allocation with an explicit header** — the ability to make an object whose class
  pointer is written after the fact, or as a raw integer. This breaks the "allocating
  requires a class, having a class requires allocating" loop by allowing a temporarily
  *invalid* object to exist. Everything else follows from this one.
- **`Object` and `Class`, mutually patched.** You allocate both with a null class pointer,
  then write each one's class pointer, then write `Object`'s superclass as nil and
  `Class`'s as `Object`. The graph is only legal after the last store, so it cannot be
  built by any operation that validates its inputs.
- **The method-dictionary representation** — whatever structure a class uses to hold
  methods. It is an object, so if it is a Dictionary written in the core library, you
  cannot install a method until you can install a method. The host builds this
  representation directly.
- **Symbol interning.** Method lookup keys on symbols. Symbols must be unique, so there is
  a table, and the table is consulted before any user code exists. It is also the reason
  the host — not the library — owns symbol identity.
- **Block/closure invocation and primitive dispatch.** The library's own methods are
  written as code, and something has to run them; the entry point to "call this compiled
  method" and "invoke primitive N" cannot itself be a method call, on pain of recursion.
- **A minimal `true`/`false`/`nil`** as distinguished values, because the first conditional
  in the library body needs them and they cannot be constructed by evaluating library code
  that itself contains a conditional.

Everything above is not "the fast path" — it is the set of things that *cannot be expressed*
in a language whose expression requires them.

**2.** All three ship a **pre-built graph rather than a construction procedure.** A
Smalltalk image is a memory snapshot of a valid object graph; CPython's static
`PyType_Type` is a C struct literal with the cycle written as an address; Rust's lang items
are the compiler knowing certain definitions by name so that `a + b` can lower to a call it
did not have to look up. In each case the answer to "how do you build the circular thing"
is *you don't — you serialise one that already exists, and the first one was built once by
a bootstrapper you no longer run.* Genuinely bootstrapping from nothing is a one-time
archaeological event in every one of these systems.

**3.** Because `class Foo { }` does not lower to a call to a library method — it lowers to
a VM operation the compiler emits, and that operation needs the metaclass shape already
fixed. Concretely: the compiler emits something like "create class named `Foo` with
superclass S", which must allocate two objects (the class and its metaclass), wire
`Foo class` as an instance of `Metaclass`, wire `Metaclass class` back into the tower, and
register a field layout. If that wiring lives in library code, then compiling the library's
*own* class definitions requires the wiring to already be installed by the library, which
is the same circularity one level up. The floor moves down only if you also give up
`class` as syntax and make class creation an ordinary message send with an explicit
receiver — which is exactly what Smalltalk does, and exactly why Smalltalk needs the image.

**Trap.** Saying "the bootstrap floor is whatever is too slow to write in the language."
Speed has nothing to do with it. The floor is the set of operations whose *use* is
presupposed by their own definition. A candidate who gives a performance answer here has
never brought up a runtime.

### A2 — Primitive or library

**1.** An operation must be primitive iff **its implementation in the language would
require the operation itself, or would require access to representation the language does
not expose.** `Object>>class` cannot be written because reading an object's header is not
an expressible operation; `Block>>value` cannot be written because writing it requires
calling something; `identityHash` cannot be written because identity is not a value the
language surfaces. `map` fails the test — you can write it with `at:` and `do:` — so it is a
performance choice, not a structural one. Note the second clause matters as much as the
first: `Array>>at:` is primitive not because indexing is hard but because "the i'th slot of
a raw allocation" is not a concept in the source language.

**2.** The two costs of writing `map` natively: (a) **duplication of semantics** — the
native version must reimplement whatever the language does about non-local return, argument
count checking, and errors thrown from the block, and it will get one of them subtly wrong;
(b) **a native frame between the caller and the user's block.** (b) is the one that bites.
It puts a C/Rust frame in the middle of a language-level call chain, which means non-local
return has to unwind through it, deep user recursion inside the block now consumes the
machine stack, coroutine suspension across it is unsound, and stack introspection shows a
hole. Every one of those is a real limitation traceable to a single decision about where
`map`'s loop lives.

**3.** `Object>>become:` (Smalltalk) — swapping the identity of two objects — is writable in
the language only if the language exposes raw pointer mutation or an object-table
indirection. Others in this family: `perform:` (needs a first-class handle to dispatch),
`instVarAt:` (needs reflective field access by index), `Object>>copy` (needs the layout).
The capability that leaks is **representation**: once you expose enough for the library to
implement these, you have exposed enough for user code to construct an object that violates
an invariant the VM depends on. That is the whole reason these end up as privileged
primitives instead of library code: the primitive is a *narrow* hole where the general
capability would be a wide one.

### A3 — Internal versus external iteration

**1.** Freedoms the collection gains: (a) **choose the traversal shape** — recursive descent
for a tree, chunked walk over an array-of-buckets for a hash table, segment-at-a-time for a
rope — without materialising the walk; (b) **hold locks, cursors, or borrow state for the
whole loop** and release them exactly once, including on the error path; (c) **specialise
the loop** — a native `each` over a contiguous array is a tight indexed loop with no
per-element protocol at all, and a chunked `each` can hoist bounds checks out. What it makes
impossible for the consumer is **interleaving two collections**: `zip` of two internally
iterated collections cannot be written without inverting one of them, because both want to
own the loop. That single limitation is why external iteration keeps getting reinvented.

**2.** The tree cursor must materialise the **traversal's control stack as data** — an
explicit path of ancestor nodes plus a child index at each level, which is O(depth) of heap
per cursor, updated on every step. The internal version gets that stack for free: it is the
interpreter's own call stack during the recursive walk. This is exactly file 07's point
about cursors versus generators, and it is the same problem: an external protocol demands
you reify the control flow that an internal protocol lets you express directly. A generator
sits between the two — it lets you *write* the recursive walk and have the machine reify it.

**3.** Ruby's `Enumerator#next` runs `each` inside a **Fiber** and yields out of it one
element at a time. The machinery is a full coroutine: a separate stack, a switch on every
`next`, and a suspended stack kept alive as long as the enumerator is. Cost per element is
two context switches — into the fiber to produce the value, back out to the consumer — plus
the memory of a parked stack per live enumerator. That is emphatically not free, and it is
why `each` is the fast path in Ruby and `next` is the compatibility path. It is also the
cleanest demonstration available that **external iteration is internal iteration plus a
coroutine**.

**Trap.** Claiming external iteration is "more general" and leaving it there. It is more
general for the *consumer* and strictly less general for the *producer*: any traversal whose
state is not a simple index costs you a hand-written reification, and any traversal that
needs to hold a resource across the loop now has a lifetime problem the internal version
did not have.

### A4 — `break` in a block

**1.** `break` compiles to a **non-local exit targeting the frame that lexically encloses
the block** — the method containing the `each` call — not the block's own frame and not
`each`'s frame. Mechanically it is the same machinery as Smalltalk's `^` in a block: the
block carries a reference to its home frame (or a home-frame identity token), and `break`
raises a typed unwind signal carrying that token, which propagates up until the frame
matching the token is reached. It must be a *signal*, not a return, because the block's
immediate caller is `each`, and `each` is not the destination.

**2.** With a real `while` loop, `break` is a **static jump**: the compiler knows the loop's
exit label at compile time, emits an unconditional branch, and the only runtime work is
possibly popping some stack slots. In the block form the destination is not statically known
from the block's body — the block could have been stored, passed around, and invoked from
anywhere — so the exit is a dynamic search up the frame chain. On the way out through
`each`'s frame the runtime must: unwind it, run any cleanup handlers it registered, and — if
`each` is a *native* method — get a native frame to abandon its loop cleanly, which means
every native combinator has to be written to propagate the signal rather than assume its
callee returned normally. That last part is the real expense: it is a tax on every native
method, paid so that one keyword works.

**3.** `continue` is expressible as `return` from the block. The block's own frame is
already the thing being exited, so it needs no non-local machinery at all — it is an
ordinary return, and `each` naturally proceeds to the next element. `break` needs to exit a
frame that is not the block's. The implication: **ship `continue` first**, because it costs
nothing and works on day one; `break` should wait until you have non-local return, and
should be built on the same mechanism rather than a second one. Two independent unwind
mechanisms in one runtime is how you get the bug where one of them silently skips the
other's cleanup handlers.

### A5 — Four ways to end a loop

**1.** Allocation per step: **A** allocates a record (unless the compiler escape-analyses it
away, which is real in optimising JS engines and absent in a simple interpreter); **B**
allocates only at the end, but the end allocation is an exception object with, potentially,
a stack trace — CPython optimises this by pre-allocating and caching `StopIteration`
precisely because it is per-loop, not per-step; **C** and **D** allocate nothing. Dynamic
dispatches per step: **C** is two (`hasNext` then `next`); **A**, **B**, and **D** are one.
So the rankings cross: **C** is allocation-optimal but dispatch-worst, and in an interpreter
where a send costs tens of nanoseconds and a nursery allocation costs a pointer bump, **C**
can lose to **A**. Anyone who ranks these on allocation alone has not measured an
interpreter.

**2.** **C.** `hasNext()` must know whether an element exists, which for any producer whose
elements are computed on demand means computing it — a line read from a file, a row from a
cursor, a value from a network stream. The observable wrongness, not the waste: if the
producer has side effects or can fail, `hasNext()` performs them or raises them, and it
does so at a point the consumer thinks is a pure query. Reading one line ahead in an
interactive stream *blocks*, so `while it.hasNext` hangs on a REPL-like source where the
element-at-a-time loop would not. This is why Java's `Scanner.hasNext` blocks and why
`BufferedReader.readLine() != null` is the idiom people actually use — the sentinel form,
i.e. **D**.

**3.** (a) **Safety**: the sentinel must be unforgeable. If user code can obtain it, it can
be stored in a collection, and then a legitimately stored element terminates every loop over
that collection — a bug with no local explanation. Keeping it unforgeable means it is not a
first-class value, which means it cannot flow through the ordinary value paths, which means
every place that touches it needs an audit. (b) **Composition**: a protocol whose
termination is a magic value cannot be wrapped generically. `map` over a **D** iterator has
to know to pass the sentinel through untransformed, and every combinator has to remember;
one that forgets calls the user's function on the sentinel. **A** and **B** wrap correctly
by construction because "done" is in the protocol's type, not in the value domain. So **D**
is the right *internal* representation and the wrong *public* one, and mature libraries use
it under a public **A**-shaped façade.

**Trap.** "Exceptions are slow, so **B** is obviously the worst." Python's loop protocol is
`StopIteration` and Python's `for` is not notably slow, because the exception fires *once per
loop*, not once per element, and the interpreter special-cases it — the `FOR_ITER` opcode
checks for exhaustion inline and never constructs the exception on the common path. The cost
you must actually account for is the one that scales with *n*, and a per-loop cost of any
size is amortised to nothing. Candidates who rank protocols by the scariest-sounding
mechanism rather than by cost-per-element get this backwards.

### A6 — The contract nobody enforces

**1.** `add` computed `hash` when `x` was 1 and filed `p` in the bucket for `hash(1)`.
`contains` computes `hash` now, gets `hash(2)`, probes that bucket, and does not find `p`.
It returns false. It is *not reliably* false because `hash(1)` and `hash(2)` may land in the
same bucket — trivially likely in a small table where the index is `hash % 8`. So you get an
object that is in the set, not findable, and occasionally findable, with the behaviour
changing when the table resizes. That non-determinism is the whole reason this bug class is
so expensive: it does not reproduce, and it changes under load.

**2.** Rust earns "not UB" by making the hash table's *internal* invariants independent of
the `Hash` implementation's honesty. Probing, resizing, and deletion are written so that any
sequence of hash values — including inconsistent ones — leaves the table structurally valid;
you get wrong answers, leaked entries, or a lookup miss, but never an out-of-bounds index.
C++'s `unordered_set` gives implementations licence to trust the hash, and some do, so a
lying hash can produce a bad bucket index. The cost of Rust's position is that the container
cannot use the hash to skip a bounds check or to assume an entry is in the bucket it
computed — a small but real per-probe tax, plus the API cost that mutating a key in place
requires `RefCell`/`Cell` or an unsafe block, which is the friction that makes the bug rare
in the first place.

**3.** (a) **Immutable keys** — the container copies or the type system forbids mutation
through the stored reference. Forbids: using a large mutable object as a key without a
clone, and any "update in place then re-file" pattern. (b) **Derive-only equality** —
`hash` and `==` are both generated from the same field list and cannot be written by hand,
as with a `derive`-only policy. Forbids: any semantic equality that is not structural —
case-insensitive strings, a `Point` where `(r, θ)` equals `(r, θ+2π)`, an object with a
cache field that should not participate. (b) is the stronger guarantee and the one nobody
ships unrestricted, because "equality is not structural" is a legitimate and common need.

**Trap.** Saying "the object is now in the wrong bucket, so lookup fails." It is in the
*right* bucket for the hash it had; the lookup probes a different one. And the confident
follow-on — "so rehashing the table would fix it" — is wrong twice: rehashing files it under
`hash(2)`, which fixes this instance, and does nothing for the next mutation, and there is no
event that would trigger the rehash anyway.

### A7 — Ordering, and the comparator that lies

**1.** It violates **antisymmetry**: it reports `a < b` and `b < a` for equal ranks, so no
two elements are ever equal. The minimum a comparison sort needs is a **strict weak
ordering**: irreflexive (`!(a < a)`), asymmetric (`a < b` implies `!(b < a)`), transitive,
and — the one everyone forgets — **transitivity of incomparability**: if neither `a < b` nor
`b < a`, and likewise for `b, c`, then likewise for `a, c`. That last property is what makes
"equivalent" an equivalence relation, and it is what a comparator on a floating-point field
containing `NaN` breaks even when it looks perfectly ordinary.

**2.** Three different internals, three different failures:

- **`std::sort`**: introsort's partition step advances two pointers with loops of the form
  `while (comp(*i, pivot)) ++i;`. The loop's only bound is the comparator eventually
  returning false — the pivot itself is the sentinel. A comparator that never reports
  equality never stops the scan, and the pointer runs off the partition and off the buffer.
  The crash is not a check that failed; it is the *absence* of a check that a valid
  comparator made unnecessary.
- **Java's TimSort**: it maintains a stack of pending runs with a size invariant, and the
  stack is allocated to a fixed depth computed from that invariant. An inconsistent
  comparator produces runs that violate it, the merge logic then indexes past the run stack
  or merges in the wrong order, and the explicit `IllegalArgumentException` is the
  defensive check added after exactly this failure was found in the wild. Java gets to
  detect it because the invariant is *explicit and checkable*; `std::sort`'s invariant is
  implicit in a pointer scan.
- **Rust's `sort_by`**: same family of algorithm, but every buffer access is bounds-checked
  and the merge logic is written to terminate on element count rather than on a sentinel, so
  the worst case is a wrong permutation or a panic from an explicit assert. The guarantee is
  bought with bounds checks the C++ version elides.

**3.** You need both the moment **one type has more than one useful order** — strings by
byte value versus by locale-aware collation, people by name versus by age, tasks by priority
then by insertion — or when you must order a type you do not own. A library with only
`Comparable` forces the sort order into the type definition, so a second order requires a
wrapper type per order, which then has to forward every other operation and breaks identity
and equality along the way. That is why Java shipped `Comparator` and why Rust's `sort_by`
exists next to `Ord`. The converse — only `Comparator` — is worse in a different way: every
sorted collection needs its comparator passed and *stored*, and two `TreeSet`s built with
equal-but-not-identical comparators are no longer interchangeable, which quietly turns the
comparator into part of the collection's type.

### A8 — Mutating while iterating

**1.** Best-effort because the modcount is an **unsynchronised heuristic that can alias**.
Concretely: a counter that increments on every structural modification and is compared to a
snapshot at each `next`. It slips through when the modifications cancel in count — a `remove`
followed by an `add` between two `next` calls leaves the count changed by two, and with a
counter of finite width or with a paired add/remove implemented as one bump, you get equal
counts and a silently wrong traversal. It also cannot detect modification of an element's
*contents*, only structural change, and it makes no promise at all across threads because
the counter is not volatile-read on every path. Java's documentation is explicit that the
exception is a bug-detection aid and must not be used for program logic, and that phrasing is
load-bearing: it is the API telling you it does not have an invariant, only a smoke alarm.

**2.** It is a language decision because it constrains **the iteration syntax itself**.
`for x in c { ... }` is a language construct that desugars to a protocol; if the collection
may raise mid-loop, then `for` is a construct that can throw from its *header* rather than
from its body, which every enclosing error-handling and cleanup construct must account for.
It also constrains internal iteration and blocks: if `each` holds a borrow or a lock for the
duration, then the language must either forbid re-entrant calls into the collection from the
block or define what happens, and that is a scoping rule, not a class method. And it
constrains optimisation: a compiler may only hoist a bounds check or cache a length out of a
loop if the language guarantees the collection cannot change underneath — so "undefined
behaviour on concurrent modification" is not laziness, it is a licence the optimiser
spends. Rust makes this maximally explicit by moving the whole question into the borrow
checker, which is precisely an admission that it belongs to the language.

**3.** Free in Clojure because the "snapshot" *is* the value — `assoc` returns a new map and
the iterator holds the old root, which remains a complete, valid, immutable structure
sharing everything untouched. Expensive in Java because a snapshot means a real copy: O(n)
time and O(n) space at iteration start, which is what `CopyOnWriteArrayList` does and why
it is only sane for read-mostly workloads. What is *not* free even in Clojure: **the
iteration sees stale data**, and staleness is a semantic cost that no amount of structural
sharing removes. A loop that removes dead items from a snapshot and writes back the result
loses any concurrent insert. Persistent structures convert a crash into a lost update, which
is better for uptime and worse for auditability.

### A9 — Persistent structures and the pair problem

**1.** Branching factor `b` trades **depth against per-update copy cost**. Depth is
log_b(n); a single `assoc` copies one node per level, so the copy cost is b·log_b(n) slots.
At b=2 you get depth 32 for a 4-billion-element map and 64 slots copied — deep pointer
chasing, many cache misses, poor. At b=32 the depth for any realistic n is 5-7 levels
(32^7 ≈ 3.4×10^10), and each copied node is 32 pointers = 256 bytes, which is four cache
lines and a `memcpy` the hardware is extremely good at. The insight is that **copying 32
contiguous pointers is cheaper than following 5 extra pointers**, because the former is
bandwidth and the latter is latency. 32 also makes the per-level index exactly 5 bits, so
the trie walk is a shift and a mask, and the bitmap of occupied slots fits in one 32-bit
word with a popcount for the compressed-node index — that is the "array mapped" half of
HAMT and it is why 32 specifically, not 16 or 64.

**2.** **Indirection and locality.** A persistent vector's elements are in leaf nodes
scattered across the heap; a flat array's are contiguous. Asymptotically both iterate in
O(n), but the flat array streams at prefetcher speed with one cache miss per line, while the
trie pays a pointer dereference per level for each new leaf and touches interior nodes that
carry no data. On a pure-iteration workload — sum a million ints — the constant factor
difference is not small, and no complexity argument will show it. Clojure's answer is
chunked sequences and transients, which is a tacit admission: the library ships an escape
hatch back to mutation for exactly the workloads where sharing does not pay.

**3.** Copy-on-write needs (a) **a reliable uniqueness test** — a reference count the
runtime maintains, checked at every mutation, so `isKnownUniquelyReferenced` can decide
whether to copy; and (b) **value-semantic assignment**, meaning the compiler treats
assignment as a potential retain and the optimiser must not reorder around it. The cliff:
holding a second reference — passing the array to a closure that captures it, storing it in a
struct, taking a slice — makes the next mutation copy the whole thing. So a loop that
appends to an array inside a function that also captured that array becomes O(n²) with no
visible change in the user's code, and the fix is a discipline about aliasing that the type
system does not enforce and the profiler reports as "memmove". This is the exact
performance failure that Rust's borrow checker turns into a compile error and Swift turns
into a silent slowdown.

**Trap.** "Persistent structures are O(log n), which is basically O(1) for real n." True and
irrelevant. The cost that matters is the cache behaviour of the constant, and the honest
comparison against a mutable array on the workloads people actually run is often 2-10×, not
"basically the same". Anyone quoting the log32 argument as a defence has not iterated one in
a hot loop.

### A10 — The string API minefield

**1.**
- **Python 3** — code points. `len` is 5. Made **code-point indexing O(1)** cheap by
  choosing a fixed-width representation per string (latin-1, UCS-2, or UCS-4 depending on
  the maximum code point), so a single non-BMP character quadruples the string's memory.
- **JavaScript** — UTF-16 code units. `len` is 5 here, but 2 for a single emoji. Made
  **compatibility with the 1995 decision** cheap; the observable consequence is that
  `"😀".length === 2` and `split("")` can produce lone surrogates.
- **Rust** — bytes. `s.len()` is 6 (UTF-8: `caf` = 3, `e` = 1, U+0301 = 2). Made **slicing
  and byte-level I/O free** — a `&str` is a pointer and a length into existing bytes with no
  copy — at the price of no integer indexing at all and a panic on a non-boundary slice.
- **Swift** — extended grapheme clusters. `count` is 4. Made **"what the user calls a
  character" correct** at the price of O(n) `count` and an opaque `String.Index` you cannot
  do arithmetic on.
- **Go** — bytes for `len`, runes for `range`. `len` is 6. Made **strings-as-byte-slices**
  free, with the explicit design position that indexing a string is a byte operation and
  anyone who wants characters must say so.

**2.** Swift compares strings by **canonical equivalence** — it normalises (in effect, NFC)
during comparison, so the decomposed and precomposed spellings of `é` are `==`. The
performance cost: equality is no longer a `memcmp`; it is a normalising walk, which also
means hashing must normalise, which means every dictionary insert of a string pays it. The
semantic cost for a library author is worse: **`==` no longer implies identical bytes**, so
two equal strings can serialise differently, produce different file names, different HTTP
signatures, and different database keys. Any code that round-trips a string through a
byte-oriented system and compares the result has a bug that only appears for non-ASCII
input from one particular platform — historically, filenames from macOS, which stored them
decomposed.

**3.** A grapheme cluster is defined by a *segmentation algorithm* over a variable-length
sequence — the boundary between cluster `i` and `i+1` depends on the properties of the
characters at that boundary, including arbitrarily long sequences of combining marks, ZWJ
emoji sequences, and regional indicator pairs. So the byte offset of cluster `i` is not a
function of `i` alone; it is a function of every code point before it. Any O(1) `s[i]`
therefore requires a precomputed index, which is O(n) space and must be invalidated on
mutation — so you have not achieved O(1) indexing, you have moved the O(n) to construction.
The two escape hatches: (a) **redefine the unit** — index by code unit or byte and let the
user opt into grapheme iteration, which is Python/JS/Rust/Go; (b) **change the index type**
— make the index an opaque cursor produced only by traversal, so `s[i]` for an arbitrary
integer is not expressible, which is Swift. There is no third option, and a library that
promises both is lying about one of them.

### A11 — The numeric tower

**1.** Because a `BigInt` and a `Number` do not agree on the values they can represent, so
any coercion loses information silently in one direction or the other. Take
`2n ** 53n + 1n`, which is `9007199254740993n`. Coerce to double and it becomes
9007199254740992 — the value is not representable, and the addition you asked for produced
a wrong answer with no signal. Coerce the other way (double → BigInt) and `0.5` has no
integer form at all. There is no coercion direction that is total and lossless, and the one
place a language must not guess is where the guess is silent and the domain is exact
arithmetic — which is the entire reason `BigInt` was added. Throwing is the only choice that
preserves the property `BigInt` exists to provide. `==` across the two is still permitted
because comparison can be defined exactly without producing a value in either domain.

**2.** It fixed **silent truncation**: `total / count` in Python 2 with two ints gave floor
division, so an average of 7 and 8 items was 0 rather than 0.875, and the bug is invisible
because the result is a perfectly plausible number. It created **silent float
contamination**: `/` now returns a float even when both operands divide exactly, so an
exact integer computation acquires a double partway through, and by the time you are past
2^53 the results are quietly wrong. Indices, byte counts, and money computations are the
victims, and the symptom appears only at large magnitudes — the exact inverse of the old
bug's profile. Python's mitigation is `//`, which shifts the burden to the author knowing
which one they meant, which is a real improvement over the old default only because the new
failure is louder in the common case.

**3.** For: **it never surprises you with a performance cliff.** Promotion to bignum means
an arithmetic operation that was a register add becomes an allocation and a multi-word loop,
non-uniformly, based on data — so a program can be fast for a year and then hit a slow path
in production with no code change. Promoting to float keeps the operation O(1) and the
representation one word. The killer: **floating point is not associative and not exact**, so
the promoted value silently stops obeying the algebraic laws every integer algorithm assumes.
`(a + b) + c ≠ a + (b + c)`, equality comparisons between computed values fail, and — the
decisive case — a loop counter or an array index that promoted to float now increments to a
value where `x + 1 == x`, and the loop does not terminate. Bignum promotion degrades
performance; float promotion degrades *correctness*, and a core library may not choose the
second.

**Trap.** "Doubles are exact up to 2^53, so promotion is safe for any realistic integer."
Exactness of the *representation* is not exactness of the *arithmetic*. A sum of values each
under 2^53 exceeds it; a product does so immediately; and the moment one intermediate rounds,
every downstream comparison and every `==` on a computed value is unreliable, at magnitudes
far below where anyone thought to check. The bound applies to values you store, not to values
you compute, and integer algorithms compute.

### A12 — What counts as false

**1.** Costs: (a) **in the interpreter** — every conditional is potentially a user-visible
method call, so the branch cannot be a plain test-and-jump; you need a fast path for known
types and a slow path that dispatches, plus an inline cache on the condition site, and a
`__bool__` that raises means *the `if` statement itself can throw*. (b) **in the type
system** — a static analyser cannot conclude anything about a condition from the value's
type, so narrowing (`if x:` implies `x is not None`) is unsound in general; Python's own type
checkers special-case a list of known types to recover it. What it buys a library author is
**making a domain type behave like a container** — a `Result` that is falsy when empty, a
`Matrix` that is falsy when zero, a query object that is falsy when it has no rows — and it
buys the extremely common `if not items:` reading naturally. NumPy's decision to *raise* on
the truthiness of a multi-element array is the counterexample that proves the cost: the
protocol is expressive enough to have no sensible answer for some types.

**2.** The VM **special-cases the send at the compiler**: when the receiver of `ifTrue:` is
compiled and the argument is a literal block, the compiler inlines both the block and the
branch, emitting a conditional jump rather than a send — and does the same for `and:`,
`or:`, `whileTrue:`, and `to:do:`. This is not an optimisation applied late; it is a fixed
list of selectors the compiler knows. What it must check to stay correct is that **the
receiver really is a Boolean at runtime** — so the emitted branch is a jump-if-true that
also verifies the value is exactly `true` or `false` and traps to a real send otherwise. That
trap is what preserves the semantics for a user object that implements `ifTrue:`, and it is
why the fast path is a two-way check, not a single bit test. The correctness obligation this
creates is that user code must not be able to *redefine* `Boolean>>ifTrue:`, or the inlined
form and the sent form diverge — which is question Q17.

**3.** **`NaN`**. The others are all "empty or zero", which is a coherent if debatable
category a reader can hold in their head. `NaN` is falsy while being a `Number` that is
neither zero nor empty, and — the part that makes it uniquely bad — it is *produced* by
ordinary arithmetic on ordinary-looking input: `parseInt("abc")`, `0/0`, arithmetic on
`undefined`. So a value flows through a numeric pipeline, becomes `NaN` at some step, and
then a downstream `if (x)` silently takes the false branch as though the value were zero.
Every other falsy value in that list is something you can see in the source; `NaN` is
something the arithmetic made for you. Second place goes to the `null`/`undefined` pair
being two distinct falsy absences, but that at least fails loudly under `===`.

### A13 — The one-armed conditional

**1.** The options, and the cost each pushes to the use site:

- **`nil`/`null`** — costs an absence check at every use, and, fatally, is ambiguous with a
  legitimate `nil` result (see part 3).
- **A wrapped `Option`/`Maybe`** — costs an unwrap and, in a naive implementation, an
  allocation per evaluation.
- **A type error** — make the one-armed form a statement, not an expression, so it has no
  value. Costs expressiveness: you can no longer write `x = cond ifTrue: [...]` at all, and
  the user writes an explicit two-armed form. This is what statement-oriented languages do
  and it is a completely defensible answer.
- **Return the receiver** — `false ifTrue: [...]` evaluates to `false`. Cheap, no new
  values, and terrible: the expression's value is now a Boolean sometimes and a `42`
  sometimes, and the two mean unrelated things.
- **Require the two-armed form syntactically** — the grammar admits only
  `ifTrue:ifFalse:`, so the question never arises. Costs the ergonomics of the guard idiom,
  and is the only option that makes the ambiguity structurally impossible.

**2.** What differs is **whether the check is enforced and whether absence is
distinguishable from a value.** With `nil`, the check is a convention: the compiler will
happily let you send a message to the result, and you find out at runtime. With `Option`,
the check is a type obligation: the value is not usable as a `42` until you discharge the
absence, and the compiler names the site where you didn't. The reader gains the same thing —
`Option<Int>` in a signature announces the absence at the API boundary, where `nil`-able
`Int` announces nothing, because in a `nil`-everywhere language *every* type is `nil`-able
and the annotation carries no information. The information content is identical; the
*enforcement* and the *locality* are not, and those are the whole value.

**3.** The ambiguity: `x = cond ifTrue: [ lookup(k) ]` yields `nil` both when `cond` was
false and when `cond` was true and the key was missing. Two different program states, one
value, and no way to distinguish them at the use site — which means the caller cannot
correctly implement "if the condition held, use the result even if it is nil". The standard
fix is to **wrap**: return `Some(v)` on the taken branch and `None` on the untaken one, so
`Some(nil)` and `None` are distinct. The allocation: a naive `Some` is a heap object
allocated on every evaluation of a conditional, which is unaffordable in a hot path. The
ways out are the usual ones — a niche-optimised representation where `None` is a reserved
bit pattern so `Option<T>` is the same size as `T` (Rust does this for references), interning
a singleton `None`, or making the wrapper a compiler-known type the optimiser unboxes. Ship
the wrapper only if you have one of these; otherwise you have traded a correctness bug for a
per-branch allocation, which is a bad trade in a core library.

**Trap.** "`Option` and nullable are the same thing with different syntax." They are the
same *information* and a different *contract*. The tell is `Option<Option<T>>`, which is
well-formed and has three distinct inhabitants, versus `T??`, which in a nullable design
collapses. The moment you need to store "an absent value" in a container that also uses
absence to mean "not present", the collapse is a real bug and the distinction stops being
cosmetic.

### A14 — `+` on strings

**1.** The diagnostic that gets worse is the one for a **type confusion between a number and
its rendering**. `total + count` where `count` came from a form field and is a string: with
`+` defined on strings and coercion, JavaScript gives `"12" + 3 === "123"` and no error at
all; without coercion but with `+` overloaded, you get "no method `+` on String accepting
Integer", which names the operator rather than the mistake. Compare a language where string
concatenation is a distinct operator — `..` in Lua, `<>` in Elixir, `++` in Haskell: the same
mistake reads as "attempt to perform arithmetic on a string value", which points at the
actual error, and the wrong-operator case points at the other one. A distinct operator makes
the two intents distinguishable *in the error*, which is the entire argument.

**2.** Overloading `+` across numeric and string types makes the operator's type a
constrained polymorphic scheme rather than a concrete signature, so `f(a, b) = a + b` no
longer infers a monotype — it infers a constrained one, and the constraint has to be either
resolved by defaulting or propagated into `f`'s signature. Haskell pays exactly this: `+`
lives in the `Num` class, so a literal `1` has type `Num a => a`, and the language needs
**type defaulting rules** plus, historically, the **monomorphism restriction** to stop
perfectly reasonable programs from being ambiguous or from silently becoming slow
polymorphic dictionaries. Haskell then *declines* to put string concatenation in `Num`,
which is the relevant data point: even the language with the machinery to overload `+`
across unrelated domains chose not to, because `Num`'s laws (associativity is fine,
commutativity is not — `"a" + "b" ≠ "b" + "a"`) do not hold for strings. Overloading an
operator across domains with different algebraic laws is how you get a class whose laws
nobody can state.

**3.** The trap is **quadratic concatenation**: `s = s + x` in a loop allocates and copies
the whole accumulated string on every iteration, so building an n-character string costs
O(n²) bytes copied. It is invisible in the source — the loop looks linear — and it is one of
the most common real performance bugs in Java, which is why `StringBuilder` exists and why
"don't concatenate in a loop" is folklore every Java programmer recites. The platform's
answer was to make the compiler rewrite concatenation *within a single expression* into a
builder, and later to compile `+` to an `invokedynamic` call site linked by a
`StringConcatFactory` bootstrap, so the JDK can change the concatenation strategy without
recompiling anything. Note what that admits: the operator was cheap enough to write that
users wrote it in the wrong place, and the fix had to be made in the compiler and the
runtime rather than in the library — which is the through-line of this whole file. A core
library operator is a language feature.

### A15 — Three jobs, one method

**1.** The user-facing role (`Display`, `__str__`, `to_s`) carries the obligation to be
**readable and lossy** — no quotes, no type name, formatted for a human reading output. The
developer-facing role (`Debug`, `__repr__`, `inspect`) carries the obligation to be
**unambiguous**: distinct values must render distinctly, the type must be evident, and
strings must be quoted and escaped so `"1"` and `1` are distinguishable. The mechanically
testable property belongs to the second: Python's convention is that `eval(repr(x)) == x`
should hold where possible, which is a round-trip property you can property-test across a
whole type. Nobody can test "is this readable"; everybody can test "does the repr round-trip
and does it separate values `==` separates".

**2.** The third job is **serialisation into a machine-consumed context** — a URL segment, a
SQL literal, a cache key, a log field a parser will read. Its obligation is neither
readability nor debuggability but **stability and a defined grammar**: the same value must
produce the same bytes across versions, locales, and platforms. Conflating it with the
user-facing role is how a float renders as `1,5` under a European locale and breaks a CSV;
how a date renders as "3 minutes ago" and lands in a database column; how changing a
`toString` for nicer log output silently changes a cache key and invalidates a cache. The
failure is that the user-facing rendering is *supposed* to change — it is presentation — and
the machine-facing one is *supposed* not to, so putting them in one method makes every
cosmetic change a compatibility break.

**3.** Java fails specifically because `toString` is invoked implicitly by `+` and by every
logging and debugging path, so **one method serves an audience that wants terse and an
audience that wants complete, and the class author must pick one**. The production failure:
a class with a `toString` designed for logs — which includes an internal ID or a full field
dump — gets concatenated into a user-visible message or an exception message that reaches an
API response, and now internal state is exfiltrated. The inverse is more common and more
tedious: a terse user-facing `toString` means every debugging session shows
`Order[id=17]` where you needed the whole object, so people write `toDebugString` by hand,
inconsistently, in a third of the classes. "Just be disciplined" fails at the library
boundary because the *choice* is made by the class's author and the *use* is made by
someone else's code — the author cannot know which audience will call it, and there is no
signature difference to warn either of them. That is the general lesson: when one name
serves two contracts, the contract is decided by whoever called it last.

### A16 — Raise or return

**1.** Operational rule: **raise when the caller cannot reasonably be expected to have a
handler at the call site, and return a value when the absence is part of the operation's
normal domain.** The test that makes it operational — ask whether a correct program would
write `try`/`catch` immediately around this call. If yes, the exception is doing the job of
a return value badly, and you should return the value. If the realistic handler is many
frames up (a request boundary, a top-level loop), raising is right, because propagating a
sentinel through those frames means every intermediate frame checks and forwards it, and one
of them will forget. Note this makes the answer *context-dependent*, which is exactly why
mature libraries ship both forms — `[]` and `fetch`, `[]` and `.get` — rather than choosing.

**2.** (a) The **unwinding path itself**: constructing the exception, capturing a stack
trace (usually the dominant cost — it is O(depth) and touches every frame), the search for a
handler, and running cleanup on the way. (b) The **optimisation barrier**, which persists
even when nothing is thrown: a call that may throw is a control-flow edge out of the current
block, so the compiler cannot freely reorder stores across it, cannot always keep values in
registers across it, and must maintain enough metadata to describe the frame at the throw
point. That second cost is paid by every call site in the program whether or not the
exception is ever constructed, and it is why "exceptions are zero-cost when not thrown" is
true of the *table-driven dispatch* and false of the *code the compiler is allowed to emit
around the call*.

**3.** Rust's `Option` makes it impossible to **use the value without discharging the
absence** — there is no path where you read a "value" that was never there, because the
missing case is not inhabited by a default. Go's `(value, ok)` returns a zero value in the
missing case, and `v, _ := m[k]` compiles fine and gives you a zero that is
indistinguishable from a stored zero. That is the entire difference, and it is the same
`Some(nil)` ambiguity from Q13. What Go's version makes possible: **ignoring the absence
locally without ceremony**, which sounds like a vice and is genuinely useful — `m[k]`
returning the zero value is the right behaviour for a counter map, and in Rust you write
`.unwrap_or_default()` or `.copied().unwrap_or(0)` to say the same thing. Go optimises for
the case where the zero value is meaningful; Rust optimises for the case where it is not.
Both are coherent, and a core library must pick one as the *default* because the default is
what appears in every example.

### A17 — The sealed kernel

**1.**
- **Inline arithmetic without a guard on the method.** `a + b` where both are integers can
  compile to a machine add behind a tag check. In an open world the tag check is not
  sufficient — the *method* could have been replaced — so you also need a version check or a
  cache on the send site, and you must be prepared to deoptimise. Sealed: the tag check
  alone is complete.
- **Compiler-inlined control flow.** `ifTrue:`, `and:`, `whileTrue:` can be emitted as
  branches rather than sends, with no fallback path. Open world: you emit the inlined form
  plus a check that the receiver is a real Boolean plus a slow send path, and you accept
  that the inlined and sent forms are two implementations that can drift.
- **Fixed field layout and direct slot access.** If `String` cannot be subclassed, its
  representation is known and every access is a constant offset — and, more importantly, a
  `String` value can be *unboxed* or given a compact representation because nothing can add
  a field. Open world: layout may change at runtime, so access goes through an indirection
  or a layout-versioned cache.
- Related, and worth naming: **method caches never need invalidating for sealed classes**,
  so the VM does not need a class-hierarchy-change hook on the hot path.

**2.** They pay for it with **speculation plus deoptimisation**. The JIT compiles the
optimistic version — inline the current `SmallInteger>>+`, assume no subclass overrides —
and records a dependency on that assumption. The safety mechanism is a **dependency
registry plus a deoptimisation point**: when someone redefines the method or loads a
subclass, the runtime invalidates every compiled method that depended on the old assumption
and forces execution back to the interpreter at a mapped bytecode index, from which it can
recompile. HotSpot does this with class hierarchy analysis and dependency tracking; V8 does
it with maps and code dependencies on those maps. The cost is not the deopt itself — that is
rare — it is that the entire runtime must be built so that *any* compiled frame can be
converted back into an interpreter frame at a safepoint, which constrains register
allocation, requires precise metadata at every deopt point, and is one of the hardest parts
of the system to get right. Sealing buys you the same speed for free; speculation buys it
for the price of an entire deoptimisation subsystem.

**3.** It forecloses the **library-as-language-extension** pattern that is the whole reason
Smalltalk and Ruby are pleasant. Concretely impossible: a decimal-money library adding
`Integer>>dollars` so `5 dollars` reads naturally; a testing library adding
`Object>>should:`; a duration library adding `Integer>>seconds`; and — the strongest case —
adding a protocol conformance to a type you do not own so it works with your generic
algorithm. That last one is a real expressiveness loss and it is why Rust has coherent trait
impls, Swift has retroactive conformance, and Clojure has protocols: all three are answers
to "extend a type you don't own *without* mutating it globally". The partial answer is
exactly that shape: **allow extension, forbid replacement, and scope the extension**. Seal
the existing methods and the layout, permit adding new selectors, and make the additions
lexically or module-scoped so two libraries adding the same name do not collide. Ruby's
refinements are this idea; the reason they are unloved is that scoped extension is much
harder to implement fast, because the method lookup now depends on the caller.

**Trap.** "Sealing is a performance hack, so a fast enough JIT makes it unnecessary."
Backwards. Speculation gets you the same *steady-state* speed and costs you the entire
deopt machinery, plus a warmup period, plus a cliff whenever someone actually does the
thing you speculated against. In a simple bytecode interpreter with no JIT — which is most
new languages for their first several years — there is no speculation to fall back on, so
sealing is not an optimisation, it is the *only* way certain operations are ever fast.
Decide it before v1, because unsealing later is compatible and sealing later is not.

### A18 — Adding a method is a breaking change

**1.** Mechanism: in an open-class language, a user or a third-party library has already
added `Object>>flatten` — or more likely `Array>>flatten` — with different semantics. There
is one global method dictionary per class, so the two definitions are the *same slot*; the
last one loaded wins, silently, with no diagnostic. A program that worked because it got the
gem's `flatten` now gets yours, or vice versa depending on require order, and the failure
appears as wrong behaviour in a call site that mentions neither library. The real instance:
Ruby 2.4 added `Array#sum` to core, and ActiveSupport had shipped its own `Array#sum` with
different semantics for some argument shapes — the resolution required ActiveSupport to
detect the core version and defer to it. Objective-C is worse: two categories defining the
same selector on the same class produce *undefined* winner with no warning at load time,
which is why every Objective-C style guide mandates prefixing category methods.

**2.** Default methods fixed **adding a method to an interface**, which before Java 8 broke
every existing implementor at compile time — the reason `Collection` could not gain
`stream()` without breaking the world. A default method supplies an implementation, so
existing implementors keep compiling. The new failure: a class that implements two
interfaces which both provide a default for the same signature is now a **diamond conflict**
— a compile error the class author must resolve explicitly, caused entirely by a change in
two libraries they do not control. And more subtly, a class that already had a method with
that name now silently *overrides* the new default, so it satisfies the interface with a
method that was never written to honour the interface's contract — a semantic break that
compiles cleanly. Java traded a loud break for a quieter one, which is usually the right
trade and is not a free one.

**3.** Ranked by cost to the user, cheapest first:

- **Namespace the extension.** Put new methods behind a scoped mechanism — a refinement, a
  protocol/trait the user imports, an extension visible only where the module is imported.
  Cost to the user: an import. This is the right answer and it is only available if the
  language shipped scoped extension *before* v1, which is the point of the last clause.
- **Prefix or qualify the name.** `Object>>libFlatten`. Cost: ugliness forever, and it does
  not compose — every library doing this yields a root class with a hundred prefixed
  methods.
- **Deference at load time.** Define the method only if it is not already defined, or detect
  and defer. Cost: the semantics of your library now depend on load order, and a user who
  loads in the other order gets a different program. This is what real ecosystems actually
  do and it is the worst of the three.

The pre-v1 decision: **make extension scoped by construction, and keep the root class
small.** A root class with three methods can gain a fourth; a root class with two hundred
methods is a namespace everyone shares and nobody owns, and every addition to it is a
collision waiting for the right two libraries to meet. Go's position — no root type at all,
and interfaces satisfied structurally — is the extreme version of this argument, and the
reason adding a method to a Go type never breaks anyone else is precisely that there is no
shared slot to collide in.

**Trap.** "Semantic versioning covers this: adding a method is a minor version." Semver's
rule assumes additions cannot break callers, which is true in a closed-world language and
false the moment classes are open — the break is not in *your* API surface, it is in a slot
you share with everybody. In an open-class ecosystem, adding a method to a widely-inherited
class is a major-version-shaped change wearing a minor-version number, and the ecosystems
that live this way (Ruby, Objective-C) have all developed load-order folklore in place of a
rule.
