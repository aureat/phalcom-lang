# Annotations and Phaldoc

> Sources: DOCS001 and DOCS003 implementation records
> Raw: [language specification and program snapshot](../raw/language/2026-09-08-language-sources.md)
> Updated: 2026-09-08

Annotations and Phaldoc are a tooling-facing language surface. Their syntax must be parsed without making documentation prose part of semantic type authority. DOCS001 is an in-progress documentation-organization program; DOCS003 is proposed and unverified.

## Pipeline

Source comments and annotations are captured at the owning declaration, normalized into documentation records, and then consumed by Phaldoc generation or editor presentation. A generator may report malformed annotation syntax, but it must not reinterpret a declaration's formal type facts.

## Status

This article intentionally distinguishes design examples and documentation plans from executable language conformance. [REPL and documentation tooling](../tooling/phaldoc-and-documentation.md) owns generation, output organization, and the remaining implementation boundary.
