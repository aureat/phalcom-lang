# Source Declarations, Access Domains, and Implementation Origin

This document is the authoritative source-language specification for declaration
modifiers, member visibility, implementation origin, and the migration boundary
for legacy implementation namespaces.

## 1. Declaration modifiers

The language reserves these declaration keywords:

```text
native foreign private protected module package internal
```

There is no `public` keyword. A declaration without a visibility modifier is
`Public`.

At most one visibility modifier and at most one implementation modifier may
precede a declaration. `native` and `foreign` are mutually exclusive. Modifier
order is otherwise insignificant to the normalized declaration product.

```phalcom
native class Buffer {
  private _storage: Bytes
  module _cursor: Int
  package _scratch: ScratchState

  internal native rawCapacity -> Int
  package normalizeIndex(_ index: Int) -> Int { ... }
  at(_ index: Int) -> Byte { ... }
}
```

`@primitive` and `@intrinsic` remain attributes because they describe metadata,
not declaration visibility or implementation origin.

## 2. Canonical semantic products

Parsing normalizes source into the shared `Visibility` and `ImplementationKind`
categories:

| Source fact | Normalized value |
| --- | --- |
| no visibility modifier | `Public` |
| `private`, `protected`, `module`, `package`, `internal` | corresponding visibility |
| no implementation modifier | `Phalcom` |
| `native`, `foreign` | corresponding implementation kind |

Names identify declarations. No selector or field spelling implies visibility,
implementation origin, or runtime privilege.

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

## 5. Native and foreign declarations

`native` identifies a declaration whose live implementation is supplied by the
native runtime and whose source/native descriptor must agree on selector,
signature, visibility, and owner provenance. It does not itself grant access,
install a primitive, or change the selector.

`foreign` is a distinct implementation category reserved for the bounded
foreign-declaration scaffold. It is not routed through the native descriptor
surface and does not imply a third-party loader or ABI. Until such a backend is
specified, a foreign declaration is metadata and a declaration boundary, not an
implicit execution mechanism.

## 6. Built-in metadata attributes

`@primitive` and `@intrinsic` are metadata-only attributes. They do not change
visibility, implementation kind, selector identity, field identity, lowering,
bytecode, representation, or runtime authorization merely by being present.
Their authored use is legal only in the canonical Universe project; use outside
Universe is a compile-time error.

The retired `@native`, `@private`, `@protected`, and `@internal` forms are not
declaration semantics. They must not populate `Visibility` or
`ImplementationKind`, alter lowering, or authorize access. A parser/compiler may
report them as migration diagnostics, but they are otherwise ordinary unknown
attributes or retired syntax.

## 7. Reflection and tooling

Reflection and editor products that expose declaration metadata must consume the
structured products above. They may present:

```text
visibility: public | private | protected | module | package | internal
implementation: phalcom | native | foreign
```

The `public` word is presentation text for the implicit semantic state, not a
source keyword. Syntax tooling highlights the seven declaration keywords and
the metadata attributes `@primitive` and `@intrinsic`; it does not highlight
`_$` or general `__field` spellings as active implementation namespaces.
