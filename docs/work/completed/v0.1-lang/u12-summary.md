# Forge Phase-2 — U12–U18 cluster: open-design-question plans

_Standalone planning artifact. Covers the previously-**unscheduled** cluster of open design
questions from [`docs/spec/open-questions.md`](../spec/open-questions.md) that had no unit plan:
**Q2, Q4, Q7, Q8, Q10, Q12, Q13, Q14**. Does **not** re-plan anything in
[PHASE2-INDEX.md](PHASE2-INDEX.md) (U1–U11, U-FE, U-LIST, U-LEX, U-STD) or the U-CORE track.
Deliberately does **not** edit PHASE2-INDEX.md or [core/HANDOFF.md](../spec/core/HANDOFF.md) —
two other planning agents (collection literals, concurrency) are working near those files._

Planned by `phalcom-architect`, 2026-07-12. All eight questions are placed; the four that
selectors §7 / open-questions flags as load-bearing tradeoffs are surfaced as **open decisions**
below, not silently chosen.

---

## 1. Unit roster (IDs claimed: U12–U18)
| Unit | Plan | Mission (1-line) | Question(s) | Spec / ADR | Status |
|---|---|---|---|---|:--:|
| **U12** | [U12-plan.md](U12-plan.md) | Numeric surface split — keep flat `Number` vs abstract `Number`→`Integer`/`Float` | Q2 | ADR-0005 (amend), 0010; object-model §4 | **BLOCKED-ON-DECISION** |
| **U13** | [U13-plan.md](U13-plan.md) | Class-hierarchy stability policy — runtime `superclass=` mutability + traits/mixins/MI | Q4, Q10 | ADR-0002/0009/0011/0018; object-model §1.5/§5 | **BLOCKED-ON-DECISION ×2** |
| **U14** | [U14-plan.md](U14-plan.md) | Destructuring bindings — `let (a,b)=…`, `let [first,*rest]=…` (irrefutable) | Q7 | ADR-0014; values-and-absence §1; messages §4–5 | architect-decidable; **dep on collection-literals unit** |
| **U15** | [U15-plan.md](U15-plan.md) | Modules & imports — give the `import` token meaning (source-only Module namespaces) | Q8 | object-model §4 (`Module`) | **BLOCKED-ON-DECISION** |
| **U16** | [U16-plan.md](U16-plan.md) | Method references `::` + `Family` value + base-name index + Family introspection | Q14 | selectors §3/§3.1; ADR-0012; U8 dNU | soft flag (surface richness) |
| **U17** | [U17-plan.md](U17-plan.md) | `Option` bootstrap formalization (ADR) + niche-encoding decision (rec: defer) | Q13 | ADR-0007/0010; values-and-absence §3 | soft flag (mostly docs) |
| **U18** | [U18-plan.md](U18-plan.md) | Default arguments — trailing-only, definer-side arity-family expansion | Q12 | ADR-0012; selectors §7.3; U9 | **BLOCKED-ON-DECISION** |

Merge rationale: **Q4+Q10 → U13** (both decide what stability dispatch/IC/slot-layout may assume
about the class graph; shared write-set `class.rs`/`vm.rs`/invariants). Q2 and Q12 were **kept
separate** (U12/U18) despite both "touching dispatch/arithmetic" — their write-sets are disjoint
(U12: `value.rs`/`primitive/number.rs`; U18: `compiler`/`signature.rs`) and each carries an
independent user decision, so combining them would violate "small, independently-verifiable."

## 2. Code-state grounding (verified against HEAD, post U1–U11 landing)
- **`Value` enum** (`value.rs` L31): `Nil, Bool(bool), Number(f64), Symbol, Obj(ObjRef)`. Heap
  `Object` variants: Instance, Class, Method, Module, Str, Closure, Block, List, Upvalue — **no
  `Tuple`, no `Option`/`None`, no `Family` arm.** Niche room exists (relevant to U12/U16/U17).
- **`import`** → `Token::Import` exists (`token.rs:62`); **no parser/AST/runtime.** U15 is greenfield.
- **`::`** → `Token::ColonColon` exists (`token.rs:149`); **no `Expr::MethodRef`, no `Family`, no
  `base_names` index** (`grep base_name` empty). U16 builds the whole `::`/Family feature, then Q14
  on top.
- **`None`** is a VM-blessed heap **singleton** (`none_singleton`, `value.rs` L268), zero-alloc,
  identity-comparable; the `None` class has **no fields** → the Q13 bootstrap cycle is **already
  avoided**. U17 is mostly an ADR + a deferred niche.
- **`Tuple`** is **spec-only, unimplemented** → U14 depends on the concurrent collection-literals
  unit (tuple/list literals + a runtime `Tuple` type).
- **U8** (Message reification, `doesNotUnderstand`, `perform`, `VM::send_dynamic`) is landed —
  U16's candidate-enriched miss path builds on it.

