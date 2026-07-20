# 05 — Values and Representation

Fitting a universe of values into 64 bits. The through-line: *every representation buys
speed by spending some other part of the design space, and the bill always comes due
somewhere far from where you signed.*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — The bits you do not have

A 64-bit IEEE-754 double has an 11-bit exponent. When every exponent bit is set and the
mantissa is non-zero, the value is a NaN — and the hardware only ever *produces* one
specific quiet NaN. NaN-boxing stuffs pointers and other immediates into the remaining
patterns. LuaJIT, JavaScriptCore, and SpiderMonkey's 64-bit build all do this. V8 does not.

1. Count the payload bits honestly, then explain why the scheme works on x86-64 and ARM64
   today and what hardware change would break it. Name what implementations do about it.
2. V8 represents doubles as heap-allocated `HeapNumber` objects instead — strictly more
   allocation on float-heavy code. What did V8 buy that NaN-boxing makes impossible?
3. In a NaN-boxed VM, `nil`, `true`, `false`, and small integers are all bit patterns in
   the same word. Name one thing you can no longer do to those values, and be specific
   about *why* the representation is what forbids it.

### Q2 — Two ways to hide an integer

Two production schemes for making integers immediate:

- **Low-bit tagging.** OCaml sets the low bit on integers; heap pointers are 8-byte
  aligned so their low bits are free. Integers are 63-bit.
- **High-half tagging.** V8's Smi on 64-bit puts a 32-bit integer in the upper half of the
  word with the low bits zero, so a Smi *looks like* an aligned pointer to nothing.

1. Write the add sequence for two OCaml-style tagged integers and explain why it is more
   than one instruction. Then explain why V8's Smi addition is cheaper and what it costs
   in range.
2. Both schemes make "is this a pointer?" a bit test. Why does that test have to be
   *branch-predictable* to be worth anything, and what does that imply about how you order
   the type checks in an arithmetic opcode?
3. Ruby made most doubles immediate too (flonums) by rotating the bit pattern so that the
   common exponent range lands on a free tag. Some doubles remain heap-allocated. Why is a
   representation that is immediate *most* of the time worse than one that is immediate
   *always*, from the compiler's point of view?

### Q3 — What an immediate cannot do

```ruby
x = 42
def x.shout; "I am #{self}"; end   # TypeError: can't define singleton method
```

Ruby lets you attach a singleton method to almost any object. Not to an Integer, a Symbol,
`nil`, `true`, or `false`.

1. Explain the refusal from the representation up. What structure does a singleton method
   require, and where would it have to live?
2. Name two other capabilities that an immediate representation forecloses, other than
   singleton methods. For each, say what per-object storage the capability needs.
3. A VM wants immediates *and* wants `42.shout` to work. Sketch the two honest ways out
   and say what each costs on the hot path.

### Q4 — The object the JIT deleted

```java
for (int i = 0; i < n; i++) {
    Point p = new Point(i, i * 2);
    sum += p.x + p.y;
}
```

HotSpot's C2 will often make this loop allocate nothing.

1. Describe what C2 actually does here. The common answer — "it stack-allocates the
   object" — is wrong for HotSpot; say what it really does and why that is a stronger
   transformation.
2. Escape analysis is famously brittle: one change to the loop body can make allocation
   reappear with no other visible difference. Name the two most common causes, and say why
   they are causes rather than coincidences.
3. The optimized code hits a deoptimization and falls back to the interpreter, which
   expects a real `Point` object at a real address. The object was never created. What
   must the runtime have recorded, and when?

### Q5 — Interning rewrites the equality contract

A VM interns all symbols in a global table so that symbol comparison is a pointer compare.

1. Interning makes `==` on symbols exact and O(1). State precisely what the program can no
   longer observe once two equal symbols are guaranteed identical, and give a case where
   that loss is load-bearing.
2. Ruby's symbol table was not collected before 2.2, and `"user_#{n}".to_sym` in a request
   handler was a denial-of-service vector. Explain the mechanism, and explain why making
   the table weak is harder than it sounds.
3. Java has `String.intern()` but strings are *not* interned by default, while symbols in
   Lisp/Ruby/Smalltalk always are. What property of the two value kinds justifies the
   different default?

### Q6 — The invariant nobody states out loud

```java
class Key {
    int id;
    public boolean equals(Object o) { return o instanceof Key k && k.id == id; }
    // no hashCode override
}
map.put(new Key(1), "a");
map.get(new Key(1));   // null
```

1. State the invariant that was violated, in the direction that matters. (One direction is
   required; the other is only a performance concern. Say which is which and why.)
2. Now suppose `hashCode` is correct, but the caller mutates `id` after insertion. Describe
   the resulting state of the table precisely — not "it breaks", but what the entry's
   physical situation is and which operations can and cannot still find it.
3. Python 3 sets `__hash__ = None` on any class that defines `__eq__` without `__hash__`,
   making instances unhashable. Java does not. Argue which is the better language design
   decision and name what the stricter choice forecloses.

### Q7 — A comparator that corrupted the heap

A C program calls `qsort` with a comparator that is not a consistent total order — say it
compares floats with `<` and the array contains a NaN, or it returns a value derived from
a mutable field that changes during the sort.

1. In a Hoare-style partition, the inner scan loops are typically written without a bounds
   check. Explain exactly why that is normally safe, and exactly how an inconsistent
   comparator turns it into an out-of-bounds write.
