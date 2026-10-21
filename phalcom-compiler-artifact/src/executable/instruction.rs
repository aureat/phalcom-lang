//! Portable bytecode instructions. Decoding does not establish executable safety.
//!
//! Tags are explicit executable ABI values. Operands are indices within the
//! containing prototype/module, never process-local symbols or heap handles.
//! The executable validator must check their domains before VM materialization.

use crate::{BinaryReader, BinaryWriter, CodecError};

/// Operand authority needed by the executable validator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperandDomainV1 {
    Access,
    AnonymousProduct,
    Arity,
    AssociatedTarget,
    CallEnvironment,
    ClosureTemplateConstant,
    Component,
    ConditionalFamily,
    Constant,
    Count,
    DataConstruction,
    DataDeclaration,
    DataTarget,
    EnumDeclaration,
    FamilyCandidates,
    FamilyDescriptor,
    FamilyKind,
    FamilyOperation,
    Field,
    Flag,
    LinkedRead,
    Local,
    MethodConstant,
    ModuleConstant,
    RelativeJump,
    SelectorSpecification,
    SendKind,
    SymbolConstant,
    TraitInvocation,
    TraitRequirementInvocation,
    TypeParameterReceiver,
    Upvalue,
    VariantTarget,
}

/// Cross-table identity relocation implied by one opcode operand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelocationRequirementV1 {
    ConstantRecipe,
    LinkedModuleRead,
    SemanticPoolEntry,
}

/// Minimum full-validator obligation implied by one opcode operand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidatorRequirementV1 {
    ConstantReference,
    TypedConstantReference,
    ClosureTemplateReference,
    MethodReference,
    ModuleReference,
    LinkedReadReference,
    SelectorSpecificationReference,
    SemanticPoolReference,
    SemanticComponentReference,
    FrameSlot,
    UpvalueSlot,
    FieldSlot,
    ArityAndStackShape,
    CountAndStackShape,
    ControlFlowTarget,
    CanonicalFlag,
    OperandTag,
}

pub const fn relocation_requirement_v1(domain: OperandDomainV1) -> Option<RelocationRequirementV1> {
    use OperandDomainV1 as Domain;
    match domain {
        Domain::Constant
        | Domain::ClosureTemplateConstant
        | Domain::MethodConstant
        | Domain::ModuleConstant
        | Domain::SelectorSpecification
        | Domain::SymbolConstant => Some(RelocationRequirementV1::ConstantRecipe),
        Domain::LinkedRead => Some(RelocationRequirementV1::LinkedModuleRead),
        Domain::AnonymousProduct
        | Domain::AssociatedTarget
        | Domain::CallEnvironment
        | Domain::Component
        | Domain::ConditionalFamily
        | Domain::DataConstruction
        | Domain::DataDeclaration
        | Domain::DataTarget
        | Domain::EnumDeclaration
        | Domain::FamilyCandidates
        | Domain::FamilyDescriptor
        | Domain::FamilyOperation
        | Domain::TraitInvocation
        | Domain::TraitRequirementInvocation
        | Domain::TypeParameterReceiver
        | Domain::VariantTarget => Some(RelocationRequirementV1::SemanticPoolEntry),
        Domain::Access
        | Domain::Arity
        | Domain::Count
        | Domain::FamilyKind
        | Domain::Field
        | Domain::Flag
        | Domain::Local
        | Domain::RelativeJump
        | Domain::SendKind
        | Domain::Upvalue => None,
    }
}

