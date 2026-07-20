# Subtractive decorators — `@native` and `@ignore`

- Status: **Built (2026-07-16)**, sharing one legality-check-then-drop pass in
  `expand_class_attributes`. As-built truth:
  [native.md](../v0.2/decorators/native.md),
  [ignore.md](../v0.2/decorators/ignore.md). The pair is the only subtractive
  Compile-tier machinery and, by ignore.md's own preclusion, the only drop
  mechanism the language gets.
- Division of meaning (verified, keep): `@ignore` = "the compiler ignores
  this" — sanctioned, permanent, no external claim. `@native` = "the real
  implementation is a Rust primitive" — an *anchor* that additionally asserts
  a native binding exists, checkable by machine. `@native` is **not**
  protection (reopening was verified to hijack native bindings before
  PDR-0001; post-PDR-0001 kernel classes are closed, but that closure is
  PDR-0001's doing, not the decorator's) and **not** a binding directive
  (bootstrap installs bindings regardless).

## Open-question dispositions (N-1…N-5, I-1…I-4)

| # | Question | Disposition here |
|---|---|---|
| N-1 | missing-binding = compile error? | Settled correctly: impossible (compiler can't see the bootstrap binding table); the invariant test is the only checkable place. No change. |
| N-2 | how does LSP find a dropped anchor? | **The provisional core. Resolution proposed below.** |
| N-3 | check anchor's `SignatureKind` against the native's? | Adopt into the invariant test the day it lands (cheap; catches `@native toString()` vs getter `toString` — a real CB-1-class mistake). |
| N-4 / I-3 | class-level `@native class` / `@ignore class`? | Keep out of scope. The invariant-test subject ambiguity N-4 names is real, and a whole-class drop hides too much. |
| N-5 / I-4 | `@native` + `@ignore` stacked? | **Rule now: error `attr.redundant`.** They assert contradictory things (a native exists / this is not code); stacking is confusion, and ruling it costs one driver check. PDR-trivial. |
| I-1 | should `@ignore` warn? | Deferred with its stated trigger: no warning tier exists. Reconsider when one does ([compiler-directives.md](compiler-directives.md) owns the warn-tier question). |
| I-2 | `@ignore` on Field? | Keep illegal — dropping a field changes layout (ADR-0011); materially different act. |

## Proposed resolution for N-2 (PDR candidate): the pre-drop harvest

The anchor exists *for tooling*, and the compiler discards it before tooling
can see it. The two candidate shapes were "LSP indexes the AST before the
drop" or "the drop records spans somewhere." Recommendation: **both are the
same fix if the harvest happens in the driver** — at the legality-check step
(which already iterates every marked member), record
`(class, selector, kind, span, attr)` into a side table on the compiled
module *before* the retain. Consumers:

1. **LSP go-to-definition**: `phalcom-lsp` reads the harvest table instead of
   the (absent) method — the anchor lands the jump. This is exactly the
   LSP-only role the `@native` intent memory records, now with a mechanism.
2. **The invariant test (DEF-10)**: `anchors ⊆ installed native bindings`
   reads the same table — one producer, two consumers, no drift between what
   LSP sees and what the test checks. N-3's kind check bolts on here.
3. Nothing survives to the VM: the table is compiler/LSP-side data, not
   retained runtime metadata — `@native` stays `runtime: false` and costs the
   image nothing.

Sequencing (native.md already fixed the exemplar): land `List#toString` as the
first anchor **together with** the harvest table and the invariant test in one
unit — an anchor without the test is a drift generator with a friendly face,
and the test without an anchor is vacuous. `Object`/`Number`/`Symbol`
anchors follow mechanically. The known-unmitigated hazards stand: silent
go-live on attribute deletion, body drift (only *existence* is checked) —
restated, not solved; nothing cheap solves them.

## Hazards

- The pair's legality-check-then-drop ordering is load-bearing (contracts on
  a dropped member must not weave; derives must not generate from a corpse).
  Any refactor of `expand_class_attributes` must keep the drop first — the
  as-built files say why in detail.
- Post-PDR-0001, a stray `@native` on a *user* class asserts a binding that
  can never exist; the invariant test catches it only if the test scans user
  corpora too. Scope the test to `core.ph` explicitly and reject `@native`
  outside the core module at compile time (`attr.native_outside_core`,
  Proposed) — a user has no legitimate use for it.

## What this precludes

Unchanged from the as-built files: no binding-directive reading, no
protection reading, no third drop mechanism, no conditional compilation via
`@ignore`.
