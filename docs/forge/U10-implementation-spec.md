# U10 — Implementation Specification (supersedes U10-plan.md on conflict)

_Grounded against actual HEAD as of commit `c9805d0` (U9 landed, green). This
document exists because `U10-plan.md` describes the frame-token mechanism at
too shallow a level to implement safely: it treats `self.frames` as a single
flat stack owned by one dispatch loop, which is **not** how this codebase
actually runs blocks. Read §2 before writing any VM code — it is the one part
of this unit that will silently corrupt the stack if skipped. **Where this
document and `U10-plan.md` disagree, follow this document.** Where this
document is silent, `U10-plan.md` still governs (mission, guardrails,
mandatory rules, return-contract shape)._

Written for a **medium-effort implementer**. If you hit a fact that
contradicts this doc, STOP and report the conflict rather than guessing —
this doc's job is to have already done the archaeology so you don't have to.

---

## 0. Corrections to `U10-plan.md`

1. **No miette anywhere in this codebase.** The plan's write-set entry for
   `error.rs` says `DeadFrameError` should be "a miette diagnostic, spans via
   `phalcom-common` ranges." This does not match reality: `RuntimeError`
   (`phalcom-core/src/error.rs:63`) is plain `thiserror`, no `#[derive(Diagnostic)]`
   anywhere in `phalcom-core` or `phalcom-ast`, and **no `RuntimeError` variant
   carries a span** — `Arity`, `Type`, `MessageNotUnderstood`, etc. are all
   plain `{field}`-interpolated messages with no `SourceRange`. `miette` is
   pinned in the workspace root `Cargo.toml` but unused. An uncaught
   `RuntimeError` is rendered by its `Display` impl and exits with code 70
   (`interpret.rs::ExitCode::RuntimeError`) — no source-mapped rendering at
   all today. **Add `DeadFrameError` as an ordinary `RuntimeError` variant,
   same shape as its neighbors** (e.g. `#[error("return from a block whose
   home frame no longer exists")] DeadFrameError`) — no span field, no miette.
   If you want to carry diagnostic detail (home-frame generation vs current),
   put it in the message string like `MessageNotUnderstood` does, not in a new
   spanned-diagnostic type.

2. **`disasm.rs` needs zero changes.** The plan's write-set lists
   `phalcom-core/bin/phalcom/disasm.rs` for "Disassemble `ReturnNonLocal`."
   Read it (`disasm.rs:16-19`): it is `for instr in chunk.code { println!("{:04}: {:?}", i, instr) }`
   — pure `{:?}` (`Debug`) formatting, with **no per-opcode match arm at all**.
   `Bytecode` already `#[derive(Debug)]`s (`bytecode.rs:2`), so adding
   `ReturnNonLocal` to the enum disassembles automatically. Do not touch this
   file (mirrors U9's correction about `signature.rs` — a plan referencing a
   file that turns out to need no edit).

3. **`frame.rs` DOES need a real change, not just "read-only if the compare
   helper suffices."** The plan hedges on this. It does not suffice — see §2.
   `CallFrame` needs a new field: the frame's own `home_frame_token: Option<FrameToken>`,
   populated **only** when the frame is a block invocation (set by
   `block_call`, `phalcom-core/src/primitive/block.rs:77-99`), `None` for
   every ordinary method/closure call (`call_method`'s `MethodKind::Closure`
   arm, `vm.rs:396`). Without this, the executing frame has no way to answer
   "what token am I running under" when `ReturnNonLocal` fires — the
   `BlockObject` that carries the token is not reachable from a live
   `CallFrame` any other way (a `CallFrame` only stores the `ClosureObject`
   handle, not the `BlockObject` that wrapped it).

