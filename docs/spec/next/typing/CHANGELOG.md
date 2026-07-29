# Changelog

## 2026-07-23 — Document 03 checkpoint

### Added

- `docs/spec/design/typing/03-type-parameters-and-generic-signatures.md`;
- generic declaration grammar for classes, protocols, instance methods, class-side methods, and protocol requirements;
- `Variance`, `TypeParameterSpec`, `TypeParameterOwner`, `TypeParameter`, `GenericSignature`, and trusted binding source models;
- owner-plus-index parameter identity and exact reflected annotation identity;
- invariant defaults and reflected `in`/`out` declaration-site variance;
- distinct upper-bound and finite-constraint representations;
- reserved-but-unsupported default metadata;
- lexical scoping, shadowing, enclosing-parameter references, and first-version recursive-restriction rules;
- compiler shell creation, AST, metadata, VM, bootstrap, GC, diagnostic, and conformance obligations.

### Changed from the Phase 1 reference package

- made public `TypeParameter` construction trusted and owner-attached rather than forgeable through `new(...)`;
- retained `default` in the descriptor shape while rejecting non-`None` defaults;
- restricted declaration-site variance to class/protocol owners and made method-owned parameters invariant;
- made finite explicit constraints use exact type equivalence and preserved one-element constraint sets;
- added atomic canonical `GenericSignature` attachment for every generic owner;
- specified lexical shadowing and exact annotation-object identity;
- deferred same-signature recursive bounds and guarded F-bounds to Document 09;
- preserved Document 01's forward-referenced runtime selectors through compatible overloads.

## 2026-07-23 — Document 02 checkpoint

### Added

- `docs/spec/design/typing/02-type-expression-foundation.md`;
- the signature-only `Type` protocol;
- direct class/protocol participation without wrapper descriptors;
- `TypeDescriptor` as the abstract trusted base for synthetic descriptors;
- normalized meanings for origin, arguments, declared parameters, free parameters, substitution, equivalence, and hash;
- trusted descriptor recognition and normalization boundaries;
- the reserved singleton `Type.currentApplication` intrinsic and its `TypeRuntime` source anchor;
- compiler, AST, VM, bootstrap, GC, reflection, diagnostic, and conformance obligations for the type-expression floor.

### Changed from the Phase 1 reference package

- removed executable `currentApplication` from the `@protocol` body;
- made bare generic origins closed descriptor objects rather than implicit `Origin<T...>` expressions;
- omitted ambiguous foundation requirements such as `isGeneric` and `isApplied`;
- made absent annotations remain explicitly absent;
- separated trusted normalized metadata from arbitrary structural descriptor lookalikes;
- specified singleton-intrinsic reflection and reservation instead of protocol default behavior.

## 2026-07-23 — Document 01 checkpoint

### Added

- `docs/spec/design/typing/01-protocol-foundation.md`
- first-class `Protocol` declaration-product semantics;
- signature-only instance-side and class-side requirements;
- validated direct manual construction through `Protocol.new(...)`;
- complete protocol foundation source model in Phalcom syntax;
- compiler, AST, metadata, interpreter, VM, bootstrap, GC, and reflection obligations;
- stable diagnostic taxonomy and source-range requirements;
- positive, negative, manual, bootstrap, mutation, GC, and malformed-metadata fixtures;
- series README and status ledger.

### Changed from the Phase 1 reference package

- Protocols are now specified before the wider type-expression library.
- Executable methods inside `@protocol` declarations are rejected.
- Protocol requirements are distinct non-executable descriptors rather than method-like stubs.
- Manual construction uses ownerless immutable drafts and trusted owner attachment.
- Class-side requirements are included as first-version metadata.
- Protocol identity, module binding, recursive shells, and malformed-metadata behavior are explicit.
