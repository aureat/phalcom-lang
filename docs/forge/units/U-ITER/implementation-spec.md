# U-ITER — Implementation Spec: cursor protocol + `for` / `break` / `continue`

> **Status:** Normative work order for a `phalcom-implementer`. Realizes
> [ADR-0035](../../../adr/0035-iteration-protocol-cursor.md) and the deepened
> [specification.md](specification.md) (which extends the normative
> [`iteration.md`](../../../spec/v0.2/iteration.md) §1–§7). Where a fact was verified
> against source it carries a `file:line`; **re-confirm line numbers at dispatch** —
> concurrent forge sessions shift them.
>
> **Baseline: grounded at HEAD `9d3b7e1`** (U-CORE track closed, floor **88**; graph
> rebuilt 16:54). **Reviewer ON** (touches the spine file `compiler/lib.rs`) — hand the
> diff to `phalcom-reviewer`; do **not** self-approve. Green gate: `./scripts/verify.sh`
> exits 0 **and** `cargo doc --workspace --no-deps` clean.
>
> **Scope in one line:** realize Phalcom's *one* iteration contract — the two-selector
> cursor protocol on the reference iterable `List`, the `for (x in coll)` surface that
> lowers to an inlined cursor `while`, and `break`/`continue` as jump-based loop control
> — with **zero new floor primitives** and the `for` lowering shaped so a `Fiber`-backed
> generator suspends freely (the load-bearing preclusion check, §5).

---

## §0. Prerequisites + scope gate

### Already landed (do not rebuild) — verified at HEAD

| Dep | What it gives U-ITER | Ground truth (re-confirm at dispatch) |
|---|---|---|
| **U5 inliner + jump opcodes** | `Bytecode::Jump(i32)` (**unconditional forward/relative jump — already present**), `JumpIfFalse(i32)`, `Loop(i32)`, plus the inliner's `emit_jump`/`patch_jump`/`emit_loop` helpers the loop lowering reuses. | `bytecode.rs:130` (`Jump`), `:139` (`JumpIfFalse`), `:145` (`Loop`); `compiler/inliner.rs:167` (`emit_jump`), `:182` (`patch_jump`), `:194` (`emit_loop`), `:157` (`compile_while_true`) |
| **U6 `Option`** | `Some`/`None` with `isSome`/`unwrap`/`map`/`unwrapOr`/`ifNone` — the "more?" signal in the desugar (§3.1 spec). `isSome` inliner-eligible. | landed U-CORE-2/U6 surface; verify exact selector spelling in `core.ph` at dispatch |
| **U-LIST `List`** | `List#size`/`at(_)` (over `rawLength`/`rawAt`) — what `List#iterate`/`iteratorValue` are written *over*. `each`/`map`/`filter`/`reduce` stay as-is (**DEC-ITER-A**). | `core.ph` `class List` (`core.ph:163`), `size => self.rawLength` (`:164`), `at(i)` (`:166`), `each(f)` (`:175`) |
| **U-FE lexer + parser** | Recursive-descent parser; `while` already parses (desugars to `whileTrue`). | `parser.rs:1338` (`parse_while`), `:1376` (dispatch) |

### Correction to the plan's preconditions (verified at HEAD)

- **The keyword tokens already exist.** `token.rs` defines `While`/`For`/`Break`/`Continue`/`In`
  and `lexer.rs:256-263` already emits them for `"while"/"for"/"break"/"continue"/"in"`.
  The plan (§2) assumed "none exist yet — confirm in token.rs"; **they do exist and
  lex.** U-ITER therefore adds **no token/lexer work** — only the AST nodes, parser
  productions, and compiler lowering that *consume* `For`/`Break`/`Continue` (and the
  contextual `In`). Re-confirm this at dispatch before touching `token.rs`/`lexer.rs`.
