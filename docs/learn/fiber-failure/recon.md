# C3 recon — when a fiber fails

Procedure: [`AUTHORING-LEAN.md`](../AUTHORING-LEAN.md) (second run of the experimental variant; the
first, C2, is logged in its §8). All line numbers are HEAD = `cdd2117`.

Sibling docs read *before* writing this file, per the lean procedure's own finding 2:
[C1](../concurrency/restricted-loop.md) (all headings; §§Go, Wren, Cut, Forward pointers in full) and
[C2](../concurrency/parked-fiber.md) (§boundary L40-60, §Forward pointers L466-480).

---

## 1. Architecture vs representation

**Architecture.** An uncaught error inside a fiber does not propagate past that fiber. `run_until`'s
top-level driver (`base_frames == 0`) wraps the interpreter in a loop whose `Err` arm is the **fiber
floor**: it converts the `PhError` into a surface `Error` value, marks the fiber `Failed`, stores the
value in its `result` slot, and hands control to a resumer (ADR-0030 §6, `vm/dispatch.rs:290-338`).

**Representation — the axis that matters.** Phalcom has **two different deliveries for one error**,
and the floor is where one becomes the other:

| | Delivery | What it is, mechanically | What it does per frame |
|---|---|---|---|
| Inside a fiber | `Err(PhError)` returning up the **Rust** stack | a Rust `Result` unwinding through `run_until_inner` and every native re-entrant caller | `unwind_to` (`dispatch.rs:110-114`) — `close_upvalues_from`, *then* `frames.truncate`, *then* `stack.truncate`, in that order and documented as load-bearing (`dispatch.rs:97-103`) |
| At the floor | a `Value` in `FiberObject::result` | one enum write + one `Value` write + three `Vec::clear()`s | **nothing.** `frames.clear()`, `stack.clear()`, `open_upvalues.clear()` (`dispatch.rs:319-321`) |

So the representation of "this fiber failed" is *not a walked stack* — it is a status enum plus a
value in a slot, and the frames that were live are dropped in bulk. The floor does not unwind. It
**deletes**.

That is the whole doc: `unwind_to` exists, is documented, states its own ordering requirement in
prose, and the failure path does not call it.

## 2. The grip, grounded

> **An error leaves a fiber twice over, and the two exits are different machines. Inside the fiber it
> is a Rust `Err` walking frames down, closing upvalue cells before it reclaims their slots. At the
> fiber floor it stops being an unwind and becomes a value in a slot — the frames are not popped,
> they are `clear()`ed. Containment is implemented as deletion, and everything the unwind was doing
> on the way down is what the deletion forgets.**

Corollary the doc must land: the cascade (`call`-mode) walks *fibers*, not *frames* — it runs no
bytecode in any intermediate fiber (`dispatch.rs:296-303`, comment). So the second machine has no
per-frame step at all, by construction, for any number of fibers.

## 3. What was actually deliberated

ADR-0030 §6 (L120-131) deliberates the *containment*, in one paragraph, and it deliberates it
correctly: the ADR-0008 unwind "operates on `self.frames` only and stops at the **fiber floor**, so a
failing fiber captures its `Error` into its result slot instead of terminating the host." It is
explicit, and it is what shipped.

What no ADR section deliberates:

- **the `call`/`try` split** — that a `call`-mode failure re-raises into the resumer and a `try`-mode
  one is delivered as a value. It appears in the spec and in `FiberResumeMode`'s rustdoc
  (`heap/fiber.rs:28-44`) as a fact, never as a choice with a rejected alternative. **Wren's
  `call`/`try` pair is where it comes from** — ten `concurrency_fiber_wren_*` fixtures — but that is
  a *port*, not a deliberation (C1 §Wren already establishes this framing; do not re-argue it).
- **whether "stops at the floor" means "unwinds to the floor" or "abandons at the floor."** The ADR
  says the ADR-0008 unwind "stops at" the floor. The implementation never runs the ADR-0008 unwind
  at all on this path. Nothing records that as a decision. **This is the doc's honesty note and it
  must be stated as an absence, not as a rejected alternative.**
- **the cascade's no-bytecode rule** — an intermediate `call`-mode fiber's own code never resumes.
  Documented in a code comment, in no ADR.

So: the design-space walk for *containment vs. termination* is genuine and short (§6 exists); the
walk for *how the floor tears down* is **pedagogical reconstruction** and must say so. Two different
labels in one doc — do not apply one caveat to both.

