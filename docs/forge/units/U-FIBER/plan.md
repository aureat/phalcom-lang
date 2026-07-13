# U-FIBER — Work order: cooperative `Fiber` (bare) on the restricted re-entrant loop

_Self-contained implementation plan for **one** implementer. **Reviewer ON** (deep VM change) — hand the
diff to `phalcom-reviewer`; never self-approve. **Worktree isolation** (mutates `vm.rs`/`heap.rs`/
`core.ph` while U-ITER and U-CORE units are live). Green gate: `./scripts/verify.sh` exits 0 +
`cargo doc --workspace --no-deps` clean. Grounded in **[ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md)**
(execution model) + **[concurrency.md §1](../../../spec/v0.2/concurrency.md)** (surface) +
**[forward-compat.md §7](../../../spec/v0.2/core/forward-compat.md)** (the code-grounded foreclosure audit,
D1–D7). Floor extension authorised by **ADR-0030 §Consequences** (the ADR-0019 amendment convention) — no
new ADR needed._

> **v0.2 scope decision (user, 2026-07-12): BARE `Fiber` ONLY.** `new`/`call`/`try`/`yield`/`current`/
> `abort` — enough for generators and the `for`-yield ergonomic. **`Future`/`async`/`await`/scheduler are
> a separate later track** ([concurrency.md §2](../../../spec/v0.2/concurrency.md), unbuilt); this unit
> ships **no** `Future`, **no** ready-queue, **no** `System` scheduler hooks.

---

## 0. Phase 0 — the forward-compat §7.3 pre-fiber audit (do this FIRST, on real HEAD)
**Decision (user): the §7.3 audit + D5/D7 hardening live INSIDE U-FIBER as Phase 0** — the typed switch
signal has no consumer to verify against until fibers exist, so it is not a separable unit. Phase 0 is a
**read + targeted-fix** gate; the unit does not proceed to Phase 1 until all five hold on HEAD.

| Invariant | Check on HEAD | Action if violated |
|---|---|---|
| **D4 — `next_frame_generation` stays VM-global** | `vm.rs` L72/583 — the monotonic counter is a `VM` field, **not** in any per-fiber struct. | **Hard stop.** A per-fiber counter lets fiber B's `(frame_index, generation)` collide with a live frame in fiber A → non-local return into the **wrong fiber**. Keep it global. |
| **D2 — no new `Value` arm for heap types** | `value.rs` L31 — `Fiber` must be `Object::Fiber` reached via `Value::Obj(ObjRef)`, like `List`. `concurrency.md` §1's `Value::Fiber(PhRef<…>)` is **stale** (predates the handle heap). | Do not add `Value::Fiber`; follow the `List` precedent. |
| **D3 — `stack_offset` frame-relative** | `frame.rs` L75 — offsets are window-relative, so a per-fiber stack starting at 0 needs no rebasing. | Keep relative; the O(1) switch (§2 below) depends on it. |
| **D5 — the typed fiber-switch signal** | `vm.rs` `call_method` Primitive arm (L~406–443) reconciles a non-local return by a `frames.len()` **shrink heuristic**. A fiber switch **also** moves `frames.len()`. | **Build the typed signal (Phase 1 §2.3).** Phase 0 just confirms the heuristic is the only frame-count consumer and scopes the change. |
| **D7 — U-CORE-6 unwind is fiber-local** ⚠️ **the live risk** | The just-landed U-CORE-6 `throw`/`ensure` unwind (`ADR-0008`) **must operate on `self.frames` only** and be able to **stop at a designated floor**, not hard-code process-floor termination. **Read the U-CORE-6 unwind code on HEAD.** | If it terminates at the process floor unconditionally, **U-FIBER fixes it first**: parameterise the unwind stop-point so a failing fiber captures its `Error` into its result slot and resumes its resumer, never killing the host (concurrency.md §1 pt 4, forward-compat §7.1 D7). This is the one Phase-0 item that can become real code. |

**Phase 0 return:** a one-paragraph verdict per row (verified / fixed-here) before any Phase 1 edit.

---

