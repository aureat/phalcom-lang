# 03 — The object model and the metaclass tower

> **Thesis:** "classes are objects" is not a slogan, it is a load-bearing structure with a
> termination problem. Solve the termination problem correctly and class-side inheritance,
> constructor dispatch, and reflection all fall out as consequences. Solve it approximately and
> you get a system where constructors *almost* inherit, and every bug looks like a different bug.

---

## 1. The regress, and the standard escape

If every object has a class, and a class is an object, then a class has a class. Call it the
metaclass. The metaclass is an object, so it has a class too. Left alone, this does not
terminate.

**`[R]`** The Smalltalk-80 answer — and the one Cointe's ObjVlisp model (1987) formalized — is to
close the loop rather than extend it: the tower ascends for exactly two levels and then bites its
own tail. The canonical simplification, repeated in most secondary sources, is "`Metaclass` is an
instance of itself."

**`[V]`** In Phalcom, that simplification is *false*, and live probing established it:
`Metaclass.class` is a **distinct object** named `"Metaclass class"`. Closure is two-step —
`Metaclass.class.class == Metaclass` holds, while `Metaclass.class == Metaclass` does **not**.
The bootstrap allocates an eight-row apex: `Object`, `Behavior`, `Class`, `Metaclass`, plus each
one's metaclass row.

This correction is worth dwelling on for two reasons. First, an earlier observation in this
project's own memory asserted the self-loop version as fact, and it took a live probe to
overturn it — the simplification is sticky because it appears in nearly every explanation of
metaclasses. Second, the two-step closure is not an implementation detail: it is what makes the
metaclass rows ordinary participants in the inheritance rule below, rather than a special case
that has to be excluded from it.

---

## 2. The parallel rule, and why it *is* constructor dispatch

**`[V]`** ADR-0002 states the invariant:

> `(X class).super == (X.super) class`, with the root case `(Object class).super == Class`

Read it as a picture. There are two ladders — the instance-side chain (`Dog → Animal → Object`)
and the class-side chain (`Dog class → Animal class → Object class → Class`) — and the rule says
they are *parallel*. Every rung on one has a matching rung on the other, in the same order.

The consequence is the entire payoff: **class-side method lookup walks a real chain, so class-side
methods inherit exactly like instance-side methods do.** A constructor declared on `Animal class`
is found from `Dog` by ordinary lookup. No special case, no separate constructor resolution
algorithm, no "static methods are not inherited" caveat.

**`[V]`** The overlay states the consequence in the strongest possible terms:

> This rule **is** constructor dispatch (ADR-0063) — every constructor bug to date came from
> opting out of it.

**`[V]`** And the failure history bears that out in a way worth reading as a sequence, because
the diagnosis moved twice before it landed:

1. Every metaclass's superclass was found wired to `Class` instead of `(X.superclass) class` —
   which silently breaks class-side inheritance, since the chain simply ends early.
2. A follow-up found the bootstrap *did* enforce the rule, with an invariant test.
3. A third pass found `create_class` already set `metaclass_superclass = superclass.class`. The
   real defect was upstream: **the compiler was pushing `Object` unconditionally** as the
   superclass, so the rule was being applied faithfully to wrong input.

**The transferable lesson is about debugging, not metaclasses.** An invariant that holds in the
constructor can still be violated in the finished object if the constructor is called with wrong
arguments. Two rounds of investigation localized the fault to the layer where the *symptom*
appeared. The rule is: when a structural invariant is enforced *and* violated, suspect the inputs
before the enforcement.

**`[V]`** A related and equally instructive failure from the same area: `ClassDef` had no
superclass field at all, so `class Dog : Animal` parsed and compiled to `Object`, and `super`
parsed successfully but compiled to `Bytecode::Nil` **with no diagnostic**. The specification was
complete — single inheritance, chain-walking lookup, `super` starting at the defining class's
superclass, all written down — while the feature was a total no-op. *Silent `Nil` lowering is the
worst failure mode a compiler has*, because it produces a running program that is wrong, and
nothing in the pipeline is in a position to notice.

---

## 3. Bootstrapping a cyclic structure without unsafe code

