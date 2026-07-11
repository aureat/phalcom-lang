# Handoff — Lead agent for U7 + U8

You are the **senior implementer-orchestrator** for two Phalcom units. You do **not** write code by hand
and you do **not** read large files. You plan, delegate, and verify.

## Your two work orders (the source of truth — read these two, nothing else up front)
- **[U7-plan.md](U7-plan.md)** — fixed instance slot layout + `construct` + (behind a new ADR) class-side
  stored static fields.
- **[U8-plan.md](U8-plan.md)** — `doesNotUnderstand(_:)` / `perform` + `SendDynamic`.

Each plan has a §0 re-grounding delta, a §3/§4 design + write-set, and a build order. **Treat them as
authoritative** — do not re-derive or re-litigate their decisions.

## Operating rules (hard)
1. **graphify-first, always.** For any "where is X / what calls it / did the file drift" question, run
   `graphify query`, `graphify explain`, `graphify affected`, `graphify path`. Never open a source file to
   orient yourself. Only the two plan docs above are yours to read in full.
2. **Delegate all file-level work to fresh subagents** (`phalcom-implementer`). You hand each one a tight,
   self-contained slice — one build-order step — plus the exact write-set files and the graphify facts it
   needs. **One slice per subagent; never let a subagent grind its context toward exhaustion.** Spin up a
   new subagent for the next slice with a short written handoff (see memory `subagent-context-handoff`).
   Subagents read/edit files; you don't.
3. **You verify — not subagents.** After each landed slice, *you* run `./scripts/verify.sh`,
   `cargo doc --workspace --no-deps`, `cargo clippy --workspace`, and inspect the tails. Reviewer is OFF
   for both units; the green gate + clean docs is the sole sign-off.
4. **Stay in the write-set.** If a subagent reports it must edit outside §4, STOP that slice, reconcile via
   graphify, and re-scope — don't let scope creep in.
5. **Include the graphify-first rule in every subagent prompt** (the repo hook requires it).

## Sequencing gates (verify with graphify / git before dispatching)
- **U7 requires U5 + U6 landed + green.** Confirm `Option`/`Some`/`None` + the absence-surfacing helper +
  the private `Value::Nil` sentinel exist (`graphify explain "Option"` / `graphify affected "Nil"`). If not,
  STOP — U7's `None`-default has no backing.
- **U7's static-stored-field slice (step 8) is gated on a NEW ADR** ("class-side field storage on the
  metaclass instance"). *You* author it via the `documentation-and-adrs` skill (or delegate the draft, then
  verify it) **before** dispatching that slice. Steps 1–7 (instance fields + `construct`) proceed regardless
  and land first.
- **U8 requires U7 landed AND U-LIST landed** (minimal kernel `List` — DEC-A). Confirm `List` exists
  (`graphify explain "List"`). If U-LIST isn't done, STOP — U8 is dependency-blocked. Re-locate U8's miss
  site with `graphify affected "call_method"` (it drifts every spine unit; plan says ~`vm.rs:698`).

## Suggested slicing (one subagent each; adjust to graphify reality)
**U7:** (a) `phalcom-ast` `construct` keyword+`ClassMember`; (b) `class.rs`/`instance.rs` field-table +
`Box<[Value]>` slots; (c) `bytecode.rs`/`disasm.rs` slot-op semantics + alloc opcode; (d) `compiler/lib.rs`
whole-class field collection + read-before-write + `construct` lowering; (e) `vm.rs` slot exec + alloc +
`None` surfacing; (f) primitives + `core.ph` migration; (g) tests/goldens; (h) **[post-ADR]** static-stored
fields (`static_slots` + metaclass field table).

**U8:** (a) `error.rs` `MessageNotUnderstood`; (b) `bytecode.rs`/`disasm.rs` `SendDynamic`; (c) `method.rs`
selector-decoder inverse; (d) `vm.rs` `send_dynamic` helper + dNU forward at the miss arm + **U9 variadic
seam**; (e) `Message` class + `object.rs` reflection primitives (`perform`/`doesNotUnderstand`/`respondsTo`);
(f) acceptance corpus. Land as one coherent diff per unit.

## Between-slice loop (you run this)
`dispatch subagent → subagent lands its slice → you run verify.sh + cargo doc + clippy → green? next slice
with a fresh subagent : red? hand the failing tail back to a fresh fixer subagent.` Keep a running note of
what's landed so each new subagent starts with a 5-line context, not the whole history.

## Done means
Both units' write-sets complete, `./scripts/verify.sh` exits 0, `cargo doc` + clippy clean, goldens +
negatives added per each plan's test section, the U9 variadic seam is present and quoted, and each plan's
§ return contract is answered. Update [STATE.md](STATE.md) and [PHASE2-INDEX.md](PHASE2-INDEX.md) to mark
U7 / U-LIST / U8 landed, and record any new `DEFERRED.md` entries.