## 3. Intra-cluster dependencies
Given the landed U1–U11/U-STD substrate, **no unit in this cluster hard-depends on another for
correctness** — they are independent. The only real cross-links are:
- **U14 → collection-literals unit** (external agent): tuple/list literal grammar + runtime `Tuple`
  type. Hard dependency; U14 waits for it.
- **U16 ↔ U13**: both hang work off **class-finalization** (U16's `base_names`, U13's optional
  trait-flatten). Share one finalization hook; if traits (U13/Q10) are ruled in, U16 must build
  `base_names` *after* the flatten. Sequence U16 after U13.
- **U18 → U16**: synthesized default-arg selectors must register into the method dict before U16
  builds `base_names` (so `obj::move.candidates` lists them). Sequence U18 before U16, *or* ensure
  registration precedes finalization.
- **U12 / U16 / U17** all touch `value.rs` (+ `core.ph`) → mutually serialize.

So ordering is driven by **write-set collision**, not correctness.

## 4. Write-set collision matrix (the parallelism constraint)
| File | Units that touch it |
|---|---|
| `phalcom-ast` (parser/ast/lexer) | **U14, U15, U16, U18** (+ U12 only if numeric literals need a lexer tag) — cannot share a wave |
| `phalcom-core/src/compiler/lib.rs` | U12, U14, U15, U16, U18 — cannot share a wave |
| `phalcom-core/src/value.rs` + `heap.rs` | U12, U16, U17 — serialize |
| `phalcom-core/src/vm.rs` | U13, U15, U16 — serialize |
| `phalcom-core/src/class.rs` (finalization) | U13, U16 — serialize (share the hook) |
| `phalcom-core/core/core.ph` (additive) | U12, U16, U17 — never co-schedule two `core.ph` editors |
| `phalcom-core/src/signature.rs` | U18 (and U12 min/max-arity accounting if split) |

**Most isolated:** **U13** (recommended/sealed form: `class.rs`/`method.rs`/`vm.rs`/`invariants.rs`,
**no `phalcom-ast`**) and **U17** (recommended/defer form: docs + one invariant test). These two are
the only genuinely wave-parallel pair.

## 5. Wave schedule (parallelize by disjoint write-sets)
Foundational/decision units first; the four `phalcom-ast`+`compiler`-bound units are **inherently
serial** (same contention PHASE2-INDEX §3 records for the spine). The orchestrator fans out one
worktree-isolated implementer per unit per wave.

```
Wave 1 (parallel, disjoint):     U13 (hierarchy policy) ‖ U17 (Option bootstrap/ADR)
                                  — neither touches phalcom-ast/compiler in its recommended form
Wave 2 (alone; value.rs+compiler+core.ph):   U12 (numeric split)
                                  — serialize before U16 (shared value.rs/core.ph)
Wave 3 (alone; phalcom-ast+compiler+value.rs+class.rs+vm.rs):   U16 (:: / Family)
                                  — after U13 (finalization hook) and U12 (value.rs)
Wave 4 (alone; phalcom-ast+compiler+signature):   U18 (default arguments)
                                  — register synthesized selectors before any later base_names rebuild
Wave 5 (alone; phalcom-ast+compiler):   U14 (destructuring)
                                  — additionally gated on the external collection-literals unit
Wave 6 (alone; phalcom-ast+compiler+module.rs+vm.rs+universe):   U15 (modules/imports)
                                  — largest, greenfield; benefits from everything else being stable
```

- **Only Wave 1 is parallel** (U13 ‖ U17). Waves 2–6 are single-unit because `phalcom-ast` +
  `compiler/lib.rs` are contended by nearly every remaining unit — the honest constraint, matching
  the existing spine's serialization.
- Within the serial tail, the internal order is flexible except: **U13 before U16** (finalization),
  **U18 before U16** (or at least before base_names rebuild), **U14 after collection-literals**.
  U15 is independent greenfield and is placed last only because it is the biggest.
- If the orchestrator wants more parallelism, the only lever is to land the `phalcom-ast` changes
  for two units in one carefully-reviewed combined slice — **not recommended**; the spine kept
  these serial for a reason (multi-editor `phalcom-ast` diffs conflict).

## 6. OPEN DECISIONS — need the user before the named sub-feature can be built
_Same pattern as PHASE2-INDEX §4. Each unit's docs/ADR/scaffolding can proceed; only the
load-bearing sub-feature waits on the ruling._

