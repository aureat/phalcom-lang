# 01 — Dispatch and the Object Model

What a method call *is* once objects can be created at runtime. The through-line: *the name
at the call site is not the code that runs, and everything expensive follows from that gap.*

Questions first. Answers below. Do not scroll.

---

## Questions

### Q1 — What the selector carries

```
Smalltalk:  dict at: key put: value      "selector: #at:put:"
Ruby:       dict.store(key, value)       # name: :store, arity checked on entry
```

Smalltalk bakes arity and argument roles into the selector: `#at:` and `#at:put:` are
unrelated symbols with unrelated method-table entries. Ruby interns only `:store`.

1. What does baking arity into the selector buy the *lookup*, and separately, what does it
   buy the *error message*?
2. Both designs must support default parameters. Show why the encoded-selector design makes
   that a compiler problem rather than a runtime problem, and say what the compiler must emit.
3. Name a feature the encoded-selector design makes genuinely hard, and explain why the
   obvious workaround — give varargs its own selector — is not free.

### Q2 — Single dispatch and the visitor tax

Java resolves `visit(Circle)` versus `visit(Square)` statically; the visitor pattern exists
to recover the second dispatch. CLOS and Julia have one generic function with methods keyed
on tuples of argument types.

1. `accept` recovers something specific. Name it precisely, then name what the pattern costs
   at the *design* level — not at runtime.
2. Multiple dispatch destroys a property single dispatch gets for free. Name it, and explain
   why losing it makes separate compilation and modularity harder.
3. Julia's multimethods are fast; CLOS's generally are not. What does Julia have that makes
   specialization pay?

### Q3 — Lookup is the whole performance story

`p.x + p.y` where `p`'s class inherits from three levels up. The naive implementation walks
the class chain hashing the selector on every send.

1. Say why the naive cost hurts, and be specific that it is not the hashing arithmetic.
2. A monomorphic inline cache turns the send into a compare and a jump. State exactly what
   is compared, what is stored, and why the guard must be *class identity* rather than
   method identity.
3. Caches must be invalidated. Give two granularity choices and the concrete failure mode
   of each.

### Q4 — Classes versus prototypes, and the way back

SELF removed classes. JS has no classes in its object model — `class` syntax desugars to a
constructor plus a prototype. Yet V8, JSC, and SpiderMonkey each give every object a hidden
class / structure / shape.

1. What did prototypes actually remove from the *implementation*, and why did every
   implementer put it back?
2. Name something prototypes make cheap that classes make expensive, with a concrete use.
3. Prototype chains and class chains are both lookup chains. Give a semantic difference
   between them that is *not* restatable as "one of them has classes".

### Q5 — Why the metaclass tower terminates

Smalltalk-80: `3 class` → `SmallInteger`; `SmallInteger class` → the metaclass
`SmallInteger class`; `SmallInteger class class` → `Metaclass`; `Metaclass class` →
`Metaclass class`; `Metaclass class class` → `Metaclass`. Python: `type(type) is type`.
Ruby: class-side methods live in singleton classes, created on demand.

1. The tower closes with a cycle rather than a base case. Argue that a cycle is the only
   option, then argue why it is harmless.
2. Smalltalk keeps the metaclass hierarchy parallel to the class hierarchy. What concrete
   user-visible behaviour does that parallel exist to produce, and what breaks without it?
3. Bootstrapping this graph from a VM written in another language has an ordering problem.
   State it, and name the standard resolution and the invariant it depends on.

### Q6 — `super` cannot mean "my superclass"

```
class A            m => "A"
class B extends A  m => "B:" + super.m()
class C extends B  (no m of its own)
```

Evaluate `C.new.m()`.

1. Suppose `super` resolved against the *receiver's* class. Produce the failure exactly,
   then say what `super` must be anchored to instead.
2. Where does the anchor physically live and when is it fixed? Answer for a bytecode VM,
   and then for Ruby, where the same method body can sit at different positions in several
   different ancestor chains.
3. Ruby's `super: no superclass method 'foo'` can fire when a superclass visibly defines
   `foo`. Give a mechanism that produces exactly that.

### Q7 — The MRO that cannot exist

```python
class X: pass
class Y: pass
class A(X, Y): pass
class B(Y, X): pass
class C(A, B): pass   # TypeError: Cannot create a consistent method resolution order
```

1. State the consistency requirement being violated, in terms of orderings.
2. Naive depth-first left-to-right — Python 2.2, and what most hand-rolled systems do —
   accepts this. What does it get *wrong* in the ordinary diamond, and why did Python
   consider that worth a breaking change?
3. C3 makes some hierarchies illegal. Argue that this is the right trade, then give the
   strongest counterargument.

### Q8 — Three answers to the same pressure

Ruby modules (`include` / `prepend`), Scala traits, Rust traits, Java interfaces with
default methods. Two mixins both define `render`.

1. For each of the four, say how the conflict resolves and *when* the programmer finds out.
2. Mixins may or may not carry state. What does allowing mixin state cost the
   implementation, and what does forbidding it cost the user?
3. Java default methods look like mixins. Give a semantic rule Java's have that Ruby's
   `include` does not, and say what that rule protects.

### Q9 — Reifying the miss

Smalltalk `doesNotUnderstand:`, Ruby `method_missing`, Python `__getattr__`, Dart
`noSuchMethod`. A lookup failure becomes a send with the failed call reified.

1. Name two things this makes possible that no static mechanism gives you, and one thing it
   makes impossible.
2. A runtime caches "present" at a call site. Why can it not simply cache "absent" the same
   way?
