---
id: LANG007.C4.P11
kind: implementation-walkthrough
title: "P11 Char scalar and literal implementation — interim walkthrough"
status: INTERIM
completion: PARTIAL
verification: UNVERIFIED
recorded_against_revision: 800e179325a4fa0d97e3cd1efa636d09d6fab393
recorded: 2026-10-07
branch: codex/lang007-c4-p4-r1-stdlib
---

# LANG007.C4.P11 — Interim Implementation Walkthrough

This is a progress record, not a plan-completion record. P11 remains
`IN_PROGRESS / PARTIAL / UNVERIFIED`; no behavioral tests or crate checks have
been run.

## Implemented in the current revision

Implementation commit [`800e1793`](https://github.com/aureat/phalcom-lang/commit/800e179325a4fa0d97e3cd1efa636d09d6fab393)
adds the following cross-layer pieces:

- A dedicated Char token and AST expression, single-quoted literal parsing,
  escape/scalar validation, and Char syntax in pattern position.
- Canonical semantic Char identity, literal typing, static scalar values,
  fingerprints, materialization, source indexing, and tooling products.
- An immediate runtime Char representation, scalar equality/rendering,
  canonical Universe class and native surface, primitive operations, and
  regenerated native-surface records.
- Semantic Char pattern resolution and a closed valid-scalar coverage domain.
  Exact literals contribute singleton coverage; witnesses exclude surrogates;
  unresolved subject types remain blocked.
- Canonical executable pattern projection and direct exact runtime Char
  comparison. Compiler conditional-pattern lowering was also adjusted to keep
  fallback behavior for non-proven results.
- A normative Char specification and specification-index entries.

## Decision and incident

INC-6 was escalated because literal patterns needed an explicit coverage and
subclassing contract. The user selected a closed Unicode scalar domain: Char is
non-subclassable for matching, exact literals contribute scalar-set coverage,
and the complete valid scalar set can prove exhaustiveness. The decision and
incident progression are recorded in the
[INC-6 incident record](LANG007.C4.P11-pattern-coverage-incident.md).

## Evidence acquired

- The source changes and generated native-surface artifact are present in
  commit `800e1793` on the pushed branch.
- Lexer/parser test cases were added under `phalcom-ast`, but they were not run.
- No semantic or core regression test files were added in this commit.
- No tests, crate checks, workspace checks, or release gates were run. The
  implementation's behavior and cross-layer compilation remain unverified.

`git diff --check` passed during the preceding implementation delivery. That is
whitespace/diff hygiene evidence only; it does not establish runtime or semantic
correctness.

## Remaining acceptance work

Use the focused gates in the
[P11 plan](LANG007.C4.P11-char-and-literals-plan.md). First confirm the test
selectors select real tests, then acquire AST lexer/parser evidence, semantic
identity/static-value/pattern-coverage evidence, and core runtime identity,
materialization, and exact-match evidence. Add missing semantic and core
regressions at their owning layers. Record failures against the current branch
and classify unrelated failures without weakening the accepted scalar contract.

Update the checkpoint, walkthrough, and handoff with the evidence actually
acquired before marking P11 complete. Do not promote P11 or C4 verification from
this interim record.
