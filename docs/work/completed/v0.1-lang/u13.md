# U13 — Work order: class-hierarchy stability policy (mutability + traits/MI) — open-Q4 + open-Q10

_Self-contained plan for **one** `phalcom-implementer` agent. Grounds in open-questions **Q4**
(hierarchy mutability, [open-questions.md](../spec/open-questions.md#L52)) and **Q10** (traits /
mixins / multiple inheritance, [open-questions.md](../spec/open-questions.md#L76)), plus
[object-model.md §1.5 / §2 / §5](../spec/object-model.md), **ADR-0002** (metaclass tower parallel
rule), **ADR-0009** (handle heap — "keeps runtime `superclass=` implementable"), **ADR-0011**
(fixed instance slot layout), and **ADR-0018** (override-epoch deopt guard — the existing
IC-invalidation prototype)._

> **This unit is BLOCKED-ON-DECISION ×2 (DEC-U13a, DEC-U13b).** Both questions decide what
> stability dispatch, the inline cache, and slot layout may assume about the class graph. The
> **conservative recommendation makes this a small enforcement + ADR unit**; the permissive
> rulings make it a large feature. Do not implement either permissive branch until ruled.

---

## 0. Mission (one sentence)
Ratify and enforce a single policy for the shape of the class graph — (a) whether a class's
`superclass` may be reassigned at runtime, and (b) whether Phalcom gains traits/mixins/multiple
inheritance — so that the inline-cache design (ADR-0012, IC-ready), the fixed slot layout
(ADR-0011), and the metaclass tower (ADR-0002) can state an explicit invariant instead of an
unstated assumption.

## 1. Hard guardrails
- **Single inheritance + fixed slot layout are the *current* invariants** (object-model §1.5,
  ADR-0011). This unit either affirms them with enforcement, or — only if ruled — replaces them.
- **`verify_invariants()` is sacrosanct.** Whatever policy lands, the metaclass-tower checks
  (object-model §5) must still pass. If mutability is ruled in, `verify_invariants()` must be
  *re-runnable after a mutation* and still pass (or the mutation is rejected).
- **Do not weaken the one-hashmap-probe dispatch** (ADR-0012). Any trait/MI resolution must be
  **flattened at class-finalization into the single method dictionary** — no per-send MRO walk,
  no second lookup path. This is non-negotiable and bounds which Q10 options are even admissible.
- Stay inside the write-set (§3).

## 2. Preconditions (verify first)
- `./scripts/verify.sh` green; `verify_invariants()` present and green (U2 landed).
- `graphify explain "class definition"` + `graphify explain "superclass"` — locate where a class
  is finalized (slot layout frozen, `base`/method dict built) and where (if anywhere) a
  `superclass=` send would land (`primitive/class.rs` / `primitive/object.rs`).
- Confirm ADR-0018's override-epoch counter exists (the IC-invalidation seam) — mutability (if
  ruled) reuses it; if it doesn't exist, that is a dependency, not this unit's invention.

## 3. Confirmed write-set (validate with `graphify affected "ClassObject"` on HEAD)
| File | Why |
|---|---|
| `phalcom-core/src/class.rs` | The stability flag on `ClassObject`; finalization gate; (if traits ruled) the trait-flatten step. **Contended with U16** (base_names) — serialize. |
| `phalcom-core/src/primitive/class.rs` (or `object.rs`) | `superclass=(_)` primitive: reject (sealed) or perform-and-invalidate (mutable). |
| `phalcom-core/src/vm.rs` | Only if mutability ruled: bump the override epoch / invalidate ICs on a hierarchy change. **Contended** — serialize. |
| `phalcom-core/tests/invariants.rs` | Assert the policy: sealing rejects `superclass=`, or mutation preserves invariants. |
| `phalcom-core/core/core.ph` | Only if traits ruled: trait-composition surface. Otherwise untouched. |
| `docs/adr/00XX-hierarchy-stability.md` | New ADR (Q4+Q10) — provisional number, grab next-free. |
| `docs/spec/object-model.md §1.5/§5`, `open-questions.md` Q4/Q10 | Flip to RESOLVED with the ruling. |

**Disjointness:** in the recommended (conservative) form this unit does **not** touch
`phalcom-ast` or `compiler/lib.rs` heavily (enforcement is runtime + invariant tests), so it can
**run in parallel** with a `phalcom-ast`/compiler-bound unit. Only the permissive traits branch
(surface syntax `class C with T`) pulls in `phalcom-ast`.

## 4. Design decisions

### DEC-U13a — hierarchy mutability (Q4) — **BLOCKED-ON-DECISION**
| Option | Behavior | Cost | Consequence for IC / slots |
|---|---|---|---|
| **A — sealed after definition (Wren-style)** | `superclass` fixed at class creation; **method reopening still allowed** (add/replace methods on an existing class — already supported) | tiny (a rejection + a test) | slot offsets + IC keyed on `ClassId` are provably stable; ADR-0011 layout never shifts under a live instance |
| **B — mutable `superclass` at runtime (Smalltalk-style)** | `Test.superclass = X` legal | large: must recompute slot layouts of all live instances, invalidate every dependent IC (reuse ADR-0018 epoch), re-run `verify_invariants()`, and define what happens to instances whose field layout changed underfoot | powerful metaprogramming; but slot layout can no longer be assumed stable → every field access becomes conditional |

**Architect recommendation:** **A (sealed) for Draft 0.1**, keeping *method* reopening (which the
tree already does). Document the escape path: a future `become:`/reshape would reuse the ADR-0018
override-epoch to invalidate ICs, so B is not foreclosed — it is deferred. ADR-0009 already notes
the handle heap "keeps this implementable," so choosing A now costs nothing later.

### DEC-U13b — traits / mixins / multiple inheritance (Q10) — **BLOCKED-ON-DECISION**
| Option | Model | Admissible under one-probe dispatch? |
|---|---|---|
| **A — single inheritance only** (status quo invariant) | one `superclass`, `Object` root | yes (trivially) |
| **B — stateless traits, flattened at finalization** | a trait is a named bag of methods (no fields); `class C < S with T1, T2` copies trait methods into `C`'s dict at finalization, explicit conflict = compile error | **yes** — dispatch stays one hashmap probe; no MRO at send time |
| **C — full MI with C3 linearization** | multiple superclasses, runtime MRO walk | **no** — breaks the single-probe invariant and the fixed slot layout (state from multiple parents) |

**Architect recommendation:** **A — affirm single inheritance, defer traits.** If the user wants
composition, **B is the only admissible extension** (it preserves ADR-0012's single-probe
dispatch and ADR-0011's fixed layout because traits carry *no state*). **Reject C outright** — it
is incompatible with the committed dispatch/layout design and would be a redesign, not a feature.
Record B's shape in the ADR as the pre-approved forward path so a later unit can build it without
re-litigating.

## 5. Risk
- **The two decisions are coupled:** choosing B (mutable) *and* C (MI) together would make both
  slot layout and dispatch fully dynamic — a different VM. Present them as a matched pair; the
  conservative pairing (A+A) is internally consistent and cheap.
- **Silent invariant rot:** if mutability is ruled in and `verify_invariants()` is *not* re-run
  after mutation, the tower can be corrupted with no signal. The test must mutate-then-verify.
- **Trait-flatten ordering:** if B is ever built, flatten order + conflict detection must be
  deterministic or two runs produce different dictionaries.

## 6. Test strategy (green gate must assert)
- **Sealed (A):** `Test.superclass = Object` raises a clean error (not a panic); the class graph
  is unchanged; `verify_invariants()` still green.
- **Method reopening still works:** adding a method to an existing class post-definition succeeds
  (proves sealing is *superclass-only*, not a blanket freeze).
- **If B ruled:** after a legal `superclass=`, `verify_invariants()` passes, dependent call sites
  deopt correctly (no stale IC), and a live instance's field access is still sound.
- **If traits (B/Q10) ruled:** a conflicting method across two traits is a compile-time error;
  a non-conflicting composition resolves in one hashmap probe (assert no MRO walk).
- Tower regression: `Number.class.superclass == Object.class` etc. unchanged under the policy.

## 7. Forward-looking — must NOT preclude
- **U16 (Family/`::`) base-name index** is built at class-finalization — the same seam as trait
  flattening. Whichever policy lands must leave finalization a single, well-defined point so U16's
  `base_names` and any future trait-flatten compose there. Coordinate the finalization hook.
- **U7 slot layout (ADR-0011):** sealing (A) is what lets ADR-0011 keep offsets stable; do not
  adopt B without the IC-invalidation story, or U7's `GetField(slot)` becomes unsound.
- **Inline caches (ADR-0012, deferred population):** the policy is the precondition the future IC
  relies on. State it as an explicit invariant so IC population is "not a redesign."
- **Concurrency (concurrency.md):** class objects are shared across fibers via the handle heap; a
  mutable hierarchy (B) becomes shared mutable metadata — under cooperative single-threading
  there is no data race, but a `superclass=` mid-computation could still surprise a suspended
  fiber. Sealing (A) sidesteps this entirely. Note the interaction if B is ruled.

## 8. Mandatory rules
- `///` on the stability flag, the finalization gate, the `superclass=` primitive, any trait
  machinery; `//!` refreshed; cite ADR-0002/0009/0011/0018 + the new ADR. `cargo doc` clean.
- Green gate = `./scripts/verify.sh` exits 0. Because this touches tower invariants, recommend
  reviewer **ON**.
- Own isolated worktree off `main`.

## 9. Return contract
Report: the DEC-U13a and DEC-U13b rulings implemented · confirmation `verify_invariants()` is
green (and, if B, green *after mutation*) · that dispatch stayed one hashmap probe · that method
reopening still works · the documented forward path for the un-chosen options · files changed ·
`verify.sh` + `cargo doc` tails.
