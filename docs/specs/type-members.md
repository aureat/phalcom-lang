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
Intrinsic     supplied directly by canonical compiler/semantic normalization
Native        supplied by the host Phalcom runtime/primitive subsystem
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

An associated algebraic static domain is declared with `type data` and is a
distinct declaration category, not a type-member projection. It contributes a
domain denotation and static case terms to the owner's associated namespace.
The owner forms and placement rules are defined by
[Type-Level Domains](type-level-domains.md),
[Data Objects](objects/data-objects.md), and
[Implementation Declarations](objects/impl.md).

The source form on a class is:

```phalcom
class Parser {
    type data State {
        Start
        Done
    }
}
```

For a runtime `data` declaration, the associated declaration is written in an
unguarded inherent `impl` in the target's owning module, not in the
representation body:

```phalcom
data Packet(bytes: ByteArray)

impl Packet {
    type data State {
        Unread
        Parsed
    }
}
```

In both forms, the nominal class or data declaration is the semantic owner.
The inherent `impl` fragment is provenance only. Declaration identity is the
stable pair `(owner declaration identity, associated name)` and is unchanged by
owner specialization. A concrete associated domain is that declaration
identity specialized by the canonical generic arguments of the owner.
Duplicate associated domain names are invalid across all direct and inherent
declarations for one owner. A conformance `impl`, guarded inherent `impl`, or
partial-target inherent `impl` cannot introduce one. Associated namespace
collisions with other member categories follow the single owner-level lookup
and collision rules; source order does not select a winner.
Associated `type data` declarations have no member-local generic parameters;
their only specialization is by the canonical generic environment of the
nominal owner.

Associated static cases belong to the associated domain. They are static terms,
not runtime values, methods, or invocation targets.

The compiler-intrinsic structural declaration surface is defined by
[Structural Products](structural-products.md). Representative members include:

```telos
impl<T: Type> PositionalPack<T> {
  intrinsic type View: PositionalPackView<T>
}

impl<T: Type> LabeledPack<T> {
  intrinsic type View: LabeledPackView<T>
}

impl<T: Type> CompletePack<T> {
  intrinsic type PositionalEntries: PositionalPack<T>
  intrinsic type LabeledEntries: LabeledPack<T>
}

impl UnorderedShape {
  intrinsic type View: UnorderedShapeView
  intrinsic type Tail: UnorderedTail
  intrinsic type Lookup<K: FieldKey>: FieldLookup
}
```

Compiler-derived Product/Sum associated surfaces may likewise return ordinary static
data such as `ProductShape`, `OrderedShape`, `UnorderedShape`, and `Variants`.

The declarations live on canonical Universe anchors for discovery, documentation, and
reflection while semantic applications reuse optimized canonical compiler terms.
Historical internal TypePack/RecordRow representations do not create independent public
member domains.

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
automatically acquire structural `Map`; a parameter known as `P: PositionalTypePack` may use the operations exposed for positional Packs.

Lookup must not choose a candidate by source order, spelling precedence, or
runtime class. Member declaration identity and applicable semantic evidence
resolve the candidate.

## 5. Application and result domains

Member arguments are checked against the member declaration's parameter domains.
The result is classified by the declaration's result domain. In particular:

