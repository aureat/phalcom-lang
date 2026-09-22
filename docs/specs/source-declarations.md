# Source Declarations, Access Domains, and Implementation Origin

This document is the authoritative source-language specification for declaration
modifiers, member visibility, implementation origin, and the migration boundary
for legacy implementation namespaces.

## 1. Declaration modifiers

The language reserves these declaration keywords:

```text
native foreign intrinsic private protected module package internal
```

There is no `public` keyword. A declaration without a visibility modifier is
`Public`.

At most one visibility modifier and at most one implementation modifier may
precede a declaration. `native`, `foreign`, and `intrinsic` are mutually
exclusive implementation modifiers. The authored order is canonical: when both
are present, visibility must precede implementation (e.g. `private native`,
`internal intrinsic`, not `intrinsic internal`). The normalized products remain
orthogonal after parsing, but noncanonical source permutations are rejected.

```phalcom
native class Buffer {
  private _storage: Bytes
  module _cursor: Int
  package _scratch: ScratchState

  internal native rawCapacity -> Int
  package normalizeIndex(_ index: Int) -> Int { ... }
  at(_ index: Int) -> Byte { ... }
}

intrinsic class Tuple {
  intrinsic type Positionals: TypePack
  intrinsic type Labeled: LabeledTypePack
  intrinsic type At<I: Int>: Type
  intrinsic type Map<F: Type -> Type>: Type
}
```

`@primitive` remains a metadata attribute because it describes native/Universe
metadata, not declaration visibility or implementation origin. The source
attribute `@intrinsic` is retired; `intrinsic` is the canonical implementation
keyword.

## 2. Canonical semantic products

Parsing normalizes source into the shared `Visibility` and `ImplementationKind`
categories:

| Source fact | Normalized value |
| --- | --- |
| no visibility modifier | `Public` |
| `private`, `protected`, `module`, `package`, `internal` | corresponding visibility |
| no implementation modifier | `Phalcom` |
| `native`, `foreign`, `intrinsic` | corresponding implementation kind |

Names identify declarations. No selector, type name, or field spelling implies
visibility, implementation origin, or runtime privilege.

## 3. Access relation

Accessibility is evaluated from the declaration's owner/provenance and the
actual caller context:

| Visibility | Access rule |
| --- | --- |
| `Public` | any caller |
| `Private` | the declaring owner identity |
| `Protected` | the declaring owner and eligible subclasses |
| `Module` | the exact declaring module |
| `Package` | the exact declaring package |
| `Internal` | the exact declaring project identity |

`internal` is not synonymous with Universe and does not use `is_universe` as
ordinary authorization. Universe provenance remains an independent fact for
bootstrap, primordial identities, and Universe-only metadata legality.

The same relation governs direct, cached, dynamic, reflective, and native
invocation. Native execution inherits the authority of the Phalcom declaration
it implements; a native implementation does not receive blanket internal
privilege.

## 4. Fields and selectors

Fields use ordinary source-field identity. A field's visibility is explicit in
the declaration model and defaults to `Public`; implementations that want
restricted storage must write the corresponding modifier. Selector identity is
the ordinary selector identity produced by the signature, independent of
visibility and implementation origin.

The authored `_$name` implementation-selector namespace is retired. The general
`__name` implementation-field namespace is also retired. Such spellings must
not be normalized into `Internal` or another privileged category. Standardized
protocol identifiers such as `__name__` remain unchanged and retain their
existing protocol semantics.

## 5. Native, foreign, and intrinsic declarations

`native` identifies a declaration whose live implementation is supplied by the
host Phalcom runtime/VM or native primitive subsystem rather than by a Phalcom
source body. The host subsystem depends on the declaration category, and the
source/native descriptor must agree on the declaration's identity,
signature/domain, visibility, and owner provenance. It does not itself grant
access, install a primitive, or change a selector.

`foreign` is a distinct implementation category reserved for the bounded
foreign-declaration scaffold. It is not routed through the native descriptor
surface and does not imply a third-party loader or ABI. Until such a backend is
specified, a foreign declaration is metadata and a declaration boundary, not an
implicit execution mechanism.

`intrinsic` identifies a declaration whose implementation is supplied directly
by canonical compiler or semantic intrinsic semantics for a declaration
category that admits intrinsic implementation.

For intrinsic type members:

```phalcom
intrinsic type Map<F: Type -> Type>: Type
```

the declaration is bodyless because its semantic evaluator is supplied by the
compiler/semantic type normalizer; it does not execute VM dispatch, create a
runtime method, or enter native-call dispatch. A bodyless trait type requirement
is bodyless because it is a requirement, not because it is intrinsic.

The existence of `intrinsic` as a lexical modifier does not mean every
declaration category admits intrinsic implementation; target legality is
defined by each owning declaration grammar and compiler rule.

## 6. Built-in metadata attributes

`@primitive` is a metadata-only attribute for canonical Universe native
declarations. It does not change visibility, implementation kind, selector
identity, field identity, lowering, bytecode, representation, or runtime
authorization merely by being present. Its authored use is legal only in the
canonical Universe project; use outside Universe is a compile-time error.

The source attribute `@intrinsic` is retired. It is no longer a builtin
attribute, receives no compiler attribute expansion, and must not be used to
declare implementation kind.

The retired `@native`, `@private`, `@protected`, `@internal`, and `@intrinsic`
forms are not declaration semantics. They must not populate `Visibility` or
`ImplementationKind`, alter lowering, or authorize access. A parser/compiler may
report them as migration diagnostics, but they are otherwise ordinary unknown
attributes or retired syntax.

## 7. Reflection and tooling

Reflection and editor products that expose declaration metadata must consume the
structured products above. They may present:

```text
visibility: public | private | protected | module | package | internal
implementation: phalcom | native | foreign | intrinsic
```

The `public` word is presentation text for the implicit semantic state, not a
source keyword. Syntax tooling highlights the eight declaration keywords and
the metadata attribute `@primitive`; it does not highlight `_$` or general
`__field` spellings as active implementation namespaces.
