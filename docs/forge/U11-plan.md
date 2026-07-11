# U11 — Work order: `Bool` tower (abstract `Bool` + `True`/`False`) (dispatch-ready)

_Self-contained implementation plan for **one** `phalcom-implementer` agent. **Reviewer OFF** — self-verify
on the green gate + the un-ignored `booleans` `lang` label. Governed by **ADR-0004** (booleans as an abstract
`Bool` with concrete `True`/`False`). Scheduled **Wave F+1** (after U-STD lands on `core.ph`, alongside U9
variadics). Depends on U2 (tower), U-STD (`core.ph` base surface), U6 (`Option`), and interacts with U5
(control-flow-as-message)._

---

## 0. Mission (one sentence)
Split the single `Bool` class into an **abstract `Bool` superclass with two singleton subclasses `True` and
`False`** per ADR-0004, so that `true.class == True`, `false.class == False`, and boolean control selectors
dispatch by class instead of by a VM `if` — **without a new `Value` variant**.

## 1. Hard guardrails (read before writing any code)
- **No `Value` representation change.** ADR-0004 is explicit: `Value::Bool(b)` stays; the class is *selected*
  from the payload at runtime (`true → True`, `false → False`). Do **not** add a `Value` variant, do not
  touch the tagged-`Value` layout (ADR-0010) — that is U1's substrate.
- **`true`/`false` are the sole instances.** `True` and `False` are singleton classes with exactly one
  instance each; there is no `True.new`. Wire the two singleton values in the universe bootstrap.
- **Do NOT re-wire the metaclass tower.** U2 owns `verify_invariants()`; you *add* `Bool`/`True`/`False` as
  ordinary classes into the already-correct tower via U2's `(name, superclass)` helper — `Bool` super `Object`,
  `True`/`False` super `Bool` — and the invariant harness must still pass afterward.
- **Coordinate with U5 (control-flow-as-message + inliner).** U5 inlines the *sacred* boolean selectors
  (`ifTrue:`, `and:`, …) with a deopt guard. U11 provides the **non-inlined method fallback** on `True`/`False`
  that the deopt path lands in. Your methods must match the selectors/semantics U5 inlines. If U5 has not
  landed the inliner yet, define the methods anyway — they are the source of truth the inliner mirrors.
- **`core.ph` is a shared, additive file.** U-STD (Wave F) has already added the non-Bool base surface; you
  add only the `Bool`/`True`/`False` block. Do not rewrite U-STD's classes.
- Stay inside the write-set (§3); on any forced overreach, **STOP and report a conflict**.

## 2. Preconditions (verify first; do not assume)
- Runs in an **isolated worktree off `main`** seeded with U2 + U6 + U-STD merged. Confirm
  `./scripts/verify.sh` green and that `verify_invariants()` passes before your first edit.
- **graphify-first:** `graphify explain "boolean"`, `graphify affected "Bool"`, and
  `graphify path "VM" "class_of"` (or the current class-of-value routine) on HEAD, to locate exactly where a
  `Value::Bool` is mapped to its class — that routine is the one you branch on `b`.
