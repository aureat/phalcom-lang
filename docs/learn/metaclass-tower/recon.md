# Recon — the metaclass tower (Phase 1)

Orchestrator scout, grounding the two briefs. Not the survey (that is Agent B).

Doc kind: **knot** — genuine circularity. Every object has a class; a class is an object;
so a class has a class, and *that* is a class, forever. Show the cycle, fail the naive tie
honestly, show the real tie.

---

## 1. Architecture vs representation

**Architecture (the shape).** Smalltalk-style *parallel metaclass tower*. Every class `X` has a
companion **metaclass** `X class` that holds `X`'s class-side (`static`) methods. The two chains
run in lockstep by the **parallel rule** (ADR-0002):

```
(X class).superclass  ==  (X.superclass) class
```

A shared kernel — `Behavior` / `Class` / `Metaclass` (ADR-0003) — owns the method dictionary,
superclass link, lookup, and instance creation, so `Class` and `Metaclass` are not special-cased.
The tower closes at the top: `Object class`'s superclass is `Class`, and `Metaclass` is an
**instance of itself**.

**Representation (what it holds — this is where the knot actually resolves).** The tower is **not**
an infinite structure in memory. It is a small finite set of `ClassObject` rows. Each row holds
its metaclass and superclass as plain **`ClassId` handles** — not references, not `Rc`, not
pointers:

```rust
// phalcom-core/src/heap/class.rs::ClassObject (~L25)
pub struct ClassObject {
    pub name: String,
    pub class: ClassId,               // its metaclass — a handle
    pub superclass: Option<ClassId>,  // None only at the apex (Object)
    pub methods: MethodsMap,
    // … field slots, static slots, base_names, attributes
}
```

The apex cycles are **handles that point at themselves**: `Metaclass.class == Metaclass` is a row
whose `class` field stores its own `ClassId`. The module doc states it directly (class.rs L1–8):
*"the kernel's cyclic wiring … is just a handle that points at itself — no `Rc`, no `Weak`, no
`RefCell`."*

**These are different axes.** The architecture is "parallel tower, Smalltalk." The representation is
"a handful of rows keyed by `ClassId`, cyclic at the top via self-handles." The consequences —
constructibility, GC, zero `unsafe` — live in the representation.

Cite: `heap/class.rs::ClassObject` @ ~L25; module doc @ L1–8; `Heap::class(id)` accessor
`heap/accessors.rs::Heap::class` @ ~L23 (resolves a `ClassId` to its row, panics on stale).

---

## 2. The grip, grounded

> **The infinite regress "the class of the class of the class…" is a regress in the *question*,
> not in memory.** Phalcom stores the whole tower as a handful of rows whose top handles point
> back into the set — the tower is finite and tied off with self-loops. And because the links are
> handles, not references, the cycle is trivial to *hold* (store your own id) and the only real
> work is *building* it: you allocate every row blank, then patch the handles.

Two halves, both must land: (a) **holding** the cycle — the `ClassId`-handle representation makes a
self-loop a non-event; (b) **building** the cycle — bootstrap must allocate-then-patch, because you
cannot construct two mutually-referential rows at once. The knot is the seam between them.

This is the same house-rule as [[upvalues]] and `ObjRef`/`FrameToken`: **a handle (name/index),
resolved through the heap — never a raw address.** A self-referential class is exactly the shape a
borrow checker forbids for references; handles dissolve it.

---

## 3. What was actually deliberated (vs pedagogical reconstruction)

Read the ADRs' real decisions — the doc must not present its full design-space walk as if it were
the deliberation (honesty pass, §5.2).

- **ADR-0002 (metaclass-tower-parallel-rule), accepted.** *Decision:* the parallel rule.
  *Deliberated alternative was NOT a fresh menu of options* — it was a **bug fix**. The prior state
  was a **flat metaclass chain** under which class-side (`static`) methods did **not** inherit
  correctly. The ADR calls the parallel rule a *"correctness fix, not an optional refinement …
  the minimum required for static-method inheritance to work at all,"* and adds
  `verify_invariants()` as a permanent guard against *"reintroducing the flat-chain bug."* No
  "Alternatives considered" list exists in the file.
- **ADR-0003 (introduce-behavior-kernel-class), accepted.** *Decision:* add `Behavior` as an
  abstract kernel class owning the shared protocol; `Class` and `Metaclass` both inherit it;
  `Behavior` inherits `Object`. *Alternative it displaced:* **asymmetric special-casing of
  `Metaclass` vs `Class`.** It exists so the parallel tower "express[es] cleanly rather than as a
  pile of special cases."
