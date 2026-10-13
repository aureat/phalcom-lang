//! Canonical exporter: projects compiler-owned semantic types, signatures, and declarations
//! into a stable, deduplicated, versioned `SemanticMetadataBundle`.

use super::stable_identity::*;
use crate::data_semantics::DataSemanticTable;
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, DeclaredTypeState};
use crate::declarations::DeclarationTypeTable;
use crate::enum_semantics::EnumSemanticTable;
use crate::field_key::{FieldKey, FieldKeyKind};
use crate::identity::{AssociatedStaticDomainDeclarationId, CallableOwnerId, DispatchSide};
use crate::signature::{CallableSignatureTable, FieldSignatureTable};
use crate::type_alias::TypeAliasTable;
use crate::types::id::{AlgebraicStaticConstructorId, KindId, ScopedTypeId, StaticDomainId, TypeId, TypeParameterId};
use crate::types::kind::KindData;
use crate::types::parameter::{
    GenericConstraint, GenericParameterMode, GenericSignature, GenericSignaturePublicationError, SelfRole, TypeParameterOwner, TypeTerm,
};
use crate::types::rigid::LocalType;
use crate::types::store::{AnonymousTupleRemainder, StaticArgument, StaticDomainData, StructuralMemberPresence, TypeData, TypeStore};
use crate::types::type_lambda::ScopedTypeData;
use crate::types::variance::Variance;
use phalcom_type_meta::bundle::{RuntimeTypeFormKey, RuntimeTypeFormRoot, SemanticMetadataBundle};
use phalcom_type_meta::declaration::{
    AlgebraicDeclarationRecord, AlgebraicPhaseRef, AlgebraicShapeRef, CallableParameterRecord, CallableSemanticRecord, DeclarationTypeFlags,
    DeclarationTypeRecord, DynamicReasonRef, FieldMutabilityRef, FieldSemanticRecord, PublishedTypeAuthority, PublishedTypeSlot,
    RestModeRef, UnknownReasonRef,
};
use phalcom_type_meta::fingerprint::{Fingerprint128, FingerprintBuilder};
use phalcom_type_meta::generic::{
    GenericConstraintRef, GenericParameterModeRef, GenericSignatureRecord, GenericSignatureRecordId, StableAssociatedTypeRequirementRef,
    StableTraitApplicationRef, StableTypeParameterOwnerRef, StableTypeParameterRef, TypeParameterRecord, VarianceRef,
};
use phalcom_type_meta::header::{
    ArtifactIdentityScheme, MetadataFeatures, MetadataProfile, NATIVE_SURFACE_SCHEMA_VERSION, ProducerIdentity, SEMANTIC_MODEL_VERSION, SemanticMetadataHeader,
    TYPE_METADATA_SCHEMA_VERSION,
};
use phalcom_type_meta::identity::SourceSpanRef;
use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef, StableProjectRef, StableStaticDomainDeclarationRef};
use phalcom_type_meta::kind::{KindNode, KindNodeEntry, KindNodeId, StaticDomainRef};
use phalcom_type_meta::scoped_type::{
    ScopedAnonymousRecordRemainderRef, ScopedAnonymousTupleRemainderRef, ScopedCallableParamRef, ScopedCallableTypeRef, ScopedOpenRecordTypeRef,
    ScopedPackTypeRef, ScopedRecordFieldRef, ScopedRecordRowTailRef, ScopedRecordRowTypeRef, ScopedRecordTailRef, ScopedTupleElementRef, ScopedTypeNode,
    ScopedTypeNodeEntry, ScopedTypeNodeId, TypeLambdaRef,
};
use phalcom_type_meta::type_node::{
    AnonymousRecordRemainderRef, AnonymousRecordTypeRef, AnonymousTupleRemainderRef, CallableParamRef, CallableTypeRef, OpenRecordTypeRef, OpenTupleTypeRef,
    PackTypeRef, RecordFieldRef, RecordRowRef, SelfRoleRef, SelfTypeRef, StaticArgumentRef, StaticConstructorRef, StaticDataRef, StructuralMemberPresenceRef,
    TupleElementRef, TypeNode, TypeNodeEntry, TypeNodeId,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use super::reachability::MetadataReachabilityClosure;

macro_rules! metadata_perf_counter {
    ($counter:expr, $amount:expr $(,)?) => {{
        #[cfg(feature = "perf-observability")]
        telos_observability::PerfContext::current_thread().record_counter($counter, $amount as u64);
    }};
}

type GenericSignatureKey = (TypeParameterOwner, Box<[TypeParameterId]>, Box<[GenericConstraint]>);

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum MetadataExportError {
    #[error("cannot export inference variable in durable metadata")]
    InferenceVariable,
    #[error("metadata projection for {entity:?} (profile {profile:?}, type {type_id:?}): {source}")]
    Projection { profile: MetadataProfile, entity: Box<MetadataEntityIdentity>, type_id: Option<TypeId>, source: Box<MetadataExportError> },
    #[error("cannot export scoped local type containing rigid variables")]
    ScopedLocalType,
    #[error("cannot export invalid generic signature: {0:?}")]
    InvalidGenericSignature(GenericSignaturePublicationError),
    #[error("non-exportable internal form: {0:?}")]
    NonExportableForm(TypeId),
    #[error("legacy metadata cannot represent a {0:?} Record field key")]
    NonExportableRecordKey(FieldKeyKind),
    #[error("legacy metadata cannot represent a Record remainder outside the Symbol key domain")]
    NonExportableRecordKeyDomain,
    #[error("cannot export scoped pack form before stable P2 metadata lowering")]
    ScopedPackForm,
    #[error("resolved-project metadata requires ProjectUniverse identity context")]
    MissingProjectIdentityContext,
    #[error("resolved project {0} is absent from ProjectUniverse")]
    MissingResolvedProject(phalcom_modules::ResolvedProjectId),
    #[error("cannot export unknown static domain {0:?}")]
    InvalidStaticDomain(StaticDomainId),
    #[error("cannot export unknown static constructor {0:?}")]
    InvalidStaticConstructor(AlgebraicStaticConstructorId),
    #[error("cannot export invalid static-domain catalog for {domain:?}: {reason}")]
    InvalidStaticDomainCatalog { domain: StaticDomainId, reason: &'static str },
    #[error("cannot export static domain without its canonical declaration product: {0:?}")]
    MissingStaticDomainDeclaration(StableStaticDomainDeclarationRef),
    #[error("cannot export scoped type application with invalid kinding: {0}")]
    InvalidKindApplication(&'static str),
}

/// Top-level retained semantic entity responsible for a metadata projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetadataEntityIdentity {
    Declaration(crate::identity::DeclarationId),
    Alias(crate::identity::DeclarationId),
    Callable(crate::identity::CallableId),
    Field(crate::identity::FieldId),
    RuntimeType { module: crate::identity::ModuleId, key: Box<str> },
    StaticDomainCatalog,
}

fn write_stable_project_fingerprint(builder: &mut FingerprintBuilder, project: &StableProjectRef) {
    match project {
        StableProjectRef::Builtin { namespace, version } => {
            builder.write_u8(1);
            builder.write_str(namespace);
            builder.write_str(version);
        }
        StableProjectRef::Package {
            package,
            version,
            artifact_fingerprint,
        } => {
            builder.write_u8(2);
            builder.write_str(package);
            builder.write_str(version);
            builder.write_fingerprint(*artifact_fingerprint);
        }
        StableProjectRef::SourceArtifact {
            logical_uri,
            source_fingerprint,
        } => {
            builder.write_u8(3);
            builder.write_str(logical_uri);
            builder.write_fingerprint(*source_fingerprint);
        }
        StableProjectRef::Session { session_fingerprint } => {
            builder.write_u8(4);
            builder.write_fingerprint(*session_fingerprint);
        }
    }
}

fn write_stable_module_fingerprint(builder: &mut FingerprintBuilder, module: &StableModuleRef) {
    write_stable_project_fingerprint(builder, &module.project);
    builder.write_u32(module.path.len() as u32);
    for component in module.path.iter() {
        builder.write_str(component);
    }
}

fn write_stable_declaration_fingerprint(builder: &mut FingerprintBuilder, declaration: &StableDeclarationRef) {
    write_stable_module_fingerprint(builder, &declaration.module);
    builder.write_u32(declaration.path.len() as u32);
    for component in declaration.path.iter() {
        builder.write_str(component);
    }
}

fn write_stable_impl_fingerprint(builder: &mut FingerprintBuilder, r#impl: &phalcom_type_meta::identity::StableImplRef) {
    write_stable_module_fingerprint(builder, &r#impl.module);
    builder.write_u32(r#impl.local_id);
}

fn write_stable_callable_fingerprint(builder: &mut FingerprintBuilder, callable: &phalcom_type_meta::identity::StableCallableRef) {
    use phalcom_type_meta::identity::StableCallableOwnerRef;
    match &callable.owner {
        StableCallableOwnerRef::Declaration(owner) => {
            builder.write_u8(1);
            write_stable_declaration_fingerprint(builder, owner);
        }
        StableCallableOwnerRef::Variant(owner) => {
            builder.write_u8(2);
            write_stable_declaration_fingerprint(builder, &owner.owner);
            builder.write_str(&owner.selector);
        }
        StableCallableOwnerRef::Conformance(owner) => {
            builder.write_u8(3);
            write_stable_impl_fingerprint(builder, owner);
        }
    }
    builder.write_u8(match callable.side {
        phalcom_type_meta::identity::StableDispatchSide::Instance => 1,
        phalcom_type_meta::identity::StableDispatchSide::Class => 2,
    });
    builder.write_str(&callable.selector);
}

#[cfg(test)]
mod callable_owner_fingerprint_tests {
    use super::*;
    use phalcom_type_meta::identity::{StableCallableOwnerRef, StableCallableRef, StableDispatchSide, StableImplRef, StableVariantRef};

    #[test]
    fn callable_parameter_owner_fingerprints_do_not_collapse_owner_kinds() {
        let module = StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "universe".into(),
                version: "0.1.0".into(),
            },
            path: Box::new([]),
        };
        let declaration = StableDeclarationRef {
            module: module.clone(),
            path: Box::new(["X".into()]),
        };
        let owners = [
            StableCallableOwnerRef::Declaration(declaration.clone()),
            StableCallableOwnerRef::Variant(StableVariantRef {
                owner: declaration.clone(),
                selector: "A()".into(),
            }),
            StableCallableOwnerRef::Variant(StableVariantRef {
                owner: declaration,
                selector: "B()".into(),
            }),
            StableCallableOwnerRef::Conformance(StableImplRef {
                module: module.clone(),
                local_id: 1,
            }),
            StableCallableOwnerRef::Conformance(StableImplRef { module, local_id: 2 }),
        ];
        let store = TypeStore::new();
        let exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
        let fingerprints: Vec<_> = owners
            .into_iter()
            .map(|owner| {
                let parameter = StableTypeParameterRef {
                    owner: StableTypeParameterOwnerRef::Callable(StableCallableRef {
                        owner,
                        side: StableDispatchSide::Instance,
                        selector: "foo()".into(),
                    }),
                    index: 0,
                };
                let mut builder = FingerprintBuilder::new();
                exporter.write_stable_parameter_fingerprint(&mut builder, &parameter);
                builder.finish()
            })
            .collect();
        assert_eq!(fingerprints.iter().collect::<std::collections::HashSet<_>>().len(), fingerprints.len());
    }
}

/// Exporter context driving hash-consing and topological node indexing.
pub struct MetadataExporter<'a> {
    store: &'a TypeStore,
    declarations: Option<&'a DeclarationTypeTable>,
    algebraic_data: Option<&'a DataSemanticTable>,
    algebraic_enums: Option<&'a EnumSemanticTable>,
    associated_static_domains:
        Option<&'a BTreeMap<AssociatedStaticDomainDeclarationId, std::sync::Arc<crate::db::product::AssociatedStaticDomainDeclarationProduct>>>,
    aliases: Option<&'a TypeAliasTable>,
    callables: Option<&'a CallableSignatureTable>,
    fields: Option<&'a FieldSignatureTable>,
    profile: MetadataProfile,
    current_entity: MetadataEntityIdentity,
    failed_type: Option<TypeId>,
    exported_static_domains: BTreeSet<StaticDomainId>,
    identity_context: Option<StableIdentityContext<'a>>,

    // Deduplication index maps
    kind_map: HashMap<KindData, KindNodeId>,
    kind_node_map: HashMap<KindNode, KindNodeId>,
    kinds: Vec<KindNodeEntry>,

    type_map: HashMap<TypeNode, TypeNodeId>,
    types: Vec<TypeNodeEntry>,

    scoped_map: HashMap<ScopedTypeNode, ScopedTypeNodeId>,
    scoped_types: Vec<ScopedTypeNodeEntry>,

    parameters: Vec<TypeParameterRecord>,
    param_map: HashMap<TypeParameterId, StableTypeParameterRef>,

    generic_signatures: Vec<GenericSignatureRecord>,
    sig_map: HashMap<GenericSignatureKey, GenericSignatureRecordId>,
    nominal_declaration_dependencies: BTreeSet<crate::identity::DeclarationId>,
    nominal_dependency_log: Vec<crate::identity::DeclarationId>,
}

impl<'a> MetadataExporter<'a> {
    fn legacy_record_field_name(key: &FieldKey) -> Result<Box<str>, MetadataExportError> {
        match key {
            FieldKey::Symbol(name) => Ok(name.as_ref().into()),
            other => Err(MetadataExportError::NonExportableRecordKey(other.kind())),
        }
    }

    fn is_legacy_symbol_key_domain(&self, key_type: TypeId) -> bool {
        let symbol = crate::core_surface::CoreDeclarationIds::default().symbol;
        matches!(self.store.get(key_type), TypeData::Nominal { declaration } if declaration == &symbol)
    }

    fn is_scoped_legacy_symbol_key_domain(&self, key_type: ScopedTypeId) -> bool {
        match self.store.arena().get_scoped(key_type) {
            ScopedTypeData::Free(ty) => self.is_legacy_symbol_key_domain(*ty),
            _ => false,
        }
    }

    fn stable_module(&self, module: &phalcom_modules::ModuleId) -> phalcom_type_meta::identity::StableModuleRef {
        self.identity_context
            .as_ref()
            .map_or_else(|| to_stable_module(module), |context| to_stable_module_with_context(module, context))
    }

    fn stable_declaration(&self, declaration: &crate::identity::DeclarationId) -> phalcom_type_meta::identity::StableDeclarationRef {
        self.identity_context.as_ref().map_or_else(
            || to_stable_declaration(declaration),
            |context| to_stable_declaration_with_context(declaration, context),
        )
    }

    fn stable_callable(&self, callable: &crate::identity::CallableId) -> phalcom_type_meta::identity::StableCallableRef {
        self.identity_context
            .as_ref()
            .map_or_else(|| to_stable_callable(callable), |context| to_stable_callable_with_context(callable, context))
    }

    fn stable_field(&self, field: &crate::identity::FieldId) -> phalcom_type_meta::identity::StableFieldRef {
        self.identity_context
            .as_ref()
            .map_or_else(|| to_stable_field(field), |context| to_stable_field_with_context(field, context))
    }

    fn stable_impl(&self, r#impl: &crate::identity::ImplId) -> phalcom_type_meta::identity::StableImplRef {
        self.identity_context
            .as_ref()
            .map_or_else(|| to_stable_impl(r#impl), |context| to_stable_impl_with_context(r#impl, context))
    }

    fn write_stable_parameter_fingerprint(&self, builder: &mut FingerprintBuilder, parameter: &StableTypeParameterRef) {
        match &parameter.owner {
            StableTypeParameterOwnerRef::Declaration(declaration) => {
                builder.write_u8(1);
                write_stable_declaration_fingerprint(builder, declaration);
            }
            StableTypeParameterOwnerRef::Callable(callable) => {
                builder.write_u8(2);
                write_stable_callable_fingerprint(builder, callable);
            }
            StableTypeParameterOwnerRef::Impl(r#impl) => {
                builder.write_u8(3);
                write_stable_impl_fingerprint(builder, r#impl);
            }
        }
        builder.write_u32(parameter.index);
    }

    pub fn new(
        store: &'a TypeStore,
        declarations: Option<&'a DeclarationTypeTable>,
        callables: Option<&'a CallableSignatureTable>,
        fields: Option<&'a FieldSignatureTable>,
        profile: MetadataProfile,
    ) -> Self {
        Self {
            store,
            declarations,
            algebraic_data: None,
            algebraic_enums: None,
            associated_static_domains: None,
            aliases: None,
            callables,
            fields,
            profile,
            current_entity: MetadataEntityIdentity::StaticDomainCatalog,
            failed_type: None,
            exported_static_domains: BTreeSet::new(),
            identity_context: None,
            kind_map: HashMap::new(),
            kind_node_map: HashMap::new(),
            kinds: Vec::new(),
            type_map: HashMap::new(),
            types: Vec::new(),
            scoped_map: HashMap::new(),
            scoped_types: Vec::new(),
            parameters: Vec::new(),
            param_map: HashMap::new(),
            generic_signatures: Vec::new(),
            sig_map: HashMap::new(),
            nominal_declaration_dependencies: BTreeSet::new(),
            nominal_dependency_log: Vec::new(),
        }
    }

    /// Supplies source/project authority for durable identity export.
    pub fn with_project_universe(mut self, projects: &'a phalcom_modules::ProjectUniverse) -> Self {
        self.identity_context = Some(StableIdentityContext::new(projects));
        self
    }

    /// Attaches the canonical transparent-alias table for durable export.
    pub fn with_aliases(mut self, aliases: &'a TypeAliasTable) -> Self {
        self.aliases = Some(aliases);
        self
    }

    /// Creates an empty exporter with the same semantic inputs for a
    /// dependency-discovery pass. Keeping its output tables separate preserves
    /// final bundle node ordering and prevents failed probes from leaving partial
    /// nodes in the published bundle.
    pub fn dependency_discovery_fork(&self) -> Self {
        let mut fork = Self::new(self.store, self.declarations, self.callables, self.fields, self.profile);
        fork.algebraic_data = self.algebraic_data;
        fork.algebraic_enums = self.algebraic_enums;
        fork.associated_static_domains = self.associated_static_domains;
        fork.aliases = self.aliases;
        fork.identity_context = self.identity_context.as_ref().map(|context| StableIdentityContext::new(context.projects));
        fork
    }

    /// Attaches the canonical algebraic projections used to publish stable
    /// phase/shape facts alongside declaration metadata.
    pub fn with_algebraic_semantics(mut self, data: &'a DataSemanticTable, enums: &'a EnumSemanticTable) -> Self {
        self.algebraic_data = Some(data);
        self.algebraic_enums = Some(enums);
        self
    }

    /// Supplies the current declaration-owned associated static-domain
    /// products. Interned TypeStore entries are identity storage and may
    /// outlive a declaration revision, so they are never used as the catalog
    /// membership authority.
    pub fn with_associated_static_domains(
        mut self,
        domains: &'a BTreeMap<AssociatedStaticDomainDeclarationId, std::sync::Arc<crate::db::product::AssociatedStaticDomainDeclarationProduct>>,
    ) -> Self {
        self.associated_static_domains = Some(domains);
        self
    }

    fn algebraic_shape(&self, declaration: &crate::identity::DeclarationId) -> Option<AlgebraicShapeRef> {
        if self.algebraic_data.is_some_and(|table| table.data_info(declaration).is_some()) {
            Some(AlgebraicShapeRef::Product)
        } else if self.algebraic_enums.is_some_and(|table| table.enum_info(declaration).is_some()) {
            Some(AlgebraicShapeRef::Sum)
        } else {
            None
        }
    }

    fn algebraic_phase(&self, kind: KindId) -> AlgebraicPhaseRef {
        let mut current = kind;
        loop {
            match self.store.get_kind(current) {
                KindData::StaticDomain(_) => return AlgebraicPhaseRef::TypeOnly,
                KindData::Arrow { result, .. } => current = *result,
                _ => return AlgebraicPhaseRef::Runtime,
            }
        }
    }

    fn export_static_domain(&mut self, domain: StaticDomainId) -> Result<StaticDomainRef, MetadataExportError> {
        self.exported_static_domains.insert(domain);
        match self.store.static_domain(domain) {
            Some(StaticDomainData::ConstInt) => Ok(StaticDomainRef::ConstInt),
            Some(StaticDomainData::Nominal { declaration }) => Ok(StaticDomainRef::Nominal {
                declaration: self.stable_declaration(declaration),
            }),
            Some(StaticDomainData::Associated { declaration, owner_arguments }) => {
                let declaration = phalcom_type_meta::identity::StableAssociatedStaticDomainDeclarationRef {
                    owner: self.stable_declaration(&declaration.owner),
                    name: declaration.name.clone(),
                };
                let owner_environment = owner_arguments
                    .iter()
                    .map(|argument| self.export_static_argument(*argument, TypeId::DUMMY))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(StaticDomainRef::Associated {
                    declaration,
                    owner_environment: owner_environment.into_boxed_slice(),
                })
            }
            None => Err(MetadataExportError::InvalidStaticDomain(domain)),
        }
    }

    fn export_static_domain_catalog(&mut self, closure: Option<&MetadataReachabilityClosure>) -> Result<Box<[phalcom_type_meta::declaration::StaticDomainDeclarationRecord]>, MetadataExportError> {
        type CaseMap = BTreeMap<Box<str>, Box<[phalcom_type_meta::kind::KindNodeId]>>;
        type DomainMap = BTreeMap<StableStaticDomainDeclarationRef, (Vec<phalcom_type_meta::declaration::StaticDomainParameterRecord>, CaseMap)>;

        let mut sources = Vec::new();
        if let Some(closure) = closure {
            let mut pending = self.exported_static_domains.clone();
            for declaration in &closure.declarations {
                if let Some(data) = self.algebraic_data.and_then(|table| table.data_info(declaration)).and_then(|info| info.static_data.as_ref()) {
                    pending.insert(data.domain);
                }
                if let Some(data) = self.algebraic_enums.and_then(|table| table.enum_info(declaration)).and_then(|info| info.static_data.as_ref()) {
                    pending.insert(data.domain);
                }
            }
            let mut visited = BTreeSet::new();
            while let Some(domain) = pending.pop_first() {
                if !visited.insert(domain) { continue; }
                match self.store.static_domain(domain).cloned() {
                    Some(StaticDomainData::Nominal { declaration }) => {
                        let constructors = self.algebraic_data.and_then(|table| table.data_info(&declaration)).and_then(|info| info.static_data.as_ref())
                            .map(|data| data.constructors.to_vec())
                            .or_else(|| self.algebraic_enums.and_then(|table| table.enum_info(&declaration)).and_then(|info| info.static_data.as_ref()).map(|data| data.constructors.to_vec()))
                            .unwrap_or_else(|| self.store.static_constructor_ids_for_domain(domain));
                        sources.push((domain, declaration.clone(), StableStaticDomainDeclarationRef::Nominal(self.stable_declaration(&declaration)), None, constructors));
                    }
                    Some(StaticDomainData::Associated { declaration, .. }) => {
                        let constructors = self.associated_static_domains.and_then(|table| table.get(&declaration)).map(|product| product.constructors.to_vec())
                            .unwrap_or_else(|| self.store.static_constructor_ids_for_domain(domain));
                        sources.push((domain, declaration.owner.clone(), StableStaticDomainDeclarationRef::Associated(phalcom_type_meta::identity::StableAssociatedStaticDomainDeclarationRef {
                            owner: self.stable_declaration(&declaration.owner), name: declaration.name.clone(),
                        }), Some(declaration), constructors));
                    }
                    Some(StaticDomainData::ConstInt) => {}
                    None => return Err(MetadataExportError::InvalidStaticDomain(domain)),
                }
                // Constructor kinds may depend on another static domain. Discover
                // those dependencies through the canonical kind exporter.
                if let Some((_, _, _, _, constructors)) = sources.last().filter(|source| source.0 == domain).cloned() {
                    for constructor in constructors {
                        let kinds = self.store.static_constructor(constructor).ok_or(MetadataExportError::InvalidStaticConstructor(constructor))?.parameter_kinds.clone();
                        for kind in kinds.iter() { self.export_kind(*kind)?; }
                    }
                }
                pending.extend(self.exported_static_domains.iter().filter(|domain| !visited.contains(domain)).copied());
            }
        } else {
        if let Some(data) = self.algebraic_data {
            for info in data.data_decls.values() {
                if let Some(static_data) = &info.static_data {
                    sources.push((
                        static_data.domain,
                        info.owner.clone(),
                        StableStaticDomainDeclarationRef::Nominal(self.stable_declaration(&info.owner)),
                        None,
                        static_data.constructors.to_vec(),
                    ));
                }
            }
        }
        if let Some(enums) = self.algebraic_enums {
            for info in enums.enums.values() {
                if let Some(static_data) = &info.static_data {
                    sources.push((
                        static_data.domain,
                        info.owner.clone(),
                        StableStaticDomainDeclarationRef::Nominal(self.stable_declaration(&info.owner)),
                        None,
                        static_data.constructors.to_vec(),
                    ));
                }
            }
        }
        if let Some(associated) = self.associated_static_domains {
            for product in associated.values() {
                sources.push((
                    product.domain,
                    product.id.owner.clone(),
                    StableStaticDomainDeclarationRef::Associated(phalcom_type_meta::identity::StableAssociatedStaticDomainDeclarationRef {
                        owner: self.stable_declaration(&product.id.owner),
                        name: product.id.name.clone(),
                    }),
                    Some(product.id.clone()),
                    product.constructors.to_vec(),
                ));
            }
        }
        if self.algebraic_data.is_none() && self.algebraic_enums.is_none() && self.associated_static_domains.is_none() {
            // A detached TypeStore-only exporter has no workspace snapshot or
            // declaration products to consult. In that standalone mode the
            // store is the complete semantic universe. Snapshot-backed export
            // always attaches canonical products above and never enters this
            // history-enumeration path.
            for domain_id in self.store.static_domain_ids() {
                match self.store.static_domain(domain_id).cloned() {
                    Some(StaticDomainData::Nominal { declaration }) => sources.push((
                        domain_id,
                        declaration.clone(),
                        StableStaticDomainDeclarationRef::Nominal(self.stable_declaration(&declaration)),
                        None,
                        self.store.static_constructor_ids_for_domain(domain_id),
                    )),
                    Some(StaticDomainData::Associated { declaration, .. }) => sources.push((
                        domain_id,
                        declaration.owner.clone(),
                        StableStaticDomainDeclarationRef::Associated(phalcom_type_meta::identity::StableAssociatedStaticDomainDeclarationRef {
                            owner: self.stable_declaration(&declaration.owner),
                            name: declaration.name.clone(),
                        }),
                        Some(declaration),
                        self.store.static_constructor_ids_for_domain(domain_id),
                    )),
                    Some(StaticDomainData::ConstInt) | None => {}
                }
            }
        }
        }
        sources.sort_by(|left, right| left.2.cmp(&right.2));

        let mut catalog = DomainMap::new();
        for (domain_id, owner, expected_declaration, expected_associated, constructors) in sources {
            let domain_data = self.store.static_domain(domain_id).ok_or(MetadataExportError::InvalidStaticDomain(domain_id))?;
            let (declaration, owner_arguments) = match (expected_associated.as_ref(), domain_data) {
                (None, StaticDomainData::Nominal { declaration }) if *declaration == owner => (expected_declaration, None),
                (Some(expected), StaticDomainData::Associated { declaration, owner_arguments }) if declaration == expected => {
                    (expected_declaration, Some(owner_arguments))
                }
                _ => {
                    return Err(MetadataExportError::InvalidStaticDomainCatalog {
                        domain: domain_id,
                        reason: "canonical declaration product does not match its TypeStore domain",
                    });
                }
            };

            // Preserve the stable specialization graph even when a domain has
            // no observed StaticData term in the exported roots.
            self.export_static_domain(domain_id)?;

            let owner_parameters = self
                .store
                .type_parameters_for_owner(&crate::types::parameter::TypeParameterOwner::Declaration(owner.clone()))
                .into_iter()
                .enumerate()
                .map(|(expected_index, parameter)| {
                    let data = self.store.type_parameter(parameter);
                    if data.index as usize != expected_index {
                        return Err(MetadataExportError::InvalidStaticDomainCatalog {
                            domain: domain_id,
                            reason: "owner generic parameter indices are not contiguous",
                        });
                    }
                    let kind = self.export_kind(data.kind)?;
                    let mode = match data.mode {
                        GenericParameterMode::Single => GenericParameterModeRef::Single,
                        GenericParameterMode::PositionalPack => GenericParameterModeRef::PositionalPack,
                        GenericParameterMode::LabeledPack => GenericParameterModeRef::LabeledPack,
                        GenericParameterMode::CompletePack => GenericParameterModeRef::CompletePack,
                    };
                    Ok(phalcom_type_meta::declaration::StaticDomainParameterRecord { kind, mode })
                })
                .collect::<Result<Vec<_>, MetadataExportError>>()?;
            if owner_arguments.is_some_and(|arguments| arguments.len() != owner_parameters.len()) {
                return Err(MetadataExportError::InvalidStaticDomainCatalog {
                    domain: domain_id,
                    reason: "associated owner environment arity does not match its declaration",
                });
            }
            if let Some(arguments) = owner_arguments {
                let parameters = self
                    .store
                    .type_parameters_for_owner(&crate::types::parameter::TypeParameterOwner::Declaration(owner.clone()));
                for (&argument, &parameter) in arguments.iter().zip(parameters.iter()) {
                    let expected = self.store.type_parameter(parameter).kind;
                    let actual = match argument {
                        StaticArgument::Term(term) => self.store.kind_of(term),
                        StaticArgument::RecordRow(_) => KindId::RECORD_ROW,
                        StaticArgument::DynamicType => KindId::TYPE,
                    };
                    if actual != expected {
                        return Err(MetadataExportError::InvalidStaticDomainCatalog {
                            domain: domain_id,
                            reason: "associated owner environment kind does not match its declaration",
                        });
                    }
                }
            }

            let entry = catalog
                .entry(declaration.clone())
                .or_insert_with(|| (owner_parameters.clone(), BTreeMap::new()));
            if entry.0 != owner_parameters {
                return Err(MetadataExportError::InvalidStaticDomainCatalog {
                    domain: domain_id,
                    reason: "associated specializations disagree about owner parameter kinds",
                });
            }

            for constructor_id in constructors {
                let constructor = self
                    .store
                    .static_constructor(constructor_id)
                    .ok_or(MetadataExportError::InvalidStaticConstructor(constructor_id))?;
                if constructor.domain != domain_id || constructor.declaration != owner {
                    return Err(MetadataExportError::InvalidStaticDomainCatalog {
                        domain: domain_id,
                        reason: "canonical constructor does not match its static-domain declaration product",
                    });
                }
                let parameter_kinds = constructor
                    .parameter_kinds
                    .iter()
                    .map(|&kind| self.export_kind(kind))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice();
                match entry.1.get(&constructor.case_name) {
                    Some(existing) if existing.as_ref() != parameter_kinds.as_ref() => {
                        return Err(MetadataExportError::InvalidStaticDomainCatalog {
                            domain: domain_id,
                            reason: "specialized constructors disagree about payload kinds",
                        });
                    }
                    Some(_) => {}
                    None => {
                        entry.1.insert(constructor.case_name.clone(), parameter_kinds);
                    }
                }
            }
        }

        let catalog_declarations = catalog.keys().cloned().collect::<std::collections::BTreeSet<_>>();
        for kind in &self.kinds {
            let KindNode::StaticDomain(domain) = &kind.node else { continue };
            let declaration = match domain {
                StaticDomainRef::ConstInt => continue,
                StaticDomainRef::Nominal { declaration } => StableStaticDomainDeclarationRef::Nominal(declaration.clone()),
                StaticDomainRef::Associated { declaration, .. } => StableStaticDomainDeclarationRef::Associated(declaration.clone()),
            };
            if !catalog_declarations.contains(&declaration) {
                return Err(MetadataExportError::MissingStaticDomainDeclaration(declaration));
            }
        }

        Ok(catalog
            .into_iter()
            .map(
                |(declaration, (owner_parameters, cases))| phalcom_type_meta::declaration::StaticDomainDeclarationRecord {
                    declaration,
                    owner_parameters: owner_parameters.into_boxed_slice(),
                    cases: cases
                        .into_iter()
                        .map(|(name, parameter_kinds)| phalcom_type_meta::declaration::StaticDomainCaseRecord { name, parameter_kinds })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                },
            )
            .collect::<Vec<_>>()
            .into_boxed_slice())
    }

    fn export_static_argument(&mut self, argument: StaticArgument, failure_context: TypeId) -> Result<StaticArgumentRef, MetadataExportError> {
        match argument {
            StaticArgument::Term(term) => Ok(StaticArgumentRef::Term(self.export_type_form(term)?)),
            StaticArgument::RecordRow(row_id) => {
                let row = self.store.record_row(row_id);
                let mut fields = Vec::with_capacity(row.fields.len());
                for field in row.fields.iter() {
                    let presence = match field.presence {
                        crate::types::store::StructuralMemberPresence::Required => StructuralMemberPresenceRef::Required,
                        crate::types::store::StructuralMemberPresence::Optional => StructuralMemberPresenceRef::Optional,
                    };
                    fields.push(RecordFieldRef {
                        name: Self::legacy_record_field_name(&field.key)?,
                        ty: self.export_type_form(field.ty)?,
                        presence,
                    });
                }
                let (tail, remainder) = match row.tail {
                    crate::types::row::RecordRowTail::Closed => (None, None),
                    crate::types::row::RecordRowTail::Parameter(parameter) => (Some(self.export_type_parameter(parameter)?), None),
                    crate::types::row::RecordRowTail::Anonymous(rem) => {
                        let rem_ref = match rem {
                            crate::types::row::AnonymousRecordRemainder::Unconstrained { key_type } => {
                                if !self.is_legacy_symbol_key_domain(key_type) {
                                    return Err(MetadataExportError::NonExportableRecordKeyDomain);
                                }
                                AnonymousRecordRemainderRef::Unconstrained
                            }
                            crate::types::row::AnonymousRecordRemainder::Typed { key_type, value_type } => {
                                if !self.is_legacy_symbol_key_domain(key_type) {
                                    return Err(MetadataExportError::NonExportableRecordKeyDomain);
                                }
                                let id = self.export_type_form(value_type)?;
                                AnonymousRecordRemainderRef::Typed(id)
                            }
                        };
                        (None, Some(rem_ref))
                    }
                    crate::types::row::RecordRowTail::Projection(_) => return Err(MetadataExportError::NonExportableForm(failure_context)),
                };
                Ok(StaticArgumentRef::RecordRow(RecordRowRef {
                    fields: fields.into_boxed_slice(),
                    tail,
                    remainder,
                }))
            }
            StaticArgument::DynamicType => Ok(StaticArgumentRef::DynamicType),
        }
    }

    fn write_static_argument_fingerprint(&self, builder: &mut FingerprintBuilder, argument: &StaticArgumentRef) {
        match argument {
            StaticArgumentRef::Term(term) => {
                builder.write_u8(1);
                builder.write_fingerprint(self.types[term.0 as usize].structural_fingerprint);
            }
            StaticArgumentRef::RecordRow(row) => {
                builder.write_u8(2);
                builder.write_u32(row.fields.len() as u32);
                for field in row.fields.iter() {
                    builder.write_str(&field.name);
                    builder.write_u8(match field.presence {
                        StructuralMemberPresenceRef::Required => 0,
                        StructuralMemberPresenceRef::Optional => 1,
                    });
                    builder.write_fingerprint(self.types[field.ty.0 as usize].structural_fingerprint);
                }
                if let Some(tail) = &row.tail {
                    builder.write_u8(1);
                    self.write_stable_parameter_fingerprint(builder, tail);
                } else {
                    builder.write_u8(0);
                }
                if let Some(rem) = &row.remainder {
                    builder.write_u8(1);
                    match rem {
                        AnonymousRecordRemainderRef::Unconstrained => builder.write_u8(0),
                        AnonymousRecordRemainderRef::Typed(ty) => {
                            builder.write_u8(1);
                            builder.write_fingerprint(self.types[ty.0 as usize].structural_fingerprint);
                        }
                    }
                } else {
                    builder.write_u8(0);
                }
            }
            StaticArgumentRef::DynamicType => {
                builder.write_u8(3);
            }
        }
    }

    fn write_static_domain_fingerprint(&self, builder: &mut FingerprintBuilder, domain: &StaticDomainRef) {
        match domain {
            StaticDomainRef::ConstInt => builder.write_u8(1),
            StaticDomainRef::Nominal { declaration } => {
                builder.write_u8(2);
                write_stable_declaration_fingerprint(builder, declaration);
            }
            StaticDomainRef::Associated {
                declaration,
                owner_environment,
            } => {
                builder.write_u8(3);
                write_stable_declaration_fingerprint(builder, &declaration.owner);
                builder.write_str(&declaration.name);
                builder.write_u32(owner_environment.len() as u32);
                for argument in owner_environment.iter() {
                    self.write_static_argument_fingerprint(builder, argument);
                }
            }
        }
    }

    pub fn export_kind(&mut self, kind: KindId) -> Result<KindNodeId, MetadataExportError> {
        let data = self.store.get_kind(kind).clone();
        if let Some(&id) = self.kind_map.get(&data) {
            return Ok(id);
        }

        let node = match data {
            KindData::Type => KindNode::Type,
            KindData::RecordRow => KindNode::RecordRow,
            KindData::TypePack => KindNode::TypePack,
            KindData::LabeledTypePack => KindNode::LabeledTypePack,
            KindData::CompleteTypePack => KindNode::CompleteTypePack,
            KindData::Arrow { ref parameters, result } => {
                let p_nodes = parameters.iter().map(|&p| self.export_kind(p)).collect::<Result<Vec<_>, _>>()?;
                let r_node = self.export_kind(result)?;
                KindNode::Arrow {
                    parameters: p_nodes.into_boxed_slice(),
                    result: r_node,
                }
            }
            KindData::StaticDomain(domain) => KindNode::StaticDomain(self.export_static_domain(domain)?),
        };

        if let Some(&id) = self.kind_node_map.get(&node) {
            self.kind_map.insert(data, id);
            return Ok(id);
        }

        let mut fp_b = FingerprintBuilder::new();
        match &node {
            KindNode::Type => fp_b.write_u8(1),
            KindNode::RecordRow => fp_b.write_u8(3),
            KindNode::TypePack => fp_b.write_u8(4),
            KindNode::LabeledTypePack => fp_b.write_u8(5),
            KindNode::CompleteTypePack => fp_b.write_u8(6),
            KindNode::StaticDomain(domain) => {
                fp_b.write_u8(7);
                self.write_static_domain_fingerprint(&mut fp_b, domain);
            }
            KindNode::Arrow { parameters, result } => {
                fp_b.write_u8(2);
                fp_b.write_u32(parameters.len() as u32);
                for &p in parameters.iter() {
                    fp_b.write_fingerprint(self.kinds[p.0 as usize].structural_fingerprint);
                }
                fp_b.write_fingerprint(self.kinds[result.0 as usize].structural_fingerprint);
            }
        }
        let structural_fingerprint = fp_b.finish();

        let id = KindNodeId(self.kinds.len() as u32);
        self.kinds.push(KindNodeEntry { node, structural_fingerprint });
        self.kind_node_map.insert(self.kinds[id.0 as usize].node.clone(), id);
        self.kind_map.insert(data, id);
        Ok(id)
    }

    fn intern_exported_arrow_kind(&mut self, mut parameters: Vec<KindNodeId>, mut result: KindNodeId) -> KindNodeId {
        if parameters.is_empty() {
            return result;
        }
        if let KindNode::Arrow {
            parameters: nested,
            result: nested_result,
        } = self.kinds[result.0 as usize].node.clone()
        {
            parameters.extend(nested.iter().copied());
            result = nested_result;
        }
        let node = KindNode::Arrow {
            parameters: parameters.into_boxed_slice(),
            result,
        };
        if let Some(&id) = self.kind_node_map.get(&node) {
            return id;
        }
        let mut fingerprint = FingerprintBuilder::new();
        fingerprint.write_u8(2);
        if let KindNode::Arrow { parameters, result } = &node {
            fingerprint.write_u32(parameters.len() as u32);
            for &parameter in parameters.iter() {
                fingerprint.write_fingerprint(self.kinds[parameter.0 as usize].structural_fingerprint);
            }
            fingerprint.write_fingerprint(self.kinds[result.0 as usize].structural_fingerprint);
        }
        let id = KindNodeId(self.kinds.len() as u32);
        self.kinds.push(KindNodeEntry {
            node: node.clone(),
            structural_fingerprint: fingerprint.finish(),
        });
        self.kind_node_map.insert(node, id);
        id
    }

    fn apply_exported_kind(&mut self, callee: KindNodeId, arguments: &[KindNodeId]) -> Result<KindNodeId, MetadataExportError> {
        if arguments.is_empty() {
            return Ok(callee);
        }
        let node = self.kinds[callee.0 as usize].node.clone();
        let KindNode::Arrow { parameters, result } = node else {
            return Err(MetadataExportError::InvalidKindApplication("callee kind is not an arrow"));
        };
        if arguments.len() > parameters.len() {
            return Err(MetadataExportError::InvalidKindApplication("too many arguments"));
        }
        if arguments.iter().zip(parameters.iter()).any(|(actual, expected)| actual != expected) {
            return Err(MetadataExportError::InvalidKindApplication("argument kind mismatch"));
        }
        if arguments.len() == parameters.len() {
            Ok(result)
        } else {
            Ok(self.intern_exported_arrow_kind(parameters[arguments.len()..].to_vec(), result))
        }
    }

    pub fn export_type_parameter(&mut self, param: TypeParameterId) -> Result<StableTypeParameterRef, MetadataExportError> {
        if let Some(r) = self.param_map.get(&param) {
            return Ok(r.clone());
        }
        let data = self.store.type_parameter(param);
        let owner_ref = match &data.owner {
            TypeParameterOwner::Declaration(d) => StableTypeParameterOwnerRef::Declaration(self.stable_declaration(d)),
            TypeParameterOwner::Callable(c) => StableTypeParameterOwnerRef::Callable(self.stable_callable(c)),
            TypeParameterOwner::Impl(i) => StableTypeParameterOwnerRef::Impl(self.stable_impl(i)),
        };
        let param_ref = StableTypeParameterRef {
            owner: owner_ref,
            index: data.index,
        };
        let kind_id = self.export_kind(data.kind)?;
        let variance_ref = match data.variance {
            Variance::Covariant => VarianceRef::Covariant,
            Variance::Contravariant => VarianceRef::Contravariant,
            Variance::Invariant => VarianceRef::Invariant,
        };
        let source_span = data.source.as_ref().map(|s| SourceSpanRef {
            start: s.range.start as u32,
            end: s.range.end as u32,
        });
        let mode = match data.mode {
            GenericParameterMode::Single => GenericParameterModeRef::Single,
            GenericParameterMode::PositionalPack => GenericParameterModeRef::PositionalPack,
            GenericParameterMode::LabeledPack => GenericParameterModeRef::LabeledPack,
            GenericParameterMode::CompletePack => GenericParameterModeRef::CompletePack,
        };
        self.parameters.push(TypeParameterRecord {
            id: param_ref.clone(),
            name: data.name.clone(),
            kind: kind_id,
            variance: variance_ref,
            mode,
            source: source_span,
        });
        self.param_map.insert(param, param_ref.clone());
        Ok(param_ref)
    }

    pub fn export_scoped_type(&mut self, scoped_id: ScopedTypeId) -> Result<ScopedTypeNodeId, MetadataExportError> {
        self.export_scoped_type_with_binders(scoped_id, &[])
    }

    pub fn export_scoped_type_with_binders(
        &mut self,
        scoped_id: ScopedTypeId,
        binder_kinds: &[Box<[KindId]>],
    ) -> Result<ScopedTypeNodeId, MetadataExportError> {
        let scoped_data = self.store.arena().get_scoped(scoped_id).clone();
        let mut computed_kind = None;
        let scoped_node = match scoped_data {
            ScopedTypeData::Bound { depth, index } => ScopedTypeNode::Bound { depth, index },
            ScopedTypeData::Free(ty) => {
                let global_id = self.export_type_form(ty)?;
                ScopedTypeNode::Free(global_id)
            }
            ScopedTypeData::Applied { origin, ref arguments } => {
                let o = self.export_scoped_type_with_binders(origin, binder_kinds)?;
                let mut args = Vec::new();
                for &arg in arguments.iter() {
                    args.push(self.export_scoped_type_with_binders(arg, binder_kinds)?);
                }
                let callee_kind = self.scoped_types[o.0 as usize].kind;
                let argument_kinds = args.iter().map(|argument| self.scoped_types[argument.0 as usize].kind).collect::<Vec<_>>();
                computed_kind = Some(self.apply_exported_kind(callee_kind, &argument_kinds)?);
                ScopedTypeNode::Applied {
                    origin: o,
                    arguments: args.into_boxed_slice(),
                }
            }
            ScopedTypeData::Union(ref members) => {
                let mut m_nodes = Vec::new();
                for &m in members.iter() {
                    m_nodes.push(self.export_scoped_type_with_binders(m, binder_kinds)?);
                }
                ScopedTypeNode::Union(m_nodes.into_boxed_slice())
            }
            ScopedTypeData::Tuple(ref elements) => {
                let mut elems = Vec::new();
                for el in elements.iter() {
                    let ty = self.export_scoped_type_with_binders(el.ty, binder_kinds)?;
                    elems.push(ScopedTupleElementRef {
                        label: el.label.clone(),
                        ty,
                        presence: StructuralMemberPresenceRef::Required,
                    });
                }
                ScopedTypeNode::Tuple(elems.into_boxed_slice())
            }
            ScopedTypeData::Record(ref fields) => {
                let mut f_nodes = Vec::new();
                for f in fields.iter() {
                    let ty = self.export_scoped_type_with_binders(f.ty, binder_kinds)?;
                    let presence = match f.presence {
                        crate::types::store::StructuralMemberPresence::Required => StructuralMemberPresenceRef::Required,
                        crate::types::store::StructuralMemberPresence::Optional => StructuralMemberPresenceRef::Optional,
                    };
                    f_nodes.push(ScopedRecordFieldRef {
                        name: Self::legacy_record_field_name(&f.key)?,
                        ty,
                        presence,
                    });
                }
                ScopedTypeNode::Record(f_nodes.into_boxed_slice())
            }
            ScopedTypeData::OpenRecord(ref record) => {
                let fields = record
                    .fields
                    .iter()
                    .map(|field| {
                        let ty = self.export_scoped_type_with_binders(field.ty, binder_kinds)?;
                        let presence = match field.presence {
                            crate::types::store::StructuralMemberPresence::Required => StructuralMemberPresenceRef::Required,
                            crate::types::store::StructuralMemberPresence::Optional => StructuralMemberPresenceRef::Optional,
                        };
                        Ok(ScopedRecordFieldRef {
                            name: Self::legacy_record_field_name(&field.key)?,
                            ty,
                            presence,
                        })
                    })
                    .collect::<Result<Vec<_>, MetadataExportError>>()?;
                let tail = match record.tail {
                    crate::types::type_lambda::ScopedRecordTail::Bound { depth, index } => ScopedRecordTailRef::Bound { depth, index },
                    crate::types::type_lambda::ScopedRecordTail::FreeParameter(parameter) => {
                        ScopedRecordTailRef::FreeParameter(self.export_type_parameter(parameter)?)
                    }
                    crate::types::type_lambda::ScopedRecordTail::Anonymous(remainder) => {
                        let rem = match remainder {
                            crate::types::row::AnonymousRecordRemainder::Unconstrained { key_type } => {
                                if !self.is_scoped_legacy_symbol_key_domain(key_type) {
                                    return Err(MetadataExportError::NonExportableRecordKeyDomain);
                                }
                                ScopedAnonymousRecordRemainderRef::Unconstrained
                            }
                            crate::types::row::AnonymousRecordRemainder::Typed { key_type, value_type } => {
                                if !self.is_scoped_legacy_symbol_key_domain(key_type) {
                                    return Err(MetadataExportError::NonExportableRecordKeyDomain);
                                }
                                let node_id = self.export_scoped_type_with_binders(value_type, binder_kinds)?;
                                ScopedAnonymousRecordRemainderRef::Typed(node_id)
                            }
                        };
                        ScopedRecordTailRef::Anonymous(rem)
                    }
                };
                ScopedTypeNode::OpenRecord(ScopedOpenRecordTypeRef {
                    fields: fields.into_boxed_slice(),
                    tail,
                })
            }
            ScopedTypeData::RecordRow(ref row) => {
                let fields = row
                    .fields
                    .iter()
                    .map(|field| {
                        let ty = self.export_scoped_type_with_binders(field.ty, binder_kinds)?;
                        let presence = match field.presence {
                            crate::types::store::StructuralMemberPresence::Required => StructuralMemberPresenceRef::Required,
                            crate::types::store::StructuralMemberPresence::Optional => StructuralMemberPresenceRef::Optional,
                        };
                        Ok(ScopedRecordFieldRef {
                            name: Self::legacy_record_field_name(&field.key)?,
                            ty,
                            presence,
                        })
                    })
                    .collect::<Result<Vec<_>, MetadataExportError>>()?;
                let tail = match row.tail {
                    crate::types::type_lambda::ScopedRecordRowTail::Closed => ScopedRecordRowTailRef::Closed,
                    crate::types::type_lambda::ScopedRecordRowTail::Bound { depth, index } => ScopedRecordRowTailRef::Bound { depth, index },
                    crate::types::type_lambda::ScopedRecordRowTail::FreeParameter(parameter) => {
                        ScopedRecordRowTailRef::FreeParameter(self.export_type_parameter(parameter)?)
                    }
                    crate::types::type_lambda::ScopedRecordRowTail::Projection(projection) => {
                        ScopedRecordRowTailRef::Projection(self.export_type_form(projection)?)
                    }
                    crate::types::type_lambda::ScopedRecordRowTail::Anonymous(remainder) => {
                        let rem = match remainder {
                            crate::types::row::AnonymousRecordRemainder::Unconstrained { key_type } => {
                                if !self.is_scoped_legacy_symbol_key_domain(key_type) {
                                    return Err(MetadataExportError::NonExportableRecordKeyDomain);
                                }
                                ScopedAnonymousRecordRemainderRef::Unconstrained
                            }
                            crate::types::row::AnonymousRecordRemainder::Typed { key_type, value_type } => {
                                if !self.is_scoped_legacy_symbol_key_domain(key_type) {
                                    return Err(MetadataExportError::NonExportableRecordKeyDomain);
                                }
                                let node_id = self.export_scoped_type_with_binders(value_type, binder_kinds)?;
                                ScopedAnonymousRecordRemainderRef::Typed(node_id)
                            }
                        };
                        ScopedRecordRowTailRef::Anonymous(rem)
                    }
                };
                computed_kind = Some(self.export_kind(KindId::RECORD_ROW)?);
                ScopedTypeNode::RecordRow(ScopedRecordRowTypeRef {
                    fields: fields.into_boxed_slice(),
                    tail,
                })
            }
            ScopedTypeData::Callable(ref call) => {
                let mut params = Vec::new();
                for p in call.parameters.iter() {
                    let ty = self.export_scoped_type_with_binders(p.ty, binder_kinds)?;
                    params.push(ScopedCallableParamRef {
                        label: p.label.clone(),
                        ty,
                        rest: p.rest,
                    });
                }
                let return_type = self.export_scoped_type_with_binders(call.return_type, binder_kinds)?;
                ScopedTypeNode::Callable(ScopedCallableTypeRef {
                    parameters: params.into_boxed_slice(),
                    return_type,
                })
            }
            ScopedTypeData::Pack(ref pack) => {
                let mut elements = Vec::with_capacity(pack.elements.len());
                for element in pack.elements.iter() {
                    elements.push(ScopedTupleElementRef {
                        label: element.label.clone(),
                        ty: self.export_scoped_type_with_binders(element.ty, binder_kinds)?,
                        presence: StructuralMemberPresenceRef::Required,
                    });
                }
                let export_remainder = |this: &mut Self, remainder| match remainder {
                    crate::types::type_lambda::ScopedPackRemainder::None => Ok(ScopedAnonymousTupleRemainderRef::None),
                    crate::types::type_lambda::ScopedPackRemainder::Unconstrained => Ok(ScopedAnonymousTupleRemainderRef::Unconstrained),
                    crate::types::type_lambda::ScopedPackRemainder::Typed(ty) => {
                        Ok(ScopedAnonymousTupleRemainderRef::Typed(this.export_scoped_type_with_binders(ty, binder_kinds)?))
                    }
                };
                ScopedTypeNode::Pack(ScopedPackTypeRef {
                    elements: elements.into_boxed_slice(),
                    positional_remainder: export_remainder(self, pack.positional_remainder)?,
                    labeled_remainder: export_remainder(self, pack.labeled_remainder)?,
                })
            }
            ScopedTypeData::TypePack(ref elements) => {
                let mut el_nodes = Vec::new();
                for &el in elements.iter() {
                    el_nodes.push(self.export_scoped_type_with_binders(el, binder_kinds)?);
                }
                ScopedTypeNode::TypePack(el_nodes.into_boxed_slice())
            }
            ScopedTypeData::LabeledTypePack(ref elements) => {
                let mut el_nodes = Vec::new();
                for el in elements.iter() {
                    let ty = self.export_scoped_type_with_binders(el.element, binder_kinds)?;
                    el_nodes.push(phalcom_type_meta::scoped_type::ScopedLabeledTypePackElementRef {
                        label: el.label.clone(),
                        element: ty,
                    });
                }
                ScopedTypeNode::LabeledTypePack(el_nodes.into_boxed_slice())
            }
            ScopedTypeData::CompleteTypePack { positionals, labeled } => ScopedTypeNode::CompleteTypePack {
                positionals: self.export_scoped_type_with_binders(positionals, binder_kinds)?,
                labeled: self.export_scoped_type_with_binders(labeled, binder_kinds)?,
            },
            ScopedTypeData::TypePackConcat(ref segments) => {
                let mut s_nodes = Vec::new();
                for &s in segments.iter() {
                    s_nodes.push(self.export_scoped_type_with_binders(s, binder_kinds)?);
                }
                ScopedTypeNode::TypePackConcat(s_nodes.into_boxed_slice())
            }
            ScopedTypeData::LabeledTypePackConcat(ref segments) => {
                let mut s_nodes = Vec::new();
                for &s in segments.iter() {
                    s_nodes.push(self.export_scoped_type_with_binders(s, binder_kinds)?);
                }
                ScopedTypeNode::LabeledTypePackConcat(s_nodes.into_boxed_slice())
            }
            ScopedTypeData::TupleFromPacks { positionals, labeled } => ScopedTypeNode::TupleFromPacks {
                positionals: self.export_scoped_type_with_binders(positionals, binder_kinds)?,
                labeled: self.export_scoped_type_with_binders(labeled, binder_kinds)?,
            },
            ScopedTypeData::PackFromPacks { positionals, labeled } => ScopedTypeNode::PackFromPacks {
                positionals: self.export_scoped_type_with_binders(positionals, binder_kinds)?,
                labeled: self.export_scoped_type_with_binders(labeled, binder_kinds)?,
            },
            ScopedTypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => ScopedTypeNode::CallableFromPacks {
                positionals: self.export_scoped_type_with_binders(positionals, binder_kinds)?,
                labeled: self.export_scoped_type_with_binders(labeled, binder_kinds)?,
                return_type: self.export_scoped_type_with_binders(return_type, binder_kinds)?,
            },
            ScopedTypeData::PackProjection { subject, lane } => ScopedTypeNode::PackProjection {
                subject: self.export_scoped_type_with_binders(subject, binder_kinds)?,
                lane: match lane {
                    crate::types::type_lambda::ScopedPackProjectionLane::Positionals => {
                        phalcom_type_meta::scoped_type::ScopedPackProjectionLaneRef::Positionals
                    }
                    crate::types::type_lambda::ScopedPackProjectionLane::Labeled => phalcom_type_meta::scoped_type::ScopedPackProjectionLaneRef::Labeled,
                    crate::types::type_lambda::ScopedPackProjectionLane::Complete => phalcom_type_meta::scoped_type::ScopedPackProjectionLaneRef::Complete,
                },
            },
            ScopedTypeData::Lambda(lam_id) => {
                let lam_data = self.store.arena().get_lambda(lam_id).clone();
                let p_kinds = lam_data.parameter_kinds.iter().map(|&pk| self.export_kind(pk)).collect::<Result<Vec<_>, _>>()?;
                let p_modes: Vec<GenericParameterModeRef> = lam_data
                    .parameter_modes
                    .iter()
                    .map(|&mode| match mode {
                        GenericParameterMode::Single => GenericParameterModeRef::Single,
                        GenericParameterMode::PositionalPack => GenericParameterModeRef::PositionalPack,
                        GenericParameterMode::LabeledPack => GenericParameterModeRef::LabeledPack,
                        GenericParameterMode::CompletePack => GenericParameterModeRef::CompletePack,
                    })
                    .collect();
                let mut nested_binders = binder_kinds.to_vec();
                nested_binders.push(lam_data.parameter_kinds.clone());
                let body = self.export_scoped_type_with_binders(lam_data.body, &nested_binders)?;
                computed_kind = Some(self.intern_exported_arrow_kind(p_kinds.clone(), self.scoped_types[body.0 as usize].kind));
                ScopedTypeNode::Lambda {
                    parameter_kinds: p_kinds.into_boxed_slice(),
                    parameter_modes: Some(p_modes.into_boxed_slice()),
                    body,
                }
            }
        };

        if let Some(&id) = self.scoped_map.get(&scoped_node) {
            return Ok(id);
        }

        let mut fp_b = FingerprintBuilder::new();
        match &scoped_node {
            ScopedTypeNode::Bound { depth, index } => {
                fp_b.write_u8(1);
                fp_b.write_u32(*depth);
                fp_b.write_u32(*index);
            }
            ScopedTypeNode::Free(t) => {
                fp_b.write_u8(2);
                fp_b.write_fingerprint(self.types[t.0 as usize].structural_fingerprint);
            }
            ScopedTypeNode::Applied { origin, arguments } => {
                fp_b.write_u8(3);
                fp_b.write_fingerprint(self.scoped_types[origin.0 as usize].structural_fingerprint);
                fp_b.write_u32(arguments.len() as u32);
                for arg in arguments.iter() {
                    fp_b.write_fingerprint(self.scoped_types[arg.0 as usize].structural_fingerprint);
                }
            }
            ScopedTypeNode::Union(members) => {
                fp_b.write_u8(4);
                fp_b.write_u32(members.len() as u32);
                for m in members.iter() {
                    fp_b.write_fingerprint(self.scoped_types[m.0 as usize].structural_fingerprint);
                }
            }
            ScopedTypeNode::Tuple(elements) => {
                fp_b.write_u8(5);
                fp_b.write_u32(elements.len() as u32);
                for el in elements.iter() {
                    if let Some(label) = &el.label {
                        fp_b.write_u8(1);
                        fp_b.write_str(label);
                    } else {
                        fp_b.write_u8(0);
                    }
                    fp_b.write_u8(match el.presence {
                        StructuralMemberPresenceRef::Required => 0,
                        StructuralMemberPresenceRef::Optional => 1,
                    });
                    fp_b.write_fingerprint(self.scoped_types[el.ty.0 as usize].structural_fingerprint);
                }
            }
            ScopedTypeNode::Record(fields) => {
                fp_b.write_u8(6);
                fp_b.write_u32(fields.len() as u32);
                for f in fields.iter() {
                    fp_b.write_str(&f.name);
                    fp_b.write_u8(match f.presence {
                        StructuralMemberPresenceRef::Required => 0,
                        StructuralMemberPresenceRef::Optional => 1,
                    });
                    fp_b.write_fingerprint(self.scoped_types[f.ty.0 as usize].structural_fingerprint);
                }
            }
            ScopedTypeNode::Callable(call) => {
                fp_b.write_u8(7);
                fp_b.write_u32(call.parameters.len() as u32);
                for p in call.parameters.iter() {
                    fp_b.write_u8(if p.rest { 1 } else { 0 });
                    fp_b.write_fingerprint(self.scoped_types[p.ty.0 as usize].structural_fingerprint);
                }
                fp_b.write_fingerprint(self.scoped_types[call.return_type.0 as usize].structural_fingerprint);
            }
            ScopedTypeNode::Lambda {
                parameter_kinds,
                parameter_modes,
                body,
            } => {
                fp_b.write_u8(8);
                fp_b.write_u32(parameter_kinds.len() as u32);
                for pk in parameter_kinds.iter() {
                    fp_b.write_fingerprint(self.kinds[pk.0 as usize].structural_fingerprint);
                }
                if let Some(modes) = parameter_modes {
                    fp_b.write_u8(1);
                    fp_b.write_u32(modes.len() as u32);
                    for m in modes.iter() {
                        fp_b.write_u8(match m {
                            GenericParameterModeRef::Single => 0,
                            GenericParameterModeRef::PositionalPack => 1,
                            GenericParameterModeRef::LabeledPack => 2,
                            GenericParameterModeRef::CompletePack => 3,
                        });
                    }
                } else {
                    fp_b.write_u8(0);
                }
                fp_b.write_fingerprint(self.scoped_types[body.0 as usize].structural_fingerprint);
            }
            ScopedTypeNode::TypePack(elements) => {
                fp_b.write_u8(14);
                fp_b.write_u32(elements.len() as u32);
                for el in elements.iter() {
                    fp_b.write_fingerprint(self.scoped_types[el.0 as usize].structural_fingerprint);
                }
            }
            ScopedTypeNode::LabeledTypePack(elements) => {
                fp_b.write_u8(15);
                fp_b.write_u32(elements.len() as u32);
                for el in elements.iter() {
                    fp_b.write_str(&el.label);
                    fp_b.write_fingerprint(self.scoped_types[el.element.0 as usize].structural_fingerprint);
                }
            }
            ScopedTypeNode::CompleteTypePack { positionals, labeled } => {
                fp_b.write_u8(18);
                fp_b.write_fingerprint(self.scoped_types[positionals.0 as usize].structural_fingerprint);
                fp_b.write_fingerprint(self.scoped_types[labeled.0 as usize].structural_fingerprint);
            }
            ScopedTypeNode::TypePackConcat(segments) => {
                fp_b.write_u8(19);
                fp_b.write_u32(segments.len() as u32);
                for segment in segments.iter() {
                    fp_b.write_fingerprint(self.scoped_types[segment.0 as usize].structural_fingerprint);
                }
            }
            ScopedTypeNode::LabeledTypePackConcat(segments) => {
                fp_b.write_u8(20);
                fp_b.write_u32(segments.len() as u32);
                for segment in segments.iter() {
                    fp_b.write_fingerprint(self.scoped_types[segment.0 as usize].structural_fingerprint);
                }
            }
            ScopedTypeNode::TupleFromPacks { positionals, labeled } => {
                fp_b.write_u8(16);
                fp_b.write_fingerprint(self.scoped_types[positionals.0 as usize].structural_fingerprint);
                fp_b.write_fingerprint(self.scoped_types[labeled.0 as usize].structural_fingerprint);
            }
            ScopedTypeNode::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                fp_b.write_u8(21);
                fp_b.write_fingerprint(self.scoped_types[positionals.0 as usize].structural_fingerprint);
                fp_b.write_fingerprint(self.scoped_types[labeled.0 as usize].structural_fingerprint);
                fp_b.write_fingerprint(self.scoped_types[return_type.0 as usize].structural_fingerprint);
            }
            ScopedTypeNode::PackProjection { subject, lane } => {
                fp_b.write_u8(17);
                fp_b.write_fingerprint(self.scoped_types[subject.0 as usize].structural_fingerprint);
                fp_b.write_u8(match lane {
                    phalcom_type_meta::scoped_type::ScopedPackProjectionLaneRef::Positionals => 0,
                    phalcom_type_meta::scoped_type::ScopedPackProjectionLaneRef::Labeled => 1,
                    phalcom_type_meta::scoped_type::ScopedPackProjectionLaneRef::Complete => 2,
                });
            }
            ScopedTypeNode::OpenRecord(open_rec) => {
                fp_b.write_u8(9);
                fp_b.write_u32(open_rec.fields.len() as u32);
                for f in open_rec.fields.iter() {
                    fp_b.write_str(&f.name);
                    fp_b.write_u8(match f.presence {
                        StructuralMemberPresenceRef::Required => 0,
                        StructuralMemberPresenceRef::Optional => 1,
                    });
                    fp_b.write_fingerprint(self.scoped_types[f.ty.0 as usize].structural_fingerprint);
                }
                match &open_rec.tail {
                    ScopedRecordTailRef::Bound { depth, index } => {
                        fp_b.write_u8(1);
                        fp_b.write_u32(*depth);
                        fp_b.write_u32(*index);
                    }
                    ScopedRecordTailRef::FreeParameter(p) => {
                        fp_b.write_u8(2);
                        self.write_stable_parameter_fingerprint(&mut fp_b, p);
                    }
                    ScopedRecordTailRef::Anonymous(rem) => {
                        fp_b.write_u8(3);
                        match rem {
                            ScopedAnonymousRecordRemainderRef::Unconstrained => fp_b.write_u8(0),
                            ScopedAnonymousRecordRemainderRef::Typed(ty) => {
                                fp_b.write_u8(1);
                                fp_b.write_fingerprint(self.scoped_types[ty.0 as usize].structural_fingerprint);
                            }
                        }
                    }
                }
            }
            ScopedTypeNode::RecordRow(row) => {
                fp_b.write_u8(22);
                fp_b.write_u32(row.fields.len() as u32);
                for field in row.fields.iter() {
                    fp_b.write_str(&field.name);
                    fp_b.write_u8(match field.presence {
                        StructuralMemberPresenceRef::Required => 0,
                        StructuralMemberPresenceRef::Optional => 1,
                    });
                    fp_b.write_fingerprint(self.scoped_types[field.ty.0 as usize].structural_fingerprint);
                }
                match &row.tail {
                    ScopedRecordRowTailRef::Closed => fp_b.write_u8(0),
                    ScopedRecordRowTailRef::Bound { depth, index } => {
                        fp_b.write_u8(1);
                        fp_b.write_u32(*depth);
                        fp_b.write_u32(*index);
                    }
                    ScopedRecordRowTailRef::FreeParameter(parameter) => {
                        fp_b.write_u8(2);
                        self.write_stable_parameter_fingerprint(&mut fp_b, parameter);
                    }
                    ScopedRecordRowTailRef::Projection(projection) => {
                        fp_b.write_u8(3);
                        fp_b.write_fingerprint(self.types[projection.0 as usize].structural_fingerprint);
                    }
                    ScopedRecordRowTailRef::Anonymous(rem) => {
                        fp_b.write_u8(4);
                        match rem {
                            ScopedAnonymousRecordRemainderRef::Unconstrained => fp_b.write_u8(0),
                            ScopedAnonymousRecordRemainderRef::Typed(ty) => {
                                fp_b.write_u8(1);
                                fp_b.write_fingerprint(self.scoped_types[ty.0 as usize].structural_fingerprint);
                            }
                        }
                    }
                }
            }
            ScopedTypeNode::OpenTuple(open_tuple) => {
                fp_b.write_u8(24);
                fp_b.write_u32(open_tuple.elements.len() as u32);
                for el in open_tuple.elements.iter() {
                    if let Some(label) = &el.label {
                        fp_b.write_u8(1);
                        fp_b.write_str(label);
                    } else {
                        fp_b.write_u8(0);
                    }
                    fp_b.write_u8(match el.presence {
                        StructuralMemberPresenceRef::Required => 0,
                        StructuralMemberPresenceRef::Optional => 1,
                    });
                    fp_b.write_fingerprint(self.scoped_types[el.ty.0 as usize].structural_fingerprint);
                }
                match &open_tuple.positional_remainder {
                    ScopedAnonymousTupleRemainderRef::None => fp_b.write_u8(0),
                    ScopedAnonymousTupleRemainderRef::Unconstrained => fp_b.write_u8(1),
                    ScopedAnonymousTupleRemainderRef::Typed(ty) => {
                        fp_b.write_u8(2);
                        fp_b.write_fingerprint(self.scoped_types[ty.0 as usize].structural_fingerprint);
                    }
                }
                match &open_tuple.labeled_remainder {
                    ScopedAnonymousTupleRemainderRef::None => fp_b.write_u8(0),
                    ScopedAnonymousTupleRemainderRef::Unconstrained => fp_b.write_u8(1),
                    ScopedAnonymousTupleRemainderRef::Typed(ty) => {
                        fp_b.write_u8(2);
                        fp_b.write_fingerprint(self.scoped_types[ty.0 as usize].structural_fingerprint);
                    }
                }
            }
            ScopedTypeNode::AnonymousRecord(anon_rec) => {
                fp_b.write_u8(25);
                fp_b.write_u32(anon_rec.fields.len() as u32);
                for f in anon_rec.fields.iter() {
                    fp_b.write_str(&f.name);
                    fp_b.write_u8(match f.presence {
                        StructuralMemberPresenceRef::Required => 0,
                        StructuralMemberPresenceRef::Optional => 1,
                    });
                    fp_b.write_fingerprint(self.scoped_types[f.ty.0 as usize].structural_fingerprint);
                }
                match &anon_rec.remainder {
                    ScopedAnonymousRecordRemainderRef::Unconstrained => fp_b.write_u8(0),
                    ScopedAnonymousRecordRemainderRef::Typed(ty) => {
                        fp_b.write_u8(1);
                        fp_b.write_fingerprint(self.scoped_types[ty.0 as usize].structural_fingerprint);
                    }
                }
            }
            ScopedTypeNode::Pack(pack) => {
                fp_b.write_u8(26);
                fp_b.write_u32(pack.elements.len() as u32);
                for el in pack.elements.iter() {
                    if let Some(label) = &el.label {
                        fp_b.write_u8(1);
                        fp_b.write_str(label);
                    } else {
                        fp_b.write_u8(0);
                    }
                    fp_b.write_u8(match el.presence {
                        StructuralMemberPresenceRef::Required => 0,
                        StructuralMemberPresenceRef::Optional => 1,
                    });
                    fp_b.write_fingerprint(self.scoped_types[el.ty.0 as usize].structural_fingerprint);
                }
                match &pack.positional_remainder {
                    ScopedAnonymousTupleRemainderRef::None => fp_b.write_u8(0),
                    ScopedAnonymousTupleRemainderRef::Unconstrained => fp_b.write_u8(1),
                    ScopedAnonymousTupleRemainderRef::Typed(ty) => {
                        fp_b.write_u8(2);
                        fp_b.write_fingerprint(self.scoped_types[ty.0 as usize].structural_fingerprint);
                    }
                }
                match &pack.labeled_remainder {
                    ScopedAnonymousTupleRemainderRef::None => fp_b.write_u8(0),
                    ScopedAnonymousTupleRemainderRef::Unconstrained => fp_b.write_u8(1),
                    ScopedAnonymousTupleRemainderRef::Typed(ty) => {
                        fp_b.write_u8(2);
                        fp_b.write_fingerprint(self.scoped_types[ty.0 as usize].structural_fingerprint);
                    }
                }
            }
            ScopedTypeNode::PackFromPacks { positionals, labeled } => {
                fp_b.write_u8(27);
                fp_b.write_fingerprint(self.scoped_types[positionals.0 as usize].structural_fingerprint);
                fp_b.write_fingerprint(self.scoped_types[labeled.0 as usize].structural_fingerprint);
            }
        }
        let structural_fingerprint = fp_b.finish();
        let kind_id = if let Some(kind) = computed_kind {
            Ok(kind)
        } else {
            match &scoped_node {
                ScopedTypeNode::Bound { depth, index } => {
                    let inferred = binder_kinds
                        .get(binder_kinds.len().saturating_sub(1).saturating_sub(*depth as usize))
                        .and_then(|layer| layer.get(*index as usize))
                        .copied()
                        .unwrap_or(KindId::TYPE);
                    self.export_kind(inferred)
                }
                ScopedTypeNode::Free(t) => Ok(self.types[t.0 as usize].kind),
                ScopedTypeNode::TypePack(_) | ScopedTypeNode::TypePackConcat(_) => self.export_kind(KindId::TYPE_PACK),
                ScopedTypeNode::LabeledTypePack(_) | ScopedTypeNode::LabeledTypePackConcat(_) => self.export_kind(KindId::LABELED_TYPE_PACK),
                ScopedTypeNode::CompleteTypePack { .. } => self.export_kind(KindId::COMPLETE_TYPE_PACK),
                ScopedTypeNode::RecordRow(_) => self.export_kind(KindId::RECORD_ROW),
                ScopedTypeNode::PackProjection { lane, .. } => self.export_kind(match lane {
                    phalcom_type_meta::scoped_type::ScopedPackProjectionLaneRef::Positionals => KindId::TYPE_PACK,
                    phalcom_type_meta::scoped_type::ScopedPackProjectionLaneRef::Labeled => KindId::LABELED_TYPE_PACK,
                    phalcom_type_meta::scoped_type::ScopedPackProjectionLaneRef::Complete => KindId::COMPLETE_TYPE_PACK,
                }),
                _ => self.export_kind(KindId::TYPE),
            }
        }?;
        let id = ScopedTypeNodeId(self.scoped_types.len() as u32);
        self.scoped_types.push(ScopedTypeNodeEntry {
            kind: kind_id,
            form: scoped_node.clone(),
            structural_fingerprint,
        });
        self.scoped_map.insert(scoped_node, id);
        Ok(id)
    }

    pub fn export_type_form(&mut self, ty: TypeId) -> Result<TypeNodeId, MetadataExportError> {
        let result = self.export_type_form_inner(ty);
        if result.is_err() && self.failed_type.is_none() { self.failed_type = Some(ty); }
        result
    }

    /// Discovers nominal declaration dependencies through the same recursive
    /// type/kind export path used by metadata serialization.
    ///
    /// A form that cannot be exported contributes no partial dependencies here;
    /// the selected-bundle pass retries it under its retained entity context so
    /// failures retain their normal provenance.
    pub fn discover_nominal_declaration_dependencies(
        &mut self,
        roots: &BTreeSet<TypeId>,
    ) -> BTreeSet<crate::identity::DeclarationId> {
        for &ty in roots {
            let dependency_checkpoint = self.nominal_dependency_log.len();
            if self.export_type_form(ty).is_err() {
                while self.nominal_dependency_log.len() > dependency_checkpoint {
                    if let Some(declaration) = self.nominal_dependency_log.pop() {
                        self.nominal_declaration_dependencies.remove(&declaration);
                    }
                }
            }
            self.failed_type = None;
        }

        self.nominal_declaration_dependencies.clone()
    }

    fn export_type_form_inner(&mut self, ty: TypeId) -> Result<TypeNodeId, MetadataExportError> {
        metadata_perf_counter!(telos_observability::PerfCounter::MetadataTypeExportRequests, 1);
        if ty.index() >= self.store.type_count() {
            return Err(MetadataExportError::NonExportableForm(ty));
        }
        let kind_id = self.export_kind(self.store.kind_of(ty));
        let data = self.store.get(ty).clone();

        let form = match data {
            TypeData::Never => TypeNode::Never,
            TypeData::Unit => TypeNode::Unit,
            TypeData::Nominal { declaration } => {
                if self.nominal_declaration_dependencies.insert(declaration.clone()) {
                    self.nominal_dependency_log.push(declaration.clone());
                }
                TypeNode::Nominal {
                    declaration: self.stable_declaration(&declaration),
                }
            }
            TypeData::Applied { origin, ref arguments } => {
                let origin_id = self.export_type_form(origin)?;
                let mut arg_ids = Vec::new();
                for arg in arguments.iter() {
                    arg_ids.push(self.export_static_argument(*arg, origin)?);
                }
                TypeNode::Applied {
                    origin: origin_id,
                    arguments: arg_ids.into_boxed_slice(),
                }
            }
            TypeData::ExactCase { enum_type, .. } => {
                // TODO(Part 4/6): propagate exact-case to metadata when format supports it
                return self.export_type_form(enum_type);
            }
            TypeData::Union(ref members) => {
                let mut member_ids = Vec::new();
                for &m in members.iter() {
                    member_ids.push(self.export_type_form(m)?);
                }
                TypeNode::Union(member_ids.into_boxed_slice())
            }
            TypeData::Tuple(ref tuple_ty) => {
                let mut elem_refs = Vec::new();
                for el in tuple_ty.elements.iter() {
                    let ty_id = self.export_type_form(el.ty)?;
                    elem_refs.push(TupleElementRef {
                        label: el.label.clone(),
                        ty: ty_id,
                        presence: StructuralMemberPresenceRef::Required,
                    });
                }
                let pos_rem = match tuple_ty.positional_remainder {
                    AnonymousTupleRemainder::None => AnonymousTupleRemainderRef::None,
                    AnonymousTupleRemainder::Unconstrained => AnonymousTupleRemainderRef::Unconstrained,
                    AnonymousTupleRemainder::Typed(t) => AnonymousTupleRemainderRef::Typed(self.export_type_form(t)?),
                };
                let lab_rem = match tuple_ty.labeled_remainder {
                    AnonymousTupleRemainder::None => AnonymousTupleRemainderRef::None,
                    AnonymousTupleRemainder::Unconstrained => AnonymousTupleRemainderRef::Unconstrained,
                    AnonymousTupleRemainder::Typed(t) => AnonymousTupleRemainderRef::Typed(self.export_type_form(t)?),
                };
                if pos_rem == AnonymousTupleRemainderRef::None && lab_rem == AnonymousTupleRemainderRef::None {
                    TypeNode::Tuple(elem_refs.into_boxed_slice())
                } else {
                    TypeNode::OpenTuple(OpenTupleTypeRef {
                        elements: elem_refs.into_boxed_slice(),
                        positional_remainder: pos_rem,
                        labeled_remainder: lab_rem,
                    })
                }
            }
            TypeData::Record(row_id) => {
                let row = self.store.record_row(row_id).clone();
                let mut field_refs = Vec::new();
                for f in row.fields.iter() {
                    let ty_id = self.export_type_form(f.ty)?;
                    let presence = match f.presence {
                        StructuralMemberPresence::Required => StructuralMemberPresenceRef::Required,
                        StructuralMemberPresence::Optional => StructuralMemberPresenceRef::Optional,
                    };
                    field_refs.push(RecordFieldRef {
                        name: Self::legacy_record_field_name(&f.key)?,
                        ty: ty_id,
                        presence,
                    });
                }
                match row.tail {
                    crate::types::row::RecordRowTail::Closed => TypeNode::Record(field_refs.into_boxed_slice()),
                    crate::types::row::RecordRowTail::Parameter(parameter) => TypeNode::OpenRecord(OpenRecordTypeRef {
                        fields: field_refs.into_boxed_slice(),
                        tail: self.export_type_parameter(parameter)?,
                    }),
                    crate::types::row::RecordRowTail::Anonymous(remainder) => {
                        let rem_ref = match remainder {
                            crate::types::row::AnonymousRecordRemainder::Unconstrained { key_type } => {
                                if !self.is_legacy_symbol_key_domain(key_type) {
                                    return Err(MetadataExportError::NonExportableRecordKeyDomain);
                                }
                                AnonymousRecordRemainderRef::Unconstrained
                            }
                            crate::types::row::AnonymousRecordRemainder::Typed { key_type, value_type } => {
                                if !self.is_legacy_symbol_key_domain(key_type) {
                                    return Err(MetadataExportError::NonExportableRecordKeyDomain);
                                }
                                AnonymousRecordRemainderRef::Typed(self.export_type_form(value_type)?)
                            }
                        };
                        TypeNode::AnonymousRecord(AnonymousRecordTypeRef {
                            fields: field_refs.into_boxed_slice(),
                            remainder: rem_ref,
                        })
                    }
                    crate::types::row::RecordRowTail::Projection(_) => return Err(MetadataExportError::NonExportableForm(ty)),
                }
            }
            TypeData::Callable(ref call) => {
                let mut params = Vec::new();
                for p in call.parameters.iter() {
                    let ty_id = self.export_type_form(p.ty)?;
                    params.push(CallableParamRef {
                        label: p.label.clone(),
                        ty: ty_id,
                        rest: p.rest != phalcom_ast::ast::RestMode::None,
                    });
                }
                let return_type = self.export_type_form(call.return_type)?;
                TypeNode::Callable(CallableTypeRef {
                    parameters: params.into_boxed_slice(),
                    return_type,
                })
            }
            TypeData::Parameter(param) => {
                let stable_param = self.export_type_parameter(param)?;
                TypeNode::Parameter(stable_param)
            }
            TypeData::SelfType(s) => TypeNode::SelfType(SelfTypeRef {
                owner: self.stable_declaration(&s.owner),
                side: to_stable_dispatch_side(s.side),
                role: match s.role {
                    SelfRole::InstanceType => SelfRoleRef::InstanceType,
                    SelfRole::ReceiverValue => SelfRoleRef::ReceiverValue,
                },
            }),
            TypeData::AssociatedProjection(proj) => {
                let subject = self.export_type_form(proj.subject)?;
                let arguments = proj
                    .trait_ref
                    .arguments
                    .iter()
                    .map(|&arg| self.export_type_form(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                let trait_application = StableTraitApplicationRef {
                    declaration: self.stable_declaration(&proj.trait_ref.declaration),
                    arguments: arguments.into_boxed_slice(),
                };
                let requirement = StableAssociatedTypeRequirementRef {
                    owner: self.stable_declaration(&proj.requirement.owner),
                    index: proj.requirement.index,
                };
                TypeNode::AssociatedProjection {
                    subject,
                    trait_application,
                    requirement,
                }
            }
            TypeData::Lambda(lam_id) => {
                let lam_data = self.store.arena().get_lambda(lam_id).clone();
                let p_kinds = lam_data.parameter_kinds.iter().map(|&pk| self.export_kind(pk)).collect::<Result<Vec<_>, _>>()?;
                let p_modes: Vec<GenericParameterModeRef> = lam_data
                    .parameter_modes
                    .iter()
                    .map(|&mode| match mode {
                        GenericParameterMode::Single => GenericParameterModeRef::Single,
                        GenericParameterMode::PositionalPack => GenericParameterModeRef::PositionalPack,
                        GenericParameterMode::LabeledPack => GenericParameterModeRef::LabeledPack,
                        GenericParameterMode::CompletePack => GenericParameterModeRef::CompletePack,
                    })
                    .collect();
                let body = self.export_scoped_type_with_binders(lam_data.body, std::slice::from_ref(&lam_data.parameter_kinds))?;
                TypeNode::TypeLambda(TypeLambdaRef {
                    parameter_kinds: p_kinds.into_boxed_slice(),
                    parameter_modes: Some(p_modes.into_boxed_slice()),
                    body,
                })
            }
            TypeData::TypePack(elements) => {
                let exported = elements.iter().map(|&el| self.export_type_form(el)).collect::<Result<Vec<_>, _>>()?;
                TypeNode::TypePack(exported.into_boxed_slice())
            }
            TypeData::LabeledTypePack(elements) => {
                let exported = elements
                    .iter()
                    .map(|el| {
                        Ok(phalcom_type_meta::type_node::LabeledTypePackElementRef {
                            label: el.label.clone(),
                            element: self.export_type_form(el.element)?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                TypeNode::LabeledTypePack(exported.into_boxed_slice())
            }
            TypeData::CompleteTypePack { positionals, labeled } => TypeNode::CompleteTypePack {
                positionals: self.export_type_form(positionals)?,
                labeled: self.export_type_form(labeled)?,
            },
            TypeData::TypePackConcat(segments) => {
                let exported = segments.iter().map(|&segment| self.export_type_form(segment)).collect::<Result<Vec<_>, _>>()?;
                TypeNode::TypePackConcat(exported.into_boxed_slice())
            }
            TypeData::LabeledTypePackConcat(segments) => {
                let exported = segments.iter().map(|&segment| self.export_type_form(segment)).collect::<Result<Vec<_>, _>>()?;
                TypeNode::LabeledTypePackConcat(exported.into_boxed_slice())
            }
            TypeData::TupleFromPacks { positionals, labeled } => {
                let pos = self.export_type_form(positionals)?;
                let lab = self.export_type_form(labeled)?;
                TypeNode::TupleFromPacks {
                    positionals: pos,
                    labeled: lab,
                }
            }
            TypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                let pos = self.export_type_form(positionals)?;
                let lab = self.export_type_form(labeled)?;
                let ret = self.export_type_form(return_type)?;
                TypeNode::CallableFromPacks {
                    positionals: pos,
                    labeled: lab,
                    return_type: ret,
                }
            }
            TypeData::TypeMemberProjection(proj) => {
                let subject = match &proj.subject {
                    crate::types::type_member::ProjectionSubject::Type(ty) => self.export_type_form(*ty)?,
                    crate::types::type_member::ProjectionSubject::RecordRow(_) => return Err(MetadataExportError::NonExportableForm(ty)),
                };
                let selector = match &proj.selector {
                    crate::types::type_member::TypeMemberSelector::Named(name) => phalcom_type_meta::type_node::TypeMemberSelectorRef::Named(name.clone()),
                    crate::types::type_member::TypeMemberSelector::Subscript => phalcom_type_meta::type_node::TypeMemberSelectorRef::Subscript,
                };
                let arguments = proj
                    .arguments
                    .iter()
                    .map(|arg| match arg {
                        crate::types::type_member::TypeMemberArgument::Type(arg_ty) => {
                            Ok(phalcom_type_meta::type_node::TypeMemberArgumentRef::Type(self.export_type_form(*arg_ty)?))
                        }
                        crate::types::type_member::TypeMemberArgument::Const(crate::types::type_member::ConstTerm::Int(i)) => {
                            Ok(phalcom_type_meta::type_node::TypeMemberArgumentRef::ConstInt(i.clone()))
                        }
                        crate::types::type_member::TypeMemberArgument::RecordRow(_) => Err(MetadataExportError::NonExportableForm(ty)),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                TypeNode::TypeMemberProjection(phalcom_type_meta::type_node::TypeMemberProjectionRef {
                    subject,
                    selector,
                    arguments: arguments.into_boxed_slice(),
                    // Preserve the identity selected by semantic resolution;
                    // never synthesize one here. Source-authored intrinsic
                    // projections are required to arrive with this identity.
                    source_declaration: proj.source_declaration.as_ref().map(|id| phalcom_type_meta::identity::StableTypeMemberRef {
                        owner: self.stable_declaration(&id.owner),
                        selector: id.selector.name().into(),
                    }),
                })
            }
            TypeData::ConstInt(value) => TypeNode::ConstInt(value.clone()),
            TypeData::StaticData(term) => {
                let domain = self.export_static_domain(term.domain)?;
                let constructor = self
                    .store
                    .static_constructor(term.constructor)
                    .ok_or(MetadataExportError::InvalidStaticConstructor(term.constructor))?;
                let arguments = term
                    .arguments
                    .iter()
                    .map(|argument| self.export_static_argument(*argument, ty))
                    .collect::<Result<Vec<_>, MetadataExportError>>()?;
                TypeNode::StaticData(StaticDataRef {
                    domain,
                    constructor: StaticConstructorRef {
                        declaration: self.stable_declaration(&constructor.declaration),
                        case_name: constructor.case_name.clone(),
                    },
                    arguments: arguments.into_boxed_slice(),
                })
            }
            TypeData::Pack(pack) => {
                let mut elem_refs = Vec::new();
                for el in pack.elements.iter() {
                    let ty_id = self.export_type_form(el.ty)?;
                    elem_refs.push(TupleElementRef {
                        label: el.label.clone(),
                        ty: ty_id,
                        presence: StructuralMemberPresenceRef::Required,
                    });
                }
                let pos_rem = match pack.positional_remainder {
                    AnonymousTupleRemainder::None => AnonymousTupleRemainderRef::None,
                    AnonymousTupleRemainder::Unconstrained => AnonymousTupleRemainderRef::Unconstrained,
                    AnonymousTupleRemainder::Typed(t) => AnonymousTupleRemainderRef::Typed(self.export_type_form(t)?),
                };
                let lab_rem = match pack.labeled_remainder {
                    AnonymousTupleRemainder::None => AnonymousTupleRemainderRef::None,
                    AnonymousTupleRemainder::Unconstrained => AnonymousTupleRemainderRef::Unconstrained,
                    AnonymousTupleRemainder::Typed(t) => AnonymousTupleRemainderRef::Typed(self.export_type_form(t)?),
                };
                TypeNode::Pack(PackTypeRef {
                    elements: elem_refs.into_boxed_slice(),
                    positional_remainder: pos_rem,
                    labeled_remainder: lab_rem,
                })
            }
            TypeData::PackFromPacks { positionals, labeled } => {
                let pos = self.export_type_form(positionals)?;
                let lab = self.export_type_form(labeled)?;
                TypeNode::PackFromPacks {
                    positionals: pos,
                    labeled: lab,
                }
            }
            TypeData::ClassObject { .. } | TypeData::Family(_) | TypeData::StaticDomainForm { .. } => return Err(MetadataExportError::NonExportableForm(ty)),
        };

        if let Some(&id) = self.type_map.get(&form) {
            metadata_perf_counter!(telos_observability::PerfCounter::MetadataTypeExportReuse, 1);
            return Ok(id);
        }

        let mut fp_b = FingerprintBuilder::new();
        match &form {
            TypeNode::Never => fp_b.write_u8(1),
            TypeNode::Unit => fp_b.write_u8(2),
            TypeNode::ConstInt(value) => {
                fp_b.write_u8(22);
                fp_b.write_str(&value.to_string());
            }
            TypeNode::StaticData(static_data) => {
                fp_b.write_u8(23);
                self.write_static_domain_fingerprint(&mut fp_b, &static_data.domain);
                write_stable_declaration_fingerprint(&mut fp_b, &static_data.constructor.declaration);
                fp_b.write_str(&static_data.constructor.case_name);
                fp_b.write_u32(static_data.arguments.len() as u32);
                for argument in static_data.arguments.iter() {
                    self.write_static_argument_fingerprint(&mut fp_b, argument);
                }
            }
            TypeNode::Nominal { declaration } => {
                fp_b.write_u8(3);
                for c in declaration.path.iter() {
                    fp_b.write_str(c);
                }
            }
            TypeNode::Applied { origin, arguments } => {
                fp_b.write_u8(4);
                fp_b.write_fingerprint(self.types[origin.0 as usize].structural_fingerprint);
                fp_b.write_u32(arguments.len() as u32);
                for arg in arguments.iter() {
                    self.write_static_argument_fingerprint(&mut fp_b, arg);
                }
            }
            TypeNode::Union(members) => {
                fp_b.write_u8(5);
                fp_b.write_u32(members.len() as u32);
                for m in members.iter() {
                    fp_b.write_fingerprint(self.types[m.0 as usize].structural_fingerprint);
                }
            }
            TypeNode::Tuple(elements) => {
                fp_b.write_u8(6);
                fp_b.write_u32(elements.len() as u32);
                for el in elements.iter() {
                    if let Some(label) = &el.label {
                        fp_b.write_u8(1);
                        fp_b.write_str(label);
                    } else {
                        fp_b.write_u8(0);
                    }
                    fp_b.write_u8(match el.presence {
                        StructuralMemberPresenceRef::Required => 0,
                        StructuralMemberPresenceRef::Optional => 1,
                    });
                    fp_b.write_fingerprint(self.types[el.ty.0 as usize].structural_fingerprint);
                }
            }
            TypeNode::Record(fields) => {
                fp_b.write_u8(7);
                fp_b.write_u32(fields.len() as u32);
                for f in fields.iter() {
                    fp_b.write_str(&f.name);
                    fp_b.write_u8(match f.presence {
                        StructuralMemberPresenceRef::Required => 0,
                        StructuralMemberPresenceRef::Optional => 1,
                    });
                    fp_b.write_fingerprint(self.types[f.ty.0 as usize].structural_fingerprint);
                }
            }
            TypeNode::OpenRecord(open_rec) => {
                fp_b.write_u8(12);
                fp_b.write_u32(open_rec.fields.len() as u32);
                for f in open_rec.fields.iter() {
                    fp_b.write_str(&f.name);
                    fp_b.write_u8(match f.presence {
                        StructuralMemberPresenceRef::Required => 0,
                        StructuralMemberPresenceRef::Optional => 1,
                    });
                    fp_b.write_fingerprint(self.types[f.ty.0 as usize].structural_fingerprint);
                }
                self.write_stable_parameter_fingerprint(&mut fp_b, &open_rec.tail);
            }
            TypeNode::OpenTuple(open_tuple) => {
                fp_b.write_u8(24);
                fp_b.write_u32(open_tuple.elements.len() as u32);
                for el in open_tuple.elements.iter() {
                    if let Some(label) = &el.label {
                        fp_b.write_u8(1);
                        fp_b.write_str(label);
                    } else {
                        fp_b.write_u8(0);
                    }
                    fp_b.write_u8(match el.presence {
                        StructuralMemberPresenceRef::Required => 0,
                        StructuralMemberPresenceRef::Optional => 1,
                    });
                    fp_b.write_fingerprint(self.types[el.ty.0 as usize].structural_fingerprint);
                }
                match &open_tuple.positional_remainder {
                    AnonymousTupleRemainderRef::None => fp_b.write_u8(0),
                    AnonymousTupleRemainderRef::Unconstrained => fp_b.write_u8(1),
                    AnonymousTupleRemainderRef::Typed(ty) => {
                        fp_b.write_u8(2);
                        fp_b.write_fingerprint(self.types[ty.0 as usize].structural_fingerprint);
                    }
                }
                match &open_tuple.labeled_remainder {
                    AnonymousTupleRemainderRef::None => fp_b.write_u8(0),
                    AnonymousTupleRemainderRef::Unconstrained => fp_b.write_u8(1),
                    AnonymousTupleRemainderRef::Typed(ty) => {
                        fp_b.write_u8(2);
                        fp_b.write_fingerprint(self.types[ty.0 as usize].structural_fingerprint);
                    }
                }
            }
            TypeNode::AnonymousRecord(anon_rec) => {
                fp_b.write_u8(25);
                fp_b.write_u32(anon_rec.fields.len() as u32);
                for f in anon_rec.fields.iter() {
                    fp_b.write_str(&f.name);
                    fp_b.write_u8(match f.presence {
                        StructuralMemberPresenceRef::Required => 0,
                        StructuralMemberPresenceRef::Optional => 1,
                    });
                    fp_b.write_fingerprint(self.types[f.ty.0 as usize].structural_fingerprint);
                }
                match &anon_rec.remainder {
                    AnonymousRecordRemainderRef::Unconstrained => fp_b.write_u8(0),
                    AnonymousRecordRemainderRef::Typed(ty) => {
                        fp_b.write_u8(1);
                        fp_b.write_fingerprint(self.types[ty.0 as usize].structural_fingerprint);
                    }
                }
            }
            TypeNode::Callable(call) => {
                fp_b.write_u8(8);
                fp_b.write_u32(call.parameters.len() as u32);
                for p in call.parameters.iter() {
                    fp_b.write_u8(if p.rest { 1 } else { 0 });
                    fp_b.write_fingerprint(self.types[p.ty.0 as usize].structural_fingerprint);
                }
                fp_b.write_fingerprint(self.types[call.return_type.0 as usize].structural_fingerprint);
            }
            TypeNode::Parameter(param) => {
                fp_b.write_u8(9);
                fp_b.write_u32(param.index);
            }
            TypeNode::SelfType(s) => {
                fp_b.write_u8(10);
                for c in s.owner.path.iter() {
                    fp_b.write_str(c);
                }
            }
            TypeNode::TypeLambda(lambda) => {
                fp_b.write_u8(11);
                fp_b.write_u32(lambda.parameter_kinds.len() as u32);
                for pk in lambda.parameter_kinds.iter() {
                    fp_b.write_fingerprint(self.kinds[pk.0 as usize].structural_fingerprint);
                }
                if let Some(modes) = &lambda.parameter_modes {
                    fp_b.write_u8(1);
                    fp_b.write_u32(modes.len() as u32);
                    for m in modes.iter() {
                        fp_b.write_u8(match m {
                            GenericParameterModeRef::Single => 0,
                            GenericParameterModeRef::PositionalPack => 1,
                            GenericParameterModeRef::LabeledPack => 2,
                            GenericParameterModeRef::CompletePack => 3,
                        });
                    }
                } else {
                    fp_b.write_u8(0);
                }
                fp_b.write_fingerprint(self.scoped_types[lambda.body.0 as usize].structural_fingerprint);
            }
            TypeNode::AssociatedProjection {
                subject,
                trait_application,
                requirement,
            } => {
                fp_b.write_u8(13);
                fp_b.write_fingerprint(self.types[subject.0 as usize].structural_fingerprint);
                for c in trait_application.declaration.path.iter() {
                    fp_b.write_str(c);
                }
                fp_b.write_u32(trait_application.arguments.len() as u32);
                for arg in trait_application.arguments.iter() {
                    fp_b.write_fingerprint(self.types[arg.0 as usize].structural_fingerprint);
                }
                for c in requirement.owner.path.iter() {
                    fp_b.write_str(c);
                }
                fp_b.write_u32(requirement.index);
            }
            TypeNode::TypePack(elements) => {
                fp_b.write_u8(14);
                fp_b.write_u32(elements.len() as u32);
                for el in elements.iter() {
                    fp_b.write_fingerprint(self.types[el.0 as usize].structural_fingerprint);
                }
            }
            TypeNode::LabeledTypePack(elements) => {
                fp_b.write_u8(15);
                fp_b.write_u32(elements.len() as u32);
                for el in elements.iter() {
                    fp_b.write_str(&el.label);
                    fp_b.write_fingerprint(self.types[el.element.0 as usize].structural_fingerprint);
                }
            }
            TypeNode::CompleteTypePack { positionals, labeled } => {
                fp_b.write_u8(18);
                fp_b.write_fingerprint(self.types[positionals.0 as usize].structural_fingerprint);
                fp_b.write_fingerprint(self.types[labeled.0 as usize].structural_fingerprint);
            }
            TypeNode::TypePackConcat(segments) => {
                fp_b.write_u8(19);
                fp_b.write_u32(segments.len() as u32);
                for segment in segments.iter() {
                    fp_b.write_fingerprint(self.types[segment.0 as usize].structural_fingerprint);
                }
            }
            TypeNode::LabeledTypePackConcat(segments) => {
                fp_b.write_u8(20);
                fp_b.write_u32(segments.len() as u32);
                for segment in segments.iter() {
                    fp_b.write_fingerprint(self.types[segment.0 as usize].structural_fingerprint);
                }
            }
            TypeNode::TupleFromPacks { positionals, labeled } => {
                fp_b.write_u8(16);
                fp_b.write_fingerprint(self.types[positionals.0 as usize].structural_fingerprint);
                fp_b.write_fingerprint(self.types[labeled.0 as usize].structural_fingerprint);
            }
            TypeNode::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                fp_b.write_u8(21);
                fp_b.write_fingerprint(self.types[positionals.0 as usize].structural_fingerprint);
                fp_b.write_fingerprint(self.types[labeled.0 as usize].structural_fingerprint);
                fp_b.write_fingerprint(self.types[return_type.0 as usize].structural_fingerprint);
            }
            TypeNode::TypeMemberProjection(proj) => {
                fp_b.write_u8(17);
                fp_b.write_fingerprint(self.types[proj.subject.0 as usize].structural_fingerprint);
                if let Some(source) = &proj.source_declaration {
                    fp_b.write_u8(1);
                    write_stable_declaration_fingerprint(&mut fp_b, &source.owner);
                    fp_b.write_str(&source.selector);
                } else {
                    fp_b.write_u8(0);
                }
                match &proj.selector {
                    phalcom_type_meta::type_node::TypeMemberSelectorRef::Named(name) => {
                        fp_b.write_u8(0);
                        fp_b.write_str(name);
                    }
                    phalcom_type_meta::type_node::TypeMemberSelectorRef::Subscript => {
                        fp_b.write_u8(1);
                    }
                }
                fp_b.write_u32(proj.arguments.len() as u32);
                for arg in proj.arguments.iter() {
                    match arg {
                        phalcom_type_meta::type_node::TypeMemberArgumentRef::Type(ty) => {
                            fp_b.write_u8(0);
                            fp_b.write_fingerprint(self.types[ty.0 as usize].structural_fingerprint);
                        }
                        phalcom_type_meta::type_node::TypeMemberArgumentRef::ConstInt(i) => {
                            fp_b.write_u8(1);
                            fp_b.write_str(&i.to_string());
                        }
                    }
                }
            }
            TypeNode::Pack(pack) => {
                fp_b.write_u8(26);
                fp_b.write_u32(pack.elements.len() as u32);
                for el in pack.elements.iter() {
                    if let Some(label) = &el.label {
                        fp_b.write_u8(1);
                        fp_b.write_str(label);
                    } else {
                        fp_b.write_u8(0);
                    }
                    fp_b.write_u8(match el.presence {
                        StructuralMemberPresenceRef::Required => 0,
                        StructuralMemberPresenceRef::Optional => 1,
                    });
                    fp_b.write_fingerprint(self.types[el.ty.0 as usize].structural_fingerprint);
                }
                match &pack.positional_remainder {
                    AnonymousTupleRemainderRef::None => fp_b.write_u8(0),
                    AnonymousTupleRemainderRef::Unconstrained => fp_b.write_u8(1),
                    AnonymousTupleRemainderRef::Typed(ty) => {
                        fp_b.write_u8(2);
                        fp_b.write_fingerprint(self.types[ty.0 as usize].structural_fingerprint);
                    }
                }
                match &pack.labeled_remainder {
                    AnonymousTupleRemainderRef::None => fp_b.write_u8(0),
                    AnonymousTupleRemainderRef::Unconstrained => fp_b.write_u8(1),
                    AnonymousTupleRemainderRef::Typed(ty) => {
                        fp_b.write_u8(2);
                        fp_b.write_fingerprint(self.types[ty.0 as usize].structural_fingerprint);
                    }
                }
            }
            TypeNode::PackFromPacks { positionals, labeled } => {
                fp_b.write_u8(27);
                fp_b.write_fingerprint(self.types[positionals.0 as usize].structural_fingerprint);
                fp_b.write_fingerprint(self.types[labeled.0 as usize].structural_fingerprint);
            }
        }
        let structural_fingerprint = fp_b.finish();

        let id = TypeNodeId(self.types.len() as u32);
        self.types.push(TypeNodeEntry {
            kind: kind_id?,
            form: form.clone(),
            structural_fingerprint,
        });
        metadata_perf_counter!(telos_observability::PerfCounter::MetadataUniqueTypeNodes, 1);
        self.type_map.insert(form, id);
        Ok(id)
    }

    pub fn export_type_term(&mut self, term: &TypeTerm) -> Result<TypeNodeId, MetadataExportError> {
        match term {
            TypeTerm::Canonical(ty) => self.export_type_form(*ty),
            TypeTerm::SelfType(s) => {
                let self_node = TypeNode::SelfType(SelfTypeRef {
                    owner: self.stable_declaration(&s.owner),
                    side: to_stable_dispatch_side(s.side),
                    role: match s.role {
                        SelfRole::InstanceType => SelfRoleRef::InstanceType,
                        SelfRole::ReceiverValue => SelfRoleRef::ReceiverValue,
                    },
                });
                if let Some(&id) = self.type_map.get(&self_node) {
                    metadata_perf_counter!(telos_observability::PerfCounter::MetadataTypeExportReuse, 1);
                    return Ok(id);
                }
                let kind_id = self.export_kind(self.store.kind_of(self.store.unit()))?;
                let mut fp_b = FingerprintBuilder::new();
                fp_b.write_u8(10);
                let stable_owner = self.stable_declaration(&s.owner);
                for c in stable_owner.module.path.iter() {
                    fp_b.write_str(c);
                }
                for c in stable_owner.path.iter() {
                    fp_b.write_str(c);
                }
                fp_b.write_u8(match s.side {
                    crate::identity::DispatchSide::Instance => 1,
                    crate::identity::DispatchSide::Class => 2,
                });
                fp_b.write_u8(match s.role {
                    SelfRole::InstanceType => 1,
                    SelfRole::ReceiverValue => 2,
                });
                let structural_fingerprint = fp_b.finish();
                let id = TypeNodeId(self.types.len() as u32);
                self.types.push(TypeNodeEntry {
                    kind: kind_id,
                    form: self_node.clone(),
                    structural_fingerprint,
                });
                metadata_perf_counter!(telos_observability::PerfCounter::MetadataUniqueTypeNodes, 1);
                self.type_map.insert(self_node, id);
                Ok(id)
            }
            TypeTerm::Infer(_) => Err(MetadataExportError::InferenceVariable),
        }
    }

    /// Durable metadata accepts canonical forms only. Query-local rigid terms
    /// stay behind this explicit guard and must be widened before publication.
    pub fn validate_local_type_export(&self, local_type: &LocalType) -> Result<(), MetadataExportError> {
        if local_type.free_rigids().is_empty() {
            Ok(())
        } else {
            Err(MetadataExportError::ScopedLocalType)
        }
    }

    pub fn export_generic_signature(&mut self, sig: &GenericSignature) -> Result<GenericSignatureRecordId, MetadataExportError> {
        metadata_perf_counter!(telos_observability::PerfCounter::MetadataGenericSignatureExportRequests, 1);
        sig.validate_publishable(self.store).map_err(MetadataExportError::InvalidGenericSignature)?;
        let key = (sig.owner.clone(), sig.parameters.clone(), sig.constraints.clone());
        if let Some(&id) = self.sig_map.get(&key) {
            metadata_perf_counter!(telos_observability::PerfCounter::MetadataGenericSignatureExportReuse, 1);
            return Ok(id);
        }

        let owner_ref = match &sig.owner {
            TypeParameterOwner::Declaration(d) => StableTypeParameterOwnerRef::Declaration(self.stable_declaration(d)),
            TypeParameterOwner::Callable(c) => StableTypeParameterOwnerRef::Callable(self.stable_callable(c)),
            TypeParameterOwner::Impl(i) => StableTypeParameterOwnerRef::Impl(self.stable_impl(i)),
        };

        let mut param_refs = Vec::new();
        for &param in sig.parameters.iter() {
            param_refs.push(self.export_type_parameter(param)?);
        }

        let mut constraint_refs = Vec::new();
        for c in sig.constraints.iter() {
            match c {
                GenericConstraint::Subtype { lower, upper } => {
                    let l = self.export_type_term(lower)?;
                    let u = self.export_type_term(upper)?;
                    constraint_refs.push(GenericConstraintRef::Subtype { lower: l, upper: u });
                }
                GenericConstraint::Equivalent { left, right } => {
                    let l = self.export_type_term(left)?;
                    let r = self.export_type_term(right)?;
                    constraint_refs.push(GenericConstraintRef::Equivalent { left: l, right: r });
                }
                GenericConstraint::Conforms { subject, trait_ref } => {
                    let subject = self.export_type_term(subject)?;
                    let arguments = trait_ref
                        .arguments
                        .iter()
                        .map(|&argument| self.export_type_form(argument))
                        .collect::<Result<Vec<_>, _>>()?;
                    constraint_refs.push(GenericConstraintRef::Conforms {
                        subject,
                        trait_application: StableTraitApplicationRef {
                            declaration: self.stable_declaration(&trait_ref.declaration),
                            arguments: arguments.into_boxed_slice(),
                        },
                    });
                }
            }
        }

        let id = GenericSignatureRecordId(self.generic_signatures.len() as u32);
        self.generic_signatures.push(GenericSignatureRecord {
            owner: owner_ref,
            parameters: param_refs.into_boxed_slice(),
            constraints: constraint_refs.into_boxed_slice(),
        });
        metadata_perf_counter!(telos_observability::PerfCounter::MetadataUniqueGenericSignatures, 1);
        self.sig_map.insert(key, id);
        Ok(id)
    }

    fn export_declared_type_fact(&mut self, fact: &DeclaredTypeFact) -> Result<PublishedTypeSlot, MetadataExportError> {
        Ok(match &fact.state {
            DeclaredTypeState::Known(term) => match self.export_type_term(term) {
                Ok(form) => {
                    let authority = match fact.basis {
                        DeclaredTypeBasis::SourceAnnotation => PublishedTypeAuthority::DeclaredAnnotation,
                        DeclaredTypeBasis::NativeSignature => PublishedTypeAuthority::TrustedNative,
                        DeclaredTypeBasis::DeclarationSemantics | DeclaredTypeBasis::ConstructorSemantics => PublishedTypeAuthority::GeneratedDeclaration,
                        DeclaredTypeBasis::InitializerInference
                        | DeclaredTypeBasis::BodyInference
                        | DeclaredTypeBasis::ContextualTyping
                        | DeclaredTypeBasis::PatternDecomposition
                        | DeclaredTypeBasis::Unspecified => PublishedTypeAuthority::CompilerInferred,
                    };
                    PublishedTypeSlot::Known { form, authority }
                }
                Err(error) => return Err(error),
            },
            DeclaredTypeState::Dynamic(reason) => PublishedTypeSlot::Dynamic {
                reason: match reason {
                    crate::types::evidence::DynamicReason::ExplicitEscape => DynamicReasonRef::ExplicitEscape,
                    crate::types::evidence::DynamicReason::DynamicRestPack | crate::types::evidence::DynamicReason::RuntimeReflection => {
                        DynamicReasonRef::UncheckedBoundary
                    }
                },
            },
            DeclaredTypeState::Unknown(reason) => PublishedTypeSlot::Unknown {
                reason: match reason {
                    crate::types::evidence::UnknownReason::UnannotatedDeclaration
                    | crate::types::evidence::UnknownReason::NoTypeEvidence
                    | crate::types::evidence::UnknownReason::MissingInitializer => UnknownReasonRef::UnannotatedDeclaration,
                    crate::types::evidence::UnknownReason::OpaqueNative => UnknownReasonRef::OpaqueNative,
                    _ => UnknownReasonRef::InferenceFailed,
                },
            },
        })
    }

    /// Explicit complete-table helper for detached metadata fixtures. Production
    /// compilation must supply its retained closure instead.
    pub fn build_complete_test_bundle(mut self, runtime_roots: &[(&phalcom_modules::ModuleId, &str, TypeId)]) -> Result<SemanticMetadataBundle, MetadataExportError> {
        let closure = MetadataReachabilityClosure {
            declarations: self.declarations.map(|table| table.iter().map(|(id, _)| id.clone()).collect()).unwrap_or_default(),
            aliases: self.aliases.map(|table| table.iter().map(|(id, _)| id.clone()).collect()).unwrap_or_default(),
            callables: self.callables.map(|table| table.iter().map(|(id, _)| id.clone()).collect()).unwrap_or_default(),
            fields: self.fields.map(|table| table.iter().map(|(id, _)| id.clone()).collect()).unwrap_or_default(),
            runtime_types: runtime_roots.iter().map(|(_, _, ty)| *ty).collect(),
        };
        self.build_bundle_inner(&closure, true, runtime_roots)
    }

    /// Projects only explicitly retained semantic entities and recursive type roots.
    pub fn build_bundle_from_closure(self, closure: &MetadataReachabilityClosure, runtime_roots: &[(&phalcom_modules::ModuleId, &str, TypeId)]) -> Result<SemanticMetadataBundle, MetadataExportError> {
        self.build_selected_bundle(closure, runtime_roots, false)
    }

    fn build_selected_bundle(mut self, closure: &MetadataReachabilityClosure, runtime_roots: &[(&phalcom_modules::ModuleId, &str, TypeId)], complete_catalog: bool) -> Result<SemanticMetadataBundle, MetadataExportError> {
        self.build_bundle_inner(closure, complete_catalog, runtime_roots).map_err(|source| MetadataExportError::Projection {
            profile: self.profile, entity: Box::new(self.current_entity.clone()), type_id: self.failed_type, source: Box::new(source),
        })
    }

    fn build_bundle_inner(&mut self, closure: &MetadataReachabilityClosure, complete_catalog: bool, runtime_roots: &[(&phalcom_modules::ModuleId, &str, TypeId)]) -> Result<SemanticMetadataBundle, MetadataExportError> {
        #[cfg(feature = "perf-observability")]
        let metadata_total_span = telos_observability::PerfContext::current_thread().span(telos_observability::PerfPhase::MetadataExport);
        let mut out_declarations = Vec::new();
        let mut out_aliases = Vec::new();
        let mut out_callables = Vec::new();
        let mut out_fields = Vec::new();
        let mut out_roots = Vec::new();

        // Durable references to resolved projects must never fall back to
        // graph-node display IDs or a made-up fingerprint.
        let require_project = |project: &phalcom_modules::ProjectIdentity| -> Result<(), MetadataExportError> {
            let phalcom_modules::ProjectIdentity::Resolved(id) = project else {
                return Ok(());
            };
            let Some(context) = self.identity_context.as_ref() else {
                return Err(MetadataExportError::MissingProjectIdentityContext);
            };
            if context.projects.get_project(*id).is_none() {
                return Err(MetadataExportError::MissingResolvedProject(*id));
            }
            Ok(())
        };
        #[cfg(feature = "perf-observability")]
        let identity_validation_span = telos_observability::PerfContext::current_thread()
            .span(telos_observability::PerfPhase::MetadataExportProjectIdentityValidation);
        if self.declarations.is_some() {
            for declaration in closure.declarations.iter() {
                require_project(&declaration.module.project)?;
            }
        }
        if self.aliases.is_some() {
            for declaration in closure.aliases.iter() {
                require_project(&declaration.module.project)?;
            }
        }
        if self.callables.is_some() {
            for callable in closure.callables.iter() {
                require_project(&callable.owner.module().project)?;
            }
        }
        if self.fields.is_some() {
            for field in closure.fields.iter() {
                require_project(&field.owner.module.project)?;
            }
        }
        for (module, _, _) in runtime_roots {
            require_project(&module.project)?;
        }
        #[cfg(feature = "perf-observability")]
        identity_validation_span.finish(telos_observability::PerfOutcome::Success);

        #[cfg(feature = "perf-observability")]
        let declarations_span =
            telos_observability::PerfContext::current_thread().span(telos_observability::PerfPhase::MetadataExportDeclarations);
        let mut callable_owner_comparisons = 0u64;
        if let Some(decls) = self.declarations {
            for decl_id in &closure.declarations {
                let Some(info) = decls.get(decl_id) else { continue };
                self.current_entity = MetadataEntityIdentity::Declaration(decl_id.clone());
                let form_id = self.export_type_form(info.form)?;
                let kind_id = self.export_kind(info.kind)?;
                let sig_id = if let Some(ref sig) = info.generic_signature {
                    Some(self.export_generic_signature(sig)?)
                } else {
                    None
                };
                let sup_template = if let Some(ref tmpl) = info.supertype_template {
                    Some(self.export_type_form(tmpl.supertype)?)
                } else {
                    None
                };
                let mut instance_callables = Vec::new();
                let mut class_callables = Vec::new();
                if let Some(callables) = self.callables {
                    for (callable, _) in callables.iter_declaration_owner(decl_id).filter(|(id, _)| closure.callables.contains(*id)) {
                        callable_owner_comparisons = callable_owner_comparisons.saturating_add(1);
                        if !matches!(&callable.owner, CallableOwnerId::Declaration(owner) if owner == decl_id) {
                            continue;
                        }
                        match callable.side {
                            DispatchSide::Instance => instance_callables.push(self.stable_callable(callable)),
                            DispatchSide::Class => class_callables.push(self.stable_callable(callable)),
                        }
                    }
                }

                let mut instance_fields = Vec::new();
                let mut class_fields = Vec::new();
                if let Some(fields) = self.fields {
                    for (field, _) in fields.iter_owner(decl_id).filter(|(id, _)| closure.fields.contains(*id)) {
                        match field.side {
                            DispatchSide::Instance => instance_fields.push(self.stable_field(field)),
                            DispatchSide::Class => class_fields.push(self.stable_field(field)),
                        }
                    }
                }
                out_declarations.push(DeclarationTypeRecord {
                    declaration: self.stable_declaration(decl_id),
                    form: form_id,
                    kind: kind_id,
                    generic_signature: sig_id,
                    superclass_template: sup_template,
                    instance_callables: instance_callables.into_boxed_slice(),
                    class_callables: class_callables.into_boxed_slice(),
                    instance_fields: instance_fields.into_boxed_slice(),
                    class_fields: class_fields.into_boxed_slice(),
                    flags: DeclarationTypeFlags::default(),
                    algebraic: self.algebraic_shape(decl_id).map(|shape| AlgebraicDeclarationRecord {
                        phase: self.algebraic_phase(info.kind),
                        shape,
                    }),
                    source: None,
                });
            }
        }
        metadata_perf_counter!(telos_observability::PerfCounter::MetadataDeclarationsExported, out_declarations.len());
        metadata_perf_counter!(telos_observability::PerfCounter::MetadataCallableOwnerComparisons, callable_owner_comparisons);
        let _ = callable_owner_comparisons;
        #[cfg(feature = "perf-observability")]
        declarations_span.finish(telos_observability::PerfOutcome::Success);

        #[cfg(feature = "perf-observability")]
        let aliases_span = telos_observability::PerfContext::current_thread().span(telos_observability::PerfPhase::MetadataExportAliases);
        if let Some(aliases) = self.aliases {
            for decl_id in &closure.aliases {
                let Some(info) = aliases.get(decl_id) else { continue };
                self.current_entity = MetadataEntityIdentity::Alias(decl_id.clone());
                let target = self.export_type_form(info.form)?;
                let generic_signature = if let Some(ref sig) = info.generic_signature {
                    Some(self.export_generic_signature(sig)?)
                } else {
                    None
                };
                out_aliases.push(phalcom_type_meta::declaration::TypeAliasRecord {
                    declaration: self.stable_declaration(decl_id),
                    generic_signature,
                    target,
                    source: Some(SourceSpanRef {
                        start: info.source.range.start as u32,
                        end: info.source.range.end as u32,
                    }),
                });
            }
        }
        metadata_perf_counter!(telos_observability::PerfCounter::MetadataAliasesExported, out_aliases.len());
        #[cfg(feature = "perf-observability")]
        aliases_span.finish(telos_observability::PerfOutcome::Success);

        #[cfg(feature = "perf-observability")]
        let callables_span =
            telos_observability::PerfContext::current_thread().span(telos_observability::PerfPhase::MetadataExportCallables);
        if let Some(call_table) = self.callables {
            for call_id in &closure.callables {
                let Some(sig) = call_table.get(call_id) else { continue };
                self.current_entity = MetadataEntityIdentity::Callable(call_id.clone());
                let sig_id = if let Some(ref g) = sig.generics {
                    Some(self.export_generic_signature(g)?)
                } else {
                    None
                };
                let mut params = Vec::new();
                for p in sig.parameters.iter() {
                    let ty_slot = self.export_declared_type_fact(&p.declared_type)?;
                    params.push(CallableParameterRecord {
                        index: p.index(),
                        local_name: p.local_name.clone(),
                        external_label: p.external_label.clone(),
                        rest: match p.rest {
                            phalcom_ast::ast::RestMode::None => RestModeRef::None,
                            phalcom_ast::ast::RestMode::Positional => RestModeRef::Anonymous,
                            phalcom_ast::ast::RestMode::Labeled | phalcom_ast::ast::RestMode::Complete => RestModeRef::Named,
                        },
                        ty: ty_slot,
                        source: None,
                    });
                }
                let return_slot = if let Some(inferred) = sig.inferred_return.as_ref().filter(|knowledge| knowledge.is_known()) {
                    match inferred.ty().and_then(|ty| self.export_type_term(&TypeTerm::Canonical(ty)).ok()) {
                        Some(form) => PublishedTypeSlot::Known {
                            form,
                            authority: PublishedTypeAuthority::CompilerInferred,
                        },
                        None => self.export_declared_type_fact(&sig.declared_return)?,
                    }
                } else {
                    self.export_declared_type_fact(&sig.declared_return)?
                };
                out_callables.push(CallableSemanticRecord {
                    callable: self.stable_callable(call_id),
                    generic_signature: sig_id,
                    parameters: params.into_boxed_slice(),
                    return_type: return_slot,
                    source: None,
                });
            }
        }
        metadata_perf_counter!(telos_observability::PerfCounter::MetadataCallablesExported, out_callables.len());
        #[cfg(feature = "perf-observability")]
        callables_span.finish(telos_observability::PerfOutcome::Success);

        #[cfg(feature = "perf-observability")]
        let fields_span = telos_observability::PerfContext::current_thread().span(telos_observability::PerfPhase::MetadataExportFields);
        if let Some(field_table) = self.fields {
            for field_id in &closure.fields {
                let Some(sig) = field_table.get(field_id) else { continue };
                self.current_entity = MetadataEntityIdentity::Field(field_id.clone());
                let ty_slot = self.export_declared_type_fact(&sig.declared_type)?;
                out_fields.push(FieldSemanticRecord {
                    field: self.stable_field(field_id),
                    mutability: match sig.mutable {
                        false => FieldMutabilityRef::Immutable,
                        true => FieldMutabilityRef::Mutable,
                    },
                    ty: ty_slot,
                    source: None,
                });
            }
        }
        metadata_perf_counter!(telos_observability::PerfCounter::MetadataFieldsExported, out_fields.len());
        #[cfg(feature = "perf-observability")]
        fields_span.finish(telos_observability::PerfOutcome::Success);

        #[cfg(feature = "perf-observability")]
        let runtime_roots_span =
            telos_observability::PerfContext::current_thread().span(telos_observability::PerfPhase::MetadataExportRuntimeRoots);
        for &(mod_id, key, ty) in runtime_roots {
            self.current_entity = MetadataEntityIdentity::RuntimeType { module: mod_id.clone(), key: key.into() };
            let form_id = self.export_type_form(ty)?;
            out_roots.push(RuntimeTypeFormRoot {
                module: self.stable_module(mod_id),
                local_key: RuntimeTypeFormKey(key.into()),
                form: form_id,
            });
        }
        metadata_perf_counter!(telos_observability::PerfCounter::MetadataRuntimeRootsExported, out_roots.len());
        #[cfg(feature = "perf-observability")]
        runtime_roots_span.finish(telos_observability::PerfOutcome::Success);

        #[cfg(feature = "perf-observability")]
        let static_domains_span =
            telos_observability::PerfContext::current_thread().span(telos_observability::PerfPhase::MetadataExportStaticDomains);
        self.current_entity = MetadataEntityIdentity::StaticDomainCatalog;
        let static_domain_catalog = self.export_static_domain_catalog(if complete_catalog { None } else { Some(closure) })?;
        #[cfg(feature = "perf-observability")]
        static_domains_span.finish(telos_observability::PerfOutcome::Success);

        #[cfg(feature = "perf-observability")]
        let fingerprints_span =
            telos_observability::PerfContext::current_thread().span(telos_observability::PerfPhase::MetadataExportFingerprints);
        self.parameters.sort_by(|left, right| left.id.cmp(&right.id));
        let mut algebraic_declarations = out_declarations.iter().filter(|record| record.algebraic.is_some()).collect::<Vec<_>>();
        algebraic_declarations.sort_by(|left, right| left.declaration.cmp(&right.declaration));

        let revision_fingerprint = self.identity_context.as_ref().map_or_else(
            || {
                let mut builder = FingerprintBuilder::new();
                builder.write_u64(out_declarations.len() as u64);
                builder.write_u64(out_aliases.len() as u64);
                builder.write_u64(out_callables.len() as u64);
                builder.write_u64(out_fields.len() as u64);
                builder.finish()
            },
            |context| {
                let mut builder = FingerprintBuilder::new();
                let mut projects = context
                    .projects
                    .projects()
                    .iter()
                    .map(|project| (project.source_identity.0.to_string_lossy().into_owned(), project.revision_fingerprint()))
                    .collect::<Vec<_>>();
                projects.sort_by(|left, right| left.0.cmp(&right.0));
                for (source, revision) in projects {
                    builder.write_str(&source);
                    builder.write_fingerprint(Fingerprint128(revision.0));
                }
                builder.finish()
            },
        );
        let mut interface_builder = FingerprintBuilder::new();
        interface_builder.write_fingerprint(revision_fingerprint);
        interface_builder.write_u32(self.parameters.len() as u32);
        for parameter in &self.parameters {
            self.write_stable_parameter_fingerprint(&mut interface_builder, &parameter.id);
            interface_builder.write_str(&parameter.name);
            interface_builder.write_fingerprint(self.kinds[parameter.kind.0 as usize].structural_fingerprint);
            interface_builder.write_u8(match parameter.variance {
                VarianceRef::Covariant => 1,
                VarianceRef::Contravariant => 2,
                VarianceRef::Invariant => 3,
            });
            interface_builder.write_u8(match parameter.mode {
                GenericParameterModeRef::Single => 1,
                GenericParameterModeRef::PositionalPack => 2,
                GenericParameterModeRef::LabeledPack => 3,
                GenericParameterModeRef::CompletePack => 4,
            });
        }
        interface_builder.write_u32(algebraic_declarations.len() as u32);
        for declaration in algebraic_declarations {
            write_stable_declaration_fingerprint(&mut interface_builder, &declaration.declaration);
            let Some(algebraic) = declaration.algebraic else { continue };
            interface_builder.write_u8(match algebraic.phase {
                AlgebraicPhaseRef::Runtime => 1,
                AlgebraicPhaseRef::TypeOnly => 2,
            });
            interface_builder.write_u8(match algebraic.shape {
                AlgebraicShapeRef::Product => 1,
                AlgebraicShapeRef::Sum => 2,
            });
        }
        interface_builder.write_u32(static_domain_catalog.len() as u32);
        for domain in static_domain_catalog.iter() {
            match &domain.declaration {
                StableStaticDomainDeclarationRef::Nominal(declaration) => {
                    interface_builder.write_u8(1);
                    write_stable_declaration_fingerprint(&mut interface_builder, declaration);
                }
                StableStaticDomainDeclarationRef::Associated(declaration) => {
                    interface_builder.write_u8(2);
                    write_stable_declaration_fingerprint(&mut interface_builder, &declaration.owner);
                    interface_builder.write_str(&declaration.name);
                }
            }
            interface_builder.write_u32(domain.owner_parameters.len() as u32);
            for parameter in domain.owner_parameters.iter() {
                interface_builder.write_fingerprint(self.kinds[parameter.kind.0 as usize].structural_fingerprint);
                interface_builder.write_u8(match parameter.mode {
                    GenericParameterModeRef::Single => 1,
                    GenericParameterModeRef::PositionalPack => 2,
                    GenericParameterModeRef::LabeledPack => 3,
                    GenericParameterModeRef::CompletePack => 4,
                });
            }
            interface_builder.write_u32(domain.cases.len() as u32);
            for case in domain.cases.iter() {
                interface_builder.write_str(&case.name);
                interface_builder.write_u32(case.parameter_kinds.len() as u32);
                for kind in case.parameter_kinds.iter() {
                    interface_builder.write_fingerprint(self.kinds[kind.0 as usize].structural_fingerprint);
                }
            }
        }
        let interface_fingerprint = interface_builder.finish();
        #[cfg(feature = "perf-observability")]
        fingerprints_span.finish(telos_observability::PerfOutcome::Success);
        let header = SemanticMetadataHeader {
            schema_version: TYPE_METADATA_SCHEMA_VERSION,
            semantic_model_version: SEMANTIC_MODEL_VERSION,
            producer: ProducerIdentity("phalcom-semantic".into()),
            producer_version: "0.1.0".into(),
            native_surface_schema_version: NATIVE_SURFACE_SCHEMA_VERSION,
            profile: self.profile,
            features: MetadataFeatures {
                type_lambdas: true,
                record_rows: true,
                static_domains: !static_domain_catalog.is_empty() || self.kinds.iter().any(|entry| matches!(entry.node, KindNode::StaticDomain(_))),
                runtime_type_constants: !out_roots.is_empty(),
                source_occurrences: false,
                advanced_sections: Box::new([]),
            },
            identity_scheme: ArtifactIdentityScheme::V1Standard,
            source_fingerprint: revision_fingerprint,
            interface_fingerprint,
        };

        let bundle = SemanticMetadataBundle {
            header,
            kinds: std::mem::take(&mut self.kinds).into_boxed_slice(),
            types: std::mem::take(&mut self.types).into_boxed_slice(),
            scoped_types: std::mem::take(&mut self.scoped_types).into_boxed_slice(),
            parameters: std::mem::take(&mut self.parameters).into_boxed_slice(),
            generic_signatures: std::mem::take(&mut self.generic_signatures).into_boxed_slice(),
            declarations: out_declarations.into_boxed_slice(),
            static_domains: static_domain_catalog,
            aliases: out_aliases.into_boxed_slice(),
            callables: out_callables.into_boxed_slice(),
            fields: out_fields.into_boxed_slice(),
            module_roots: Box::new([]),
            runtime_roots: out_roots.into_boxed_slice(),
            occurrences: Box::new([]),
            extensions: Box::new([]),
        };
        #[cfg(feature = "perf-observability")]
        metadata_total_span.finish(telos_observability::PerfOutcome::Success);
        Ok(bundle)
    }
}