3. Python splits `__getattr__` (runs only on failure) from `__getattribute__` (runs on every
   access). Explain the performance consequence of that split, and separately, explain why
   Ruby needs `respond_to_missing?` at all.

### Q10 — Open classes and the invalidation bill

```ruby
class String; def blank?; strip.empty?; end; end
```

executed after ten thousand call sites have been compiled and cached.

1. Name the three invalidation strategies a real runtime chooses among, with the cost
   profile of each.
2. Redefining a method on a superclass invalidates assumptions held about *subclasses*.
   What data structure does that require, and what does it cost on a class with many
   descendants?
3. Sealing is the alternative. Name the specific optimization sealing unlocks that no
   amount of caching can, and name what sealing forecloses.

### Q11 — Fields, offsets, and the fragile base class

`Base` has fields `a, b`. `Derived < Base` adds `c`. Instances are flat slot arrays.

1. Why must `Derived` place `a, b` at the same indices `Base` uses, and what exactly does
   that buy?
2. Now `Base` gains a field after `Derived` is compiled and after instances exist. Enumerate
   what breaks. Name the language where this was a famous shipping problem and the mechanism
   that fixed it.
3. A dictionary-per-object design has none of these problems. Say what it costs, and what
   CPython does to claw the cost back.

### Q12 — Shapes, transitions, and the cost of order

```js
function A() { this.x = 1; this.y = 2 }
function B() { this.y = 2; this.x = 1 }
```

1. Explain why A-objects and B-objects get different hidden classes despite identical
   property sets, and what that does to a call site that sees both.
2. Hidden classes form a *shared transition tree* rather than a per-object descriptor. Say
   why the sharing is essential, and describe the tree's root and branching.
3. `delete obj.x`, and adding very many properties dynamically, both push an object into
   dictionary mode. Why is that a cliff rather than a slope, and what does it mean for
   anyone writing a benchmark?

### Q13 — Interface dispatch is not virtual dispatch

JVM `invokevirtual` versus `invokeinterface`. Go: `var w io.Writer = f`.

1. Why is a class method's vtable index a compile-time constant while an interface method's
   is not?
2. Describe Go's itab, and say precisely what work happens at *conversion* rather than at
   the call.
3. Given (1), why do JITs usually make interface calls as fast as virtual calls anyway, and
   what is the residual cost?

### Q14 — What the receiver is bound to, and when

```js
const f = obj.method;  f();     // this === undefined (strict mode)
```
```python
f = obj.method;  f()            # works fine
```

1. Give the mechanism in each language, and be specific about *when* the receiver is
   attached.
2. Python's mechanism allocates. Say what it allocates, and what CPython does to avoid it in
   the common case.
3. JS added arrow functions. State the problem they solve in terms of the mechanism from
   (1), and say why `bind` was not sufficient.

### Q15 — When the cache gives up

A call site inside a serialization framework sees forty receiver types.

1. Trace the tiers — monomorphic, polymorphic, megamorphic. What is stored and what is
   compared at each?
2. Polymorphic caches cap at a small number of entries. Give the actual cost-curve reason,
   not "to save memory".
3. A megamorphic site falls back to a global cache, not to a chain walk. Describe that cache
   and say what makes it correct.

### Q16 — Private is a convention, and that is a design decision

Python `_x` (convention) and `__x` (mangled to `_Cls__x`); Ruby `private` (a rule about the
call *form*); Smalltalk (all instance variables private, all methods public); Java `private`
(verified, then defeated by reflection).

1. Python's `__x` mangling is usually described as weak encapsulation. State what it
   actually protects against, and why that is a different problem entirely.
2. Enforcing privacy in a dynamic runtime forecloses things people rely on. Name three,
   concretely.
3. You want `private` to be *dispatch-visible*: a private send resolves only within the
   defining class. What does that require of the selector or of the send instruction, and
   what does it break?

### Q17 — Ship the closed version first

v1 is class-based with no per-object behaviour: methods live only in classes. v2 must add
singleton methods (`def obj.foo`, or a function as an own property) without breaking v1
programs or v1's speed.

1. What must v1's dispatch path get right so that v2 is purely additive?
2. What must v1's *reflection* surface get right, and what is the specific mistake that
   would trap you?
3. Retrofitting singleton classes carries a cost that appears nowhere in any language spec.
   Name it.

---

## Answers

### A1 — What the selector carries

**1.** Lookup: the selector is a single interned symbol, so a method table is a map from one
word to one method — no arity check, no overload set, no secondary discrimination, and
comparison is pointer equality on interned symbols. The whole dispatch key is computable at
compile time and can be embedded in the call site as a constant, which means an inline cache
guard is one compare against the receiver's class with a pre-resolved key. Error message:
"does not understand `#at:put:`" identifies a *missing protocol*; "wrong number of arguments
(given 2, expected 3)" identifies a *mistyped call to an existing method*. The difference is
load-bearing, because only the first is a thing a `doesNotUnderstand`-style handler can act
on — a proxy can forward `#at:put:` without knowing anything; it cannot meaningfully forward
"you called `store` wrong".

**2.** Because `foo(a, b = 1)` under encoded selectors is really two table entries,
`foo(_,_)` and `foo(_)` — or one entry plus a shim. Something must materialize the shim, and
the runtime cannot, since the runtime only ever sees a fully formed selector arriving from a
call site. So the compiler emits either (a) *N* stub methods, one per selector, each filling
in defaults and tailing into the real body, or (b) one canonical selector plus a call-site
rewrite that evaluates the default expressions *at the caller*. (b) is faster and is wrong
whenever a default must be evaluated in the callee's scope or freshly per call — Python's
mutable-default-argument bug is precisely this choice made in the other direction, evaluated
once at definition instead of per call.

