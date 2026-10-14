//! Portable `QUERY_IMAGE` wire DTOs and exhaustive profile dispositions (`PERF003.C2.P7.T9`).

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
