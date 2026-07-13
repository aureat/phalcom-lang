# Experimental spec drafts

Staging area for **unratified** design proposals. Nothing here is committed;
each file resolves an open question or records a hazard the [overlay](../../../../.claude/skills/language-design/phalcom/overlay.md)
flags. Promote to `docs/adr/` + `docs/spec/` on ratification.

| Draft | Resolves | Status |
|-------|----------|--------|
| [default-arguments.md](default-arguments.md) | open-Q12 — defaults ⊗ selector identity | Proposed |
| [concurrency-adr.md](concurrency-adr.md) | Fiber/Future ADR + GC-root rule | Proposed |
| [scheduler-unit.md](scheduler-unit.md) | scheduler ownership + bootstrap order | Proposed |
| [bound-callable-unification.md](bound-callable-unification.md) | `Family` vs `Method.bind` (open-Q14) | Proposed |
| [annotations-core.md](annotations-core.md) | selectors.md §4 — `@` mechanism, expander registry, phases | Proposed |
| [annotations-contracts.md](annotations-contracts.md) | `@requires`/`@ensures`/`@invariant` (DbC, reflectable) | Proposed |
| [annotations-construct.md](annotations-construct.md) | `@construct`/`@get`/`@set` + field-decl layout tier | Proposed |
| [annotation-paradigm-bridges.md](annotation-paradigm-bridges.md) | `@data`/`@observable` bridges; two-tier finding | Proposed |
| [annotations-legality-grammar.md](annotations-legality-grammar.md) | `@` EBNF, `Target`/legality table, unknown-attr, newline binding | Proposed |
| [annotations-contract-semantics.md](annotations-contract-semantics.md) | invariant re-entrancy, predicate purity, release stripping | Proposed |
| [annotations-construct-inheritance.md](annotations-construct-inheritance.md) | super-construct chaining, collisions, field defaults | Proposed |
| [annotations-test-strategy.md](annotations-test-strategy.md) | AST snapshots, `.ph` corpus, diagnostics catalog | Proposed |
| [annotations-data.md](annotations-data.md) | `@data`/`@sealed`/`@variant` — structural records, closed hierarchies, generated visitor dispatch (no new `match` grammar) | Proposed |
| [iteration-protocol.md](iteration-protocol.md) | untracked — `iterate(_)`/`iteratorValue(_)`; unblocks `for` | Proposed |
| [equality-and-hash.md](equality-and-hash.md) | untracked — `==`/`hash` ladder, NaN keys, mutable keys | Proposed |
| [numeric-and-string-indexing.md](numeric-and-string-indexing.md) | untracked — integral indices, bitwise, codepoint strings | Proposed |
| [fiber-ensure-and-limits.md](fiber-ensure-and-limits.md) | untracked — dropped-fiber `ensure`, stack/alloc caps | Proposed |
| [typing.md](typing.md) | untracked — optional/structural/erasable type layer (design note) | Experimental |
| [typing-initialization.md](typing-initialization.md) | typing.md #1 — typed field/`var` init vs "unassigned ⇒ `None`" | Proposed |
| [typing-subtyping.md](typing-subtyping.md) | typing.md #2/#3 — conformance termination + override/Liskov | Proposed |
| [typing-inference.md](typing-inference.md) | typing.md #4/#7 — local type-arg inference + default return | Proposed |
| [typing-stdlib-surface.md](typing-stdlib-surface.md) | typing.md #5/#6/#8 — root protocol, `==`, variadics, catch, literals | Proposed |
| [bootstrapping-and-self-hosting.md](bootstrapping-and-self-hosting.md) | untracked — stdlib-in-itself + compiler-in-Phalcom ladder; reopens overlay §Compiler; DEC-A pivot | Experimental |
| [doc-comments-phaldoc.md](doc-comments-phaldoc.md) | untracked — Phaldoc `///`/`//!` doc standard; inert-today convention; disjoint from code `@` registry | Proposed |