- Read [ADR-0004](../adr/0004-boolean-as-abstract-bool-with-true-false.md) and object-model §4 (Bool row) +
  values-and-absence §3.1 (which mirrors Option's shape onto Bool/True/False) — note the spec-doc drift in §7.

## 3. Confirmed write-set
| File | Why it's in scope |
|---|---|
| `phalcom-core/src/boolean.rs` | Currently only `TRUE`/`FALSE` consts. Add the True/False singleton value handles / class-selection helper. |
| `phalcom-core/src/primitive/boolean.rs` | Replace the buggy `bool_class_new` (has stray `println!` debug + coerces `Nil`) with the correct `True`/`False` primitives (`not`, `and(_:)`, `or(_:)`, `ifTrue(_:)`, `ifTrue(_:)ifFalse(_:)` backing as needed). |
| `phalcom-core/src/universe.rs` (bootstrap) | Register `Bool`/`True`/`False` classes into the tower; select `True`/`False` as the class of `Value::Bool(true/false)`; bind the singleton globals. |
| `phalcom-core/core/core.ph` | Define `Bool` (abstract) + `True`/`False` with the control selectors in Phalcom (dispatch-by-class). **Additive block only.** |
| `phalcom-core/tests/lang.rs` | Un-ignore the `booleans` label (drop the `#[ignore]`). |
| `phalcom-core/tests/lang/booleans/*` | Promote cases from `booleans/pending/` and add class-identity + short-circuit + Option-return cases. |

## 4. Design decisions (grounded in ADR-0004 + object-model §4 + values-and-absence §3)

### D1 — Class shape (ADR-0004)
- `Bool` **abstract**, super `Object`. `True` and `False` **concrete singleton subclasses**, super `Bool`.
- Class-of selection: the routine mapping a `Value::Bool(b)` to its class returns `True`-class handle when
  `b`, else `False`-class handle. Store both class handles + both singleton values on the `Universe`.

### D2 — Control selectors dispatch by class (object-model §4; control-flow.md)
Each boolean operation is **two method definitions**, one per subclass — no runtime `if`:
- `True>>not → false`, `False>>not → true`.
- `True>>and(b) → b`, `False>>and(b) → false` (short-circuit: `False>>and:` ignores its arg);
  `True>>or(b) → true`, `False>>or(b) → b`. Mirror the `and`/`or` keyword short-circuit from U5.
- **`ifTrue`/`ifFalse` return `Option`** (object-model §4, values-and-absence §3): `True>>ifTrue(blk) →
  Some(blk.call)`, `False>>ifTrue(blk) → None`; and the paired `ifTrue(_:)ifFalse(_:)`. **This is why U11
  depends on U6 (`Option`/`Some`/`None`).**

### D3 — `==` / identity
`true`/`false` are identity-comparable singletons; `==` on booleans is identity (same as `None`).

### D4 — Bootstrap ordering
Register `Bool` before `True`/`False`; wire the two singletons and the class-of-bool selection **before**
`verify_invariants()` runs so the harness sees a consistent tower. Add `Bool`/`True`/`False`/`Number` class
identity to the invariant sanity checks where U2 left hooks.

## 5. Build order (each step verifies green before the next)
1. **`boolean.rs` + `universe.rs`** — add `True`/`False` classes to the tower, singleton globals, and the
   class-of-`Value::Bool` selection. Confirm `verify_invariants()` still passes.
2. **`primitive/boolean.rs`** — delete the debug `println!`s and the `Nil` coercion; implement the correct
   primitives (or leave control selectors to pure-Phalcom in step 3 if no native backing is needed).
3. **`core.ph`** — the `Bool`/`True`/`False` block with `not`/`and`/`or`/`ifTrue`/`ifFalse` per D2.
4. **Un-ignore `booleans`** in `lang.rs`; promote `booleans/pending/*` and add cases: `true.class == True`,
   `false.class == False`, short-circuit (`and`/`or` do not evaluate the dead branch), and
   `(x>0).ifTrue { … }` returning `Some`/`None`. Verify green + golden byte-identical.

## 6. BLOCKED-ON-DECISION
- **None hard-blocking** — ADR-0004 is Accepted and authoritative (STATE.md ADR mapping); values-and-absence
  §3.1 explicitly commits to the `Bool`/`True`/`False` shape.
- **Spec-doc reconciliation (flag, not a blocker):** [object-model.md](../spec/object-model.md) §3 (value
  table row "true/false → `Bool`, one class") and §4 ("users see one class, `Bool`"; True/False "may be
  realized internally") are **in tension** with ADR-0004 + values-and-absence §3.1, which make `True`/`False`
  **surface-visible real subclasses** (`true.class == True`). **Recommendation: follow ADR-0004** (True/False
  visible) — it is the authoritative, more recent decision and the one values-and-absence already mirrors.
  **Note for the `documentation-and-adrs` skill:** update object-model.md §3/§4 to state `true.class == True`
  (not `Bool`) and drop the "users see one class" wording, so the spec is internally consistent. This is a
  doc edit, outside U11's code write-set — file it, do not silently edit the spec here.

## 7. Risks
- **Inliner interaction (U5):** if U5's inlined `ifTrue:`/`and:` fast path and U11's `True`/`False` methods
  disagree on selector or Option-return semantics, the deopt path breaks. Pin both with the same corpus cases.
- **Invariant harness:** adding two classes to the tower must not perturb the metaclass parallel-rule checks
  U2 installed — re-run `verify_invariants()` after step 1, not just at the end.
- **`class_of` hot path:** the `Value::Bool → class` branch is on a hot dispatch path; keep it a cheap
  handle-select, no allocation.

## 8. Mandatory rules
- **Docs:** full rustdoc on every touched/added Rust item (`boolean.rs`, `primitive/boolean.rs`,
  `universe.rs` additions) with ADR-0004 citations; `core.ph` block carries a spec-referencing `//` comment.
  `cargo doc --workspace --no-deps` clean.
- **Green gate:** `./scripts/verify.sh` exits 0 — `booleans` label un-ignored and green, `verify_invariants()`
  passes, golden byte-identical, no new clippy warnings.
- **graphify update** `.` `--no-cluster` after edits.

## 9. Return contract (self-verify; reviewer OFF)
Report: the `Bool`/`True`/`False` wiring + class-of-bool selection · which selectors are pure-Phalcom vs
primitive-backed · proof `verify_invariants()` still passes and `booleans` is green (`verify.sh` tail) ·
`cargo doc` tail · the object-model.md reconciliation filed for the docs skill · explicit confirmation you
added **no `Value` variant** and did not re-wire the U2 tower.