## 4. Findings that change the doc

**F1 — E002 reproduces at HEAD, verbatim, but not with the recorded source.** The repro in
[`docs/errors/E002`](../../errors/E002-fiber-floor-upvalue-crash.md) uses `var`, which no longer
lexes at HEAD (`Token::Var` was removed by U-BINDINGS, commit `42aafce`); the file fails with
`Expected one of ";", newline`. Rewritten with `let` it panics exactly as recorded:

```
thread 'main' panicked at phalcom-core/src/vm/dispatch.rs:1062:61:
index out of bounds: the len is 0 but the index is 1
```

The plan's standing warning ("reproduce from scratch; do not cite from memory") paid off on the
first command. **E002's own repro block is stale and should be corrected in the same pass** — a
crash record whose repro does not compile is worse than none.

**F2 — the doc's *second* scar was fixed underneath it, three commits ago.**
[`docs/errors/E001`](../../errors/E001-gc-ensure-temp-root-uaf.md) (the `block_ensure` unrooted
pending result) is listed **OPEN** in [`docs/errors/README.md`](../../errors/README.md), and states
"There is no `temp_roots` — `push_temp_root` has zero occurrences in the tree." Both are false at
HEAD:

- `vm/gc.rs:148` defines `push_temp_root`; `primitive/block.rs:318-319` calls it on both the value
  and the `Raise` error paths — commit `cdd2117`, *"fix(vm): commit temp_roots GC escape-hatch"*.
- E001's own repro now runs clean: `{ "fresh" + "string" }.ensure { System.gc }` prints
  `freshstring`.

C2's forward pointer hands C3 "the two confirmed scars"; **one of them is paid.** This is a gift, not
a loss: the pair is now *the same defect family with one member fixed and one not*, and the fixed one
shows what the fix costs — an extra root vector consulted by `collect_roots`. E002 has no equivalent
hook, because the failure path has no unwind to extend. **The README and E001's status must be
updated in the same pass; a stale OPEN row is exactly the failure mode this repo has burned sessions on.**

**F3 — the cascade is observable, and it stops where the modes say.** Verified by running programs
(no tracing exists — see risks):

| Program | Observed |
|---|---|
| `inner` throws; `mid` does `inner.call()` then prints; root does `mid.try()` | root's `try` yields the *inner* error (`"inner boom"`), `mid`'s print never runs, **both** fibers report `isDone == true` |
| same, but `mid` does `inner.try()` | `mid` sees the error as a value, continues, returns `99` — the cascade stops at the first `try` |
| root does `f.call()` on a throwing fiber | host dies, prints `boom` — the root fiber has no resumer, so `run_until` returns `Err` (`dispatch.rs:324`) |
| resume a `Done` fiber | `cannot resume a finished fiber` |

**F4 — the natural way to observe "does the cascade skip an intermediate `ensure`?" is illegal, and
the reason is C1's.** Wrapping the resume in a cleanup —
`{ inner.call() }.ensure { … }` inside a fiber — does **not** test the cascade: `block_ensure` is a
native re-entrant frame, so the switch is refused and the fiber fails with
`CannotYieldAcrossNativeFrame` before `inner` ever starts. Observed: the `ensure` cleanup *did* run
(it is on the ordinary `Err` path), `inner.error` was `None`, and `mid.error` was
`Some(<CannotYieldAcrossNativeFrame>)` — the guard's error, not the inner one.

Consequence: **the restricted-yield guard is what keeps the cascade's no-cleanup behaviour mostly
unreachable**, exactly as C2 found for its `checking` clear-set. Same shape, second instance. Whether
it is *fully* unreachable is a B question (adversarial) — do not assert it.

**F5 — the failure path is the only teardown in the VM that skips `unwind_to`, and `unwind_to`'s
rustdoc argues the case against it in advance.** `dispatch.rs:97-103` says the order exists "so a
closure that escaped the throwing block still observes its captured locals rather than a
use-after-free once its stack slot is reclaimed." E002 is that sentence, happening. The doc should
quote the comment *before* showing the crash: the codebase predicted its own bug and then routed
around the prediction.

