# Forge — Phase 3 (Implement / Review) Index

_The living map for implementation work. It supersedes the Phase-2 planning index
(now archived at [`archive/phase2/PHASE2-INDEX.md`](archive/phase2/PHASE2-INDEX.md)) as the
forward-looking coordination doc._

**This index deliberately does not re-list per-unit status** — that would fork the roster
(the mistake [`DEFERRED.md`](DEFERRED.md) #29 warned against). It *points* at the status of
record and carries only the cross-cutting coordination knowledge (collision matrix, build-order
discipline, resolved-decision register, successor-track pointer) that outlives any single unit.

Chronological landing detail → [`STATE.md`](STATE.md). Deferral ledger → [`DEFERRED.md`](DEFERRED.md).
Per-unit as-built specs → [`../spec/v0.2/units/`](../../spec/v0.2/units/).

---

## 1. Where unit status lives (pointers, not a copy)

| Track | Status of record | As-built specs |
|---|---|---|
| Spine roster — U-FE, U0–U11, U-LIST, U-LEX, U-STD (**all landed**) | [`STATE.md`](STATE.md) "— LANDED ✅" sections | [`../spec/v0.2/units/U/`](../../spec/v0.2/units/U/) |
| Successor core library — U-CORE-1..6 (partly landed) | [`../spec/v0.2/core/README.md`](../../spec/v0.2/core/README.md) §"Status" | [`../spec/v0.2/units/U-CORE/`](../../spec/v0.2/units/U-CORE/) |
| In-flight batch — U12–U20, U-COLL (planning) | their per-unit `U*-plan.md` in this directory | (not yet) |

---

## 2. Standing coordination constraints — write-set collision matrix

_These are the durable parallelism rules; they apply to **every** new unit, not just the landed
spine. Any unit touching these files must respect them._

**`phalcom-ast` (parser/AST) is a serialization point** — historically contended by five spine units
(U4 block literals, U5 `if`/`while` desugar, U6 `var`/`??`/`?.`, U7 `construct`, U-LEX surface tokens).
A unit that edits `phalcom-ast` must run alone in it — never co-schedule two `phalcom-ast` editors.

**`core.ph` is a single shared file** edited additively (U6 → U-STD → U11 → the U-CORE track).
**Never co-schedule two `core.ph` editors** — sequence them and keep every edit additive.

**Other shared hot spots:**
- `vm.rs` / `compiler/lib.rs` / `bytecode.rs` — touched by nearly every runtime unit → keep the
  runtime spine serial.
- `primitive/object.rs` — any two units wanting `Object`-level primitives collide; sequence them.
- `universe.rs::create_core_classes` — the `Method` re-parent is shared by U-CORE-1 and U-CORE-3;
  whichever lands first makes the change, the other asserts it (R-INV-1.5/3.1). Never co-schedule.

---

## 3. Build-order dependency discipline

The spine was sequenced so each unit's prerequisites landed first; the same discipline governs new work.

```
U1 (heap) ──┬─> U2 (tower) ──┬─> U7 (fields/construct)
            │                └─> U11 (Bool tower)
            ├─> U4 (blocks) ──┬─> U5 (control-flow+inliner)
            │                 ├─> U10 (non-local return)
            │                 └─> U9 (variadics)
            ├─> U6 (Option/let-var)
            └─> U3 ──> U8 (dNU/perform)

Serial spine:  U1 → U2 → U4 → U5 → U6 → U7 → U-LIST
Parallel waves (disjoint write-sets):  U8 ‖ U-LEX ‖ U-STD → U10 ;  U9 ‖ U11
```

Rule of thumb: a unit that is a hard prerequisite of others (e.g. U-LIST for U8/U9) lands at the
**spine tail before the wave that needs it**, and never shares a wave with a write-set collision.

---

## 4. Open-decision register — all resolved for the landed roster

Kept as a record so the rationale isn't lost; every decision below is closed.

| ID | Unit | Decision | Resolution |
|---|---|---|---|
| DEC-A | U8/U9/U-STD | Kernel `List` was an unscheduled hard dependency | **U-LIST landed** (ADR-0019/0020) — unblocked the `List` dependency. |
| DEC-B | U9 | Variadic dispatch-table key not implementable as specced | Key by **bare name** via `<name>(*)` in the existing `ClassObject.methods` — no new table (DEFERRED #24). |
| DEC-C | U6 | How is `if(opt)` a compile error with no flow analysis? | Runtime no-coercion floor + compile-time rejection of syntactically-literal Option conditions → **ADR-0021**. |
| DEC-D | U7 | Class-side *stored* static fields were unspecified | Apply ADR-0011 up the tower; class object gets `static_slots` via a per-metaclass field table → **ADR-0017**. |
| DEC-E | U5/U-LEX | Who owns `if`/`while`/`for` surface parsing? | **U5 owns** parse-time desugaring to block sends (sets the U5↔U-LEX `phalcom-ast` boundary). |
| DEC-F | U-LEX | String-interpolation sigil | User ratified **`\(expr)`** (Swift-style) → **ADR-0022**. |

Full decision text + soft-flag rulings: [`archive/phase2/PHASE2-INDEX.md`](archive/phase2/PHASE2-INDEX.md) §4.

---

## 5. Successor track — core library (U-CORE-1..6)

The spine roster is closed. Ongoing core-library work is planned in a separate, HEAD-grounded track
whose **index of record is [`../spec/v0.2/core/README.md`](../../spec/v0.2/core/README.md)** — do not
fork its roster here.

| Unit | Mission (1-line) | As-built spec | Status |
|---|---|---|---|
| U-CORE-0 | requirements/rulings: floor census, bootstrap phases, catalog delta, invariant-requirements, forward-compat | [`core/README.md`](../../spec/v0.2/core/README.md) | ✅ docs done |
| U-CORE-1 | kernel reflection — `Object#hash`/`isA(_)`, `Behavior#name`/`methods`, `Method < Function` re-parent | [`1-kernel-reflection.md`](../../spec/v0.2/units/U-CORE/1-kernel-reflection.md) | ✅ landed (`03764e3`) |
| U-CORE-2 | `Bool` half-Option fix + core `Option` combinators | [`2-bool-and-option-residue.md`](../../spec/v0.2/units/U-CORE/2-bool-and-option-residue.md) | mostly landed (`0da64d6`) |
| U-CORE-3 | callables/`Block`/`Method` reflection — iteration-method prereq | [`3-callable-reflection.md`](../../spec/v0.2/units/U-CORE/3-callable-reflection.md) | dispatch-ready (track head) |
| U-CORE-4 | value classes: per-type `toString` overrides (closes DEFERRED #30) | [`4-value-tostring.md`](../../spec/v0.2/units/U-CORE/4-value-tostring.md) | dispatch-ready |
| U-CORE-5 | collection protocol contract (shared interface) | [`5-collection-contract.md`](../../spec/v0.2/units/U-CORE/5-collection-contract.md) | dispatch-ready |
| U-CORE-6 | `Error` root + wire the dNU miss path to raise `MessageNotUnderstood` | [`6-errors.md`](../../spec/v0.2/units/U-CORE/6-errors.md) | dispatch-ready |

**Recommended order:** U-CORE-1 → U-CORE-3 → U-CORE-2 (residue check) → U-CORE-4 → U-CORE-5 → U-CORE-6.

**Cross-unit gate:** the `Method` re-parent in `universe.rs::create_core_classes` is shared by U-CORE-1
and U-CORE-3 (R-INV-1.5/3.1) — never co-schedule them. `Object#hash` (U-CORE-1) blocks a future
`Map`/`Set` unit (DEFERRED #27). The floor-admission amendments are folded into
[ADR-0023](../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md).

---

## Provenance

Sections 2–5 were transferred (2026-07-12) from the Phase-2 planning index, now archived at
[`archive/phase2/PHASE2-INDEX.md`](archive/phase2/PHASE2-INDEX.md) — which remains the historical
planning record (full roster, dependency notes, ADR/spec-edit backlog, cross-unit brief corrections).
Successor-track links were re-grounded to the versioned spec tree (`../spec/v0.2/`).
