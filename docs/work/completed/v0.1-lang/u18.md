# U18 — Work order: default arguments — open-Q12

_Self-contained plan for **one** `phalcom-implementer` agent. Grounds in open-question **Q12**
([open-questions.md](../spec/open-questions.md#L82)) and [selectors.md §7 item 3
"Default arguments"](../spec/selectors.md#7-open-questions-not-decided), **ADR-0012** (label-encoded
selectors — the mechanism default args collide with), and the U9 variadics machinery (shared
param-list + call-prologue). selectors §7.3 flags this **"decide before shipping"** because
retrofitting after selector identity is load-bearing is expensive. **This unit is
BLOCKED-ON-DECISION** on whether Phalcom supports default arguments at all, and if so, the
expansion policy._

---

## 0. Mission (one sentence)
Decide whether a method/block parameter may declare a default value (`move(x, y = 0)`), and — if
ruled in — realize it via **definer-side arity-family expansion**: at method-definition time the
compiler synthesizes one selector per legal omission of trailing defaults, each pointing at the
**same** compiled body whose prologue supplies the missing defaults, so every call still hits an
exact selector on the one-hashmap-probe fast path (ADR-0012) with **no** caller-side or
dispatch-time change.

## 1. Why this is hard (the committed-design collision — read first)
Selector identity is `name + labels` (ADR-0012, Invariant 2). A call that **omits** a defaulted
argument produces a **different selector** (`move(_)` vs `move(_,_)`), so a single full-arity
method `move(_,_)` is **not found** by a `move(_)` call. The two candidate resolutions in
selectors §7.3 are:
- **arity-family expansion** — register one method per omitted-argument combination (feared
  "combinatorial"); but the *definer* statically knows every default, so this is **definer-side
  work, bounded to k+1** if only *trailing* defaults with *left-to-right* omission are allowed;
- **static callee knowledge at the call site** — **unavailable** under dynamic dispatch (the
  callee is not known until dispatch). Rejected by the spec's own reasoning (selectors R4).

The plannable path is therefore **trailing-only, left-to-right-omission arity-family expansion**,
which is bounded (not `2^k`) and keeps dispatch untouched.

## 2. Design decision — **BLOCKED-ON-DECISION (DEC-U18)**
**Question 1 — support default arguments at all?**
| Option | Consequence |
|---|---|
| **A — no default arguments** | selector identity stays pristine; callers must pass all args or the definer provides overloads by hand; simplest, most Smalltalk-honest |
| **B — trailing-only defaults via definer-side expansion (recommended if wanted)** | ergonomic defaults; bounded k+1 synthesized selectors sharing one body; **no** dispatch/caller change |
| **C — arbitrary-position defaults / keyword defaults** | rejected — interior optional positionals violate R2 (positionals precede labels) and blow up combinatorially; not admissible |

**Question 2 (only if B) — expansion policy:** confirm **trailing-only + left-to-right omission**
→ for k trailing defaults, synthesize selectors for arities `full, full-1, …, full-k` (k+1 total),
each dropping the rightmost defaults. A labelled defaulted parameter (`to: = p`) defaults the
*label's* slot but the same trailing/left-to-right rule applies among labels.

**Architect recommendation:** if the user wants the ergonomics, **B with trailing-only +
left-to-right omission**; otherwise **A**. Do **not** pick — default arguments are a permanent
surface-language commitment that, once shipped, other code depends on (selectors §7.3's "expensive
to retrofit" cuts both ways). Present A/B to the user.

## 3. Hard guardrails (apply if B is ruled in)
- **Runs on the landed U3 (selectors) + U7 (construct) + U9 (variadics) substrate.** Reuse
  `encode_selector` (ADR-0012) for every synthesized selector — never hand-format.
- **Trailing-only.** A default may appear only on a **trailing** run of parameters:
  `move(x, y = 0, z = 1)` is legal; `move(x = 0, y)` is a **compile error** (a required param after
  a defaulted one). Enforce at compile time with a clean diagnostic.
- **Default ⊗ rest is exclusive.** A parameter list may not combine a defaulted param and a `*rest`
  param ambiguously (both consume trailing args) — reject `foo(a = 0, *rest)` at compile time.
  Coordinate with U9's rest-param rules; they share the param-list parser.
- **One body, N selectors.** All synthesized selectors resolve to the **same** `MethodObject`; the
  short-arity entry's prologue pushes the default expressions for the omitted trailing slots before
  the shared body runs. Do **not** duplicate the body.
- **Defaults are expressions evaluated at call time, in the callee frame** (like Python's
  *evaluated per call*, not a shared mutable default — avoid the Python mutable-default footgun by
  evaluating the default expression each call, not once at definition).
- Stay inside the write-set (§4).

## 4. Confirmed write-set (validate with `graphify affected "ParameterDef"` / `"encode_selector"`)
| File | Why |
|---|---|
| `phalcom-ast/src/ast.rs` | `ParameterDef.default: Option<Box<Expr>>`. **Contended (`phalcom-ast`)** — serialize with U14/U15/U16. |
| `phalcom-ast/src/parser.rs` | Parse `name = expr` / `label: = expr` in a param; reject required-after-default; reject default+rest. |
| `phalcom-core/src/signature.rs` | Account for min-vs-max arity (the defaulted range); the synthesized-selector set. Reuse U9's `min_positional_arity`. |
| `phalcom-core/src/method.rs` | The shared `MethodObject` registered under multiple selectors; carry the default-expr thunks / prologue metadata. |
| `phalcom-core/src/compiler/lib.rs` | Synthesize the k+1 selectors; compile the shared body; emit the default-supplying prologue per short-arity entry; enforce the compile-time rules. **Contended** — serialize. |
| `phalcom-core/src/vm.rs` | Only if the default-supplying prologue needs a VM-side reshape (prefer to bake it into the compiled prologue like U9's rest collection). **Contended** — serialize. |
| `phalcom-core/tests/lang.rs` (+ fixtures) | Default-arg corpus (§6). |
| `docs/adr/00XX-default-arguments.md` | New ADR (extends ADR-0012) — provisional number, grab next-free. |
| `docs/spec/open-questions.md` Q12, `docs/spec/selectors.md §7`, `messages-and-selectors.md §3` | Flip Q12 to RESOLVED; document the expansion. |

## 5. Risk
- **Combinatorial blow-up if the trailing-only rule leaks:** allowing interior or arbitrary-subset
  defaults reintroduces the `2^k` explosion selectors §7.3 warns of. The parser-level enforcement
  of "trailing-only, left-to-right" is the guardrail — test it hard.
- **Selector collision:** a synthesized `move(_)` may collide with a *hand-written* `move(_)` on
  the same class. Decide + document: rec **reject at definition** (a defaulted method whose
  synthesized arity duplicates an existing selector is a compile error) — do not silently
  overwrite (unlike U9's duplicate-variadic "last wins", defaults are less obviously intentional).
- **Default-expr scope:** a default expression may reference earlier parameters
  (`f(a, b = a + 1)`) or only outer scope — pin it (rec: may reference earlier params, evaluated
  in the callee frame after earlier params are bound). Getting the evaluation frame wrong is the
  subtle correctness point.
- **Default ⊗ variadic ⊗ inliner:** the sacred-selector inliner (ADR-0018) must not inline a
  synthesized short-arity selector without running its default-supplying prologue.

## 6. Test strategy (green gate must assert)
- `move(x, y = 0)`: `p.move(1, 2)` → uses 2; `p.move(1)` → `y` defaults to 0 — both dispatch to the
  **same** body (assert one `MethodObject`, two selectors).
- Two defaults: `f(a, b = 1, c = 2)` callable with 1, 2, or 3 args (k+1 = 3 entries); `f(a)` and
  `f(a, b)` and `f(a, b, c)` all work; `f()` errors (required `a`).
- Per-call evaluation: a default `b = sideEffect()` runs the effect **each** call that omits `b`,
  and **not** when `b` is supplied (guards against once-at-definition mutable-default footgun).
- Default referencing an earlier param: `f(a, b = a + 1)`; `f(3)` → `b == 4`.
- Compile-time rejections (clean diagnostics, no panic): `f(a = 0, b)` (required after default);
  `f(a = 0, *rest)` (default + rest); a synthesized selector colliding with a hand-written method.
- Inliner parity: an inlined call that omits a default equals the non-inlined result.

## 7. Forward-looking — must NOT preclude
- **U9 variadics:** default args and rest params share the param-list parser + call-prologue —
  build U18's prologue in the same style as U9's rest-collection so the two compose (they are
  mutually exclusive per-list, but the machinery is shared). Do not fork a second prologue path.
- **External/internal param names (open-Q3, ADR-0012 reserved a field):** a defaulted parameter
  must still slot its future external label without changing selector identity — keep the default
  orthogonal to the label field.
- **U14 destructuring:** a default value could later be a destructuring target; keep the
  default-expr a plain `Expr` so this is additive.
- **U16 Family `::`:** a Family over a defaulted method should see all k+1 synthesized selectors in
  the base-name index (so `obj::move.candidates` lists them). Ensure synthesized selectors register
  into the class method dict *before* U16 builds `base_names` (finalization ordering).
- **Concurrency:** defaults evaluate in the callee frame (fiber-local); no shared default state —
  keeps the no-data-race invariant. Do not hoist default evaluation to a shared/global slot.

## 8. Mandatory rules
- `///` on `ParameterDef.default`, the synthesis helper, the prologue; `//!` refreshed; cite
  ADR-0012 + the new ADR. `cargo doc` clean.
- Green gate = `./scripts/verify.sh` exits 0. This touches dispatch-adjacent compiler code;
  recommend reviewer **ON**.
- Own isolated worktree off `main`.

## 9. Return contract
Report: the DEC-U18 ruling (A no-defaults vs B expansion) · if B, the trailing-only/left-to-right
policy + the collision rule (reject vs overwrite) + the default-expr evaluation frame · confirmation
one body backs N selectors and dispatch is unchanged · the U9-prologue sharing · files changed ·
`verify.sh` + `cargo doc` tails · DEFERRED entries (arbitrary-position defaults, keyword defaults).
