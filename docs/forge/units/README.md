# `docs/forge/units/` — per-unit record

One folder per implementation unit. A landed unit holds `as-built.md` (moved here from
`docs/spec/v0.2/units/`, now retired); an in-flight unit holds its `plan.md`/`handoff.md`.

## U-CORE track — the core library, built in Phalcom over the frozen primitive floor

| Unit | Spec | Status |
|---|---|---|
| U-CORE-1 | [as-built](U-CORE-1/as-built.md) | ✅ landed (`03764e3`/`b1109c2`) — `hash`, `isA`, `Behavior` reflection, `Method < Function` |
| U-CORE-2 | [as-built](U-CORE-2/as-built.md) | mostly landed (`0da64d6`); verify/harden |
| U-CORE-3 | [as-built spec](U-CORE-3/as-built.md) · [handoff](U-CORE-3/handoff.md) | dispatch-ready — **next** |
| U-CORE-4 | [as-built](U-CORE-4/as-built.md) | dispatch-ready |
| U-CORE-5 | [as-built](U-CORE-5/as-built.md) | dispatch-ready |
| U-CORE-6 | [as-built](U-CORE-6/as-built.md) | dispatch-ready |

## Spine track — the landed language core (forge roster, closed)

| Unit | Spec | Realizes |
|---|---|---|
| U0 | [as-built](U0/as-built.md) | verification substrate (`verify.sh`, golden corpus, invariants) |
| U1 | [as-built](U1/as-built.md) | ADR-0009 handle/arena heap · ADR-0010 tagged `Value` |
| U2 | [as-built](U2/as-built.md) | ADR-0002 parallel rule · ADR-0003 `Behavior` · `verify_invariants` |
| U3 | [as-built](U3/as-built.md) | ADR-0012 label-encoded selectors · IC-ready dispatch |
| U4 | [as-built](U4/as-built.md) | ADR-0013 upvalues/frame tokens · ADR-0006 `Function` root |
| U5 | [as-built](U5/as-built.md) | ADR-0018 sacred-selector inliner + deopt guard |
| U6 | [as-built](U6/as-built.md) | ADR-0007 Option · ADR-0014 let/var · ADR-0021 no-truthiness |
| U7 | [as-built](U7/as-built.md) | ADR-0011 slot layout · `construct` · ADR-0017 static fields |
| U8 | [as-built](U8/as-built.md) | ADR-0012 `doesNotUnderstand`/`perform` · `Message` |
| U9 | [as-built](U9/as-built.md) | ADR-0012amd rest params `*xs` · `(*)` selector |
| U10 | [as-built](U10/as-built.md) | ADR-0013 `ReturnNonLocal` + frame-token unwind |
| U11 | [as-built spec](U11/as-built.md) · [U-CORE bridge](U11-UCORE/handoff.md) | ADR-0004 abstract `Bool` + `True`/`False` |
| U-FE | [as-built](U-FE/as-built.md) | ADR-0016 hand-written lexer + recursive-descent parser |
| U-LEX | [as-built](U-LEX/as-built.md) | block comments, digit separators, ADR-0022 `\(expr)` |
| U-LIST | [as-built](U-LIST/as-built.md) | ADR-0019/0020 native-array `List` |
| U-STD | [as-built](U-STD/as-built.md) | pure-`.ph` Option/List combinators |

## In-flight planning batch

`U12`–`U20`, `U-COLL` — per-unit `plan.md` (+ `cluster-summary.md` for U12), not yet dispatched.

## Convention
- `<unit>/as-built.md` — factual record of what landed: mission / surface / implementation /
  invariants & tests / deviations / sources (landing commits). Cites the ADR(s)/spec section it
  realizes.
- `<unit>/plan.md` — pre-implementation work order, for units not yet built.
- `<unit>/handoff.md` — session-resume note for a specific unit, when one exists.
- `<unit>/IMPL-SPEC-*.md` — corrective/prescriptive companion spec for remaining steps of a plan
  already in progress, when re-grounding against HEAD found the original plan's framing wrong for
  part of the work (e.g. `U-GC/IMPL-SPEC-steps-3-5.md`). Supersedes the named section(s) of
  `plan.md`; does not replace it.

## Performance / GC track (`§11` of `UNITS-TRACKER.md`)

| Unit | Spec | Status |
|---|---|---|
| U-BENCH | [plan](U-BENCH/plan.md) | landed — Tier 0 Wren-suite reference programs; gates the rest of this track |
| U-GC | [plan](U-GC/plan.md) · [impl spec steps 3–5](U-GC/IMPL-SPEC-steps-3-5.md) | steps 0–4 done, uncommitted; step 5 (fiber-stack pool re-measure) open |
| U-PRIM-ABI | [plan](U-PRIM-ABI/plan.md) | not started |
| U-IC | [plan](U-IC/plan.md) | not started |
| U-HOTPATH | [plan](U-HOTPATH/plan.md) | not started |
| U-COMPILE | [plan](U-COMPILE/plan.md) | not started |
