# REQUIREMENTS — the metaclass tower

Working spec for the doc. Grip copied from `recon.md`, grounded in the source read.
Shipped file lands at `docs/learn/object-model/metaclass-tower.md`.

## 0. The one obligation

After reading, the reader can **re-derive** why Phalcom's class-of-a-class is a finite ring of
handles rather than an infinite regress — from the constraints alone (every object has a class; a
class is an object; class-side methods must inherit; you must be able to *build* the thing from
nothing). Description fails. ≥1 predict-then-check moment required.

## 1. The reader

Knows PL design, not fluent in implementation. Cannot hold moving-state mechanisms in their head;
lacks stable notation. Here the moving state is **two orthogonal arrows** (instance-of vs
subclass-of) plus a **construction ordering** (bootstrap). The doc's job is to give notation for
both so the regress stops feeling infinite.

## 2. Doc kind — **knot**

Genuine circularity. Structure: show the cycle → let the naive tie (infinite tower) fail honestly →
show the real tie (finite rows, self-pointing handles, allocate-then-patch). Not a fork doc; the
"design space" is pedagogical scaffolding (see §10 honesty risk).

## 3. The grip

**The infinite regress is in the question, not in memory.** The tower is a handful of `ClassObject`
rows whose apex handles point back into the set (`Metaclass.class == Metaclass`;
`Object class superclass == Class`). Two halves: **holding** the cycle is free because links are
`ClassId` handles, not references (store your own id); **building** it is the only real work —
allocate every row blank, then patch the handles. The knot is that seam.

House-rule callback: same as `[[upvalues]]`/`ObjRef`/`FrameToken` — a handle resolved through the
heap, never a raw address. A self-referential class is exactly what a borrow checker forbids for
references; handles dissolve it.

## 4. The design space (walk as a space; MARK as reconstruction — §10)

| Branch | Who | Buys | Costs / forecloses |
|---|---|---|---|
| No metaclass — class-side methods are just entries on the class object | Java `static`, C++ | simplest; one object per class | `static` methods **don't inherit** as real dispatch; no uniform "everything is an object" |
| One shared metaclass for all classes | early Smalltalk; Ruby's felt model | uniform object-ness, cheap | every class shares one class-side protocol → can't give `X` its own inheritable class methods |
| Parallel metaclass per class (the tower) | Smalltalk-80, **Phalcom** | class-side methods inherit by the same rule as instance-side | must model the regress + terminate it; must *build* a cycle |
| No classes at all (prototypes) | JS, Self | no metaclass problem exists | different object model entirely — cut to a sentence |

The real deliberation (per ADRs) was narrower: a **flat-chain bug** where static methods didn't
inherit → the **parallel rule** fix (0002), plus factoring out **`Behavior`** (0003), plus the
`Rc`-cycle → **handle** re-representation (0009). Say so.

## 5. Comparison filter (≤~6 survive; name cuts)

Enters only if it (1) took the other branch with the bill, (2) has a scar, (3) **names something
Phalcom does anonymously**, or (4) is an ancestor.

- **Smalltalk-80** — ancestor + names everything (metaclass, the parallel rule, `Metaclass class
  == Metaclass`). Mandatory.
- **Ruby** — names the *eigenclass/singleton class*; its C-level `BasicObject`/`Object`/`Class`/
  `Module` boot is a real chicken-and-egg with scars. Strong.
- **Python** — `type` is its own instance (`type(type) is type`) and `type`/`object` are mutually
  dependent at C init — the cleanest small statement of the self-instantiation tie. Strong.
