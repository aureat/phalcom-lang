# U10 — Work order: non-local return + `DeadFrameError`

_Self-contained implementation plan for **one** `phalcom-implementer` agent. **Reviewer OFF** — U10 is
not in the load-bearing set (U1/U2/U4/U6); it self-verifies on the green gate + `cargo doc`. Grounded in
**ADR-0013** (frame-token non-local return — the infrastructure **U4 already stood up**) and
[`blocks.md`](../spec/blocks.md) §5 / [`functions.md`](../spec/functions.md) §2 /
[`object-model.md`](../spec/object-model.md) §4 (`DeadFrameError`)._

---

## 0. Mission (one sentence)
Make `return` inside a block unwind to the **enclosing method activation** (not just the block), by
consuming the **frame token U4 built** — comparing the block's `home_frame_token` generation against
the live frame and raising **`DeadFrameError`** when the home frame is already gone.

## 1. Hard guardrails (read before writing any code)
- **This unit consumes U4's frame token; it does NOT invent a new mechanism.** U4 already added the
  generation counter to `CallFrame`, the `home_frame_token` on `BlockObject`, and the mint/compare
  helper (`frame.rs`). U10 adds only the **return-unwind semantics** on top. If that infrastructure is
  absent on your base, **STOP and report** — U10 is scheduled in Wave F *after* U4 lands (handoff order
  U1→U2→U4→…→U10), never before.
- **Post-U1 substrate.** Frames, blocks, and upvalue cells live in the `Heap`, reached by `Copy`
  handles through `heap.get*`. No `Rc<RefCell>`/`PhRef`.
- **Scope is the `^`/`return`-in-block path only.** Do not touch closure creation, upvalue capture,
  block dispatch, or the tower — those are U4's and are frozen. Do not add `break`/`continue`
  (blocks §6 — they exist only in loop sugar, which is U5).
- **Realize, don't re-litigate.** The token = (frame ptr + generation); the failure mode = a cheap
  integer generation compare → `DeadFrameError` (ADR-0013 §Decision; the "raw pointer, no generation"
  alternative is *rejected* there). Implement it; do not survey alternatives.
- Stay inside the write-set (§3). If forced outside it, **STOP and report a conflict**; append
  out-of-scope ideas to [`DEFERRED.md`](DEFERRED.md).

## 2. Preconditions (verify first; do not assume)
- Runs in its own worktree off `main` (`feat/u10-nonlocal-return`), branched from the U4-inclusive green
  base. Confirm `./scripts/verify.sh` is green before the first edit.
- Confirm U4's infra exists on HEAD: `CallFrame` carries a generation counter, `BlockObject` carries a
  `home_frame_token`, and `frame.rs` exposes a token mint/compare helper. `graphify affected
  "CallFrame"`, `graphify explain "BlockObject"`, `graphify explain "FrameToken"` first.
- Confirm the compiler currently compiles a braced-block `return` as U4 left it (deferred / expression-
  position placeholder — U4 shipped no non-local path). U10 is the unit that gives it meaning.

