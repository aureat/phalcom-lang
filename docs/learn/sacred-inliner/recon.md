# Inliner doc — recon

Phase 1 of [AUTHORING-LEAN](../AUTHORING-LEAN.md), written before drafting. Every claim is a cited
line, a quoted record, or output of a program run at HEAD.

---

## 1. Architecture vs representation

The sacred-selector inliner is a **compile-time recognizer plus a dual-emission scheme**, not a
runtime optimizer. `recognize` (`compiler/inliner.rs:138`) matches purely on AST *shape* — selector
name, arity, and whether each block argument is a literal `Expr::Block` — and returns either a
`SacredCall` or the original call unchanged. Zero runtime cost, zero type information, no profiling,
no tiering.

The representational fact that decides the whole document: **both the fast path and the fallback are
emitted into the same chunk, adjacent, at compile time.** The guard is a *forward jump* whose target
is the fallback (`emit_forward_jump(Bytecode::GuardBool, …)`, `inliner.rs:298`). Nothing is patched,
recompiled, or reconstructed at runtime.

Architecturally this is "deoptimization." Representationally there is no deopt machinery at all —
no OSR, no frame reconstruction, no code cache, no invalidation sweep. Per [x-style ≠
representation], do not import the mainstream meaning of the word: the slow path is ten instructions
away and was always there.

## 2. The grip (grounded)

**Phalcom's deoptimization has no mechanism, because both programs are already in the bytecode.**
The compiler emits the inlined fast path and the real-send fallback side by side, and "deopt" is a
jump taken. That buys a genuinely free runtime — one predictable branch — and the bill lands
somewhere nobody was watching: **every inlined block body is compiled twice**, and for one commit
range each nesting level doubled it again.

Cites: `inliner.rs:295-318` (both paths in one method), `dispatch.rs:1190-1201` (the guards),
the 17-instruction disassembly in §4 F1, and SCOREBOARD §3d for the bill.

## 3. Deliberated vs reconstructed

**Actually deliberated, in ADR-0018:**

