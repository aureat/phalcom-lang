//! Portable executable constant recipes. These rows describe immutable
//! construction inputs; they do not serialize VM objects or heap state.

use crate::module_world::{Decode, Wire, record, tagged};
use crate::{ArtifactError, BinaryWriter, ValidationLimits};
use phalcom_type_meta::identity::{StableDeclarationRef, StableDispatchSide, StableModuleRef, StableProjectRef};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeAnchorId(pub u32);
impl Wire for NativeAnchorId {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        self.0.write(out)
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        Ok(Self(u32::read(input)?))
    }
}

record!(StableClassRefV1 {
    declaration: StableDeclarationRef,
    side: StableDispatchSide,
});

tagged!(MethodSignatureKindV1 {
    0 => Method { slots: u8 },
    1 => Getter,
    2 => Setter,
    3 => SubscriptGet { slots: u8 },
    4 => SubscriptSet { slots: u8 },
});

tagged!(RestModeV1 {
    0 => Positional { param_index: u16 },
    1 => Labeled { param_index: u16 },
    2 => Split { positional_param_index: u16, labeled_param_index: u16 },
    3 => Complete { param_index: u16 },
});
record!(RestLayoutV1 {
    fixed_positionals: u8,
    fixed_labels: Vec<String>,
    mode: RestModeV1,
});
record!(MethodSignatureV1 {
    selector: String,
    kind: MethodSignatureKindV1,
    positional_arity: u8,
    rest: Option<RestLayoutV1>,
});

tagged!(VisibilityV1 {
    0 => Public,
    1 => Private,
    2 => Protected,
    3 => Module,
    4 => Package,
    5 => Internal,
});
record!(MethodOriginV1 {
    module: Option<StableModuleRef>,
    package: Option<StableModuleRef>,
    project: Option<StableProjectRef>,
});
record!(ContractRecipeV1 {
    predicate: String,
    value: super::prototype::ConstantId,
});
tagged!(MethodBodyV1 {
    0 => Closure { template: super::prototype::ConstantId },
    1 => Native { anchor: NativeAnchorId },
});
record!(MethodRecipeV1 {
    body: MethodBodyV1,
    signature: MethodSignatureV1,
    holder: Option<StableClassRefV1>,
    access_owner: Option<StableClassRefV1>,
    visibility: VisibilityV1,
    origin: MethodOriginV1,
    contracts: Option<Vec<ContractRecipeV1>>,
    attributes: Vec<super::prototype::ConstantId>,
    attributes_frozen: bool,
});

tagged!(SelectorBaseV1 {
    0 => Named { name: String },
    1 => Subscript,
});
tagged!(SelectorKindV1 {
    0 => Getter,
    1 => Setter,
    2 => Method,
    3 => SubscriptGet,
    4 => SubscriptSet,
});
tagged!(SelectorSlotV1 {
    0 => Positional,
    1 => Label { label: String },
});
tagged!(SelectorKindPatternV1 {
    0 => AnyNamed,
    1 => NamedAccessors,
    2 => AnySubscript,
    3 => Exact { kind: SelectorKindV1 },
});
record!(SelectorPatternV1 {
    base: SelectorBaseV1,
    kind: SelectorKindPatternV1,
    prefix: Vec<SelectorSlotV1>,
    suffix: Vec<SelectorSlotV1>,
    has_gap: bool,
});

tagged!(ConstantRecipeV1 {
    0 => None,
    1 => Unit,
    2 => EmptyRecord,
    3 => EmptyPack,
    4 => Boolean { value: bool },
    5 => Integer { bits: u64 },
    6 => Float { bits: u64 },
    7 => Character { scalar: u32 },
    8 => Symbol { text: String },
    9 => String { text: String },
    10 => BigInteger { negative: bool, magnitude: Vec<u8> },
    11 => Class { reference: StableClassRefV1 },
    12 => Module { module: StableModuleRef },
    13 => ClosureTemplate {
        prototype: super::prototype::PrototypeId,
        module: StableModuleRef,
        lexical_class: Option<StableClassRefV1>,
        super_anchor: Option<StableClassRefV1>
    },
    14 => Method { recipe: MethodRecipeV1 },
    15 => SelectorPattern { pattern: SelectorPatternV1 },
    16 => Vector { elements: Vec<super::prototype::ConstantId> },
    17 => FrozenArray { elements: Vec<super::prototype::ConstantId> },
    18 => Tuple {
        positionals: Vec<super::prototype::ConstantId>,
        labels: Vec<String>,
        labeled_values: Vec<super::prototype::ConstantId>
    },
    19 => Record { labels: Vec<String>, values: Vec<super::prototype::ConstantId> },
    20 => Pack {
        positionals: Vec<super::prototype::ConstantId>,
        labels: Vec<String>,
        labeled_values: Vec<super::prototype::ConstantId>
    },
});