- **DEC-ITER-C resolves to "no new opcode."** `bytecode.rs:130` already carries an
  unconditional relative `Jump(i32)`; `break`/`continue` reuse it. No `bytecode.rs`
  edit is required (verify at dispatch).

### Explicitly OUT of scope (RESERVE, do not implement)

- **Combinator migration** onto `iterate`/`iteratorValue` — **DEC-ITER-A: U-STD
  follow-on** (ADR-0035 §5). U-ITER's `core.ph` edit is limited to adding
  `List#iterate`/`iteratorValue`; `each`/`map`/`filter`/`reduce`/`includes` stay on
  `size`/`at`. Add a `docs/forge/DEFERRED.md` pointer.
- **`Map`/`Set`/`Tuple`/`Range` iterability** — later; each conforms by implementing the
  two selectors (spec §7.3).
- **Comprehensions, `for`-`else`, labelled `break`, `Stream`/lazy layer** — not v0.2
  (spec §10).

### Non-negotiables carried in

- **Zero new floor** (ADR-0035 §Consequences). No `primitive!`, no `bytecode.rs` addition
  (barring a DEC-ITER-C surprise that does not exist at this HEAD). **No ADR-0019
  amendment, no census bump** — the floor stays **88**.
- **`for` never lowers to `.each`** (ADR-0035 §2) — the crown-jewel constraint (§5).
- **`iterate`/`iteratorValue` never inlined** (ADR-0035 §4).
- No `unsafe`; no new clippy; full rustdoc on every new item.

---

## §1. What exists vs what is missing (grounded)

### Exists (verified at HEAD)

1. **`while` surface** — `parse_while` (`parser.rs:1338`) parses `while (c) { b }` and
   **desugars it at parse time** to `Expr::MethodCall { object: {c}, method: "whileTrue",
   args: [{b}] }` (`parser.rs:1348-1357`). There is **no `While` AST node** — `while` is
   pure desugar to a `whileTrue` send.
2. **The inliner** recognizes `whileTrue` (`inliner.rs:123` → `SacredCall::WhileTrue`) and
   lowers it via `compile_while_true` (`inliner.rs:157`) using `emit_jump`/`patch_jump`/
   `emit_loop`. The `GuardBlock` deopt guard rides along (`bytecode.rs:172`).
3. **Jump opcodes** — `Jump`/`JumpIfFalse`/`Loop` (`bytecode.rs:130/139/145`), disassembled
   by the derived `Debug` (`bin/phalcom/disasm.rs:18` prints `{:?}` — **no per-opcode
   match arm**).
4. **`List`** — `size`/`at(_)`/`each(f)` in `core.ph` (`:164/:166/:175`). `each` is
   `while (i < self.size) { f.call(self.at(i)); … }` — an inlined `while` whose
   `f.call(x)` is a **`block_call`** (the native frame that makes `.each { yield }`
   yield-opaque; spec §7.1).

### Missing (this unit adds)

| Missing | Add in | Note |
|---|---|---|
| `For { binding, iter, body }`, `Break`, `Continue` AST nodes | `phalcom-ast/src/ast.rs` | full rustdoc citing ADR-0035 / iteration.md § |
| `parse_for` + `break`/`continue` statement parsing; contextual `in` consumption | `phalcom-ast/src/parser.rs` | reuse `parse_brace_block`, `wrap_expr_as_block` |
| `for` → §3.1 cursor-`while` lowering; `break`/`continue` → §3.2 jump loop; the **loop-context label stack** | `phalcom-core/src/compiler/lib.rs` **(SPINE)** | reuse inliner `emit_jump`/`patch_jump`/`emit_loop` |
| `List#iterate(_)` / `iteratorValue(_)` | `phalcom-core/core/core.ph` | pure `.ph` over `size`/`at`; **0 primitives** |
| `iteration` corpus label + `MANIFEST.md` bump | `phalcom-core/tests/lang/iteration/` | goldens + negatives + disasm + PENDING |

