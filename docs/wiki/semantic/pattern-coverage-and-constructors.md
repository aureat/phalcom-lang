# Pattern coverage and constructors

> Sources: `docs/spec/adts.md`, SEMA008–SEMA009
> Raw: [semantic specification and program snapshot](../raw/semantic/2026-09-08-semantic-sources.md)
> Updated: 2026-09-08

Coverage analysis computes usefulness, exhaustiveness, and witnesses over the semantic match space. Constructor completion computes which constructor declarations are legal and complete. They are related but not interchangeable: coverage proves cases against an established family; completion establishes the family surface.

## Proof boundary

GADT refinements and recursive patterns require canonical constructor identity and relation evidence. A witness is a diagnostic product, not a substitute for the failed proof. The compiler may lower only after semantic status and match-space facts are available.

SEMA008 and SEMA009 are proposed and unverified. [Patterns and matching](../language/patterns-and-matching.md) covers syntax; [report model](../diagnostics/report-model.md) covers witness presentation.