pub const fn validator_requirement_v1(domain: OperandDomainV1) -> ValidatorRequirementV1 {
    use OperandDomainV1 as Domain;
    match domain {
        Domain::Constant => ValidatorRequirementV1::ConstantReference,
        Domain::SymbolConstant => ValidatorRequirementV1::TypedConstantReference,
        Domain::ClosureTemplateConstant => ValidatorRequirementV1::ClosureTemplateReference,
        Domain::MethodConstant => ValidatorRequirementV1::MethodReference,
        Domain::ModuleConstant => ValidatorRequirementV1::ModuleReference,
        Domain::LinkedRead => ValidatorRequirementV1::LinkedReadReference,
        Domain::SelectorSpecification => ValidatorRequirementV1::SelectorSpecificationReference,
        Domain::AnonymousProduct
        | Domain::AssociatedTarget
        | Domain::CallEnvironment
        | Domain::ConditionalFamily
        | Domain::DataConstruction
        | Domain::DataDeclaration
        | Domain::DataTarget
        | Domain::EnumDeclaration
        | Domain::FamilyCandidates
        | Domain::FamilyDescriptor
        | Domain::FamilyOperation
        | Domain::TraitInvocation
        | Domain::TraitRequirementInvocation
        | Domain::TypeParameterReceiver
        | Domain::VariantTarget => ValidatorRequirementV1::SemanticPoolReference,
        Domain::Component => ValidatorRequirementV1::SemanticComponentReference,
        Domain::Local => ValidatorRequirementV1::FrameSlot,
        Domain::Upvalue => ValidatorRequirementV1::UpvalueSlot,
        Domain::Field => ValidatorRequirementV1::FieldSlot,
        Domain::Arity => ValidatorRequirementV1::ArityAndStackShape,
        Domain::Count => ValidatorRequirementV1::CountAndStackShape,
        Domain::RelativeJump => ValidatorRequirementV1::ControlFlowTarget,
        Domain::Flag => ValidatorRequirementV1::CanonicalFlag,
        Domain::Access | Domain::FamilyKind | Domain::SendKind => ValidatorRequirementV1::OperandTag,
    }
}

/// Canonical byte encoding used for one fixed-width instruction operand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperandEncodingV1 {
    U8,
    U16Le,
    U32Le,
    I32Le,
    BoolU8,
    TagU8,
}

impl OperandEncodingV1 {
    pub const fn width_bytes(self) -> u8 {
        match self {
            Self::U8 | Self::BoolU8 | Self::TagU8 => 1,
            Self::U16Le => 2,
            Self::U32Le | Self::I32Le => 4,
        }
    }
}

/// One operand's interpretation and wire width in the stable opcode manifest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperandManifestEntryV1 {
    pub domain: OperandDomainV1,
    pub encoding: OperandEncodingV1,
    pub relocation: Option<RelocationRequirementV1>,
    pub validation: ValidatorRequirementV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpcodeManifestRowV1 {
    pub tag: u16,
    pub name: &'static str,
    pub operands: &'static [OperandManifestEntryV1],
}

