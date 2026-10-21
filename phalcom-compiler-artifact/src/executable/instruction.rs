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

/// Normal control-flow effect of a portable opcode. Every opcode must declare
/// one class in the instruction table; adding a new opcode cannot implicitly
/// make it an ordinary fallthrough instruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlFlowKindV1 {
    Fallthrough,
    UnconditionalBranch,
    ConditionalBranch,
    FunctionExit,
    NonLocalExit,
    ExceptionalTermination,
}

/// Process-independent value exposed by the generated operand visitor so the
/// core validator can apply each manifest domain without maintaining a second
/// match over the opcode set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortableOperandValueV1 {
    Unsigned(u32),
    Signed(i32),
    Boolean(bool),
    Tag(u8),
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
    pub control_flow: ControlFlowKindV1,
}

trait WireOperand: Sized + Copy {
    const ENCODING: OperandEncodingV1;
    fn write(self, writer: &mut BinaryWriter);
    fn read(reader: &mut BinaryReader<'_>) -> Result<Self, CodecError>;
    fn portable_value(self) -> PortableOperandValueV1;
}

macro_rules! primitive_operand {
    ($ty:ty, $write:ident, $read:ident, $encoding:ident, $value_variant:ident) => {
        impl WireOperand for $ty {
            const ENCODING: OperandEncodingV1 = OperandEncodingV1::$encoding;
            fn write(self, writer: &mut BinaryWriter) {
                writer.$write(self);
            }
            fn read(reader: &mut BinaryReader<'_>) -> Result<Self, CodecError> {
                reader.$read()
            }
            fn portable_value(self) -> PortableOperandValueV1 {
                PortableOperandValueV1::$value_variant(self as _)
            }
        }
    };
}
primitive_operand!(u8, write_u8, read_u8, U8, Unsigned);
primitive_operand!(u16, write_u16, read_u16, U16Le, Unsigned);
primitive_operand!(u32, write_u32, read_u32, U32Le, Unsigned);
primitive_operand!(bool, write_bool, read_bool, BoolU8, Boolean);
impl WireOperand for i32 {
    const ENCODING: OperandEncodingV1 = OperandEncodingV1::I32Le;
    fn write(self, writer: &mut BinaryWriter) {
        writer.write_bytes(&self.to_le_bytes());
    }
    fn read(reader: &mut BinaryReader<'_>) -> Result<Self, CodecError> {
        Ok(Self::from_le_bytes(reader.read_fixed()?))
    }
    fn portable_value(self) -> PortableOperandValueV1 {
        PortableOperandValueV1::Signed(self)
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
            fn portable_value(self) -> PortableOperandValueV1 {
                PortableOperandValueV1::Tag(match self { $(Self::$variant => $tag),+ })
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
    ($($tag:literal => $name:ident => $control_flow:ident { $($field:ident: $ty:ty => $domain:ident),* $(,)? }),+ $(,)?) => {
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
                control_flow: ControlFlowKindV1::$control_flow,
            }),+
        ];

        impl PortableInstructionV1 {
            pub fn opcode_tag(&self) -> u16 { match self { $(Self::$name { .. } => $tag),+ } }

            pub const fn control_flow_kind(&self) -> ControlFlowKindV1 {
                match self { $(Self::$name { .. } => ControlFlowKindV1::$control_flow),+ }
            }

