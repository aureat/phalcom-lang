# U-ITER — Work order: the cursor iteration protocol + `for` / `break` / `continue`

_Self-contained implementation plan for **one** implementer. Post-U5 (inliner) / post-U-LIST /
post-U6 (Option) surface+compiler unit. **Reviewer ON** (touches a spine file, `compiler/lib.rs`) —
hand the diff to `phalcom-reviewer`; do not self-approve. Green gate: `./scripts/verify.sh` exits 0 +
`cargo doc --workspace --no-deps` clean. Grounded in **[ADR-0035](../../../adr/0035-iteration-protocol-cursor.md)**
and normative **[iteration.md](../../../spec/v0.2/iteration.md) §1–§6**. New governing ADR: none —
ADR-0035 is ratified; this unit realises it._

> **Unit-name note.** ADR-0035 §3 says `for`/`break`/`continue` are "owned by U-LEX", but **U-LEX is
> closed** (as-built: block comments, digit separators, `\(expr)` only). This is a fresh named unit,
> `U-ITER`, verified free at plan time. It does **not** edit `docs/forge/PHASE2-INDEX.md` (shared file,
> concurrent editors).

---

## 1. Mission (one sentence)
Realise Phalcom's **one** iteration contract — the two-selector cursor protocol `iterate(_)` /
`iteratorValue(_)` ([iteration.md §1](../../../spec/v0.2/iteration.md)) on the reference iterable `List`,
the **`for (x in coll)`** surface that lowers to an inlined cursor `while` (§2), and **`break`/`continue`**
as jump-based loop control (§3) — with **zero new floor primitives** and the `for` lowering shaped so a
`Fiber`-backed generator suspends freely (§6, the load-bearing preclusion check).