**Deliberately NOT touched:** `vm.rs`, `bytecode.rs`, `primitive/*`, `universe.rs`,
`value.rs`, `heap.rs` — no runtime, opcode, or floor change.

---

## §2. The native/`.ph` split + exact insertion points

**Decision: no native code at all.** The protocol is `.ph` over the existing floor; the
surface is parser + compiler lowering onto existing opcodes. **Net floor delta: 0**
(census stays 88, no ADR-0019 amendment) — this is the ADR-0035 §Consequences promise.

| Concern | Native (Rust) | `.ph` / surface |
|---|---|---|
| `iterate(_)` / `iteratorValue(_)` on `List` | — | ✅ `core.ph` reopen of `class List` |
| `for` / `break` / `continue` grammar | ✅ AST nodes + `parser.rs` productions | — |
| `for` lowering, jump loop, loop-context stack | ✅ `compiler/lib.rs` | — |
| jump opcodes | reuse `Jump`/`JumpIfFalse`/`Loop` (no new) | — |

### Insertion points (exact — re-confirm at dispatch)

1. **`phalcom-ast/src/ast.rs`** — add three nodes to the statement/expression enum
   (match the existing `Expr`/statement shape; `while` currently produces
   `Expr::MethodCall`, so decide whether `For` is an `Expr` variant used as a statement
   or a `Statement` variant — see D-ITER-1). Each with `///` rustdoc citing ADR-0035 §2/§3.
   ```rust
   /// `for (binding in iter) { body }` — the cursor loop (ADR-0035 §2,
   /// iteration.md §2). Lowered to an inlined cursor `while` (never `.each`).
   For { binding: String, iter: Box<Expr>, body: BlockExpr, range: SourceRange },
   /// `break` — leave the innermost enclosing loop (ADR-0035 §3).
   Break { range: SourceRange },
   /// `continue` — jump to the innermost loop's cursor-step (ADR-0035 §3).
   Continue { range: SourceRange },
   ```

2. **`phalcom-ast/src/parser.rs`** — add `parse_for` next to `parse_while`
   (`parser.rs:1338`); wire `Token::For`/`Token::Break`/`Token::Continue` into the
   statement/primary dispatch (the `while` arm is at `:1376`). `parse_for`:
   - `expect(LParen)`; parse `binding` as `IDENT`; `expect(Token::In)` (contextual —
     only consumed here); parse the iterable `expr`; `expect(RParen)`;
     `parse_brace_block()` for the body.
   - `break`/`continue` parse as bare statements (advance the keyword; produce the node).

3. **`phalcom-core/src/compiler/lib.rs` (SPINE)** — the lowering (§3 below) + a
   **loop-context stack** field on the compiler (`Vec<LoopCtx>` where
   `LoopCtx { exit_jumps: Vec<usize>, step_label: usize }`). Reuse the inliner helpers
   (`inliner.rs:167/182/194`). Push a `LoopCtx` on entering a `for`/`while` body that can
   contain `break`/`continue`; pop on exit.

4. **`phalcom-core/core/core.ph`** — reopen `class List` (`core.ph:163`) and add the two
   selectors (§3.3). **Serialize against every U-CORE `core.ph` editor** — `core.ph` is
   never edited by two units at once (plan §4.1). The `phalcom-ast` + `compiler` slices
   are free of this collision and can land first.

5. **`phalcom-core/tests/lang/iteration/`** (new label) + `tests/lang/MANIFEST.md` bump
   (§4).

---

## §3. Concrete bodies / pseudocode

### 3.1 `for` without `break`/`continue` (ADR-0035 §2, spec §3.1)

Lower `For { binding, iter, body }` to the cursor `while`, evaluating `iter` **once**:

