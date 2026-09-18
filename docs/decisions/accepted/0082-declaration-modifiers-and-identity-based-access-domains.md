# TDR-0082 — Declaration Modifiers and Identity-Based Access Domains

- Legacy record: PDR-0037
- Ratified: 2026-09-18

- Status: Accepted
- Date: 2026-09-18
- Supersedes: TDR-0076, for declaration modifiers, visibility, implementation
  origin, and implementation-namespace rules
- Amends: `docs/specs/source-declarations.md`; `docs/specs/universe.md` where
  it describes native declaration ownership
- Related: LANG006.C1; TDR-0011 (selector identity); TDR-0038 (module
  identity); TDR-0053 (closed classes)
- Repository evidence revision: `aureat/phalcom-lang@bf4e12ba`

## Context

Phalcom historically represented some visibility and implementation facts with
attributes and reserved identifier prefixes. That made selector identity,
access control, native binding ownership, and Universe provenance depend on
parallel naming conventions. It also allowed a native implementation to be
mistaken for a blanket runtime privilege.

LANG006.C1 establishes one source-to-runtime model for these facts.

## Decision

1. The source keywords `native`, `foreign`, `private`, `protected`, `module`,
   `package`, and `internal` are declaration modifiers. There is no `public`
   keyword; omitted visibility is `Public`.
2. Parsing normalizes modifiers into the shared `Visibility` and
   `ImplementationKind` products. Names never imply either product.
3. `Private` is owner-identity scoped, `Protected` is owner/subclass scoped,
   `Module` is exact-module scoped, `Package` is exact-package scoped, and
   `Internal` is exact-project scoped. The same relation applies to direct,
   cached, dynamic, reflective, and native calls.
4. `native` is declaration-derived implementation origin, not an access grant.
   Native descriptors inherit the authority of the declaration they implement.
   `foreign` remains a distinct bounded scaffold and is not an implicit native
   loader or ABI.
5. `@primitive` and `@intrinsic` are metadata-only and Universe-only. They do
   not change visibility, implementation kind, lowering, representation, or
   authorization.
6. Authored `_$name` selectors and general `__name` implementation fields are
   retired. They must not parse or normalize as Internal declarations.
   Standardized `__name__` protocol identifiers remain unchanged.
7. `is_universe` remains an independent bootstrap/provenance fact. It is not
   ordinary Internal authorization.

## Consequences

- AST, semantic surfaces, native metadata, compiler lowering, VM access origin,
  reflection, LSP, and syntax tooling consume one structured authority.
- Legacy spellings can receive migration diagnostics, but no compatibility
  path may derive visibility or implementation origin from them.
- Universe source uses ordinary selector/field identity and explicit modifiers;
  native registrations use the same selector identities as source.
- Historical plans and decision records retain their original wording; this
  record and the owning `docs/specs` documents define the current contract.