**3.** Keyword arguments in arbitrary order, and true varargs. If `f(a:, b:)` and `f(b:, a:)`
must name one method, you either canonicalize the order in the compiler — fine, but now the
selector is no longer the surface syntax, and reflection, `perform:`, and error messages all
have to un-canonicalize — or you dispatch on the base name and sort at runtime, which
abandons the constant key that was the entire benefit. Giving varargs its own selector is not
free because a call site with *N* arguments must then decide whether to look for `f(_,_,_)`
or fall back to `f(*)`, and the fallback is a *second* lookup taken on a miss. A miss is
exactly the path you cannot afford to slow down, because inline caches key on the resolved
target, so a site that oscillates between the exact and the varargs form never stabilizes.

**Trap.** Calling arity encoding "just name mangling". Mangling is a naming convention over
an unchanged lookup; this changes what a *miss* is. Under encoded selectors an arity error
and a missing-method error are the same event and reach the same handler, which is why
proxies and DNU-based forwarding work uniformly — and why you cannot produce Ruby's arity
diagnostic without additional machinery.

### A2 — Single dispatch and the visitor tax

**1.** `accept` converts a static overload choice into a virtual call on the *first*
argument, so the receiver's own vtable selects the concrete `visit` — the second dispatch is
recovered by promoting the second argument's dynamic type to receiver position, once. The
design cost: the operation set becomes closed against new *types* (adding a shape means
editing every visitor) in exchange for being open against new *operations*. That is the
expression problem, and the visitor pattern is a decision about which side to lose on, made
permanent in the shape of the code.

**2.** **Ownership.** Single dispatch has exactly one owner for a method — the receiver's
class. Multiple dispatch has none: `intersect(Circle, Square)` belongs to neither module.
Three consequences follow. (a) Ambiguity becomes a real error class — two applicable methods,
neither more specific — and it can be *created by loading a third module* that owns neither
type, so a program that worked can break because of a dependency it never mentions. (b) You
cannot compile a call site to a table index, because the applicable set is not known until
every method has been loaded. (c) Encapsulation weakens: a multimethod defined outside a
class gets no privileged access to it, so either the class's internals are public or
multimethods are second-class citizens. Julia lives with the consequences daily; loading a
package can invalidate and force recompilation of already-specialized code.

**3.** Argument types that are known at the call site. Julia is dynamically typed but
aggressively type-*inferred*: it propagates concrete argument types through inference and
emits a specialized method instance per concrete signature, so the multimethod dispatch is
usually resolved during compilation and the call becomes direct. CLOS dispatches at runtime
through a generic function's discriminating function, and the surrounding Lisp code does not
generally supply the type information that would let you skip it. The general lesson is
sharper than "Julia has a good compiler": **multiple dispatch is affordable exactly to the
degree that it is not actually performed.**

### A3 — Lookup is the whole performance story

**1.** The naive cost is a chain walk with a hash probe per level, per send, and what hurts
is not arithmetic — it is that the work is *unpredictable memory traffic*. Each level is a
separate cache line, each probe is data-dependent, and the branch that terminates the walk is
unpredictable as soon as receivers vary. You have turned a call into a pointer-chasing
search. The second-order cost is worse than the first: because you rediscover the target each
time, you can never inline the callee, so the send is also an optimization barrier for
everything around it.

**2.** Compared: the receiver's class pointer (or shape) against a class recorded at that
site. Stored: that class, plus the resolved method — ideally its code address directly. The
guard must be class identity because the method is the thing you are trying to avoid
computing; a guard on the answer is not a guard. The deeper reason is that class identity
determines the *entire* lookup result, including inherited methods and absent ones, so a
single pointer compare certifies a whole chain walk you did not perform. That is the actual
trick, and it is worth saying in exactly those words: the guard's job is to certify a search
that never happened.

**3.** (a) **Global version counter.** Any method definition anywhere bumps it and every
cache dies. O(1) to maintain, catastrophic in a program that defines methods steadily — a
REPL, a lazily-loading framework, a test suite defining doubles — because you get repeated
cold starts and never warm up. Ruby historically worked this way. (b) **Per-class version
words checked in the guard.** Finer-grained, but a redefinition in a *superclass* must
invalidate every descendant, so you either walk the subclass set at definition time (needs a
subclass list, cost proportional to descendants) or check a version at every level of the
chain on every dispatch, which is the cost you were avoiding. (c) **Dependency lists**, which
is what serious JITs actually do: compiled code registers "I assumed `Foo#bar` resolves
here", and redefinition walks the dependents and invalidates or deoptimizes them. Precise,
but it requires a working deoptimization mechanism and metadata proportional to compiled
code.

**Trap.** "The inline cache stores the method, so we skip the lookup." Half of the mechanism,
and the less important half. The load-bearing part is that one cheap comparison is a *valid
proof* about a search over an entire inheritance chain — which is why the interesting design
work is all in choosing what the guard compares and what can silently change underneath it,
not in the storing.

### A4 — Classes versus prototypes, and the way back

**1.** Prototypes removed the one thing implementations depend on most: a stable description
of layout and behaviour that is shared by many objects. A class *is* that description, and it
is what lets you store fields at fixed offsets, identify a whole layout with one header word,
and validate a lookup with one compare. Without it, every object is its own layout. So
implementers reintroduced it under other names — **maps** in SELF, **hidden classes** in V8,
**structures** in JSC, **shapes** in SpiderMonkey. The interesting part is that these are
*derived* rather than declared: the runtime infers the class the programmer refused to write,
which means it can infer a bad one, and the entire performance model of JS is downstream of
whether your code lets it infer a good one.