```
compile_for(binding, iter, body):
    # 1. Evaluate the iterable once into a temp local `_coll`.
    slot_coll = declare_synthetic_local("_coll")
    compile_expr(iter); emit(SetLocal(slot_coll)); emit(Pop)

    # 2. _c = _coll.iterate(None)
    slot_c = declare_synthetic_local("_c")
    emit(GetLocal(slot_coll)); emit_none(); emit(Invoke(1, sel"iterate(_)"))
    emit(SetLocal(slot_c)); emit(Pop)

    # 3. while (_c.isSome) { let binding = _coll.iteratorValue(_c.unwrap); body; _c = _coll.iterate(_c) }
    #    Emit the while skeleton directly as jumps (the whileTrue skeleton), NOT a
    #    `whileTrue(_)` MethodCall — see D-ITER-2. isSome may inline.
    loop_start = here()
    emit(GetLocal(slot_c)); emit(Invoke(0, sel"isSome"))
    exit = emit_jump(JumpIfFalse)
        # bind loop var
        emit(GetLocal(slot_coll)); emit(GetLocal(slot_c)); emit(Invoke(0, sel"unwrap"))
        emit(Invoke(1, sel"iteratorValue(_)"))
        bind_local(binding)                       # let binding = <top>
        compile_block_body(body)                  # may contain sends, NOT block_call
        # step: _c = _coll.iterate(_c)
        emit(GetLocal(slot_coll)); emit(GetLocal(slot_c)); emit(Invoke(1, sel"iterate(_)"))
        emit(SetLocal(slot_c)); emit(Pop)
    emit_loop(loop_start)
    patch_jump(exit)
```

The taken path is pure `Invoke` (protocol sends) + jumps — **no `block_call`** (guards
C-ITER-4 / §5).

### 3.2 `for` / `while` containing `break`/`continue` (ADR-0035 §3, spec §3.2)

Same skeleton, but **push a `LoopCtx` before the body** so `break`/`continue` resolve:

```
compile_loop_with_control(...):
    loop_start = here()
    <cond>; exit0 = emit_jump(JumpIfFalse)          # exit0 joins ctx.exit_jumps
    push LoopCtx { exit_jumps: [exit0], step_label: <patched after body> }
    <bind + body>                                    # break→emit ctx.exit_jumps.push(emit_jump(Jump))
                                                     # continue→emit Jump to step_label (backpatch)
    ctx = pop()
    step_label := here()                             # continue lands here
    <cursor step: _c = _coll.iterate(_c)>            # for only
    emit_loop(loop_start)
    for j in ctx.exit_jumps: patch_jump(j)           # cond-false AND every break land past the loop
```

- **`continue`** — `Jump` to `ctx.step_label` (the cursor-advance for `for`, the
  condition re-test for `while`). Because `step_label` is only known after the body, emit
  a placeholder `Jump` and record it for backpatching, or reserve the label slot.
- **`break`** — `Jump` recorded in `ctx.exit_jumps`, patched to `end:` alongside the
  cond-false exit.
- **Empty loop-context stack on `break`/`continue`** → `CompilerError` with the keyword's
  span (spec §6 / §9 C-ITER-7).

> The plain form (§3.1) may reuse this same routine with an empty (never-consulted)
> `LoopCtx`; whether to keep one lowering path or two is **D-ITER-3**. One path (always
> the dedicated jump loop) is simpler and makes the "no `block_call`, no deopt fallback"
> guarantee **unconditional** — see D-ITER-2.

### 3.3 `List#iterate` / `iteratorValue` — pure `.ph` (spec §2.2)

Reopen `class List` in `core.ph` (`:163`) and add — **verify the exact `Option`/`ifTrue`/
`ifNone`/`unwrapOr` spellings against the landed U6/U-CORE-2 surface on HEAD**:

```phalcom
// Cursor iteration (ADR-0035 §1, iteration.md §1). Cursor is the integer index;
// Option carries the "more?" signal — no surface nil (Invariant 4). Zero primitives.
iterate(cursor) {
  let next = cursor.map { c => c + 1 }.unwrapOr(0)   // None → 0; Some(i) → i+1
  return (next < self.size).ifTrue { Some(next) }.ifNone { None }
}
iteratorValue(cursor) => self.at(cursor)
```