trait WireOperand: Sized + Copy {
    const ENCODING: OperandEncodingV1;
    fn write(self, writer: &mut BinaryWriter);
    fn read(reader: &mut BinaryReader<'_>) -> Result<Self, CodecError>;
}

macro_rules! primitive_operand {
    ($ty:ty, $write:ident, $read:ident, $encoding:ident) => {
        impl WireOperand for $ty {
            const ENCODING: OperandEncodingV1 = OperandEncodingV1::$encoding;
            fn write(self, writer: &mut BinaryWriter) {
                writer.$write(self);
            }
            fn read(reader: &mut BinaryReader<'_>) -> Result<Self, CodecError> {
                reader.$read()
            }
        }
    };
}
primitive_operand!(u8, write_u8, read_u8, U8);
primitive_operand!(u16, write_u16, read_u16, U16Le);
primitive_operand!(u32, write_u32, read_u32, U32Le);
primitive_operand!(bool, write_bool, read_bool, BoolU8);
impl WireOperand for i32 {
    const ENCODING: OperandEncodingV1 = OperandEncodingV1::I32Le;
    fn write(self, writer: &mut BinaryWriter) {
        writer.write_bytes(&self.to_le_bytes());
    }
    fn read(reader: &mut BinaryReader<'_>) -> Result<Self, CodecError> {
        Ok(Self::from_le_bytes(reader.read_fixed()?))
    }
}

macro_rules! tagged_operand {
    ($name:ident { $($variant:ident = $tag:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name { $($variant),+ }
        impl WireOperand for $name {
            const ENCODING: OperandEncodingV1 = OperandEncodingV1::TagU8;
            fn write(self, writer: &mut BinaryWriter) {
                writer.write_u8(match self { $(Self::$variant => $tag),+ });
            }
            fn read(reader: &mut BinaryReader<'_>) -> Result<Self, CodecError> {
                match reader.read_u8()? {
                    $($tag => Ok(Self::$variant),)+
                    tag => Err(CodecError::InvalidTag { type_name: stringify!($name), tag: u32::from(tag) }),
                }
            }
        }
    };
}
tagged_operand!(PortableFamilyKindV1 { Exact = 0, Pattern = 1 });
tagged_operand!(PortableSendKindV1 { Method = 0, SubscriptGet = 1, SubscriptSet = 2 });
tagged_operand!(PortableAccessV1 { Ordinary = 0, CompilerInternal = 1 });

#[cfg(test)]
trait TestOperand: Copy {
    const SAMPLE: Self;
}

#[cfg(test)]
macro_rules! test_operand_samples {
    ($($ty:ty => $sample:expr),+ $(,)?) => {
        $(impl TestOperand for $ty { const SAMPLE: Self = $sample; })+
    };
}

#[cfg(test)]
test_operand_samples!(
    u8 => 0,
    u16 => 0,
    u32 => 0,
    i32 => -17,
    bool => true,
    PortableFamilyKindV1 => PortableFamilyKindV1::Pattern,
    PortableSendKindV1 => PortableSendKindV1::SubscriptSet,
    PortableAccessV1 => PortableAccessV1::CompilerInternal,
);

macro_rules! instructions {
    ($($tag:literal => $name:ident { $($field:ident: $ty:ty => $domain:ident),* $(,)? }),+ $(,)?) => {
        /// One decoded instruction. References are unvalidated until the full
        /// executable image has passed its cross-table and control-flow checks.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum PortableInstructionV1 { $($name { $($field: $ty),* }),+ }

        pub const OPCODE_MANIFEST_V1: &[OpcodeManifestRowV1] = &[
            $(OpcodeManifestRowV1 {
                tag: $tag,
                name: stringify!($name),
                operands: &[$(OperandManifestEntryV1 {
                    domain: OperandDomainV1::$domain,
                    encoding: <$ty as WireOperand>::ENCODING,
                    relocation: relocation_requirement_v1(OperandDomainV1::$domain),
                    validation: validator_requirement_v1(OperandDomainV1::$domain),
                }),*],
            }),+
        ];

        impl PortableInstructionV1 {
            pub fn opcode_tag(&self) -> u16 { match self { $(Self::$name { .. } => $tag),+ } }

            #[cfg(test)]
            fn test_variants() -> Vec<Self> {
                vec![$(Self::$name { $($field: <$ty as TestOperand>::SAMPLE),* }),+]
            }

            pub fn write_to(&self, writer: &mut BinaryWriter) {
                writer.write_u16(self.opcode_tag());
                match *self { $(Self::$name { $($field),* } => { $($field.write(writer);)* }),+ }
            }

            pub fn read_from(reader: &mut BinaryReader<'_>) -> Result<Self, CodecError> {
                match reader.read_u16()? {
                    $($tag => Ok(Self::$name { $($field: <$ty as WireOperand>::read(reader)?),* }),)+
                    tag => Err(CodecError::InvalidTag { type_name: "PortableInstructionV1", tag: u32::from(tag) }),
                }
            }

        }
    };
}

instructions! {
    1 => Constant { operand0: u16 => Constant },
    2 => Nil {  },
    3 => True {  },
    4 => False {  },
    5 => Pop {  },
    6 => GetLocal { operand0: u16 => Local },
    7 => SetLocal { operand0: u16 => Local },
    8 => DefineGlobal { operand0: u16 => SymbolConstant },
    9 => GetGlobal { operand0: u16 => SymbolConstant },
    10 => SetGlobal { operand0: u16 => SymbolConstant },
    11 => GetField { operand0: u16 => Field },
    12 => SetField { operand0: u16 => Field },
    13 => GetSelf {  },
    14 => Invoke { operand0: u8 => Arity, operand1: u16 => SymbolConstant },
    15 => SuperSend { operand0: u8 => Arity, operand1: u16 => SymbolConstant },
    16 => Class { operand0: u16 => SymbolConstant },
    17 => Method { operand0: u16 => SymbolConstant, operand1: bool => Flag },
    18 => Return {  },
    19 => ReturnNonLocal {  },
    20 => Closure { operand0: u16 => ClosureTemplateConstant },
    21 => GetUpvalue { operand0: u16 => Upvalue },
    22 => SetUpvalue { operand0: u16 => Upvalue },
    23 => CloseUpvalue { operand0: u16 => Local },
    24 => Jump { operand0: i32 => RelativeJump },
    25 => JumpIfFalse { operand0: i32 => RelativeJump },
    26 => JumpIfNone { operand0: i32 => RelativeJump },
    27 => Loop { operand0: i32 => RelativeJump },
    28 => GuardBool { operand0: i32 => RelativeJump },
    29 => GuardBlock { operand0: i32 => RelativeJump },
    30 => NewInstance {  },
    31 => Dup {  },
    32 => WrapSome {  },
    33 => GetLinked { operand0: u16 => LinkedRead },
    34 => MakeFamily { spec: u16 => SelectorSpecification, kind: PortableFamilyKindV1 => FamilyKind },
    35 => FinalizeClass {  },
    36 => InvokeLocal { operand0: u16 => Local, operand1: u8 => Arity, operand2: u16 => SymbolConstant },
    37 => InvokeConst { operand0: u16 => Constant, operand1: u8 => Arity, operand2: u16 => SymbolConstant },
    38 => GuardSymbol {  },
    39 => BuildTuple { positional: u16 => Count, labeled: u16 => Count },
    40 => BuildRecord { fields: u16 => Count },
    41 => BeginMapLiteral {  },
    42 => MapLiteralInsertUnique {  },
    43 => FinishMapLiteral {  },
    44 => BeginSetLiteral {  },
    45 => SetLiteralAdd {  },
    46 => FinishSetLiteral {  },
    47 => BuildRange { has_lower: bool => Flag, has_upper: bool => Flag, upper_inclusive: bool => Flag },
    48 => InvokeCompilerInternal { operand0: u8 => Arity, operand1: u16 => SymbolConstant },
    49 => BuildList { operand0: u16 => Count },
    50 => NewArgumentPack {  },
    51 => PackPushPositional {  },
    52 => PackReserveStaticLabel { operand0: u16 => SymbolConstant },
    53 => PackReserveComputedLabel {  },
    54 => PackFillReservedLabel {  },
    55 => PackExpandLabels {  },
    56 => PackExpandComplete {  },
    57 => PackTryExpandTuplePositionals {  },
    58 => InvokePack { base_name: u16 => SymbolConstant, kind: PortableSendKindV1 => SendKind, access: PortableAccessV1 => Access },
    59 => SuperSendPack { base_name: u16 => SymbolConstant },
    60 => FinishTuplePack {  },
    61 => ReserveScratchLocal { operand0: u16 => Local },
    62 => ReleaseScratchLocal { operand0: u16 => Local },
    63 => BeginListLiteral {  },
    64 => ListLiteralAppend {  },
    65 => FinishListLiteral {  },
    66 => ListTryExpandTuplePositionals {  },
    67 => NewRecordLiteralBuilder {  },
    68 => RecordLiteralAppend {  },
    69 => RecordLiteralExpandLabels {  },
    70 => FinishRecordLiteral {  },
    71 => MapLiteralExpandLabels {  },
    72 => GetEllipsis {  },
    73 => BilateralPreferReflected { operand0: u16 => SymbolConstant },
    74 => TryInvokeExact { arity: u8 => Arity, selector: u16 => SymbolConstant, missing_offset: i32 => RelativeJump },
    75 => JumpIfUnsupported { operand0: i32 => RelativeJump },
    76 => ValidateOrdering { reverse: bool => Flag },
    77 => Same {  },
    78 => RaiseUnsupported { operator: u16 => SymbolConstant, direct: u16 => SymbolConstant, reflected: u16 => SymbolConstant },
    79 => RaiseNonIterableStarOperand {  },
    80 => Enum { operand0: u16 => EnumDeclaration },
    81 => VariantMethod { variant: u16 => VariantTarget, selector: u16 => SymbolConstant },
    82 => FinalizeEnum { operand0: u16 => EnumDeclaration },
    83 => LoadVariantSingleton { operand0: u16 => VariantTarget },
    84 => ConstructVariant { variant: u16 => VariantTarget, arity: u8 => Arity },
    85 => MakeResolvedBoundMethod { operand0: u16 => AssociatedTarget },
    86 => InvokeResolvedAssociated { target: u16 => AssociatedTarget, arity: u8 => Arity },
    87 => MakeAssociatedFamily { operand0: u16 => FamilyDescriptor },
    88 => InvokeAssociatedFamilyStatic { operation: u16 => FamilyOperation, arity: u8 => Arity },
    89 => InvokeAssociatedFamilyPack { candidates: u16 => FamilyCandidates },
    90 => IsVariant { operand0: u16 => VariantTarget },
    91 => GetVariantPayload { operand0: u16 => Component },
    92 => MatchInvariantFailure {  },
    93 => InvokeSubscriptSetPack { base_name: u16 => SymbolConstant, access: PortableAccessV1 => Access },
    94 => Data { operand0: u16 => DataDeclaration },
    95 => FinalizeData { operand0: u16 => DataDeclaration },
    96 => LoadDataSingleton { operand0: u16 => DataConstruction },
    97 => ConstructData { constructor: u16 => DataConstruction, arity: u8 => Arity },
    98 => GetDataComponent { operand0: u16 => Component },
    99 => BuildStaticTuple { spec: u16 => AnonymousProduct },
    100 => BuildStaticRecord { spec: u16 => AnonymousProduct },
    101 => InvokeConditional { arity: u8 => Arity, selector: u16 => SymbolConstant, declaring_module: u16 => ModuleConstant, declaring_name: u16 => SymbolConstant, class_side: bool => Flag, fallback_method: u16 => MethodConstant, call_environment_recipe: u16 => CallEnvironment },
    102 => MakeConditionalFamily { spec: u16 => SelectorSpecification, kind: PortableFamilyKindV1 => FamilyKind, conditional: u16 => ConditionalFamily },
    103 => InvokeSpecialized { operand0: u8 => Arity, operand1: u16 => SymbolConstant, operand2: u16 => CallEnvironment },
    104 => InvokeTraitSelected { selection: u16 => TraitInvocation, arity: u8 => Arity },
    105 => InvokeTraitRequirement { slot: u16 => TraitRequirementInvocation, arity: u8 => Arity },
    106 => MakeTraitBoundMethod { operand0: u16 => TraitInvocation },
    107 => GetDataProductSlot { operand0: u16 => Component },
    108 => UninitializedBinding {  },
    109 => GetLocalChecked { operand0: u16 => Local, operand1: u16 => SymbolConstant },
    110 => GetUpvalueChecked { operand0: u16 => Upvalue, operand1: u16 => SymbolConstant },
    111 => GetGlobalChecked { operand0: u16 => SymbolConstant },
    112 => GetFieldChecked { operand0: u16 => Field, operand1: u16 => SymbolConstant },
    113 => FinishPack {  },
    114 => BuildStaticPack { spec: u16 => AnonymousProduct },
    115 => BuildPack { positional: u16 => Count, labeled: u16 => Count },
    116 => GuardFieldKey {  },
    117 => RecordLiteralCheckKey {  },
    118 => LoadTypeParameterClass { operand0: u16 => TypeParameterReceiver },
    119 => BuildArray { operand0: u16 => Count },
    120 => BeginArrayLiteral {  },
    121 => ArrayLiteralAppend {  },
    122 => FinishArrayLiteral {  },
    123 => ArrayTryExpandTuplePositionals {  },
    124 => BuildVector { operand0: u16 => Count },
    125 => VectorExpandExactPositionals { vector: bool => Flag, expected: u32 => Count },
    126 => FinishVectorLiteral { operand0: u32 => Count },
    127 => BeginVectorLiteral {  },
    128 => CopyVector {  },
}

const _: () = {
    let mut left = 0;
    while left < OPCODE_MANIFEST_V1.len() {
        let mut right = left + 1;
        while right < OPCODE_MANIFEST_V1.len() {
            assert!(OPCODE_MANIFEST_V1[left].tag != OPCODE_MANIFEST_V1[right].tag);
            right += 1;
        }
        left += 1;
    }
};

#[cfg(test)]
mod tests {
    use super::{OPCODE_MANIFEST_V1, PortableInstructionV1, relocation_requirement_v1, validator_requirement_v1};
    use crate::{BinaryReader, BinaryWriter};
    use std::collections::BTreeSet;

    #[test]
    fn every_portable_opcode_variant_round_trips_and_matches_the_manifest() {
        let variants = PortableInstructionV1::test_variants();
        assert_eq!(variants.len(), 128);
        assert_eq!(OPCODE_MANIFEST_V1.len(), variants.len());

        let mut seen_tags = BTreeSet::new();
        for instruction in variants {
            let tag = instruction.opcode_tag();
            assert!(seen_tags.insert(tag), "duplicate sampled opcode tag {tag}");
            let row = OPCODE_MANIFEST_V1
                .iter()
                .find(|row| row.tag == tag)
                .unwrap_or_else(|| panic!("opcode {tag} is missing from the manifest"));
            for operand in row.operands {
                assert_eq!(operand.relocation, relocation_requirement_v1(operand.domain), "{} relocation", row.name);
                assert_eq!(operand.validation, validator_requirement_v1(operand.domain), "{} validation", row.name);
                assert_ne!(operand.encoding.width_bytes(), 0, "{} operand width", row.name);
            }

            let mut writer = BinaryWriter::new();
            instruction.write_to(&mut writer);
            let bytes = writer.into_vec();
            let mut reader = BinaryReader::new(&bytes);
            assert_eq!(PortableInstructionV1::read_from(&mut reader).unwrap(), instruction, "opcode {}", row.name);
        }
        assert_eq!(seen_tags.len(), OPCODE_MANIFEST_V1.len());
    }
}
