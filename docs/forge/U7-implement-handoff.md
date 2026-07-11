# Handoff — implement U7 directly, sequentially, no subagents

You are a **single implementer agent**. You write the code yourself, in order, in this tree — no
delegation, no worktree isolation. Work directly on `main` (no worktree; not pushed).

## Source of truth
**[U7-plan.md](U7-plan.md)** — read it in full before starting. It is authoritative: don't re-derive or
re-litigate its design decisions. This handoff only sets ground rules and the stop condition.

## Ground rules
1. **graphify-first.** For "where is X / what calls it / did a line number drift" — run `graphify query`,
   `graphify explain`, `graphify affected` before opening a source file to orient yourself. U5/U6 landed
   since the plan was written, so **re-locate every cited line number before editing** (the plan expects
   this; it's not a sign something is wrong).
2. **Follow §5's build order, steps 1 → 8, in order.** Step 8 (class-side stored static fields, DEC-D) is
   **cleared to proceed** — its prerequisite, [ADR-0017](../adr/0017-class-side-stored-static-fields.md),
   is `Status: Accepted`. Do not skip or reorder steps; each depends on the slot/field-table work the
   previous one lands.
3. **Verify after every step, not just at the end.** Run `./scripts/verify.sh`, `cargo doc --workspace
   --no-deps`, `cargo clippy --workspace`. Fix red before moving to the next step.
4. **Commit at each green checkpoint.** Small commits per landed step, not one end-of-unit batch. Never
   commit a non-compiling tree. Run `graphify update . --no-cluster` before each commit.
5. **Reviewer is OFF for U7** (STATE.md policy) — the green gate + clean `cargo doc` is the sole sign-off.
   No separate review pass needed.
6. **Stay in §4's write-set.** If you find you must edit outside it, stop and reconcile via graphify
   before continuing — don't let scope creep in silently.
7. **Docs are mandatory**, not optional cleanup: `//!` on every touched module, `///` on every new public
   item, with ADR-0011/ADR-0017 citations per §6. `cargo doc` must add zero new warnings.

## Stop condition — hard boundary, do not cross
**When U7 (through step 8) is green, STOP.** Do not begin U8 or U-LIST. Reasons:
- **U-LIST has no detailed build-order plan.** Only a scope paragraph exists (`U8-plan.md` §3) — no
  numbered steps, no per-step write-set, no test section like U7/U8 have. There is nothing to execute yet.
- **U-LIST and U8 are both gated on ADR ratification.** [ADR-0019](../adr/0019-freeze-vm-blessed-primitive-floor.md)
  (freeze the VM-blessed primitive floor) and [ADR-0020](../adr/0020-kernel-list-native-array-protocol.md)
  (kernel `List` design) are `Status: Proposed`, not Accepted. Do not infer or improvise a `List`
  implementation to unblock yourself — that is exactly the ambiguity those ADRs exist to close.

On reaching this boundary: update [STATE.md](STATE.md) and [PHASE2-INDEX.md](PHASE2-INDEX.md) to mark U7
landed (commit, offsets, `construct`, DEC-D static fields), answer U7-plan §7's return contract in full,
record any new `DEFERRED.md` entries, and report back — don't proceed further on your own initiative.

## Done means (for this handoff)
`./scripts/verify.sh` exits 0, `cargo doc` + clippy clean, all §7 goldens/negatives + the layout/offset
invariants added, U7-plan §7 return contract answered, STATE.md/PHASE2-INDEX.md updated, stopped at the
U8/U-LIST boundary as above.