- The fork itself. `## Alternatives considered` rejects **grammar-level control flow** ("compile
  `if`/`while` straight to jumps, no selector") as making control flow non-overridable and splitting
  "looks like a send, isn't a send." Also rejects **no guard / forbid overriding** as making
  `Bool`/`Block` second-class, and **defers** per-selector invalidation as bookkeeping for a case that
  never happens. Three branches, each with a stated reason — this is a real fork, not a
  reconstruction.
- The coarse-flag trade. `## Consequences` states it outright: one flag per class *family*, a
  redefinition of any sacred `Bool` method conservatively dirties all of them.
- Non-local return through an inlined body (ADR-0013), called out as unchanged by inlining.
- The `want_value`/`WrapSome` dual path and the whole one-armed-`Option` surface, argued at length in
  `inliner.rs`'s module doc §"v0.2 conditional-surface decision" with five rejected alternatives.

**My reconstruction, not in any record:**

- That dual-emission was *chosen* over recompilation. The ADR never names recompile-on-invalidate as
  an option — it goes straight to guards. Whether anyone weighed the alternative is not recorded;
  I will label this as the shape the code implies, not a decision anyone wrote down.
- That the compile-time cost was unanticipated. F13's existence is evidence, not proof of intent.

## 4. Findings that change the doc

**F1 — the artifact.** A two-line source file disassembles to 17 instructions containing *both*
programs. `if (c) { print A } else { print B }`:

```
0003: GuardBool(9)      ; not a pristine Bool -> jump to 0013
0004: JumpIfFalse(4)
0005-0007: then arm, spliced inline
0008: Jump(7)
0009-0011: else arm, spliced inline
0012: Jump(3)
0013: Closure(8)        ; FALLBACK: materialize the then block
0014: Closure(9)        ;           materialize the else block
0015: Invoke(2, 10)     ;           real ifTrue(_, ifFalse:) send
0016: Return
```

Each arm exists twice — spliced at 0005/0009, and as closure constants [8]/[9]. This single listing
carries the grip, and it is the doc's centrepiece. Note this is a case where `disasm`'s
top-level-chunk-only limitation does not bite: the inlined body *is* in the top-level chunk. That is
the point.

**F2 — the compile-time scar, measured and landed.** The duplication was exponential in nest depth
until `0274f10` ("perf(compiler): stop inlining inside deopt-fallback copies (F13)"). The fallback
copy was itself run through the inliner, so each level doubled. SCOREBOARD §3d:

| nest depth | 16 | 18 | 20 | 26 |
|---|---|---|---|---|
| before `0274f10` | 0.17 s | 0.70 s | **2.8 s** | **70.9 s** |
| after | — | — | — | **0.022 s** |

Plus: bootstrap regressed **35×** (~5 ms → 180 ms) across `3b2dd97`…`0274f10`, and `--test lang`
went **122 s → 2.8 s**. SCOREBOARD's own note on the bootstrap row: *"passed every gate — nothing
measured bootstrap."*

**Verified independently at HEAD.** Depth 1→26, top-level instruction count and compile wall-clock:

| depth | 1 | 2 | 4 | 8 | 12 | 16 | 20 | 26 |
|---|---|---|---|---|---|---|---|---|
| instructions | 15 | 24 | 42 | 78 | 114 | 150 | 186 | 240 |
| compile s | 0.023 | 0.023 | 0.022 | 0.023 | 0.023 | 0.024 | 0.025 | 0.026 |

Exactly linear — 9 instructions per level, which is precisely the guard + fallback sequence. My 0.026 s
at depth 26 matches SCOREBOARD's post-fix 0.022 s; the pre-fix 70.9 s is a ~2700× difference.

**F3 — the suppression is a soundness argument that happens to bound code size.** `in_deopt_fallback`
(`compiler/lib/mod.rs:187`) is read at exactly one place, `compiler/lib/expr.rs:56`, where recognition
is skipped inside a fallback copy. The comment there gives the perf reason. The *soundness* reason —
a fallback exists because a sacred selector was overridden, so inlining within it would re-assume the
thing that just proved false — is not stated anywhere. Worth checking with B whether that is even
true, or whether suppression is purely a size fix.

**F4 — ADR-vs-HEAD gap in the ADR's own Decision.** It says "**`GuardBool`/`GuardBlock`** verify the
receiver's type before the inline body runs." `GuardBlock` does **not** peek a receiver —
`bytecode.rs:250` says so explicitly and `dispatch.rs:1197-1200` tests only the pristine flag. The
receiver of an inlined `whileTrue` is a compiler-materialized literal, so its type is static. The ADR
overstates a symmetry the implementation does not have.

**F5 — `GuardBool` is two questions in one instruction.**
`matches!(top, Value::Bool(_)) && self.universe.bool_sacred_pristine` (`dispatch.rs:1192`) — a
*type* check and a *global override* check, sharing one branch and one deopt target. Two completely
different reasons to fall back, indistinguishable at runtime.

**F6 — `whileTrue`'s guard runs once, not per iteration** (`inliner.rs:418-426`), because the
receiver is statically a block; the *condition* is type-checked every iteration by `JumpIfFalse`
itself, "which is what gives `while`'s 'no truthiness' floor without a second guard opcode." A second
mechanism doing guard duty without being called a guard.

**F7 — recognition is syntactic and therefore fragile-by-design.**
`let b = { … }; c.ifTrue(b)` is not recognized (`inliner.rs:126-132`) and compiles to an ordinary
send — "correct, just not fast." A one-token source edit silently changes the emitted shape.

## 5. Forbidden list

| Material | Owner |
|---|---|
| The five `pristine` flags, `note_method_installed`, epoch-vs-flag correction, the `world_version` IC contrast | [Doc 5 `caches-and-fusion.md`](../vm/caches-and-fusion.md) — it already made the "no epoch, five latching bools" correction; do **not** re-derive |
| "The two `ifTrue` blocks are not blocks at runtime" trace, frame tokens, frame identity | [Doc 6 `frame-identity.md`](../vm/frame-identity.md) |
| Closure representation, upvalue capture | [`upvalues.md`](../vm/upvalues.md) |
| Inline caches, `InvokeConst`, opcode fusion | Doc 5 |
| Non-local return machinery | Doc 3 `frames.md` |

This doc owns: `recognize`, `compile_sacred_call` and the per-selector emitters, the dual-emission
scheme itself, the guard opcodes' runtime semantics and their asymmetry, and the compile-time bill.

## 6. Open risks

| Risk | Disposition |
|---|---|
| Is the exponential really gone, or did F13 just move it? | Measured at HEAD myself: linear, 9 instrs/level. Confirmed. |
| Is F3's soundness argument real, or did I invent it? | Explicit REFUTE ask to B. If suppression is purely a size fix, say so and drop the soundness framing. |
| Does an override actually deopt correctly end-to-end? | Run it. ADR names two tests (`control_flow_inline_override_honored`); verify by program, not by test name. |
| Are the fast and fallback paths really observationally identical? | `inliner.rs:22-26` claims it. Probe the seams: `want_value`/`WrapSome` elision, and stack depth. Ask B. |
| Doc 5 may already own more than I think | Read its §forward-pointers directly — done; it hands off exactly `compile_sacred_call` + "its override-epoch deopt". Territory confirmed. |

## 7. Doc-kind gate

**Fork.** ADR-0018 has a real `## Alternatives considered` with three branches and stated reasons,
and the rejected one — grammar-level control flow — is the branch nearly every other language takes.
The doc's spine is: *why is `if` a message send here, what does keeping it one cost, and who pays.*
Per AUTHORING-LEAN §3, **fork ⇒ Agent A runs.**

---

## 8. Reconciliation (phase 4.1) — added after the agents returned

| Claim from recon / Agent A | What the tree says |
|---|---|
| **Recon F3:** fallback suppression is a *soundness* requirement | **REFUTED — size-only.** `0274f10`'s message: *"the inliner is a guarded optimization over the `bool_if_true`/`bool_and` primitives, not a semantic."* A nested guard inside a fallback reads the same global flag and answers correctly; nesting would be sound, just 2^depth. My reasoning was that a fallback runs *because* an override exists, so inlining inside re-assumes the refuted thing — wrong, because the guard's information is global and position-independent. Corrected in the doc as an explicit "I got this wrong" note. |
| **Agent A, ranked #1 bug:** "non-local return diverges silently between fast and slow paths… the fast path is silently narrower semantics wearing identical syntax" | **CONFIRMED, and worse than predicted.** Reproduced: inlined `ifTrue { return "A" }` → `A`; non-inlined → `Some(A)`. A assumed an override was needed to expose it; a `let` is enough, so it is reachable in ordinary code today. Filed [E005](../../errors/E005-nonlocal-return-some-wrapped.md). Second time the A-brief's "name the bug you'd expect" question has produced a real, previously-unrecorded defect (C3 → E002). |
| **Agent A, ranked #2 bug:** bootstrap self-poisoning of the epoch by the prelude's own installs | **Partially confirmed — latent, not active.** The leaf `toString` flags have exactly this problem and solve it with seed-`false`-then-snapshot (`bootstrap.rs:134-140`), *because* `core.ph` installs `String>>toString`. The sacred flags seed `true` with **no** re-snapshot, so the landmine is armed; `core.ph:421-431` deliberately does not step on it. |
| **Agent A, ranked #3:** tag-space collision on the boolean identity check | Not applicable — `matches!(top, Value::Bool(_))` is an enum discriminant match, not a bit-pattern test. Cut. |
| **Recon assumption:** `bootstrap.rs:134`'s snapshot is part of the sacred-flag mechanism | **Wrong.** It is a *different* mechanism — the leaf `toString` fast path (`number_/symbol_/str_tostring_pristine`), not `GuardBool`/`GuardBlock`. Doc says so explicitly rather than eliding it. |
| **Recon F7:** losing recognition is "correct, just not fast" (quoting the source) | **Wrong twice.** It also changes the answer (E005) and, for `whileTrue`, is a hard compile error — `push_loop_context` is only called from `compile_while_true`, so `break` in a non-recognized loop fails with *"`break` outside of a loop."* Three failure modes from one cliff; became a doc section. |
| **Open risk:** does an override deopt correctly end-to-end? | **Unanswerable from surface Phalcom.** `class Bool { … }` is rejected — `class.reserved_name` — since decision 0065 closed kernel classes. The five golden fixtures were rewritten in-crate to hand-install via `install_kernel_method`. Became the doc's "the guard outlives its threat" section. |
| **Open risk:** is the runtime win measured? | **No measurement exists** anywhere in `perf-log/`. Every inliner number is compile time. Stated as an honesty note rather than estimated. |

Both agents earned their slots: A supplied the doc's sharpest finding before seeing any code, B
supplied the repro and refuted my soundness hypothesis. The fork gate was right.

[x-style ≠ representation]: ../../../CLAUDE.md
