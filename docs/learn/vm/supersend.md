# `SuperSend` — the static send that is resolved by name

*Pays the forward pointers in [Doc 1](execution-loop.md) (the opcode table's `SuperSend` row),
[Doc 4](message-send.md) (§"SuperSend — its own opcode"), and [Doc 5](caches-and-fusion.md)
(§"`SuperSend` is uncached"). The ordinary resolve loop, `doesNotUnderstand` forwarding, inline
caches and fusion belong to Docs 4 and 5 and are used here as known words.*

---

## The grip

> **`super` is the least dynamic thing a Phalcom program can write, and it is resolved dynamically,
> by name, on every single call.**
>
> The compiler knows exactly which class body each `super` sits in. It is the one send whose lookup
> start owes nothing to the runtime receiver. And yet what gets baked into the instruction is a
> **name symbol** — not a method pointer, not a class handle — which the VM resolves through a hash
> map on every dispatch, reads a `superclass` field from, and then walks uncached.
>
> Three separate decisions each converted a fact the compiler already had into a lookup the VM
> redoes forever. Every one of them is individually correct. Nobody priced the sum.

Doc 5 called `SuperSend` "a statically-known target." That is true of the *name* and false of
everything else, and the gap between those two readings is this document.

---

## Predict before you read

Three classes. `C` does not define `greet` at all — it inherits `B`'s.

```phalcom
class A {
  construct new() { }
  greet => "A-greet"
}
class B extends A {
  construct new() { }
  greet => super.greet + "-B-greet"
}
class C extends B {
  construct new() { }
}
System.print(C.new().greet)
```

`C.new().greet` finds no `greet` on `C`, climbs to `B`, and runs B's method — with the receiver
being a **C** instance. Now B's body hits `super.greet`. Where does that lookup start?

There are two plausible rules and they are hard to tell apart:

- **"above the receiver's class"** — receiver is a `C`, so start at `C.superclass` = **B**. Finds
  `B.greet`. Which calls `super.greet`. Which starts at `C.superclass` = B again. **Infinite loop.**
- **"above the class whose body lexically contains this call"** — that class is `B`, so start at
  `B.superclass` = **A**. Finds `A.greet`. Terminates.

Phalcom takes the second, and prints:

```
A-greet-B-greet
```

The two rules coincide *exactly* when the receiver's class is the same as the class that defines the
running method — which is every two-level example anyone writes first. `B.new().greet` gives the
right answer under **both** rules. The divergence needs an inheritance gap: a method reached through
inheritance, running on a receiver further down the chain than the class that wrote it.

This is the classic `super` bug, and Phalcom does not have it. **It also has no test for it.** Of the
four fixtures that exercise `super`, the closest — `inheritance_super_skips_middle.ph` — puts the
`super` call *in the leaf class itself*, which is the case both rules agree on. The program above was
written for this document; nothing in the committed corpus constructs it. The single most classic
failure mode of the feature is uncovered, and the code is correct anyway.

---

## What is actually in the instruction

```rust
// bytecode.rs:138
SuperSend(u8, u16, u16),
```

Three operands: argument count, constant-pool index of the selector, and constant-pool index of
**the defining class's name**. Not its superclass — and not a class handle. The reason is stated at
`bytecode.rs:119-123`:

> The class object does not exist at compile time, so its *name* is baked and resolved to a class
> handle at dispatch (the same global lookup as `GetGlobal`).

That is the constraint everything else follows from. A class in Phalcom comes into being by
*executing* its declaration; while the compiler is walking a method body, the class that body belongs
to has no runtime identity to point at. So the call site cannot carry the thing it wants to carry,
and carries its name instead.

Here is the whole dispatch, edited only for length (`dispatch.rs:851-922`):

```rust
let closure_module = self.heap.closure(closure_id).module;
let defining_key = ClassKey { module: closure_module, name: defining_sym };
let parent = if let Some(&c) = self.classes.get(&defining_key) {
    self.heap.class(c).superclass                      // own module
} else if /* … resolve the core module … */ {
    let core_key = ClassKey { module: core_mod, name: defining_sym };
    self.classes.get(&core_key).and_then(|&c| self.heap.class(c).superclass)   // then core
} else { None };

let effective_selector = self.constructor_aliases.get(&(defining_key, selector_sym))
    .copied().unwrap_or(selector_sym);
let mut method = parent.and_then(|p| lookup_method_in_hierarchy(&self.heap, p, effective_selector));
```

Per `super`, per execution: two constant-pool reads, a `ClassKey` construction, **up to two hash
lookups** (own module, then core), a `superclass` field read, a `constructor_aliases` map probe, and
then the ordinary uncached chain walk from Doc 4. For the send whose target the compiler knew all
along.

The receiver never moves. Only the lookup *start* does — that is the entire semantic content of the
opcode, and it is why an overridden method can run its parent's definition against the same instance.

---

## Three decisions, each trading a static fact for a lookup

**1. Bake the name, because the class does not exist yet.** Forced by the execution model, not
chosen. But note what it costs beyond speed: the binding is *late*, so a `super` resolves against
whatever class currently answers to that name in that module.

**2. Bake the defining class, not its superclass** — DEC-INH-B, and this one *is* a choice.
ADR-0040 rejects baking the superclass directly because it "would go stale under a runtime
`superclass=` and does not match 'superclass of the defining class' literally." So the VM reads
`defining.superclass` at dispatch rather than at compile time, and stays correct under a mutation
feature — U13 — **that does not exist at HEAD**. A real per-dispatch indirection, paid on every
`super` in every program, to keep a hypothetical honest. That is a defensible bet, and it is a bet.

**3. Do not cache it** — DEC-INH-F. The stated reason is precise and worth quoting, because it is
the opposite of the intuition:

> Unlike `Invoke`, the start class is a static per-call-site constant, so a `SuperSend` cache key
> differs from a receiver-polymorphic one; this first cut is uncached.

The send with the *most* cacheable shape is the one that gets no cache, because the existing cache
was built for a different shape. A `SuperSend` site has no receiver-class variation at all — its
resolution could be computed once and never rechecked except on hierarchy mutation. The inline cache
that exists is keyed on the receiver's class, so `SuperSend` does not fit it, and rather than build a
second mechanism the first cut does the full lookup forever.

*(On the decision IDs: `bytecode.rs` cites **DEC-INH-F**, TRACKER and Doc 5 cite **DEC-IC-B**. These
are not in conflict — DEC-INH-F is U-INH's original "first cut is uncached"; DEC-IC-B is U-IC's
still-open question of whether to add one. I nearly wrote this up as a citation error.)*

---

## The name is module-scoped, and that took a bug to get right

`ClassKey` is `{ module: ObjRef, name: Symbol }` (`vm/mod.rs:36-42`). Two modules that each declare
`class Point` get **distinct** `ClassId`s. Its own doc comment records that this was not always so:
class bindings were always per-module, but class *identity* used to be keyed by bare name VM-wide,
"so two modules declaring the same class name silently collapsed into one class."

The `SuperSend` arm resolves **own module first, then core**. That fallback is deliberate — a
superclass name is a bare identifier with no qualified form, so own-module-then-core is the complete
resolution space, not a heuristic. And because own-module always wins, neither direction can hijack
the other.

But there are *two* kinds of by-name class lookup in this compiler and they are not
interchangeable. A lookup for **a reference** (find the class this name denotes) takes the core
fallback. A lookup for **self** (is *this* class, the one being declared, sealed?) must not — and
`14cdfb9` exists because one of them did:

> `fix(compiler): U-CLASSNS sealed-check self-lookup must not fall back to core`

A non-core module's own class sharing a kernel name — `Option`, say — inherited the kernel class's
sealed status by name collision alone, letting `@variant` bypass the check. Same map, same key type,
same-looking call; opposite correct answers. `SuperSend`'s use is the reference kind, so its fallback
is right.

---

## Super-construct, and a bug Phalcom does not have

Constructors are installed on the **metaclass**, not the instance side. So `super.new(x)` misses on
the instance walk and needs a second chance (`dispatch.rs:891-915`): re-target the superclass's
metaclass and retry the same selector — gated so that only a method whose kind is
`SignatureKind::Initializer` may be reached this way. Without the gate, a parent *static* method
sharing the selector would capture the send and run against an instance receiver it was never
written for.

The other half is allocation. A subclass constructor allocates the instance in its prologue; when it
then calls `super.new(x)`, the parent's initializer must fill the **inherited slots of that same
object**, not build a second one. `NewInstance` is therefore idempotent (`dispatch.rs:996-1020`): if
its operand is already an instance rather than a class, it pushes it straight back.

This is worth dwelling on because a blind design review of this feature — an agent given the problem
and no access to this repository — named non-idempotent `super.new` as one of the two most likely
bugs a competent implementation would ship, and described the symptom precisely: two objects where
the program believes there is one, the outer caller holding an instance whose parent-level fields
were written to an orphan. Phalcom has the fix. Verified on a three-level constructor chain: all
three fields survive independently, one allocation.

The same review ranked **first** a different bug: that field-slot resolution would conflate the
home class with the runtime class, the field-storage sibling of the `super` bug in the
predict-before-you-read section. Phalcom does not have that one either — field access resolves
against `self.current_class`, the class being compiled (`compiler/lib/expr.rs:254-278`), and
`ClassLayout` states the rule outright: fields are non-inherited, "no superclass merge."

Two independent predictions of what should break here, and the answer to both is *the implementation
already got this right*. Which makes the one thing that did go wrong more interesting, because it is
not in either list.

---

## The trap: a diagnostic whose advice creates a bug

Fields are private to their declaring class. `classes.md:125`: **"Fields are private to the declaring
class and not inherited-visible."** `classes.md:172`: **"A subclass gets its own fresh slot."** Both
rules deliberate, both stated.

Now read an inherited field from a subclass:

```phalcom
class Base {
  construct new(x) { _x = x }
}
class Derived extends Base {
  construct new(x) { super.new(x) }
  peek => _x
}
```
```
Error: Read-before-write: field '_x' is used before being assigned anywhere in this class.
```

That is not a privacy error. It is a **flow-analysis** error (`compiler/lib/error.rs:101`), and it is
the only diagnostic that can fire here — no field-privacy-specific error variant exists in
`CompilerError` at all. The analysis is genuinely per-class: `expr.rs` fetches the layout for
`self.current_class` alone and raises if the name is absent.

So the message tells you the field was never assigned. Its implied remedy is to assign it. Do that:

```phalcom
class Base    { construct new(x) { _x = x }              baseSees    => _x }
class Derived extends Base {
  construct new(x) { super.new(x); _x = 999 }            derivedSees => _x }

const d = Derived.new(7)
System.print("Base sees:    " + d.baseSees.toString)
System.print("Derived sees: " + d.derivedSees.toString)
```
```
Base sees:    7
Derived sees: 999
```

**One object, one field name, two values.** Every individual step is spec-correct — the subclass got
its own fresh slot exactly as `classes.md:172` promises. The defect is the path: a privacy violation
reported as a missing assignment, whose suggested fix silently produces field shadowing. And it sits
directly against super-construct, whose entire job is filling the parent slot that the subclass then
cannot read.

Two things sharpen it. First, **no test covers the diagnostic at all** — `ReadBeforeWrite` has zero
hits anywhere under `tests/`. Second, and more pointed: there *is* a passing fixture for the two-slot
outcome — `inheritance_super_construct_same_field.ph` — which presents it as an intentional feature,
"fields stack, never alias." It is right that this is a feature when a subclass deliberately declares
a same-named field. Nothing anywhere records that the same mechanism, reached through the
`ReadBeforeWrite` message, is a trap. The corpus documents the destination and not the road.

Filed as [E006](../../errors/E006-inherited-field-diagnostic-shadowing.md).

---

## What the tooling cannot show you

Every other doc in this course that needed to show bytecode used the same workaround: hoist the
construct to module level, where `disasm` — which walks only the top-level chunk — can see it.

That workaround is **structurally unavailable here**. `super` outside a method is a compile error, by
design:

```
Error: `super` cannot be used outside a method: there is no defining class to start the lookup above.
```

So the one place `SuperSend` can legally appear is the one place the disassembler does not look. A
working `super` program disassembles with no `SuperSend` anywhere in the output. The opcode this
document is about cannot be displayed by the only tool that displays opcodes — which is why every
runtime claim above is a source citation or a program's output, and none is a disassembly.

---

## The design space

The blind review enumerated four things a call site could bake, and the ranking is instructive
because Phalcom's choice comes third:

- **A direct method pointer** — fastest, and impossible here: the class does not exist at compile
  time, and it would foreclose any later redefinition.
- **A forward cell for the class** — allocate a stub when the class body starts compiling, patch it
  once when the class object is built. Dispatch reads `cell.value.superclass`: one indirection, no
  hashing, no name.
- **The class name** — what Phalcom does. Always available, needs no new machinery, and pays a map
  lookup forever. Also semantically looser: it resolves against whatever answers to that name at
  dispatch time.
- **A `home_class` field on the compiled method itself**, set at *install* time — after the class
  exists — with the instruction carrying only the selector. The review picked this one, on the
  grounds that install time, not authoring time, is the moment that actually determines which
  hierarchy a method's `super` should walk.

ADR-0040's `## Alternatives considered` weighs four branches — amend `Invoke`, bake the superclass,
dynamic-receiver-class super, implicit constructor chaining — and **none of them is about what to
bake**. The name is treated as the obvious encoding rather than as a choice with alternatives. That
is not a criticism of the decision, which is fine and cheap to change; it is an observation that the
option space recorded in the ADR is narrower than the option space that exists, and the doc that
inherits it should say so.

Comparison worth keeping: **Smalltalk**, where `super` is likewise resolved against the compiled
method's own class rather than the receiver's, and where the same "wrong rule loops forever" hazard
is folklore. Cut: languages where `super` is a static link resolved at compile time (Java, C++) —
they never face the "class does not exist yet" constraint that produces this entire design, so
comparing to them smuggles in a world where the problem is absent.

---

## What you can now re-derive

1. Why a `super` call site cannot carry a method pointer or a class handle, from when classes come
   into existence.
2. Why the lookup start must be the *lexically defining* class, and what specifically breaks when it
   is the receiver's — three levels, one inherited method, an infinite loop.
3. Why `SuperSend`, the most statically-determined send in the language, is the only one with no
   cache — and why "its key shape differs" is the whole reason.
4. Why `super.new` needs a metaclass retry and an idempotent allocator, and what two objects instead
   of one would look like.
5. Why reading an inherited field tells you it was never assigned, and why doing what it says gives
   you two of them.

---

## Anchors

| Claim | Where | Verified by |
|---|---|---|
| Three operands; operand 2 is the defining class's **name** | `bytecode.rs:119-138` | quoted |
| Name is baked because the class does not exist at compile time | `bytecode.rs:121-123` | quoted |
| Dispatch: `ClassKey`, own module then core, then chain walk | `dispatch.rs:851-889` | quoted |
| `ClassKey` is `{module, name}`; two modules get distinct `ClassId`s | `vm/mod.rs:36-42`, `vm/api.rs:69,79` | quoted |
| Reference lookups take the core fallback; self-checks must not | `14cdfb9` | commit subject + spec §4.2 |
| Defining class baked for a `superclass=` feature that does not exist | `bytecode.rs:125-127`, ADR-0040 | quoted |
| Uncached by DEC-INH-F; DEC-IC-B is the later open question | `bytecode.rs:136`, `U-IC/plan.md:109` | both read |
| Lookup starts above the *defining* class, not the receiver's | — | program run: `A-greet-B-greet`, terminates |
| No committed fixture covers super-from-an-inherited-method | `tests/lang/inheritance/` | corpus search; 4 fixtures, none does |
| Super-construct: `Initializer`-gated metaclass retry | `dispatch.rs:891-915` | quoted |
| `NewInstance` is idempotent on an existing instance | `dispatch.rs:996-1020` | quoted; 3-level chain run |
| Field access resolves against the compiling class, no superclass merge | `compiler/lib/expr.rs:254-278`, `vm/mod.rs:52-57` | quoted |
| Inherited-field read raises `ReadBeforeWrite`; no privacy error exists | `compiler/lib/error.rs:101` | quoted; zero `CompilerError` privacy variant |
| Following that advice yields two slots | `classes.md:172` | program run: `7` / `999` |
| `ReadBeforeWrite` has no test anywhere | `tests/` | corpus search: zero hits |
| `super` at top level is a compile error; `disasm` cannot show `SuperSend` | ADR-0040 | both program runs |
| `size_of::<Bytecode>()` = 8; `SuperSend` ties `InvokeLocal`/`InvokeConst` | `bytecode.rs:352,362` | measured |

Defect record: [E006](../../errors/E006-inherited-field-diagnostic-shadowing.md).

---

## Forward

- **E006 is open** — a diagnostics defect, not a semantics one, which is why it is easy to leave.
- **The classic `super` correctness case is untested.** The code is right; nothing holds it right.
  A three-level fixture with an inherited-but-not-overridden method is four lines.
- **`SuperSend` caching** is DEC-IC-B, still open, and the honest argument against doing it first is
  that it needs the general hierarchy-invalidation machinery to exist anyway — build that, then
  piggyback.
- **The ADR's option space is narrower than the real one** — nothing recorded weighs a forward cell
  or an install-time `home_class` against the baked name.

With this, the three ranked gaps in [TRACKER](../TRACKER.md)'s Owed list are all paid.