4. **The two `pending/` fixtures the plan implicitly expects to promote are
   not runnable as written — rewrite, don't just un-ignore.**
   - `phalcom-core/tests/lang/blocks/pending/blocks_non_local_return.ph` uses
     `[3, -5, 8]` list-literal syntax. **List literals do not parse** —
     confirmed no `[`-prefixed list-literal grammar exists anywhere in
     `phalcom-ast/src/parser.rs`; collection literals are deferred to U-LEX/U-STD
     (`DEFERRED.md` #6/#20). Rewrite using `List.new()` + `.add(_)`, the same
     pattern U9's own fixtures use (see `phalcom-core/tests/lang/variadics/variadics_zero_prefix.ph`
     for the idiom).
   - `phalcom-core/tests/lang/blocks/pending/blocks_argument_to_method.ph` uses
     `numbers.reduce(0) { acc, n => acc + n }`. **`reduce` does not exist on
     `List`** — confirmed `core.ph:50`: "along with `map`/`reduce`/`filter`/literal
     syntax. Do not add those bodies" (U-STD's job). This fixture cannot be
     promoted this unit at all; either delete it, replace it with an
     `each`-based non-local-return case (mirrors the `findNegative` idiom, no
     `reduce` needed), or leave it in `pending/` with a comment noting the
     `reduce` blocker (it is *not* a U10 blocker — non-local return itself
     doesn't need `reduce`; the fixture author just happened to reach for it).
   These are not blockers for U10 itself — they're stale test-authoring
   assumptions from before U9/U-LIST's actual `List` API existed. Write fresh
   fixtures against the real `List` protocol (`at(_)`/`size`/`add(_)`/`each(_)`)
   instead of trying to make the pending ones parse as-is.

---

## 1. Preconditions — already verified, do not re-check

- U1–U9 are merged and green (commit `c9805d0`). `./scripts/verify.sh` is
  green on `main` right now. No worktree exists for this unit yet — the plan
  says branch `feat/u10-nonlocal-return`, but U8/U9 both actually landed
  **in-tree on `main`, no worktree** (see `STATE.md`'s "Working model" notes
  for both). Confirm with the user which convention to use before starting;
  do not assume the plan's worktree instruction still holds without asking
  (U9-implementation-spec.md §1 made the same call: "mirrors how U8 was
  actually executed, not the plan's worktree instruction").
- **U4's frame-token infrastructure is confirmed present and exactly as
  ADR-0013 describes:**
  - `FrameToken { frame_index: usize, generation: u64 }` — `frame.rs:19-31`.
  - `CallFrame.generation: u64`, minted monotonically by
    `VM::new_call_frame` via `self.next_frame_generation` (`vm.rs:540-552`).
  - `CallFrame::token(&self, frame_index: usize) -> FrameToken` — `frame.rs:102-104`.
  - `BlockObject { closure: ObjRef, home_frame_token: FrameToken }` — `block.rs:18-23`.
  - `VM::current_frame_token()` (`vm.rs:556-562`) stamps every
    `Bytecode::Closure` execution's resulting `BlockObject` with the *creating*
    frame's token (`vm.rs:761-762`) — this is where a block's home token gets
    set; U10 does not touch this.
- **`return` inside a block compiles today, but to the WRONG opcode.**
  `Statement::Return` (`compiler/lib.rs:536-551`) unconditionally emits
  `Bytecode::Return` regardless of whether the enclosing `FunctionState` is a
  method body or a block-literal body — there is no distinction today. This
  means `return` inside a block currently returns from **the block's own
  frame only** (a silent semantic bug, not a compile error — `findNegative`
  above would currently return early from the *inner* block's `.call()`
  activation, not from `findNegative` itself). No `// U10:` placeholder
  comments exist anywhere in the tree — the plan's §6 "remove any U4 `//
  U10:` placeholder markers" has nothing to find; skip that step.
- **The `is_method` parameter to `compile_block` is your compile-time
  block-vs-method discriminant — already exists, no new plumbing needed.**
  `compile_block(statements, name_sym, params, is_method, is_constructor)`
  (`compiler/lib.rs:362-369`) is called with `is_method=true` from all four
  method-family sites (`Method`/`Getter`/`Setter`/`Construct`,
  `compiler/lib.rs:761,806,827,867`) and `is_method=false` from exactly one
  site — `Expr::Block`'s compilation (`compiler/lib.rs:1156`), the only real
  block-literal path. Add a field to `FunctionState` (e.g. `is_block: bool`,
  set to `!is_method` in `compile_block`) and gate `Statement::Return`'s
  opcode choice on `self.functions.last().unwrap().is_block`.
- **No `ReturnOutsideFunction` enforcement currently fires from the
  compiler** — `SyntaxErrorKind::ReturnOutsideFunction` exists
  (`phalcom-ast/src/error.rs:105-107`) but nothing in `compiler/lib.rs`
  raises it; not this unit's job to add that check, just noting it so you
  don't go looking for where `return` gets rejected outside a function (it
  doesn't, today).
- Run `./scripts/verify.sh` on your starting tree before the first edit to
  confirm your baseline is green.

---

## 2. The one architectural fact that makes or breaks this unit

**Every block invocation, with no exception, runs through
`primitive::block::block_call`, which re-enters `VM::run_until` recursively
on the Rust call stack.** `.call()`/`.call(_:…)`, the sacred `whileTrue`
fallback, and any future `.each(_:)`/collection combinator written in `.ph`
that calls `block.call(x)` — all of them dispatch `call` as an ordinary
`Bytecode::Invoke` to the `MethodKind::Primitive(block_call)` native fn
(`primitive/block.rs:77-99`), and `block_call` does:

```rust
let base_frames = vm.frames.len();
vm.frames.push(frame);
vm.run_until(base_frames)   // <-- recursive Rust call, returns synchronously
```

`vm.frames` is one shared `Vec` across every level of this recursion, but
**there can be multiple live `run_until` Rust stack frames at once**, each
with its own `base_frames` floor, each mid-iteration of its own `loop { }`
dispatch. A block's home frame — the method activation `return` must unwind
to — is, by construction, **always** in an *outer, currently-suspended*
`run_until` invocation, never the innermost one actually executing the
block's `return` bytecode (the block itself was only reached via at least one
`block_call` reentry). Concretely, for `numbers.each { n => ... return n }`
where `each` is `.ph`-defined and calls `block.call(n)` per element: the
`return` executes inside `run_until(base=2)` (the reentry `each`'s own
`.call(n)` triggered), but its target — `findNegative`'s frame — lives at
index 0, *below* that floor, inside the **outermost** `run_until(base=0)`.

**This means the naive version of the plan's design — "walk `self.frames`,
pop down to the home frame, return its value" — is only correct if you also
handle every intermediate Rust-level `run_until`/`block_call` frame that sits
between where `return` executes and where the home frame lives.** You cannot
"return" past those Rust stack frames by mutating `self.frames` alone; if you
don't also prevent `call_method`'s `MethodKind::Primitive` arm from doing its
*normal* post-call stack bookkeeping when this happens, you'll silently push
a duplicate copy of the return value and corrupt the stack the very first
time a non-local return crosses more than one call boundary (i.e. the exact
`findNegative`/`each` case the plan's own test strategy calls for — this is
not an edge case, it's the primary acceptance test).

**The fix (derived from reading `run_until`'s and `call_method`'s actual
bodies — verify against HEAD, this is not speculative):**

1. `Bytecode::Return`'s existing single-frame handler (`vm.rs:1018-1034`) is
   the pattern to generalize: pop the return value, `close_upvalues_from`,
   truncate the stack to the popped frame's `stack_offset`, and — critically
   — check `self.frames.len() <= base_frames` to decide whether to `return
   Ok(value)` from `run_until` or push the value and keep looping. **You do
   not need to touch this check or `run_until` at all.** It already does the
   right thing for *every* level of nested `run_until`, for free, **as long
   as the stack and frame vec are in the correct final state by the time each
   level re-checks it** — which is exactly what step 2 guarantees.

2. `Bytecode::ReturnNonLocal`'s handler does the **entire** unwind eagerly, in
   one shot, at the single point where it executes (it does not "wait" for
   control to reach the right Rust frame — there is no way to do that other
   than mutating shared VM state up front):
   - Read the executing block frame's `home_frame_token` (the new
     `CallFrame.home_frame_token` field from §0 point 3 — read it off
     `self.frames.last()`, i.e. the frame currently running the `return`).
   - Walk `self.frames` looking for the frame at `token.frame_index` whose
     `.generation == token.generation`.
     - **Not found (or found but generation mismatch)** → the home method
       already returned. Raise `DeadFrameError` (§0 point 1). Do **not**
       mutate `self.frames`/`self.stack` before confirming the token is
       live — a partial mutation followed by an error would corrupt state
       for whatever catches the error.
     - **Found** → this is the frame `return` is unwinding to and *through*
       (the return also exits this frame, same as an ordinary `return`
       exits its own frame). Evaluate `e` (already on the stack top from
       compiling the return expression), then:
       - `self.close_upvalues_from(home_frame.stack_offset)` — one call
         suffices for *every* frame being popped, not just the innermost:
         `close_upvalues_from(from)` (`vm.rs:590-597`) scans
         `self.open_upvalues.range(from..)`, i.e. it closes every open
         upvalue at or above `from`, and every popped frame's own
         `stack_offset` is `>= home_frame.stack_offset` by construction
         (call depth only grows the stack). Must run **before** truncating
         the stack, same ordering `Bytecode::Return` already uses.
       - `self.stack.truncate(home_frame.stack_offset)`
       - `self.stack.push(return_value)` (surfaced through
         `self.surface_absence`, matching `Bytecode::Return`'s bare-`return`→`None`
         handling)
       - `self.frames.truncate(token.frame_index)` — removes the home frame
         and everything above it in one call.
   - Do **not** attempt to "return" a value from this match arm the way
     `Bytecode::Return` does (`return Ok(value)` out of `run_until`) — the
     surrounding `loop { }` just continues to its next iteration, and the
     **existing, unmodified** top-of-loop check
     (`if self.frames.len() <= base_frames { pop stack, return Ok(...) }`)
     picks the value up correctly for whichever `run_until` instance (there
     may be several nested ones) first finds its own floor at or above the
     new, shrunk `self.frames.len()`.

3. **The one other change needed:** guard `call_method`'s
   `MethodKind::Primitive` arm (`vm.rs:386-395`) against exactly this
   situation. It currently does:
   ```rust
   let result = native_fn(self, &receiver, &args);
   result.map(|result| {
       self.stack.truncate(receiver_idx);
       self.stack.push(result);
   })
   ```
   `receiver_idx` was computed **before** calling `native_fn`. If `native_fn`
   (e.g. `block_call`) internally triggered a non-local return whose target
   was at or below the frame that issued this call, step 2 above has *already*
   truncated `self.stack` well past `receiver_idx` and pushed the correct
   value — so `self.stack.truncate(receiver_idx)` is a silent no-op
   (`Vec::truncate` past the current length does nothing) and the subsequent
   `self.stack.push(result)` pushes a **second, duplicate** copy of the same
   value, corrupting the stack for whatever frame resumes next. Fix: capture
   `let frames_before = self.frames.len();` immediately before calling
   `native_fn`; if `self.frames.len() < frames_before` afterward, skip the
   truncate+push entirely (the correct value is already in place — just
   propagate `Ok(())`/the error through unchanged). No new `VM` field is
   needed; this is a local before/after comparison.
   - Trace this through the `findNegative`/`each` case to convince yourself:
     `each`'s own frame and `findNegative`'s frame both get removed by step
     2's `self.frames.truncate`; the guard fires in the `run_until` instance
     that was running `each`'s `.call(n)` `Invoke` (skips its stack write);
     control falls through to that instance's own top-of-loop check, which
     now sees `self.frames.len() <= its own base_frames` and exits cleanly
     via the *unmodified* `Bytecode::Return`-style epilogue, propagating
     `Ok(n)` up through `block_call` → `call_method`'s Primitive arm (which
     also correctly no-ops, same guard) → the next `run_until` up, and so on,
     until the outermost instance whose floor is finally at or above the new
     `self.frames.len()` picks up the value for real. No opcode after
     `ReturnNonLocal` in any of these frames' bytecode ever executes again —
     confirm this by checking that neither `Bytecode::Invoke`'s hit-path nor
     `call_method`'s `Closure` arm does anything else after the call
     completes (they don't, as of HEAD — verify this hasn't changed).

**Concurrency note (forward-looking, per the plan's §4):** everything above
operates on the single `self.frames`/`self.stack` the current fiber owns.
Don't hardcode an assumption that would make a later per-fiber frame vector
impossible — e.g. don't reach for a *global* mutable index into a
single-fiber-wide `Vec` anywhere outside `VM`'s own fields you're already
touching. Nothing in this design requires anything beyond `VM`'s existing
`frames`/`stack`/`open_upvalues`, so this should already be satisfied; just
don't add anything that assumes "there is exactly one call stack, ever."

---

## 3. Confirmed write-set (re-grep line numbers before editing — your own
edits will shift later ones in this list)

| File | Exact change |
|---|---|
| `phalcom-core/src/error.rs` | Add `RuntimeError::DeadFrameError` — plain `thiserror` variant, no span, no miette (§0 point 1). Message should name the non-local-return-to-a-dead-frame failure clearly enough for a `check_negative` substring match. |
| `phalcom-core/src/bytecode.rs` | Add `Bytecode::ReturnNonLocal` (no operand needed — the token comes from the executing frame, not the instruction stream). Full rustdoc citing ADR-0013 + blocks.md §5, and explicitly noting it does **not** return a value the way `Bytecode::Return` does (§2 point 2's "does not push Ok() out of run_until" behavior) — this is exactly the kind of non-obvious control-flow fact §7's docs mandate belongs in the `///`. |
| `phalcom-core/src/frame.rs` | Add `CallFrame.home_frame_token: Option<FrameToken>` (§0 point 3). Update `CallFrame::new` (keep it `None` by default, matching `generation: 0`'s existing pattern) and add a setter or a second constructor `block_call` can use — implementer's choice, document it. `FrameToken` is already `Copy`, so `Option<FrameToken>` keeps `CallFrame` `Copy` — do not break that (`vm.rs:1` doc comment on `CallFrame` calls this out as load-bearing: "the whole frame is `Copy`, so the VM keeps frames in a plain `Vec`"). |
| `phalcom-core/src/compiler/lib.rs` | (a) `FunctionState`: add `is_block: bool` (or equivalent), set from `!is_method` in `compile_block` (`compiler/lib.rs:362-379`). (b) `Statement::Return` (`compiler/lib.rs:536-551`): emit `Bytecode::ReturnNonLocal` instead of `Bytecode::Return` when `self.functions.last().unwrap().is_block` (and not a constructor — constructors can't be block bodies, `is_method=true` always accompanies `is_constructor=true`, so this shouldn't even be reachable, but don't assume; gate on `is_block` specifically). |
| `phalcom-core/src/vm.rs` | (a) `call_method`'s `MethodKind::Primitive` arm (`vm.rs:386-395`) — the frames-before/after guard (§2 point 3). (b) New `Bytecode::ReturnNonLocal` match arm in `run_until`'s dispatch loop — the eager unwind (§2 point 2). (c) `primitive::block::block_call` is **not** in `vm.rs` (it's `primitive/block.rs`) but needs to read the receiver's `Object::Block(block).home_frame_token` and thread it into the `CallFrame` it constructs — check whether `new_call_frame` needs a new parameter or whether setting the field after construction (`frame.home_frame_token = Some(token)`) is cleaner given `CallFrame` is `Copy`; document the choice. |
| `phalcom-core/src/primitive/block.rs` | `resolve_callable` (`block.rs:26-35`) currently discards the `BlockObject` and returns only the `ClosureObject` handle — it needs to also surface the block's `home_frame_token` (`None` if the receiver is a bare `Object::Closure`, e.g. a `Method`'s callable reflectively used as a `Function`, functions.md — those have no lexical home frame and no non-local-return target). `block_call` (`block.rs:77-99`) stamps the pushed `CallFrame` with that token. |
| `phalcom-core/tests/lang.rs` | Un-ignore `blocks_pending` cases as they get rewritten (§0 point 4) — either fold the rewritten fixtures into the main `blocks` group or leave `blocks_pending()` pointing at whatever's left in `pending/`. Add a `DeadFrameError` negative case to `runtime-errors` (`check_negative`, mirroring `runtime_perform_unknown_selector.ph`'s pattern) — an escaped block called after its home method already returned. |
| `phalcom-core/tests/fixtures/golden/` + `golden.rs` | Optional: a `findNegative`-style golden if you want byte-exact regression coverage beyond the `lang.rs` fixture pair; not required if the `tests/lang/blocks/` PASS case already covers it (prefer one strong `.ph`/`.expected` pair over duplicating in both harnesses unless the golden harness's full-pipeline subprocess execution catches something the `lang.rs` harness wouldn't — probably not needed here). |

**Explicitly NOT touched:** `phalcom-core/bin/phalcom/disasm.rs` (§0 point
2), `phalcom-core/src/closure.rs`, `phalcom-core/src/callable.rs` (U4's
capture/upvalue-descriptor machinery is unrelated to *which* opcode a
`return` compiles to), `run_until`'s top-of-loop epilogue itself (§2 point 2
— it needs zero changes, that's the whole point of the design).

---

## 4. Build order

1. `error.rs` — `RuntimeError::DeadFrameError`.
2. `bytecode.rs` — `Bytecode::ReturnNonLocal`.
3. `frame.rs` — `CallFrame.home_frame_token`.
4. `primitive/block.rs` — `resolve_callable` surfaces the token; `block_call`
   stamps the frame.
5. `compiler/lib.rs` — `FunctionState.is_block`; `Statement::Return` opcode
   choice.
6. `vm.rs` — the `ReturnNonLocal` handler (§2 point 2) **and** the
   `call_method` Primitive-arm guard (§2 point 3) **together, in the same
   checkpoint** — the first is unsound without the second; don't land one
   without the other even if `verify.sh` happens to stay green on a
   single-level test in between (write the multi-level `findNegative`/`each`
   case *before* declaring this checkpoint done, not after).
7. Tests — rewritten `blocks_non_local_return`/escaped-block `DeadFrameError`
   fixtures + `lang.rs` wiring + the negative golden.

---

## 5. Test strategy — concrete fixtures

- **`findNegative`-equivalent (multi-level unwind, the load-bearing case).**
  Rewrite `pending/blocks_non_local_return.ph` off `List.new()`/`.add(_)`
  (§0 point 4) — this is the one that actually exercises §2's reentrant
  unwind (`each` calling `.call()` internally), not a trivial single-level
  `.call()` return. Do not accept a version of this test that only calls the
  block directly (`{ return x }.call()`) as sufficient — that never crosses
  more than one `run_until` boundary and would pass even with the naive,
  broken design from §2's opening paragraph.
- **`DeadFrameError` on an escaped, dead-home block.** A method that builds
  and returns a block referencing `return` without calling it, then a
  *separate* call site invokes that block after the method has already
  returned — e.g.:
  ```phalcom
  class Maker {
    make() { return { return 1 } }
  }
  let b = Maker.new().make()
  b.call()   // Maker.make's frame is gone — DeadFrameError
  ```
  Goes in `tests/lang/runtime-errors/` as a `check_negative` case
  (substring-match the `DeadFrameError` message, same convention as every
  other `runtime-errors` fixture).
- **Escaping-capture regression guard.** Confirm `blocks_escape.ph`
  (`tests/lang/blocks/blocks_escape.ph` — the `makeCounter` upvalue-promotion
  golden) still passes byte-identical: it doesn't use `return`, but it's the
  test most likely to regress if `close_upvalues_from`'s call site/ordering
  inside the new `ReturnNonLocal` handler is wrong.
- **Value-less non-local return.** `return` with no expression inside a
  block should surface `None`, mirroring `Bytecode::Return`'s existing
  bare-`return`→`None` behavior (`vm.rs:1019-1023`) — add one small case
  asserting this, since it's an easy spot to regress by forgetting
  `surface_absence` in the new handler.

---

## 6. Mandatory rules (unchanged from `U10-plan.md` §7 — repeated for emphasis)

- Full rustdoc on every new/changed public item: `Bytecode::ReturnNonLocal`,
  `RuntimeError::DeadFrameError`, `CallFrame.home_frame_token`, the
  `ReturnNonLocal` handler and the `call_method` guard (explain the
  frames-before/after invariant inline — §2 point 3 — since it's the least
  obvious line in the whole diff). `cargo doc --workspace --no-deps` — zero
  new warnings.
- `./scripts/verify.sh` exit 0 is your sole sign-off — reviewer is OFF for
  U10 (not in the load-bearing set U1/U2/U4/U6). Self-verify harder than
  usual: prove the dead-frame path with a real escaped-block test, not just
  a live-frame happy path, and confirm the unwind pops upvalues correctly
  (§5's escaping-capture regression guard).
- Run `graphify update . --no-cluster` before every commit.
- Commit at each green checkpoint (per build-order step or logical cluster),
  never a non-compiling tree.
- Record `STATE.md`/`PHASE2-INDEX.md`/`DEFERRED.md` updates the same way U8
  and U9 did — mark U10 landed, note the two corrected pending fixtures as a
  `DEFERRED.md` entry if `blocks_argument_to_method.ph`'s `reduce` blocker
  isn't resolved in this unit.
- **Hard stop when green:** do not begin U11/U-LEX/U-STD.

---

## 7. Return contract (answer all of these)

- Confirm (or report a deviation from) each correction in §0.
- Walk through exactly how you implemented §2's eager-unwind + Primitive-arm
  guard — quote the actual committed code for both, and confirm you tested
  the multi-level (`each`-calling-`.call()`) case specifically, not just a
  direct single-level `.call()` return.
- The `findNegative`-equivalent golden's output, and the `DeadFrameError`
  golden's output.
- Confirmation `close_upvalues_from` runs before the stack truncation in the
  new handler, and that `blocks_escape.ph` still passes byte-identical.
- Confirmation U4's closure/capture/tower code (`closure.rs`, `callable.rs`,
  the `Bytecode::Closure` handler itself) was **not** modified — only the new
  `CallFrame` field and its two write sites (`block_call`, and wherever you
  chose to set it) touch anything block/closure-adjacent.
- Which two `pending/` fixtures you touched, and exactly how (rewritten
  in-place vs deleted vs left pending with a note — §0 point 4).
- Files changed, `verify.sh` tail, `cargo doc` tail.
- Any new `DEFERRED.md` entries.