## 2. Preconditions (verify on actual HEAD — do not assume)
- **U5 landed** — the sacred-selector inliner + `Jump` / `JumpIfFalse` / `Loop` opcodes
  ([ADR-0018](../../../adr/0018-sacred-selector-inliner-and-override-guard.md)). `for` reuses the
  **inlined `whileTrue` skeleton**; `break`/`continue` reuse the same jump opcodes. **Confirm `Jump`
  (forward) and `Loop` (backward) exist and are emittable outside the `whileTrue` path** — `graphify
  explain "Bytecode"` / read `bytecode.rs`. If a forward unconditional jump is missing, that is the only
  candidate new opcode (flag, don't invent silently).
- **U6 landed** — `Option` with `isSome` / `unwrap` (the cursor "more?" signal; §2 desugar uses both).
  `isSome` is inliner-eligible per iteration.md §4.
- **U-LIST landed** — `List` has native `size` / `at(_)` in `core.ph`; these are what `List#iterate` /
  `iteratorValue` are written *over*. **The `each(_)` already in `core.ph` stays as-is** (DEC-ITER-A).
- **U-FE landed** — hand-written lexer + recursive-descent parser; `for`/`break`/`continue` are **new
  keyword tokens** and `in` a **new contextual keyword** (none exist yet — confirm in `token.rs`).
- **Fiber unit is NOT a precondition** — iteration.md §6: "the cursor protocol needs **no** `Fiber`."
  This unit ships and verifies independently. The *generator-suspension* proof is a **PENDING** cross-unit
  test that graduates when the Fiber unit lands (§7, §9).
- Baseline `./scripts/verify.sh` green before the first edit. Re-run `graphify affected "core.ph"` and
  `graphify affected "compiler/lib.rs"` **and check concurrent `phalcom-ast`/`core.ph` editors** (§4.1).

## 3. Design (realise iteration.md — the semantics are fully specified; do not re-litigate)

### 3.1 The cursor protocol on `List` — pure `.ph`, zero primitives (iteration.md §1)
Reopen `class List` in `core.ph` and add the two selectors; the cursor **is an integer index** (no
iterator object allocated), and `Option` carries the "more?" signal so there is no surface `nil`
(Invariant 4):
```phalcom
iterate(cursor) {
  let next = cursor.map { c => c + 1 }.unwrapOr(0)   // None → start at 0; Some(i) → i+1
  return (next < self.size).ifTrue { Some(next) }.ifNone { None }
}
iteratorValue(cursor) => self.at(cursor)
```
Exact `Option`/`ifTrue`/`ifNone` spellings must match the landed U6/U-CORE-2 surface — verify on HEAD.
**No `size`/`at` change, no new primitive.**

### 3.2 `for (x in coll) { body }` → inlined cursor `while` (iteration.md §2) — **NOT** `.each`
Compile to the authoritative desugar (§2), evaluating `coll` **once** into a temp:
```phalcom
var _c = coll.iterate(None)
while (_c.isSome) { let x = coll.iteratorValue(_c.unwrap); body; _c = coll.iterate(_c) }
```
The `while` scaffold and `isSome` test inline to jumps (§4); `iterate`/`iteratorValue` are **ordinary
sends, never inlined** (§4 — they are type-specific and overridable). Lowering to `.each` is
**forbidden**: it would reintroduce a `block_call` (native frame) and break the §6 generator — this is
the crown-jewel constraint, see the rubric.

### 3.3 `break` / `continue` → dedicated jump loop (iteration.md §3)
A `for`/`while` that **contains** `break`/`continue` compiles to a **direct jump-based loop** —
condition, body, `break` → loop-exit label, `continue` → the **cursor-step** label (re-runs
`iterate(_)`) — **bypassing the overridable `whileTrue(_)` send** so the jump targets are always valid
(there is **no inliner-deopt path** to fall back to). Loops **without** `break`/`continue` keep the plain
§2 desugar. Semantics are identical; the direct lowering is purely how loop control stays sound. Requires
a **loop-context stack** in the compiler (each `for`/`while` pushes its exit + step labels; `break`/
`continue` resolve to the innermost) — a `break`/`continue` outside any loop is a **compile error**.

### 3.4 Native-vs-`.ph` split & the frozen floor
- Protocol (`iterate`/`iteratorValue`): **pure `.ph`** over `size`/`at`. **0 primitives.**
- `for` / `break` / `continue`: **parser + compiler lowering** to existing jump opcodes. **0 primitives,
  0 new bytecode** (unless §2 finds no forward-`Jump` — the one flagged candidate).
- **Net floor delta: 0.** No ADR-0019 amendment.

### Rubric — hazards & preclusion (mandatory)
- **`for` ⊗ the Fiber generator (THE load-bearing check, iteration.md §6 / ADR-0030 §4 / ADR-0033).**
  `for` must lower to an **inlined `while`** (§3.2/§3.3), emitting **no `block_call`**, so that
  `Fiber.new { for (x in coll) { Fiber.yield(x) } }` has only inlined control flow between the fiber floor
  and the yield → suspends freely. If `for` ever routed through `.each`/a block send, the generator would
  raise `CannotYieldAcrossNativeFrame`. Guard this with a **disasm golden** proving the `for` body emits
  jumps + `iterate`/`iteratorValue` sends and **contains no `call(_:)`/`block_call`**, plus the PENDING
  runtime suspension test (§7).
- **`iterate`/`iteratorValue` must stay non-inlined (§4).** A user type opts into `for` by implementing
  the two selectors; inlining them would freeze `List`'s impl into every call site. Pin the **Countdown**
  user-iterable test (iteration.md §1) to prove a non-`List` iterable drives `for`.
- **break/continue target validity (§3).** The dedicated jump loop has no deopt fallback; a mis-computed
  label is a miscompile, not a graceful send. Pin `continue`-re-runs-`iterate` and `break`-exits goldens,
  and nested-loop innermost-binding.
- **Representation/dispatch impact:** none. No `Value` tag, selector-encoding, or (barring the §2 flagged
  forward-jump) opcode change.
- **Precedent:** Wren's `iterate`/`iteratorValue` two-selector cursor (the direct model, ADR-0035);
  rejected alternatives (Python external `__iter__`/`__next__` object; Smalltalk `do:`-only) are in
  ADR-0035 §Alternatives — do not reopen.

## 4. Confirmed write-set (tight & disjoint; re-validate with `graphify affected` on HEAD)
| File | Why | Slice |
|---|---|---|
| `phalcom-ast/src/token.rs` | `for` / `break` / `continue` keyword tokens; `in` contextual (DEC-ITER-B) | surface |
| `phalcom-ast/src/lexer.rs` | emit the new keywords | surface |
| `phalcom-ast/src/ast.rs` | `For { binding, iter, body }`, `Break`, `Continue` nodes (full rustdoc) | surface |
| `phalcom-ast/src/parser.rs` | parse `for (x in e) { … }`, `break`, `continue` | surface |
| `phalcom-core/src/compiler/lib.rs` **(SPINE — reviewer ON)** | lower `for` → §2 cursor while; `break`/`continue` → §3 jump loop; **loop-context label stack** | compiler |
| `phalcom-core/core/core.ph` | `List#iterate(_)` / `iteratorValue(_)` reopen | protocol |
| `phalcom-core/tests/lang/iteration/` (**new label**) + `tests/lang/MANIFEST.md` | goldens + negatives + disasm + PENDING | all |

**Deliberately NOT in scope:** `vm.rs`, `bytecode.rs` (reuse `Jump`/`JumpIfFalse`/`Loop` — unless §2's
forward-jump gap forces exactly one addition), `primitive/*`, `universe.rs`, `value.rs`, `heap.rs`. Also
**not** the `each`/`map`/`filter`/`reduce` combinators (DEC-ITER-A → U-STD follow-on).

### 4.1 Write-set collision risk (flag, don't resolve)
- **`phalcom-ast/src/parser.rs`** is contended by the live U14/U15/U16/U-COLL cluster (all `phalcom-ast`
  editors). **Serialize** — U-ITER takes its own slot; cannot share a parallel wave.
- **`phalcom-core/core/core.ph` — never two editors.** The U-CORE track actively edits `core.ph`
  (`Map`/`Set`, live in git status). U-ITER's `List` reopen must be **serialized against every U-CORE
  `core.ph` unit**. The compiler + `phalcom-ast` slices are free of this and can land first.
- **`compiler/lib.rs`** — spine file; check no concurrent unit holds it before dispatch.

## 5. Build order (small, independently-green diffs)
1. **Surface** — tokens + AST + parser for `for`/`break`/`continue`; parse-only/AST goldens. Green.
   *(collides only in `phalcom-ast`.)*
2. **Protocol** — `List#iterate`/`iteratorValue` in `core.ph`; a hand-written `while` golden driving them
   proves the contract before `for` exists. Green. *(serialize vs U-CORE `core.ph`.)*
3. **`for` (no break/continue)** — §2 inlined cursor-while lowering + the **disasm golden** (no
   `block_call`) + `List` and **Countdown** runtime goldens. Green.
4. **`break`/`continue`** — §3 dedicated jump loop + loop-context stack + goldens (break exits, continue
   re-runs `iterate`, nested innermost, out-of-loop = compile error). Green.
5. **PENDING generator** — `tests/lang/iteration/pending/for_generator_suspends.ph` (`Fiber.new { for (x
   in [1,2,3]) { Fiber.yield(x) } }`), `.expected` pinned to the intended yields; `#[ignore]` until the
   Fiber unit lands. Also pin the **negative** `each_generator_raises` once Fiber lands (documents §6).

Each step is a self-verifiable commit.

## 6. Mandatory rules
- **Docs:** `///` on every new AST node/field and parser/compiler fn, citing ADR-0035 / iteration.md §.
  `cargo doc --workspace --no-deps` adds no warnings.
- **Green gate:** `./scripts/verify.sh` exits 0; no new clippy; no `unsafe`. Follow `rust-best-practices`.
- **Reviewer ON** (spine file) — `phalcom-reviewer` gates the diff; the writer never self-approves.

## 7. Test strategy (the green gate must assert) — new `iteration` label
- **Protocol (PASS):** direct `xs.iterate(None)` → `Some(0)`; `xs.iterate(Some(0))` → `Some(1)`; past-end
  → `None`; `xs.iteratorValue(0)` round-trips `at(0)`.
- **`for` over `List` (PASS):** `for (x in [10,20,30]) { … }` visits 10,20,30 in order; **empty** `[]`
  runs the body zero times; `coll` is evaluated **once** (side-effecting-receiver golden).
- **`for` disasm (PASS — the §6 preclusion guard):** the `for` chunk contains `Jump`/`Loop` +
  `iterate`/`iteratorValue`/`isSome` sends and **no `call(_:)`/`block_call`**.
- **User iterable / Countdown (PASS — §4 non-inlined proof):** the iteration.md §1 `Countdown` type drives
  `for` purely through its two `.ph` selectors.
- **break/continue (PASS):** `continue` skips to the next `iterate`; `break` exits; nested loops bind to
  the **innermost**; `for`+`break` and `while`+`break` both.
- **NEGATIVE:** `break`/`continue` outside any loop → compile error with a clear span.
- **PENDING (`iteration/pending/`):** `for_generator_suspends` (graduates with the Fiber unit); once Fiber
  lands, add `each_generator_raises` asserting `.each { Fiber.yield }` → `CannotYieldAcrossNativeFrame`.

## 8. Decisions flagged (flag, don't pick)
| ID | Decision | Options | Architect recommendation |
|---|---|---|---|
| **DEC-ITER-A** | **Migrate the combinators now?** `each`/`map`/`filter`/`reduce` currently sit on `size`/`at`; ADR-0035 §5 wants them as `.ph` defaults over `iterate`/`iteratorValue`. | **(A)** leave them as-is; U-ITER ships protocol + `for` only; a **U-STD follow-on** rewrites the combinators onto the protocol. **(B)** rewrite them here. | **(A)** — (B) enlarges the `core.ph` write-set against the concurrent U-CORE editors for no `for`-path gain. Both mechanisms are correct in parallel meanwhile; one-contract convergence is a clean follow-on. |
| **DEC-ITER-B** | **`in` keyword form.** | **(A)** contextual (special only inside `for (… in …)`); **(B)** reserved word. | **(A)** — avoids breaking any existing identifier `in`; the `for` arm is the only site that needs it. |
| **DEC-ITER-C** | **Forward-jump opcode.** If `bytecode.rs` has no unconditional forward `Jump` (only `JumpIfFalse`/`Loop`), `break` needs one. | **(A)** add a single `Jump` opcode; **(B)** synthesise via `JumpIfFalse` on a constant `true`. | **(A)** if genuinely absent — one honest opcode beats a synthetic-condition hack. Verify on HEAD first (§2); likely already present from U5. |

## 9. Must-not-preclude check
- **Fiber generators (ADR-0030 / ADR-0033):** actively *served*, not precluded — the inlined-`while`
  lowering (§3.2) is exactly what lets `for (x in coll) { Fiber.yield(x) }` suspend. The deferred ADR-0033
  `CallBlock` lift (for `.each { yield }` etc.) is **orthogonal** and untouched: U-ITER adds no block-call
  path and no fiber machinery, so it neither blocks nor pre-empts that future slice.
- **`Map`/`Set`/`Tuple`/`Range` iterability:** not precluded — each conforms later by implementing the two
  selectors; `for` and every combinator fall out with no further compiler work.
- **`Stream`/lazy layer (ADR-0035 §Alternatives):** not precluded — builds on `Fiber`, not on this
  protocol; U-ITER touches neither.
- **A `for`-`else` / labelled-break future:** not precluded — the loop-context stack (§3.3) is the natural
  extension point; U-ITER ships the unlabelled forms only.

## 10. Return contract (report to `phalcom-reviewer`)
The tokens/AST/parser arms added · the `List#iterate`/`iteratorValue` `.ph` shape (verified `Option`
spelling on HEAD) · the `for` §2 desugar + confirmation `coll` is evaluated once · the **disasm golden
proving no `block_call` in a `for` body** (the §6 guard) · the §3 break/continue jump-loop + loop-context
stack + out-of-loop compile error · the Countdown user-iterable pass · how DEC-ITER-A/B/C resolved (and if
C, the exact opcode delta) · confirmation **net floor delta = 0** · the `iteration` corpus label + MANIFEST
bump + the PENDING `for_generator_suspends` fixture that graduates with the Fiber unit · `verify.sh` +
`cargo doc` tails · any new `DEFERRED.md` entries (combinator migration, `each_generator_raises` pending).