**F6 — the error is always reified, so a fiber's failure is never a Rust type.**
`capture_error_value` (`dispatch.rs:370-379`) passes a `RuntimeError::Raise`'s already-surface
`error` through, and wraps anything else (native VM errors with no surface form) in a bare kernel
`Error` instance carrying the rendered message. Hence `CannotYieldAcrossNativeFrame` — a VM-internal
condition — arrives in user code as a catchable `Error` value (C1 named this; C3 shows the machine).

## 5. Forbidden list

| Material | Owner | What C3 may do |
|---|---|---|
| The restricted-yield guard, `floor_depth` vs `native_reentry_depth`, `switch_pending`, "the loop is never told", the A/B/C execution-model design space, Lua's restricted-yield history, coloring, Go's stacks + `cgo`, Wren-as-surface-port | **C1** (shipped) | name in one line as a *cause* — F4's unreachability rests on it. Never re-derive. Go may reappear **only** on a different axis (unrecovered `panic` terminating the process — a containment argument C1 does not make); if it does, say explicitly that C1 spent Go on stacks |
| The four `mem::take`s, the twelve-field partition, `next_frame_generation` staying VM-global, no-rebasing, `current` is bookkeeping, fiber pooling, the `checking` clear-set asymmetry | **C2** (shipped) | cite as established. The three `clear()`s at `dispatch.rs:319-321` are C2's *fact*; C3 owns what they **fail to do first**. If a draft explains what parking is, cut it |
| `Upvalue::Open { fiber, slot }`, the fiber-aware `GetUpvalue`/`SetUpvalue` read branch | **[`vm/upvalues.md`](../vm/upvalues.md)** (shipped) | the crash lands *in* that branch (`dispatch.rs:1062`); show the panic and link. Do not re-teach the read path |
| `FrameToken`, generation counters, `DeadFrameError` mechanics | **[`vm/frame-identity.md`](../vm/frame-identity.md)** (shipped) | a cross-fiber return token failing its generation check is one line, cited |
| `System.schedule(_)`, `ready_queue`, the root-drive pump, `Future`, `async`/`await` | **C4** | the pump lives in the same `Ok` arm C3 quotes — one line, "C4's" |
| GC root enumeration as a subject, `collect_roots`, the mark-sweep design | **unowned; ADR-0050** | E001's fix is `push_temp_root` and that is all this doc needs. No collector tour |

## 6. Open risks

| Assumption | If wrong | Disposition |
|---|---|---|
| The failure path calls no upvalue-closing on any route — not for the failing fiber, not for cascaded resumers | the doc's thesis is a partial truth and F5 collapses | **B question 2, REFUTE** |
| E001 is genuinely fixed (not merely unreproducible in the one shape tested), so the pair is "one paid, one open" | F2's framing inverts and the doc claims a fix that is not one | **B question 3, REFUTE** |
| The cascade's skipped-cleanup behaviour is unreachable at HEAD because every cleanup construct is a native frame | the doc understates a live bug — the opposite error from C2's | **B question 4, REFUTE** |
| `Fiber.abort(_)` and `throw` reach the same floor by the same route | the doc conflates two paths | **B question 5** |
| No live tracing (`vm-trace` is `LevelFilter::OFF`; `disasm` walks only the top-level chunk) | every dynamic claim must be an observed program or labelled INFERRED | standing — run programs, do not trace |

## 7. Doc-kind gate (phase 2)

**Kind: tension. Agent A runs.** ([`AUTHORING-LEAN.md` §2](../AUTHORING-LEAN.md#phase-2--the-doc-kind-gate-one-line-decides-phase-3a); the plan also called it a tension, but this is answered from §1 and F5.)

Two *committed, shipped* features collide, which is the definition:

- **ADR-0008's unwind is terminating and ordered.** It walks frames down and closes upvalue cells
  before reclaiming slots, and its own rustdoc says why.
- **ADR-0030 §6's containment forbids that walk from continuing past the fiber floor.**

They meet at exactly one `Err` arm, and the shipped resolution — stop the walk by *discarding* the
frames rather than by *finishing* them — is what produces E002. Neither feature is wrong; the
collision is real; there is no fork with occupants on both branches. Tension.

A is briefed **blind and redacted** (no `recon.md`, no findings, no branch names, no E00x): its job
is the design space for *how an uncaught failure should leave a coroutine*, so that §4.1's
reconciliation has something honest to check the code against — and, per §4.4, so its confident
wrong answers can be mined for the predict-then-check.
