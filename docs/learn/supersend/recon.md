# SuperSend doc — recon

Phase 1 of [AUTHORING-LEAN](../AUTHORING-LEAN.md), written before drafting. Cited line, quoted
record, or program output at HEAD.

---

## 1. Architecture vs representation

`super.sel(…)` compiles to its own opcode, `SuperSend(argc: u8, selector: u16, defining: u16)`
(`bytecode.rs:138`). Architecturally it is the *most statically determined send in the language* —
the compiler knows, at the call site, exactly which class body it sits in, and the lookup start is a
per-call-site constant rather than something derived from a runtime receiver.

Representationally it is nothing of the kind. Operand 2 is **the defining class's name symbol**, not
a class handle:

> The class object does not exist at compile time, so its *name* is baked and resolved to a class
> handle at dispatch (the same global lookup as `GetGlobal`). — `bytecode.rs:119-123`

So every `super` performs, at dispatch: a constant-pool read, a `ClassKey { module, name }` hash
lookup (with a core-module fallback), a `superclass` field read, and then an uncached chain walk.
Do not import "statically known" from the architecture into the representation — Doc 5's own forward
pointer calls it "a statically-known target," which is true of the *name* and false of everything else.

## 2. The grip (grounded)

**`super` is the least dynamic thing a Phalcom program can write, and it is resolved dynamically, by
name, on every single call.** Three separate decisions each converted a fact the compiler already had
into a lookup the VM redoes forever — bake the name because the class does not exist yet; bake the
*defining* class rather than its superclass so a mutation feature that does not exist stays correct;
and do not cache, because the cache key shape differs from a receiver-polymorphic one.

Cites: `bytecode.rs:119-137`, `dispatch.rs:851-885`, ADR-0040 §Decision.

## 3. Deliberated vs reconstructed

**Deliberated, in ADR-0040 `## Alternatives considered`** — four branches, each with a reason:
amend `Invoke` instead (rejected: blurs two dispatch shapes, complicates a future IC); bake the
superclass directly (rejected, DEC-INH-B: goes stale under `superclass=`); dynamic-receiver-class
super (rejected: breaks the defining-class rule for multi-level chains); implicit constructor
auto-chaining (rejected, DEC-INH-C: explicit `super.construct` matches Smalltalk/Wren).
Also deliberated: miss routes to `doesNotUnderstand`, never a panic; bare `super` is a compile error.

**My reconstruction:** that the three dynamic steps *compound* into a cost nobody priced. Each
decision is individually argued in the ADR; their sum is not discussed anywhere, and there is no
measurement of it (see F3). Labelled as my framing, not a recorded finding.

## 4. Findings

**F1 — name-baked, resolved per dispatch.** `dispatch.rs:874-881`: builds
`ClassKey { module: closure_module, name: defining_sym }`, looks it up in `self.classes`, and on a
miss retries against the **core module**. Two hash lookups worst case, per `super`, forever.

**F2 — DEC-INH-B buys late binding for an unbuilt feature.** The defining class is baked and
`.superclass` read at dispatch specifically so the send "stays correct under a future runtime
`superclass=` mutation (U13)" (`bytecode.rs:125-127`, ADR-0040). U13 does not exist at HEAD. A real
per-dispatch indirection is being paid to keep a hypothetical honest.

**F3 — uncached, and the decision IDs are *not* in conflict.** `bytecode.rs:136-137` cites
**DEC-INH-F**; TRACKER and Doc 5's recon cite **DEC-IC-B**. `docs/forge/units/U-IC/plan.md:109`
reconciles them: DEC-INH-F is U-INH's original "first cut is uncached," DEC-IC-B is U-IC's still-open
decision about whether to add one. I nearly wrote this up as a citation error; it is not one.

**F4 — a small correction to Doc 1.** [`execution-loop.md:170`](../vm/execution-loop.md) annotates
`SuperSend(u8, u16, u16)` as "the widest variant, ~8 bytes". Measured: `size_of::<Bytecode>()` is
**8 bytes**, so the size is right — but `InvokeLocal(u16, u8, u16)` and `InvokeConst(u16, u8, u16)`
(`bytecode.rs:352,362`) tie it at five operand bytes. `SuperSend` is *a* widest variant, not *the*
one, so it is not solely responsible for the enum's width. True within Doc 1's three-variant excerpt;
worth stating precisely here rather than repeating.

