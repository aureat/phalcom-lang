# Type Members

**Status:** Normative
**Language area:** Type-member declarations, projection, lookup, normalization
**Specification role:** Authoritative specification for general type-member semantics

A **type member** is a declaration that produces a type-level result from a
subject and, optionally, member-local arguments. Type members are distinct
from executable methods, fields, and runtime properties. Their result domain
may be any domain defined by [Type-Level Domains](type-level-domains.md), not
only `Type`.

## 1. Type-member categories

The semantic categories are:

```text
TypeMember
    Inherent
    Associated
    DomainNative
```

These categories describe ownership. The implementation forms are a separate
axis:

```text
Requirement   supplied by conformance evidence
Alias         defined by another type-level expression
Native        supplied by the host Phalcom implementation
```

An inherent type member is owned by a nominal declaration. An associated type
member is owned by a trait and supplied by conformance evidence. A domain-native
member is owned by a compiler-defined structural domain. Domain-native members
do not reopen each anonymous structural type as an ordinary source `impl`
target.

## 2. Declaration shape

A type-member declaration has a name or presentation form, zero or more
member-local parameters, and a result domain:

```phalcom
type Element: Type
```

The exact declaration grammar is subject to the declaration forms that expose
type members, but the semantic requirements are fixed:

- member parameters belong to the member application, not the subject;
- each parameter has a domain from the type-level-domain specification;
- the result domain is declared and checked;
- omitted result-domain syntax defaults to `Type` only where the owning
  declaration form specifies that default.

The initial compiler-native structural declaration surface is:

```phalcom
native type Positionals: TypePack
native type Labeled: LabeledTypePack
native type At<I: Int>: Type
native type Map<F: Type -> Type>: Type
```

These declarations may be authored on canonical Universe anchors for discovery,
documentation, and reflection while remaining semantically owned by the
structural domain. `At<I>` is the canonical named member; type subscript is a
presentation alias of the same identity, not a second declaration.

## 3. Identity

Type-member identity is declaration/domain identity, not spelling identity. A
projection is identified conceptually by:

```text
Projection(subject, member declaration identity, member arguments, applicable evidence)
```

Two members named `Item` in different traits remain distinct. A domain-native
member named `Map` is distinct from a trait-associated `Map` even when both
have the same spelling and result domain.

The identity of an operation must not depend on whether a concrete reduction
happens to produce an equal type.

## 4. Lookup and qualification

`Subject::Name` is accepted only when semantic lookup resolves `Name` to one
accessible applicable member for the subject's known domain or evidence
context. It is not a runtime message send.

Trait-associated members remain trait-owned. If more than one trait context can
supply the same name, contextual lookup is ambiguous and explicit qualification
is required, for example:

```phalcom
<T as Trait>::Item
```

Domain-native members are available only when the subject domain is statically
known to provide them. A parameter known only as `T: Type` does not
automatically acquire structural `Map`; a parameter known as `P: TypePack` may
use the operations exposed for `TypePack`.

Lookup must not choose a candidate by source order, spelling precedence, or
runtime class. Member declaration identity and applicable semantic evidence
resolve the candidate.

## 5. Application and result domains

Member arguments are checked against the member declaration's parameter domains.
The result is classified by the declaration's result domain. In particular:

```text
Positionals : TypePack
Labeled     : LabeledTypePack
At<I>       : Type
Map<F>      : Type
```

A projection is therefore domain-neutral. It is not required to be represented
as a proper type merely because it appears in type-level syntax.

## 6. Subscript identity

When a type supports integer type-level indexing, subscript syntax and the
named `At` spelling denote one member operation:

```text
T[I] ≡ T::At<I>
```

This is semantic identity, not merely coincidental equality after normalization.
The two spellings must not declare independent members. A language surface may
provide both presentations as aliases of one declaration identity.

## 7. Projection states

A well-formed projection may be:

```text
Reduced   the member operation has a canonical result
Stuck     the operation is valid but an abstract subject/argument prevents reduction
Invalid   a concrete domain, arity, lookup, or operation obligation fails
```

For example, `P::Map<F>` is a valid symbolic projection when `P: TypePack` and
`F: Type -> Type`, even when `P` is abstract. Conversely, a concrete
out-of-range `At` index or an argument of the wrong kind is invalid. A stuck
projection must not be silently changed to `Dynamic`, `Never`, or an unrelated
proper type.

Normalization is semantic. It must preserve symbolic structure and cannot use
runtime VM dispatch, materialize runtime values, or execute arbitrary source
code. Internal cancellation, cycle, and budget states may exist, but they must
not change the language meaning of a valid symbolic projection.

## 8. Native type members

`native` identifies implementation origin, not a particular runtime subsystem:

> A native declaration is implemented by the host Phalcom implementation
> rather than by a Phalcom source body. The host subsystem depends on the
> declaration category.

For an executable member, the host implementation may be the VM/native runtime.
For a native type member, the host implementation is the compiler/semantic type
normalizer. A native type member does not execute a VM frame, materialize a
Tuple or Record value, send a runtime message, or enter ordinary native-call
dispatch.

`@intrinsic` remains metadata identifying or describing a builtin declaration;
it is not an alternate implementation-origin keyword and does not itself
define executable semantics. Native declarations remain subject to visibility,
owner, and canonical declaration identity rules.

## 9. Trait specialization

Trait-associated declarations are a specialization of this general model.
Their declaration identity and conformance-owned bindings remain governed by
[Associated Types](associated-types.md); result domains, projection substrate,
and symbolic/invalid distinctions are governed here when that category admits
them. Existing unannotated associated declarations default to result domain
`Type`. The initial associated-type surface permits explicit result domains but
remains unparameterized; member-local parameters are initially required for
compiler/domain-native members.

An associated requirement being bodyless means that it is a requirement. It is
not thereby a native declaration. A bodyless `native type` is bodyless because
its host implementation supplies the result.

## 10. Structural domain members

The following are domain-native members specified by
[Structural Products](structural-products.md):

```text
Tuple-family       Positionals, Labeled, At<I>, Map<F>
TypePack           Map<F>
LabeledTypePack    Map<F>
Record-family      Map<F>
RecordRow          Map<F>
```

Their availability is domain-based. A source declaration may provide a
Universe/documentation/reflection anchor, but such an anchor does not create a
competing semantic member database.

## 11. Invariants

| ID | Requirement |
|---|---|
| **TM-DECLARATION-IDENTITY** | Type-member identity is declaration/domain identity, not spelling identity. |
| **TM-PROJECTION-DOMAIN** | Projection results may be `Type`, `TypePack`, `LabeledTypePack`, `RecordRow`, or another declared domain. |
| **TM-PARAMETERS-LOCAL** | Member-local parameters belong to the member application. |
| **TM-PROJECTION-SYMBOLIC** | A valid unreduced projection remains symbolic. |
| **TM-INVALID-DISTINCT-FROM-STUCK** | Concrete invalidity and lack of reduction information are distinct outcomes. |
| **TM-SUBSCRIPT-AT-IDENTITY** | `T[I]` and `T::At<I>` are one semantic operation. |
| **TM-NATIVE-NO-VM** | Native type-member normalization is compiler/semantic work and never runtime VM dispatch. |
| **TM-CLOSED-CALCULUS** | Domain-native operations do not introduce arbitrary user-defined type families or arbitrary compile-time execution. |
