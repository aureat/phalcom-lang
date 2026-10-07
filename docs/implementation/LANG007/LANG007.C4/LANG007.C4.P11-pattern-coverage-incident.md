# IMPLEMENTATION INCIDENT — INC-6 Char Literal Pattern Coverage

## 1. Identity

- Program/checkpoint/plan/task: `LANG007.C4.P11.T4`
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/lang007-c4-p4-r1-stdlib`
- Branch: `codex/lang007-c4-p4-r1-stdlib`
- Starting revision: `615f5887`
- Current revision: `615f5887` with uncommitted implementation changes

## 2. Escalation trigger

Mandatory architectural consultation: the accepted follow-on requires Char
literal pattern behavior, while the live semantic coverage model only defines
declaration-backed constructor and structural patterns. Adding scalar-domain
coverage changes semantic authority and the compiler/lowering contract.

## 3. Required invariant

- Char patterns compare by Unicode scalar value and remain disjoint from Int.
- Semantic coverage remains authoritative; compiler lowering consumes its
  canonical product and must not re-solve or approximate coverage.
- Uncertainty must remain blocked/unknown rather than being reported as
  unreachable or exhaustive.

## 4. Expected architecture

The accepted follow-on requires Char literal patterns, their type diagnostics,
static identity, and materialization. The Char specification establishes the
scalar domain and equality, but does not define pattern subject compatibility,
usefulness, exhaustiveness, or subclass interaction.

## 5. Observed behavior

The live parser's `parse_pattern_atom` handles wildcard, tuple, list, record,
and identifier forms; `Token::Char` is not accepted as a pattern. Semantic
`PatternResolution`, `CoveragePattern`, and compiler `ExecutablePattern` have
no scalar-literal case. Coverage domain decomposition is currently based on
constructors, tuples, and lists.

## 6. Minimal reproduction

```telos
match value {
  'a' => 1,
  _ => 0,
}
```

Static parser inspection shows the Char token reaches the identifier fallback
in `parse_pattern_atom`. No runtime test was run.

## 7. Relevant code path

```text
phalcom-ast::Parser::parse_pattern_atom
  -> phalcom_ast::ast::Pattern
  -> phalcom-semantic::checker::pattern::resolve_pattern
  -> coverage_pattern_for_resolution
  -> CoverageEngine usefulness/exhaustiveness
  -> semantic MatchResolution
  -> phalcom-core executable pattern lowering
```

Relevant files: `phalcom-ast/src/parser.rs`, `phalcom-ast/src/ast.rs`,
`phalcom-semantic/src/match_semantics.rs`,
`phalcom-semantic/src/checker/pattern.rs`,
`phalcom-semantic/src/checker/coverage/{pattern,domain,usefulness}.rs`, and
`phalcom-core/src/modules/semantic_lowering.rs`.

## 8. Scoped implementation diff

The current work adds Char tokens/expressions, static scalar values, an
immediate runtime tag, canonical Universe identity/class, native Char
operations, and generated native-surface records. It does not change pattern
parsing, semantic resolution, coverage, or lowering.

## 9. Evidence

- The accepted follow-on states that S1 semantic acceptance includes literal
  pattern behavior (`LANG007.C4.P4.R1-stdlib-parse-telos-follow-on-plan.md`,
  requirements-to-gates ledger).
- The live `PatternResolution` variants are Wildcard, Binding, Variant, Or,
  Tuple, List, Record, and Map.
- The live `CoveragePattern` variants are Wildcard, Variant, Or, Tuple, List,
  RecordPredicate, and MapPredicate.
- The live `DomainDecomposition` variants are Empty, Closed, Open, and Blocked;
  constructor heads currently represent variants, tuples, and lists.
- The Char specification defines equality by scalar value and pins its
  Unicode-property behavior to Unicode 17.0.0; it does not define pattern
  coverage.

## 10. Implementer's causal hypothesis

This is not a missing local arm. Literal patterns require a formal scalar
coverage domain and an executable exact-value matcher. The language must decide
how Char's finite scalar set participates in exhaustiveness and how nominal
subclassing interacts with the immediate scalar category.

## 11. Alternative hypotheses

1. Treat Char as a closed scalar domain for pattern analysis. Track covered
   scalar sets/ranges, diagnose duplicate scalar literals as impossible, and
   prove exhaustiveness only when the set covers every valid scalar or a
   catch-all is present.
2. Treat Char as an open domain for coverage. Exact duplicate scalar literals
   can be refuted, but only a catch-all proves exhaustiveness.
3. Keep literal patterns outside P11 until Char finality/subclassing and match
   semantics are specified. This conflicts with the accepted P11 follow-on and
   therefore requires an explicit amendment if chosen.

## 12. Attempts already made

No pattern implementation attempt was made. The issue was identified by
comparing the accepted P11 requirement with the live parser and coverage model.

## 13. Why local authority is insufficient

The choice changes source-visible reachability and exhaustiveness diagnostics,
formal semantic products, and executable pattern lowering. The accepted Char
specification supplies equality but not this match contract. Selecting one
coverage model locally would silently invent language semantics.

## 14. Decision requested

Choose the Char literal-pattern contract: closed Unicode-scalar coverage with
scalar-set exhaustiveness; conservative open-domain coverage with catch-all
exhaustiveness; or an explicit P11 scope amendment that defers patterns pending
a separate match-semantics plan. Also confirm whether Char may be subclassed as
a nominal type independently of immediate scalar values.

## 15. Verification

No tests or crate checks were run. The native-surface generator was run to
produce its checked-in artifact; it completed with 373 primitive declarations.

## 16. User decision and resolution

On 2026-10-07 the user selected the closed scalar-domain contract: Char is
non-subclassable for matching; exact literals contribute scalar-set coverage;
all valid Unicode scalar values may prove exhaustiveness. This decision is
recorded normatively in `docs/specs/objects/characters.md` and in the P11 plan.

Implementation resumes across AST pattern syntax, semantic pattern resolution,
scalar coverage and witnesses, semantic product fingerprints/tooling, and
canonical executable pattern lowering. The matcher must compare the runtime
Char scalar directly. Unknown subject types remain blocked rather than being
reported impossible. The plan remains `IN_PROGRESS / PARTIAL / UNVERIFIED`
until implementation and required evidence are complete.