## 1. Mission (one sentence)
Give Phalcom its sole concurrency primitive — a **cooperative, single-threaded `Fiber`** as an
`Object::Fiber` arena variant owning its own value+frame stacks, switched by an **O(1) pointer swap**, with
`Fiber.yield` integrating with the **top-level** `run_until` only and raising a catchable
**`CannotYieldAcrossNativeFrame`** when a native frame sits between the fiber floor and the yield
(**restricted Option A**, [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §4).

## 2. Design (realise ADR-0030 §1–§7; do not re-litigate the model)

### 2.1 `FiberObject` is one arena variant, no new `Value` arm (ADR-0030 §2, D2)
Add `Object::Fiber(FiberObject)` to `heap.rs`. `FiberObject` owns: `stack: Vec<Value>`,
`frames: Vec<CallFrame>`, `status` (`suspended` / `running` / `done` / `failed`), a **resumer** link
(`Option<ObjRef>` — the fiber to hand control back to), a **result slot** (last yielded/returned value or
captured `Error`), and its **entry closure**. Reached through `Value::Obj(ObjRef)` exactly as native `List`
is.

### 2.2 Fiber switch is an O(1) pointer swap (ADR-0030 §3, D3)
Relocate the VM's "current stack / current frames" behind **`current: ObjRef`** into the running
`FiberObject`. `call`/`yield` swap **which fiber the dispatch loop reads** — never copy stacks. Because
`stack_offset` is frame-relative (Phase 0 D3), per-fiber stacks starting at 0 need no rebasing.

### 2.3 The switch signal is TYPED, not a length delta (ADR-0030 §5, D5) — the crux
`call`/`yield` reconcile with the dispatch loop through an explicit `ControlFlow`/switch value out of the
primitive — **not** the `frames.len()` heuristic the Primitive arm uses to detect a non-local return. A
fiber switch also changes `frames.len()`; conflating the two misreads a swap as a return. This is the
distinct **third cause** forward-compat §7.3 told every pre-fiber unit to leave room for — and the exact
dependency **ADR-0033** waits on (kept Deferred per the user; U-FIBER only *builds* the signal, it does
not trampoline the block call-site).

### 2.4 `Yield` opcode + restricted-yield guard (ADR-0030 §4, §Consequences)
Add a `Yield` opcode (+ disasm arm). `Fiber.yield` integrates with the **top-level** `run_until`; if a
re-entrant native primitive (`block_call` and everything above it) is on the native Rust stack, raise
**`CannotYieldAcrossNativeFrame`** (a thrown, catchable `Error`) rather than corrupting the suspended
position. **Suspends freely:** pure sends + inlined control flow (the `counter` generator; and — once
U-ITER lands — `for (x in coll) { Fiber.yield(x) }`). **Foreclosed:** `.each { yield }` under a native
`block_call` (documented, ADR-0033 is the deferred lift).

### 2.5 Non-local return & error unwind stay fiber-local (ADR-0030 §6, D4/D7)
Once `self.frames` is the current fiber's vector, `ReturnNonLocal` searches only that fiber; a token whose
home is on another fiber fails the generation check → `DeadFrameError` (falls out for free, D4). The
Phase-0-hardened error unwind operates on `self.frames` and stops at the **fiber floor**, so a failing
fiber captures its `Error` into its result slot instead of terminating the host.

### 2.6 Floor extension (ADR-0019 amendment, authorised by ADR-0030)
The primitive set — `new`/`call`/`try`/`yield`/`current`/`abort`, the `Yield` opcode, per-fiber stack
machinery — is a deliberate floor extension, authorised by ADR-0030 §Consequences (as ADR-0020/0023 did
for `List`/`hash`). **Bump the floor census;** no new ADR. Class-side `yield(_)`/`current`/`abort(_)` are
ordinary metaclass methods (D6) and must pass `verify_invariants()` at bootstrap.

### Rubric — hazards & preclusion (mandatory)
- **native-stack ⊗ suspendable control (CROWN JEWEL).** Handled by restriction, not machinery: the guard
  (§2.4) refuses the unsafe case rather than corrupting it. No native fiber stacks ⇒ nothing new for a
  future moving GC to scan (ADR-0009 preserved).
- **GC roots for parked fibers (ADR-0030 §7, D1).** Invariant to encode now even pre-GC: a reachable,
  non-`done`/`failed` `FiberObject`'s stacks are roots — keeping them *inside the arena object* is what
  lets the future collector reach them. Do not stash fiber stacks in native memory.
- **`next_frame_generation` VM-global (D4).** The single load-bearing invariant; a per-fiber counter is a
  silent cross-fiber miscompile. Pin an invariant test.
- **Option-B additivity.** A→B (full trampoline / ADR-0033) must stay purely additive: de-recursing the
  callback primitives later only *removes* the guard. Do not bake the restriction anywhere it can't be
  lifted by deletion.
- **Precedent:** Lua 5.1 restricted coroutines (the direct model). Stackful (`corosensei`) rejected
  (ADR-0030 §Alternatives) — permanently constrains the GC; do not reopen.

## 3. Confirmed write-set (re-validate with `graphify affected` on HEAD)
| File | Why | Phase |
|---|---|---|
| `phalcom-core/src/heap.rs` | `Object::Fiber(FiberObject)` variant + the struct | 1 |
| `phalcom-core/src/vm.rs` **(SPINE)** | `current: ObjRef`; read current stack/frames through it; typed `ControlFlow` switch signal in `call_method` Primitive arm + `run_until`; `Yield` handling | 1 (+Phase 0 D5/D7) |
| `phalcom-core/src/bytecode.rs` + `bin/phalcom/disasm.rs` | `Yield` opcode + disasm arm | 1 |
| `phalcom-core/src/primitive/fiber.rs` (**new**) | `fiber_new`/`call`/`try`/`yield`/`current`/`abort` | 1 |
| `phalcom-core/src/universe.rs` | register `Fiber` class + primitives; bootstrap; floor census bump | 1 |
| `phalcom-core/core/core.ph` | `class Fiber` surface wiring to the primitives (class-side `yield`/`current`) | 1 |
| `phalcom-core/tests/lang/concurrency/` + `tests/lang/MANIFEST.md` | goldens; graduate `pending/concurrency_fiber_yield_resume`; the restricted-yield guard negative | all |
| `phalcom-core/tests/invariants.rs` | `next_frame_generation`-global + fiber-local-return invariant tests | 1 |
| `value.rs` | **NO change** (D2 — no `Value::Fiber` arm); listed to make the non-edit explicit | — |

**Deliberately NOT in scope:** any `Future`/`async`/scheduler/`System` code (v0.2 scope decision); the
`iterate`/`for` machinery (U-ITER); the block-call trampoline (ADR-0033, Deferred).

### 3.1 Collision risk (flag, don't resolve)
- **`vm.rs` / `heap.rs`** — spine files; confirm no concurrent unit holds them. **Worktree isolation** is
  mandated for this unit for exactly this reason.
- **`core.ph` — never two editors.** U-ITER (`List#iterate`) and the U-CORE track both edit `core.ph`.
  U-FIBER's `class Fiber` reopen must **serialize** against them.

## 4. Build order (small, independently-green diffs)
0. **Phase 0 audit** (§0) — verdicts + the D7 unwind fix if needed. Green (existing suite still passes).
1. **`Object::Fiber` + `current: ObjRef` plumbing** — VM reads current stack/frames through `current`; a
   single default "main" fiber wraps today's behaviour. Verify the whole existing suite stays green (this
   is a pure refactor with no surface change). **This is the highest-risk diff — land it alone.**