**2.** Per-object behaviour with no class-shaped ceremony. A singleton, a test double, an
object that gained a method because of how it was configured, differential inheritance —
clone the thing that is ninety-five percent right and override two slots. In a class world
each of those needs a fresh class, which means either a class explosion, or singleton classes
bolted on afterwards (Ruby), or metaclass gymnastics. SELF's argument was that "make a new
kind of thing" and "make a new thing" should be one operation, and that the class/instance
split forces you to decide up front which one you are doing.

**3.** **Mutating an ancestor is observable through already-created descendants, and the
ancestry link itself is per-object and reassignable.** Setting `Foo.prototype.bar` changes
every existing instance because the link is to a live object, not to a description — a
language with class reopening can match that. But `Object.setPrototypeOf(o, other)` changes
one object's ancestry retroactively, and essentially no class-based language offers that. It
is exactly why `setPrototypeOf` is a deoptimizing operation in every JS engine: the derived
shape encodes an assumption that ancestry is stable, and that operation falsifies it for a
single object, which is the worst granularity to have to handle.

### A5 — Why the metaclass tower terminates

**1.** Every object must be able to answer `class`. If the answer must always be a *new*
object, the chain is infinite. A base case would require some object with no class — an
object that cannot receive `class` — which destroys uniformity, the single property the whole
design exists to provide. So the chain must eventually revisit a node it already contains. It
is harmless because the cycle is confined to the *instance-of* relation, while method lookup
walks the *inheritance* relation, and that chain is a finite path terminating at
`Object`/`nil`. A cycle only endangers the relation somebody traverses transitively, and
nobody traverses `class` transitively — reflection asks for it one step at a time.

**2.** It makes **class-side methods inherit**. `Foo class` inheriting from `Bar class` is
what makes a factory or constructor defined on `Bar class` available through `Foo`. Omit it
and every class must redeclare its entire class-side protocol, or `new` becomes a VM
special case rather than a method. Ruby produces the same effect with different vocabulary:
the singleton class of a class inherits from the singleton class of its superclass, which is
why `def self.create` on a parent is callable on a child. Python takes a third route —
metaclasses are ordinary classes, so class-side behaviour inherits through the metaclass's
own MRO, which composes nicely and is also why "metaclass conflict" is a real, reportable
error there.

**3.** The ordering problem: to allocate a class you need its metaclass; to allocate a
metaclass you need `Metaclass`; to allocate `Metaclass` you need `Metaclass class`, which
needs `Metaclass`. The standard resolution is a two-phase bootstrap — allocate the core
objects with null or provisional class pointers in a raw phase, then **patch every pointer**
once all nodes exist, and only then begin executing any bootstrap code. The invariant that
must hold is **no message send before patch-up**, and the classic bug is a helper that sends
something innocuous during phase one: an equality test, a hash, a debug print, a
collection insert that compares keys. Each of those is a send, and a send during phase one
dereferences a class pointer that is not yet real.

**Trap.** "The tower terminates because `Metaclass` is its own class." In Smalltalk-80 it is a
two-cycle, not a self-loop: `Metaclass class class` is `Metaclass`. Python's `type` is the
self-loop. Getting the two systems' shapes swapped is the tell that the answer is recalled
rather than derived — and the derivation is the part that generalizes, because it tells you
that any uniform object model must close this relation somewhere and that where you close it
is a free design choice.

### A6 — `super` cannot mean "my superclass"

**1.** Receiver-anchored, `C.new.m()` starts lookup at `C`, finds `B`'s `m`. Inside it,
`super.m()` means "superclass of the receiver's class" — superclass of `C` is `B` — which
finds `B`'s `m` again, and again, until the stack overflows. `super` must be anchored to the
class in which the *executing method was defined*, with lookup starting at that class's
superclass. The essential point: the anchor is a property of the **code**, not of the call.

