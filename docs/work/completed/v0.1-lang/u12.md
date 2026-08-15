# U12 — Work order: numeric surface split (`Integer` / `Float`) — open-Q2

_Self-contained implementation plan for **one** `phalcom-implementer` agent. Grounds in
open-question **Q2** ([open-questions.md](../spec/open-questions.md#L40)), **ADR-0005**
(single flat `f64` `Number`, which this unit *amends/supersedes*), **ADR-0010** (tagged
`Value` enum), and [object-model.md §4 "Numeric note"](../spec/object-model.md) + [values-and-absence.md §1](../spec/values-and-absence.md). U-STD deliberately wrote `Number`
against an abstract numeric protocol so this split "isn't foreclosed" (PHASE2-INDEX soft-flag)._

> **This unit is BLOCKED-ON-DECISION.** It changes user-visible arithmetic semantics
> (`5 / 2`, `==`, literal typing) and the VM value representation. Do **not** implement the
> split until the user rules DEC-U12 (§4). The scaffolding/ADR work below is what proceeds
> once ruled; the recommendation is stated but not chosen.

---

## 0. Mission (one sentence)
Decide whether Phalcom keeps the single flat `Number` (`f64`, ADR-0005) or introduces a
**surface** `Integer`/`Float` split under an abstract `Number`, and — if the split is ruled in —
realize it as an immediate `Value::Int(i64)` arm plus an abstract `Number` → concrete
`Integer`/`Float` class tower, with defined coercion, without breaking any existing arithmetic
selector or the metaclass tower.

## 1. Hard guardrails
- **Runs on the landed U1–U11 + U-STD substrate.** `Value` is the ADR-0010 tagged enum
  (`Nil, Bool, Number(f64), Symbol, Obj`); arithmetic primitives live in
  `phalcom-core/src/primitive/number.rs`; the `Number` class is `f64`-backed in `core.ph`.
- **Do not touch the metaclass tower wiring.** object-model.md §5 already states "the tower rules
  in §5 already accommodate" an `Integer`/`Float` split — you add classes *under* `Number`,
  you do **not** re-parent kernel classes. `verify_invariants()` must stay green unchanged.
- **One coercion table, one place.** All mixed-arithmetic promotion rules go through a single
  Rust helper (`numeric_binop` or equivalent in `primitive/number.rs`) — do not scatter
  `match (Int, Float)` promotion across opcodes. F8-class divergence (two encoders) is the
  cautionary tale.
- Stay inside the write-set (§3). Out-of-scope ideas → [`DEFERRED.md`](DEFERRED.md).

## 2. Preconditions (verify first)
- `./scripts/verify.sh` green on the worktree base.
- Confirm the numeric literal path: does the lexer/`Token` preserve enough of a numeric
  literal's text for the **compiler** to classify `42` (Integer) vs `42.0`/`4e3` (Float)
  *without* a lexer change? Run `graphify explain "Number literal"` and inspect
  `phalcom-ast/src/lexer.rs` + the compiler's numeric-constant emission. If the token already
  carries the raw slice or a float/int discriminant, U12 stays **compiler-side** and does **not**
  enter the `phalcom-ast` contention chain; if not, a minimal lexer tag is required (adds
  `phalcom-ast` to the write-set — reschedule accordingly).
- Confirm `primitive/number.rs` arithmetic currently keys on `Value::Number(f64)` only.

## 3. Confirmed write-set (validate with `graphify affected "Value"` / `"numeric"` on HEAD)
| File | Why |
|---|---|
| `phalcom-core/src/value.rs` | Add `Value::Int(i64)` arm (if split ruled in); extend `type_name`, `class()`, `value_eq`, `Hash`, `Display`. **Contended with U16/U17** — serialize. |
| `phalcom-core/src/primitive/number.rs` | Split arithmetic/comparison primitives; the single coercion helper; `Integer`/`Float` protocol methods. |
| `phalcom-core/src/primitive/mod.rs` | Register `Integer`/`Float` primitive tables. |
| `phalcom-core/src/universe.rs` | `CoreClasses`: add `integer_class` / `float_class` (keep `number_class` as the abstract parent). |
| `phalcom-core/src/compiler/lib.rs` | Classify numeric literals → emit `Int` vs `Float` constant. **Contended** — serialize. |
| `phalcom-core/core/core.ph` | Abstract `class Number`; `class Integer < Number`; `class Float < Number` skeletons + protocol. **Contended (additive)** — never co-schedule another `core.ph` editor. |
| `phalcom-ast/src/lexer.rs` / `token.rs` | **Only if** the numeric token cannot distinguish int/float (see §2). Prefer to avoid. |
| `phalcom-core/tests/lang.rs` (+ fixtures) | Arithmetic + coercion corpus (§7). |
| `docs/adr/00XX-numeric-surface-split.md` | New ADR amending ADR-0005 (number TBD — see cluster summary; grab next-free at authoring). |
| `docs/spec/object-model.md §4`, `values-and-absence.md §1`, `open-questions.md` Q2 | Flip Q2 to RESOLVED; update the numeric note. |

**Adopted debt (incidental — fix in this unit's `number.rs` pass; was orphaned, no prior owner).**
- `primitive/number.rs:~34` — the string-parse-failure arm of the numeric coercion error hardcodes the
  literal `"value"` instead of the argument's `type_name()` (the sibling arm already uses
  `arg.type_name()`). Correct it while splitting the arithmetic primitives and pin a negative test
  asserting the message names the real type. Trivial, no ADR. Applies whether DEC-U12 rules A or B (the
  arm exists in the status-quo `number.rs`).

## 4. Design decision — **BLOCKED-ON-DECISION (DEC-U12)**
**Question:** single flat `Number` (status quo) vs a surface `Integer`/`Float` split?

| Option | Shape | Cost | Consequence |
|---|---|---|---|
| **A — keep flat `Number` (f64)** | status quo, ADR-0005 stands | zero | `0.1+0.2` rounding; `42` is really a float; no exact large ints / bitwise / clean indexing; **List/Map indices are floats** |
| **B — abstract `Number` → immediate `Integer(i64)` + `Float(f64)`** | new `Value::Int(i64)` arm; `Integer`/`Float` classes under abstract `Number` | new Value arm + coercion rules + literal classification | exact ints, bitwise ops, natural indices; `int⊕int=int`, mixed→float, `/` always Float, add `//` (int div); `==` across int/float by numeric value |
| **C — one `Number` class, two hidden reprs** | `Value::Number` becomes an int-or-float payload, single surface class | coercion without surface classes | no surface `Integer`/`Float` (introspection can't tell them apart) — a half-measure |

**Architect recommendation:** **B, but only if the user wants integer semantics now**
(indexing, bitwise, exact counters). The groundwork already favors B (object-model §4 note;
U-STD's abstract-protocol `Number`). If the user has no concrete driver, **A stands and this
unit closes as an ADR affirming ADR-0005** (mark Q2 resolved-as-flat). Do **not** pick;
the `/`-semantics and literal-typing changes in B are user-facing and irreversible-ish.

**If B is ruled in, the concrete decisions (architect-owned, record in the ADR):**
- `Value::Int(i64)` immediate arm (not boxed) — keeps `Copy`, no heap fetch, mirrors
  `Number(f64)`. NaN-boxing later still folds both behind the same enum API (ADR-0010).
- Coercion: `Int op Int → Int` (overflow → promote to `Float`, or wrap — **sub-decision, rec:
  promote to Float** to avoid silent wrap surprises); `Int op Float → Float`; comparison and
  `==` compare by numeric value so `1 == 1.0` is `true` (rec) — confirm, it affects `hash`.
- `/` is **always** Float division (`5 / 2 == 2.5`); integer division is a distinct selector
  `//(_)` (or `.idiv`). Bitwise (`&`, `|`, `<<`, …) live on `Integer` only.
- Literal typing: a literal with no `.`/exponent is `Integer`; otherwise `Float`. Classified in
  the **compiler**, not the lexer, if the token preserves the raw text (§2).

## 5. Risk
- **Borrow-model / stack fragility (standing risk):** adding a `Value` arm touches every
  exhaustive `match self` on `Value` in the crate — miss one and it won't compile (good) but the
  *semantic* arms (`class()`, `value_eq`, `Hash`, `Display`, `type_name`) must all be updated
  consistently or dispatch silently misroutes.
- **`==`/`hash` coherence:** if `1 == 1.0`, then `1` and `1.0` must `hash` identically or they
  desync `Map`/`Set` keys — a subtle correctness trap.
- **Inliner interaction (ADR-0018):** the sacred-selector inliner may inline `+`/`<` on the fast
  path assuming `Number`. Splitting arms means the inliner's type guard must deopt on
  `Int`-vs-`Float` mismatch exactly like it does for `Bool`. Do not let the inlined `+` skip the
  coercion helper.
- **Overflow policy** is a genuine semantic fork (wrap vs promote) — must be explicit, tested.

## 6. Test strategy (green gate must assert)
- Literal typing: `42.class == Integer`, `4.2.class == Float`, `4e3.class == Float`.
- Coercion: `1 + 2 == 3` (Integer), `1 + 2.0 == 3.0` (Float), `5 / 2 == 2.5`, `5 // 2 == 2`.
- Cross-type equality/hash: `1 == 1.0` is `true`; a `Set` (once available) treats `1`/`1.0` as one
  key (or, if the decision is *not* to unify, the opposite — pin it and test it).
- Overflow: `Integer` at `i64::MAX + 1` behaves per the ruled policy (promote or wrap) — asserted.
- Tower unchanged: `verify_invariants()` green; `Integer.superclass == Number`,
  `Number.superclass == Object`, `Integer.class.superclass == Number.class` (parallel rule).
- Inliner parity: inlined `+`/`<` results equal the non-inlined send results across mixed types.
- Fuzz (opt-in): random int/float arithmetic never panics (overflow path is total).

## 7. Forward-looking — must NOT preclude
- **Bignum / rational (open, not in Draft 0.1):** keep `Number` abstract so a future
  `BigInt`/`Rational` slots in as another `Number` subclass without a third `Value` arm being
  load-bearing at call sites — dispatch stays through the abstract protocol.
- **NaN-boxing (deferred, ADR-0010):** the `Value::Int` arm must be packable behind the same
  enum API; do not expose the representation to callers.
- **Default arguments (U18) / destructuring (U14):** numeric defaults and destructured numbers
  must not assume a single `Number` — write them against the abstract protocol.
- **Concurrency (concurrency.md):** numbers are immediates (`Copy`), so they cross `Fiber`
  boundaries by value — the split adds no shared-mutable state, preserving the no-data-race
  invariant. Keep it that way (no interior mutability on numeric objects).

## 8. Mandatory rules
- `///` on the new `Value::Int` arm, `integer_class`/`float_class`, the coercion helper, every new
  primitive; `//!` refreshed on touched modules; cite ADR-0005 (amended) + the new ADR.
  `cargo doc --workspace --no-deps` adds no warnings.
- Green gate = `./scripts/verify.sh` exits 0 (reviewer OFF unless the orchestrator flips it —
  this is load-bearing arithmetic, recommend reviewer **ON**).
- Own isolated worktree off `main`.

## 9. Return contract
Report: the DEC-U12 resolution actually implemented · the coercion + `/` + overflow policy chosen
· whether a lexer change was needed · confirmation `verify_invariants()` is unchanged-green · the
inliner deopt-guard interaction · files changed · `verify.sh` + `cargo doc` tails · new `DEFERRED`
entries (bignum, `//` naming if unresolved).
