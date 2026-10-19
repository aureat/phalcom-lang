//! Portable `QUERY_IMAGE` wire DTOs and exhaustive profile dispositions (`PERF003.C2.P7.T9`).

use crate::codec::{BinaryReader, BinaryWriter};
use crate::error::{ArtifactError, ValidationError};
use crate::limits::ValidationLimits;
use crate::module_world::{Decode, ModuleWorldV1, ModuleWorldV2, Wire};
use crate::semantic::world::SemanticWorldV3;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::identity::{
    StableAssociatedStaticDomainDeclarationRef, StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableFieldRef,
    StableImplRef, StableModuleRef, StableProjectRef, StableStaticConstantRef, StableStaticDomainDeclarationRef, StableTypeMemberRef, StableVariantRef,
};

pub const QUERY_IMAGE_SCHEMA_V2: u32 = 2;
pub const QUERY_IMAGE_SCHEMA_V3: u32 = 3;


/// QUERY_FINGERPRINT_ABI_V2 domains for portable structural query evidence.
///
/// These fingerprints belong to the artifact wire contract, not to
/// `SemanticDb`. They are computed from stable-owner semantic rows and remain
/// valid when the receiving workspace relocates a persistent project to a
/// different graph-local `ResolvedProjectId`.
const DOMAIN_STRUCTURAL_QUERY_INPUT_V2: &str = "telos.telosc.v2.query-input.structural";
const DOMAIN_STRUCTURAL_QUERY_PRODUCT_V2: &str = "telos.telosc.v2.query-product.structural";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u16)]
pub enum PortableQueryKindV1 {
    ParsedModule = 1,
    UnlinkedInterface = 2,
    ResolvedImports = 3,
    LinkedInterface = 4,
    DeclarationShell = 5,
    HierarchyEdge = 6,
    CallableDefinition = 7,
    CallableSignature = 8,
    FieldSignature = 9,
    DeclarationSurface = 10,
    TraitHeader = 11,
    TraitSurface = 12,
    DataDeclaration = 13,
    EnumDeclaration = 14,
    AssociatedSurface = 15,
    TypeMemberDeclaration = 16,
    ModuleStaticBindingCatalog = 17,
    ModuleStaticBindings = 18,
    SemanticComponent = 19,
    ConformanceDispatch = 20,
    TraitDispatchBucket = 21,
    GenericConformance = 22,
    ConstructorInitialization = 23,
    CallableBody = 24,
    CallableEffects = 25,
    CallableControl = 26,
    CallableTermination = 27,
    CallableContracts = 28,
    AttributeArgument = 29,
    VerificationConditions = 30,
    SourceStructure = 31,
    SourceFormalAttachment = 32,
    AdvisoryCallable = 33,
    AdvisoryModule = 34,
    ModuleDiagnostics = 35,
    ModuleMetadata = 36,
    EnumRequirements = 37,
    StaticDomainCaseCatalog = 38,
    StaticCaseSignature = 39,
    AssociatedStaticDomainDeclaration = 40,
    ResolvedImport = 41,
    LinkedName = 42,
    PublicExport = 43,
    StaticDefinition = 44,
    StaticConstant = 45,
    CheckedStaticBody = 46,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryProfileSupport {
    OmittedSourceDependent,
    AggregateDependencyEvidenceOnly,
    RepresentedByModuleWorld,
    RepresentedBySemanticWorld,
    OmittedAdvisoryOnly,
}

impl PortableQueryKindV1 {
    pub const fn profile_support(self) -> QueryProfileSupport {
        match self {
            Self::ParsedModule | Self::SourceStructure | Self::SourceFormalAttachment => QueryProfileSupport::OmittedSourceDependent,
            Self::HierarchyEdge => QueryProfileSupport::AggregateDependencyEvidenceOnly,
            Self::UnlinkedInterface
            | Self::ResolvedImports
            | Self::LinkedInterface
            | Self::SemanticComponent
            | Self::ModuleDiagnostics
            | Self::ModuleMetadata
            | Self::ResolvedImport
            | Self::LinkedName
            | Self::PublicExport => QueryProfileSupport::RepresentedByModuleWorld,
            Self::DeclarationShell
            | Self::DeclarationSurface
            | Self::TraitHeader
            | Self::TraitSurface
            | Self::CallableDefinition
            | Self::CallableSignature
            | Self::FieldSignature
            | Self::DataDeclaration
            | Self::EnumDeclaration
            | Self::EnumRequirements
            | Self::AssociatedSurface
            | Self::TypeMemberDeclaration
            | Self::ModuleStaticBindings
            | Self::ConformanceDispatch
            | Self::TraitDispatchBucket
            | Self::GenericConformance
            | Self::ConstructorInitialization
            | Self::CallableBody
            | Self::CallableEffects
            | Self::CallableControl
            | Self::CallableTermination
            | Self::CallableContracts
            | Self::AttributeArgument
            | Self::VerificationConditions
            | Self::StaticDomainCaseCatalog
            | Self::StaticCaseSignature
            | Self::AssociatedStaticDomainDeclaration
            | Self::StaticDefinition
            | Self::StaticConstant
            | Self::ModuleStaticBindingCatalog
            | Self::CheckedStaticBody => QueryProfileSupport::RepresentedBySemanticWorld,
            Self::AdvisoryCallable | Self::AdvisoryModule => QueryProfileSupport::OmittedAdvisoryOnly,
        }
    }

