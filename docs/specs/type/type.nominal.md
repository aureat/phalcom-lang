# Phalcom Type Algebra Registry — `type.nominal`

**Status:** Normative specification  
**Registry family:** `type.nominal`  
**Identifier range used here:** `T0230–T0244`

## Purpose

This file defines the semantic laws of nominal type identity and nominal subtyping.

Nominal types derive identity from declaration identity rather than from accidental structural equality. Two distinct nominal declarations remain distinct semantic types unless the language explicitly introduces an equivalence relation between them.

Structural views, interfaces, traits, and public record-like projections may participate in subtyping, but they do not erase nominal identity.

## Domain and notation

Unless stated otherwise:

- `decl(T)` denotes the nominal declaration that defines `T`.
- `shape(T)` denotes a structural view of `T` when one exists.
- `A`, `B`, and `C` denote proper nominal types.
- `==` means semantic type identity.
- `<:` means semantic subtype.

## Laws

```phalcom
@id(T0230)
@name(type.nominal.kind)
NominalTypeKind<A> {
    kind(A) == Type
}

@id(T0231)
@name(type.nominal.identity.declaration)
NominalIdentityByDeclaration<A, B> {
    if decl(A) == decl(B) {
        A == B
    }
}

@id(T0232)
@name(type.nominal.distinct-declarations)
DistinctNominalDeclarations<A, B> {
    if decl(A) != decl(B) {
        A != B
    }
}

@id(T0233)
@name(type.nominal.structure.not-identity)
StructuralEqualityDoesNotImplyNominalIdentity<A, B> {
    if shape(A) == shape(B) and decl(A) != decl(B) {
        A != B
    }
}

@id(T0234)
@name(type.nominal.identity.implies-declaration)
NominalIdentityImpliesSameDeclaration<A, B> {
    if A == B {
        decl(A) == decl(B)
    }
}

@id(T0235)
@name(type.nominal.subtype.reflexive)
NominalSubtypeReflexive<A> {
    A <: A
}

@id(T0236)
@name(type.nominal.subtype.declared-parent)
DeclaredNominalParentSubtype<A, B> {
    if declaredParent(A, B) {
        A <: B
    }
}

@id(T0237)
@name(type.nominal.subtype.transitive)
NominalSubtypeTransitive<A, B, C> {
    if A <: B and B <: C {
        A <: C
    }
}

@id(T0238)
@name(type.nominal.structure.compatibility-not-identity)
NominalStructuralCompatibilityNotIdentity<A, S> {
    if shape(A) <: S {
        A <: S

        // does not imply:
        // A == S
    }
}

@id(T0239)
@name(type.nominal.alias.identity)
TransparentNominalAliasIdentity<A, B> {
    if transparentAlias(A, B) {
        A == B
    }
}

@id(T0240)
@name(type.nominal.wrapper.distinct)
NominalWrapperDistinct<A, B> {
    if wraps(A, B) and not transparentAlias(A, B) {
        A != B
    }
}

@id(T0241)
@name(type.nominal.generic.declaration-identity)
NominalGenericDeclarationIdentity<F, G, A> {
    if decl(F) != decl(G) {
        F<A> != G<A>
    }
}

@id(T0242)
@name(type.nominal.inheritance.identity-preserved)
InheritanceDoesNotCollapseIdentity<A, B> {
    if A <: B and A != B {
        decl(A) != decl(B)
    }
}

@id(T0243)
@name(type.nominal.never.distinct)
NominalTypeNotNever<A> {
    if inhabited(A) {
        A != Never
    }
}

@id(T0244)
@name(type.nominal.any.distinct)
NominalTypeNotAny<A> {
    if exists B where not (B <: A) {
        A != Any
    }
}
```

## Law notes

### T0230 — `NominalTypeKind`

A nominal type is a proper type.

**Kind:** Semantic law  
**Depends on:** T0018

### T0231 / T0232 — Declaration identity

Nominal identity is keyed by declaration identity. Distinct declarations remain distinct semantic types.

**Kind:** Semantic identity laws  
**Depends on:** T0001–T0003

### T0233 — `StructuralEqualityDoesNotImplyNominalIdentity`

Two nominal declarations do not become identical merely because their visible structure coincides.

**Kind:** Semantic identity law  
**Depends on:** T0231, T0232

### T0234 — `NominalIdentityImpliesSameDeclaration`

Semantic nominal identity implies common declaration identity.

**Kind:** Semantic identity law  
**Depends on:** T0231, T0232

### T0235–T0237 — Nominal subtype order

Nominal subtype edges declared by the language participate in the global subtype order and compose transitively.

**Kind:** Semantic subtype laws  
**Depends on:** T0020–T0023

### T0238 — `NominalStructuralCompatibilityNotIdentity`

A nominal type may satisfy a structural contract without becoming identical to that structure.

**Kind:** Cross-model semantic law  
**Depends on:** T0205–T0209

### T0239 — `TransparentNominalAliasIdentity`

A truly transparent alias does not introduce a new semantic type identity.

**Kind:** Semantic identity law  
**Depends on:** T0001–T0003

### T0240 — `NominalWrapperDistinct`

A nominal wrapper introduces distinct identity unless the language explicitly declares it transparent.

**Kind:** Semantic identity law  
**Depends on:** T0232, T0239

### T0241 — `NominalGenericDeclarationIdentity`

Generic applications preserve constructor declaration identity. Two different generic declarations do not collapse merely because they are supplied the same arguments.

**Kind:** Cross-family identity law  
**Depends on:** T0232 and `type.generic`

### T0242 — `InheritanceDoesNotCollapseIdentity`

Subtype inheritance does not imply identity.

**Kind:** Semantic subtype/identity boundary law  
**Depends on:** T0023, T0232

### T0243 — `NominalTypeNotNever`

Any inhabited nominal type is distinct from `Never`.

**Kind:** Derived semantic law  
**Depends on:** T0032–T0034

### T0244 — `NominalTypeNotAny`

A nominal type is not `Any` when there exists some proper type not beneath it.

**Kind:** Derived semantic law  
**Depends on:** T0041–T0043

## Explicitly deferred laws

- trait/interface conformance evidence
- class/metaclass reflection
- sealed/final nominal domains
- nominal representation transparency
- ABI/layout identity
- object/class distinction
