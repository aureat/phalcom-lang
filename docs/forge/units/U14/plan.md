# U14 — Work order: destructuring bindings — open-Q7

_Self-contained plan for **one** `phalcom-implementer` agent. Grounds in open-question **Q7**
([open-questions.md](../spec/open-questions.md#L64)), **ADR-0014** (`let`/`var` bindings),
[values-and-absence.md §1](../spec/values-and-absence.md) (Tuple/List value types), and
[messages-and-selectors.md §4/§5](../spec/messages-and-selectors.md) (rest `*name`, spread).
This unit is **architect-decidable** (no user ruling needed on the core), but has a **hard
dependency on the collection-literals unit** being planned concurrently (Tuple/List literals +
the `(a,b)` / `[…]` grammar) and on a runtime `Tuple` type, which **does not exist today**
(confirmed: no `Tuple` in `value.rs` `Object` or `phalcom-ast`)._

---

## 0. Mission (one sentence)
Let a `let`/`var` binding destructure a tuple or list — `let (a, b) = point` and
`let [first, *rest] = list` — by desugaring the pattern into a single evaluation of the RHS
followed by positional element reads, reusing the collection-literals unit's pattern grammar and
the existing `*rest` machinery, as an **irrefutable** binding (a shape mismatch is a runtime
error, not a pattern failure — there is no `match` yet).

## 1. Hard guardrails
- **Depends on the collection-literals unit.** That unit (planned by another agent) owns
  `Expr::Tuple`/`Expr::List` literal *construction* and the `(a,b)` / `[…]` grammar, and should
  introduce the runtime `Tuple` type. **STOP and report** if it has not landed — U14 reuses its
  AST/grammar for the *left-hand* pattern and must not fork a second tuple parser.
- **Irrefutable only.** `let (a,b) = expr` binds `a`,`b` by position; if the RHS has the wrong
  arity it raises a clean runtime error (rec: `ArgumentError`/`RangeError`), it does **not**
  silently produce `None` and it is **not** a boolean-testable pattern. Refutable patterns and a
  `match` expression are a separate future unit (see §7).
- **One evaluation of the RHS.** `let (a,b) = f()` calls `f` **once**, into a temp, then reads
  elements. Do not re-evaluate the scrutinee per binding.
- **No new absence semantics.** A destructured slot that reads `None` is just `None` (ADR-0007);
  destructuring does not introduce a new sentinel.
- Stay inside the write-set (§3).

## 2. Preconditions (verify first)
- `./scripts/verify.sh` green.
- Collection-literals unit landed: confirm `Expr::Tuple`/`Expr::List`, the runtime `Tuple` type,
  and the element-access protocol it exposes (`at(_)` / index / a `.size`). Run
  `graphify explain "tuple"` / `graphify explain "list literal"` on HEAD.
- `let`/`var` binding lowering (U6, ADR-0014) — confirm where a `let` binding compiles
  (`compiler/lib.rs`) and how a binding target is emitted, so a *pattern* target slots in.
- `*rest` collection into a `List` (U9) exists — the list-tail pattern `[first, *rest]` reuses it.

## 3. Confirmed write-set (validate with `graphify affected` on HEAD)
| File | Why |
|---|---|
| `phalcom-ast/src/ast.rs` | Add a binding **pattern** node — `Pattern::{ Name, Tuple(Vec<Pattern>), List(Vec<Pattern>, rest: Option<Box<Pattern>>) }` — and let `let`/`var` take a `Pattern`, not just a name. **Contended (`phalcom-ast`)** — serialize with U15/U16/U18. |
| `phalcom-ast/src/parser.rs` | Parse the LHS pattern (reuse the collection-literals `(…)`/`[…]` grammar in pattern position); reject interior `*` (rest must be last, like U9). |
| `phalcom-core/src/compiler/lib.rs` | Desugar: eval RHS → temp; for each sub-pattern emit an element read + bind; `*rest` → slice/drop of the tail into a `List`. **Contended** — serialize. |
| `phalcom-core/tests/lang.rs` (+ fixtures) | Destructuring corpus (§6). |
| `docs/adr/00XX-destructuring-bindings.md` | New ADR (desugaring protocol + irrefutable semantics) — provisional number, grab next-free. |
| `docs/spec/open-questions.md` Q7 + a short `destructuring.md` (or a §in a binding spec) | Flip Q7 to RESOLVED; document the desugaring. |

## 4. Design decision (architect-owned — realize, record in the ADR)
- **Desugaring target = a positional element-read protocol.** `let (a,b) = t` compiles to
  `let $t = t; let a = $t.<elem 0>; let b = $t.<elem 1>`. Choose the element accessor from what
  the collection-literals unit exposes — **preferred: the same `at(_)` used by List/Tuple
  indexing** (keeps one indexing path; ADR-0020 List already has native `at`). Do **not** invent a
  parallel `_0`/`_1` field protocol unless the collection-literals unit already ships one.
- **List rest:** `let [first, *rest] = xs` → `first = xs.at(0)`; `rest = xs` tail from index 1
  (reuse the U9 rest-collection / a `List` slice). `*rest` **must be last** (same rule as U9).
- **Arity checking:** tuple patterns require an exact-arity RHS (a `(a,b)` against a 3-tuple is a
  runtime error); list patterns with a `*rest` require `>= fixedCount`. Emit the check inline.
- **Nesting:** `let ((a,b),c) = …` — support nested patterns recursively (the pattern node is
  already recursive). Nesting is free once the recursion is right; include it.
- **`var` vs `let`:** a destructuring `var` binds mutable slots; `let` immutable — inherit
  ADR-0014, no new rule.

**Soft flag (confirm, do not block):** whether the accessor is `at(_)` vs a dedicated
`destructure`/iterator protocol. Recommendation: `at(_)`. If the user or the collection-literals
unit prefers an iterator-based spread (so *any* `Iterable` destructures, not just Tuple/List),
that is a strictly larger surface — note it and default to `at(_)` on concrete Tuple/List.

## 5. Risk
- **Double-evaluation bug:** the single subtle correctness point — the RHS must hit a temp exactly
  once. A fixture must prove `let (a,b) = sideEffect()` runs the effect once.
- **Grammar ambiguity with collection literals:** `(a, b)` is both a tuple literal (RHS) and a
  pattern (LHS). The parser must interpret `(…)`/`[…]` as a *pattern* only in binding-target
  position. Reusing the collection-literals grammar in the wrong position mis-parses. Keep the
  pattern parse path distinct-but-shared.
- **Empty/one-element cases:** `let (a,) = …` (1-tuple) and `let () = …` — pin whether these are
  legal (rec: follow the collection-literals unit's tuple arity rules exactly).

## 6. Test strategy (green gate must assert)
- `let (a, b) = (1, 2)` → `a==1`, `b==2`.
- `let [first, *rest] = [1, 2, 3]` → `first==1`, `rest == [2,3]` (a real `List`, `rest.size==2`).
- Nested: `let ((a, b), c) = ((1, 2), 3)` binds all three.
- Single-eval: `let (a, b) = counterTuple()` invokes the producer exactly once (assert via a
  side-effect counter).
- Arity mismatch: `let (a, b) = (1, 2, 3)` raises a clean error, not a panic; `let [a,b] = [1]`
  likewise.
- `var` destructuring binds mutable slots (`a = a + 1` after `var (a,b) = …` works).
- Rest-not-last: `let [*rest, last] = …` is a parse error with a precise span.

## 7. Forward-looking — must NOT preclude
- **A future `match` / refutable patterns:** the `Pattern` AST node introduced here is the same
  node a later `match` will reuse. Design it so a *refutable* variant (returning success/failure
  rather than raising) can be added without reshaping it — keep the "raise on mismatch" logic in
  the *lowering*, not baked into the node. (overlay hazard: "adding real `match` must respect
  selector identity + open-classes/sealedness tension".)
- **Q4/Q10 (U13):** destructuring reads elements via a message send (`at(_)`), so it inherits
  whatever dispatch policy U13 settles — it makes no independent assumption about class stability.
- **U12 numeric split:** destructured numeric slots must not assume a single `Number`; they hold
  whatever the element is.
- **Concurrency (concurrency.md):** the desugared temp is frame-local, so destructuring is
  fiber-local and introduces no shared state — keep the temp on the value stack, not a global.
- **Spread parity (messages §5):** `*rest` in a pattern and `*args` in a call should feel
  symmetric; reuse the U9 rest machinery so the two never diverge.

## 8. Mandatory rules
- `///` on the `Pattern` node + variants, the desugaring helper; `//!` refreshed; cite ADR-0014 +
  the new ADR + the collection-literals ADR. `cargo doc` clean.
- Green gate = `./scripts/verify.sh` exits 0. Reviewer OFF unless orchestrator flips it.
- Own isolated worktree off `main`.

## 9. Return contract
Report: the element-accessor protocol chosen (`at(_)` vs other) · confirmation of single RHS
evaluation + the guarding fixture · irrefutable-mismatch behavior (which error) · the
collection-literals unit version it built on · files changed · `verify.sh` + `cargo doc` tails ·
any DEFERRED entries (refutable patterns / `match`, iterator-based destructuring).