2. **Typed switch signal** — replace the `frames.len()` non-local-return heuristic with the `ControlFlow`
   enum; existing non-local-return goldens must stay green.
3. **`Yield` opcode + `new`/`call`/`yield`** — the minimal generator; graduate
   `pending/concurrency_fiber_yield_resume`; pin the `counter` generator.
4. **Restricted-yield guard** — `.each { yield }` → `CannotYieldAcrossNativeFrame` (catchable); negative
   golden.
5. **`try`/`current`/`abort` + failure capture** — a failing fiber stores its `Error` and resumes the
   resumer without host termination (D7 exercised).

## 5. Mandatory rules
- **Docs:** `///` on `FiberObject` + every field/variant/primitive, citing ADR-0030 §. `cargo doc` clean.
- **Green gate:** `verify.sh` exits 0; no new clippy; `unsafe` **forbidden** (the whole point of Option A
  over stackful C is zero `unsafe` — if a slice seems to need it, stop and flag).
- **Reviewer ON**; **worktree isolation**. Follow `rust-best-practices`, `rust-sanitizers-miri` for the
  stack-swap plumbing (run the miri lane if U0 wired it).

## 6. Test strategy (the green gate must assert) — `concurrency` label
- **counter generator (PASS):** `Fiber.new { let n = 0; while (true) { Fiber.yield(n); n = n + 1 } }` →
  successive `call`s yield 0,1,2… (ADR-0030 §4 canonical).
- **resume value (PASS):** the argument to `call(_)` becomes the value of the suspended `yield`.
- **restricted-yield guard (NEGATIVE, PASS):** `Fiber.new { [1,2].each { x => Fiber.yield(x) } }` raises a
  **catchable** `CannotYieldAcrossNativeFrame` (not a host abort).
- **failure capture (PASS, D7):** a fiber that `throw`s ends `failed`, `try` yields the `Error`, the host
  keeps running (proves the unwind stops at the fiber floor).
- **fiber-local non-local return (PASS/NEGATIVE, D4):** a `return` token from fiber A used in fiber B →
  `DeadFrameError`; pin `next_frame_generation` stays VM-global (invariants.rs).