- **ADR-0009 (handle-arena-heap), accepted — supersedes the original representation.** ADR-0002 was
  first built on `Rc<RefCell<T>>` + `PhRef::new_cyclic` (a real cyclic-`Rc` construction dance).
  U2 (2026-07-11) replaced that with a `slotmap`-backed `Heap` and `Copy` `ClassId` handles;
  "instance of itself" became "a handle pointing at itself," and allocate-then-wire became
  **allocate-then-patch over `ClassId`s** in `Universe::create_core_classes`. **This is the scar** —
  the same tower, two representations, and the second one deletes an entire class of construction
  pain.

**Honesty flag for synthesis:** the pedagogical fork I'll draw (no metaclass / one shared metaclass /
parallel tower) is *reconstruction*. What Phalcom actually did was fix a flat-chain bug (0002) and
factor out `Behavior` (0003), then re-representation the cycle from `Rc` to handles (0009). Mark the
walk as scaffolding; the `Rc`-to-handle change is the one genuinely-deliberated representation move
and it earns the most weight.

---

## 4. Brief-steering notes

**Agent A (theory) — emphasis (do NOT reveal Phalcom's answer):**
- Go DEEP on: *why a metaclass exists at all* (class-side methods need somewhere to live and to
  inherit); the classic **"turtles all the way down" regress** and how real systems terminate it
  (Smalltalk's `Metaclass class class == Metaclass`, the `Object`/`Class` closure). This is the
  named problem — get the Smalltalk-80 history exact.
- Go DEEP on: the **bootstrap / chicken-and-egg** angle — constructing a cyclic class graph from
  nothing. This is half the knot; theory has rich precedent (Smalltalk image, Ruby's C-level
  `BasicObject`/`Class`/`Module` init, Python `type`/`object` mutual instantiation).
- One sentence each (branches that don't earn depth here): prototype-based escape (JS/Self — no
  classes so no metaclass), and typeclass/trait dispatch. Name why cut.
- Design space to walk as a *space* (make each tempting): (1) **no metaclasses** — class-methods
  are just entries in the class, no separate object (Java `static`, Python-ish); (2) **one shared
  metaclass** for all classes (early Smalltalk, Ruby's practical feel); (3) **parallel metaclass
  per class** (Smalltalk-80). Cost/benefit of each: what breaks in static-method *inheritance*
  under (1)/(2).
- Vocabulary the reader needs named: *metaclass, eigenclass/singleton class (Ruby), `type` as its
  own instance (Python), instance-of vs subclass-of as two orthogonal arrows, the parallel rule.*

**Agent B (source map) — the dominating question, plus must-confirm:**
- **Headline (answer first, with the line):** *How does Phalcom represent the class-of-a-class link
  — an address, a name/handle, an embedded object, or is the tower not fully built at HEAD?* State
  the candidates so B can't pattern-match. (Recon's read says: `ClassId` handle, cyclic at apex via
  self-handle — B must confirm the field and the self-cycle, and must NOT assume it's `Rc`.)
- Confirm & quote: `ClassObject.class` / `.superclass` fields (`heap/class.rs` ~L25);
  `lookup_method_in_hierarchy` (`heap/class.rs` ~L74) — the superclass walk;
  `Heap::class(id)` accessor (`heap/accessors.rs` ~L23).
- Confirm & quote the **bootstrap tie**: `Universe::create_core_classes` and
  `Universe::verify_invariants` in `phalcom-core/src/universe.rs` — the allocate-then-patch ordering
  and the parallel-rule assertion. This is the heart of a knot doc; get the ordered steps and the
  invariant checks verbatim.
- Distinguish the two "class-of" paths: `Heap::class(ClassId)` (row→row) vs the **value-level**
  `Value::class()` / `.class()` accessor (`value/mod.rs` ~L121, `heap/accessors.rs` ~L23) — how a
  `Value` (including immediates: number, nil, bool) maps to its class. The doc will conflate these
  if not separated.
- **Run live** (reflection surface): with `cargo run -p phalcom-core --bin phalcom`, observe from
  `.ph` — `X class`, `X class class`, whether `Metaclass class == Metaclass` holds, and
  `Object class superclass == Class`. Report actual output. If any reflection selector is absent at
  HEAD, say so plainly — that bounds what the doc can claim.
- Bounded spec/ADR: `docs/spec/v0.2/core/core-classes.md` §"Kernel tower classes" (Object @ ~L100,
  Behavior @ ~L132, Class @ ~L157, Metaclass @ ~L170) + `object-model.md` §5–6. ADRs 0002 / 0003 /
  0009 Decision sections only.
- GC angle: confirm the collector marks through `ClassId` links and never moves/patches them
  (contrast the upvalue-doc GC note). One or two lines.

**Diagram note (for synthesis):** the shape *is* the point here — draw the finite cyclic
instance-of/subclass-of graph (two arrow kinds), showing the self-loop at `Metaclass` and the
`Object class → Class` closure. This diagram earns its place (gate §6). Do **not** draw it as an
infinite chain.