            /// Visits every operand together with the exact domain emitted into
            /// `OPCODE_MANIFEST_V1`. Both views are generated from this single
            /// opcode declaration, so the core validator cannot silently miss
            /// an operand when a new instruction is added.
            pub fn visit_operands(
                &self,
                mut visit: impl FnMut(OperandDomainV1, PortableOperandValueV1),
            ) {
                match *self {
                    $(Self::$name { $($field),* } => {
                        $(visit(
                            OperandDomainV1::$domain,
                            <$ty as WireOperand>::portable_value($field),
                        );)*
                    }),+
                }
            }

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
    1 => Constant => Fallthrough { operand0: u16 => Constant },
    2 => Nil => Fallthrough {  },
    3 => True => Fallthrough {  },
    4 => False => Fallthrough {  },
    5 => Pop => Fallthrough {  },
    6 => GetLocal => Fallthrough { operand0: u16 => Local },
    7 => SetLocal => Fallthrough { operand0: u16 => Local },
    8 => DefineGlobal => Fallthrough { operand0: u16 => SymbolConstant },
    9 => GetGlobal => Fallthrough { operand0: u16 => SymbolConstant },
    10 => SetGlobal => Fallthrough { operand0: u16 => SymbolConstant },
    11 => GetField => Fallthrough { operand0: u16 => Field },
    12 => SetField => Fallthrough { operand0: u16 => Field },
    13 => GetSelf => Fallthrough {  },
    14 => Invoke => Fallthrough { operand0: u8 => Arity, operand1: u16 => SymbolConstant },
    15 => SuperSend => Fallthrough { operand0: u8 => Arity, operand1: u16 => SymbolConstant },
    16 => Class => Fallthrough { operand0: u16 => SymbolConstant },
    17 => Method => Fallthrough { operand0: u16 => SymbolConstant, operand1: bool => Flag },
    18 => Return => FunctionExit {  },
    19 => ReturnNonLocal => NonLocalExit {  },
    20 => Closure => Fallthrough { operand0: u16 => ClosureTemplateConstant },
    21 => GetUpvalue => Fallthrough { operand0: u16 => Upvalue },
    22 => SetUpvalue => Fallthrough { operand0: u16 => Upvalue },
    23 => CloseUpvalue => Fallthrough { operand0: u16 => Local },
    24 => Jump => UnconditionalBranch { operand0: i32 => RelativeJump },
    25 => JumpIfFalse => ConditionalBranch { operand0: i32 => RelativeJump },
    26 => JumpIfNone => ConditionalBranch { operand0: i32 => RelativeJump },
    27 => Loop => UnconditionalBranch { operand0: i32 => RelativeJump },
    28 => GuardBool => ConditionalBranch { operand0: i32 => RelativeJump },
    29 => GuardBlock => ConditionalBranch { operand0: i32 => RelativeJump },
    30 => NewInstance => Fallthrough {  },
    31 => Dup => Fallthrough {  },
    32 => WrapSome => Fallthrough {  },
    33 => GetLinked => Fallthrough { operand0: u16 => LinkedRead },
    34 => MakeFamily => Fallthrough { spec: u16 => SelectorSpecification, kind: PortableFamilyKindV1 => FamilyKind },
    35 => FinalizeClass => Fallthrough {  },
    36 => InvokeLocal => Fallthrough { operand0: u16 => Local, operand1: u8 => Arity, operand2: u16 => SymbolConstant },
    37 => InvokeConst => Fallthrough { operand0: u16 => Constant, operand1: u8 => Arity, operand2: u16 => SymbolConstant },
    38 => GuardSymbol => Fallthrough {  },
    39 => BuildTuple => Fallthrough { positional: u16 => Count, labeled: u16 => Count },
    40 => BuildRecord => Fallthrough { fields: u16 => Count },
    41 => BeginMapLiteral => Fallthrough {  },
    42 => MapLiteralInsertUnique => Fallthrough {  },
    43 => FinishMapLiteral => Fallthrough {  },
    44 => BeginSetLiteral => Fallthrough {  },
    45 => SetLiteralAdd => Fallthrough {  },
    46 => FinishSetLiteral => Fallthrough {  },
    47 => BuildRange => Fallthrough { has_lower: bool => Flag, has_upper: bool => Flag, upper_inclusive: bool => Flag },
    48 => InvokeCompilerInternal => Fallthrough { operand0: u8 => Arity, operand1: u16 => SymbolConstant },
    49 => BuildList => Fallthrough { operand0: u16 => Count },
    50 => NewArgumentPack => Fallthrough {  },
    51 => PackPushPositional => Fallthrough {  },
    52 => PackReserveStaticLabel => Fallthrough { operand0: u16 => SymbolConstant },
    53 => PackReserveComputedLabel => Fallthrough {  },
    54 => PackFillReservedLabel => Fallthrough {  },
    55 => PackExpandLabels => Fallthrough {  },
    56 => PackExpandComplete => Fallthrough {  },
    57 => PackTryExpandTuplePositionals => Fallthrough {  },
    58 => InvokePack => Fallthrough { base_name: u16 => SymbolConstant, kind: PortableSendKindV1 => SendKind, access: PortableAccessV1 => Access },
    59 => SuperSendPack => Fallthrough { base_name: u16 => SymbolConstant },
    60 => FinishTuplePack => Fallthrough {  },
    61 => ReserveScratchLocal => Fallthrough { operand0: u16 => Local },
    62 => ReleaseScratchLocal => Fallthrough { operand0: u16 => Local },
    63 => BeginListLiteral => Fallthrough {  },
    64 => ListLiteralAppend => Fallthrough {  },
    65 => FinishListLiteral => Fallthrough {  },
    66 => ListTryExpandTuplePositionals => Fallthrough {  },
    67 => NewRecordLiteralBuilder => Fallthrough {  },
    68 => RecordLiteralAppend => Fallthrough {  },
    69 => RecordLiteralExpandLabels => Fallthrough {  },
    70 => FinishRecordLiteral => Fallthrough {  },
    71 => MapLiteralExpandLabels => Fallthrough {  },
    72 => GetEllipsis => Fallthrough {  },
    73 => BilateralPreferReflected => Fallthrough { operand0: u16 => SymbolConstant },
    74 => TryInvokeExact => ConditionalBranch { arity: u8 => Arity, selector: u16 => SymbolConstant, missing_offset: i32 => RelativeJump },
    75 => JumpIfUnsupported => ConditionalBranch { operand0: i32 => RelativeJump },
    76 => ValidateOrdering => Fallthrough { reverse: bool => Flag },
    77 => Same => Fallthrough {  },
    78 => RaiseUnsupported => ExceptionalTermination { operator: u16 => SymbolConstant, direct: u16 => SymbolConstant, reflected: u16 => SymbolConstant },
    79 => RaiseNonIterableStarOperand => ExceptionalTermination {  },
    80 => Enum => Fallthrough { operand0: u16 => EnumDeclaration },
    81 => VariantMethod => Fallthrough { variant: u16 => VariantTarget, selector: u16 => SymbolConstant },
    82 => FinalizeEnum => Fallthrough { operand0: u16 => EnumDeclaration },
    83 => LoadVariantSingleton => Fallthrough { operand0: u16 => VariantTarget },
    84 => ConstructVariant => Fallthrough { variant: u16 => VariantTarget, arity: u8 => Arity },
    85 => MakeResolvedBoundMethod => Fallthrough { operand0: u16 => AssociatedTarget },
    86 => InvokeResolvedAssociated => Fallthrough { target: u16 => AssociatedTarget, arity: u8 => Arity },
    87 => MakeAssociatedFamily => Fallthrough { operand0: u16 => FamilyDescriptor },
    88 => InvokeAssociatedFamilyStatic => Fallthrough { operation: u16 => FamilyOperation, arity: u8 => Arity },
    89 => InvokeAssociatedFamilyPack => Fallthrough { candidates: u16 => FamilyCandidates },
    90 => IsVariant => Fallthrough { operand0: u16 => VariantTarget },
    91 => GetVariantPayload => Fallthrough { operand0: u16 => Component },
    92 => MatchInvariantFailure => ExceptionalTermination {  },
    93 => InvokeSubscriptSetPack => Fallthrough { base_name: u16 => SymbolConstant, access: PortableAccessV1 => Access },
    94 => Data => Fallthrough { operand0: u16 => DataDeclaration },
    95 => FinalizeData => Fallthrough { operand0: u16 => DataDeclaration },
    96 => LoadDataSingleton => Fallthrough { operand0: u16 => DataConstruction },
    97 => ConstructData => Fallthrough { constructor: u16 => DataConstruction, arity: u8 => Arity },
    98 => GetDataComponent => Fallthrough { operand0: u16 => Component },
    99 => BuildStaticTuple => Fallthrough { spec: u16 => AnonymousProduct },
    100 => BuildStaticRecord => Fallthrough { spec: u16 => AnonymousProduct },
    101 => InvokeConditional => Fallthrough { arity: u8 => Arity, selector: u16 => SymbolConstant, declaring_module: u16 => ModuleConstant, declaring_name: u16 => SymbolConstant, class_side: bool => Flag, fallback_method: u16 => MethodConstant, call_environment_recipe: u16 => CallEnvironment },
    102 => MakeConditionalFamily => Fallthrough { spec: u16 => SelectorSpecification, kind: PortableFamilyKindV1 => FamilyKind, conditional: u16 => ConditionalFamily },
    103 => InvokeSpecialized => Fallthrough { operand0: u8 => Arity, operand1: u16 => SymbolConstant, operand2: u16 => CallEnvironment },
    104 => InvokeTraitSelected => Fallthrough { selection: u16 => TraitInvocation, arity: u8 => Arity },
    105 => InvokeTraitRequirement => Fallthrough { slot: u16 => TraitRequirementInvocation, arity: u8 => Arity },
    106 => MakeTraitBoundMethod => Fallthrough { operand0: u16 => TraitInvocation },
    107 => GetDataProductSlot => Fallthrough { operand0: u16 => Component },
    108 => UninitializedBinding => Fallthrough {  },
    109 => GetLocalChecked => Fallthrough { operand0: u16 => Local, operand1: u16 => SymbolConstant },
    110 => GetUpvalueChecked => Fallthrough { operand0: u16 => Upvalue, operand1: u16 => SymbolConstant },
    111 => GetGlobalChecked => Fallthrough { operand0: u16 => SymbolConstant },
    112 => GetFieldChecked => Fallthrough { operand0: u16 => Field, operand1: u16 => SymbolConstant },
    113 => FinishPack => Fallthrough {  },
    114 => BuildStaticPack => Fallthrough { spec: u16 => AnonymousProduct },
    115 => BuildPack => Fallthrough { positional: u16 => Count, labeled: u16 => Count },
    116 => GuardFieldKey => Fallthrough {  },
    117 => RecordLiteralCheckKey => Fallthrough {  },
    118 => LoadTypeParameterClass => Fallthrough { operand0: u16 => TypeParameterReceiver },
    119 => BuildArray => Fallthrough { operand0: u16 => Count },
    120 => BeginArrayLiteral => Fallthrough {  },
    121 => ArrayLiteralAppend => Fallthrough {  },
    122 => FinishArrayLiteral => Fallthrough {  },
    123 => ArrayTryExpandTuplePositionals => Fallthrough {  },
    124 => BuildVector => Fallthrough { operand0: u16 => Count },
    125 => VectorExpandExactPositionals => Fallthrough { vector: bool => Flag, expected: u32 => Count },
    126 => FinishVectorLiteral => Fallthrough { operand0: u32 => Count },
    127 => BeginVectorLiteral => Fallthrough {  },
    128 => CopyVector => Fallthrough {  },
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
    use super::{ControlFlowKindV1, OPCODE_MANIFEST_V1, PortableInstructionV1, relocation_requirement_v1, validator_requirement_v1};
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

    #[test]
    fn opcode_manifest_declares_current_control_flow_contracts() {
        for row in OPCODE_MANIFEST_V1 {
            let expected = match row.tag {
                18 => ControlFlowKindV1::FunctionExit,
                19 => ControlFlowKindV1::NonLocalExit,
                24 | 27 => ControlFlowKindV1::UnconditionalBranch,
                25 | 26 | 28 | 29 | 74 | 75 => ControlFlowKindV1::ConditionalBranch,
                78 | 79 | 92 => ControlFlowKindV1::ExceptionalTermination,
                _ => ControlFlowKindV1::Fallthrough,
            };
            assert_eq!(row.control_flow, expected, "opcode {}", row.name);
        }
    }
}