```text
View                         : PositionalPackView<T> / LabeledPackView<T>
PositionalEntries            : PositionalPack<T>
LabeledEntries               : LabeledPack<T>
UnorderedShape::Lookup<K>    : FieldLookup
Product::Shape               : ProductShape
Sum::Variants                : Variants
At<I>                        : Type
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
The two spellings must not declare independent members. The canonical semantic
identity is the type-level Subscript member; `At` is a standardized named
presentation alias for that same member identity.

## 7. Projection states

A well-formed projection may be:

```text
Reduced   the member operation has a canonical result
Stuck     the operation is valid but an abstract subject/argument prevents reduction
Invalid   a concrete domain, arity, lookup, or operation obligation fails
```

For example, `P::View` is a valid symbolic projection when `P: PositionalPack<Type>`, even when `P` is abstract. Conversely, a concrete
out-of-range `At` index or an argument of the wrong kind is invalid. A stuck
projection must not be silently changed to `Dynamic`, `Never`, or an unrelated
proper type.

Normalization is semantic. It must preserve symbolic structure and cannot use
runtime VM dispatch, materialize runtime values, or execute arbitrary source
code. Internal cancellation, cycle, and budget states may exist, but they must
not change the language meaning of a valid symbolic projection.

### Declaration-signature dependency cycles

Transparent type aliases, nominal or trait generic headers, and source type-member
declarations may depend on one another while their declared forms, parameter
domains, and result domains are checked. A declaration-signature dependency cycle
containing one or more source type-member declarations is invalid, including when
the cycle also contains a generic header. Every declaration participating in such
a cycle remains unavailable as a checked semantic product, and the compiler reports
a deterministic type-signature-cycle diagnostic identifying the participating
declarations. Existing alias-only and generic-header-only cycle behavior is
unchanged by this rule. Editing any declaration in the cycle requires rechecking
the affected cycle component and its dependents.

This rule concerns declaration formation, not projection normalization. A
well-formed symbolic projection remains valid and may remain stuck as described
above; it is not rejected merely because normalization encounters a recursive
or unavailable reduction.

## 8. Intrinsic type members

`intrinsic` identifies declarations whose implementation is provided directly
by the compiler/semantic normalizer:

> An intrinsic declaration is implemented by canonical compiler/semantic
> normalization rather than by a Phalcom source body or host native binding.

An intrinsic type member does not execute a VM frame, materialize a Tuple or
Record value, send a runtime message, or enter native-call dispatch.

The source attribute `@intrinsic` is retired and must not be used to declare
implementation kind. Internal `NativeIntrinsicId` metadata (used for compiler/runtime
optimizations on selected native primitives) is separate from source implementation
kind and does not alter type-member semantics.

Intrinsic type declarations remain subject to visibility, owner, and canonical
declaration identity rules.

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
not thereby an intrinsic or native declaration. A bodyless `intrinsic type` is
bodyless because its compiler/semantic normalizer supplies the result.

## 10. Structural and reflection members

Structural members are specified by [Structural Products](structural-products.md) and
[Structural Sums](structural-sums.md).

They may return proper Types or ordinary static data domains. Availability is based on
canonical semantic subject/domain facts and compiler-derived capability evidence.

Representative operations include:

```text
PositionalPack<T>  View
LabeledPack<T>     View
CompletePack<T>    PositionalEntries, LabeledEntries
UnorderedShape     View, Tail, Lookup<K>
Product            Kind, Shape
OrderedProduct     OrderedShape, Entries, PositionalEntries, LabeledEntries
UnorderedProduct   UnorderedShape
Sum                Variants
```

A source declaration may provide a Universe/documentation/reflection anchor, but such an
anchor does not create a competing semantic member database.

## 11. Invariants

| ID | Requirement |
|---|---|
| **TM-DECLARATION-IDENTITY** | Type-member identity is declaration/domain identity, not spelling identity. |
| **TM-PROJECTION-DOMAIN** | Projection results may be `Type` or any declared static result domain, including Pack/Product/Sum reflection data. |
| **TM-PARAMETERS-LOCAL** | Member-local parameters belong to the member application. |
| **TM-PROJECTION-SYMBOLIC** | A valid unreduced projection remains symbolic. |
| **TM-INVALID-DISTINCT-FROM-STUCK** | Concrete invalidity and lack of reduction information are distinct outcomes. |
| **TM-SUBSCRIPT-AT-IDENTITY** | `T[I]` and `T::At<I>` are one semantic operation with Subscript as canonical identity and `At` as presentation alias. |
| **TM-INTRINSIC-NO-VM** | Intrinsic type-member normalization is compiler/semantic work and never runtime VM dispatch. |
| **TM-STATIC-BOUNDARY** | Intrinsic/domain-native member normalization remains compiler-defined; reusable source static computation is expressed through the static `type` model from `static-evaluation.md`, not runtime dispatch. |