**No `size`/`at` change; no new primitive.** A hand-written `while` golden driving these
directly (before `for` exists) proves the contract (§4, build step 2).

---

## §4. Test strategy — new `iteration` corpus label

Add `phalcom-core/tests/lang/iteration/` and bump `tests/lang/MANIFEST.md`.

### Goldens (PASS)

1. **Protocol round-trip** (C-ITER-9) — `[7,8].iterate(None)` → `Some(0)`;
   `.iterate(Some(0))` → `Some(1)`; past-end → `None`; `.iteratorValue(0)` round-trips
   `at(0)`. *(Hand-written `while`, provable before `for` lands.)*
2. **`for` over `List`** (C-ITER-1/2/3) — `for (x in [10,20,30])` prints `10 20 30`;
   `for (x in [])` prints nothing; a side-effecting receiver (e.g. a counter-incrementing
   getter) is evaluated **once**.
3. **`for` disasm** (C-ITER-4, the §5 guard) — the `for` chunk's taken path is
   `Jump`/`Loop` + `iterate`/`iteratorValue`/`isSome` `Invoke`s and **contains no
   `Invoke(_, call(_))`/`block_call`**. Bless from the built binary
   (`disasm_source`); the disasm prints opcodes via `{:?}`.
4. **User iterable / `Countdown`** (C-ITER-5) — the spec §2.3 `Countdown` drives `for`
   through its two `.ph` selectors → `3 2 1 0`.
5. **`break`/`continue`** (C-ITER-6) — `continue` skips to the next `iterate`; `break`
   exits; nested loops bind to the **innermost**; both `for`+`break` and `while`+`break`.

### Negatives (compile error)

6. **`break`/`continue` outside any loop** (C-ITER-7) — a `.ph` with a top-level `break`
   → compile error with a span at the keyword. `// status: NEGATIVE`.

### PENDING / cross-unit

7. **`iteration/pending/for_generator_suspends`** (C-ITER-8) —
   `Fiber.new { for (x in [1,2,3]) { Fiber.yield(x) } }`, `.expected` pinned to `1 2 3`
   over three `call()`s; `#[ignore]`/`pending/` until [[U-FIBER]](../U-FIBER/implementation-spec.md)
   lands. This is the runtime half of the §5 preclusion proof.
8. **`iteration/pending/each_generator_raises`** *(add once U-FIBER lands)* —
   `Fiber.new { [1,2].each { x => Fiber.yield(x) } }` raises `CannotYieldAcrossNativeFrame`
   — documents §7.1 / ADR-0030 §4. Add a `DEFERRED.md` pointer.

### Census

9. **Floor delta = 0** (C-ITER-10) — assert the installed-binding count is unchanged
   (**88**) and no new primitive fn appears. No `floor-census.md` edit.

---

## §5. Must-not-preclude

*Grounded in [`iteration.md`](../../../spec/v0.2/iteration.md) §6,
[ADR-0035](../../../adr/0035-iteration-protocol-cursor.md) §5, and the
[[U-FIBER]](../U-FIBER/specification.md) seam.*