    pub fn from_raw(raw: u16) -> Option<Self> {
        match raw {
            1 => Some(Self::ParsedModule),
            2 => Some(Self::UnlinkedInterface),
            3 => Some(Self::ResolvedImports),
            4 => Some(Self::LinkedInterface),
            5 => Some(Self::DeclarationShell),
            6 => Some(Self::HierarchyEdge),
            7 => Some(Self::CallableDefinition),
            8 => Some(Self::CallableSignature),
            9 => Some(Self::FieldSignature),
            10 => Some(Self::DeclarationSurface),
            11 => Some(Self::TraitHeader),
            12 => Some(Self::TraitSurface),
            13 => Some(Self::DataDeclaration),
            14 => Some(Self::EnumDeclaration),
            15 => Some(Self::AssociatedSurface),
            16 => Some(Self::TypeMemberDeclaration),
            17 => Some(Self::ModuleStaticBindingCatalog),
            18 => Some(Self::ModuleStaticBindings),
            19 => Some(Self::SemanticComponent),
            20 => Some(Self::ConformanceDispatch),
            21 => Some(Self::TraitDispatchBucket),
            22 => Some(Self::GenericConformance),
            23 => Some(Self::ConstructorInitialization),
            24 => Some(Self::CallableBody),
            25 => Some(Self::CallableEffects),
            26 => Some(Self::CallableControl),
            27 => Some(Self::CallableTermination),
            28 => Some(Self::CallableContracts),
            29 => Some(Self::AttributeArgument),
            30 => Some(Self::VerificationConditions),
            31 => Some(Self::SourceStructure),
            32 => Some(Self::SourceFormalAttachment),
            33 => Some(Self::AdvisoryCallable),
            34 => Some(Self::AdvisoryModule),
            35 => Some(Self::ModuleDiagnostics),
            36 => Some(Self::ModuleMetadata),
            37 => Some(Self::EnumRequirements),
            38 => Some(Self::StaticDomainCaseCatalog),
            39 => Some(Self::StaticCaseSignature),
            40 => Some(Self::AssociatedStaticDomainDeclaration),
            41 => Some(Self::ResolvedImport),
            42 => Some(Self::LinkedName),
            43 => Some(Self::PublicExport),
            44 => Some(Self::StaticDefinition),
            45 => Some(Self::StaticConstant),
            46 => Some(Self::CheckedStaticBody),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LegacyPortableQueryKeyV1 {
    pub kind: PortableQueryKindV1,
    pub stable_owner_key: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryProductRefV2 {
    /// Product family is the current query kind; `row_index` addresses that
    /// family's typed row in MODULE_WORLD or SEMANTIC_WORLD.
    pub kind: PortableQueryKindV1,
    pub row_index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryRootEntryV2 {
    pub key: LegacyPortableQueryKeyV1,
    pub product: QueryProductRefV2,
    pub input_fingerprint: u64,
    pub product_fingerprint: u64,
    pub dependency_indices: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DependencyEvidenceEntryV2 {
    pub key: LegacyPortableQueryKeyV1,
    pub stable_aggregate_owner: String,
    pub product_fingerprint: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct QueryImageV2 {
    pub supported_roots: Vec<QueryRootEntryV2>,
    pub dependency_evidence: Vec<DependencyEvidenceEntryV2>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PortableQueryOwnerKeyV1 {
    Module(StableModuleRef),
    Declaration(StableDeclarationRef),
    Callable(StableCallableRef),
    Field(StableFieldRef),
    Impl(StableImplRef),
    TraitDispatchBucket {
        target: StableDeclarationRef,
        selector: Box<str>,
        side: StableDispatchSide,
    },
    GenericConformance {
        implementation: StableImplRef,
        context_ordinal: u32,
    },
    AttributeArgument {
        callable: StableCallableRef,
        argument_index: u32,
    },
    TypeMember(StableTypeMemberRef),
    StaticDomain(StableStaticDomainDeclarationRef),
    StaticCase(StableVariantRef),
    AssociatedStaticDomain(StableAssociatedStaticDomainDeclarationRef),
    ImportSite {
        module: StableModuleRef,
        site_id: u32,
    },
    ModuleName {
        module: StableModuleRef,
        name: Box<str>,
    },
    StaticConstant(StableStaticConstantRef),
    CheckedStaticBody {
        owner: StableDeclarationRef,
        body_index: u32,
    },
}

impl PortableQueryOwnerKeyV1 {
    pub fn project(&self) -> &StableProjectRef {
        match self {
            Self::Module(m) => &m.project,
            Self::Declaration(d) => &d.module.project,
            Self::Callable(c) => match &c.owner {
                StableCallableOwnerRef::Declaration(d) => &d.module.project,
                StableCallableOwnerRef::Variant(v) => &v.owner.module.project,
                StableCallableOwnerRef::Conformance(i) => &i.module.project,
            },
            Self::Field(f) => &f.owner.module.project,
            Self::Impl(i) => &i.module.project,
            Self::TraitDispatchBucket { target, .. } => &target.module.project,
            Self::GenericConformance { implementation, .. } => &implementation.module.project,
            Self::AttributeArgument { callable, .. } => match &callable.owner {
                StableCallableOwnerRef::Declaration(d) => &d.module.project,
                StableCallableOwnerRef::Variant(v) => &v.owner.module.project,
                StableCallableOwnerRef::Conformance(i) => &i.module.project,
            },
            Self::TypeMember(t) => &t.owner.module.project,
            Self::StaticDomain(sd) => match sd {
                StableStaticDomainDeclarationRef::Nominal(d) => &d.module.project,
                StableStaticDomainDeclarationRef::Associated(a) => &a.owner.module.project,
            },
            Self::StaticCase(v) => &v.owner.module.project,
            Self::AssociatedStaticDomain(a) => &a.owner.module.project,
            Self::ImportSite { module, .. } => &module.project,
            Self::ModuleName { module, .. } => &module.project,
            Self::StaticConstant(s) => &s.module.project,
            Self::CheckedStaticBody { owner, .. } => &owner.module.project,
        }
    }

    pub fn module_path(&self) -> &[Box<str>] {
        match self {
            Self::Module(m) => &m.path,
            Self::Declaration(d) => &d.module.path,
            Self::Callable(c) => match &c.owner {
                StableCallableOwnerRef::Declaration(d) => &d.module.path,
                StableCallableOwnerRef::Variant(v) => &v.owner.module.path,
                StableCallableOwnerRef::Conformance(i) => &i.module.path,
            },
            Self::Field(f) => &f.owner.module.path,
            Self::Impl(i) => &i.module.path,
            Self::TraitDispatchBucket { target, .. } => &target.module.path,
            Self::GenericConformance { implementation, .. } => &implementation.module.path,
            Self::AttributeArgument { callable, .. } => match &callable.owner {
                StableCallableOwnerRef::Declaration(d) => &d.module.path,
                StableCallableOwnerRef::Variant(v) => &v.owner.module.path,
                StableCallableOwnerRef::Conformance(i) => &i.module.path,
            },
            Self::TypeMember(t) => &t.owner.module.path,
            Self::StaticDomain(sd) => match sd {
                StableStaticDomainDeclarationRef::Nominal(d) => &d.module.path,
                StableStaticDomainDeclarationRef::Associated(a) => &a.owner.module.path,
            },
            Self::StaticCase(v) => &v.owner.module.path,
            Self::AssociatedStaticDomain(a) => &a.owner.module.path,
            Self::ImportSite { module, .. } => &module.path,
            Self::ModuleName { module, .. } => &module.path,
            Self::StaticConstant(s) => &s.module.path,
            Self::CheckedStaticBody { owner, .. } => &owner.module.path,
        }
    }

    pub fn module(&self) -> Option<&StableModuleRef> {
        match self {
            Self::Module(m) => Some(m),
            Self::Declaration(d) => Some(&d.module),
            Self::Callable(c) => match &c.owner {
                StableCallableOwnerRef::Declaration(d) => Some(&d.module),
                StableCallableOwnerRef::Variant(v) => Some(&v.owner.module),
                StableCallableOwnerRef::Conformance(i) => Some(&i.module),
            },
            Self::Field(f) => Some(&f.owner.module),
            Self::Impl(i) => Some(&i.module),
            Self::TraitDispatchBucket { target, .. } => Some(&target.module),
            Self::GenericConformance { implementation, .. } => Some(&implementation.module),
            Self::AttributeArgument { callable, .. } => match &callable.owner {
                StableCallableOwnerRef::Declaration(d) => Some(&d.module),
                StableCallableOwnerRef::Variant(v) => Some(&v.owner.module),
                StableCallableOwnerRef::Conformance(i) => Some(&i.module),
            },
            Self::TypeMember(t) => Some(&t.owner.module),
            Self::StaticDomain(sd) => match sd {
                StableStaticDomainDeclarationRef::Nominal(d) => Some(&d.module),
                StableStaticDomainDeclarationRef::Associated(a) => Some(&a.owner.module),
            },
            Self::StaticCase(v) => Some(&v.owner.module),
            Self::AssociatedStaticDomain(a) => Some(&a.owner.module),
            Self::ImportSite { module, .. } => Some(module),
            Self::ModuleName { module, .. } => Some(module),
            Self::StaticConstant(s) => Some(&s.module),
            Self::CheckedStaticBody { owner, .. } => Some(&owner.module),
        }
    }
}

impl Wire for PortableQueryOwnerKeyV1 {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        match self {
            Self::Module(m) => {
                out.write_u8(0);
                m.write(out)?;
            }
            Self::Declaration(d) => {
                out.write_u8(1);
                d.write(out)?;
            }
            Self::Callable(c) => {
                out.write_u8(2);
                c.write(out)?;
            }
            Self::Field(f) => {
                out.write_u8(3);
                f.write(out)?;
            }
            Self::Impl(i) => {
                out.write_u8(4);
                i.write(out)?;
            }
            Self::TraitDispatchBucket { target, selector, side } => {
                out.write_u8(5);
                target.write(out)?;
                selector.write(out)?;
                side.write(out)?;
            }
            Self::GenericConformance { implementation, context_ordinal } => {
                out.write_u8(6);
                implementation.write(out)?;
                context_ordinal.write(out)?;
            }
            Self::AttributeArgument { callable, argument_index } => {
                out.write_u8(7);
                callable.write(out)?;
                argument_index.write(out)?;
            }
            Self::TypeMember(t) => {
                out.write_u8(8);
                t.write(out)?;
            }
            Self::StaticDomain(sd) => {
                out.write_u8(9);
                sd.write(out)?;
            }
            Self::StaticCase(v) => {
                out.write_u8(10);
                v.write(out)?;
            }
            Self::AssociatedStaticDomain(a) => {
                out.write_u8(11);
                a.write(out)?;
            }
            Self::ImportSite { module, site_id } => {
                out.write_u8(12);
                module.write(out)?;
                site_id.write(out)?;
            }
            Self::ModuleName { module, name } => {
                out.write_u8(13);
                module.write(out)?;
                name.write(out)?;
            }
            Self::StaticConstant(s) => {
                out.write_u8(14);
                s.write(out)?;
            }
            Self::CheckedStaticBody { owner, body_index } => {
                out.write_u8(15);
                owner.write(out)?;
                body_index.write(out)?;
            }
        }
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        input.nested(|input| match input.reader.read_u8()? {
            0 => Ok(Self::Module(StableModuleRef::read(input)?)),
            1 => Ok(Self::Declaration(StableDeclarationRef::read(input)?)),
            2 => Ok(Self::Callable(StableCallableRef::read(input)?)),
            3 => Ok(Self::Field(StableFieldRef::read(input)?)),
            4 => Ok(Self::Impl(StableImplRef::read(input)?)),
            5 => Ok(Self::TraitDispatchBucket {
                target: StableDeclarationRef::read(input)?,
                selector: Box::<str>::read(input)?,
                side: StableDispatchSide::read(input)?,
            }),
            6 => Ok(Self::GenericConformance {
                implementation: StableImplRef::read(input)?,
                context_ordinal: u32::read(input)?,
            }),
            7 => Ok(Self::AttributeArgument {
                callable: StableCallableRef::read(input)?,
                argument_index: u32::read(input)?,
            }),
            8 => Ok(Self::TypeMember(StableTypeMemberRef::read(input)?)),
            9 => Ok(Self::StaticDomain(StableStaticDomainDeclarationRef::read(input)?)),
            10 => Ok(Self::StaticCase(StableVariantRef::read(input)?)),
            11 => Ok(Self::AssociatedStaticDomain(StableAssociatedStaticDomainDeclarationRef::read(input)?)),
            12 => Ok(Self::ImportSite {
                module: StableModuleRef::read(input)?,
                site_id: u32::read(input)?,
            }),
            13 => Ok(Self::ModuleName {
                module: StableModuleRef::read(input)?,
                name: Box::<str>::read(input)?,
            }),
            14 => Ok(Self::StaticConstant(StableStaticConstantRef::read(input)?)),
            15 => Ok(Self::CheckedStaticBody {
                owner: StableDeclarationRef::read(input)?,
                body_index: u32::read(input)?,
            }),
            _ => Err(crate::module_world::invalid("invalid PortableQueryOwnerKeyV1 tag")),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PortableQueryKeyV1 {
    pub kind: PortableQueryKindV1,
    pub owner: PortableQueryOwnerKeyV1,
}

impl Wire for PortableQueryKeyV1 {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        out.write_u16(self.kind as u16);
        self.owner.write(out)
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        input.nested(|input| {
            let kind_raw = input.reader.read_u16()?;
            let kind = PortableQueryKindV1::from_raw(kind_raw)
                .ok_or_else(|| ValidationError::ReferentialIntegrity("unknown query kind".into()))?;
            let owner = PortableQueryOwnerKeyV1::read(input)?;
            Ok(Self { kind, owner })
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryRootEntryV3 {
    pub key: PortableQueryKeyV1,
    pub product: QueryProductRefV2,
    pub input_fingerprint: u64,
    pub product_fingerprint: u64,
    pub dependency_indices: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DependencyEvidenceEntryV3 {
    pub key: PortableQueryKeyV1,
    pub stable_aggregate_owner: PortableQueryOwnerKeyV1,
    pub product_fingerprint: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct QueryImageV3 {
    pub supported_roots: Vec<QueryRootEntryV3>,
    pub dependency_evidence: Vec<DependencyEvidenceEntryV3>,
}


fn declaration_owner(owner: &PortableQueryOwnerKeyV1) -> Result<&StableDeclarationRef, ArtifactError> {
    match owner {
        PortableQueryOwnerKeyV1::Declaration(declaration) => Ok(declaration),
        _ => Err(ValidationError::ReferentialIntegrity("structural declaration query owner is not a declaration".into()).into()),
    }
}

fn type_member_owner(owner: &PortableQueryOwnerKeyV1) -> Result<&StableTypeMemberRef, ArtifactError> {
    match owner {
        PortableQueryOwnerKeyV1::TypeMember(member) => Ok(member),
        _ => Err(ValidationError::ReferentialIntegrity("type-member query owner is not a type-member identity".into()).into()),
    }
}

fn callable_owner(owner: &PortableQueryOwnerKeyV1) -> Result<&StableCallableRef, ArtifactError> {
    match owner {
        PortableQueryOwnerKeyV1::Callable(callable) => Ok(callable),
        _ => Err(ValidationError::ReferentialIntegrity("callable-definition query owner is not a callable identity".into()).into()),
    }
}

fn callable_definition_row_matches_owner(
    row: &crate::semantic::world::CallableDefinitionRecordV1,
    owner: &StableCallableRef,
) -> bool {
    &row.callable == owner && &row.signature.callable == owner
}

fn portable_callable_definition_digest64(
    domain: &str,
    row_index: usize,
    owner: &StableCallableRef,
    world: &SemanticWorldV3,
) -> Result<u64, ArtifactError> {
    let mut out = BinaryWriter::digest(domain);
    out.write_u16(PortableQueryKindV1::CallableDefinition as u16);
    let row = world
        .callable_definitions
        .get(row_index)
        .ok_or_else(|| ValidationError::ReferentialIntegrity("callable-definition query product row is out of bounds".into()))?;
    if !callable_definition_row_matches_owner(row, owner) {
        return Err(ValidationError::ReferentialIntegrity(
            "callable-definition query product row owner differs from query owner".into(),
        )
        .into());
    }
    row.write(&mut out)?;
    let digest = out.into_digest();
    Ok(u64::from_le_bytes(
        digest.0[..8].try_into().expect("digest prefix is eight bytes"),
    ))
}

fn type_member_row_matches_owner(
    row: &crate::semantic::world::TypeMemberDeclarationRecordV1,
    owner: &StableTypeMemberRef,
) -> bool {
    if &row.owner != &owner.owner {
        return false;
    }
    match &row.selector {
        crate::semantic::type_store::TypeMemberSelectorV1::Named { name } => name.as_ref() == owner.selector.as_ref(),
        crate::semantic::type_store::TypeMemberSelectorV1::Subscript => owner.selector.as_ref() == "[]",
    }
}

fn portable_type_member_digest64(
    domain: &str,
    row_index: usize,
    owner: &StableTypeMemberRef,
    world: &SemanticWorldV3,
) -> Result<u64, ArtifactError> {
    let mut out = BinaryWriter::digest(domain);
    out.write_u16(PortableQueryKindV1::TypeMemberDeclaration as u16);
    let row = world
        .type_member_declarations
        .get(row_index)
        .ok_or_else(|| ValidationError::ReferentialIntegrity("type-member query product row is out of bounds".into()))?;
    if !type_member_row_matches_owner(row, owner) {
        return Err(ValidationError::ReferentialIntegrity("type-member query product row owner differs from query owner".into()).into());
    }
    row.write(&mut out)?;
    let digest = out.into_digest();
    Ok(u64::from_le_bytes(
        digest.0[..8].try_into().expect("digest prefix is eight bytes"),
    ))
}

fn write_structural_product(
    out: &mut BinaryWriter,
    kind: PortableQueryKindV1,
    row_index: usize,
    owner: &StableDeclarationRef,
    world: &SemanticWorldV3,
) -> Result<(), ArtifactError> {
    out.write_u16(kind as u16);
    match kind {
        PortableQueryKindV1::DataDeclaration => {
            let row = world
                .structural_contracts
                .data_contracts
                .get(row_index)
                .ok_or_else(|| ValidationError::ReferentialIntegrity("data query product row is out of bounds".into()))?;
            if &row.owner != owner {
                return Err(ValidationError::ReferentialIntegrity("data query product row owner differs from query owner".into()).into());
            }
            row.write(out)?;
        }
        PortableQueryKindV1::EnumDeclaration => {
            let row = world
                .structural_contracts
                .enum_contracts
                .get(row_index)
                .ok_or_else(|| ValidationError::ReferentialIntegrity("enum query product row is out of bounds".into()))?;
            if &row.owner != owner {
                return Err(ValidationError::ReferentialIntegrity("enum query product row owner differs from query owner".into()).into());
            }
            row.write(out)?;
            out.write_u32(u32::try_from(row.variants.len()).map_err(|_| ValidationError::ReferentialIntegrity("enum variant count exceeds u32".into()))?);
            for variant in &row.variants {
                let mut matches = world.structural_contracts.variant_contracts.iter().filter(|candidate| candidate.variant == *variant);
                let variant_row = matches
                    .next()
                    .ok_or_else(|| ValidationError::ReferentialIntegrity("enum query product references a missing variant row".into()))?;
                if matches.next().is_some() {
                    return Err(ValidationError::ReferentialIntegrity("enum query product variant row is ambiguous".into()).into());
                }
                variant_row.write(out)?;
            }
        }
        PortableQueryKindV1::AssociatedSurface => {
            let row = world
                .associated_surfaces
                .get(row_index)
                .ok_or_else(|| ValidationError::ReferentialIntegrity("associated-surface query product row is out of bounds".into()))?;
            if &row.owner != owner {
                return Err(ValidationError::ReferentialIntegrity(
                    "associated-surface query product row owner differs from query owner".into(),
                )
                .into());
            }
            row.write(out)?;
        }
        _ => {
            return Err(ValidationError::ReferentialIntegrity(
                "portable structural fingerprint requested for unsupported query family".into(),
            )
            .into());
        }
    }
    Ok(())
}

fn portable_structural_digest64(
    domain: &str,
    kind: PortableQueryKindV1,
    row_index: usize,
    owner: &StableDeclarationRef,
    world: &SemanticWorldV3,
) -> Result<u64, ArtifactError> {
    let mut out = BinaryWriter::digest(domain);
    write_structural_product(&mut out, kind, row_index, owner, world)?;
    let digest = out.into_digest();
    Ok(u64::from_le_bytes(
        digest.0[..8].try_into().expect("digest prefix is eight bytes"),
    ))
}

/// Computes QUERY_FINGERPRINT_ABI_V2 evidence for structural query roots.
///
/// The returned values are portable artifact fingerprints. A receiving semantic
/// database must compute its own current-main input/product fingerprints after
/// typed reconstruction; graph-local DB fingerprints are never compared to
/// these wire fingerprints.
pub fn portable_structural_root_fingerprints(
    root: &QueryRootEntryV3,
    world: &SemanticWorldV3,
) -> Result<Option<(u64, u64)>, ArtifactError> {
    if !matches!(
        root.key.kind,
        PortableQueryKindV1::DataDeclaration
            | PortableQueryKindV1::EnumDeclaration
            | PortableQueryKindV1::AssociatedSurface
            | PortableQueryKindV1::TypeMemberDeclaration
            | PortableQueryKindV1::CallableDefinition
    ) {
        return Ok(None);
    }
    if root.product.kind != root.key.kind {
        return Err(ValidationError::ReferentialIntegrity("query root product family differs from its key kind".into()).into());
    }
    let row_index = root.product.row_index as usize;
    if root.key.kind == PortableQueryKindV1::TypeMemberDeclaration {
        let owner = type_member_owner(&root.key.owner)?;
        return Ok(Some((
            portable_type_member_digest64(DOMAIN_STRUCTURAL_QUERY_INPUT_V2, row_index, owner, world)?,
            portable_type_member_digest64(DOMAIN_STRUCTURAL_QUERY_PRODUCT_V2, row_index, owner, world)?,
        )));
    }
    if root.key.kind == PortableQueryKindV1::CallableDefinition {
        let owner = callable_owner(&root.key.owner)?;
        return Ok(Some((
            portable_callable_definition_digest64(DOMAIN_STRUCTURAL_QUERY_INPUT_V2, row_index, owner, world)?,
            portable_callable_definition_digest64(DOMAIN_STRUCTURAL_QUERY_PRODUCT_V2, row_index, owner, world)?,
        )));
    }
    let owner = declaration_owner(&root.key.owner)?;
    Ok(Some((
        portable_structural_digest64(DOMAIN_STRUCTURAL_QUERY_INPUT_V2, root.key.kind, row_index, owner, world)?,
        portable_structural_digest64(DOMAIN_STRUCTURAL_QUERY_PRODUCT_V2, root.key.kind, row_index, owner, world)?,
    )))
}

/// Computes portable structural product evidence for a dependency-only key.
///
/// Dependency evidence has no row index, so the semantic row is selected by
/// exact stable declaration owner. Duplicate/missing rows fail closed.
pub fn portable_structural_dependency_product_fingerprint(
    key: &PortableQueryKeyV1,
    world: &SemanticWorldV3,
) -> Result<Option<u64>, ArtifactError> {
    if !matches!(
        key.kind,
        PortableQueryKindV1::DataDeclaration
            | PortableQueryKindV1::EnumDeclaration
            | PortableQueryKindV1::AssociatedSurface
            | PortableQueryKindV1::TypeMemberDeclaration
            | PortableQueryKindV1::CallableDefinition
    ) {
        return Ok(None);
    }
    if key.kind == PortableQueryKindV1::TypeMemberDeclaration {
        let owner = type_member_owner(&key.owner)?;
        let mut matches = world
            .type_member_declarations
            .iter()
            .enumerate()
            .filter(|(_, row)| type_member_row_matches_owner(row, owner));
        let (row_index, _) = matches
            .next()
            .ok_or_else(|| ValidationError::ReferentialIntegrity("type-member dependency evidence has no semantic product row".into()))?;
        if matches.next().is_some() {
            return Err(ValidationError::ReferentialIntegrity(
                "type-member dependency evidence semantic owner is ambiguous".into(),
            )
            .into());
        }
        return Ok(Some(portable_type_member_digest64(
            DOMAIN_STRUCTURAL_QUERY_PRODUCT_V2,
            row_index,
            owner,
            world,
        )?));
    }
    if key.kind == PortableQueryKindV1::CallableDefinition {
        let owner = callable_owner(&key.owner)?;
        let mut matches = world
            .callable_definitions
            .iter()
            .enumerate()
            .filter(|(_, row)| callable_definition_row_matches_owner(row, owner));
        let (row_index, _) = matches
            .next()
            .ok_or_else(|| ValidationError::ReferentialIntegrity("callable-definition dependency evidence has no semantic product row".into()))?;
        if matches.next().is_some() {
            return Err(ValidationError::ReferentialIntegrity(
                "callable-definition dependency evidence semantic owner is ambiguous".into(),
            )
            .into());
        }
        return Ok(Some(portable_callable_definition_digest64(
            DOMAIN_STRUCTURAL_QUERY_PRODUCT_V2,
            row_index,
            owner,
            world,
        )?));
    }
    let owner = declaration_owner(&key.owner)?;
    let row_index = match key.kind {
        PortableQueryKindV1::DataDeclaration => {
            let mut matches = world.structural_contracts.data_contracts.iter().enumerate().filter(|(_, row)| &row.owner == owner);
            let (index, _) = matches
                .next()
                .ok_or_else(|| ValidationError::ReferentialIntegrity("data dependency evidence has no structural product row".into()))?;
            if matches.next().is_some() {
                return Err(ValidationError::ReferentialIntegrity("data dependency evidence structural owner is ambiguous".into()).into());
            }
            index
        }
        PortableQueryKindV1::EnumDeclaration => {
            let mut matches = world.structural_contracts.enum_contracts.iter().enumerate().filter(|(_, row)| &row.owner == owner);
            let (index, _) = matches
                .next()
                .ok_or_else(|| ValidationError::ReferentialIntegrity("enum dependency evidence has no structural product row".into()))?;
            if matches.next().is_some() {
                return Err(ValidationError::ReferentialIntegrity("enum dependency evidence structural owner is ambiguous".into()).into());
            }
            index
        }
        PortableQueryKindV1::AssociatedSurface => {
            let mut matches = world.associated_surfaces.iter().enumerate().filter(|(_, row)| &row.owner == owner);
            let (index, _) = matches
                .next()
                .ok_or_else(|| ValidationError::ReferentialIntegrity("associated-surface dependency evidence has no semantic product row".into()))?;
            if matches.next().is_some() {
                return Err(ValidationError::ReferentialIntegrity(
                    "associated-surface dependency evidence owner is ambiguous".into(),
                )
                .into());
            }
            index
        }
        _ => unreachable!("guarded structural kind"),
    };
    Ok(Some(portable_structural_digest64(
        DOMAIN_STRUCTURAL_QUERY_PRODUCT_V2,
        key.kind,
        row_index,
        owner,
        world,
    )?))
}

impl QueryImageV3 {
    /// Recomputes QUERY_FINGERPRINT_ABI_V2 evidence for the structural query
    /// families currently covered by the portable contract.
    ///
    /// Artifact production should call this after semantic rows and query keys
    /// are finalized and before encoding. Unsupported families are left
    /// untouched until their own portable fingerprint contract is defined.
    pub fn refresh_portable_structural_fingerprints(&mut self, world: &SemanticWorldV3) -> Result<(), ArtifactError> {
        for root in &mut self.supported_roots {
            if let Some((input, product)) = portable_structural_root_fingerprints(root, world)? {
                root.input_fingerprint = input;
                root.product_fingerprint = product;
            }
        }
        for evidence in &mut self.dependency_evidence {
            if matches!(
                evidence.key.kind,
                PortableQueryKindV1::DataDeclaration
                    | PortableQueryKindV1::EnumDeclaration
                    | PortableQueryKindV1::AssociatedSurface
                    | PortableQueryKindV1::TypeMemberDeclaration
                    | PortableQueryKindV1::CallableDefinition
            ) {
                if evidence.key.owner != evidence.stable_aggregate_owner {
                    return Err(ValidationError::ReferentialIntegrity(
                        "structural dependency evidence key and aggregate owner differ".into(),
                    )
                    .into());
                }
                evidence.product_fingerprint = portable_structural_dependency_product_fingerprint(&evidence.key, world)?
                    .ok_or_else(|| ValidationError::ReferentialIntegrity("missing portable structural dependency fingerprint".into()))?;
            }
        }
        Ok(())
    }

    /// Validates QUERY_FINGERPRINT_ABI_V2 structural evidence against the
    /// canonical semantic rows carried by the same artifact.
    ///
    /// This check is independent of any receiving workspace graph. It therefore
    /// belongs to cross-section artifact validation and may run before live
    /// semantic IDs are reconstructed.
    pub fn validate_portable_structural_fingerprints(&self, world: &SemanticWorldV3) -> Result<(), ArtifactError> {
        for root in &self.supported_roots {
            if let Some((input, product)) = portable_structural_root_fingerprints(root, world)? {
                if root.input_fingerprint != input {
                    return Err(ValidationError::ReferentialIntegrity(format!(
                        "query root {:?} portable input fingerprint does not match its semantic product",
                        root.key.kind
                    ))
                    .into());
                }
                if root.product_fingerprint != product {
                    return Err(ValidationError::ReferentialIntegrity(format!(
                        "query root {:?} portable product fingerprint does not match its semantic product",
                        root.key.kind
                    ))
                    .into());
                }
            }
        }

        for evidence in &self.dependency_evidence {
            if matches!(
                evidence.key.kind,
                PortableQueryKindV1::DataDeclaration
                    | PortableQueryKindV1::EnumDeclaration
                    | PortableQueryKindV1::AssociatedSurface
                    | PortableQueryKindV1::TypeMemberDeclaration
                    | PortableQueryKindV1::CallableDefinition
            ) {
                if evidence.key.owner != evidence.stable_aggregate_owner {
                    return Err(ValidationError::ReferentialIntegrity(
                        "structural dependency evidence key and aggregate owner differ".into(),
                    )
                    .into());
                }
                let expected = portable_structural_dependency_product_fingerprint(&evidence.key, world)?
                    .ok_or_else(|| ValidationError::ReferentialIntegrity("missing portable structural dependency fingerprint".into()))?;
                if evidence.product_fingerprint != expected {
                    return Err(ValidationError::ReferentialIntegrity(format!(
                        "query dependency {:?} portable product fingerprint does not match its semantic product",
                        evidence.key.kind
                    ))
                    .into());
                }
            }
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let mut out = BinaryWriter::new();
        out.write_u32(QUERY_IMAGE_SCHEMA_V3);
        out.write_u32(self.supported_roots.len() as u32);
        for root in &self.supported_roots {
            root.key.write(&mut out)?;
            out.write_u16(root.product.kind as u16);
            out.write_u32(root.product.row_index);
            out.write_u64(root.input_fingerprint);
            out.write_u64(root.product_fingerprint);
            out.write_u32(root.dependency_indices.len() as u32);
            for &dep in &root.dependency_indices {
                out.write_u32(dep);
            }
        }
        out.write_u32(self.dependency_evidence.len() as u32);
        for ev in &self.dependency_evidence {
            ev.key.write(&mut out)?;
            ev.stable_aggregate_owner.write(&mut out)?;
            out.write_u64(ev.product_fingerprint);
        }
        Ok(out.into_vec())
    }

    pub fn decode_with_allocation_budget(bytes: &[u8], limits: &ValidationLimits, transaction: &mut AllocationBudget) -> Result<Self, ArtifactError> {
        let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut allocation_budget = transaction.child(section_limit);
        let image = {
            let mut input = Decode {
                reader: BinaryReader::new(bytes),
                limits: *limits,
                allocations: &mut allocation_budget,
                items_left: limits.max_semantic_entries_per_family,
                depth: 0,
            };
            let schema = input.reader.read_u32()?;
            if schema != QUERY_IMAGE_SCHEMA_V3 {
                return Err(ValidationError::IncompatibleAbi {
                    kind: "query_image_schema",
                    found: schema,
                    expected: QUERY_IMAGE_SCHEMA_V3,
                }
                .into());
            }
            let root_count = input.reader.read_u32()? as usize;
            if root_count > limits.max_semantic_entries_per_family {
                return Err(ArtifactError::LimitExceeded("too many query roots".into()));
            }
            let mut supported_roots = reserve_vec::<QueryRootEntryV3>(input.allocations, root_count)?;
            for _ in 0..root_count {
                let key = PortableQueryKeyV1::read(&mut input)?;
                let product_kind_raw = input.reader.read_u16()?;
                let product_kind = PortableQueryKindV1::from_raw(product_kind_raw)
                    .ok_or_else(|| ValidationError::ReferentialIntegrity("unknown query product kind".into()))?;
                let row_index = input.reader.read_u32()?;
                let input_fingerprint = input.reader.read_u64()?;
                let product_fingerprint = input.reader.read_u64()?;
                let dep_count = input.reader.read_u32()? as usize;
                if dep_count > limits.max_semantic_entries_per_family {
                    return Err(ArtifactError::LimitExceeded("too many query dependencies".into()));
                }
                let mut dependency_indices = reserve_vec::<u32>(input.allocations, dep_count)?;
                for _ in 0..dep_count {
                    dependency_indices.push(input.reader.read_u32()?);
                }
                supported_roots.push(QueryRootEntryV3 {
                    key,
                    product: QueryProductRefV2 { kind: product_kind, row_index },
                    input_fingerprint,
                    product_fingerprint,
                    dependency_indices,
                });
            }
            let ev_count = input.reader.read_u32()? as usize;
            if ev_count > limits.max_semantic_entries_per_family {
                return Err(ArtifactError::LimitExceeded("too many dependency evidence entries".into()));
            }
            let mut dependency_evidence = reserve_vec::<DependencyEvidenceEntryV3>(input.allocations, ev_count)?;
            for _ in 0..ev_count {
                let key = PortableQueryKeyV1::read(&mut input)?;
                let stable_aggregate_owner = PortableQueryOwnerKeyV1::read(&mut input)?;
                let product_fingerprint = input.reader.read_u64()?;
                dependency_evidence.push(DependencyEvidenceEntryV3 {
                    key,
                    stable_aggregate_owner,
                    product_fingerprint,
                });
            }
            if input.reader.has_remaining() {
                return Err(ValidationError::ReferentialIntegrity("trailing query image bytes".into()).into());
            }
            QueryImageV3 {
                supported_roots,
                dependency_evidence,
            }
        };
        image.validate(limits)?;
        transaction.commit_child(allocation_budget).map_err(allocation_error)?;
        Ok(image)
    }

    pub fn decode_with_budget(bytes: &[u8], limits: &ValidationLimits, remaining_budget: &mut u64) -> Result<Self, ArtifactError> {
        let available = usize::try_from(*remaining_budget).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut transaction = AllocationBudget::new(available);
        let image = Self::decode_with_allocation_budget(bytes, limits, &mut transaction)?;
        *remaining_budget = u64::try_from(transaction.remaining()).map_err(|_| allocation_error(AllocationError::Overflow))?;
        Ok(image)
    }

    pub fn decode(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        let mut budget = limits.max_decoded_section_bytes;
        Self::decode_with_budget(bytes, limits, &mut budget)
    }

    pub fn validate(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        if self.supported_roots.len() > limits.max_semantic_entries_per_family {
            return Err(ArtifactError::LimitExceeded("query roots exceed limit".into()));
        }
        if self.dependency_evidence.len() > limits.max_semantic_entries_per_family {
            return Err(ArtifactError::LimitExceeded("query dependency evidence exceeds limit".into()));
        }
        for adjacent in self.supported_roots.windows(2) {
            if adjacent[0].key == adjacent[1].key {
                return Err(ValidationError::ReferentialIntegrity(format!("duplicate query root key: {:?}", adjacent[1].key)).into());
            }
            if adjacent[0].key > adjacent[1].key {
                return Err(ValidationError::ReferentialIntegrity("query roots are not in canonical key order".into()).into());
            }
        }
        for root in &self.supported_roots {
            if root.product.kind != root.key.kind {
                return Err(ValidationError::ReferentialIntegrity("query root product family differs from its key kind".into()).into());
            }
            if !matches!(
                root.product.kind.profile_support(),
                QueryProfileSupport::RepresentedByModuleWorld | QueryProfileSupport::RepresentedBySemanticWorld
            ) {
                return Err(ValidationError::ReferentialIntegrity("query kind is not reusable in the canonical artifact profile".into()).into());
            }
            if matches!(root.key.owner.project(), StableProjectRef::Session { .. }) {
                return Err(ValidationError::ReferentialIntegrity("session project query owner is forbidden".into()).into());
            }
            for adjacent in root.dependency_indices.windows(2) {
                if adjacent[0] >= adjacent[1] {
                    return Err(ValidationError::ReferentialIntegrity("query dependency indices are not in canonical unique order".into()).into());
                }
            }
        }
        for adjacent in self.dependency_evidence.windows(2) {
            if adjacent[0] >= adjacent[1] {
                return Err(ValidationError::ReferentialIntegrity("query dependency evidence is not in canonical unique order".into()).into());
            }
        }
        for ev in &self.dependency_evidence {
            if matches!(ev.key.owner.project(), StableProjectRef::Session { .. })
                || matches!(ev.stable_aggregate_owner.project(), StableProjectRef::Session { .. })
            {
                return Err(ValidationError::ReferentialIntegrity("session project in dependency evidence is forbidden".into()).into());
            }
        }
        for root in &self.supported_roots {
            for &dep in &root.dependency_indices {
                if (dep as usize) >= self.dependency_evidence.len() {
                    return Err(ValidationError::ReferentialIntegrity(format!(
                        "query dependency index {dep} is out of bounds (evidence count {})",
                        self.dependency_evidence.len()
                    ))
                    .into());
                }
            }
        }
        Ok(())
    }

    /// Validates typed query product references against the artifact sections.
    pub fn validate_product_references(&self, modules: &ModuleWorldV2, world: &SemanticWorldV3) -> Result<(), ArtifactError> {
        for root in &self.supported_roots {
            let row_count = match root.product.kind.profile_support() {
                QueryProfileSupport::RepresentedByModuleWorld => modules.modules.len(),
                QueryProfileSupport::RepresentedBySemanticWorld => semantic_product_row_count(root.product.kind, world)
                    .ok_or_else(|| ValidationError::ReferentialIntegrity("query kind has no semantic product table".into()))?,
                QueryProfileSupport::OmittedSourceDependent
                | QueryProfileSupport::AggregateDependencyEvidenceOnly
                | QueryProfileSupport::OmittedAdvisoryOnly => {
                    return Err(ValidationError::ReferentialIntegrity("query product reference targets a non-root profile kind".into()).into());
                }
            };
            if root.product.row_index as usize >= row_count {
                return Err(ValidationError::ReferentialIntegrity(format!(
                    "query product row {} is out of bounds for {:?} (row count {row_count})",
                    root.product.row_index, root.product.kind
                ))
                .into());
            }

            // Verify owner agreement for module-world products
            if root.product.kind.profile_support() == QueryProfileSupport::RepresentedByModuleWorld {
                let module = &modules.modules[root.product.row_index as usize];
                if root.key.owner.module_path() != module.identity.path.as_ref()
                    || root.key.owner.project() != &module.identity.project
                {
                    return Err(ValidationError::ReferentialIntegrity(format!(
                        "query root owner {:?} does not match referenced module {:?}",
                        root.key.owner, module.identity
                    ))
                    .into());
                }
            }
        }
        Ok(())
    }
}

impl QueryImageV2 {
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let mut out = BinaryWriter::new();
        out.write_u32(QUERY_IMAGE_SCHEMA_V2);
        out.write_u32(self.supported_roots.len() as u32);
        for root in &self.supported_roots {
            out.write_u16(root.key.kind as u16);
            out.write_str(&root.key.stable_owner_key);
            out.write_u16(root.product.kind as u16);
            out.write_u32(root.product.row_index);
            out.write_u64(root.input_fingerprint);
            out.write_u64(root.product_fingerprint);
            out.write_u32(root.dependency_indices.len() as u32);
            for &dep in &root.dependency_indices {
                out.write_u32(dep);
            }
        }
        out.write_u32(self.dependency_evidence.len() as u32);
        for ev in &self.dependency_evidence {
            out.write_u16(ev.key.kind as u16);
            out.write_str(&ev.key.stable_owner_key);
            out.write_str(&ev.stable_aggregate_owner);
            out.write_u64(ev.product_fingerprint);
        }
        Ok(out.into_vec())
    }

    pub fn decode_with_allocation_budget(bytes: &[u8], limits: &ValidationLimits, transaction: &mut AllocationBudget) -> Result<Self, ArtifactError> {
        // The input is borrowed. Charge only allocations made while constructing
        // the DTO. A child ledger enforces the section limit independently of
        // the parent transaction and commits only after complete validation.
        let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut allocation_budget = transaction.child(section_limit);
        let mut reader = BinaryReader::new(bytes);
        let schema = reader.read_u32()?;
        if schema != QUERY_IMAGE_SCHEMA_V2 {
            return Err(ValidationError::ReferentialIntegrity("unsupported query image schema".into()).into());
        }
        let root_count = reader.read_u32()? as usize;
        if root_count > limits.max_semantic_entries_per_family {
            return Err(ArtifactError::LimitExceeded("too many query roots".into()));
        }
        let mut supported_roots = reserve_vec::<QueryRootEntryV2>(&mut allocation_budget, root_count)?;
        for _ in 0..root_count {
            let kind_raw = reader.read_u16()?;
            let kind = PortableQueryKindV1::from_raw(kind_raw).ok_or_else(|| ValidationError::ReferentialIntegrity("unknown query kind".into()))?;
            let stable_owner_key = read_owned_string(&mut reader, limits, &mut allocation_budget)?;
            let product_kind_raw = reader.read_u16()?;
            let product_kind =
                PortableQueryKindV1::from_raw(product_kind_raw).ok_or_else(|| ValidationError::ReferentialIntegrity("unknown query product kind".into()))?;
            let row_index = reader.read_u32()?;
            let input_fingerprint = reader.read_u64()?;
            let product_fingerprint = reader.read_u64()?;
            let dep_count = reader.read_u32()? as usize;
            if dep_count > limits.max_semantic_entries_per_family {
                return Err(ArtifactError::LimitExceeded("too many query dependencies".into()));
            }
            let mut dependency_indices = reserve_vec::<u32>(&mut allocation_budget, dep_count)?;
            for _ in 0..dep_count {
                dependency_indices.push(reader.read_u32()?);
            }
            supported_roots.push(QueryRootEntryV2 {
                key: LegacyPortableQueryKeyV1 { kind, stable_owner_key },
                product: QueryProductRefV2 { kind: product_kind, row_index },
                input_fingerprint,
                product_fingerprint,
                dependency_indices,
            });
        }
        let ev_count = reader.read_u32()? as usize;
        if ev_count > limits.max_semantic_entries_per_family {
            return Err(ArtifactError::LimitExceeded("too many dependency evidence entries".into()));
        }
        let mut dependency_evidence = reserve_vec::<DependencyEvidenceEntryV2>(&mut allocation_budget, ev_count)?;
        for _ in 0..ev_count {
            let kind_raw = reader.read_u16()?;
            let kind = PortableQueryKindV1::from_raw(kind_raw).ok_or_else(|| ValidationError::ReferentialIntegrity("unknown query kind".into()))?;
            let stable_owner_key = read_owned_string(&mut reader, limits, &mut allocation_budget)?;
            let stable_aggregate_owner = read_owned_string(&mut reader, limits, &mut allocation_budget)?;
            let product_fingerprint = reader.read_u64()?;
            dependency_evidence.push(DependencyEvidenceEntryV2 {
                key: LegacyPortableQueryKeyV1 { kind, stable_owner_key },
                stable_aggregate_owner,
                product_fingerprint,
            });
        }
        if reader.has_remaining() {
            return Err(ValidationError::ReferentialIntegrity("trailing query image bytes".into()).into());
        }
        let image = Self {
            supported_roots,
            dependency_evidence,
        };
        image.validate(limits)?;
        transaction.commit_child(allocation_budget).map_err(allocation_error)?;
        Ok(image)
    }

    pub fn decode_with_budget(bytes: &[u8], limits: &ValidationLimits, remaining_budget: &mut u64) -> Result<Self, ArtifactError> {
        let available = usize::try_from(*remaining_budget).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut transaction = AllocationBudget::new(available);
        let image = Self::decode_with_allocation_budget(bytes, limits, &mut transaction)?;
        *remaining_budget = u64::try_from(transaction.remaining()).map_err(|_| allocation_error(AllocationError::Overflow))?;
        Ok(image)
    }

    pub fn decode(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        let mut budget = limits.max_decoded_section_bytes;
        Self::decode_with_budget(bytes, limits, &mut budget)
    }

    pub fn validate(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        if self.supported_roots.len() > limits.max_semantic_entries_per_family {
            return Err(ArtifactError::LimitExceeded("query roots exceed limit".into()));
        }
        if self.dependency_evidence.len() > limits.max_semantic_entries_per_family {
            return Err(ArtifactError::LimitExceeded("query dependency evidence exceeds limit".into()));
        }
        for adjacent in self.supported_roots.windows(2) {
            if adjacent[0].key == adjacent[1].key {
                return Err(ValidationError::ReferentialIntegrity(format!("duplicate query root key: {:?}", adjacent[1].key)).into());
            }
            if adjacent[0].key > adjacent[1].key {
                return Err(ValidationError::ReferentialIntegrity("query roots are not in canonical key order".into()).into());
            }
        }
        for root in &self.supported_roots {
            if root.product.kind != root.key.kind {
                return Err(ValidationError::ReferentialIntegrity("query root product family differs from its key kind".into()).into());
            }
            if !matches!(
                root.product.kind.profile_support(),
                QueryProfileSupport::RepresentedByModuleWorld | QueryProfileSupport::RepresentedBySemanticWorld
            ) {
                return Err(ValidationError::ReferentialIntegrity("query kind is not reusable in the canonical artifact profile".into()).into());
            }
            for adjacent in root.dependency_indices.windows(2) {
                if adjacent[0] >= adjacent[1] {
                    return Err(ValidationError::ReferentialIntegrity("query dependency indices are not in canonical unique order".into()).into());
                }
            }
        }
        for adjacent in self.dependency_evidence.windows(2) {
            if adjacent[0] >= adjacent[1] {
                return Err(ValidationError::ReferentialIntegrity("query dependency evidence is not in canonical unique order".into()).into());
            }
        }
        for root in &self.supported_roots {
            for &dep in &root.dependency_indices {
                if (dep as usize) >= self.dependency_evidence.len() {
                    return Err(ValidationError::ReferentialIntegrity(format!(
                        "query dependency index {dep} is out of bounds (evidence count {})",
                        self.dependency_evidence.len()
                    ))
                    .into());
                }
            }
        }
        Ok(())
    }

    /// Validates typed query product references against the artifact sections.
    pub fn validate_product_references(&self, modules: &ModuleWorldV1, world: &SemanticWorldV3) -> Result<(), ArtifactError> {
        for root in &self.supported_roots {
            let row_count = match root.product.kind.profile_support() {
                QueryProfileSupport::RepresentedByModuleWorld => modules.modules.len(),
                QueryProfileSupport::RepresentedBySemanticWorld => semantic_product_row_count(root.product.kind, world)
                    .ok_or_else(|| ValidationError::ReferentialIntegrity("query kind has no semantic product table".into()))?,
                QueryProfileSupport::OmittedSourceDependent
                | QueryProfileSupport::AggregateDependencyEvidenceOnly
                | QueryProfileSupport::OmittedAdvisoryOnly => {
                    return Err(ValidationError::ReferentialIntegrity("query product reference targets a non-root profile kind".into()).into());
                }
            };
            if root.product.row_index as usize >= row_count {
                return Err(ValidationError::ReferentialIntegrity(format!(
                    "query product row {} is out of bounds for {:?} (row count {row_count})",
                    root.product.row_index, root.product.kind
                ))
                .into());
            }
        }
        Ok(())
    }
}

fn semantic_product_row_count(kind: PortableQueryKindV1, world: &SemanticWorldV3) -> Option<usize> {
    Some(match kind {
        PortableQueryKindV1::DeclarationShell => world.structural_contracts.declaration_shells.len(),
        PortableQueryKindV1::HierarchyEdge => world.structural_contracts.hierarchy_and_aliases.len(),
        PortableQueryKindV1::TraitHeader | PortableQueryKindV1::TraitSurface => world.trait_surfaces.len(),
        PortableQueryKindV1::DeclarationSurface | PortableQueryKindV1::FieldSignature => world.dispatch_surfaces.len(),
        PortableQueryKindV1::ConformanceDispatch => world.conformance_heads.len(),
        PortableQueryKindV1::TraitDispatchBucket | PortableQueryKindV1::GenericConformance => world.witness_plans.len(),
        PortableQueryKindV1::CallableDefinition => world.callable_definitions.len(),
        PortableQueryKindV1::ConstructorInitialization => world.constructor_initializations.len(),
        PortableQueryKindV1::CallableSignature => world.callable_signatures.len(),
        PortableQueryKindV1::CallableEffects => world.callable_signatures.len(),
        PortableQueryKindV1::CallableBody
        | PortableQueryKindV1::CallableControl
        | PortableQueryKindV1::CallableTermination
        | PortableQueryKindV1::CallableContracts
        | PortableQueryKindV1::AttributeArgument
        | PortableQueryKindV1::VerificationConditions => world.callable_analyses.len(),
        PortableQueryKindV1::DataDeclaration => world.structural_contracts.data_contracts.len(),
        PortableQueryKindV1::EnumDeclaration => world.structural_contracts.enum_contracts.len(),
        PortableQueryKindV1::EnumRequirements => world.enum_requirements.len(),
        PortableQueryKindV1::TypeMemberDeclaration => world.type_member_declarations.len(),
        PortableQueryKindV1::StaticDomainCaseCatalog => world.static_case_catalogs.len(),
        PortableQueryKindV1::StaticCaseSignature => world.static_case_signatures.len(),
        PortableQueryKindV1::AssociatedStaticDomainDeclaration => world.associated_static_domains.len(),
        PortableQueryKindV1::AssociatedSurface => world.associated_surfaces.len(),
        PortableQueryKindV1::StaticDefinition | PortableQueryKindV1::CheckedStaticBody => world.static_definitions.len(),
        PortableQueryKindV1::StaticConstant => world.static_constants.len(),
        PortableQueryKindV1::ModuleStaticBindingCatalog | PortableQueryKindV1::ModuleStaticBindings => world.module_static_bindings.len(),
        PortableQueryKindV1::ParsedModule
        | PortableQueryKindV1::UnlinkedInterface
        | PortableQueryKindV1::ResolvedImports
        | PortableQueryKindV1::LinkedInterface
        | PortableQueryKindV1::SemanticComponent
        | PortableQueryKindV1::ModuleDiagnostics
        | PortableQueryKindV1::ModuleMetadata
        | PortableQueryKindV1::SourceStructure
        | PortableQueryKindV1::SourceFormalAttachment
        | PortableQueryKindV1::AdvisoryCallable
        | PortableQueryKindV1::AdvisoryModule
        | PortableQueryKindV1::ResolvedImport
        | PortableQueryKindV1::LinkedName
        | PortableQueryKindV1::PublicExport => return None,
    })
}

fn allocation_error(error: AllocationError) -> ArtifactError {
    ArtifactError::Allocation(error)
}

fn reserve_vec<T>(remaining: &mut AllocationBudget, count: usize) -> Result<Vec<T>, ArtifactError> {
    remaining.vector(count).map_err(allocation_error)
}

fn read_owned_string(reader: &mut BinaryReader<'_>, limits: &ValidationLimits, remaining: &mut AllocationBudget) -> Result<String, ArtifactError> {
    let value = reader.read_str(limits.max_string_bytes)?;
    remaining.string(value).map_err(allocation_error)
}
