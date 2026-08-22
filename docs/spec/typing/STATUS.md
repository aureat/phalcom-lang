# Typing Specification Status

- **Checkpoint date:** 2026-07-23
- **Completed:** Documents 01–03
- **Next:** Document 04 — Type Application and Applied Types

## Decisions locked by Document 01

- `@protocol class Name { ... }` produces a distinct first-class `Protocol` descriptor, not a flagged `Class`.
- Protocols are signature-only and never provide default implementations, traits, mixins, or inherited stubs.
- Instance-side and class-side requirements are both supported from the first version.
- Class-side requirements are metadata about candidate class objects; they are not methods on the protocol descriptor.
- Protocol conformance is structurally interpreted later, while protocol identity remains declaration-object identity.
- Selector identity remains the normal selector, including labels and positional structure; types never become dispatch keys.
- A public validated `Protocol.new(...)` constructor is the canonical manual-construction API.
- Manual construction requires an explicit module owner and creates a fresh identity without automatic namespace binding.
- Decorator declarations use trusted recursive shells, exact compiler source metadata, and automatic lexical binding.
- Requirements and parameters are immutable owned descriptors with owner-plus-index identity.
- Protocols cannot be instantiated, reopened, mutated, or used as allocation origins.
- Only retain-tier requirement attributes are legal; behavior-changing attributes are rejected.
- Protocol inheritance/composition and the complete structural-conformance algorithm are deferred to Document 10.

## Decisions locked by Document 02

- Historical: `Type` was proposed as a signature-only `Protocol` descriptor (superseded by `ontology.md`, where `Type` is the atomic kind and `TypeForm` names the common type-denoting role).
- Existing `Class` and `Protocol` objects are type expressions directly; no `ClassType` or `ProtocolType` wrappers are observable.
- Bare class/protocol normalization preserves exact object identity.
- Bare generic origins expose declared `typeParameters` but have no free-parameter occurrences and are not implicit open applications.
- `TypeDescriptor` is the abstract trusted implementation base for synthetic type-expression descriptors.
- First-version compiler metadata accepts recognized trusted descriptor kinds; arbitrary structural lookalikes are not automatically authoritative metadata.
- `displayName` is presentation, not identity.
- `equivalentTo(_:)` is distinct from subtype, consistency, acceptance, conformance, and ordinary value equality.
- Bare class and protocol equivalence is descriptor identity.
- Synthetic descriptors are immutable; equivalence and hash are stable and compatible.
- Missing annotations remain `None` and are not rewritten to `Dynamic`, `Any`, or `Object`.
- `Type.currentApplication` is a reserved singleton intrinsic on the exact canonical `Type` object, backed by `TypeRuntime.currentApplication`.
- The intrinsic is not a protocol requirement, default method, per-protocol metaclass method, or user-installable extension.
- Detailed current-application frame behavior remains assigned to Document 06.

## Decisions locked by Document 03

- Generic classes, protocols, methods, and protocol requirements own first-class immutable `TypeParameter` objects.
- Parameter identity is owner descriptor identity plus zero-based declaration index; name is descriptive only.
- Repeated reflection returns the same parameter objects, and member annotations reference those exact objects.
- Unmarked class and protocol parameters are invariant; `out` and `in` produce real reflected variance values.
- Method- and requirement-owned type parameters are invariant; declaration-site `in`/`out` is rejected on them.
- `T: Bound` is an upper bound; `T in (A, B)` is a finite exact constraint set.
- A parameter may declare a bound or finite constraints, never both.
- A one-element finite constraint remains legal and differs from an upper bound by exact-equivalence semantics.
- Default type arguments are represented by `default -> Option<Type>` but must be `None` in version one.
- `TypeParameterSpec` is the ownerless public manual-construction input; binding creates fresh owned identities.
- `TypeParameterOwner` is structurally public, while compiler metadata accepts only trusted owner kinds.
- Generic owners expose `genericSignature -> Option<GenericSignature>`; non-generic owners expose `None`.
- Nested declarations may lexically shadow outer type-parameter names; nearest scope wins.
- Restrictions may refer to enclosing-owner parameters.
- Same-signature recursive restrictions, including guarded F-bounds, are rejected until Document 09.
- Generic metadata never changes selector identity, ordinary dispatch, layout, allocation, or automatic value validation.
- Document 01's `validateSpecifications(... sourceLocation:)` and `bindOwned(owner:, specifications:)` selectors remain supported through compatible overloads.

## Open issues assigned to later documents

No unresolved decision remains inside Documents 01–03. The following work is intentionally assigned:

- Document 04: `TypeConstructor`, `<...>`, `AppliedType`, validation, interning, and raw-generic legality.
- Document 05: `TypeEnvironment`, `TypeBinding`, recursive substitution, and applied member/requirement views.
- Document 06: applied class-side forwarding and full current-application stack semantics.
- Document 07: special types and the equivalence/subtype/consistency/acceptance relation split.
- Document 08: variance-position validation, nested composition, and generic subtyping.
- Document 09: guarded F-bounds, intersection bounds, constraint solving, and inference.
- Document 10: structural conformance, protocol inheritance/composition, recursive protocols, explicit declarations, and cache invalidation.
- Document 11: generic abstract obligations.
- Document 12: generic inheritance, specialization, and inherited annotation substitution.
- Document 17: source-versus-normalized annotation records, final metadata encoding, custom descriptor portability, and intrinsic reflection records.
- Document 18: hardened native authority, malformed metadata policy, GC roots, weak caches, and selector security.
- Document 19: checker modes, raw-generic policy, LSP rendering, and compatibility.

## Dependency input for Document 04

Document 04 must preserve these earlier invariants:

- A bare generic origin is the closed declaration/type-constructor object; it is not an implicit `Origin<T...>` application.
- `<...>` is a reserved, reflectable primitive operation that users cannot declare, override, replace, or intercept.
- Exact application arity equals `GenericSignature.arity`; partial application and defaults are unsupported.
- Application validates each argument through the corresponding `TypeParameter.validate(_:)` in source order.
- Explicit finite constraints use equivalence membership; upper bounds use the subtype relation.
- Applied types are canonical immutable synthetic descriptors and do not create runtime subclasses or reified generic instances.
- Type parameters resolve by owner/index identity and remain non-dispatching metadata.
- Class and protocol origins remain unwrapped, identity-stable type expressions.
- Document 04 must use the stable diagnostics `type.application.argument_count`, `type.argument.bound_violation`, and `type.argument.constraint_violation`.