- **Java** — the other branch (no metaclass; `static` doesn't truly inherit) with a concrete bill.
  Keep short — it's the foil for §4 row 1.
- Likely **cut**: JS/Self (no classes → one sentence), C# (near-Java), functional/typeclasses
  (different axis). Name them and why.

## 6. Tensions to surface

- **Hold vs build.** The representation makes holding the cycle trivial but shoves all difficulty
  into construction order. That trade is the doc's spine.
- **Borrow checker vs cyclic data.** Two mutually-referential rows can't be constructed at once in
  safe Rust with references; handles turn the cycle into ordinary data you patch. This is why
  `ClassId`, not `&ClassObject`.
- **`Rc`-cycle → handle (ADR-0009).** Same tower, two representations; the second deletes
  `new_cyclic`/`Weak` construction pain. The scar.

## 7. Structural rules

- Grip stated early; **earned** by the bootstrap trace at the end.
- Two arrow kinds get **distinct notation** and a diagram (instance-of vs subclass-of). The diagram
  is the finite cyclic graph with the `Metaclass` self-loop and `Object class → Class` closure —
  it earns its place (the shape *is* the point). Do not draw an infinite chain.
- Spiral / mark lies: if an earlier `docs/learn` doc said "an object has a class" as if flat, note
  this doc is where the class-of-a-class is destroyed. Any simplification here (e.g. "ignore
  `Behavior` for a moment") flagged as a lie with a forward pointer.
- Anchors symbol-first: `file.rs::Type::method` (~Lxxx).
- HEAD-as-implemented; spec-intent only where v0.2 is unfinished, and say which.

## 8. Checklist (gate is AUTHORING §6)

- [ ] Re-derive + ≥1 predict-then-check (e.g. *"you have `Number`; where does `Number sqrt`-style
      class-side method live, and what is the class of that place?"* or the construction puzzle:
      *"row A needs B's id and B needs A's — build it in safe Rust."*).
- [ ] Grip grounded (handle, not `Rc`; confirmed from the type).
- [ ] Reconciliation table (A's theory vs Phalcom's representation) non-empty.
- [ ] Honesty: parallel rule labelled a **bug fix**, not a from-menu choice; design space labelled
      reconstruction.
- [ ] Claims ledger clean: every perf/forward/comparative claim cited, labelled unmeasured, or cut.
      GC and "zero unsafe" claims cite the line. Links resolve.
- [ ] **Hard trace** = the **bootstrap**: allocate-blank → patch-by-parallel-rule →
      verify_invariants, over the *real* `Universe::create_core_classes` steps. Not a toy.
- [ ] Weighted; design-space bloat cut ~30%.
- [ ] Comparison filter applied; cut list named.
- [ ] Diagram earns its place (finite cyclic graph, two arrow kinds).

## 9. Build sequence

Recon (done) → this file → Agents A+B parallel → synthesis (5 passes) → gate. Object-model track:
this is a foundational knot; later docs (dispatch, value representation) will lean on its notation.

## 10. Open risk — RESOLVED (post-synthesis)

- **R-APEX (fired; not foreseen in recon) — recon's grip named the wrong apex shape.** Recon §1/§2
  copied `heap/class.rs`'s module doc-comment: apex = a 1-node self-loop, `Metaclass.class ==
  Metaclass`. Agent B ran it live: **false**. The real apex is a 2-node loop (`Metaclass ⇄
  Metaclass class`), enforced by `verify_invariants` (`invariants.rs` L58). Recon got the
  *representation* right (`ClassId` handle, not `Rc`) but trusted a docstring for the *shape*. The
  live-run half of the two-agent split is exactly what caught it — the same failure family as the
  upvalue doc's contaminated grip, caught the same way. The doc turns this into the payoff
  predict-then-check and an honesty exemplar. **In-source defect noted for the user: the two
  `heap/class.rs` doc-comments (L1–8, L28–29) and ADR-0002's prose ("instance of itself") describe
  a shape the bootstrap does not build.**
- **R1 — reflection surface at HEAD.** The predict-then-check and any `X class class` examples
  depend on `.ph`-observable class reflection. If selectors like `class` on a class are absent at
  HEAD, the doc must trace at the Rust/heap level instead and say the surface isn't exposed.
  **Agent B must run this and report actual output.** If wrong, the reader-facing examples move
  from `.ph` to heap-diagram form.
- **R2 — honesty (fired-preemptively).** The four-branch design space is reconstruction; the real
  record is bug-fix + factor-out + re-represent. If the doc presents the menu as the deliberation,
  it repeats the upvalue doc's flattery error. Mark the walk as scaffolding (§5.2). *Resolution
  plan: single explicit sentence at the design-space head + the ADR record in the bootstrap
  section.*
- **R3 — two "class-of" paths conflated.** `Heap::class(ClassId)` (row→row) vs value-level
  `.class()` (Value→class, incl. immediates). If the doc uses one symbol for both, the reader
  mis-models immediates. **B must map both**; synthesis keeps them distinct.
