---
id: LANG007.C4.P11
kind: implementation-handoff
title: "P11 Char scalar and literals — interim handoff"
status: INTERIM
completion: PARTIAL
verification: UNVERIFIED
recorded_against_revision: 800e179325a4fa0d97e3cd1efa636d09d6fab393
recorded: 2026-10-07
updated: 2026-10-07
branch: codex/lang007-c4-p4-r1-stdlib
---

# LANG007.C4.P11 — Interim Handoff

P11 implementation is present on the pushed branch at `800e1793`. This is an
interim handoff for audit and verification; P11 remains
`IN_PROGRESS / PARTIAL / UNVERIFIED`.

## Read first

1. [C4 checkpoint](LANG007.C4-CHECKPOINT.md) for shared checkpoint authority
   and P11's place as a parallel follow-on while P4 remains the primary plan.
2. [P11 implementation plan](LANG007.C4.P11-char-and-literals-plan.md) for the
   fixed architecture and focused acceptance gates.
3. [Interim walkthrough](LANG007.C4.P11-walkthrough.md) for what the current
   commit contains and exactly what evidence is absent.
4. [INC-6 incident](LANG007.C4.P11-pattern-coverage-incident.md) for the
   escalation, user decision, and historical observations.
5. [Normative Char specification](../../../specs/objects/characters.md) for
   scalar/literal rules and the selected closed matching domain.

## Established contract

- Char represents one valid Unicode scalar, distinct from Int and String, in
  the existing immediate value representation.
- The canonical Char declaration and semantic identity come from the Universe;
  native metadata enriches that identity.
- For matching, Char is a closed scalar domain and is non-subclassable. A
  literal pattern matches its exact scalar and contributes singleton coverage.
  Exhaustiveness over literals requires coverage of every valid scalar or a
  catch-all; surrogate code points are excluded.
- Semantic coverage owns the proof. Executable lowering consumes the resolved
  product and compares the scalar directly. Unknown subject types remain
  blocked rather than being treated as impossible.
- The above pattern contract was selected by the user on 2026-10-07 and is
  recorded in INC-6 and the normative specification. Do not reopen it absent
  new evidence that forces a different architecture or accepted semantic rule.

## Current state and audit targets

The implementation commit adds lexer/parser syntax, semantic and static Char
support, immediate runtime representation, Universe/native integration,
generated surface records, semantic scalar coverage, match products, and exact
runtime lowering. AST lexer/parser test cases are present. No tests or crate
checks have been run, and no semantic or core regression test files were added
in the implementation commit.

For the remote audit, use branch
[`codex/lang007-c4-p4-r1-stdlib`](https://github.com/aureat/phalcom-lang/tree/codex/lang007-c4-p4-r1-stdlib)
at `800e1793`. Inspect the cross-layer paths named by the plan and verify in
particular:

1. Parser handling of valid/invalid Char literals and pattern syntax.
2. Semantic identity, literal typing, scalar-set coverage, duplicate and
   usefulness diagnostics, witness behavior, and blocked unknown subjects.
3. Semantic-product serialization/fingerprints and tooling/source-index
   projection for Char patterns.
4. Lowering of proven patterns to direct exact Char comparisons and safe
   behavior for blocked/non-proven match results.
5. Runtime immediate-value validity, equality/hash/rendering, Universe class
   identity, static materialization, and generated native-surface consistency.
6. Missing semantic and core regression tests, added at their owning layers.

## Next actions

1. Confirm exact test names/selectors and run the P11 focused gates from the
   plan: AST lexer/parser, semantic literal/static/pattern coverage, and core
   runtime language-corpus behavior. Run directly affected crate checks if a
   focused target cannot compile. Do not treat zero-selected tests as evidence.
2. Fix failures caused by P11; classify unclear or unrelated failures with
   bounded evidence and preserve them in the checkpoint if deferred.
3. Update this checkpoint and replace this interim walkthrough/handoff with
   final records only after the plan's required implementation and evidence are
   complete. Keep broad workspace/release gates outside P11.

Do not change C4's primary `active_plan` from P4 or mark P11 complete based only
on the committed source or a clean/pushed branch.
