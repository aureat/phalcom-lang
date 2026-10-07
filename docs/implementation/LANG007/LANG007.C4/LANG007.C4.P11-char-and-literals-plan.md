---
id: LANG007.C4.P11
category: LANG
program: LANG007
checkpoint: LANG007.C4
kind: implementation-plan
title: "Unicode Char scalar and character literals"
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
prepared_against_revision: 615f5887
prepared: 2026-10-07
---

# LANG007.C4.P11 — Unicode Char Scalar and Character Literals

## Objective

Implement the accepted first slice of the standard-library text substrate: a
first-class immediate `Char`, single-quoted character literals and literal
patterns, canonical semantic identity, static value support, and runtime class
behavior. This plan does not implement String decoding, character iteration, or
parser text combinators; those belong to P12.

## Fixed architecture

- `Char` is a distinct scalar value, not an `Int` alias, a one-scalar String,
  or a per-character heap object.
- Valid values are Unicode scalar values U+0000..U+D7FF and U+E000..U+10FFFF.
- Single-quoted literals decode escapes first and then require exactly one
  scalar. Errors retain UTF-8 byte ranges.
- The canonical `Char` declaration belongs to the Universe source and semantic
  identity derives from that declaration. Native metadata enriches it; it does
  not create a second declaration owner.
- Equality, exact sameness, and hashing use the scalar value. There is no
  implicit conversion or arithmetic with `Int`.
- `Char` is a closed scalar domain and is non-subclassable for matching. A
  literal pattern matches one exact scalar and contributes that singleton to
  coverage. Matching does not invoke user-defined equality.
- Coverage can prove a `Char` match exhaustive only when all valid Unicode
  scalars are covered or a catch-all is present. Surrogates are excluded.
- Semantic coverage owns the scalar-set proof. Executable lowering consumes the
  resolved scalar and emits direct exact-value matching.
- `Char` is immediate in the existing 16-byte `Value` representation. A new
  representation/tag change must stop for consultation if it cannot fit the
  existing specified layout.
- Parser and source spans remain byte-addressed.

## Verify first

- Confirm the live normative authority and generated native-surface workflow.
- Confirm every exhaustive AST expression walker and every `ValueTag` match.
- Confirm how canonical Universe declarations flow into semantic core IDs,
  native metadata, VM classes, and bootstrap installation.
- Record the pinned Rust Unicode property-data version used for whitespace and
  control classification: Unicode 17.0.0 from `nightly-2026-07-10`.
- Preserve the existing user's dirty worktree changes; do not reset, restore,
  or broadly format files.

## Tasks

### T1 — Normative contract and AST literal path

- Own `docs/specs/objects/characters.md` and register it in the specification
  index.
- Add a dedicated character token and AST expression carrying a validated
  scalar; do not reuse String or Int expressions.
- Scan plain UTF-8 scalars and the specified escapes, including `\\u{...}`.
- Reject empty/multi-scalar bodies, raw CR/LF, malformed/unknown escapes,
  surrogates, and values above U+10FFFF with precise source ranges.
- Add parser/lexer cases for valid ASCII, multibyte, escaped, and invalid forms.

### T2 — Semantic and static identity

- Add the canonical `Char` core declaration identity and type knowledge for
  literal expressions.
- Add scalar payloads to semantic static values, interning/fingerprinting,
  static evaluation, and semantic-to-runtime materialization.
- Preserve exactness and ensure `Char` remains disjoint from `Int` in types,
  equality, and diagnostics.

### T3 — Immediate runtime value and canonical class

- Add a checked Char constructor/accessor to `Value`, scalar equality/hash, and
  rendering/debug presentation.
- Add canonical Universe class metadata and runtime class mapping under
  `Object`, plus native surface declarations for the required Char API.
- Implement scalar conversion, UTF-8 width, string conversion, and the named
  ASCII/Unicode predicates at the owning primitive boundary.
- Update exhaustive value consumers, serialization/materialization, and
  generated native-surface artifacts where required.

### T4 — Literal patterns and acceptance record

- Lower `Expr::Char` to an immediate runtime constant and support static Char
  values without executing arbitrary native code during static eval.
- Parse Char literals in pattern position, resolve them as exact scalar
  patterns, diagnose impossible/redundant patterns, and integrate singleton
  scalar coverage with match usefulness and exhaustiveness.
- Project canonical Char pattern resolution to executable patterns and compare
  by exact Char value in the VM.
- Add focused semantic, static-value, runtime identity, and round-trip
  regressions at their owning layers.
- Update this checkpoint ledger, walkthrough, and handoff with evidence that
  actually ran.

## Focused gates

Run only after implementation reaches a coherent boundary, and confirm filters
select tests:

1. Lexer/parser character-literal tests in `phalcom-ast`.
2. Semantic literal identity and static-value tests in `phalcom-semantic`.
3. Core language corpus tests for scalar construction, class identity,
   classifications, equality, hashing, and String round-trip.
4. Directly affected crate checks if a focused test cannot compile the changed
   library surface.

Workspace/release gates are outside this plan. Do not claim them from a clean
tree or from focused tests.

## Stop and consult

Stop before editing if the two-word runtime representation cannot encode the
scalar without changing the accepted VM contract, if semantic identity would
need a second Universe registry, or if the static Char representation would
cross an unresolved ownership/materialization boundary. The Char pattern
contract was resolved by the user on 2026-10-07 and recorded in
`LANG007.C4.P11-pattern-coverage-incident.md`; continue with that decision and
escalate again only if the live implementation requires a different semantic
or ownership contract.

## Starting state

- Worktree branch: `codex/lang007-c4-p4-r1-stdlib`.
- Starting revision: `615f5887`.
- The worktree already contains the Char normative spec, VM tag-table update,
  specification-index entries, and initial lexer/parser smoke tests. These are
  inherited edits and must be reviewed in place, not replaced wholesale.
- No P11 production implementation or focused verification is established at
  plan start.