**`[V]`** The apex is genuinely cyclic: `Object`'s metaclass inherits from `Class`, which is
itself an object with a metaclass. Rust's ownership model has no comfortable answer for building a
cycle of owned values, and the ergonomic escapes (`Rc<RefCell<…>>`, `new_cyclic`, raw pointers)
each carry a real cost — reference cycles leak, `RefCell` introduces a runtime borrow-panic
surface, raw pointers introduce `unsafe`.

Phalcom's answer is **allocate-then-wire**. Every class is allocated first with placeholder links;
then the links are patched. This is possible only because a class reference is a `Copy` arena
handle (`ClassId`, `ObjRef`) rather than an owning pointer — so "patching a link" is writing an
integer into a slot, and a cycle in the object graph is not a cycle in the ownership graph at all.

**`[V]`** The overlay records the consequence precisely: the cyclic apex is "expressed as handle
patches, not `new_cyclic`," and the memory-safety posture is "**no `unsafe` for the object
graph**." The hazards this dodges — `Rc<RefCell>` cycle-leak and double-borrow panic — are listed
as *removed by construction*.

**The generalization**, and it is one of the most reusable ideas in this codebase: **indirection
through an arena converts a graph problem into an indexing problem.** Cycles, back-edges, and
self-reference stop being ownership questions. The costs are real and should be stated —
dereferencing is a bounds-checked table lookup rather than a pointer chase, and the arena must
be threaded through every function that touches an object — but the correctness properties are
overwhelming for a language runtime, where the object graph is arbitrary by definition.

**`[V]`** The correctness of the wired result is not assumed: a `verify_invariants()` guard runs
over the bootstrapped tower, and the invariant suite is the authoritative census of what must
hold.

---

## 4. Where the shared behavior lives

**`[V]`** ADR-0003 introduces `Behavior` as an abstract superclass of both `Class` and
`Metaclass`, sitting under `Object`. The method dictionary, the lookup protocol, and the
allocation protocol live there, in one place, rather than being duplicated across the two kinds
of class-like object.

This is a small decision with an outsized effect on the reflective surface. Because both `Class`
and `Metaclass` are `Behavior`s, reflective operations — enumerate methods, ask for a name, look
up a selector — are written once and work uniformly on both sides of the tower. **`[R]`** In
Smalltalk terms, this is the same move that makes the metaclass tower a genuine metaobject
protocol rather than a curiosity: the machinery that implements classes is itself expressed in
the object model, and is therefore programmable.

---

## 5. Class-side state: the naming trap

**`[V]`** Class-side stored fields are implemented as `ClassObject.static_slots`, indexed by a
per-*metaclass* field table — literally ADR-0011's instance slot vector shifted one level up the
tower. No new storage primitive, no new absence path, one mechanism instead of two.

The semantics that follow are sharp and were **ratified as correct rather than fixed**:
subclasses receive *fresh, unset* slots for an inherited class-side field declaration. Measured:
`Base.count` reads 2 while `Derived.count` reads `None`, when read through an inherited class-side
method. Re-running the initializer per subclass was explicitly rejected, because it would diverge
from instance fields, which read `None` until written.

**`[R]`** The naming argument is the transferable part. This is a Smalltalk **class-instance
variable**, not a class variable. A class variable is shared across the hierarchy; a
class-instance variable is per-class state that each subclass gets its own copy of. Java and C#
`static` carries the sharing connotation, which is why `@static`, `@shared`, and `@classvar` were
all rejected as names — each would teach the wrong model, and users would then write code that is
correct only under the model the name implies.

**The general point:** when a mechanism's name imports semantics from another language, users
inherit that language's mental model along with the keyword — including the parts that do not
apply. Naming is not decoration; it is the primary documentation channel, and it is read by
people who will never open the specification.

---

## 6. Openness, and the reversal that priced it correctly

**`[V]`** The hierarchy has two mutability axes, and they were resolved differently:

- **Superclass: sealed** at definition. Reassignment raises. This deletes an entire
  cache-invalidation case — a future inline cache keys on `ClassId` with **no
  invalidate-on-reparent case at all**.
- **Methods: open** — redefinable at runtime, with the override-epoch guard forcing deopt of any
  speculatively inlined site.

**`[V]`** Class *reopening* was then reversed by a later decision, and the reasoning is the most
interesting thing in this file. The earlier ruling had priced reopening as free — correctly, as
far as it went, because the *dispatch* cost was already paid by the override-epoch guard. What it
never priced was the **namespace** cost, which turned out to be larger. Two silent-bug classes:

1. **Cross-module collision.** The class registry (`classes`, `field_layouts`) is a set of VM-wide
   maps keyed by `Symbol` alone. Two modules each declaring `class Point` therefore *merge into
   one class*, with import order as the tiebreaker. Bindings are per-module; class identity is
   not.
2. **Partial overwrite.** `add_method` is last-writer-wins, and a displaced method — **including a
   Rust native** — simply becomes unreachable, with no super-to-previous mechanism to recover it.

**`[V]`** And the audit finding that made removal cheap: `core.ph` never actually reopens a class.
The twenty-odd apparent reopenings are *stub completions* through a distinct code path. A feature
believed to be load-bearing was, on inspection, used zero times.

**Two lessons.** First, **price every axis a feature touches, not the one you happen to be
thinking about**; the dispatch analysis was rigorous and complete, and it was the wrong analysis.
Second, before defending a feature's cost, **count its uses** — the argument frequently
evaporates.

**`[V]`** A related trap worth its own note: sealing is not closure. A live probe showed
`@sealed` prevents `extends` but not reopening. Reopening `class None` adds the method to the
`None` *class object*, while the `None` global is an immediate value. Bootstrap must therefore
protect the value binding if real `None` members are ever introduced.

---

## 7. Constructors need no machinery

**`[V]`** ADR-0063: a constructor is an **ordinary class-side method**. It compiles to a static
method on the metaclass carrying a normal `SignatureKind::Method(arity)` selector, and it shadows
the inherited bare allocator `Class >> new()` through ordinary metaclass-tower lookup. The special
`SignatureKind::Initializer` survives only as a flag gating super-constructor hops.

That is the payoff of section 2 stated as a design result: **if your metaclass tower is real,
constructors require no dispatch machinery of their own.** Languages that bolt constructors on as
a separate concept end up with a separate resolution algorithm, separate inheritance rules, and a
standing supply of bugs at the seam between the two systems.

**`[V]`** One genuine sharp edge remains, and it is a good illustration of the limits of
syntactic guards. `Factory.new()` with a wrong arity is rejected at compile time; the identical
call through `C.new()` where `C = Factory` is silently accepted and returns an instance with
`None` fields. A compile-time guard keyed on a *literal* receiver leaks the moment a class flows
through a variable — which, in a language where classes are first-class values, is always
possible. Syntactic guards are real defenses with a precisely describable hole, and the hole
should be documented rather than discovered.

---

## 8. Inheritance: one slot, and what that forces

**`[V]`** Single inheritance only. `Object` is the root; every class has exactly one superclass.
Full multiple inheritance with a linearization was rejected outright; stateless traits flattened
at finalization are the *only* pre-approved future extension.

**`[V]`** The consequence surfaces immediately in library design. A `File` wants to be a Resource,
a Reader, a Writer, and Seekable, and has one slot. The ruling: **closeability earns the slot**,
because two language mechanisms (leak reporting and resource-table cleanup) must interrogate the
type *at runtime*. Reader, Writer, and Seekable stay informal duck-typed protocols, because code
just sends the message and never needs to introspect.

**The decision rule generalizes cleanly, and is better than the usual "is-a versus has-a"
heuristic:** *the inheritance slot goes to the capability that something other than the call site
needs to detect.* If a capability is only ever exercised by sending a message, duck typing
suffices. If a runtime mechanism must ask "is this one of those?", it needs a reified type
relationship. **`[V]`** The internal precedent cited is `Iterable` — a kernel root, while
iteration mechanics stay informal.

---

## 9. The reflective surface, and its one missing piece

**`[V]`** A failed send is reified as a first-class `Message` carrying `selector`, `name`,
`labels`, and `args`, and forwarded to a user-overridable `doesNotUnderstand(_)`. That gives
proxies, DSLs, and `respondsTo` essentially for free — the standard Smalltalk dividend.

**`[V]`** Except it does not, quite, and the gap is precise: `perform` accepts only a `Symbol`.
A handler can therefore *observe* an intercepted call but cannot *forward* it. Reification
without re-dispatch blocks the entire proxy pattern — the one use case the mechanism exists for.

**The rule:** *any language adding `doesNotUnderstand` should ship `perform(Message)` in the same
change.* Interception and re-dispatch are two halves of one feature, and shipping the observable
half first produces a mechanism that demos beautifully and cannot be used.