2. Java's `Arrays.sort` on objects can throw `IllegalArgumentException: Comparison method
   violates its general contract!`. Why did the JDK add that check, and why is it a
   *detection* rather than a *prevention*?
3. Rust's `slice::sort_by` documents that a bad comparator may panic or produce a
   nonsensical order but will never violate memory safety. What does the implementation
   have to give up to promise that, and why is it the right call for Rust specifically?

### Q8 — NaN is not a total order

```rust
let mut v = vec![3.0, f64::NAN, 1.0];
v.sort();            // does not compile
v.sort_by(|a, b| a.partial_cmp(b).unwrap());  // compiles, may panic
```

Rust splits `PartialOrd` from `Ord`. `f64` implements only the first.

1. NaN makes `<`, `>`, and `==` all false against everything including itself. Show
   concretely which axiom of a total order that breaks, and why a sort algorithm's
   correctness proof depends on that axiom rather than on the comparator merely being
   "sensible".
2. IEEE-754 specifies a `totalOrder` predicate; Rust exposes it as `f64::total_cmp`. It
   orders `-NaN < -inf < -0.0 < +0.0 < +inf < +NaN`. Why is `-0.0 < +0.0` under total order
   but `-0.0 == +0.0` under the arithmetic comparison, and why is having *both* the right
   answer rather than a wart?
3. Most languages let you `sort` an array of floats with no ceremony. Name what those
   languages have decided not to tell you, and give a program whose output differs between
   two conforming implementations because of it.

### Q9 — Zero, negative zero, and the not-a-number key

```js
new Set([NaN, NaN]).size        // 1
NaN === NaN                     // false
new Map().set(-0, "a").get(0)   // "a"
Object.is(-0, 0)                // false
```

```java
Double.valueOf(Double.NaN).equals(Double.valueOf(Double.NaN))  // true
Double.valueOf(0.0).equals(Double.valueOf(-0.0))               // false
0.0 == -0.0                                                     // true (primitives)
```

JS and Java reconciled the same conflict in opposite directions.

1. Explain the conflict that forces a choice: what does a hash-table key contract demand
   that IEEE-754 comparison refuses to supply?
2. JS's collections use SameValueZero (NaN is its own key, ±0 are the same key); Java's
   boxed `Double.equals` does the reverse on both counts. Defend each choice on its own
   terms — each one is right for something.
3. In CPython, `x = float('nan'); [x].count(x)` is `1`, but `[float('nan')].count(float('nan'))`
   is `0`. What implementation shortcut in the container explains this, and what does it
   imply about writing `__eq__` for a type you intend to put in containers?

### Q10 — Three integer semantics

For `a * b` overflowing a 64-bit signed integer, three real answers:

- **Wrap** — C's unsigned, Java, Go, Rust release builds.
- **Promote** — Python, Ruby, Smalltalk, Scheme: silently widen to arbitrary precision.
- **Trap** — Rust debug builds, Swift by default, C# `checked`.

1. Take each in turn and say what it forecloses *for the compiler* — not for the
   programmer. Be specific about which optimizations become legal or illegal.
2. Promotion means the static type `Integer` covers both an immediate and a heap bignum.
   What does that do to the inline caches and the arithmetic fast path, and what is the
   standard shape of the generated code?
3. Swift traps on overflow and is used in shipping applications where a crash is worse than
   a wrong number. Justify Swift's choice against that objection, and name the escape hatch
   the language must therefore provide.

### Q11 — "Length" is a design decision

```
Java/JS   "😀".length  == 2
Python    len("😀")    == 1
Go        len("😀")    == 4
Swift     "😀".count   == 1
Swift     "👨‍👩‍👧".count  == 1        // three people, one Character
```

Four encodings — UTF-16 code units, code points, UTF-8 bytes, extended grapheme clusters —
and four different answers to the same question.

1. Java and JS get O(1) indexing at the cost of `length` not counting characters. Python
   gets O(1) indexing *and* code-point counting. Explain the mechanism Python uses and the
   pathological memory case it creates.
2. Rust's `String` refuses `s[3]` entirely; Swift's `String` refuses integer subscripting
   and makes you carry an `Index`. These look like ergonomic hostility. Argue the design:
   what invariant is each one protecting, and what class of bug does it eliminate?
3. You must pick one representation for a new language's string type. Argue for UTF-8 with
   byte indices, and then state honestly the two operations you have just made O(n) that
   users will expect to be O(1).

### Q12 — Grapheme clusters and the moving target

Swift's `Character` is an extended grapheme cluster. `"e\u{301}"` (e + combining acute) has
`count == 1`.

1. Grapheme cluster boundaries are defined by Unicode's segmentation algorithm, which
   changes between Unicode versions. Name the concrete failure this creates for a language
   that makes grapheme clusters the *unit of its string type*, as opposed to a library
   function.
2. Given that, why is grapheme-cluster `count` still arguably the correct default for a
   general-purpose language, and what does making it the default cost at runtime?
3. A hash table keyed on strings. Should `"é"` (one code point) and `"e\u{301}"` (two) be
   the same key? Answer, then explain what your answer forces you to do at insertion time
   and what it forecloses.

### Q13 — Why Rust's String has no small-string optimization

`std::string` in libstdc++ and libc++ both store short strings inline in the object with no
heap allocation. Rust's `String` never does — it is always a heap pointer, length, and
capacity, even for `"hi"`.

1. Name the language-level rule in Rust that makes the C++ technique unavailable, and be
   precise about the mechanism, not the philosophy.
2. There is an SSO design that *is* compatible with that rule. Describe it and say what it
   costs on every single string access.
3. C++ pays for its SSO in a place people rarely name. Identify at least two costs — one at
   the call site of a copy or move, one in the object layout.

### Q14 — Concatenation that does not copy

```js
let s = "";
for (let i = 0; i < 100000; i++) s += "x";   // fast in V8 and JSC
```

Naively this is O(n²). It is not, because engines represent the result as a rope — V8 calls
it a `ConsString` — a tree of concatenation nodes flattened lazily.

1. Ropes make `+` O(1). Name the operation they make slower and the code shape that turns
   the optimization into a latency spike.
2. Java's `String.substring` was O(1) before 7u6 — it shared the parent's `char[]` with an
   offset and count. It was changed to O(n) copying. Explain the bug class that motivated
   the change, and note the language that still has it by design.
3. Given (1) and (2), a string type can be a rope, a slice, or a flat buffer, but a general
   string type usually cannot be all three. Say why, in terms of what each representation
   requires of the accessor path.

### Q15 — Copy-on-write and the copy you did not ask for

Swift's `Array` and `String` are value types with copy-on-write: assignment is a retain,
and mutation checks `isKnownUniquelyReferenced` before writing.

1. Describe the exact condition under which a mutation triggers a full copy, and construct
   a case where a user's innocuous refactor introduces a copy inside a hot loop.
2. Immutability lets Java cache a `String`'s hash code in the object. Explain why that
   caching is not merely an optimization enabled by immutability but *impossible* without
   it — including the subtlety about a zero hash.
3. COW gives value semantics with reference-level sharing. Name what it costs at the
   instruction level on every mutation, and name the concurrency hazard that COW with
   non-atomic reference counts introduces.

### Q16 — The absent value you cannot forge

```lua
t = {1, 2, nil, 4}
#t          -- may be 4 or 2; the manual says it is undefined
t[3] = nil  -- indistinguishable from "never set"
```

Lua uses `nil` as both a first-class value and the marker for "no entry". JS's
`Map.get` returns `undefined` for both a missing key and a key stored with value
`undefined`, which is why `Map.has` exists.

1. Explain the general failure: what breaks when a protocol's "no value" signal is drawn
   from the same domain as its values? Give the two standard repairs and say what each
   costs.
2. Kotlin's `T?` compiles to a plain nullable reference — zero allocation. Scala's
   `Option[T]` allocates a `Some`. Rust's `Option<Box<T>>` is the same size as `Box<T>`.
   Explain the third case's mechanism and what property of the inner type it requires.
3. Python library code uses a module-private `_MISSING = object()` as a default argument
   rather than `None`. Explain why `None` is insufficient and what property of `object()`
   makes it sufficient. Then name the property that makes this pattern *fail* across a
   process boundary.

### Q17 — The identity of a copy

```csharp
struct Counter { public int N; public void Inc() => N++; }
object boxed = new Counter();
lock (boxed) { }                 // locks the box
List<Counter> list = new();
list.Add(new Counter());
list[0].Inc();                   // compiler error, and for a good reason
```

```go
type S struct{ mu sync.Mutex }
func (s S) Do() { s.mu.Lock() }  // go vet: Do passes lock by value
```

1. Value types have no identity: two copies with equal fields are indistinguishable.
   Name three language features that *require* identity and therefore have to do something
   awkward or illegal when handed a value type.
2. Explain the Go example concretely. What is physically wrong at the moment `Lock` is
   called, and why does the failure mode make it a *vet* check rather than a compile error?
3. Java's Valhalla project introduces value classes that explicitly renounce identity. List
   what a class gives up by renouncing it, and explain why renouncing it is what buys the
   performance — the flattening argument, stated in terms of memory layout.

---

## Answers

### A1 — The bits you do not have

**1.** A quiet NaN is: exponent all ones (11 bits) plus the top mantissa bit set. That
pins 12 bits, plus the sign bit is free, leaving **51 bits of mantissa payload plus the
sign bit — 52 bits** to encode a tag and a value. A pointer needs to fit in that. It does,
because x86-64 and ARM64 implementations currently use **48-bit virtual addresses** with
the top bits sign-extended, and user-space allocations land in the low half, so the
meaningful pointer is 47 bits. That leaves a few bits for a tag.

The hardware change that breaks it is **wider virtual addressing** — Intel's 5-level paging
(LA57) gives 57-bit virtual addresses, and ARM has 52-bit variants. A 57-bit pointer does
not fit alongside a tag in 52 bits. Implementations respond by *constraining where the heap
lives*: LuaJIT's original x64 port required all GC memory in the low 2 GB and famously fell
over on systems that would not cooperate, which is why **GC64 mode** exists — it widened
the value representation and gave up the tightest packing. Most systems only enable
57-bit addressing for processes that explicitly request high mappings, so the scheme
survives by convention rather than by guarantee.

**2.** **Pointer compression.** V8 stores tagged values as **4-byte** words inside a 4 GB
heap cage, with the 32-bit value being an offset from a base register. That halves the size
of every object field and every array-of-pointers, which is an enormous cache-footprint
win — and it is completely incompatible with NaN-boxing, which is intrinsically a 64-bit
word format. V8 accepted boxing doubles (`HeapNumber`) as the price, then clawed most of it
back with unboxed double fields in objects and typed arrays, plus a JIT that keeps doubles
in registers across the parts that matter. The lesson: NaN-boxing optimizes the *value
word*; pointer compression optimizes the *object graph*, and the object graph is bigger.

**3.** You cannot give them **per-object state of any kind**, because there is no object —
there is no header, no address, nothing to hang a side table off that is keyed by identity
rather than by value. Concretely: no per-object identity hash distinct from the value, no
instance variables, no per-object monitor/lock word, no weak reference to them (weakness is
meaningless for something with no allocation to reclaim), and no per-object class pointer,
which is why the class of an immediate must be derived from its *tag* by a switch rather
than loaded from memory.

**Trap.** Saying NaN-boxing "gives you 48 bits for a pointer, which is all pointers." It
gives you the pointers *this generation of hardware and this OS's allocation policy* hand
you. It is a bet on an implementation detail of the MMU, and it is the one part of a VM
that a kernel change can invalidate.

### A2 — Two ways to hide an integer

**1.** OCaml's tagged integer `n` is stored as `2n+1`. Adding two of them:
`(2a+1) + (2b+1) = 2(a+b)+2` — the tag bit is lost, so the sequence is
`add; sub 1` (or `add; dec`, or `lea` tricks). Multiplication is worse: you must untag one
operand (`>>1`), multiply, and re-tag, so `a*b` is roughly `shift, dec, imul, inc`. Every
arithmetic operation carries a fixed tag-repair tax, and the integer is 63-bit, so `Int64`
is a *boxed* type in OCaml — a real, visible language consequence.

V8's Smi on 64-bit is the value in the **upper 32 bits**, low bits zero. Addition of two
Smis is a plain 64-bit `add`: the low halves are zero so nothing carries into the tag, and
the result is already a valid Smi. No repair at all. The cost is **range**: 32 bits (31
with pointer compression), so V8 has to fall back to `HeapNumber` far earlier than OCaml
falls back to bignums. Different point on the same curve: OCaml pays per-operation to keep
range; V8 pays range to make the operation free.

**2.** Because the test is on the hot path of *every* arithmetic and *every* field load, and
a mispredicted branch costs more than the arithmetic it guards. Modern predictors handle
this well precisely because real programs are monomorphic in value shape — a loop over ints
takes the "is Smi" branch a million times identically. That implies the ordering rule:
**check the overwhelmingly common representation first, with a fallthrough (not-taken)
fast path**, and put the general case out of line. It also implies you should not write the
check as a switch over four tags — you write it as one predicted test with everything else
funnelled into a cold slow path, because a four-way indirect dispatch is not predictable in
the same way.

**3.** Because a *sometimes*-immediate representation forces the compiler to keep the boxed
path live everywhere. If every double is boxed, the compiler has one shape to reason about
and can specialize a loop to unboxed machine doubles once it proves the type. If most
doubles are immediate and some are not, every operation still needs the guard and the
fallback, and — worse — the property "is this double immediate?" depends on the *exponent
range of the runtime value*, which is not a type. You cannot hoist the check out of a loop
by proving a type; you have to re-check per value. It buys real memory wins on typical
numeric data and costs the compiler a representation it can never fully specialize away.

### A3 — What an immediate cannot do

**1.** A singleton method requires a **singleton class**: a fresh, per-object class inserted
into the object's lookup chain, which means the object's class pointer must be *mutable and
per-object*. An immediate has no class pointer; its class is computed from its tag. There is
nowhere to write the new class, and no header to write it into. You would need a side table
keyed by object identity — but for an immediate, identity *is* the value, so `42`'s
singleton class would be shared by every `42` in the program, which is not a singleton class,
it is a subclass of Integer with an implicit and confusing scope.

**2.** (a) **Per-object monitors/locks** — need a lock word in the header (Java puts it in
the mark word). (b) **Weak references and finalization** — need an allocation whose death is
observable; an immediate never dies. Also credible: (c) **identity hash distinct from
value hash**, which needs a header field or a hash-of-address; (d) **instance variables**;
(e) **object mutation in place** — you cannot `become:` an immediate in a Smalltalk sense
because there is no cell to swap.

**3.** (a) **Auto-boxing at the boundary**: when someone asks for behaviour the immediate
cannot supply, promote to a heap object with a header, and accept that `42` and the boxed
`42` are now two representations of one value — which means every `==`, every hash, and
every dispatch site has to normalize, and you have re-created Java's `Integer`/`int` mess
including its identity-comparison trap. (b) **Give up immediates for that type**, which
costs an allocation and an indirection on the hottest data in the language. Most languages
pick (a) and then spend a decade explaining the seams; the honest hot-path cost of (a) is
that the class-of check in dispatch becomes "tag switch, then if boxed, load header",
i.e. two shapes for one type at every send site.

**Trap.** "It's just a Ruby quirk / an implementation limitation they could lift." It is a
direct consequence of choosing immediates, and every language with immediates has the same
hole in the same place — the only variable is whether the language admits it in the manual
or papers over it with boxing.

### A4 — The object the JIT deleted

**1.** C2 performs **scalar replacement**: after escape analysis proves the object does not
escape the compilation unit, the object is *deleted* and its fields become individual SSA
values living in registers. There is no object, on the stack or anywhere. This is stronger
than stack allocation because the fields become first-class candidates for every subsequent
optimization — constant folding, loop-invariant hoisting, register allocation, dead-store
elimination. A stack-allocated object is still memory with loads and stores; a
scalar-replaced object is registers. The distinction is why "the JVM stack-allocates
objects" is a persistent and wrong piece of folklore.

**2.** (a) **A call that did not get inlined.** Escape analysis in HotSpot runs after
inlining and is intraprocedural over the inlined region; passing the object to a method the
inliner refused (too big, too polymorphic, hit an inlining budget) means the object escapes
by definition. So a change that pushes a callee over the inline threshold — anywhere in the
call tree — can un-optimize an allocation three frames away. (b) **The object being stored
into anything that outlives the region**: a field, an array, a static, or being returned,
or being thrown. Also: a megamorphic call site on the object, and — historically — being
used as a lock. These are causes rather than coincidences because escape analysis is a
*may-escape* analysis: it must be conservative, so one unanalyzable use poisons the whole
allocation, not just that use.

**3.** The compiler must have recorded, in the **debug info / deoptimization metadata
attached to every safepoint in the compiled code**, a description of the deleted object:
its class, and for each field, which SSA value (register or stack slot) currently holds it.
On deopt, the runtime **rematerializes** the object — allocates it for real and populates it
from those locations — before building the interpreter frame. This has to be recorded at
compile time, at every point where deopt is possible, which is exactly why aggressive
optimization inflates metadata size, and why deopt is not free even when it never happens.
It also implies a nasty corner: rematerialization *allocates*, during deoptimization, which
can trigger GC at a moment the runtime would rather not.

**Trap.** "Escape analysis eliminates the allocation, so the allocation is gone." It is gone
*in that compiled version*. A deopt puts it back, and if the deopt is in a loop that
re-enters the interpreter, you can get an allocation rate *higher* than the unoptimized
program while the method is being recompiled.

### A5 — Interning rewrites the equality contract

**1.** The program can no longer observe that two equal symbols came from **different
origins** — different source locations, different parses, different network payloads.
Identity has been redefined from "same allocation" to "same value", so identity carries no
provenance. That matters wherever identity was doing work: you can no longer use an
identity-keyed side table to attach distinct metadata to two occurrences of the same name,
and you cannot use identity to distinguish a symbol you created from one a caller passed in.
The load-bearing case is **capability-style protocols**: if you wanted "only the holder of
*this particular* token may call this", interning destroys it, because anyone who can write
the same characters can forge the token. Interned values are unforgeable only if the
*construction* is restricted, never because of identity.

**2.** Every distinct symbol created a permanent entry in a global table that was a GC root.
An attacker sending unbounded distinct strings that the app converts to symbols grows the
table without bound — memory exhaustion with no way to release. Making the table weak is
hard because (a) the symbol is often *represented as an index or a tagged immediate*, so
there is no object whose death the collector can observe; (b) the intern table must be
consulted on every symbol creation, so it is on a hot path and a weak table with clearing
and rehashing is more expensive; and (c) you must guarantee that a symbol reachable only
from *compiled code constants* or from the intern table's own bookkeeping is still counted
as live, which means the compiler's literal pool becomes a root set you have to be exact
about. Ruby's fix was to split symbols into static (from source literals, immortal) and
dynamic (from runtime strings, collectable) — which is really a confession that the two
have different lifetimes and should never have shared a table.

**3.** **Cardinality and lifetime.** Symbols are drawn from a small, program-determined set:
they come from source code, they are used as identifiers, and their count is bounded by the
program's size. Strings are drawn from data: unbounded, attacker-influenced, and often
short-lived. Interning a bounded set of long-lived values is pure win; interning an
unbounded set of short-lived ones is a memory leak with extra hashing. Java's `intern()` is
opt-in for exactly that reason — and it was still a foot-gun when the table lived in
PermGen, where over-interning produced `OutOfMemoryError: PermGen space` rather than normal
heap pressure.

### A6 — The invariant nobody states out loud

**1.** **`a.equals(b)` implies `a.hashCode() == b.hashCode()`.** That direction is required.
The converse — equal hashes implying equality — is not required and is impossible anyway
(pigeonhole); it is purely a performance property, since collisions just mean the bucket
holds more candidates that then get `equals`-tested. The required direction is what makes
the table's *pruning* sound: a hash table finds candidates by bucket, so if equal objects
can land in different buckets, the table will confidently report absence for something it
contains. Note the failure is silent and wrong, not slow.

**2.** The entry is **stranded**. It is physically present in the bucket determined by the
*old* hash, but every lookup computes the *new* hash and searches a different bucket. So:
`get(k)` fails, `containsKey(k)` fails, `remove(k)` fails to remove it, and `size()` still
counts it. Iteration *will* find it, and `entrySet()` will hand you a live entry whose key
does not resolve. If the table rehashes on resize, the entry moves to the bucket for the new
hash and may spontaneously become findable again — meaning the bug's symptom depends on
insertion history and load factor, which is why it is usually diagnosed late and from a
heap dump rather than from a stack trace.

**3.** Python's is the better *language* decision and Java's is the better *compatibility*
decision. Python's rule turns a latent correctness bug into an immediate `TypeError` at the
point of misuse, and it encodes the real relationship: hashability is a property you must
opt into once you have declared a custom notion of equality. What it forecloses is
**identity-hashed mutable objects with value equality** — a perfectly reasonable design
(mutable node objects you want to compare structurally but store in a set by identity) now
requires you to write `__hash__` explicitly, and beginners write `__hash__ = object.__hash__`
without understanding that they have just re-broken the invariant in the other direction.
Java cannot adopt the rule because `Object.hashCode` already exists and every class inherits
it; the strictness has to be there from day one or not at all.

**Trap.** "Just make `hashCode` return a constant — it satisfies the contract." It does
satisfy it, and it is legal, and it turns your `HashMap` into a linked list with O(n)
lookups — and in a web-facing service, into an algorithmic-complexity DoS. Satisfying a
contract is not the same as being usable.

### A7 — A comparator that corrupted the heap

**1.** The classic partition scans inward: `while (cmp(a[i], pivot) < 0) i++;` and
`while (cmp(a[j], pivot) > 0) j--;`. There is no `i < hi` bound because the **pivot itself
is a sentinel** — the scan is guaranteed to stop at the pivot's position, since
`cmp(pivot, pivot)` is not `< 0`. That guarantee comes entirely from the comparator being a
consistent total order. If the comparator is inconsistent — because it says `x < pivot` for
a value that is also `>= pivot` on a later call, or because NaN makes every comparison
false so `cmp` returns a value that never stops the scan — the sentinel property evaporates
and `i` walks off the end of the array. The subsequent swap writes **outside the array**.
This is not a hypothetical; it is the standard mechanism by which "my comparator was
sloppy" becomes heap corruption in C.

**2.** Because Java's TimSort merge relies on invariants about run lengths, and a comparator
that violates transitivity can put the merge state into a configuration the algorithm's
proof excludes, previously producing an `ArrayIndexOutOfBoundsException` from deep inside
library code with no hint of the real cause. The JDK added the explicit check to convert an
incomprehensible internal failure into a message that names the actual bug. It is detection
rather than prevention because **verifying a comparator is a total order is O(n³)** in
general (transitivity is a three-element property) — you cannot afford to check it, you can
only notice when the algorithm's own invariants are violated as a side effect. So it is
best-effort: some inconsistent comparators sort quietly and wrongly, and never trigger it.

**3.** The implementation must **bound every access independently of the comparator** — no
sentinel-based unbounded scans, explicit index checks in the partition and merge loops, and
careful handling of temporary buffers so that a comparator that panics or mutates the slice
mid-sort cannot leave the buffer holding duplicated ownership of the same element. Rust's
sorts also have to be panic-safe: if `cmp` panics halfway through a merge, every element
must be back in the slice exactly once, or you get a double-free during unwinding. This is
the right call for Rust because **safe code must not be able to cause UB**, full stop — a
comparator is safe code, so a safe `sort_by` that could corrupt memory would be a soundness
hole in the language, not a bug in the caller. The cost is a small constant factor of bounds
checks and a much more intricate implementation, which Rust pays because the alternative
would break the language's central promise.

### A8 — NaN is not a total order

**1.** It breaks **totality** (for all `a, b`: `a ≤ b` or `b ≤ a`) and it breaks
**reflexivity** of `≤` (`NaN ≤ NaN` is false). A comparison sort's correctness argument is
not "the comparator is sensible", it is that the comparator induces a total preorder, which
is what licenses the algorithm's *transitive* reasoning: having established `a ≤ p` and
`p ≤ b` during partitioning, the algorithm never re-compares `a` and `b`, it *deduces*
`a ≤ b`. NaN destroys the deduction, so the algorithm's skipped comparisons become wrong
assumptions, and (per A7) can escape the array's bounds. The axiom is load-bearing at the
level of the proof, not at the level of taste.

**2.** Because they answer different questions. Arithmetic comparison answers "are these the
same *number*", and `-0.0` and `+0.0` are the same number — every arithmetic identity you
want (`x == 0` implying `x + 0 == x`, and so on) depends on that. Total order answers "how
do I lay these out in a sequence such that every element has a defined position", and there
`-0.0` and `+0.0` are distinct *bit patterns* that must be given an order. Having both is
correct because the two are genuinely different relations on the same set, and collapsing
them would break one or the other: if you made `==` distinguish signed zeros, arithmetic
breaks; if you made sorting use `==`, sorting breaks. Rust's `PartialOrd`/`Ord` split is the
type system admitting out loud that `f64` supports one and not the other — and `total_cmp`
is the escape hatch that lets you opt into the other relation explicitly.

**3.** They have decided not to tell you that **the result is unspecified when NaN is
present**, and often that the sort's *stability* or even its termination is only guaranteed
for well-ordered input. A program: sort `[3.0, NaN, 1.0, 2.0]` in a language whose sort is
implemented as a comparison sort with a `<`-based comparator. The final position of `NaN`
and, crucially, whether `1.0` and `2.0` end up correctly ordered relative to each other,
depend on the pivot choices and the algorithm — so the same source produces different
arrays under two conforming implementations, or even under the same implementation for
inputs of different lengths (many libraries switch to insertion sort under a threshold).
Nothing in the spec is violated; the spec simply never said.

**Trap.** "Just filter the NaNs out first." That handles floats. The general bug is any
comparator whose result depends on mutable state or on a non-transitive relation —
`sort` by "is a friend of", `sort` by a `Comparator` that special-cases a "pinned" element
to always come first (which is famously non-transitive), or a comparator reading a field
that another thread mutates. NaN is the memorable instance, not the category.

### A9 — Zero, negative zero, and the not-a-number key

**1.** A hash table key needs an **equivalence relation**: reflexive, symmetric, transitive,
and consistent with the hash. IEEE-754 `==` is not reflexive (NaN) and it identifies two
values with distinct bit patterns (`±0`) that a bit-pattern-derived hash would put in
different buckets. So the table is offered a relation that fails on both ends: one value
that is never equal to itself (so you can store it and never retrieve it, and store it twice
and get two entries), and two values that are equal but might not hash the same (so lookup
finds the wrong bucket — the A6 failure). Something has to be overridden.

**2.** **JS's SameValueZero** optimizes for the collection being *useful*: `NaN` as a key
works, `set.has(NaN)` after `set.add(NaN)` is true, and `-0`/`+0` collapsing means numeric
keys behave like numbers. It is the pragmatic choice for a language where `Map`/`Set` are
general-purpose containers used by ordinary programmers, and it deliberately diverges from
`===` because `===` would make `Set` silently useless for NaN.

**Java's `Double.equals`** optimizes for the boxed `Double` being a faithful *wrapper around
a bit pattern* — it is documented to behave "as if" comparing `doubleToLongBits`. That makes
`Double` a well-behaved hash key with a total, reflexive equality where `equals` and
`hashCode` agree by construction, and it makes `Double[]` sortable and `TreeMap`-able
without surprises. The cost is the famous inconsistency with `==` on primitives, which
surprises everyone; the benefit is that the *collection contract* is satisfied exactly,
mechanically, with no special cases. Java chose "the wrapper is a bit pattern"; JS chose
"the container should do what you meant".

**3.** CPython's `list.__contains__` (and `index`, `count`, `remove`) does an **identity
check before the equality check** — `x is y or x == y`. So a NaN compared against *itself*
short-circuits to true on identity and never reaches the IEEE comparison; two distinct NaN
objects fall through to `==`, which is false. The implication for `__eq__`: the container may
*never call your `__eq__` at all* for identical objects, so you cannot write a type whose
`__eq__` returns false for `x == x` and expect containers to honour it. Practically, your
`__eq__` should be reflexive, because the container has already assumed it is, and if it is
not, your object's behaviour inside containers will differ from its behaviour outside them
in ways that depend on whether the caller happened to pass the same object.

**Trap.** "Python's `in` uses `==`." It uses identity-or-equality, and the difference is
observable for NaN, for objects with expensive `__eq__` (the shortcut is also a real
performance feature), and for any `__eq__` that is not reflexive. Same shortcut exists in
`dict` lookup, where it is the reason a dict can find a key whose `__eq__` would be
expensive or would raise.

### A10 — Three integer semantics

**1.** **Wrap** foreclosures: the compiler cannot assume `a + 1 > a`, so it loses a large
family of loop optimizations that depend on induction variables not wrapping — bounds-check
elimination, loop-invariant motion, strength reduction, and reasoning about trip counts.
This is precisely why C and C++ leave *signed* overflow **undefined** rather than wrapping:
UB restores `a + 1 > a` as an assumption and buys back the optimizations, at the cost of
programs that overflow becoming nondemonic. Java chose wrapping and accepted the weaker
optimizer; Go did too.

**Promote** forecloses: the compiler cannot assume the value fits in a machine register at
all. Every arithmetic site is potentially a call, potentially allocating, and therefore
potentially a **GC safepoint** and potentially a place that can throw (out of memory). That
poisons instruction scheduling around every `+`, and it means integer arithmetic can no
longer be hoisted freely across allocation-sensitive regions.

**Trap** forecloses: arithmetic becomes an operation with a **side exit**, so it is not pure
and cannot be freely reordered, speculated, or eliminated as dead — you cannot delete an
unused `a * b` if it might have trapped. It also constrains vectorization, since SIMD
integer ops do not raise per-lane traps. What it *buys* the compiler is the wrap
assumption: since overflow cannot silently occur, `a + 1 > a` holds on all non-trapping
paths, so you get back much of what wrapping lost.

**2.** The static type covers two representations, so the fast path is a **guarded
monomorphic sequence**: check both operands are the immediate form, do the machine
operation, check the overflow flag, and on either failure branch to a cold path that boxes,
promotes, and calls the general routine. The generated code shape is "unbox-free fast path +
out-of-line slow path", with the slow path never inlined so it does not blow up the hot
block's I-cache footprint. For inline caches specifically, this means the *type* is not a
sufficient cache key: a site that sees `Integer` may still take two different code paths per
value, so either your cache keys on representation (not just class) or your fast path
includes the representation guard unconditionally. Most VMs do the latter for arithmetic and
reserve the cache for genuine method dispatch.

**3.** Swift's argument is that a silently wrapped integer is a *corrupted value that
propagates*: it becomes an array index, an allocation size, a length. The overwhelmingly
common consequence of an unchecked overflow in systems code is not a slightly wrong number,
it is a buffer overflow — so a trap converts a memory-safety bug into a controlled crash,
and crashing is the safe direction when the alternative is executing attacker-influenced
memory operations. Against the "a crash is worse" objection: the crash is *at the site of
the bug*, which makes it findable, whereas the wrapped value fails somewhere else. The
required escape hatch is **explicit wrapping operators** — Swift's `&+`, `&*`, `&-` — for
the cases where wrapping is the intended semantics (hashing, checksums, PRNGs, ring
buffers), plus reporting forms like `addingReportingOverflow` for code that wants to handle
it. A trapping language without those operators is unusable for exactly the code that needs
it most.

### A11 — "Length" is a design decision

**1.** Python uses **PEP 393's flexible string representation**: at construction, the string
is scanned and stored in the narrowest of latin-1 (1 byte/char), UCS-2 (2), or UCS-4 (4)
that fits its widest code point. Every element is then the same width, so `s[i]` is one
indexed load — O(1) indexing *and* code-point semantics. The pathological case is that the
width is a property of the **whole string**, so a single astral character in a megabyte of
ASCII inflates the entire string 4×. Concatenating one emoji onto a large ASCII string
produces a UCS-4 copy. Java's compact strings (JEP 254) are the same idea with two levels
(latin-1 or UTF-16) and the same cliff.

**2.** **Rust** is protecting the invariant that a `String` is **always valid UTF-8**. Byte
index 3 might land in the middle of a multi-byte sequence, so `&s[..3]` could produce an
invalid string; Rust makes slicing check the boundary and panic rather than silently
constructing an ill-formed value. The bug class eliminated is *mojibake and invalid
encodings propagating silently* — including the security-relevant version where a truncated
UTF-8 sequence changes how a downstream parser interprets the rest.

**Swift** is protecting the invariant that a `Character` is a **user-perceived character**.
An integer subscript would have to be O(n) (grapheme boundaries are not fixed-width) and,
worse, would tempt users to write index arithmetic that silently splits a grapheme cluster.
By making the index an opaque type produced only by the string itself, Swift eliminates the
class of bugs where slicing a string cuts a combining mark, a surrogate pair, or a ZWJ
sequence in half — the bug that produces broken emoji in every text field that truncates by
"characters".

**3.** UTF-8 with byte indices: it is the wire format, so I/O is zero-conversion; it is
ASCII-compatible so the common case is 1 byte per character; slicing and concatenation are
`memcpy`; and there is exactly one representation, so no width-promotion cliff. The two
operations I have made O(n): **indexing by character position** (`s[i]` for the i-th code
point or grapheme requires a scan) and **`length` in anything other than bytes**. Users
expect both to be O(1) because their previous language gave them that. The honest mitigation
is to make the O(n) forms *look* O(n) in the API — iterators rather than subscripts — so
nobody writes `for i in 0..s.len() { s.char_at(i) }` and gets a quadratic loop, which is the
actual damage.

### A12 — Grapheme clusters and the moving target

**1.** **The value of `count` — and the identity of the elements — changes when you update
the Unicode tables.** A string containing a newly-defined ZWJ emoji sequence has count 3
under an old table and 1 under a new one. If graphemes are a library function, that is a
library version issue. If they are *the unit of the string type*, then the type's own
semantics are version-dependent: iteration yields a different sequence of `Character`s, any
persisted index is invalid across versions, and — the sharp one — a hash table or sorted
structure built by one version can be *misread* by another. On platforms where the Unicode
tables come from the operating system rather than the binary, the same program produces
different results on two machines. That is why Swift eventually shipped its own Unicode
support in the standard library rather than deferring to the host ICU: to make string
semantics a property of the toolchain, not the deployment target.

**2.** Because the alternatives are all wrong in ways users hit constantly: code units are
wrong for anything outside the BMP, and code points are wrong for every accented character,
every emoji with a modifier, every Devanagari or Hangul cluster. The number a user means by
"how long is this" is the grapheme count, and every other default silently mis-truncates
text in a UI. The runtime cost is that `count` is **O(n) with a table-driven state machine
per character** rather than a field read, that comparison must be canonical-equivalence
aware (so `==` on strings is not `memcmp`), and that the string type carries a nontrivial
Unicode data dependency. Swift accepted all three; the fact that `String.count` is O(n) is a
standing complaint and a standing correctness win.

**3.** **They should be the same key** if your language claims Unicode-correct string
equality, because they are canonically equivalent and users cannot tell them apart. That
forces you to **normalize (NFC or NFD) at insertion and at lookup**, or to hash over the
normalized form without materializing it. What it forecloses: your string hash is no longer
`hash(bytes)`, so it is meaningfully more expensive; you lose the ability to round-trip
arbitrary byte sequences through the table unchanged; and you have imported the Unicode
version dependency from (1) into your *hashing*, meaning a persisted hash table can be
invalidated by a toolchain upgrade. The alternative — bytewise keys — is fast and stable and
will one day tell a user that the name they typed is not the name in the database.

### A13 — Why Rust's String has no small-string optimization

**1.** **In Rust, moves are `memcpy` and nothing else.** There is no move constructor; the
compiler is free to relocate a value bit-for-bit, and it does so routinely (returning by
value, pushing into a `Vec`, matching). An SSO string stores the data *inside itself*, so if
the object also holds a pointer to that inline buffer, the pointer is a **self-reference**,
and a bitwise move leaves it pointing at the old address. C++ can do SSO because a move
constructor is user code that runs on every move and fixes the pointer up. Rust deliberately
has no such hook — that is the whole basis of its "moves are trivial" model, which is what
lets it avoid generating and inlining move constructors everywhere.

**2.** The compatible design stores no self-pointer: instead use a **tagged union with a
discriminant** — either (ptr, len, cap) or (inline bytes, small len) — and branch on the
discriminant at every access to compute the data address as either "load the pointer" or
"take the address of `self` plus an offset". No stored self-reference, so a bitwise move is
fine. The cost is a **branch on every single access** to the string's bytes, including
`len()`, `as_bytes()`, `deref` to `&str`, and every iteration setup. That branch is
predictable, but it also blocks the compiler from treating `String` as a simple pointer, and
it makes `&str` extraction from a `String` not free. This is exactly what crates like
`smallstr`/`compact_str` do — and the fact that they are crates rather than the default is
the language saying "pay this only where you have measured it".

**3.** (a) **At the call site**: a move or copy of `std::string` is not a `memcpy` of a
fixed-size struct — it is a call to a constructor that branches on whether the source is
inline (copy the bytes) or heap (steal the pointer, null the source). That code is inlined at
every move, inflating code size, and it is why passing `std::string` by value has a real cost
even when "moving". (b) **In the object layout**: the object is much larger than three words
— typically 32 bytes in libstdc++ and libc++ — so every `std::string` field in every struct
costs 32 bytes and every `vector<string>` has 32-byte stride, worsening cache density for
the *long* strings that gain nothing from SSO. Also credible: (c) the implementations are
ABI-locked to their layouts, which is why libstdc++'s COW-to-SSO change required a
dual-ABI (`_GLIBCXX_USE_CXX11_ABI`) that still causes link errors today.

### A14 — Concatenation that does not copy

**1.** Ropes make **indexing, and anything that needs contiguous bytes, slower** — the
accessor must either walk the tree or *flatten* first. Engines flatten lazily on first
access. The code shape that turns it into a latency spike is: build a huge rope in a loop,
then touch it once — `s[0]`, `s.length` in some engines, a regex match, passing it to a
native API, or writing it out. The flatten is a single O(n) allocation-and-copy of the whole
string at an unpredictable moment, so a program that appeared to have amortized-constant
concatenation suddenly stalls on a multi-megabyte copy at the first read. Worse: it is a
*large* allocation, so it can also trigger a GC. The optimization has moved the cost, not
removed it, and it has moved it somewhere the programmer is not looking.

**2.** **Unbounded retention.** `hugeString.substring(0, 5)` returned a 5-character string
that held a reference to the parent's entire `char[]`. Parse a 100 MB document, keep ten
small field values, retain 100 MB. Since the small string looked small to every profiler
metric a developer would check, the leak was invisible without a heap dump showing the
shared backing array. Java traded O(1) substring for O(n) copying to make retention
proportional to what you kept. **Go still has this by design**: a string is a (pointer, len)
into a backing array, so `s[:5]` of a huge string retains the whole allocation, and the
documented remedy is to force a copy (`strings.Clone`, or a round-trip through `[]byte`).
Rust has the same property with `&str` but makes it visible in the type system — a `&str`
obviously borrows, so lifetime rules tell you the parent is alive, and you must `to_owned()`
to detach.

**3.** Because each representation demands a *different accessor contract*. A flat buffer
promises "there is a contiguous byte range at a known address" — that is what lets you hand
a pointer to `write(2)`, to a regex engine, to FFI, with zero work. A rope cannot promise it
without flattening. A slice can promise it but cannot promise "this allocation is exactly my
data", which is what breaks retention. A general string type must pick which promise its
*entire API surface* makes, because the promise leaks into every consumer: the moment one
function does `s.as_ptr()`, ropes must flatten there, and if the flatten is implicit and
common, the rope buys nothing. Engines get away with all three only because the string type
is *not* exposed at the ABI level — JS has no `as_ptr`, so the engine is free to change
representation under the program. A systems language cannot hide it, which is why Rust ships
`String` (flat, owned) and `&str` (slice) as distinct types and leaves ropes to libraries.

### A15 — Copy-on-write and the copy you did not ask for

**1.** A copy triggers when a mutation occurs and the buffer's reference count is **greater
than one** — i.e. someone else is currently holding the same storage. The innocuous refactor:
a loop that mutates `self.items` in place, and then someone adds a line that passes
`self.items` to a logging or validation helper, or captures it in a closure, or stores it in
a local for readability. If that reference is live across the mutation, every iteration now
finds refcount 2, copies the whole array, mutates the copy, and drops it — turning an O(n)
loop into O(n²) with no change to the loop body. The reason it is hard to spot is that the
extra reference is often in code that looks read-only, and "reading" is exactly what
*prevents* the in-place fast path.

**2.** Because a cached hash is only correct if the value cannot change after the cache is
populated. With a mutable string, any write invalidates the cache, and there is no hook that
the string type can rely on to notice — the string might be mutated through an alias, or
through a `char[]` obtained from it. Immutability makes the cache a *pure memoization of a
function of the object's permanent state*, which is why it can be filled lazily and
non-atomically: two threads racing to compute it will compute the same value, so the
unsynchronized write is benign. The subtlety about zero: the cache field is initialized to 0
and 0 is used as the "not computed yet" sentinel, so a string whose hash genuinely *is* 0
(the empty string, and `"f5a5a608"` among others) recomputes on every call. That is a
harmless performance quirk precisely because the recomputation is guaranteed to produce the
same answer — again, an immutability consequence. Java eventually added a separate
`hashIsZero` flag to fix it.

**3.** At the instruction level, every mutating operation pays a **reference-count load and
compare, and a branch** — before the actual write. That is cheap in isolation, but it is on
every element write, and it defeats vectorization of the write loop unless the compiler can
hoist the uniqueness check out (which Swift's optimizer tries hard to do, and which is the
main reason `array[i] = x` in a loop is sometimes fast and sometimes not). The concurrency
hazard: **if the reference count is non-atomic, two threads sharing the value can race on
it**, and losing that race means both threads conclude they are unique and both mutate the
same buffer — or the count is decremented twice and the buffer is freed while still in use.
So COW forces you to choose between atomic refcounts everywhere (a real throughput cost on
every assignment) or a language rule that the value cannot be shared across threads, which
is what Swift's concurrency checking and `Sendable` exist to enforce.

### A16 — The absent value you cannot forge

**1.** The failure is **loss of injectivity in the protocol**: the reply "no value" is
indistinguishable from the reply "the value is *this*", so the consumer cannot recover which
happened. Concretely it costs you the ability to store the sentinel value, to distinguish
"absent" from "present but empty", and to define `length` on any structure that permits
holes. The two standard repairs: (a) **a separate query** — `has`/`contains`/`hasNext` —
which costs a second lookup and introduces a TOCTOU window in concurrent code; (b) **a
wrapper type** — `Option`/`Maybe`/`{found, value}` — which is exact and composable but costs
an allocation per lookup unless the compiler can avoid it. There is a third, cheaper repair:
(c) **an unforgeable sentinel** outside the user-visible value domain, which is zero-cost
but requires the runtime to have a value the language cannot construct.

**2.** Rust's is the **niche optimization**. `Box<T>` is a non-null pointer — the compiler
knows the type has an invalid bit pattern (all zeros) that no valid value can occupy — so
`Option<Box<T>>` encodes `None` as that niche and needs no discriminant. Same size, and
`match` compiles to a null check. It requires the inner type to have a *statically known
invalid representation*: non-null pointers, `&T`, `NonZeroU32`, `char` (which excludes
surrogates, so it has spare patterns in 32 bits), and enums with unused discriminants all
qualify. `Option<u64>` does not, and is 16 bytes. Kotlin gets the same zero-cost result for a
much narrower case by simply being nullable references at the JVM level, which is why
Kotlin's `T?` cannot nest (`T??` is meaningless) while Rust's `Option<Option<T>>` is a
distinct, well-defined type — the type-level answer composes, the representation-level one
does not.

**3.** `None` is insufficient because `None` is a **legal argument value**, so
`def f(x=None)` cannot distinguish `f()` from `f(None)` — which matters for any API where
"explicitly pass nothing" differs from "don't pass" (config merging, `dict.get`-style
defaults, ORM field updates). `object()` works because it creates a **fresh instance with a
unique identity**, and equality on a bare `object` is identity, so no caller can produce a
value equal to it without having a reference to the module-private name. The property that
makes it fail across a process boundary is that **identity is not preserved by
serialization**: pickle, JSON, an RPC, or a `multiprocessing` fork-and-send will either fail
to serialize it or produce a *different* object on the other side that is no longer
identical to the local `_MISSING`. Unforgeability by identity is a property of a single
address space; the moment your protocol crosses one, you need the distinction to be in the
data, not in the pointer.

**Trap.** "Just use a sentinel like `-1` or an empty string." That is exactly the Lua
problem with a different domain. A sentinel is only sound if you can *prove* it is outside
the value domain, and for any type a user can extend, you cannot.

### A17 — The identity of a copy

**1.** (a) **Locking / monitors** — `lock`/`synchronized` need a stable object to associate a
monitor with; a value has no such thing, so C# must box it, and each box is a *new* object,
which is why `lock (someStruct)` locks a fresh monitor each time and synchronizes nothing.
(b) **Weak references and finalization** — both are about observing the death of a specific
allocation; a value has no allocation of its own to die. (c) **Reference equality and
identity hash** — `ReferenceEquals`/`==` on a boxed value compares boxes, so two boxes of the
same value are unequal, which quietly breaks identity-keyed caches. Also credible:
(d) **mutation through an alias** — `list[0].Inc()` is a compile error in C# because the
indexer returns a *copy*, and mutating a copy that is immediately discarded is always a bug;
the language chose to reject it rather than let it silently do nothing.

**2.** The method has a **value receiver**, so calling `s.Do()` copies the entire struct,
mutex included, and `Lock` is taken on the **copy's** mutex. The copy dies when `Do`
returns. So the lock protects nothing: two goroutines both "succeed" in locking their own
private copies and enter the critical section together. Worse, if the original was already
locked when the copy was made, the copy is born in the locked state and locking it deadlocks
— copying a mutex duplicates its *state*, which is meaningless. It is a vet check rather
than a compile error because Go's type system has no way to express "this type may not be
copied": copying is a universal, unconditional operation on every Go value, and `sync.Mutex`
is an ordinary struct. Making it an error would require a `noCopy` concept in the language.
(Go's stdlib fakes it with an embedded `noCopy` type that vet recognizes — a lint convention
standing in for a missing type-system feature. Rust expresses the same constraint natively
as "not `Copy`, and `MutexGuard` is not `Send`".)

**3.** A value class gives up: **reference equality** (`==` becomes componentwise),
**identity hash** (`System.identityHashCode` has nothing to hash), **synchronization** on
instances, **weak references / `Cleaner` registration** targeting them, and **mutability**
(a value class must be immutable, because there is no single canonical instance to mutate).

Renouncing identity is what buys the performance because identity is exactly what forces a
value to have **its own address**. If two `Point` objects must be distinguishable even when
their fields are equal, then every `Point` needs a distinct location, so a `Point[]` must be
an array of *references* to separately allocated headers — one indirection per element, one
header per element, and a scattered access pattern. Renounce identity and the JVM may
**flatten**: store the fields inline in the array or in the containing object, so
`Point[1000]` is 1000 × 16 bytes of contiguous field data with zero headers and zero
pointer-chasing, and passing a `Point` can be done in registers rather than as a pointer.
Every one of those wins is a direct consequence of "you cannot ask which copy this is."

**Trap.** "Value types are faster because they avoid heap allocation." That is the
consequence, not the reason, and stating it that way leaves you unable to explain why the
feature took so long or why it is hard: the hard part is not allocation, it is retrofitting
a language whose entire object model — locking, `equals`, `==`, weak refs, the null
reference itself — was defined in terms of identity, and finding out which of those a value
class must refuse.
