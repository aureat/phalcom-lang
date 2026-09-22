# Type-Level Domains

**Status:** Normative
**Language area:** Kinds, type-level forms, structural domains, compile-time arguments
**Specification role:** Authoritative specification for Phalcom type-level domains and parameter-domain classification

This specification defines the domains in which type-level forms and restricted
compile-time arguments are classified. It does not define generic conformance,
trait evidence, projection lookup, structural product operations, or any Rust,
arena, metadata, or solver representation.

## 1. Kinds and domain judgments

Phalcom distinguishes a type-level kind from a compile-time value domain. A
kinding judgment has the form:

```text
Γ ⊢ A : κ
```

The initial kind grammar is:

```text
κ ::= Type
    | RecordRow
    | TypePack
    | LabeledTypePack
    | StaticDomain(D)
    | (κ₁, ..., κₙ) -> κ
```

The domains `TypePack` and `LabeledTypePack` are type-level domains. They are
not runtime classes, proper Tuple/Record types, or aliases for those types.
`RecordRow` is a separate unordered field domain. `StaticDomain(D)` is the
domain of canonical static terms belonging to the nominal type-data domain
`D`. A form classified in one domain is not implicitly reinterpreted as a form
in another domain.

Examples:

```text
Γ ⊢ Int : Type
Γ ⊢ Option : Type -> Type
Γ ⊢ P : TypePack
Γ ⊢ L : LabeledTypePack
Γ ⊢ R : RecordRow
Γ ⊢ Shape::Scalar : StaticDomain(Shape)
```

`Option` in the second example is a type-level form of function kind. It is not
a proper type until applied to an argument of kind `Type`.

## 2. Proper types and type forms

A **proper type** is a type-level form of kind `Type`. A higher-kinded form may
be valid without being a proper type:

```text
Option       : Type -> Type
Option<Int>  : Type
```

Kind application and partial application are governed by ordinary type
formation. This specification does not make runtime class identity the source
of kind identity, and it does not require a distinct runtime object for any
kinded form.

`Shape::Scalar : StaticDomain(Shape)` is a static term, not a proper runtime
type and not a runtime value. Static domains and their terms remain compiler-
owned semantic forms.

## Type-data declarations

A `type data` declaration introduces one nominal static domain and its closed
set of static cases. It may be declared at module scope or as an associated
declaration of an eligible nominal owner. Class-owned declarations appear in
the class body. For a runtime `data` owner, the representation body remains
limited to its components or cases; an associated `type data` declaration is
written in an unguarded inherent `impl` for that owner in the owner's module.

The associated declaration is semantically owned by the target declaration,
not by the source `impl` fragment. The `impl` contributes source provenance
only. A conformance `impl`, a guarded inherent `impl`, or a partial-target
inherent `impl` does not introduce an associated static domain. Associated
declaration identity, duplicate rules, and namespace collisions are defined
by [Type Members](type-members.md); inherent-implementation placement and
authorization are defined by [Implementation Declarations](objects/impl.md).

An associated static domain has no runtime inhabitants or invocation target.
Its cases denote static terms in that domain. Specializing a generic owner uses
the canonical owner arguments to produce the corresponding concrete static
domain; it does not create a new source declaration.

## 3. Compile-time value domains

Restricted compile-time values use a separate judgment:

```text
Γ ⊢ c :const τ
```

The initial LANG007 surface requires only the forms needed by type-member
arguments such as `At<1>`:

- integer literals;
- references to compile-time parameters;
- signed integer values, including unary negation when the syntax is lowered
  into a signed constant term.

The initial calculus does not include arbitrary arithmetic, method calls,
runtime variables, mutation, effects, general blocks, or unrestricted const
evaluation.

Examples:

```text
Γ ⊢ 0 :const Int
Γ ⊢ -1 :const Int
Γ ⊢ I :const Int       when Domain(I) = Const(Int)
```

`Int` remains a proper type of kind `Type`; it is not a kind merely because it
appears in a parameter-domain annotation. A static term may have a non-`Type`
kind such as `StaticDomain(Shape)` and therefore cannot be wrapped in
`ProperTypeId` or materialized through the runtime object model.

## 4. Generic parameter domains

Generic parameter classification has two semantic forms:

```text
GenericParameterDomain ::= Kind(κ) | Const(τ)
```

Consequently:

```phalcom
T: Type
P: TypePack
F: Type -> Type
I: Int
```

means, respectively:

```text
Domain(T) = Kind(Type)
Domain(P) = Kind(TypePack)
Domain(F) = Kind(Type -> Type)
Domain(I) = Const(Int)
```

The colon in these binder annotations classifies the parameter domain. It does
not turn a proper type into a kind and does not introduce trait conformance.
Trait requirements remain expressed with `|=` as specified by
[Generic Constraints](generic-constraints.md).

The general domain model is available to later generic forms, but LANG007's
initial required heterogeneous arguments are type-member applications and
structural type computation. General nominal or trait const generics are not
introduced by this specification.

## 5. Generic arguments

Where a type-member or structural operation accepts a generic argument, the
argument is checked in its declared domain. Conceptually, argument categories
include:

```text
GenericArgument ::= TypeForm | RecordRow | TypePack | LabeledTypePack | Const
```

`TypeForm` includes both proper and higher-kinded forms. Thus `Map<Option>` is
valid when `Option : Type -> Type`, while `Map<Int>` is invalid because
`Int : Type`.

## 6. Symbolic forms and validity

Generic domains include abstract parameters and valid symbolic operations over
them. A concrete array or map is not the complete language meaning of a pack or
row. Later normalization may reduce a symbolic form, but lack of concrete
information does not make a well-formed form `Dynamic`, `Never`, or an error.

The semantic outcomes of a domain operation are distinct:

```text
Reduced   valid and normalized
Stuck     valid but not reducible with available information
Invalid   violates a domain, arity, or operation contract
```

An implementation may use additional internal states, but those states must
preserve this semantic distinction.

## 7. Invariants

| ID | Requirement |
|---|---|
| **TL-DOMAIN-SEPARATION** | `Type`, `RecordRow`, `TypePack`, and `LabeledTypePack` are distinct domains. |
| **TL-CONST-NOT-KIND** | `I: Int` classifies `I` in `Const(Int)`; `Int` does not become a kind. |
| **TL-PROPER-TYPE** | A proper type has kind `Type`; higher-kinded forms are not proper types until saturated. |
| **TL-STATIC-DOMAIN** | A `type data` declaration owns one nominal `StaticDomain(D)` and its canonical case/constructor terms. Distinct declarations have distinct domains. |
| **TL-STATIC-RUNTIME-SEPARATION** | Static-domain terms are compiler-owned type-level forms; they are not ordinary runtime values and do not require VM static-term execution. |
| **TL-SYMBOLIC-DOMAIN** | Valid abstract packs, rows, constants, and operations remain symbolic rather than being approximated as unrelated types. |
| **TL-CLOSED-ARGUMENTS** | Initial heterogeneous non-`Type` source arguments are required for type-member and structural applications, not for all nominal applications. |

## 8. Ownership and boundaries

Generic conformance propositions and evidence remain owned by
[Generic Constraints](generic-constraints.md). Trait-owned associated
declarations are specialized by [Associated Types](associated-types.md) and
[Type Members](type-members.md). Tuple/Record formation and `Positionals`,
`Labeled`, `At`, and `Map` are owned by
[Structural Products](structural-products.md).

This specification does not introduce runtime `TypePack` or `RecordRow`
objects, arbitrary compile-time Phalcom execution, open type families, or a
general theorem prover.