| Hazard | How this design clears it |
|---|---|
| **`for` ⊗ the `Fiber` generator (CROWN JEWEL).** If `for` interposed a `block_call`, `Fiber.new { for (x in c) { Fiber.yield(x) } }` would raise `CannotYieldAcrossNativeFrame`. | `for` lowers to an **inlined `while`** (§3), emitting **no `block_call`** on the taken path — so a `Fiber.yield` in the body sits under only jumps between the fiber floor and the yield → suspends freely ([[U-FIBER §4.3]](../U-FIBER/specification.md#yield-guard)). Guarded by the disasm golden (C-ITER-4) + the PENDING runtime test (C-ITER-8). |
| **`iterate`/`iteratorValue` frozen into call-sites.** | They stay **non-inlined ordinary sends** (§3, spec §5) — a user type opts into `for` by defining them; `Countdown` (C-ITER-5) proves a non-`List` iterable drives `for`. |
| **`break`/`continue` target validity.** The dedicated jump loop has no deopt fallback. | Emitting the loop directly (§3.2) means **no inliner-deopt path** exists to invalidate jump targets; the loop-context stack resolves labels lexically. |
| **ADR-0033 (`CallBlock` trampoline) precluded?** | **No — orthogonal.** U-ITER adds no block-call path and no fiber machinery, so it neither blocks nor pre-empts the Deferred ADR-0033 lift for `.each { yield }` (spec §7.1). |
| **`Map`/`Set`/`Tuple`/`Range` / `Stream` / `for`-`else` / labelled break.** | Not precluded (spec §7.3, §10) — collections conform by adding two selectors; the loop-context stack is the extension point for labelled break / `for`-`else`. |
| **Representation / dispatch / floor.** | No `Value` tag, no selector-encoding change, no opcode (barring the non-existent DEC-ITER-C surprise), **floor stays 88**. |

---

## §6. Open sub-decisions, build order, traceability

### Sub-decisions (recommend; flag if deviating)

- **D-ITER-1 — `For` as `Expr` or `Statement`.** *Recommended:* mirror the existing
  `while`, which is an `Expr` used in statement position (`parser.rs:1348` produces
  `Expr::MethodCall`). Keeping `for` an `Expr` (value unspecified/`None`) is the smallest
  fit and leaves a comprehension-value future open. *Alternative:* a dedicated
  `Statement::For`. Either works; match whatever `if`/`while` do on HEAD.
- **D-ITER-2 — realize the `while` skeleton directly, not via a `whileTrue` MethodCall.**
  *Recommended and load-bearing:* emit the cursor loop's condition/back-edge **directly**
  as `Jump`/`JumpIfFalse`/`Loop` (reusing `emit_jump`/`emit_loop`), **not** by
  synthesizing a `{cond}.whileTrue{body}` `Expr::MethodCall`. Rationale: an inlined
  `whileTrue` carries a **`GuardBlock` deopt fallback** that emits a *real* `whileTrue(_)`
  send (→ `block_call`) into the chunk. Even though that fallback is untaken while
  `Block>>whileTrue` is pristine, emitting `for` directly makes the **"no `block_call` in
  the chunk" guarantee unconditional** and keeps C-ITER-4 clean and honest.
  > **⚠ verify-on-HEAD / spec reconciliation.** [`iteration.md`](../../../spec/v0.2/iteration.md)
  > §3 currently phrases the *plain* case as "keep the plain `whileTrue`/cursor desugar."
  > If the implementer instead routes the plain `for` through an actual `whileTrue`
  > `MethodCall`, the emitted chunk **will contain a `block_call` in the deopt fallback
  > arm**, and C-ITER-4 must then assert on the *taken fast path only* (and the fiber
  > generator would raise `CannotYieldAcrossNativeFrame` if a user overrode
  > `Block>>whileTrue`). **Recommend the direct-jump lowering (both cases) to avoid this
  > entirely; if deviating, update C-ITER-4's wording and note the deopt caveat.** This is
  > the one genuine subtlety in the unit — do not paper over it.
- **D-ITER-3 — one lowering path or two.** *Recommended:* a **single** `compile_for`
  that always emits the dedicated jump loop (§3.2) with a loop-context frame, used for
  both the plain and `break`/`continue` cases. Simpler, and makes D-ITER-2's guarantee
  uniform. *Alternative:* a fast path for the no-control case. Prefer one path.
- **DEC-ITER-B (settled) — `in` contextual, not reserved** (ADR-0035; plan §8). Consume
  `Token::In` only inside `parse_for`.
- **DEC-ITER-C (settled at HEAD) — no new opcode.** `Bytecode::Jump(i32)` already exists
  (`bytecode.rs:130`); `break`/`continue` reuse it. Confirm no forward-jump gap at
  dispatch (there is none at this HEAD).
- **DEC-ITER-A (settled, user 2026-07-12) — combinator migration is a U-STD follow-on**,
  not this unit. `core.ph` edit limited to `List#iterate`/`iteratorValue`; add a
  `DEFERRED.md` pointer.

### Build order (small, independently-green diffs — from plan §5)

1. **Surface** — `For`/`Break`/`Continue` AST nodes + `parse_for` + `break`/`continue`
   parsing; parse-only/AST goldens. Green. *(Collides only in `phalcom-ast`; serialize
   against the U14/U15/U16/U-COLL parser cluster — take a dedicated slot.)*
2. **Protocol** — `List#iterate`/`iteratorValue` in `core.ph`; a hand-written `while`
   golden driving them (C-ITER-9) proves the contract before `for` exists. Green.
   *(Serialize against every U-CORE `core.ph` editor.)*
3. **`for` (no break/continue)** — §3.1 lowering + the **disasm golden** (C-ITER-4,
   no `block_call`) + `List` and `Countdown` runtime goldens (C-ITER-1/2/3/5). Green.
4. **`break`/`continue`** — §3.2 jump loop + loop-context stack + goldens (break exits,
   continue re-runs `iterate`, nested innermost, out-of-loop compile error —
   C-ITER-6/7). Green.
5. **PENDING generator** — `pending/for_generator_suspends` (C-ITER-8), `#[ignore]` until
   [[U-FIBER]](../U-FIBER/implementation-spec.md) lands; `DEFERRED.md` entries
   (combinator migration; `each_generator_raises` pending).

Each step is a self-verifiable commit; reviewer gates the `compiler/lib.rs` diff.

### Write-set collision risk (flag, don't resolve — plan §4.1)

- **`parser.rs`** contended by the U14/U15/U16/U-COLL cluster — **serialize**, own slot.
- **`core.ph`** — never two editors; serialize the `List` reopen against every U-CORE
  `core.ph` unit. The `phalcom-ast` + `compiler` slices are collision-free and land first.
- **`compiler/lib.rs`** — spine file; confirm no concurrent holder before dispatch.

### Traceability

| Claim / requirement | Source |
|---|---|
| Two-selector cursor protocol; `for` → cursor `while` not `.each`; `iterate`/`iteratorValue` non-inlined; break/continue jumps; U-STD combinator follow-on | [ADR-0035](../../../adr/0035-iteration-protocol-cursor.md) §1–§5; [iteration.md](../../../spec/v0.2/iteration.md) §1–§6; [specification.md](specification.md) |
| `for` ⊗ fiber generator (the load-bearing preclusion) | [iteration.md](../../../spec/v0.2/iteration.md) §6; [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §4; [ADR-0033](../../../adr/0033-amend-fiber-execution-trampolined-block-callsite.md) Context; [[U-FIBER]](../U-FIBER/specification.md#the-crown-jewel) |
| Jump opcodes / inliner helpers exist | `bytecode.rs:130/139/145`; `compiler/inliner.rs:157/167/182/194` |
| `while` desugars to `whileTrue` at parse time | `parser.rs:1338-1357` |
| tokens already lex (plan-precondition correction) | `token.rs` `For`/`Break`/`Continue`/`In`; `lexer.rs:256-263` |
| `List` reference iterable (`size`/`at`/`each`) | `core.ph:163/164/166/175`; [ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md) |
| disasm renders via `Debug` (no arm needed) | `bin/phalcom/disasm.rs:18` |
| DEC-ITER-A/B/C | plan §8; [ADR-0035](../../../adr/0035-iteration-protocol-cursor.md); this HEAD |