**2.** In a bytecode VM the defining class — or better, a direct pointer to the start-of-search
class — is baked into the method object or into the `SuperSend` instruction at compile time.
That is why super sends can be cached more aggressively than ordinary sends: the search start
does not vary with the receiver, so the only variable left is nothing at all, and the target
can be resolved once and cached unconditionally (modulo redefinition). Ruby cannot do that
statically, because a method defined in a module is *one method object* occupying different
positions in different classes' ancestor chains. So Ruby anchors on the method entry's
**owner** and searches the receiver's linearized ancestors starting *after* that owner's
position — the anchor is the pair (defining module, receiver's ancestry), resolved at call
time. Same idea; one is an absolute pointer, the other an index into a per-receiver list.

**3.** That resolution rule is the mechanism. A module method calling `super` works when the
module sits above something that defines the method in *that* linearization, and raises when
it does not — so a class that visibly defines `foo` may sit *before* the module in the
ancestor list rather than after it, especially with `prepend` in play, or the method's owner
may be somewhere unexpected because it was installed with `define_method` or inside an
`instance_eval`. The error is about a position in an ancestor list, not about a class
hierarchy, which is precisely why it reads as absurd until you print `ancestors` and see the
order.

### A7 — The MRO that cannot exist

**1.** C3 requires the linearization to preserve two things: **local precedence order** —
each class's own base list order is respected — and **monotonicity** — if `P` precedes `Q` in
any ancestor's linearization, `P` precedes `Q` in every descendant's. Here `A` demands X
before Y, `B` demands Y before X, and `C` inherits both constraints. No total order satisfies
both, so no linearization exists. The error is not a heuristic giving up; it is a proof of
non-existence, which is why the right response is to change the hierarchy rather than to look
for a flag.

**2.** Naive depth-first on the ordinary diamond `D(B, C)` with `B, C < A` visits B, then
climbs into A through B's chain, then reaches C — so `A` precedes `C`, and a method defined
in `A` shadows an override in `C`, a class `D` explicitly listed. That is the diamond bug in
its purest form: an override you asked for is silently skipped. Python treated it as worth
breaking compatibility over because with a universal root (`object`), *every* new-style
hierarchy is a diamond — the bug stopped being exotic and became the default. It is also a
precondition for cooperative `super()`: chained `super()` calls only visit each class once,
in an order all participants agree on, which requires a genuine linear order and not a
traversal.

**3.** For: dispatch order is a semantic contract, and a system that silently *invents* an
order when the programmer's stated constraints are contradictory has chosen to be quietly
wrong in a way no test will catch. Making it an error converts a future debugging session
into a class-definition-time failure with a printable explanation. Against: legality becomes
non-local. A hierarchy that works today can be broken by a third-party library reordering a
base list two levels away, and the error surfaces at the *combining* class — which may be
code you do not own and cannot fix without changing inheritance you do not control. The
strongest form of the counterargument is what Rust and Go did: have no implementation
inheritance at all, at which point C3 is the best possible answer to a question you should
not have asked.

### A8 — Three answers to the same pressure

**1.** Ruby: linearization decides — the last module included wins, silently, at include
time; you discover it by reading `ancestors`. `prepend` puts a module *ahead* of the class so
it can wrap the class's own method and call `super` into it. Scala: linearization too
(right-most trait wins), also silent, though the compiler rejects some cases where a trait's
`super` call has nothing to resolve against in a given mix order. Rust: no conflict exists at
definition time — both methods exist on the type — and the *call site* becomes ambiguous,
requiring `<T as Trait>::render(x)`. Error at use. Java: compile error at the implementing
class, which must override, optionally delegating with `Trait.super.render()`. Error at
declaration. The real axis is **order-resolves-silently** (Ruby, Scala) versus **must-
disambiguate-explicitly** (Rust, Java): one optimizes for composition working by default, the
other for the system never guessing.

**2.** State costs layout. A stateful mixin must contribute slots to every class that mixes
it in, so either offsets differ per mixing class — and then the mixin's own compiled code
cannot use a fixed offset, forcing an indirection or per-class specialization — or you
allocate the mixin's state as a separate object and pay a pointer hop on every access. It
also reintroduces the diamond problem *for data*: mix the same stateful trait in via two
paths and whether there is one field or two is a genuine semantic question. C++ virtual
inheritance exists solely to answer it, and its cost is that field access goes through a
vptr-mediated offset lookup. Forbidding state costs the user the most natural form of
encapsulation — they must declare the fields in every implementing class and expose whatever
the trait needs. Rust's shape is the tell: traits declare required *methods*, never fields,
so `fn value(&self) -> u32` stands where a mixin would simply have had a field.

**3.** Java's rule is that **a class method always beats an interface default method**,
regardless of declaration order or specificity. Ruby's `include` inserts the module into the
ancestor chain and a `prepend`ed module can beat the class outright. The Java rule protects
**source and binary compatibility for library evolution**: adding a default method to an
interface can never change the behaviour of a class that already had a method by that name.
That guarantee is the entire reason default methods could be added to `Collection` and
friends without breaking every implementation in the world, which was the feature's actual
purpose — the mixin-like expressiveness was a side effect.

**Trap.** Treating traits, mixins, and interfaces as three syntaxes for one feature. They
differ on two independent axes — whether state is permitted, and whether conflicts resolve by
order or must be resolved explicitly — and the four systems above occupy four different
corners. A candidate who says "traits are just interfaces with implementations" cannot then
explain why Rust has no linearization and Ruby has no ambiguity error.

### A9 — Reifying the miss

**1.** Possible: (a) **proxies and forwarding** — an object that implements any protocol by
delegating, without being told in advance what the protocol is, which is how remoting,
mocking, and decoration are built; (b) **generated APIs** where the method name is the data —
`find_by_name_and_email`, DSLs, dynamic accessors. And a third worth naming: genuinely good
failure handling, since the reified message carries selector and arguments, so a handler can
suggest a spelling, log the call, retry against a fallback, or re-dispatch. Impossible:
knowing statically what an object responds to. Every "does it respond to X" check becomes an
approximation, completion and refactoring tools degrade, and no whole-program analysis can
conclude that a call site is unreachable or that a method is dead.

**2.** Because "absent" is a *negative* fact about an entire chain, and it must be invalidated
by any definition anywhere in that chain — including on a class that did not exist when you
cached it. It is the same invalidation problem as positive caching with a strictly larger
trigger set. You *can* cache the miss handler, and real systems do maintain negative caches;
the point is that a negative entry has more dependencies than a positive one and buys far
less, because the handler it dispatches to is typically a single generic method that
immediately re-dispatches on a string or symbol. You have cached your way to the door of a
second, uncacheable lookup.

**3.** `__getattribute__` runs on *every* attribute access, so defining it disables the fast
path wholesale: CPython's specializing interpreter keys its attribute caches on the type
still having the default slot, and overriding it forces the generic path for all accesses on
that type, hits included. `__getattr__` runs only after normal lookup fails, so hits keep the
fast path and only misses pay. That asymmetry is why "use `__getattr__`, not
`__getattribute__`" is standard guidance — it is a performance rule wearing a style rule's
clothes. Ruby needs `respond_to_missing?` because `method_missing` alone breaks reflection:
`respond_to?` reports false for a method that in fact works, and `method(:x)` raises. That is
the general tax on reified misses — you must implement the *introspection* half by hand as
well, almost nobody does, and the result is that duck-typed checks against
`method_missing`-based objects are unreliable across the whole ecosystem.

**Trap.** "`method_missing` is a fallback, so it costs nothing unless you use it." Its mere
availability costs everyone: a miss can no longer be a statically decidable error, tooling
cannot know an object's protocol, and the runtime must keep a slow, general path reachable
from every send site. The cost is paid by programs that never define it.

### A10 — Open classes and the invalidation bill

**1.** (a) **Global version counter** — O(1) invalidate, invalidates everything, so a program
that defines methods steadily never reaches steady state. (b) **Per-class or per-shape
version words checked in the guard** — finer, but the guard now compares two things and you
must propagate invalidation to descendants. (c) **Dependency lists and code patching** —
compiled code registers its assumptions, redefinition walks the dependents and patches or
deoptimizes. Most precise, requires a deopt mechanism, and the metadata grows with the amount
of compiled code rather than with the source.

**2.** A **descendant list per class**, maintained at class-creation time, or an equivalent
downward-walkable class hierarchy structure. Cost at redefinition is proportional to the
transitive descendant count, which is why redefining a method on `Object` is pathological in
a deep hierarchy and why runtimes special-case it by degrading to "invalidate everything"
once the subtree exceeds a threshold. Note the symmetric obligation, which people forget: the
walk must also happen when a *new* subclass appears, because assumptions of the form "this
method has exactly one implementation" are falsified by class *loading*, not only by method
redefinition.

**3.** **Devirtualization followed by unguarded inlining**, and the second half is where the
win actually lives. If a method is provably the only implementation, the call becomes direct
and then the body is inlined, and the inlined body can be optimized with the caller's context
— constant propagation, dead branch elimination, escape analysis across what used to be a
call boundary. In an open world you can still inline speculatively, but every inline carries
a guard and a deopt path, which costs code size, blocks some optimizations across the guard,
and requires the entire deoptimization apparatus. HotSpot's class-hierarchy-analysis
devirtualization is exactly this bet: assume a single implementor, deoptimize when a second
one loads. Sealing forecloses monkey patching, runtime extension of third-party classes,
patch-based test doubles, and plugins that add behaviour to core types — which is to say it
forecloses Ruby, and languages like Wren that never offered class reopening did so precisely
to keep this closed-world assumption.

### A11 — Fields, offsets, and the fragile base class

**1.** Prefix layout means an instance of `Derived` *is* a valid `Base` instance for field
access, so a method compiled once against `Base` uses constant offsets and runs unmodified on
every subclass. That is what makes field access a single indexed load instead of a lookup,
and what makes inherited method code shareable rather than recompiled per subclass. It is the
same trick as struct prefixing in C and single-inheritance vtable prefixing in C++, and it is
why multiple implementation inheritance is hard: two prefixes cannot both start at offset
zero.

**2.** Every offset compiled into `Derived`'s methods for `c` shifts; every existing instance
has the wrong shape; every inline cache holding "field `c` lives at index 2" is now wrong;
subclass field-count metadata is stale; any serialized or externally-held layout breaks. The
famous case is **Objective-C's fragile base class problem** — adding an instance variable to
a framework class broke every already-compiled subclass, which meant Apple could not add
ivars to shipping classes. The modern 64-bit runtime fixed it with **non-fragile ivars**: the
compiler emits ivar offsets as *symbols* resolved at load time instead of as immediates, and
the runtime slides subclass ivars down during class realization. The cost is one extra load
per ivar access — an offset fetched from a global rather than a constant folded into the
instruction — which is exactly the sort of pervasive small tax you pay once to buy binary
compatibility forever.

**3.** Costs: a hash probe per access instead of an indexed load; a per-object dictionary with
its own header, table, and resize behaviour; and no shared layout, so no shape-based inline
caching unless you reinvent shapes. CPython claws it back with **key-sharing dictionaries** —
instances of one class share a key table and store only a values array, which is a hidden
class in all but name — plus `__slots__` as the explicit "give me a fixed layout" opt-out,
plus the specializing interpreter's per-instruction attribute caches keyed on type version
and dict-keys version. The direction of travel is the lesson: under performance pressure the
fully dynamic design converges on the static one, every time, and the only question is
whether the convergence is visible to the programmer or hidden in the runtime.

**Trap.** "`__slots__` is a memory optimization." It is a *layout* declaration; the memory
saving is downstream. Describing it as memory-only means missing that it also makes attribute
access a fixed-offset operation and removes the possibility of arbitrary attributes — which
is why adding `__slots__` can break code that was assigning attributes you did not declare.

### A12 — Shapes, transitions, and the cost of order

**1.** A shape is produced by a *sequence* of add-property transitions, and it encodes the
offsets that sequence assigned — `x` at 0 then `y` at 1, versus `y` at 0 then `x` at 1. So
A-objects and B-objects have different shapes and different offsets for the same names. A
site reading `o.x` that sees both goes polymorphic: it must hold two (shape → offset) pairs
and test both, which costs extra compares and inline-cache space, but more importantly stops
the optimizing compiler from folding the load to a single constant offset. The engine cannot
simply sort the property names to canonicalize, because property *enumeration order* is
observable in JS and is specified in terms of insertion order.

**2.** Sharing is essential because the entire value of a shape is that a shape *pointer* is a
cheap proxy for a layout. If every object carried its own descriptor, comparing descriptor
pointers would tell you nothing and you would be back to per-object lookup. So the engine
interns them: the tree's root is "empty object with this prototype", each edge is "add
property *name* with these attributes", and two objects that walked the same path arrive at
the identical node. Branching happens where two construction sequences diverge after a shared
prefix. The engineering consequence people actually use: **initialize every field in the
constructor, in a fixed order**, so all instances share one path and every site stays
monomorphic.

**3.** Because dictionary mode is a **representation change**, not a slowdown knob. Once the
object is a hash, its shape no longer describes offsets at all, so every inline cache
referencing it fails permanently, the optimizing compiler's assumptions are void, and what
was a load becomes a probe. There is no partial credit and — critically — no automatic path
back for that object. For benchmarking it means a micro-benchmark can be badly wrong in
either direction: a single `delete` in setup can push every object into dictionary mode so
you measure the slow path forever, and conversely a benchmark that only ever constructs
objects one way measures a monomorphic ideal your real program never reaches. Inspect the
shape, do not just read the clock.

**Trap.** "Hidden classes make JS as fast as Java." They make *property access* comparable
when shapes are stable and sites are monomorphic. They do nothing about the fact that the
shape is a speculation which arbitrary code can invalidate, that every access still carries a
guard, and that the compiler must retain a deoptimization path Java never needs. The
achievement is real and the equivalence is not.

### A13 — Interface dispatch is not virtual dispatch

**1.** Under single inheritance a subclass's vtable is a prefix-extension of its superclass's,
so the slot for `Base.m` sits at the same index in every subclass — a compile-time constant,
and one indexed load. An interface can be implemented by classes with entirely unrelated
vtable layouts, and one class can implement many interfaces, so no consistent index exists:
assigning one would require every implementor of `I` to reserve the same slot, which is
impossible to arrange across an open program compiled in pieces. So `invokeinterface` has to
*search* — locate the itable for the interface within the receiver's class metadata, then
index within that.

**2.** A Go interface value is a two-word pair: an **itab** pointer and a data pointer. The
itab records the interface type, the concrete type, and an array of function pointers for the
interface's methods in the interface's own order. Building it — matching the concrete type's
method set against the interface's method list — happens once, at the **conversion**, and
itabs are memoized in a global table keyed by (interface type, concrete type); when both
types are statically known the itab is emitted at compile time. So the call itself is: load
itab, load slot *i*, indirect call. The entire search cost was moved from the call site to the
conversion site, which is the design in one sentence.

**3.** Because inline caching subsumes the difference. The JIT guards on the receiver's
concrete class and then makes a direct — often inlined — call, and once you have that guard,
how you *would have* located the method is irrelevant. The residual costs are two. Interface
sites tend to be genuinely more polymorphic in practice (polymorphism is why the interface
exists at all), so they degrade to polymorphic or megamorphic more often, and that is where
the extra indirection reappears. And class-hierarchy analysis is weaker for interfaces:
proving a single implementor is harder when implementors share no superclass, so the
unguarded-devirtualization win from A10 fires less often.

### A14 — What the receiver is bound to, and when

**1.** JS: `this` is an **implicit parameter supplied by the call form**. `obj.method()` is a
call whose reference base is `obj`, so `obj` is passed; `f()` has no base, so `undefined`
(strict) or the global object (sloppy) is passed. The function value carries no receiver at
all. Python: attribute access on an instance goes through the **descriptor protocol** —
functions are descriptors, so `obj.method` evaluates `function.__get__(obj, type)` and
returns a *bound method object* holding (function, instance). The receiver is attached at
attribute-access time, so the resulting value is self-contained. Smalltalk sidesteps the
question: there is no expression that extracts "a method" as a callable value from a send;
reflection can hand you a `CompiledMethod`, but invoking it requires supplying a receiver
explicitly.

**2.** It allocates a bound-method object per attribute access — one short-lived heap object
per `obj.method(...)` call, immediately garbage. CPython avoids it with a **`LOAD_METHOD` /
`CALL_METHOD` pair**: `LOAD_METHOD` detects the common case (a plain function found on the
type and not shadowed in the instance dict) and pushes the unbound function and the receiver
as two separate stack entries, so the call can prepend the receiver directly with no wrapper
object. Anything unusual falls back to the general path. Structurally that is an inline
cache: a fast path admitted by a cheap check, with a general path behind it.

**3.** Because call-form-determined `this` means every callback loses the receiver —
`arr.map(this.f)`, `setTimeout(this.tick)`, an event handler, a promise continuation. Arrow
functions have no `this` of their own; they capture the enclosing lexical `this` like an
ordinary variable, converting a dynamic binding into a lexical one. `bind` was insufficient
for three reasons: it allocates a fresh function object per call, so you cannot
`removeEventListener` a bound handler unless you stored it; it must be applied at every
capture site, so it is verbose and easy to omit; and the idiom it competed with — `var self =
this` — was already direct evidence that programmers wanted lexical capture rather than a
rebinding operation.

**Trap.** "Python binds `self` at call time." It binds at **attribute access** time, which is
why `f = obj.method` then `f()` works and why the bound method keeps `obj` alive. Saying
"call time" makes the CPython `LOAD_METHOD` optimization incomprehensible — there would be
nothing to optimize away — and it makes the JS difference sound like a detail rather than a
different location in the pipeline.

### A15 — When the cache gives up

**1.** Monomorphic: one (class → target) pair stored inline at the site; compare one pointer,
then jump. Polymorphic: a short linear array of (class → target) pairs, compared in sequence
until a hit, usually laid out as a small stub the site jumps into. Megamorphic: the site stops
recording types entirely and calls a shared routine that probes a **global hash table keyed
on (class, selector)** for the resolved method.

**2.** Because the polymorphic check is a *linear scan on the hot path*: its expected cost
grows with the number of entries while its marginal hit-rate improvement flattens. Past a
handful of entries the expected number of compares exceeds one hash probe into the global
cache — and worse, the branch pattern becomes unpredictable, so you add mispredictions on top
of the compares. There is a second reason that is really about the optimizer: a site with two
or three targets can be inlined as a guarded polymorphic inline cache with real bodies, and a
site with twenty cannot be inlined at all, so retaining the entries buys nothing downstream.
The cap sits exactly where "a short, predictable sequence of compares" stops being true.

**3.** It is a direct-mapped or low-associativity hash table indexed by combining the class
pointer with the selector, holding the resolved method. It is correct because it caches a
*pure function* — (class, selector) → method — plus an invalidation hook that must flush
entries whenever a definition could change that function's value anywhere. Direct-mapped
means a collision simply evicts, which is always safe because the fallback is to recompute.
The one non-negotiable detail: each entry must store its key, because the index is lossy —
you are checking a guess, not trusting a lookup.

### A16 — Private is a convention, and that is a design decision

**1.** It protects against **accidental name collision between a class and its subclasses**,
not against access. `self.__cache` in `Base` and `self.__cache` in `Derived` become
`_Base__cache` and `_Derived__cache`, so a subclass author who picks a private-looking name
cannot silently clobber a base class's internal state. That is a real and valuable property —
it is the same problem hygiene solves for macros — and it is entirely orthogonal to keeping
callers out, since writing the mangled name works fine and is documented. Describing mangling
as "weak privacy" is grading it against a goal it never had.

**2.** (a) Serialization, ORMs, and pickling — reconstructing an object's state without going
through its constructor. (b) Debuggers, REPL inspection, and test doubles that need to observe
or replace internals. (c) Framework machinery: dependency injection, mocking, proxying, and
anything built on `method_missing`/`__getattr__` that forwards a protocol it does not own.
Migration tooling and hot reload belong on the list too. Java is the instructive case: it
enforced `private` in the verifier, then let reflection defeat it because the ecosystem
needed all of the above, then spent the module system trying to partially un-defeat it —
which is honest evidence that enforcement and ecosystem are in genuine tension rather than
one being simply correct.

**3.** It requires the send to carry the caller's identity: either a distinct instruction — a
`SendPrivate` resolved only against the defining class, with that class baked in exactly like
a super send — or a selector namespaced by the defining class, so a private `#reset` is
really `#Foo::reset` and is unforgeable from outside. Both are implementable and both are
*fast*, because resolution is static. What breaks: private methods stop being usefully
overridable, since a subclass's `reset` is a different selector and a base method calling
`reset` will never reach it — that is exactly C++'s non-virtual private semantics, and it is
surprising in a dynamic language where people expect overriding to work. Reflective invocation
then needs an escape hatch that reopens the hole. And `respond_to?` / `doesNotUnderstand` now
need a notion of "absent from here but present from there", so a miss is no longer a property
of the receiver alone but of the (receiver, caller) pair — which is a real complication to a
core protocol in exchange for a guarantee most dynamic languages decided they did not want.

### A17 — Ship the closed version first

**1.** Lookup must already begin at something the *object* owns, rather than at a class the
object merely names. Concretely: both the guard and the start-of-lookup must go through one
indirection stored in the object — call it its shape or class pointer — and nothing anywhere
may assume that two objects of the same *declared* class share a lookup start. This matters
because the failure is silent: if v1's inline cache guard compares declared class, then in v2
two objects with singleton methods compare equal on that guard while having different method
tables, so a call site returns the wrong method with no error. In v1 the extra indirection
costs nothing — the pointer always points at the declared class — and it is the entire
migration.

**2.** Reflection must never promise that "the class I am an instance of" and "where lookup
begins" are the same thing. If v1 exposes a single accessor conflating them, v2 must either
lie — Ruby's `class` deliberately skips singleton classes, and that lie is now permanent — or
break code. Ship the two concepts as two separate accessors from day one, even though in v1
they always return the same object. The trap is that in v1 they *are* indistinguishable, so no
test you can write will catch the conflation; the only defence is deciding it on paper before
there is any evidence.

**3.** **Class-population growth and the cache pressure it creates.** Every object with a
singleton method acquires its own class, so the class table, per-class metadata, method
caches, and every "walk all descendants" invalidation structure now scale with the number of
*specialized objects* rather than with the number of declared classes. Systems that use
singleton classes casually — Ruby creates one for every `def self.method` and every `extend`
— accumulate a large population of one-instance classes, which makes global method caches
thrash and makes descendant-walking invalidation measurably more expensive. It appears in no
specification, it is invisible in small programs, and it is why engines special-case objects
whose singleton class is still empty so they can keep pointing at the shared class.

**Trap.** "Adding singleton methods later is just adding a per-object method dictionary." The
dictionary is the easy half. The hard half is that every cache guard, every reflective
accessor, and every invalidation walk in the system was written against the assumption that
behaviour is a function of the declared class — and that assumption is not written down
anywhere, because in v1 it was true by construction.
