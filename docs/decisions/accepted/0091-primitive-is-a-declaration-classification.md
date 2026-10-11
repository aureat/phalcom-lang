# TDR-0091 — `primitive` Is a Declaration Classification Modifier

- Status: Accepted
- Ratified: 2026-10-11
- Amends: [TDR-0082](./0082-declaration-modifiers-and-identity-based-access-domains.md) for `@primitive`; [TDR-0089](./0089-intrinsic-is-a-first-class-implementation-keyword.md) for the status of `@primitive`
- Amends specification: [Source Declarations](../../specs/source-declarations.md)
- Related: TDR-0017 (the installed VM primitive floor); TDR-0050 (String byte accessors and raw output)

## Context

The source language needs to identify declarations that form an irreducible
runtime foundation. The old `@primitive` attribute mixed that classification
with ordinary source attributes and made it easy to confuse source metadata
with native implementation registration. `native` already describes the
implementation origin, so the language needs a separate declaration modifier
for the classification.

The source classification is distinct from Rust `#[primitive]`, native
descriptor registration, compiler intrinsic IDs, and the frozen installed
primitive floor. Those mechanisms determine implementation and runtime
installation. This declaration modifier does not.

## Decision

1. `primitive` is a reserved declaration keyword and an orthogonal declaration
   classification. It does not add an `ImplementationKind` or visibility.
2. Canonical source order is `[visibility] [implementation] [primitive]`.
   `primitive` requires `native`.
3. Initially, the modifier is legal on native class declarations and native
   methods, getters, setters, and index declarations in the canonical Universe
   project. Other declaration targets and uses outside canonical Universe are
   rejected.
4. A class marked `primitive` does not mark its methods or subclasses. A
   primitive method does not mark overrides.
5. The modifier does not change access, selector or field identity, lowering,
   bytecode, dispatch, representation, native linking, runtime installation,
   or authorization. It grants no privilege.
6. The compiler and semantic core-surface products retain the classification
   where those source declaration products are available. It does not create
   or promise runtime reflection for source declarations.
7. `@primitive` is retired, with a migration diagnostic directing source to
   `native primitive`. It is not a compatibility alias.
8. The Universe declarations marked in the primitive inventory are explicit
   source classifications. Candidate collection and convenience APIs remain
   unclassified unless separately accepted.

## Consequences

- Source syntax, AST metadata, Universe validation, semantic source surfaces,
  and tooling presentation share one classification flag.
- Existing native primitive installation tables and Rust native registrations
  remain unchanged.
- TDR-0082 and TDR-0089 remain historical records; this decision supersedes
  only their prior `@primitive` rule.
