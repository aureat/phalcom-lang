# U16 — Work order: method references (`::`) + Family introspection — open-Q14

_Self-contained plan for **one** `phalcom-implementer` agent. Grounds in open-question **Q14**
([open-questions.md](../spec/open-questions.md#L102)) and [selectors.md §3 "Method references
(`::`)"](../spec/selectors.md#3-method-references-) + §3.1 (base-name index), **ADR-0012**
(selector encoding — `::` reuses `encode_selector`), and U8's landed `Message`/`doesNotUnderstand`/
`perform`/`send_dynamic` machinery (the candidate-enriched miss path). The `::` token **exists**
(`phalcom-ast/src/token.rs:149` `Token::ColonColon`) but there is **no `Expr::MethodRef`, no
`Family` value, and no base-name index** (confirmed: `grep base_name` in `phalcom-core/src` is
empty). So this unit **builds the `::`/`Family` feature itself**, then answers Q14 on top of it._

---

## 0. Mission (one sentence)
Implement `::` method references — producing a callable **`Family`** value (Open/Pinned, bound or
unbound per selectors §3) whose call *is an ordinary send* — build the per-class **base-name
index** (selectors §3.1) that backs the reference-time empty-family check and the candidate list
in `doesNotUnderstand` errors, and expose a **minimal reflective surface** on `Family` (Q14) so it
is a first-class object, not merely error-message plumbing.

## 1. Hard guardrails
- **A Family call IS a send.** selectors §3: there is no second dispatch mechanism. An *Open*
  family builds its selector at call time from `family.name` + the call-site labels and enters the
  ordinary send path; a *Pinned* family uses its fixed selector directly. Do **not** add a parallel
  dispatch path — route both through the existing send / `send_dynamic` (U8).
- **One encoder.** The call-time selector is built via `encode_selector` (ADR-0012 / F8). Never
  hand-format a selector string.
- **Empty-family check honors `doesNotUnderstand`.** `obj::typo` is an error *at reference time*
  **only if** the class has no method named `typo` **and** no `doesNotUnderstand` hook (selectors
  §3 error table). If a DNU hook exists, the family is callable and routes to it. Do not
  hard-error when a DNU hook is present.
- **Base-name index is flattened through inheritance** (selectors §3.1), built at
  class-finalization — the same finalization seam U13 touches. Coordinate so there is one
  finalization hook.
- Stay inside the write-set (§3).

## 2. Preconditions (verify first)
- `./scripts/verify.sh` green.
- U8 landed: `graphify explain "doesNotUnderstand"` / `graphify explain "send_dynamic"` — confirm
  `VM::send_dynamic`, `Message` reification, and the miss path exist (they do — `vm.rs` L456+).
- `graphify explain "class finalization"` — locate where a class's method dict is finalized, to
  hang the `base_names` build. Confirm `base_names` truly does not exist yet.
- Confirm the parser's postfix/primary expression path so `expr :: name` and `expr :: #sel(...)`
  parse with `#`-lookahead (selectors §3: "after `::`, peek for `#`"), and unbound `Type::name`.
- Decide the `Family` runtime home: a new `Object::Family` heap variant (`heap.rs`) vs a
  `Value` arm. **Recommendation: a heap `Object::Family` variant** (families are not hot immediates
  and carry a `recv: Option<Value>` + a `Symbol`), mapping to a `Family` class via `value.rs`
  `class()`. Do **not** add a `Value` arm — keep `Value` minimal.

## 3. Confirmed write-set (validate with `graphify affected` on HEAD)
| File | Why |
|---|---|
| `phalcom-ast/src/ast.rs` | `Expr::MethodRef { receiver: Option<Box<Expr>>, target: NameOrSelector }` (selectors §6). **Contended (`phalcom-ast`)** — serialize. |
| `phalcom-ast/src/parser.rs` | Postfix `::` with `#`-lookahead; unbound `Type::name`; Open vs Pinned. |
| `phalcom-core/src/heap.rs` | `Object::Family { recv: Option<Value>, kind: FamilyKind }` (Open{name} / Pinned{selector}). **Contended (`value.rs`/`heap.rs` group with U12/U17)** — serialize. |
| `phalcom-core/src/value.rs` | `class()` maps `Object::Family` → `family_class`; `type_name`. **Contended** — serialize. |
| `phalcom-core/src/class.rs` | Build + store the flattened `base_names: HashMap<Symbol, SmallVec<[Symbol;2]>>` at finalization; the empty-family accessor. **Contended with U13** — serialize. |
| `phalcom-core/src/universe.rs` | `family_class` in `CoreClasses`. |
| `phalcom-core/src/vm.rs` | The Family call: Open → build selector → send; Pinned → send; reference-time empty check; enrich the miss with candidates. **Contended** — serialize. |
| `phalcom-core/src/compiler/lib.rs` | Compile `Expr::MethodRef` → construct a `Family`; emit the constant label-suffix for Open call sites. **Contended** — serialize. |
| `phalcom-core/src/primitive/*.rs` (new `family.rs`) | `Family` reflective + call protocol (Q14 surface). |
| `phalcom-core/core/core.ph` | `class Family` skeleton (docs/protocol). **Contended (additive)** — serialize. |
| `phalcom-core/tests/lang.rs` (+ fixtures) | `::`/Family corpus (§6). |
| `docs/adr/00XX-method-references-and-family.md` | New ADR realizing selectors §3 + the Q14 surface — provisional number, grab next-free. |
| `docs/spec/open-questions.md` Q14, `docs/spec/selectors.md §3` | Flip Q14 to RESOLVED; mark §3 implemented. |

## 4. Design decision
**Core (`::`/Family) — architect-owned, realize per selectors §3:**
- `Family` = `Open { recv: Option<Value>, name: Symbol }` | `Pinned { recv: Option<Value>,
  selector: Symbol }`; `recv: None` = unbound (receiver becomes the first call argument).
- **Open resolves at call time** (never stale, subclass overrides work); **Pinned** goes straight
  to the send (fast path, names one overload).
- **base-name index** (§3.1): `HashMap<Symbol /* "move" */, SmallVec<[Symbol;2]> /* selectors */>`,
  built per class at finalization, flattened through inheritance. Serves the empty-family check,
  the candidate list in DNU errors, and reflection.
- Call-time miss where supplied labels are a strict subset of exactly one candidate → report the
  *specific* missing label (selectors §3 error table), not the whole candidate list.

**Q14 — Family reflective surface — soft flag (recommend, confirm if disagreed):**
| Option | Surface on `Family` | Cost |
|---|---|---|
| **Minimal** | error-enrichment only; `Family` is opaque to user code | least |
| **Small reflective (recommended)** | `name` → name symbol; `candidates` → `List` of selector symbols (straight from `base_names`); `isBound` → `Bool`; `receiver` → `Option`; `isPinned`/`selector` for Pinned | small — data already exists |
| **Rich** | per-candidate arity/`Method` objects, parameter labels, doc strings | large — pulls in `Method` reflection |

**Recommendation: Small reflective surface.** The `base_names` index already holds the candidate
list "for free," so exposing `name`/`candidates`/`isBound`/`receiver` is nearly zero marginal cost
and makes `::` genuinely reflective (respondsTo-style queries, tooling). Defer per-candidate
`Method` objects to a future reflection unit. This is a soft flag — proceed on the recommendation;
confirm only if the user wants the minimal or rich variant.

## 5. Risk
- **Open-family call cost:** selectors §3 promises "same cost as a normal send" via a constant
  label-suffix + a monomorphic IC keyed by `(call_site, class_id)`. IC population is deferred
  (ADR-0012), so today the Open call re-interns per call — acceptable, but keep the *shape*
  IC-ready (don't bake in a slow re-intern that a later IC can't bypass).
- **Reference-time vs call-time error split** is subtle: empty-family (no such base name, no DNU) →
  error at `::`; label mismatch on a non-empty family → ordinary DNU at call. Getting these
  reversed is a spec violation — test both.
- **base_names ⊗ U13 finalization:** both hang off class-finalization. If U13 (traits, if ruled)
  flattens methods there too, the base-name index must be built *after* the flatten or it misses
  trait methods. Sequence U16 after U13 or share the hook explicitly.
- **Standing heap risk:** `Family` carries a `recv: Value` that may be an `Obj` handle — ensure it
  is traced/kept-alive by the heap like any other object field (no dangling handle if a GC lands).

## 6. Test strategy (green gate must assert)
- Open bound: `let f = p::move; f(to: q, duration: 2)` dispatches `move(to,duration)`;
  `f(q, 2)` dispatches `move(_,_)` — proves call-time selector building from labels.
- Open unbound: `Point::move` applied with a receiver arg dispatches on the **actual** receiver
  (subclass override wins).
- Pinned: `p::#move(_,to,duration)` calls exactly that selector, no re-intern.
- Empty family: `obj::typo` (no method, no DNU) errors **at reference time**, names the class;
  `obj::typo` on a class **with** a DNU hook is callable and routes to DNU (not an error).
- Candidate enrichment: a call-time label miss produces a DNU whose message includes the family's
  candidate selectors; a strict-subset miss names the specific missing label.
- Q14 surface: `p::move.name == #move`; `p::move.candidates` is a `List` including `move(_,_)`;
  `p::move.isBound == true`; `Point::move.isBound == false`.
- Finalization: base-name index is inheritance-flattened (a subclass's family includes inherited
  selectors).

## 7. Forward-looking — must NOT preclude
- **`Family` vs `Method.bind` unification (functions §3, open):** two routes to a bound callable
  coexist today. Keep `Family` a `Function`-conforming callable so a later unification does not
  require reshaping it. (overlay: "`Family` vs `Method.bind` unification — OPEN, two routes
  coexist".)
- **Inline-cache population (deferred, ADR-0012):** keep the Open call site's IC slot shape so
  populating it later is not a redesign.
- **U15 (modules):** module members reached via normal sends means `module::member` composes with
  Family for free — do not special-case module access out of the send path.
- **U13 (hierarchy):** the base-name index assumes a finalized hierarchy; if U13 rules mutability
  in, the index must be rebuilt/invalidated on `superclass=` (same IC-epoch seam). Note it.
- **Concurrency (concurrency.md):** a `Family` value crosses fiber boundaries by handle; its call
  is an ordinary (fiber-local) send — introduces no shared mutable state. Keep it that way.

## 8. Mandatory rules
- `///` on `Expr::MethodRef`, `Object::Family`/`FamilyKind`, `base_names` + its accessor, every
  reflective primitive; `//!` refreshed; cite selectors §3 + ADR-0012 + the new ADR. `cargo doc`
  clean.
- Green gate = `./scripts/verify.sh` exits 0. Reviewer OFF unless orchestrator flips it.
- Own isolated worktree off `main`.

## 9. Return contract
Report: the Family runtime representation (heap variant vs other) · the Q14 surface built
(minimal/small/rich) and why · confirmation the Family call reuses the ordinary send / `send_dynamic`
(no second dispatch path) · the reference-time vs call-time error split, quoting the seam · the
finalization hook shared with U13/base_names · files changed · `verify.sh` + `cargo doc` tails ·
DEFERRED entries (per-candidate `Method` reflection, IC population, `Family`/`Method.bind`
unification).
