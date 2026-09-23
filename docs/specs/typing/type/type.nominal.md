# Phalcom Type Algebra Registry — `type.nominal`

**Status:** Normative specification  
**Registry family:** `type.nominal`  
**Identifier range used here:** `T0750–T0766`

## Purpose

This file defines the semantic laws of nominal type identity and nominal subtyping.

Nominal types derive identity from declaration identity rather than from accidental structural equality. Two distinct nominal declarations remain distinct semantic types unless the language explicitly introduces an equivalence relation between them.

Structural views, interfaces, traits, and public record-like projections may participate in subtyping, but they do not erase nominal identity.

## Domain and notation

Unless stated otherwise:

- `decl(T)` denotes the nominal declaration that defines `T`.
- `shape(T)` denotes a structural view of `T` when one exists.
- `args(T)` denotes the ordered static argument tuple of a nominal instantiation; it is empty for a non-generic nominal type.
- `A`, `B`, and `C` denote proper nominal types.
- `==` means semantic type identity.
- `<:` means semantic subtype.

## Laws

```phalcom
@id(T0750)
@name(type.nominal.kind)
NominalTypeKind<A> {
    kind(A) == Type
}

@id(T0751)
@name(type.nominal.identity.requires-declaration)
NominalIdentityRequiresSameDeclaration<A, B> {
    if A == B {
        decl(A) == decl(B)
    }
}

@id(T0752)
@name(type.nominal.identity.requires-arguments)
NominalIdentityRequiresSameArguments<A, B> {
    if A == B {
        staticTupleEquivalent(args(A), args(B))
    }
}

@id(T0753)
@name(type.nominal.identity.declaration-and-arguments)
NominalIdentityFromDeclarationAndArguments<A, B> {
    if decl(A) == decl(B)
        and staticTupleEquivalent(args(A), args(B))
    {
        A == B
    }
}

@id(T0754)
@name(type.nominal.distinct-declarations)
DistinctNominalDeclarations<A, B> {
    if decl(A) != decl(B) {
        A != B
    }
}

@id(T0755)
@name(type.nominal.distinct-instantiations)
DistinctNominalInstantiations<A, B> {
    if decl(A) == decl(B)
        and not staticTupleEquivalent(args(A), args(B))
    {
        A != B
    }
}

@id(T0756)
@name(type.nominal.structure.not-identity)
StructuralEqualityDoesNotImplyNominalIdentity<A, B> {
    if shape(A) == shape(B)
        and (decl(A) != decl(B) or not staticTupleEquivalent(args(A), args(B)))
    {
        A != B
    }
}

@id(T0757)
@name(type.nominal.subtype.reflexive)
NominalSubtypeReflexive<A> {
    A <: A
}

@id(T0758)
@name(type.nominal.subtype.declared-parent)
DeclaredNominalParentSubtype<A, B> {
    if declaredParent(A, B) {
        A <: B
    }
}

@id(T0759)
@name(type.nominal.subtype.transitive)
NominalSubtypeTransitive<A, B, C> {
    if A <: B and B <: C {
        A <: C
    }
}

@id(T0760)
@name(type.nominal.structure.compatibility-not-identity)
NominalStructuralCompatibilityNotIdentity<A, S> {
    if shape(A) <: S {
        A <: S

        // does not imply:
        // A == S
    }
}

@id(T0761)
@name(type.nominal.alias.identity)
TransparentNominalAliasIdentity<A, B> {
    if transparentAlias(A, B) {
        A == B
    }
}

@id(T0762)
@name(type.nominal.wrapper.distinct)
NominalWrapperDistinct<A, B> {
    if wraps(A, B) and not transparentAlias(A, B) {
        A != B
    }
}

@id(T0763)
@name(type.nominal.generic.declaration-identity)
NominalGenericDeclarationIdentity<F, G, A> {
    if nominalConstructor(F)
        and nominalConstructor(G)
        and decl(F) != decl(G)
    {
        F<A> != G<A>
    }
}

@id(T0764)
@name(type.nominal.subtype.not-identity)
NominalSubtypeDoesNotImplyIdentity<A, B> {
    if A <: B and A != B {
        not (A == B)
    }
}

@id(T0765)
@name(type.nominal.never.distinct)
NominalTypeNotNever<A> {
    if inhabited(A) {
        A != Never
    }
}

@id(T0766)
@name(type.nominal.any.distinct)
NominalTypeNotAny<A> {
    if exists B where not (B <: A) {
        A != Any
    }
}
```

## Law notes

### T0750 — `NominalTypeKind`

A nominal type is a proper type.

**Kind:** Semantic law  
**Depends on:** T0060

### T0751–T0754 and T0755 — Nominal identity

Nominal identity is the product of declaration identity and the ordered semantic identity of static generic arguments. Declaration identity alone identifies the nominal constructor/head; it is insufficient to identify a specialization such as `List<Int>` versus `List<String>`.

Distinct declarations or distinct argument tuples produce distinct nominal types.

**Kind:** Semantic identity laws  
**Depends on:** T0001–T0003 and `type.static-value`

### T0756 — `StructuralEqualityDoesNotImplyNominalIdentity`

Coincident structural shape does not erase declaration or argument identity.

**Kind:** Semantic identity law  
**Depends on:** the preceding nominal identity laws

### T0757–T0759 — Nominal subtype order

Nominal subtype edges declared by the language participate in the global subtype order and compose transitively.

**Kind:** Semantic subtype laws  
**Depends on:** T0100–T0103

### T0760 — `NominalStructuralCompatibilityNotIdentity`

A nominal type may satisfy a structural contract without becoming identical to that structure.

**Kind:** Cross-model semantic law  
**Depends on:** T0705–T0709

### T0761 — `TransparentNominalAliasIdentity`

A truly transparent alias does not introduce a new semantic type identity.

**Kind:** Semantic identity law  
**Depends on:** T0001–T0003

### T0762 — `NominalWrapperDistinct`

A nominal wrapper introduces distinct identity unless the language explicitly declares it transparent.

**Kind:** Semantic identity law  
**Depends on:** the nominal identity laws and T0761

### T0763 — `NominalGenericDeclarationIdentity`

Nominal generic applications preserve constructor declaration identity. This law is restricted to nominal constructors and does not apply to arbitrary reducible type functions.

**Kind:** Nominal identity law  
**Depends on:** the nominal identity laws above

### T0764 — `NominalSubtypeDoesNotImplyIdentity`

A nominal subtype relation does not by itself collapse two distinct nominal types. The declaration may be the same for covariant generic specializations, so distinctness cannot be inferred from declaration inequality.

**Kind:** Semantic subtype/identity boundary law  
**Depends on:** T0103

### T0765 — `NominalTypeNotNever`

Any inhabited nominal type is distinct from `Never`.

**Kind:** Derived semantic law  
**Depends on:** T0152–T0154

### T0766 — `NominalTypeNotAny`

A nominal type is not `Any` when there exists some proper type not beneath it.

**Kind:** Derived semantic law  
**Depends on:** T0201–T0203

## Explicitly deferred laws

- trait/interface conformance evidence
- class/metaclass reflection
- sealed/final nominal domains
- nominal representation transparency
- ABI/layout identity
- object/class distinction
