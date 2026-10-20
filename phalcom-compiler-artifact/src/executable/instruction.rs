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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpcodeManifestRowV1 {
    pub tag: u16,
    pub name: &'static str,
    pub operands: &'static [OperandDomainV1],
}

trait WireOperand: Sized + Copy {
    fn write(self, writer: &mut BinaryWriter);
    fn read(reader: &mut BinaryReader<'_>) -> Result<Self, CodecError>;
}

macro_rules! primitive_operand {
    ($ty:ty, $write:ident, $read:ident) => {
        impl WireOperand for $ty {
            fn write(self, writer: &mut BinaryWriter) {
                writer.$write(self);
            }
            fn read(reader: &mut BinaryReader<'_>) -> Result<Self, CodecError> {
                reader.$read()
            }
        }
    };
}
primitive_operand!(u8, write_u8, read_u8);
primitive_operand!(u16, write_u16, read_u16);
primitive_operand!(u32, write_u32, read_u32);
primitive_operand!(bool, write_bool, read_bool);
impl WireOperand for i32 {
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

macro_rules! instructions {
    ($($tag:literal => $name:ident { $($field:ident: $ty:ty => $domain:ident),* $(,)? }),+ $(,)?) => {
        /// One decoded instruction. References are unvalidated until the full
        /// executable image has passed its cross-table and control-flow checks.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum PortableInstructionV1 { $($name { $($field: $ty),* }),+ }

        pub const OPCODE_MANIFEST_V1: &[OpcodeManifestRowV1] = &[
            $(OpcodeManifestRowV1 { tag: $tag, name: stringify!($name), operands: &[$(OperandDomainV1::$domain),*] }),+
        ];

        impl PortableInstructionV1 {
            pub fn opcode_tag(&self) -> u16 { match self { $(Self::$name { .. } => $tag),+ } }

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
