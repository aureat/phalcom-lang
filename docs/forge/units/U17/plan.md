# U17 — Work order: `Option` bootstrap formalization + niche-encoding decision — open-Q13

_Self-contained plan for **one** `phalcom-implementer` agent. Grounds in open-question **Q13**
([open-questions.md](../spec/open-questions.md#L92)) and [selectors.md §7 item 4], **ADR-0007**
(abstract `Option` + `Some`/`None`), **ADR-0010** (tagged `Value` enum), and
[values-and-absence.md §3](../spec/values-and-absence.md). **Grounding note (important):** U6/U-STD
already landed `Option`/`Some`/`None`, and `None` is a **VM-blessed heap singleton**
(`none_singleton`, `phalcom-core/src/value.rs` L268) — a zero-allocation, identity-comparable
instance. Because the `None` class **has no instance fields**, the "fields default to `None`"
bootstrap cycle that Q13 warns about **is already avoided**. So the *correctness* half of Q13 is
resolved; what remains is (a) writing it down in an ADR and (b) deciding whether to
**niche-encode** `Option` into `Value` as a performance optimization. This is therefore a **small,
mostly-documentation unit with a deferred-leaning optimization** — not a critical-path feature._

---

## 0. Mission (one sentence)
Formalize — in an ADR and the spec — how `None`/`Option` are special-cased relative to ordinary
classes (blessed singleton, no fields, no cycle), and rule on whether to niche-encode `Option` in
`Value` now (removing a heap fetch on `.class`/`match`) or defer it behind the existing `Option`
API as a speed item alongside NaN-boxing.

## 1. Hard guardrails
- **Do not regress the landed `Option`.** `Some`/`None`/`match`/combinators (U6/U-STD) must stay
  green. This unit adds an ADR + optionally an *invisible* representation change behind the same
  surface — user-visible `Option` semantics do not change.
- **`None` stays identity-comparable and zero-allocation.** Whatever representation, `None == None`
  by identity and constructing `None` allocates nothing (values-and-absence §3.1).
- **`nil` stays private.** Any niche must not let the private `Value::Nil` sentinel and surface
  `None` become confusable — they are distinct (Invariant 4). A niche for `None` is **not** a niche
  for `nil`.
- Stay inside the write-set (§3).

## 2. Preconditions (verify first)
- `./scripts/verify.sh` green; the `Option` fixtures from U6/U-STD pass.
- `graphify explain "None"` / `graphify explain "Option"` — confirm `none_singleton`, `none_class`,
  `some_class`, and how `Some(_value)` is constructed in Rust (`vm.rs`/`primitive`) and where
  `.class`/equality resolve `None`.
- Confirm the "no fields on `None`" fact in `core.ph` / the class field-count wiring (like
  `Message`'s `field_count = 4`, `None` should be `0`).

## 3. Confirmed write-set (validate with `graphify affected "none_singleton"` on HEAD)
| File | Why |
|---|---|
| `docs/adr/00XX-option-bootstrap-and-niche.md` | **Primary deliverable** — the ADR formalizing the blessed-singleton bootstrap + the niche decision. Provisional number, grab next-free. |
| `docs/spec/open-questions.md` Q13, `docs/spec/values-and-absence.md §3.1` | Flip Q13 to RESOLVED; document the bootstrap special-casing precisely. |
| `phalcom-core/src/value.rs` | **Only if niche ruled in:** the representation change (e.g. `Value::None` immediate arm or a reserved niche) + `class()`/`value_eq`/`type_name`/`Hash`. **Contended (`value.rs` group with U12/U16)** — serialize. |
| `phalcom-core/src/universe.rs`, `primitive/*` (option) | **Only if niche ruled in:** construct/compare `None` via the niche. |
| `phalcom-core/core/core.ph` | **Only if niche ruled in:** confirm `None` field-count = 0. **Contended (additive)** — serialize. |
| `phalcom-core/tests/invariants.rs` (+ fixtures) | Assert the bootstrap invariant (`None` no fields, zero-alloc, identity) regardless of representation. |

**Disjointness:** the *recommended* (defer-niche) form of this unit is **docs + one invariant
test** — it touches no contended Rust file and can run in **parallel with almost anything** (it is
a good Wave-1 companion). The niche branch pulls in `value.rs`/`core.ph` and must serialize with
U12/U16.

## 4. Design decision — **soft flag (recommend, confirm if disagreed)**
**Question:** niche-encode `Option`/`None` into `Value` now, or defer?

| Option | Representation | Benefit | Cost |
|---|---|---|---|
| **Defer (recommended)** | keep `None` as the blessed `Value::Obj(none_singleton)`; `Some` a normal instance | none new (already zero-alloc for `None`) — but `.class`/`match` on `None` do a heap fetch | zero now; formalized as a deferred speed item behind the `Option` API |
| **Niche now** | add `Value::None` immediate (or a reserved niche) so `None` needs no heap fetch; `Some` may pack its payload | removes a heap indirection from every `None` test/`match`; a common hot value | touches every `match self` on `Value`; interacts with `==`/`hash`; a `Some` niche is more invasive |

**Architect recommendation: DEFER the niche; ship the ADR.** `None` is already zero-allocation and
identity-comparable, so the niche is a *micro-optimization* on the `.class`/`match` heap fetch —
exactly the class of change ADR-0010 reserves as "a deferred optimization behind the same enum
API" (like NaN-boxing). Doing it now adds `Value`-arm churn (every exhaustive match) for a benefit
that only profiling can justify. **Record the niche as a DEFERRED speed item** and spend this unit
on the ADR that pins the bootstrap story (which is the actually-open part of Q13). Do not pick the
niche without a profiling driver; if the user wants it, it becomes a `value.rs`-serialized unit.

## 5. Risk
- **Over-scoping:** the temptation is to "just do the niche." Resist — it converts a safe docs unit
  into a `Value`-repr change that ripples through the whole crate and collides with U12/U16.
- **Nil/None confusion (if niche):** a poorly chosen niche could alias the private `Nil` sentinel
  with surface `None`, breaking Invariant 4. Keep them provably distinct.
- **`==`/`hash` (if niche):** `Some(a) == Some(b)` delegates to the inner `==`; a packed-`Some`
  niche must preserve that delegation and hash coherence.

## 6. Test strategy (green gate must assert)
- Bootstrap invariant: `None` class has zero instance fields; `None` is a single shared instance
  (`None === None` by identity); constructing/reading `None` allocates nothing.
- No cycle: the universe boots and `verify_invariants()` passes with `Option`/`Some`/`None` wired —
  proving the no-fields-on-`None` design breaks the would-be cycle.
- `nil` ≠ `None`: no path lets the private `Nil` sentinel surface as `None` (Invariant 4) — a
  fixture asserting user code cannot obtain `nil`, and that `None.class` is the `None` class,
  distinct from any `nil` handling.
- **If niche ruled:** all existing `Option` fixtures stay green; hash coherence for `Some` holds;
  the representation change is invisible to `.ph` code.

## 7. Forward-looking — must NOT preclude
- **NaN-boxing (deferred, ADR-0010):** whatever is decided, `Option` stays *behind* the `Value`
  API so a future boxed layout can encode `None`/`Some` without touching callers.
- **U12 numeric split:** if U12 adds `Value::Int`, and U17 later adds a `None` niche, both must
  coexist in the same enum — sequence them (both touch `value.rs`); the ADR should note the arm
  budget.
- **`Result` parity (values-and-absence §4):** `Result`/`Ok`/`Err` mirror `Option` and are *not*
  niche-encoded — keep the bootstrap story general enough that `Result` uses the same
  blessed-singleton reasoning (`Err` carries an `Error`, so it is never a niche candidate) without
  a second special case.
- **Concurrency:** `None` as a shared singleton is read-only and crosses fibers safely; a niche
  keeps it an immediate (even safer). No shared-mutable concern either way.

## 8. Mandatory rules
- The ADR is the deliverable — full context/decision/consequences/alternatives, citing
  ADR-0007/0010 and the U6/U-STD as-landed reality. If the niche is built, `///` on the new arm +
  every touched match; `cargo doc` clean.
- Green gate = `./scripts/verify.sh` exits 0. Reviewer OFF (docs-heavy) unless the niche is built,
  then recommend reviewer **ON** (it is a `Value`-repr change).
- Own worktree off `main` (docs edits can be in-tree if the orchestrator prefers).

## 9. Return contract
Report: the Q13 niche decision (defer vs build) and rationale · confirmation the ADR pins the
blessed-singleton/no-cycle bootstrap · that `None` stays zero-alloc + identity-comparable + distinct
from `nil` · (if niche) the arm added and match sites touched · files changed · `verify.sh` +
`cargo doc` tails · the DEFERRED entry for the niche if deferred.
