@!documentation("Normative Product, Sum, and structural reflection declarations.")

from universe.collections.pack import PositionalPack, LabeledPack, CompletePack
from universe.collections.record import FieldKey
from universe.reflection.typing.kind import Type

trait Product {
    type Kind: ProductKind
    type Shape: ProductShape
}

trait OrderedProduct {
    type OrderedShape: OrderedShape
    type Entries: CompletePack<Type>
    type PositionalEntries: PositionalPack<Type>
    type LabeledEntries: LabeledPack<Type>
}

trait UnorderedProduct {
    type UnorderedShape: UnorderedShape
}

intrinsic data ProductKind {
    Ordered
    Unordered
}

intrinsic data ProductShape {
    Ordered(_ shape: OrderedShape)
    Unordered(_ shape: UnorderedShape)
}

intrinsic data OrderedTail<Rest> {
    Closed
    Typed(_ valueType: Type)
    Unconstrained
    Captured(_ rest: Rest)
}

type PositionalTail = OrderedTail<PositionalPack<Type>>
type LabeledTail = OrderedTail<LabeledPack<Type>>

intrinsic data OrderedShape(
    _ entries: CompletePack<Type>,
    _ positionalTail: PositionalTail,
    _ labeledTail: LabeledTail,
)

intrinsic data Presence {
    Required
    Optional
}

intrinsic data FieldInfo(
    _ valueType: Type,
    _ presence: Presence,
)

intrinsic data ProductField(
    _ key: FieldKey,
    _ valueType: Type,
    _ presence: Presence,
)

type FixedFields = { **[FieldKey]: FieldInfo }

intrinsic data UnorderedTail {
    Closed
    Typed(_ keyType: Type, _ valueType: Type)
    Unconstrained(_ keyType: Type)
    Captured(_ rest: UnorderedShape)
}

intrinsic data UnorderedShape(
    _ fields: FixedFields,
    _ tail: UnorderedTail,
)

intrinsic data FieldLookup {
    Required(_ valueType: Type)
    Optional(_ valueType: Type)
    Absent
    Unknown
}

intrinsic data UnorderedShapeView {
    Empty(_ tail: UnorderedTail)
    Field(_ field: ProductField, _ rest: UnorderedShape)
}

impl UnorderedShape {
    intrinsic type Lookup<K: FieldKey>: FieldLookup
    intrinsic type View: UnorderedShapeView
    intrinsic type Tail: UnorderedTail
}

trait Sum {
    type Variants: Variants
}

intrinsic data Variant(
    _ name: Symbol,
    _ shape: ProductShape,
)

type Variants = PositionalPack<Variant>
type VariantNames = PositionalPack<Symbol>

export Product, OrderedProduct, UnorderedProduct, Sum
export ProductKind, ProductShape, OrderedTail, PositionalTail, LabeledTail, OrderedShape
export Presence, FieldInfo, ProductField, FixedFields, UnorderedTail, UnorderedShape, FieldLookup, UnorderedShapeView
export Variant, Variants, VariantNames
