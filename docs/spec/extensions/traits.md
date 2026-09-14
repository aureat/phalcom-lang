# First-Class Traits

**Status:** Ratified effective extension for `LANG005.C3`
**Scope:** Instance behavioral contracts and their declaration-local defaults

This is the canonical language rule for first-class traits. It supersedes the
protocol-shaped behavioral-contract rules in the historical typing documents
listed in [the extension migration note](README.md#migration-note).

## 1. Declaration

A trait is a named behavioral declaration:

```text
trait_decl := "trait" IDENT [ generic_parameters ] [ where_clause ]
               "{" { behavior_member } "}"
```

For example:

```phalcom
trait Collection<T> where T <: Object {
  required(_ value: T) -> T
  name -> String
  value=(_ next: T) -> T
  [_ index: Int] -> T
  fallback(_ value: T) -> T { value }
}
```

The declaration name and its generic binders are resolved through the ordinary
module declaration and generic-signature machinery. A trait is a distinct
declaration category; it is not `class` syntax with an attribute, a nominal
superclass, or an alias for an ordinary class.

Trait members reuse the behavior-member grammar for methods, getters, setters,
and index accessors. Fields, stored components, constructors, `@class`
members, and `super` are not legal in a trait. Trait member attributes remain
subject to the ordinary attribute legality rules, except that attributes which
would introduce storage, construction, class-side behavior, or a second
contract mechanism are rejected.

## 2. Requirements and defaults

Every trait behavior member defines one requirement. The member's selector,
dispatch side, parameter structure, annotations, visibility, and source
identity are part of that requirement.

- A bodyless member is an abstract requirement.
- A bodyful member is the same abstract requirement plus a declaration-local
  default body.

The two forms therefore share one stable `TraitRequirementId`. The canonical
source/default body uses the ordinary `CallableId` for the member; it is not
replaced by a parallel trait-member or default-member identity.

All member signatures are published into the complete `TraitSurface` before
any default body is checked. A default is checked once against the trait's
abstract `Self` surface. Calls through `Self` can therefore resolve only to
declared trait requirements and defaults remain contract-relative rather than
being bound to a future conforming class.

## 3. Identity and generic contracts

The trait declaration is identified by the canonical module `DeclarationId`
and verified `DeclarationKind::Trait`. Its generic parameters use the
declaration-owned generic signature.

`TraitRef` is a contract reference consisting of the trait declaration identity
and canonical generic arguments. It is not a `TypeId`, an inhabitable value
type, a runtime trait object, a class object, or conformance evidence. A trait
generic signature does not create an entry in the nominal declaration type
table.

Trait requirements have a dedicated `TraitRequirementId` so requirement
identity remains stable across bodyless/bodyful edits. Member source navigation
continues to use the canonical declaration and callable semantic targets.

## 4. Representation and execution boundary

A trait owns no instance representation. It has no fields, stored components,
product layout, superclass edge, constructor-managed storage, ordinary runtime
class, method-injection table, witness table, or conformance registry.

In `LANG005.C3`, the compiler and VM treat a trait declaration as compile-time
and type-level contract information. Declaration processing publishes semantic
headers, surfaces, requirements, defaults, dependencies, and source targets;
it does not allocate a runtime class or install runtime methods. Ordinary class
dispatch remains trait-unaware.

## 5. C3 boundary

This extension defines instance-contract-first trait declarations only. The
following are not part of the effective C3 language rule:

- `impl Trait for Type`, conformance proofs, structural or nominal conformance
  inference, witness/default selection, and overlap/coherence;
- associated types or associated bindings;
- `T: Trait` constraints or conditional conformances;
- trait objects, existentials, runtime trait descriptors, or conformance
  reflection;
- supertraits and class-side/metatype trait requirements;
- trait-driven runtime dispatch or method injection.

Those capabilities require later checkpoint authority and must not be inferred
from the existence of a `TraitRef`, a default body, or a `TraitSurface`.
