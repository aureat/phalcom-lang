//! Portable `QUERY_IMAGE` wire DTOs and exhaustive profile dispositions (`PERF003.C2.P7.T9`).

use crate::codec::{BinaryReader, BinaryWriter};
use crate::error::{ArtifactError, ValidationError};
use crate::limits::ValidationLimits;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};

pub const QUERY_IMAGE_SCHEMA_V1: u32 = 1;

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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryProfileSupport {
    OmittedSourceDependent,
    SupportedReusableRoot,
    AggregateDependencyEvidenceOnly,
}

impl PortableQueryKindV1 {
    pub const fn profile_support(self) -> QueryProfileSupport {
        match self {
            Self::ParsedModule => QueryProfileSupport::OmittedSourceDependent,
            Self::HierarchyEdge => QueryProfileSupport::AggregateDependencyEvidenceOnly,
            _ => QueryProfileSupport::SupportedReusableRoot,
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
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PortableQueryKeyV1 {
    pub kind: PortableQueryKindV1,
    pub stable_owner_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryRootEntryV1 {
    pub key: PortableQueryKeyV1,
    pub dependency_indices: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyEvidenceEntryV1 {
    pub key: PortableQueryKeyV1,
    pub stable_aggregate_owner: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct QueryImageV1 {
    pub supported_roots: Vec<QueryRootEntryV1>,
    pub dependency_evidence: Vec<DependencyEvidenceEntryV1>,
}

impl QueryImageV1 {
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let mut out = BinaryWriter::new();
        out.write_u32(QUERY_IMAGE_SCHEMA_V1);
        out.write_u32(self.supported_roots.len() as u32);
        for root in &self.supported_roots {
            out.write_u16(root.key.kind as u16);
            out.write_str(&root.key.stable_owner_key);
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
        if schema != QUERY_IMAGE_SCHEMA_V1 {
            return Err(ValidationError::ReferentialIntegrity("unsupported query image schema".into()).into());
        }
        let root_count = reader.read_u32()? as usize;
        if root_count > limits.max_semantic_entries_per_family {
            return Err(ArtifactError::LimitExceeded("too many query roots".into()));
        }
        let mut supported_roots = reserve_vec::<QueryRootEntryV1>(&mut allocation_budget, root_count)?;
        for _ in 0..root_count {
            let kind_raw = reader.read_u16()?;
            let kind = PortableQueryKindV1::from_raw(kind_raw).ok_or_else(|| ValidationError::ReferentialIntegrity("unknown query kind".into()))?;
            let stable_owner_key = read_owned_string(&mut reader, limits, &mut allocation_budget)?;
            let dep_count = reader.read_u32()? as usize;
            if dep_count > limits.max_semantic_entries_per_family {
                return Err(ArtifactError::LimitExceeded("too many query dependencies".into()));
            }
            let mut dependency_indices = reserve_vec::<u32>(&mut allocation_budget, dep_count)?;
            for _ in 0..dep_count {
                dependency_indices.push(reader.read_u32()?);
            }
            supported_roots.push(QueryRootEntryV1 {
                key: PortableQueryKeyV1 { kind, stable_owner_key },
                dependency_indices,
            });
        }
        let ev_count = reader.read_u32()? as usize;
        if ev_count > limits.max_semantic_entries_per_family {
            return Err(ArtifactError::LimitExceeded("too many dependency evidence entries".into()));
        }
        let mut dependency_evidence = reserve_vec::<DependencyEvidenceEntryV1>(&mut allocation_budget, ev_count)?;
        for _ in 0..ev_count {
            let kind_raw = reader.read_u16()?;
            let kind = PortableQueryKindV1::from_raw(kind_raw).ok_or_else(|| ValidationError::ReferentialIntegrity("unknown query kind".into()))?;
            let stable_owner_key = read_owned_string(&mut reader, limits, &mut allocation_budget)?;
            let stable_aggregate_owner = read_owned_string(&mut reader, limits, &mut allocation_budget)?;
            dependency_evidence.push(DependencyEvidenceEntryV1 {
                key: PortableQueryKeyV1 { kind, stable_owner_key },
                stable_aggregate_owner,
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
        Ok(())
    }
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