**F5 — this opcode cannot be shown with the tooling.** `disasm` walks only the top-level chunk, and
`super` is a **compile error** outside a method (`ADR-0040`, verified: *"`super` cannot be used
outside a method: there is no defining class to start the lookup above."*). The hoist-to-module-level
workaround that other docs in this course used to make bytecode visible is *structurally unavailable
for exactly this construct*. Verified: a working `super` program disassembles with no `SuperSend`
visible anywhere. Every runtime claim must be source-cited or labelled INFERRED.

**F6 — the no-superclass miss names a method the class visibly defines.**

```phalcom
class Lonely {
  greet => super.greet
}
System.print(Lonely.new().greet)
```
```
<Lonely instance> does not understand 'greet'
```

Correct per ADR-0040 (empty walk → dNU, never a panic) and technically true about receiver and
selector — while the class defines `greet` three lines up. The diagnostic reports the *receiver's*
failure, not the *walk's*.

**F7 — the inherited-field diagnostic routes users into a shadowing bug.** This is the strongest
finding and it is a *feature interaction*, not a bug in either feature.

`classes.md:125` states the rule: **"Fields are private to the declaring class and not
inherited-visible."** `classes.md:172`: **"A subclass gets its own fresh slot."** Both deliberate.

But a subclass reading an inherited field gets a **flow-analysis** error, not a privacy one:

```
Error: Read-before-write: field '_x' is used before being assigned anywhere in this class.
```
(`compiler/lib/error.rs:101`)

Its implied remedy is "assign `_x` in this class." Follow it and you get a second slot:

```phalcom
class Base    { construct new(x) { _x = x }        baseSees    => _x }
class Derived extends Base {
  construct new(x) { super.new(x); _x = 999 }      derivedSees => _x }
```
```
Base sees:    7
Derived sees: 999
```

One object, one field name, two values. Spec-correct in every step; the path there is a diagnostic
that misnames a privacy violation as a missing assignment and whose fix is the trap. It lands
directly against super-construct, whose entire job (`super.new(x)`, ADR-0040 + ADR-0011 idempotent
`NewInstance`) is filling the parent slot the subclass then cannot read.

Verified working control: `super.new(x)` plus an accessor (`self.x`) prints `7` — the slot really is
filled.

**F8 — ADR-0040's "global lookup" language predates module scoping.** It says the name resolves via
"the same global lookup `GetGlobal` performs." The code uses a module-scoped `ClassKey` with a
core-module fallback (`dispatch.rs:874-881`), which is U-CLASSNS work. Ask B what the fallback means
for two modules that both define the same class name.

## 5. Forbidden list

| Material | Owner |
|---|---|
| The ordinary resolve loop, the superclass chain walk, `doesNotUnderstand` forwarding | [Doc 4 `message-send.md`](../vm/message-send.md) |
| Inline caches, `world_version`, fusion/superinstructions | [Doc 5 `caches-and-fusion.md`](../vm/caches-and-fusion.md) |
| Opcode width as a *fetch* story, the dispatch loop's shape | [Doc 1 `execution-loop.md`](../vm/execution-loop.md) |
| `GuardBool`/`GuardBlock`, dual emission | [`sacred-inliner.md`](../vm/sacred-inliner.md) |
| The metaclass tower / parallel rule | [`metaclass-tower.md`](../object-model/metaclass-tower.md) |

This doc owns: the `SuperSend` opcode and its three operands, name-baking and dispatch-time
resolution, the defining-class-not-superclass rule, super-construct's metaclass re-encode, and the
super/field-privacy interaction.

## 6. Open risks

| Risk | Disposition |
|---|---|
| Is there any memoization I missed? | REFUTE ask to B. |
| Is F7 a known/covered behaviour or genuinely unrecorded? | REFUTE ask to B; check for a fixture. |
| Multi-level (3-deep) chains, and `super` from an inherited-but-not-overridden method | Ask B to run both — the defining-class rule is subtle here. |
| Does the core-module fallback let a user class hijack a core name (or vice versa)? | Ask B; memory says the class registry was once name-keyed VM-wide, and `ClassKey` shows it is now module-scoped — that memory may be stale. |
| Is the doc long enough to stand alone? | TRACKER warned it "may be a section rather than its own doc." F5/F6/F7 alone answer that: yes. |

## 7. Doc-kind gate

**Fork.** ADR-0040 records four rejected branches with reasons, and the doc's spine is what those
choices cost: *`super` looks static, and every decision made it dynamic anyway.*
Per AUTHORING-LEAN §3, **fork ⇒ Agent A runs.**