## 3. Confirmed write-set (tight — sequenced after U4, so shared files are safe)
| File | Why it's in scope |
|---|---|
| `phalcom-core/src/bytecode.rs` | New `ReturnNonLocal` opcode (unwinds to the block's home frame). |
| `phalcom-core/src/compiler/lib.rs` | Emit `ReturnNonLocal` for a `return` inside a **block** body; a `return` in a **method** body keeps the ordinary `Return`. |
| `phalcom-core/src/vm.rs` | `ReturnNonLocal` handler: read the executing block's `home_frame_token`, compare generation against the live `CallFrame` stack, pop frames down to the home activation and return its value; on generation mismatch raise `DeadFrameError`. |
| `phalcom-core/src/error.rs` | `DeadFrameError` runtime-error variant (miette diagnostic, spans via `phalcom-common` ranges) (object-model §4). |
| `phalcom-core/src/frame.rs` | **Read-only if U4's compare helper suffices;** touch only to expose a "find home frame by token" search, and justify. |
| `phalcom-core/bin/phalcom/disasm.rs` | Disassemble `ReturnNonLocal`. |
| `phalcom-core/tests/lang.rs` | Non-local-return cases: `findNegative` early-exit; a stored escaped block whose home frame is dead → `DeadFrameError`. |
| `phalcom-core/tests/fixtures/golden/` + `golden.rs` | `findNegative`-style golden (block `return` exits the enclosing method) + a `DeadFrameError` golden. |

## 4. Design decisions (ADR-0013 / blocks.md §5 — realize, don't re-litigate)
- **`ReturnNonLocal` unwind.** When a block body executes `return e`, the VM evaluates `e`, then reads
  the executing `BlockObject`'s `home_frame_token`. It walks the live `CallFrame` stack to the frame
  whose (index, generation) matches the token, pops every frame above **and** the home frame, and yields
  `e` as the home method's result. This is exactly the `Return`-carrying-the-home-frame-token item
  functions.md §2 lists as unrealized.
- **`DeadFrameError` (blocks §5, object-model §4).** If no live frame matches the token's generation —
  the home method already returned — raise `DeadFrameError`. The generation compare is a cheap integer
  check that turns a use-after-free hazard into a clean runtime error (Smalltalk's `BlockCannotReturn`).
- **Safe by construction (blocks §2).** Only braced blocks can carry `return`; unbraced arrows are
  expression-only (enforced in U4's parser). U10 relies on that — it does not need to guard the unbraced
  form, and must not weaken it.
- **Unwind primitive alignment.** ADR-0008 notes `throw`/`return`/`abort` unify as one stack-unwinding
  primitive; U10 implements the `return` slice. Do **not** build the exception machinery here (U8/error
  handling own it) — but shape the unwind loop so it does not preclude reuse. **No BLOCKED-ON-DECISION**
  items: the mechanism is fully pinned by ADR-0013 + blocks.md §5.
- **Concurrency guard (forward-looking).** A non-local return must not cross a `Fiber` boundary
  (concurrency.md) — the home frame lives in the same fiber's stack. Fibers aren't implemented yet;
  don't hardcode an assumption (e.g. a single global frame vector indexed without a fiber tag) that
  would make the per-fiber stack impossible later. Note it in the module doc.

## 5. Build order (land as one coherent, reviewable diff)
1. **`DeadFrameError`** in `error.rs` (miette diagnostic + message).
2. **`ReturnNonLocal`** opcode in `bytecode.rs` + `disasm.rs`.
3. **Compiler** — emit `ReturnNonLocal` for a block-body `return`; leave method-body `Return` unchanged.
4. **VM** — the unwind loop: token read, generation compare, multi-frame pop, `DeadFrameError` on
   mismatch. Ensure closed-upvalue promotion (U4's `CloseUpvalue`) still runs for every frame popped
   during the unwind — escaping captures must survive the non-local exit.
5. **Tests** — `findNegative` golden + `lang.rs` case; a stored/escaped-block `DeadFrameError` golden +
   case. Snapshot the `ReturnNonLocal` disassembly.

## 6. Fold-in cleanup
None. Remove any U4 `// U10:` placeholder markers you now satisfy (they're inside your write-set).

## 7. Mandatory rules
- **Docs** ([`docs/rust-documentation-guidelines.md`](../rust-documentation-guidelines.md)): `///` on
  the `ReturnNonLocal` opcode, `DeadFrameError`, and every touched item, with `# Panics`/`# Errors`,
  intra-doc links, and ADR-0013 + blocks.md §5 citations. `cargo doc --workspace --no-deps` adds no new
  warnings.
- **Green gate:** `./scripts/verify.sh` exits 0. Pre-existing goldens byte-identical; new non-local /
  `DeadFrameError` goldens added deliberately. No new clippy warnings.
- **Self-verify (reviewer OFF).** U10 is not load-bearing; there is **no** independent `phalcom-reviewer`
  pass. That raises your own bar: prove the dead-frame path with a real escaped-block test, not just a
  live-frame happy path, and confirm the unwind pops upvalues correctly.

## 8. Return contract (to the orchestrator)
Report: the `ReturnNonLocal` semantics + how the token compare distinguishes live-home vs `DeadFrameError`
· the `findNegative` golden output · the escaped-block `DeadFrameError` golden output · confirmation that
closed-upvalue promotion still runs across an unwind · confirmation U4's closure/capture/tower code was
**not** modified · files changed · `verify.sh` + `cargo doc` tails · any `DEFERRED.md` entries. Because
reviewer is OFF, state explicitly that you self-verified the green gate and the dead-frame path.
