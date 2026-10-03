# Structural Sums

**Status:** Normative  
**Semantic ownership:** Sum capability, Variant reflection, variant families,
specialization, declaration order, and the relationship between sums and product
payload shapes.

## 1. Sum capability

Every enumerated `data` declaration is a Sum.

```telos
trait Sum {
    type Variants: Variants
}
```

`Sum` is a compiler-derived sealed structural capability. Ordinary user `impl`
declarations cannot manufacture Sum identity for a non-sum declaration.

A Product has one product payload shape. A Sum has a finite declaration-ordered
sequence of structural alternatives, and every alternative has a `ProductShape`.

## 2. Variant reflection

```telos
intrinsic data Variant(
    _ name: Symbol,
    _ shape: ProductShape,
)

type Variants =
    PositionalPack<Variant>

type VariantNames =
    PositionalPack<Symbol>
```

`Variants` is an ordinary positional Pack, not a special reflection collection.

Variant order is declaration order.

Every Variant has a ProductShape, including nullary alternatives.

## 3. Variant names are families

A Variant Symbol is not required to be unique within a Sum.

```telos
data Value {
    Number(_ value: Int)
    Number(_ value: Float)
}
```

reflects two distinct Variants named `#Number`.

A Symbol therefore identifies a variant family. Construction, matching, or dispatch
that names a family resolves among the exact candidates using their structural payload
shapes and the existing canonical variant-family machinery.

Reflection never collapses repeated names.

## 4. Specialization

Variant reflection specializes with the owning Sum.

For:

```telos
data Result<T, E> {
    Ok(_ value: T)
    Err(_ error: E)
}
```

`Result<T,E>::Variants` contains Type references to symbolic `T` and `E`.
`Result<Int,String>::Variants` contains the corresponding specialized payload types.

Reflection uses canonical semantic specialization rather than textual substitution.

## 5. GADT boundary

C4 reflects ordinary structural Variant information only:

```text
name
product payload shape
```

GADT branch equality/proof metadata remains owned by semantic match analysis and is not
part of public Variant reflection in C4.

Static matching still receives the normal branch proof environment when a GADT pattern
selects a Variant.

## 6. Source-defined operations

Because `Variants` is a Pack, ordinary static Telos can define operations such as
filtering, counting, mapping, and name selection.

```telos
type VariantsNamed<
    Vs: Variants,
    Name: Symbol,
>: Variants {
    let mut result = Pack()

    for variant in Vs {
        match variant {
            Variant(name, _) =>
                if name == Name {
                    result = Pack(*result, variant)
                }
        }
    }

    result
}
```

Zero, one, or many matches are represented naturally by the resulting Pack.

## 7. Reflection recursion safety

A Variant shape describes only the immediate payload.

If a payload entry has a recursive Type, the reflected entry stores that Type value;
it does not recursively expand the Type's own Sum/Product reflection.

## 8. Invariants

- **SUM-DERIVED** — Sum capability is compiler-derived and unforgeable by ordinary
  user implementations.
- **SUM-ORDER** — Variants preserve declaration order.
- **SUM-FAMILY** — repeated Symbol names remain distinct exact Variants.
- **SUM-PACK** — Variants uses ordinary Pack semantics.
- **SUM-NOMINALITY** — reflected payload shape does not erase owning Sum identity.
- **SUM-GADT-BOUNDARY** — GADT proof metadata remains match-owned in C4.