| ID | Unit | Decision | Architect recommendation |
|---|---|---|---|
| **DEC-U12** | U12 | **Number: single flat `Number` (f64) vs surface `Integer`/`Float` split.** Changes `5/2`, literal typing, `==`/`hash`, adds a `Value::Int` arm. | If a concrete driver exists (indexing, bitwise, exact ints): **abstract `Number`→immediate `Integer(i64)`+`Float(f64)`**, `/` always Float, add `//`, mixed→Float, `1==1.0`. Else **keep flat** and close Q2 as resolved-flat. **User must rule** (surface-semantic, ~irreversible). |
| **DEC-U13a** | U13 | **Class-hierarchy mutability: runtime `superclass=` legal (Smalltalk) or sealed (Wren)?** | **Sealed for Draft 0.1** (keeps ADR-0011 slot layout + ADR-0012 IC stable); method *reopening* still allowed; document the ADR-0018-epoch escape path so mutability isn't foreclosed. |
| **DEC-U13b** | U13 | **Traits / mixins / multiple inheritance?** | **Affirm single inheritance, defer traits.** If wanted, the *only* admissible form is **stateless traits flattened at finalization** (preserves one-hashmap-probe dispatch); **reject C3/full-MI** (breaks committed dispatch + slot layout). |
| **DEC-U15** | U15 | **Module resolution + binding model** (path scheme, whole-module vs selective binding, export policy). | **Relative file-path source import + whole-module `import "p" as Name` + "everything top-level is a member" (no `export`) for Draft 0.1.** Reserve `from`/`export` as future keywords. Compiled-unit imports deferred (need a bytecode verifier). **User must rule** (surface model). |
| **DEC-U18** | U18 | **Default arguments at all? + expansion policy.** Fights selector identity (an omitted default → different selector → lookup miss). | **If wanted: trailing-only defaults via definer-side arity-family expansion** (bounded k+1 selectors sharing one body; no dispatch/caller change; reject default+rest, required-after-default, and synthesized/hand-written selector collision). Else **no default args** (most Smalltalk-honest). **selectors §7.3 "decide before shipping."** |

**Soft flags (architect proceeds on the recommendation; confirm only if you disagree):**
- **U14 (Q7):** desugar to the `at(_)` element-read protocol; **irrefutable** binding (mismatch =
  runtime error, no `match` yet). Confirm if you want iterator-based (any `Iterable`) destructuring
  or refutable patterns instead.
- **U16 (Q14):** build the **small reflective surface** on `Family` (`name`/`candidates`/`isBound`/
  `receiver`) — the base-name index already holds the data. Confirm if you want minimal
  (error-enrichment only) or rich (per-candidate `Method` objects).
- **U17 (Q13):** **defer** the `Value` niche-encoding (`None` is already zero-alloc); ship the ADR
  formalizing the blessed-singleton/no-cycle bootstrap. Confirm if you want the niche now.

## 7. New ADRs to draft (via `documentation-and-adrs`)
Provisional numbers **0024–0030** — **ADR-0023 is already reserved** (core-floor omnibus, per
`docs/spec/current/core/README.md`), and the concurrent **collection-literals** and **concurrency** planning
agents are also claiming numbers. **Do not hard-code these; grab the next-free at authoring time**
and update the plan's `00XX` placeholder.

| Provisional | Unit | Kind |
|---|---|---|
| ADR-0024 | U12 | numeric surface split (amends ADR-0005) |
| ADR-0025 | U13 | class-hierarchy stability (sealing + single-inheritance affirmation / trait forward-path) |
| ADR-0026 | U14 | destructuring binding desugaring (irrefutable) |
| ADR-0027 | U15 | module & import model (source-only, whole-module) |
| ADR-0028 | U16 | method references `::` / `Family` + reflective surface (realizes selectors §3, extends ADR-0012) |
| ADR-0029 | U17 | `Option` bootstrap blessing + niche decision (extends ADR-0007/0010) |
| ADR-0030 | U18 | default arguments via arity-family expansion (extends ADR-0012) |

## 8. Deferred (speed/scope items, not on any critical path)
- `Option`/`None` niche-encoding + `Some` payload packing (U17) — behind the `Value` API, with
  NaN-boxing (ADR-0010).
- Compiled-unit imports + bytecode verifier; selective import; `export`/visibility; import
  sandboxing / path-traversal policy (U15).
- Per-candidate `Method` reflection on `Family`; inline-cache population for Open-family calls (U16).
- Refutable patterns / a real `match`; iterator-based destructuring (U14).
- Bignum / rational numeric subclasses; `//` naming (U12).
- Runtime `superclass=` (mutable hierarchy) + stateless traits — pre-approved forward paths if
  DEC-U13a/b are later revisited.

## 9. Guardrail note — no unit lacks spec/ADR coverage
Every unit cites an existing spec § / ADR **and** proposes a new ADR where it makes a fresh
decision (U12–U18 each draft one). None is orphaned. Each plan carries an explicit
"must-not-preclude" gate against the other cluster questions and against
[concurrency.md](../spec/concurrency.md) (fiber-locality of frames/temps; no new shared-mutable
state; module init single-shot).
