# Handoff — implement U-LIST then U8, sequentially, no subagents

You are a **single implementer agent**. You write the code yourself, in order, in this tree — no
delegation, no worktree isolation. Work directly on `main` (no worktree; not pushed).

## Source of truth
**[U-LIST-plan.md](U-LIST-plan.md)**, then **[U8-plan.md](U8-plan.md)** — read each in full immediately
before starting it. Both are authoritative; don't re-derive or re-litigate their design decisions. This
handoff only sets ground rules, ordering, and stop conditions.

## Before you start anything — two hard preconditions
1. **U7 must be landed and green.** Confirm via `graphify affected "InstanceObject"` /
   `graphify explain "ClassObject"` that `field_slots`/`static_slots` exist, and `./scripts/verify.sh` is
   green on current HEAD. `Message` (part of U8, not U-LIST) needs U7's slot layout. If U7 isn't landed,
   **STOP** — do not start.
2. **ADR-0019 and ADR-0020 must both be `Status: Accepted`** (check `docs/adr/README.md`). They currently
   read `Proposed`. **If either is still Proposed, STOP before writing any U-LIST code** — do not infer or
   improvise the `List` storage design; that ambiguity is exactly what those ADRs exist to close. Report
   back and wait for ratification rather than guessing.

Both gates clear → proceed in order below. Either gate unclear → stop and report, don't proceed partway.

## Order: U-LIST first, then U8 — do not interleave
U8 has a hard dependency on U-LIST (`Message.args`/`labels`, `perform(_:List)`). Land U-LIST completely,
green, committed, before opening U8's write-set. They share `core.ph` — **never edit it for both units in
the same uncommitted change.**

## Ground rules (apply to both units)
1. **graphify-first.** For "where is X / what calls it / did a line number drift" — run `graphify query`,
   `graphify explain`, `graphify affected` before opening a source file to orient yourself. **Re-locate
   every cited line number against actual HEAD before editing** — both plans were written before U7
   landed; U7's edits will have moved things (especially in `core.ph`, `vm.rs`, `compiler/lib.rs`).
2. **Follow each plan's build order in sequence**, step by step. Don't skip or reorder within a unit.
3. **Verify after every step, not just at the end.** Run `./scripts/verify.sh`, `cargo doc --workspace
   --no-deps`, `cargo clippy --workspace`. Fix red before moving to the next step.
4. **Commit at each green checkpoint.** Small commits per landed step, not one end-of-unit batch. Never
   commit a non-compiling tree. Run `graphify update . --no-cluster` before each commit.
5. **Reviewer is OFF for both U-LIST and U8** (STATE.md policy) — the green gate + clean `cargo doc` is the
   sole sign-off.
6. **Stay in each plan's §4 write-set.** If you find you must edit outside it, stop and reconcile via
   graphify before continuing.
7. **Docs are mandatory:** `//!` on every touched/new module, `///` on every new public item, with
   ADR citations. `cargo doc` must add zero new warnings.

## Unit-specific notes
- **U-LIST:** `List` is a native heap variant (`Object::List`), **not** built on U7's `InstanceObject`
  slots — see U-LIST-plan §2 for why. Absence at the `at(_:)` boundary must surface `None` via U6's
  existing helper — never a panic, never the raw `Value::Nil` sentinel.
- **U8:** re-locate the miss arm in `vm.rs` (plan cites `~L698-708`, drifted from U5/U6/U7 edits — find it
  fresh via `graphify affected "call_method"`). Leave the **U9 variadic seam** exactly as U8-plan §3
  specifies (a comment + ordering hook immediately before the dNU forward) — quote it in your return
  report so U9 needs no rewrite. `Message` is an ordinary `InstanceObject` using U7's `construct` —
  confirm U7's mechanism works as expected before building on it.

## Stop condition — hard boundary, do not cross
**When U8 is green (through its full build order), STOP.** Do not begin U9, U10, U11, U-LEX, or U-STD.
Update [STATE.md](STATE.md) and [PHASE2-INDEX.md](PHASE2-INDEX.md) to mark U-LIST and U8 landed, answer
each plan's §return-contract in full (U-LIST-plan §8, U8-plan §9), record any new `DEFERRED.md` entries,
and report back.

## Done means
`./scripts/verify.sh` exits 0, `cargo doc` + clippy clean, both units' goldens/negatives added, both
return contracts answered, STATE.md/PHASE2-INDEX.md updated, U-LIST landed strictly before U8 opened,
stopped at the U9/U10/U11/U-LEX/U-STD boundary as above.
