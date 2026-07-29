# Experimental spec drafts

Staging area for **unratified** design proposals. Nothing here is committed;
each file resolves an open question or records a hazard the [overlay](../../../../.claude/skills/language-design/phalcom/overlay.md)
flags. Promote to `docs/adr/` + `docs/spec/` on ratification.

| Draft | Resolves | Status |
|-------|----------|--------|
| ~~default-arguments.md~~ → [drafts/default-arguments.md](../drafts/default-arguments.md) | open-Q12 — defaults ⊗ selector identity | **RETIRED 2026-07-15** (DEFERRED CB-4). open-Q12 is RULED (no defaults, [ADR-0043](../../../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md)); the doc advocated in its body the caller-side desugar its own banner permanently forbade. The draft supersedes it. |
| [concurrency-adr.md](concurrency-adr.md) | Fiber/Future ADR + GC-root rule | Proposed |
| [scheduler-unit.md](scheduler-unit.md) | scheduler ownership + bootstrap order | Proposed |
| [bound-callable-unification.md](bound-callable-unification.md) | `Family` vs `Method.bind` (open-Q14) | Proposed |
| [annotations-core.md](annotations-core.md) | selectors.md §4 — `@` mechanism, expander registry, phases | Proposed |
| [annotations-contracts.md](annotations-contracts.md) | `@requires`/`@ensures`/`@invariant` (DbC, reflectable) | Proposed |
| [construct-derive.md](../../../work/pending/ctor/notes/construct-derive.md) | `@construct`/`@get`/`@set` + field-decl layout tier | Pending archive |
| [annotation-paradigm-bridges.md](annotation-paradigm-bridges.md) | `@data`/`@observable` bridges; two-tier finding | Proposed |
| [legality-grammar.md](../../../work/pending/ctor/notes/legality-grammar.md) | `@` EBNF, `Target`/legality table, unknown-attr, newline binding | Pending archive |
| [annotations-contract-semantics.md](annotations-contract-semantics.md) | invariant re-entrancy, predicate purity, release stripping | Proposed |
| [construct-inheritance.md](../../../work/pending/ctor/notes/construct-inheritance.md) | super-construct chaining, collisions, field defaults | Pending archive |
| [test-strategy.md](../../../work/pending/ctor/notes/test-strategy.md) | AST snapshots, `.ph` corpus, diagnostics catalog | Pending archive |
| [annotations-data.md](annotations-data.md) | `@data`/`@sealed`/`@variant` — structural records, closed hierarchies, generated visitor dispatch (no new `match` grammar) | Proposed |
| [iteration-protocol.md](iteration-protocol.md) | untracked — `iterate(_)`/`iteratorValue(_)`; unblocks `for` | Proposed |
| [equality-and-hash.md](equality-and-hash.md) | untracked — `==`/`hash` ladder, NaN keys, mutable keys | Proposed |
| [numeric-and-string-indexing.md](numeric-and-string-indexing.md) | untracked — integral indices, bitwise, codepoint strings | Proposed |
| [fiber-ensure-and-limits.md](fiber-ensure-and-limits.md) | untracked — dropped-fiber `ensure`, stack/alloc caps | Proposed |
| [typing.md](spec/design/experimental/typing.md) | untracked — optional/structural/erasable type layer (design note) | Experimental |
| [typing-initialization.md](typing-initialization.md) | typing.md #1 — typed field/`var` init vs "unassigned ⇒ `None`" | Proposed |
| [typing-subtyping.md](typing-subtyping.md) | typing.md #2/#3 — conformance termination + override/Liskov | Proposed |
| [typing-inference.md](typing-inference.md) | typing.md #4/#7 — local type-arg inference + default return | Proposed |
| [typing-stdlib-surface.md](typing-stdlib-surface.md) | typing.md #5/#6/#8 — root protocol, `==`, variadics, catch, literals | Proposed |
| [bootstrapping-and-self-hosting.md](bootstrapping-and-self-hosting.md) | untracked — stdlib-in-itself + compiler-in-Phalcom ladder; reopens overlay §Compiler; DEC-A pivot | Experimental |
| [doc-comments-phaldoc.md](doc-comments-phaldoc.md) | untracked — Phaldoc `///`/`//!` doc standard; inert-today convention; disjoint from code `@` registry | Proposed |