- **cross-unit (PENDING → graduates):** U-ITER's `for_generator_suspends` flips to PASS once both land —
  `Fiber.new { for (x in [1,2,3]) { Fiber.yield(x) } }` suspends freely.

## 7. Decisions (mostly settled — recorded, not open)
| ID | Decision | Resolution |
|---|---|---|
| DEC-FIB-scope | What ships in v0.2 | **Bare Fiber only** (user, 2026-07-12). Future/async/scheduler = separate track. |
| DEC-FIB-audit | §7.3 audit structuring | **Phase 0 inside U-FIBER** (user). |
| DEC-FIB-0033 | ADR-0033 timing | **Keep Deferred** (user). U-FIBER builds the switch signal but does **not** trampoline the block call-site. |
| DEC-FIB-A ⚠️ | Only genuinely-open item: **does the U-CORE-6 unwind already stop at a parameterisable floor**, or must U-FIBER refactor it? | **Resolve in Phase 0 against HEAD.** If already parameterisable, Phase 0 is read-only; else U-FIBER owns the fix. |

## 8. Must-not-preclude check
- **`Future`/`async`/`await` (deferred track):** not precluded — they derive from `Fiber` + a ready-queue;
  U-FIBER's `yield`/`resumer`/result-slot are exactly what `await` will suspend through. Leave the resumer
  link and result slot general (not generator-specific).
- **ADR-0033 (`CallBlock` trampoline):** not precluded — actively enabled. U-FIBER ships the typed switch
  (§2.3); ADR-0033 later only removes the guard. Keep the restriction lift-by-deletion.
- **Moving/tracing GC (ADR-0009):** not precluded — no native fiber stacks; parked-fiber stacks are arena
  roots (§Rubric). Do not introduce native-memory fiber state.
- **`ensure`-on-abandoned-fiber + resource limits** (experimental/fiber-ensure-and-limits.md): not
  precluded — a post-v0.2 layer over the same unwind; U-FIBER's fiber-local unwind is its seam.

## 9. Return contract (report to `phalcom-reviewer`)
Phase 0 verdicts per §7.3 row (esp. the D7/U-CORE-6 finding + any fix) · `Object::Fiber` shape + the
confirmed **no `Value::Fiber` arm** · the `current: ObjRef` refactor + proof the existing suite is green
after it (step 1) · the typed `ControlFlow` switch replacing the `frames.len()` heuristic + non-local-return
goldens still green · the `Yield` opcode + counter/resume goldens · the restricted-yield guard negative
(catchable) · failure-capture proof (host survives) · `next_frame_generation`-global invariant test · the
floor census bump (no new ADR) · confirmation **zero `unsafe`** · worktree/serialization notes vs U-ITER &
U-CORE `core.ph` · `verify.sh` + `cargo doc` (+ miri) tails · the U-ITER PENDING fixture that graduates.

## 10. Landed — status + follow-ons (2026-07-13)

**LANDED.** `Object::Fiber` + `current: ObjRef` (475dee8), the typed switch signal +
`call`/`try`/`yield`/`current`/`abort` + fiber-floor capture (482f235), then
U-FIBER-FIX's correctness follow-ons (root-fiber guards, gate-message split,
cross-fiber `DeadFrameError` golden, failure-cascade parked-frame cleanup, a
dedup pass) and a further arity-ordering + retention-cleanup fix (94487af — the
failure cascade now clears `.stack`/`.open_upvalues` alongside `.frames`, and
`fiber_resume`'s first-resume arity check runs before, not after, stealing the
resumer's live stacks). No `Yield` bytecode opcode landed (D-FIB-7 deviation,
sanctioned) — `Fiber.yield` is an ordinary class-side primitive send instead.

Two items this unit deliberately left open, now tracked as their own work:
- **[U-FIBER-REFLECT](../U-FIBER-REFLECT/plan.md)** — `Fiber#isDone`/`error`, the
  two Interface-table (concurrency.md §1) accessors this unit didn't ship. Needs
  no scheduler; unblocked now.
- **[U-GC](../U-GC/plan.md)** — this unit's own §Rubric flagged "GC roots for
  parked fibers" as an invariant to hold *pre-GC* (keep fiber stacks inside the
  arena object, never native memory) so a future collector could reach them.
  U-GC is now a real, owned unit plan (ADR-0050) that does exactly this — every
  `FiberObject` a program ever creates is a live-looking-but-often-dead retention
  root until it lands (no GC exists yet at all, not fiber-specific, but a
  fiber's full stack+frames+upvalues make the cost visible fastest of any single
  allocation in the language).
