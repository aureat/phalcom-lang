//! Program-level compiler seam for closed linked module plans.

use super::artifact::ModuleMaterializationPlan;
use crate::typing::{MetadataPoolId, RuntimeCallEnvironmentRecipe, RuntimeTypeRecipe, RuntimeTypeRef};
use phalcom_common::range::SourceRange;
use phalcom_modules::{
    EntryOwnership, ExecutableLinkedProgram, FilesystemSourceProvider, ImportSurface, InterfaceBuilder, InterfaceError, LinkError, LinkedModule,
    LinkedReadSpec, ModuleComponent, ModuleId, ModuleKind, ModuleLinker, ModulePath, ModuleResolutionError, ModuleResolver, ProjectError, ProjectUniverse,
    RuntimeDependencyReason, RuntimeDependencyRequirement, RuntimeRequirementProducer, RuntimeRequirementProduct, RuntimeRequirementProvenance,
    RuntimeRequirementTarget, RuntimeRequirementTiming, SourceError, SourceId, SourceLocation, SourceProvider, SymbolId, UniverseSourceProvider,
    classify_entry_ownership,
};
use phalcom_semantic::SemanticDiagnostic;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

type RuntimeTraitEnvironmentRootSpec = (
    ModuleId,
    SourceRange,
    phalcom_semantic::identity::ImplId,
    phalcom_semantic::traits::TraitRequirementId,
    phalcom_semantic::types::id::TypeParameterId,
    phalcom_semantic::types::id::TypeId,
    String,
);

/// A covering impl body uses impl-owned parameters while its public call
/// specialization uses declaration-owned parameters. Keep both coordinates in
/// the callee's executable environment without changing the semantic product.
fn close_covering_body_bindings(
    snapshot: &phalcom_semantic::SemanticSnapshot,
    callee: &phalcom_semantic::identity::CallableId,
    bindings: impl IntoIterator<Item = (phalcom_semantic::types::id::TypeParameterId, phalcom_semantic::types::id::TypeId)>,
) -> Result<BTreeMap<phalcom_semantic::types::id::TypeParameterId, phalcom_semantic::types::id::TypeId>, ProgramCompileError> {
    let mut closed = BTreeMap::new();
    for (parameter, ty) in bindings {
        if closed.insert(parameter, ty).is_some_and(|previous| previous != ty) {
            return Err(ProgramCompileError::GenericEvidenceLayout(format!(
                "conflicting runtime call binding for {parameter:?} at {callee:?}"
            )));
        }
    }
    let Some(covering) = snapshot
        .callable_definitions
        .get(callee)
        .and_then(|definition| definition.inherent_body_context.as_ref())
        .and_then(|context| context.covering.as_ref())
    else {
        return Ok(closed);
    };
    for (&impl_parameter, &declaration_parameter) in &covering.impl_to_decl {
        match (closed.get(&impl_parameter), closed.get(&declaration_parameter)) {
            (Some(impl_type), Some(declaration_type)) if impl_type != declaration_type => {
                return Err(ProgramCompileError::GenericEvidenceLayout(format!(
                    "conflicting covering-impl call bindings for {impl_parameter:?} and {declaration_parameter:?} at {callee:?}"
                )));
            }
            (Some(&ty), None) => {
                closed.insert(declaration_parameter, ty);
            }
            (None, Some(&ty)) => {
                closed.insert(impl_parameter, ty);
            }
            _ => {}
        }
    }
    Ok(closed)
}

/// Collects canonical owners referenced by a runtime metadata type root.
/// These are graph requirements rather than linked reads: metadata preserves
/// declaration identity, while runtime values are projected by their own
/// semantic requirements.
struct RuntimeMetadataTypeOwnerCollector<'a> {
    store: &'a phalcom_semantic::types::store::TypeStore,
    static_domain_constructors: &'a BTreeMap<phalcom_semantic::types::id::StaticDomainId, Box<[phalcom_semantic::types::id::AlgebraicStaticConstructorId]>>,
    associated_domain_constructors:
        &'a BTreeMap<phalcom_semantic::identity::AssociatedStaticDomainDeclarationId, Box<[phalcom_semantic::types::id::AlgebraicStaticConstructorId]>>,
    owners: BTreeSet<ModuleId>,
    seen_types: BTreeSet<phalcom_semantic::types::id::TypeId>,
    seen_rows: BTreeSet<phalcom_semantic::types::id::RecordRowId>,
    seen_lambdas: BTreeSet<phalcom_semantic::types::id::TypeLambdaId>,
    seen_scoped: BTreeSet<phalcom_semantic::types::id::ScopedTypeId>,
    seen_kinds: BTreeSet<phalcom_semantic::types::id::KindId>,
    seen_static_domains: BTreeSet<phalcom_semantic::types::id::StaticDomainId>,
    seen_static_constructors: BTreeSet<phalcom_semantic::types::id::AlgebraicStaticConstructorId>,
}

impl<'a> RuntimeMetadataTypeOwnerCollector<'a> {
    fn new(
        store: &'a phalcom_semantic::types::store::TypeStore,
        static_domain_constructors: &'a BTreeMap<phalcom_semantic::types::id::StaticDomainId, Box<[phalcom_semantic::types::id::AlgebraicStaticConstructorId]>>,
        associated_domain_constructors: &'a BTreeMap<
            phalcom_semantic::identity::AssociatedStaticDomainDeclarationId,
            Box<[phalcom_semantic::types::id::AlgebraicStaticConstructorId]>,
        >,
    ) -> Self {
        Self {
            store,
            static_domain_constructors,
            associated_domain_constructors,
            owners: BTreeSet::new(),
            seen_types: BTreeSet::new(),
            seen_rows: BTreeSet::new(),
            seen_lambdas: BTreeSet::new(),
            seen_scoped: BTreeSet::new(),
            seen_kinds: BTreeSet::new(),
            seen_static_domains: BTreeSet::new(),
            seen_static_constructors: BTreeSet::new(),
        }
    }

    fn add_declaration(&mut self, declaration: &phalcom_modules::DeclarationId) {
        self.owners.insert(declaration.module.clone());
    }

    fn add_parameter(&mut self, parameter: phalcom_semantic::types::id::TypeParameterId) {
        use phalcom_semantic::types::parameter::TypeParameterOwner;

        match &self.store.type_parameter(parameter).owner {
            TypeParameterOwner::Declaration(declaration) => self.add_declaration(declaration),
            TypeParameterOwner::Callable(callable) => {
                self.owners.insert(callable.module().clone());
            }
            TypeParameterOwner::Impl(implementation) => {
                self.owners.insert(implementation.module.clone());
            }
        }
    }

    fn add_static_domain(&mut self, domain: phalcom_semantic::types::id::StaticDomainId) {
        use phalcom_semantic::types::{parameter::TypeParameterOwner, store::StaticDomainData};

        if !self.seen_static_domains.insert(domain) {
            return;
        }
        let (owner, constructors) = match self.store.static_domain(domain).cloned() {
            Some(StaticDomainData::Nominal { declaration }) => {
                self.add_declaration(&declaration);
                (Some(declaration), self.static_domain_constructors.get(&domain).cloned())
            }
            Some(StaticDomainData::Associated { declaration, owner_arguments }) => {
                self.add_declaration(&declaration.owner);
                for argument in owner_arguments.iter().copied() {
                    self.add_static_argument(argument);
                }
                (Some(declaration.owner.clone()), self.associated_domain_constructors.get(&declaration).cloned())
            }
            Some(StaticDomainData::ConstInt) | None => (None, None),
        };

        if let Some(owner) = owner {
            let parameters = self.store.type_parameters_for_owner(&TypeParameterOwner::Declaration(owner));
            let kinds = parameters
                .into_iter()
                .map(|parameter| self.store.type_parameter(parameter).kind)
                .collect::<Vec<_>>();
            for kind in kinds {
                self.add_kind(kind);
            }
        }

        if let Some(constructors) = constructors {
            for constructor in constructors.iter().copied() {
                self.add_static_constructor(constructor);
            }
        }
    }

    fn add_static_constructor(&mut self, constructor: phalcom_semantic::types::id::AlgebraicStaticConstructorId) {
        if !self.seen_static_constructors.insert(constructor) {
            return;
        }
        let Some(constructor) = self.store.static_constructor(constructor) else {
            return;
        };
        let declaration = constructor.declaration.clone();
        let parameter_kinds = constructor.parameter_kinds.to_vec();
        self.add_declaration(&declaration);
        for kind in parameter_kinds {
            self.add_kind(kind);
        }
    }

    fn add_kind(&mut self, kind: phalcom_semantic::types::id::KindId) {
        use phalcom_semantic::types::kind::KindData;

        if !self.seen_kinds.insert(kind) {
            return;
        }
        let arrow = match self.store.get_kind(kind) {
            KindData::Arrow { parameters, result } => Some((parameters.to_vec(), *result)),
            KindData::StaticDomain(domain) => {
                self.add_static_domain(*domain);
                None
            }
            KindData::Type | KindData::RecordRow | KindData::TypePack | KindData::LabeledTypePack | KindData::CompleteTypePack => None,
        };
        if let Some((parameters, result)) = arrow {
            for parameter in parameters {
                self.add_kind(parameter);
            }
            self.add_kind(result);
        }
    }

    fn add_static_argument(&mut self, argument: phalcom_semantic::types::store::StaticArgument) {
        use phalcom_semantic::types::store::StaticArgument;

        match argument {
            StaticArgument::Term(ty) => self.add_type(ty),
            StaticArgument::RecordRow(row) => self.add_row(row),
            StaticArgument::DynamicType => {}
        }
    }

    fn add_tuple_remainder(&mut self, remainder: phalcom_semantic::types::store::AnonymousTupleRemainder) {
        if let phalcom_semantic::types::store::AnonymousTupleRemainder::Typed(ty) = remainder {
            self.add_type(ty);
        }
    }

    fn add_record_remainder(&mut self, remainder: phalcom_semantic::types::row::AnonymousRecordRemainder) {
        self.add_type(remainder.key_type());
        if let Some(value_type) = remainder.value_type() {
            self.add_type(value_type);
        }
    }

    fn add_row(&mut self, row_id: phalcom_semantic::types::id::RecordRowId) {
        use phalcom_semantic::types::row::RecordRowTail;

        if !self.seen_rows.insert(row_id) {
            return;
        }
        let row = self.store.record_row(row_id);
        for field in row.fields.iter() {
            self.add_type(field.ty);
        }
        match row.tail {
            RecordRowTail::Closed => {}
            RecordRowTail::Anonymous(remainder) => self.add_record_remainder(remainder),
            RecordRowTail::Parameter(parameter) => self.add_parameter(parameter),
            RecordRowTail::Projection(ty) => self.add_type(ty),
        }
    }

    fn add_type(&mut self, ty: phalcom_semantic::types::id::TypeId) {
        use phalcom_semantic::types::store::TypeData;
        use phalcom_semantic::types::type_member::{ProjectionSubject, TypeMemberArgument};

        if !self.seen_types.insert(ty) {
            return;
        }
        match self.store.get(ty) {
            TypeData::Nominal { declaration } | TypeData::ClassObject { declaration } => self.add_declaration(declaration),
            TypeData::StaticDomainForm { domain } => self.add_static_domain(*domain),
            TypeData::Applied { origin, arguments } => {
                self.add_type(*origin);
                for argument in arguments.iter().copied() {
                    self.add_static_argument(argument);
                }
            }
            TypeData::ExactCase { variant, enum_type } => {
                self.add_declaration(&self.store.variant_identity(*variant).owner);
                self.add_type(*enum_type);
            }
            TypeData::Union(members) | TypeData::TypePack(members) | TypeData::TypePackConcat(members) | TypeData::LabeledTypePackConcat(members) => {
                for member in members.iter().copied() {
                    self.add_type(member);
                }
            }
            TypeData::Tuple(tuple) => {
                for element in tuple.elements.iter() {
                    self.add_type(element.ty);
                }
                self.add_tuple_remainder(tuple.positional_remainder);
                self.add_tuple_remainder(tuple.labeled_remainder);
            }
            TypeData::Record(row) => self.add_row(*row),
            TypeData::LabeledTypePack(elements) => {
                for element in elements.iter() {
                    self.add_type(element.element);
                }
            }
            TypeData::CompleteTypePack { positionals, labeled }
            | TypeData::TupleFromPacks { positionals, labeled }
            | TypeData::PackFromPacks { positionals, labeled } => {
                self.add_type(*positionals);
                self.add_type(*labeled);
            }
            TypeData::Pack(pack) => {
                for element in pack.elements.iter() {
                    self.add_type(element.ty);
                }
                self.add_tuple_remainder(pack.positional_remainder);
                self.add_tuple_remainder(pack.labeled_remainder);
            }
            TypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                self.add_type(*positionals);
                self.add_type(*labeled);
                self.add_type(*return_type);
            }
            TypeData::Callable(callable) => {
                for parameter in callable.parameters.iter() {
                    self.add_type(parameter.ty);
                }
                self.add_type(callable.return_type);
            }
            TypeData::Family(family) => {
                for member in self.store.get_family(*family).members.iter() {
                    self.add_type(member.ty);
                }
            }
            TypeData::Parameter(parameter) => self.add_parameter(*parameter),
            TypeData::Lambda(lambda) => self.add_lambda(*lambda),
            TypeData::SelfType(term) => self.add_declaration(&term.owner),
            TypeData::AssociatedProjection(projection) => {
                self.add_declaration(&projection.trait_ref.declaration);
                self.add_declaration(&projection.requirement.owner);
                self.add_type(projection.subject);
                for argument in projection.trait_ref.arguments.iter().copied() {
                    self.add_type(argument);
                }
            }
            TypeData::TypeMemberProjection(projection) => {
                if let Some(source) = &projection.source_declaration {
                    self.add_declaration(&source.owner);
                }
                match &projection.subject {
                    ProjectionSubject::Type(subject) => self.add_type(*subject),
                    ProjectionSubject::RecordRow(row) => self.add_row(*row),
                }
                for argument in projection.arguments.iter() {
                    match argument {
                        TypeMemberArgument::Type(argument) => self.add_type(*argument),
                        TypeMemberArgument::RecordRow(row) => self.add_row(*row),
                        TypeMemberArgument::Const(_) => {}
                    }
                }
            }
            TypeData::StaticData(term) => {
                self.add_static_domain(term.domain);
                if let Some(constructor) = self.store.static_constructor(term.constructor) {
                    self.add_declaration(&constructor.declaration);
                }
                for argument in term.arguments.iter().copied() {
                    self.add_static_argument(argument);
                }
            }
            TypeData::Never | TypeData::Unit | TypeData::ConstInt(_) => {}
        }
    }

    fn add_lambda(&mut self, lambda: phalcom_semantic::types::id::TypeLambdaId) {
        if self.seen_lambdas.insert(lambda) {
            let (parameter_kinds, result_kind, body) = {
                let lambda = self.store.arena().get_lambda(lambda);
                (lambda.parameter_kinds.to_vec(), lambda.result_kind, lambda.body)
            };
            for kind in parameter_kinds {
                self.add_kind(kind);
            }
            self.add_kind(result_kind);
            self.add_scoped_type(body);
        }
    }

    fn add_scoped_type(&mut self, scoped: phalcom_semantic::types::id::ScopedTypeId) {
        use phalcom_semantic::types::type_lambda::{ScopedRecordRowTail, ScopedRecordTail, ScopedTypeData};

        if !self.seen_scoped.insert(scoped) {
            return;
        }
        match self.store.arena().get_scoped(scoped) {
            ScopedTypeData::Bound { .. } => {}
            ScopedTypeData::Free(ty) => self.add_type(*ty),
            ScopedTypeData::Applied { origin, arguments } => {
                self.add_scoped_type(*origin);
                for argument in arguments.iter().copied() {
                    self.add_scoped_type(argument);
                }
            }
            ScopedTypeData::Union(members)
            | ScopedTypeData::TypePack(members)
            | ScopedTypeData::TypePackConcat(members)
            | ScopedTypeData::LabeledTypePackConcat(members) => {
                for member in members.iter().copied() {
                    self.add_scoped_type(member);
                }
            }
            ScopedTypeData::Tuple(elements) => {
                for element in elements.iter() {
                    self.add_scoped_type(element.ty);
                }
            }
            ScopedTypeData::Record(fields) => {
                for field in fields.iter() {
                    self.add_scoped_type(field.ty);
                }
            }
            ScopedTypeData::OpenRecord(record) => {
                for field in record.fields.iter() {
                    self.add_scoped_type(field.ty);
                }
                match record.tail {
                    ScopedRecordTail::Bound { .. } => {}
                    ScopedRecordTail::FreeParameter(parameter) => self.add_parameter(parameter),
                    ScopedRecordTail::Anonymous(remainder) => {
                        self.add_scoped_type(remainder.key_type());
                        if let Some(value_type) = remainder.value_type() {
                            self.add_scoped_type(value_type);
                        }
                    }
                }
            }
            ScopedTypeData::RecordRow(row) => {
                for field in row.fields.iter() {
                    self.add_scoped_type(field.ty);
                }
                match row.tail {
                    ScopedRecordRowTail::Closed | ScopedRecordRowTail::Bound { .. } => {}
                    ScopedRecordRowTail::FreeParameter(parameter) => self.add_parameter(parameter),
                    ScopedRecordRowTail::Anonymous(remainder) => {
                        self.add_scoped_type(remainder.key_type());
                        if let Some(value_type) = remainder.value_type() {
                            self.add_scoped_type(value_type);
                        }
                    }
                    ScopedRecordRowTail::Projection(ty) => self.add_type(ty),
                }
            }
            ScopedTypeData::Callable(callable) => {
                for parameter in callable.parameters.iter() {
                    self.add_scoped_type(parameter.ty);
                }
                self.add_scoped_type(callable.return_type);
            }
            ScopedTypeData::Pack(pack) => {
                for element in pack.elements.iter() {
                    self.add_scoped_type(element.ty);
                }
                if let phalcom_semantic::types::type_lambda::ScopedPackRemainder::Typed(ty) = pack.positional_remainder {
                    self.add_scoped_type(ty);
                }
                if let phalcom_semantic::types::type_lambda::ScopedPackRemainder::Typed(ty) = pack.labeled_remainder {
                    self.add_scoped_type(ty);
                }
            }
            ScopedTypeData::LabeledTypePack(elements) => {
                for element in elements.iter() {
                    self.add_scoped_type(element.element);
                }
            }
            ScopedTypeData::CompleteTypePack { positionals, labeled }
            | ScopedTypeData::TupleFromPacks { positionals, labeled }
            | ScopedTypeData::PackFromPacks { positionals, labeled } => {
                self.add_scoped_type(*positionals);
                self.add_scoped_type(*labeled);
            }
            ScopedTypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                self.add_scoped_type(*positionals);
                self.add_scoped_type(*labeled);
                self.add_scoped_type(*return_type);
            }
            ScopedTypeData::PackProjection { subject, .. } => self.add_scoped_type(*subject),
            ScopedTypeData::Lambda(lambda) => self.add_lambda(*lambda),
        }
    }
}

type RuntimeMetadataTypeRoot = (ModuleId, SourceRange, phalcom_semantic::types::id::TypeId);

fn add_conformance_type_roots(
    module: &ModuleId,
    range: SourceRange,
    evidence: &phalcom_semantic::generic_evidence::GenericConformanceEvidence,
    roots: &mut Vec<RuntimeMetadataTypeRoot>,
) {
    use phalcom_semantic::generic_evidence::GenericConformanceEvidence;

    let bindings = match evidence {
        GenericConformanceEvidence::Exact(evidence) => evidence.impl_environment.bindings.values().copied().collect::<Vec<_>>(),
        GenericConformanceEvidence::Derived(evidence) => evidence.impl_bindings.bindings().values().copied().collect(),
        GenericConformanceEvidence::Assumed(_) | GenericConformanceEvidence::CompilerDerived(_) => Vec::new(),
    };
    roots.extend(bindings.into_iter().map(|ty| (module.clone(), range, ty)));
}

fn add_realization_selection_type_roots(
    module: &ModuleId,
    range: SourceRange,
    selection: &phalcom_semantic::checker::analysis::RealizationCapabilitySelection,
    roots: &mut Vec<RuntimeMetadataTypeRoot>,
) {
    let phalcom_semantic::checker::analysis::RealizationCapabilitySelection::Evidenced(selection) = selection else {
        return;
    };
    add_conformance_type_roots(module, range, &selection.evidence, roots);
}

fn contextual_receiver_read_requirement(
    importer: &ModuleId,
    target_type: phalcom_semantic::types::id::TypeId,
    range: SourceRange,
    store: &phalcom_semantic::types::store::TypeStore,
) -> Option<RuntimeDependencyRequirement> {
    let declaration = store.nominal_origin_declaration(target_type)?;
    (&declaration.module != importer).then(|| {
        let target = RuntimeRequirementTarget::Binding {
            module: declaration.module.clone(),
            name: declaration.name.clone(),
        };
        RuntimeDependencyRequirement {
            importer: importer.clone(),
            dependency: declaration.module.clone(),
            range,
            reason: RuntimeDependencyReason::RuntimeDeclarationReference,
            read_target: Some(LinkedReadSpec::Binding(SymbolId {
                module: declaration.module.clone(),
                name: declaration.name.clone(),
            })),
            provenance: RuntimeRequirementProvenance {
                consumer: importer.clone(),
                producer: RuntimeRequirementProducer::ContextualReceiverRead,
                product: target.conservative_product(),
                target,
                timing: RuntimeRequirementTiming::DuringModuleInitializer,
            },
        }
    })
}

/// Projects the same runtime reification/type roots consumed by metadata
/// export into graph-only owner requirements before executable finalization.
pub(super) fn runtime_metadata_type_requirements(
    snapshot: &phalcom_semantic::SemanticSnapshot,
    sources: &BTreeMap<ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
    runtime_modules: &BTreeSet<ModuleId>,
) -> Vec<RuntimeDependencyRequirement> {
    use phalcom_semantic::checker::analysis::{ContextualRealizationOperation, RealizationCapabilitySelection, RealizationCoefficientSource};
    use phalcom_semantic::generic_evidence::GenericConformanceEvidence;

    let mut roots = Vec::<RuntimeMetadataTypeRoot>::new();
    let mut contextual_receiver_requirements = Vec::new();
    for module in runtime_modules {
        if let Some(source) = sources.get(module) {
            roots.extend(
                super::semantic_lowering::anonymous_product_type_roots(module, snapshot, &source.program)
                    .into_iter()
                    .map(|(range, ty)| (module.clone(), range, ty)),
            );
        }
        for (callable, analysis) in snapshot.callable_analyses.iter().filter(|(callable, _)| callable.owner.module() == module) {
            let _ = callable;
            for expression in analysis.expressions.values() {
                let range = expression.range;
                if let Some(specialization) = expression.call_specialization.as_ref() {
                    roots.extend(specialization.bindings.iter().map(|(_, ty)| (module.clone(), range, *ty)));
                    for conformance in specialization.conformances.iter() {
                        if let phalcom_semantic::checker::analysis::GenericCallConformanceSource::Derived { evidence } = &conformance.source {
                            roots.extend(evidence.impl_bindings.bindings().values().copied().map(|ty| (module.clone(), range, ty)));
                        }
                    }
                }
                if let Some(capability) = expression.conditional_capability.as_ref() {
                    roots.extend(capability.applicability.bindings.values().copied().map(|ty| (module.clone(), range, ty)));
                    for conformance in capability.applicability.conformance_provenance.iter().flatten() {
                        if let phalcom_semantic::checker::analysis::GenericCallConformanceSource::Derived { evidence } = &conformance.source {
                            roots.extend(evidence.impl_bindings.bindings().values().copied().map(|ty| (module.clone(), range, ty)));
                        }
                    }
                }
                if let Some(realization) = expression.contextual_realization.as_ref() {
                    add_realization_selection_type_roots(module, realization.source_range, &realization.realization, &mut roots);
                    if let RealizationCapabilitySelection::Evidenced(selection) = &realization.realization
                        && let Some(requirement) =
                            contextual_receiver_read_requirement(module, selection.exact_target, realization.source_range, snapshot.store())
                    {
                        contextual_receiver_requirements.push(requirement);
                    }
                    if let ContextualRealizationOperation::Basis { basis_type, coefficient, .. } = &realization.operation {
                        // The basis constructor is materialized from its canonical
                        // declaration and runtime type metadata, not a GetLinked
                        // class read. Keep its owner in the graph/type lane.
                        roots.push((module.clone(), realization.source_range, *basis_type));
                        if let RealizationCoefficientSource::One { one, .. } = coefficient {
                            add_realization_selection_type_roots(module, realization.source_range, one, &mut roots);
                            if let RealizationCapabilitySelection::Evidenced(selection) = one
                                && let Some(requirement) =
                                    contextual_receiver_read_requirement(module, selection.exact_target, realization.source_range, snapshot.store())
                            {
                                contextual_receiver_requirements.push(requirement);
                            }
                        }
                    }
                }
                if let Some(phalcom_semantic::trait_dispatch::TraitDispatchSite::Evidenced(selection)) = expression.trait_dispatch.as_ref()
                    && let Some(evidence) = snapshot.conformance_evidence_for(selection.exact_target, &selection.exact_trait_ref)
                {
                    roots.extend(evidence.impl_environment.bindings.values().copied().map(|ty| (module.clone(), range, ty)));
                }
            }
            for (expression_id, resolution) in analysis.callable_reference_resolutions.iter() {
                let Some(range) = analysis.expressions.get(expression_id).map(|expression| expression.range) else {
                    continue;
                };
                if let phalcom_semantic::checker::associated::CallableReferenceResolutionKind::BoundFamily { members, .. } = &resolution.kind {
                    for capability in members.iter().filter_map(|member| member.conditional_capability.as_ref()) {
                        roots.extend(capability.applicability.bindings.values().copied().map(|ty| (module.clone(), range, ty)));
                        for conformance in capability.applicability.conformance_provenance.iter().flatten() {
                            if let phalcom_semantic::checker::analysis::GenericCallConformanceSource::Derived { evidence } = &conformance.source {
                                roots.extend(evidence.impl_bindings.bindings().values().copied().map(|ty| (module.clone(), range, ty)));
                            }
                        }
                    }
                }
            }
            for lane in analysis.for_iteration_lanes.values() {
                for resolution in [&lane.iterator, &lane.next] {
                    if let phalcom_semantic::checker::ForIterationRequirementResolution::Evidence { evidence, .. } = resolution {
                        if matches!(evidence, GenericConformanceEvidence::Exact(_) | GenericConformanceEvidence::Derived(_)) {
                            add_conformance_type_roots(module, lane.source_range, evidence, &mut roots);
                        }
                    }
                }
            }
        }
    }

    let mut static_domain_constructors = BTreeMap::new();
    for info in snapshot.data_semantics.data_decls.values() {
        if let Some(static_data) = &info.static_data {
            static_domain_constructors.insert(static_data.domain, static_data.constructors.clone());
        }
    }
    for info in snapshot.enum_semantics.enums.values() {
        if let Some(static_data) = &info.static_data {
            static_domain_constructors.insert(static_data.domain, static_data.constructors.clone());
        }
    }
    let associated_domain_constructors = snapshot
        .associated_static_domains
        .values()
        .map(|product| (product.id.clone(), product.constructors.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut seen = BTreeSet::new();
    let mut requirements = contextual_receiver_requirements;
    for (importer, range, ty) in roots {
        let mut owners = RuntimeMetadataTypeOwnerCollector::new(snapshot.store(), &static_domain_constructors, &associated_domain_constructors);
        owners.add_type(ty);
        for dependency in owners.owners {
            if dependency == importer || !seen.insert((importer.clone(), dependency.clone(), range)) {
                continue;
            }
            requirements.push(RuntimeDependencyRequirement {
                importer: importer.clone(),
                dependency: dependency.clone(),
                range,
                reason: RuntimeDependencyReason::RuntimeDeclarationReference,
                read_target: None,
                provenance: RuntimeRequirementProvenance {
                    consumer: importer.clone(),
                    producer: RuntimeRequirementProducer::MetadataTypeOwner,
                    target: RuntimeRequirementTarget::Module(dependency.clone()),
                    product: RuntimeRequirementProduct::CanonicalIdentity,
                    timing: RuntimeRequirementTiming::BeforeSourceExecution,
                },
            });
        }
    }
    requirements
}

fn add_runtime_trait_environment_roots(
    roots: &mut Vec<RuntimeTraitEnvironmentRootSpec>,
    seen: &mut BTreeSet<(
        ModuleId,
        SourceRange,
        phalcom_semantic::identity::ImplId,
        phalcom_semantic::traits::TraitRequirementId,
        phalcom_semantic::types::id::TypeParameterId,
    )>,
    module: &ModuleId,
    range: SourceRange,
    source_impl: &phalcom_semantic::identity::ImplId,
    requirement: &phalcom_semantic::traits::TraitRequirementId,
    bindings: impl IntoIterator<Item = (phalcom_semantic::types::id::TypeParameterId, phalcom_semantic::types::id::TypeId)>,
) {
    for (parameter, ty) in bindings {
        let identity = (module.clone(), range, source_impl.clone(), requirement.clone(), parameter);
        if !seen.insert(identity) {
            continue;
        }
        let key = format!(
            "trait-environment-{}-{}-{}-{}-{}",
            range.start,
            range.end,
            source_impl.local.0,
            requirement.selector.encode(),
            parameter.index(),
        );
        roots.push((module.clone(), range, source_impl.clone(), requirement.clone(), parameter, ty, key));
    }
}

fn add_contextual_realization_trait_roots(
    roots: &mut Vec<RuntimeTraitEnvironmentRootSpec>,
    seen: &mut BTreeSet<(
        ModuleId,
        SourceRange,
        phalcom_semantic::identity::ImplId,
        phalcom_semantic::traits::TraitRequirementId,
        phalcom_semantic::types::id::TypeParameterId,
    )>,
    module: &ModuleId,
    realization: &phalcom_semantic::checker::analysis::ContextualRealization,
) {
    use phalcom_semantic::checker::analysis::{ContextualRealizationOperation, RealizationCapabilitySelection, RealizationCoefficientSource};

    let mut add_selection = |selection: &RealizationCapabilitySelection| {
        let RealizationCapabilitySelection::Evidenced(selection) = selection else {
            return;
        };
        let bindings = match &selection.evidence {
            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(evidence) => evidence
                .impl_environment
                .bindings
                .iter()
                .map(|(&parameter, &ty)| (parameter, ty))
                .collect::<Vec<_>>(),
            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(evidence) => {
                evidence.impl_bindings.bindings().iter().map(|(&parameter, &ty)| (parameter, ty)).collect()
            }
            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Assumed(_)
            | phalcom_semantic::generic_evidence::GenericConformanceEvidence::CompilerDerived(_) => Vec::new(),
        };
        add_runtime_trait_environment_roots(
            roots,
            seen,
            module,
            realization.source_range,
            &selection.source_impl,
            &selection.requirement,
            bindings,
        );
    };

    add_selection(&realization.realization);
    if let ContextualRealizationOperation::Basis {
        coefficient: RealizationCoefficientSource::One { one, .. },
        ..
    } = &realization.operation
    {
        add_selection(one);
    }
}

fn inherent_impl_domain_for_callable<'a>(
    snapshot: &'a phalcom_semantic::SemanticSnapshot,
    callable: &phalcom_semantic::identity::CallableId,
    expected_impl: Option<&phalcom_semantic::identity::ImplId>,
) -> Option<&'a phalcom_semantic::impls::InherentImplDomain> {
    let phalcom_semantic::impls::CallableDefinitionOrigin::InherentImpl(source_impl) = &snapshot.callable_definitions.get(callable)?.origin else {
        return None;
    };
    if expected_impl.is_some_and(|expected| expected != source_impl) {
        return None;
    }
    snapshot
        .dispatch
        .conditional_members()
        .values()
        .flat_map(|members| members.members.iter())
        .find(|member| &member.impl_id == source_impl)
        .map(|member| member.domain.as_ref())
}

fn inherent_impl_generic_signature<'a>(
    snapshot: &'a phalcom_semantic::SemanticSnapshot,
    callable: &phalcom_semantic::identity::CallableId,
    expected_impl: Option<&phalcom_semantic::identity::ImplId>,
) -> Option<&'a phalcom_semantic::types::parameter::GenericSignature> {
    inherent_impl_domain_for_callable(snapshot, callable, expected_impl).and_then(|domain| domain.generic_signature.as_ref())
}

/// Entry selection for a program compilation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntrySelection {
    /// Compile an already linked module ID.
    ModuleId(ModuleId),
    /// Compile a project rooted at directory with `project.toml`.
    Project(PathBuf),
    /// Compile a package directory containing `package.ph` and `main.ph`.
    Package(PathBuf),
    /// Compile a single module file path.
    Module(PathBuf),
    /// Compile standalone inline source text.
    Inline(Arc<str>),
}

/// One VM-independent compiled module.
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledModule {
    /// Module identity.
    pub id: ModuleId,
    /// Source kind from the linked interface.
    pub kind: ModuleKind,
    /// Source location is attached by a source-backed compiler facade when
    /// available; the VM-independent linker itself only knows logical identity.
    pub source: Option<SourceLocation>,
    /// Source text retained for compilation.
    pub source_text: Option<Arc<str>>,
    /// Linked source interface.
    pub interface: Arc<phalcom_modules::LinkedModuleInterface>,
    /// Materialization/initializer artifact.
    pub plan: ModuleMaterializationPlan,
    /// Symbolic reads retained for compiler/runtime lowering.
    pub linked_reads: Vec<phalcom_modules::LinkedReadSpec>,
    /// Lowering semantics projected from formal analysis.
    pub lowering: Arc<super::semantic_lowering::ModuleLoweringSemantics>,
}

/// Closed compiled program passed to runtime materialization.
#[derive(Clone, Debug)]
pub struct CompiledProgram {
    /// Resolved project universe shared by every linked module.
    pub project_universe: Arc<ProjectUniverse>,
    /// Original linked program plan.
    pub linked: Arc<ExecutableLinkedProgram>,
    /// Compiled modules keyed by semantic identity.
    pub modules: BTreeMap<ModuleId, CompiledModule>,
    /// Program entry module.
    pub entry: ModuleId,
    /// Precomputed initialization order.
    pub initialization_order: Vec<ModuleId>,
    /// Immutable semantic metadata bundle, if retained by build profile.
    pub semantic_metadata: Option<Arc<phalcom_type_meta::SemanticMetadataBundle>>,
}

/// Semantic diagnostics grouped by module.
#[derive(Clone, Default)]
pub struct ProgramSemanticDiagnostics {
    pub by_module: BTreeMap<ModuleId, Vec<SemanticDiagnostic>>,
    pub snapshot: Option<Arc<phalcom_semantic::SemanticSnapshot>>,
}

impl std::fmt::Display for ProgramSemanticDiagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for (module, diags) in &self.by_module {
            for diag in diags {
                if !first {
                    writeln!(f)?;
                }
                first = false;
                writeln!(
                    f,
                    "{}: {:?} {:?}: {} at {:?}",
                    module, diag.severity, diag.code, diag.message, diag.primary_range
                )?;
                for label in &diag.labels {
                    writeln!(f, "  --> label: {} at {:?}", label.message, label.span.range)?;
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for ProgramSemanticDiagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProgramSemanticDiagnostics(\n{}\n)", self)
    }
}

impl ProgramSemanticDiagnostics {
    pub fn from_snapshot(snapshot: &phalcom_semantic::SemanticSnapshot) -> Self {
        let by_module = snapshot
            .diagnostics
            .iter()
            .map(|(module, diagnostics)| (module.clone(), diagnostics.to_vec()))
            .collect();
        Self {
            by_module,
            snapshot: Some(Arc::new(snapshot.clone())),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.by_module.values().all(|v| v.is_empty())
    }

    pub fn has_errors(&self) -> bool {
        self.by_module
            .values()
            .any(|v| v.iter().any(|d| d.severity == phalcom_semantic::DiagnosticSeverity::Error))
    }

    pub fn for_module(&self, module: &ModuleId) -> &[SemanticDiagnostic] {
        self.by_module.get(module).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ModuleId, &[SemanticDiagnostic])> {
        self.by_module.iter().map(|(m, v)| (m, v.as_slice()))
    }
}

/// Program-level compile errors remain structured by phase.
#[derive(Debug, Error, Clone)]
pub enum ProgramCompileError {
    /// Static linking failed.
    #[error(transparent)]
    Link(#[from] LinkError),
    /// Parser error.
    #[error(transparent)]
    Parse(#[from] phalcom_ast::error::SyntaxError),
    /// Interface error.
    #[error(transparent)]
    Interface(#[from] InterfaceError),
    /// Requested entry is not part of the linked plan.
    #[error("entry module {0} is not present in linked program")]
    MissingEntry(ModuleId),
    /// Project is not executable.
    #[error("project manifest at '{0}' is not executable (no entry declared)")]
    ProjectNotExecutable(String),
    /// Package is not executable.
    #[error("package at '{0}' is not executable (missing main.ph)")]
    PackageNotExecutable(String),
    /// Project resolution failure.
    #[error("project error: {0}")]
    Project(#[from] ProjectError),
    /// Module resolution failure.
    #[error("resolution error: {0}")]
    Resolution(#[from] ModuleResolutionError),
    /// Module interface/load failure.
    #[error("module load error: {0}")]
    ModuleLoad(#[from] phalcom_modules::ModuleLoadError),
    /// Source I/O failure.
    #[error("source error: {0}")]
    Source(#[from] SourceError),
    /// Standalone module cannot import arbitrary sibling or project modules.
    #[error(
        "standalone module execution cannot resolve import '{import_name}' (standalone modules only support builtin 'universe' roots; sibling modules require a Project or Package)"
    )]
    StandaloneImportRequiresPackageContext { import_name: Box<str> },
    /// Context-free REPL cannot import modules without project context.
    #[error("context-free inline/REPL execution does not support module import '{import_name}' (requires project context)")]
    ReplImportRequiresProjectContext { import_name: Box<str> },
    /// Semantic type checking errors.
    #[error("semantic error:\n{0}")]
    Semantic(ProgramSemanticDiagnostics),
    /// Semantic update failed without publishing a current executable snapshot.
    #[error(transparent)]
    SemanticUpdate(#[from] phalcom_semantic::SemanticUpdateFailure),
    /// Generic I/O error.
    #[error("io error: {0}")]
    Io(String),
    /// Canonical generic evidence could not be represented for lowering.
    #[error("generic evidence layout error: {0}")]
    GenericEvidenceLayout(String),
    /// Canonical availability proofs could not be established.
    #[error("canonical availability contract error: {0}")]
    AvailabilityContract(String),
}

/// Fully analyzed program snapshot ready for code generation.
#[derive(Clone, Debug)]
pub struct AnalyzedProgram {
    pub project_universe: Arc<ProjectUniverse>,
    pub linked: Arc<ExecutableLinkedProgram>,
    pub semantic: Arc<phalcom_semantic::SemanticSnapshot>,
    pub sources: BTreeMap<ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
    /// Modules that this program owns at runtime. Semantic support modules may
    /// be present in `linked` and `sources` without being rematerialized into
    /// an already-bootstrapped Universe VM.
    pub runtime_modules: BTreeSet<ModuleId>,
    pub entry: ModuleId,
}

/// Analyzer that coordinates discovery, linking, and whole-workspace semantic analysis.
pub struct ProgramAnalyzer;

pub(crate) struct DiscoveredProgramClosure {
    pub(crate) interfaces: BTreeMap<ModuleId, phalcom_modules::UnlinkedModuleInterface>,
    pub(crate) sources: BTreeMap<ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
    pub(crate) resolved: BTreeMap<(ModuleId, String), ModuleId>,
}

pub(crate) fn discover_reachable_dependencies<P: SourceProvider>(
    resolver: &mut ModuleResolver<'_, P>,
    universe: &ProjectUniverse,
    mut interfaces: BTreeMap<ModuleId, phalcom_modules::UnlinkedModuleInterface>,
    mut sources: BTreeMap<ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
    mut resolved: BTreeMap<(ModuleId, String), ModuleId>,
    pending: impl IntoIterator<Item = ModuleId>,
    standalone_entry: Option<&ModuleId>,
) -> Result<DiscoveredProgramClosure, ProgramCompileError> {
    let mut visited = interfaces.keys().cloned().collect::<HashSet<_>>();
    let mut pending = pending.into_iter().collect::<Vec<_>>();
    for module in &pending {
        visited.insert(module.clone());
    }

    while let Some(current_id) = pending.pop() {
        let interface = if let Some(interface) = interfaces.get(&current_id) {
            interface.clone()
        } else {
            let parsed = resolver.load_parsed(&current_id)?;
            let interface = resolver.load_interface(&current_id)?;
            sources.insert(current_id.clone(), parsed);
            interfaces.insert(current_id.clone(), interface.clone());
            interface
        };

        if let Some(project_id) = current_id.project.as_resolved() {
            if universe.get_project(project_id).is_some() {
                let mut curr_parent = current_id.path.parent();
                while let Some(parent) = curr_parent {
                    let pkg_id = ModuleId::resolved(project_id, parent.clone());
                    if visited.insert(pkg_id.clone()) {
                        pending.push(pkg_id);
                    }
                    curr_parent = parent.parent();
                }
            }
        }

        for import_surface in &interface.imports {
            let import_path = match import_surface {
                phalcom_modules::ImportSurface::Module(m) => &m.path,
                phalcom_modules::ImportSurface::Selective(s) => &s.path,
                phalcom_modules::ImportSurface::ReExport(r) => &r.path,
            };

            if standalone_entry.is_some_and(|entry| entry == &current_id) {
                match &import_path.root {
                    phalcom_ast::ast::ImportRoot::Absolute(root) if root.name == "universe" => {}
                    _ => {
                        return Err(ProgramCompileError::StandaloneImportRequiresPackageContext {
                            import_name: import_path.to_string().into(),
                        });
                    }
                }
            }

            let target_unit = resolver.resolve_import(&current_id, import_path)?;
            resolved.insert((current_id.clone(), import_path.to_string()), target_unit.id.clone());
            if visited.insert(target_unit.id.clone()) {
                pending.push(target_unit.id);
            }
        }
    }

    Ok(DiscoveredProgramClosure { interfaces, sources, resolved })
}

/// Runs semantic analysis between symbolic linking and executable finalization.
/// The semantic import identities stay available during analysis; only the
/// finalized projection is returned to compiler consumers.
fn link_analyze_finalize(
    linker: &ModuleLinker,
    entry: ModuleId,
    resolved: &BTreeMap<(ModuleId, String), ModuleId>,
    sources: &BTreeMap<ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
    runtime_modules: &BTreeSet<ModuleId>,
    generation: u64,
    semantic_session: Option<&mut phalcom_semantic::SemanticWorkspaceSession>,
) -> Result<(Arc<ExecutableLinkedProgram>, Arc<phalcom_semantic::SemanticSnapshot>), ProgramCompileError> {
    let semantic = Arc::new(linker.link_semantic(entry.clone(), resolved)?);
    let semantic_input =
        phalcom_semantic::SemanticWorkspaceInput::new(semantic.to_unfinalized_program(), sources.clone(), generation).with_semantic_linked(semantic.clone());

    let mut owned_semantic_session;
    let semantic_session = match semantic_session {
        Some(semantic_session) => semantic_session,
        None => {
            owned_semantic_session = phalcom_semantic::SemanticWorkspaceSession::new();
            &mut owned_semantic_session
        }
    };
    let snapshot = semantic_session
        .update_with_budget_and_cancel(
            semantic_input,
            phalcom_semantic::db::QueryBudget::default(),
            &phalcom_semantic::db::CancellationToken::new(),
        )
        .map_err(phalcom_semantic::SemanticUpdateFailure::from)?
        .snapshot;

    // Semantic snapshots also cover source-backed Universe support modules.
    // Those modules are already initialized by the VM and are not part of this
    // program's materialization closure, so only generated requirements owned
    // by modules we compile can contribute this program's runtime DAG/read
    // layout. Requirements from a user module that target a preinitialized
    // Universe declaration remain in the projection.
    let mut generated_requirements = snapshot
        .generated_runtime_requirements
        .iter()
        .filter(|requirement| runtime_modules.contains(&requirement.importer))
        .cloned()
        .collect::<Vec<_>>();
    generated_requirements.extend(runtime_metadata_type_requirements(&snapshot, sources, runtime_modules));

    let canonical = crate::modules::canonical_universe_program().map_err(|error| ProgramCompileError::AvailabilityContract(error.to_string()))?;
    let executable = linker.finalize_executable_with_availability(
        (*semantic).clone(),
        &snapshot.import_phase_products,
        &generated_requirements,
        canonical.availability_after_bootstrap(),
    )?;
    Ok((Arc::new(executable), snapshot))
}

impl ProgramAnalyzer {
    /// Analyzes an entry selection into an [`AnalyzedProgram`].
    pub fn analyze_entry_selection(entry: EntrySelection) -> Result<AnalyzedProgram, ProgramCompileError> {
        Self::analyze_entry_selection_inner(entry, None)
    }

    /// Analyzes an entry selection using a caller-configured semantic session.
    ///
    /// This supports embeddings and tests that need to control the canonical
    /// semantic source provider while exercising the same link/analyze/finalize
    /// pipeline used by [`Self::analyze_entry_selection`]. The session is
    /// consumed by this one analysis so its workspace generation remains local
    /// to the resulting program.
    pub fn analyze_entry_selection_with_session(
        entry: EntrySelection,
        mut semantic_session: phalcom_semantic::SemanticWorkspaceSession,
    ) -> Result<AnalyzedProgram, ProgramCompileError> {
        Self::analyze_entry_selection_inner(entry, Some(&mut semantic_session))
    }

    fn analyze_entry_selection_inner(
        entry: EntrySelection,
        mut semantic_session: Option<&mut phalcom_semantic::SemanticWorkspaceSession>,
    ) -> Result<AnalyzedProgram, ProgramCompileError> {
        match entry {
            EntrySelection::ModuleId(entry_id) => Err(ProgramCompileError::Io(format!(
                "EntrySelection::ModuleId({entry_id}) cannot be analyzed without an existing linked project universe"
            ))),
            EntrySelection::Project(root_dir) => {
                let manifest_path = if root_dir.ends_with("project.toml") {
                    root_dir
                } else {
                    root_dir.join("project.toml")
                };
                if !manifest_path.exists() {
                    return Err(ProgramCompileError::Io(format!("project manifest not found at {}", manifest_path.display())));
                }
                let mut universe = ProjectUniverse::new();
                let root_id = universe.load_root(&manifest_path)?;
                let project = universe
                    .get_project(root_id)
                    .ok_or_else(|| ProgramCompileError::Io(format!("project {root_id:?} not found")))?;
                let entry_path = project
                    .entry
                    .clone()
                    .ok_or_else(|| ProgramCompileError::ProjectNotExecutable(manifest_path.display().to_string()))?;
                let entry_id = ModuleId {
                    project: root_id.into(),
                    path: entry_path,
                };
                let provider = FilesystemSourceProvider::new();
                Self::discover_and_analyze(Arc::new(universe), provider, entry_id, semantic_session.as_deref_mut())
            }
            EntrySelection::Package(pkg_dir) => {
                let canonical_pkg = pkg_dir
                    .canonicalize()
                    .map_err(|e| ProgramCompileError::Io(format!("{}: {e}", pkg_dir.display())))?;
                let pkg_file = canonical_pkg.join("package.ph");
                if !pkg_file.is_file() {
                    return Err(ProgramCompileError::Project(ProjectError::MissingRootPackage(canonical_pkg)));
                }
                let main_file = canonical_pkg.join("main.ph");
                if !main_file.is_file() {
                    return Err(ProgramCompileError::PackageNotExecutable(canonical_pkg.display().to_string()));
                }
                let mut universe = ProjectUniverse::new();
                let root_id = universe.load_standalone_package(&canonical_pkg, Some("main"))?;
                let entry_id = ModuleId {
                    project: root_id.into(),
                    path: ModulePath::from_components(vec![
                        ModuleComponent::from_identifier("main").map_err(|e| ProgramCompileError::Io(e.to_string()))?,
                    ]),
                };
                let provider = FilesystemSourceProvider::new();
                Self::discover_and_analyze(Arc::new(universe), provider, entry_id, semantic_session.as_deref_mut())
            }
            EntrySelection::Module(file_path) => {
                let canonical = file_path
                    .canonicalize()
                    .map_err(|e| ProgramCompileError::Io(format!("{}: {}", file_path.display(), e)))?;
                let mut universe = ProjectUniverse::new();
                let ownership = classify_entry_ownership(&canonical, &mut universe)?;
                match ownership {
                    EntryOwnership::ProjectOwned { project: root_id } => {
                        let project = universe.get_project(root_id).unwrap();
                        let rel_path = canonical.strip_prefix(&project.source_root).map_err(|_| {
                            ProgramCompileError::Io(format!("file {} not under source root {}", file_path.display(), project.source_root.display()))
                        })?;
                        let module_path = relative_path_to_module_path(rel_path)?;
                        let entry_id = ModuleId {
                            project: root_id.into(),
                            path: module_path,
                        };
                        let provider = FilesystemSourceProvider::new();
                        Self::discover_and_analyze(Arc::new(universe), provider, entry_id, semantic_session.as_deref_mut())
                    }
                    EntryOwnership::StandalonePackageOwned { package_root } => {
                        let root_id = universe.load_standalone_package(&package_root, None)?;
                        let project = universe.get_project(root_id).unwrap();
                        let rel_path = canonical.strip_prefix(&project.source_root).map_err(|_| {
                            ProgramCompileError::Io(format!("file {} not under package root {}", file_path.display(), project.source_root.display()))
                        })?;
                        let module_path = relative_path_to_module_path(rel_path)?;
                        let entry_id = ModuleId {
                            project: root_id.into(),
                            path: module_path,
                        };
                        let provider = FilesystemSourceProvider::new();
                        Self::discover_and_analyze(Arc::new(universe), provider, entry_id, semantic_session.as_deref_mut())
                    }
                    EntryOwnership::StandaloneModule { file } => Self::analyze_standalone_module(file, semantic_session.as_deref_mut()),
                    EntryOwnership::Inline { .. } => Self::analyze_standalone_module(file_path, semantic_session.as_deref_mut()),
                }
            }
            EntrySelection::Inline(source_text) => {
                let parsed = phalcom_ast::parse(&source_text, 0);
                if let Some(error) = parsed.errors.first() {
                    return Err(ProgramCompileError::Parse(error.clone()));
                }
                if !parsed.program.preamble.dependencies.is_empty() {
                    let dep = &parsed.program.preamble.dependencies[0];
                    let import_name = match dep {
                        phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Module(m)) => m.path.to_string(),
                        phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Selective(s)) => s.path.to_string(),
                        phalcom_ast::ast::DependencyDecl::ReExport(r) => r.path.to_string(),
                        phalcom_ast::ast::DependencyDecl::Expose(e) => e.child.name.clone(),
                    };
                    return Err(ProgramCompileError::ReplImportRequiresProjectContext {
                        import_name: import_name.into(),
                    });
                }
                let mut ids = phalcom_modules::SyntheticProjectIdAllocator;
                let entry_id = ModuleId::synthetic(ids.allocate(), ModulePath::root());
                let universe = Arc::new(ProjectUniverse::new());
                let interface = InterfaceBuilder::build(entry_id.clone(), ModuleKind::Module, &parsed.program)?;
                let mut interfaces = BTreeMap::new();
                interfaces.insert(entry_id.clone(), interface);

                let parsed_unit = Arc::new(phalcom_modules::source::ParsedModuleUnit::new(
                    entry_id.clone(),
                    ModuleKind::Module,
                    None,
                    source_text,
                    Arc::new(parsed.program),
                ));
                let mut sources = BTreeMap::new();
                sources.insert(entry_id.clone(), parsed_unit);

                // Inline evaluation has an implicit Universe runtime. Its semantic
                // workspace must therefore include the same source-authored
                // conformance declarations, rather than treating native member
                // surfaces as a substitute for canonical trait evidence.
                let provider = UniverseSourceProvider::new();
                let filesystem = FilesystemSourceProvider::new();
                let mut resolver = ModuleResolver::new(&universe, &filesystem);
                for node in provider
                    .nodes()
                    .iter()
                    .filter(|node| matches!(node.path, ["collections", ..] | ["scalar", ..] | ["text", ..] | ["option", "unit"]))
                {
                    let path = ModulePath::from_components(
                        node.path
                            .iter()
                            .map(|component| ModuleComponent::from_identifier(component).expect("canonical Universe component"))
                            .collect::<Vec<_>>(),
                    );
                    let module = ModuleId::universe(path);
                    interfaces.insert(module.clone(), provider.load_interface(&module)?);
                    sources.insert(module.clone(), provider.load_parsed(&module)?);
                }
                let mut resolved = BTreeMap::new();
                for (module, interface) in &interfaces {
                    if !module.project.is_universe() {
                        continue;
                    }
                    for import in &interface.imports {
                        let path = match import {
                            ImportSurface::Module(decl) => &decl.path,
                            ImportSurface::Selective(decl) => &decl.path,
                            ImportSurface::ReExport(decl) => &decl.path,
                        };
                        let target = resolver.resolve_import(module, path)?;
                        resolved.insert((module.clone(), path.to_string()), target.id);
                    }
                }
                let linker = ModuleLinker::new(universe.clone(), interfaces);
                let runtime_modules = BTreeSet::from([entry_id.clone()]);
                let (linked, semantic) = link_analyze_finalize(&linker, entry_id.clone(), &resolved, &sources, &runtime_modules, 0, semantic_session)?;

                Ok(AnalyzedProgram {
                    project_universe: universe,
                    linked,
                    semantic,
                    sources,
                    runtime_modules,
                    entry: entry_id,
                })
            }
        }
    }

    fn analyze_standalone_module(
        file_path: PathBuf,
        semantic_session: Option<&mut phalcom_semantic::SemanticWorkspaceSession>,
    ) -> Result<AnalyzedProgram, ProgramCompileError> {
        let canonical = file_path
            .canonicalize()
            .map_err(|e| ProgramCompileError::Io(format!("{}: {e}", file_path.display())))?;
        if !canonical.is_file() {
            return Err(ProgramCompileError::Io(format!("{} is not a module file", canonical.display())));
        }
        let source_text: Arc<str> =
            Arc::from(std::fs::read_to_string(&canonical).map_err(|e| ProgramCompileError::Io(format!("{}: {e}", canonical.display())))?);
        let parsed = phalcom_ast::parse(&source_text, 0);
        if let Some(error) = parsed.errors.first() {
            return Err(ProgramCompileError::Parse(error.clone()));
        }
        let mut ids = phalcom_modules::SyntheticProjectIdAllocator;
        let entry_id = ModuleId::synthetic(ids.allocate(), ModulePath::root());
        let universe = Arc::new(ProjectUniverse::new());
        let interface = InterfaceBuilder::build(entry_id.clone(), ModuleKind::Module, &parsed.program)?;
        let mut interfaces = BTreeMap::from([(entry_id.clone(), interface)]);

        let mut sources = BTreeMap::new();
        let parsed_unit = Arc::new(phalcom_modules::source::ParsedModuleUnit::new(
            entry_id.clone(),
            ModuleKind::Module,
            Some(SourceLocation {
                source_id: SourceId(canonical.to_string_lossy().into()),
                display_path: canonical.clone(),
            }),
            source_text,
            Arc::new(parsed.program.clone()),
        ));
        sources.insert(entry_id.clone(), parsed_unit);

        // Standalone execution has the same implicit Universe runtime as inline
        // evaluation. Include the source-authored collection and scalar
        // conformance modules so semantic products such as canonical `for`
        // lowering are derived from their declared contracts, not native
        // surfaces or compiler fallback lookup.
        let universe_provider = UniverseSourceProvider::new();
        for node in universe_provider
            .nodes()
            .iter()
            .filter(|node| matches!(node.path, ["collections", ..] | ["scalar", ..] | ["text", ..] | ["option", "unit"]))
        {
            let path = ModulePath::from_components(
                node.path
                    .iter()
                    .map(|component| ModuleComponent::from_identifier(component).expect("canonical Universe component"))
                    .collect::<Vec<_>>(),
            );
            let module = ModuleId::universe(path);
            interfaces.insert(module.clone(), universe_provider.load_interface(&module)?);
            sources.insert(module.clone(), universe_provider.load_parsed(&module)?);
        }

        for dependency in &parsed.program.preamble.dependencies {
            let path = match dependency {
                phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Module(decl)) => &decl.path,
                phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Selective(decl)) => &decl.path,
                phalcom_ast::ast::DependencyDecl::ReExport(decl) => &decl.path,
                phalcom_ast::ast::DependencyDecl::Expose(decl) => {
                    return Err(ProgramCompileError::StandaloneImportRequiresPackageContext {
                        import_name: decl.child.name.clone().into(),
                    });
                }
            };

            if !matches!(&path.root, phalcom_ast::ast::ImportRoot::Absolute(root) if root.name == "universe") {
                return Err(ProgramCompileError::StandaloneImportRequiresPackageContext {
                    import_name: path.to_string().into(),
                });
            }
        }

        let provider = FilesystemSourceProvider::new();
        let mut resolver = ModuleResolver::new(&universe, &provider);
        let mut resolved = BTreeMap::new();
        for (module, interface) in &interfaces {
            if !module.project.is_universe() {
                continue;
            }
            for import in &interface.imports {
                let path = match import {
                    ImportSurface::Module(decl) => &decl.path,
                    ImportSurface::Selective(decl) => &decl.path,
                    ImportSurface::ReExport(decl) => &decl.path,
                };
                let target = resolver.resolve_import(module, path)?;
                resolved.insert((module.clone(), path.to_string()), target.id);
            }
        }
        let discovered = discover_reachable_dependencies(&mut resolver, &universe, interfaces, sources, resolved, [entry_id.clone()], Some(&entry_id))?;

        let runtime_modules = BTreeSet::from([entry_id.clone()]);
        let linker = ModuleLinker::new(universe.clone(), discovered.interfaces);
        let (linked, semantic) = link_analyze_finalize(
            &linker,
            entry_id.clone(),
            &discovered.resolved,
            &discovered.sources,
            &runtime_modules,
            0,
            semantic_session,
        )?;

        Ok(AnalyzedProgram {
            project_universe: universe,
            linked,
            semantic,
            sources: discovered.sources,
            runtime_modules,
            entry: entry_id,
        })
    }

    fn discover_and_analyze(
        universe: Arc<ProjectUniverse>,
        source_provider: FilesystemSourceProvider,
        entry: ModuleId,
        semantic_session: Option<&mut phalcom_semantic::SemanticWorkspaceSession>,
    ) -> Result<AnalyzedProgram, ProgramCompileError> {
        let mut resolver = ModuleResolver::new(&universe, &source_provider);
        let mut interfaces = BTreeMap::new();
        let mut sources = BTreeMap::new();
        let mut resolved = BTreeMap::new();
        let mut pending = vec![entry.clone()];
        if let Some(project_id) = entry.project.as_resolved() {
            let root_id = ModuleId::resolved(project_id, ModulePath::root());
            pending.push(root_id);
        }

        // Package compilation has the same implicit Universe runtime as inline
        // and standalone execution. Include the source-authored collection and
        // scalar conformance modules even when the entry module does not import
        // the module that declares the canonical conformance.
        let universe_provider = UniverseSourceProvider::new();
        for node in universe_provider
            .nodes()
            .iter()
            .filter(|node| matches!(node.path, ["collections", ..] | ["scalar", ..] | ["text", ..] | ["option", "unit"]))
        {
            let path = ModulePath::from_components(
                node.path
                    .iter()
                    .map(|component| ModuleComponent::from_identifier(component).expect("canonical Universe component"))
                    .collect::<Vec<_>>(),
            );
            let module = ModuleId::universe(path);
            interfaces.insert(module.clone(), universe_provider.load_interface(&module)?);
            sources.insert(module.clone(), universe_provider.load_parsed(&module)?);
        }

        // Resolve imports among the seeded Universe modules before discovery so
        // the linker can retain their cross-module dependency edges.
        for (module, interface) in &interfaces {
            if !module.project.is_universe() {
                continue;
            }
            for import in &interface.imports {
                let path = match import {
                    ImportSurface::Module(decl) => &decl.path,
                    ImportSurface::Selective(decl) => &decl.path,
                    ImportSurface::ReExport(decl) => &decl.path,
                };
                let target = resolver.resolve_import(module, path)?;
                resolved.insert((module.clone(), path.to_string()), target.id);
            }
        }

        let discovered = discover_reachable_dependencies(&mut resolver, &universe, interfaces, sources, resolved, pending, None)?;

        let runtime_modules = discovered
            .interfaces
            .keys()
            .filter(|id| !id.project.is_universe())
            .cloned()
            .collect::<BTreeSet<_>>();
        let linker = ModuleLinker::new(universe.clone(), discovered.interfaces);
        let (linked, semantic) = link_analyze_finalize(
            &linker,
            entry.clone(),
            &discovered.resolved,
            &discovered.sources,
            &runtime_modules,
            0,
            semantic_session,
        )?;

        Ok(AnalyzedProgram {
            project_universe: universe,
            // Universe support modules are already canonical, bootstrapped
            // runtime state. Keep them in the linked/semantic closure, but do
            // not rematerialize their project-specific artifacts over that
            // state when an ordinary package is run in a full VM.
            runtime_modules,
            linked,
            semantic,
            sources: discovered.sources,
            entry,
        })
    }
}

/// Compiler facade over an analyzed semantic program.
pub struct ProgramCompiler;

impl ProgramCompiler {
    /// Compiles an analyzed program into a fully linked `CompiledProgram`.
    pub fn compile_analyzed(analyzed: &AnalyzedProgram) -> Result<CompiledProgram, ProgramCompileError> {
        if analyzed.semantic.has_errors() {
            return Err(ProgramCompileError::Semantic(ProgramSemanticDiagnostics::from_snapshot(&analyzed.semantic)));
        }
        Self::project_analyzed(analyzed)
    }

    /// Compiles the canonical source-complete Universe projection.
    ///
    /// Canonical bootstrap intentionally projects its reviewed source surface
    /// even when semantic diagnostics change. Ordinary user/project compilation
    /// continues to reject semantic errors through [`Self::compile_analyzed`].
    pub(crate) fn compile_analyzed_for_canonical_bootstrap(analyzed: &AnalyzedProgram) -> Result<CompiledProgram, ProgramCompileError> {
        Self::project_analyzed(analyzed)
    }

    fn project_analyzed(analyzed: &AnalyzedProgram) -> Result<CompiledProgram, ProgramCompileError> {
        let mut runtime_root_specs = Vec::<(ModuleId, SourceRange, phalcom_semantic::types::id::TypeId, String)>::new();
        let mut runtime_call_root_specs = Vec::<(
            ModuleId,
            SourceRange,
            phalcom_semantic::identity::CallableId,
            phalcom_semantic::types::id::TypeParameterId,
            phalcom_semantic::types::id::TypeId,
            String,
        )>::new();
        let mut runtime_trait_root_specs = Vec::<RuntimeTraitEnvironmentRootSpec>::new();
        let mut runtime_trait_root_seen = BTreeSet::new();
        for id in analyzed.linked.modules.keys() {
            if !analyzed.runtime_modules.contains(id) {
                continue;
            }
            if let Some(source) = analyzed.sources.get(id) {
                for (range, ty) in super::semantic_lowering::anonymous_product_type_roots(id, &analyzed.semantic, &source.program) {
                    runtime_root_specs.push((id.clone(), range, ty, format!("anonymous-product-{}-{}", range.start, range.end)));
                }
            }
            for (callable, analysis) in analyzed.semantic.callable_analyses.iter().filter(|(callable, _)| callable.owner.module() == id) {
                for expression in analysis.expressions.values() {
                    if let Some(realization) = expression.contextual_realization.as_ref() {
                        add_contextual_realization_trait_roots(&mut runtime_trait_root_specs, &mut runtime_trait_root_seen, id, realization);
                    }
                    let specialization = expression.call_specialization.as_ref();
                    let capability = expression.conditional_capability.as_ref();
                    if specialization.is_none() && capability.is_none() {
                        continue;
                    }
                    let selected_callable = specialization
                        .map(|specialization| &specialization.callable)
                        .or_else(|| capability.map(|capability| &capability.selection.callable))
                        .expect("a specialization or conditional capability was checked above");
                    if let Some(specialization) = specialization {
                        for (parameter, ty) in
                            close_covering_body_bindings(&analyzed.semantic, &specialization.callable, specialization.bindings.iter().copied())?
                        {
                            runtime_call_root_specs.push((
                                id.clone(),
                                expression.range,
                                specialization.callable.clone(),
                                parameter,
                                ty,
                                format!(
                                    "call-environment-{}-{}-{}-{}",
                                    expression.range.start,
                                    expression.range.end,
                                    specialization.callable.selector,
                                    parameter.index()
                                ),
                            ));
                        }
                        for conformance in specialization.conformances.iter() {
                            if let phalcom_semantic::checker::analysis::GenericCallConformanceSource::Derived { evidence } = &conformance.source {
                                for (&parameter, &ty) in evidence.impl_bindings.bindings().iter() {
                                    runtime_call_root_specs.push((
                                        id.clone(),
                                        expression.range,
                                        specialization.callable.clone(),
                                        parameter,
                                        ty,
                                        format!(
                                            "call-derived-{}-{}-{}-{}",
                                            expression.range.start,
                                            expression.range.end,
                                            specialization.callable.selector,
                                            parameter.index()
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                    if let Some(capability) = capability {
                        for (parameter, ty) in close_covering_body_bindings(
                            &analyzed.semantic,
                            &capability.selection.callable,
                            capability.applicability.bindings.iter().map(|(&parameter, &ty)| (parameter, ty)),
                        )? {
                            runtime_call_root_specs.push((
                                id.clone(),
                                expression.range,
                                capability.selection.callable.clone(),
                                parameter,
                                ty,
                                format!(
                                    "call-environment-conditional-{}-{}-{}-{}",
                                    expression.range.start,
                                    expression.range.end,
                                    capability.selection.callable.selector,
                                    parameter.index()
                                ),
                            ));
                        }
                        for conformance in capability.applicability.conformance_provenance.iter().flatten() {
                            if let phalcom_semantic::checker::analysis::GenericCallConformanceSource::Derived { evidence } = &conformance.source {
                                for (&parameter, &ty) in evidence.impl_bindings.bindings().iter() {
                                    runtime_call_root_specs.push((
                                        id.clone(),
                                        expression.range,
                                        capability.selection.callable.clone(),
                                        parameter,
                                        ty,
                                        format!(
                                            "call-derived-conditional-{}-{}-{}-{}",
                                            expression.range.start,
                                            expression.range.end,
                                            capability.selection.callable.selector,
                                            parameter.index()
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                    if let Some(phalcom_semantic::trait_dispatch::TraitDispatchSite::Evidenced(selection)) = expression.trait_dispatch.as_ref()
                        && let Some(evidence) = analyzed.semantic.conformance_evidence_for(selection.exact_target, &selection.exact_trait_ref)
                    {
                        add_runtime_trait_environment_roots(
                            &mut runtime_trait_root_specs,
                            &mut runtime_trait_root_seen,
                            id,
                            expression.range,
                            &selection.source_impl,
                            &selection.requirement,
                            evidence.impl_environment.bindings.iter().map(|(&parameter, &ty)| (parameter, ty)),
                        );
                    }
                    let _ = callable;
                }
                for (expr_id, resolution) in analysis.callable_reference_resolutions.iter() {
                    let Some(range) = analysis.expressions.get(expr_id).map(|expression| expression.range) else {
                        continue;
                    };
                    let phalcom_semantic::checker::associated::CallableReferenceResolutionKind::BoundFamily { members, .. } = &resolution.kind else {
                        continue;
                    };
                    for capability in members.iter().filter_map(|member| member.conditional_capability.as_ref()) {
                        for (parameter, ty) in close_covering_body_bindings(
                            &analyzed.semantic,
                            &capability.selection.callable,
                            capability.applicability.bindings.iter().map(|(&parameter, &ty)| (parameter, ty)),
                        )? {
                            runtime_call_root_specs.push((
                                id.clone(),
                                range,
                                capability.selection.callable.clone(),
                                parameter,
                                ty,
                                format!(
                                    "call-environment-conditional-{}-{}-{}-{}",
                                    range.start,
                                    range.end,
                                    capability.selection.callable.selector,
                                    parameter.index()
                                ),
                            ));
                        }
                        for conformance in capability.applicability.conformance_provenance.iter().flatten() {
                            if let phalcom_semantic::checker::analysis::GenericCallConformanceSource::Derived { evidence } = &conformance.source {
                                for (&parameter, &ty) in evidence.impl_bindings.bindings().iter() {
                                    runtime_call_root_specs.push((
                                        id.clone(),
                                        range,
                                        capability.selection.callable.clone(),
                                        parameter,
                                        ty,
                                        format!(
                                            "call-derived-conditional-{}-{}-{}-{}",
                                            range.start,
                                            range.end,
                                            capability.selection.callable.selector,
                                            parameter.index()
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                }
                for lane in analysis.for_iteration_lanes.values() {
                    for requirement_resolution in [&lane.iterator, &lane.next] {
                        let phalcom_semantic::checker::ForIterationRequirementResolution::Evidence { evidence, requirement, .. } = requirement_resolution
                        else {
                            continue;
                        };
                        match evidence {
                            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(exact) => {
                                add_runtime_trait_environment_roots(
                                    &mut runtime_trait_root_specs,
                                    &mut runtime_trait_root_seen,
                                    id,
                                    lane.source_range,
                                    &exact.source_impl,
                                    requirement,
                                    exact.impl_environment.bindings.iter().map(|(&parameter, &ty)| (parameter, ty)),
                                );
                            }
                            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(derived) => match &derived.origin {
                                phalcom_semantic::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source } => {
                                    add_runtime_trait_environment_roots(
                                        &mut runtime_trait_root_specs,
                                        &mut runtime_trait_root_seen,
                                        id,
                                        lane.source_range,
                                        &exact_source.source_impl,
                                        requirement,
                                        exact_source.impl_environment.bindings.iter().map(|(&parameter, &ty)| (parameter, ty)),
                                    );
                                }
                                phalcom_semantic::generic_evidence::DerivedConformanceOrigin::GuardedRule { rule, .. } => {
                                    add_runtime_trait_environment_roots(
                                        &mut runtime_trait_root_specs,
                                        &mut runtime_trait_root_seen,
                                        id,
                                        lane.source_range,
                                        &rule.impl_id,
                                        requirement,
                                        derived.impl_bindings.bindings().iter().map(|(&parameter, &ty)| (parameter, ty)),
                                    );
                                }
                            },
                            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Assumed(_)
                            | phalcom_semantic::generic_evidence::GenericConformanceEvidence::CompilerDerived(_) => {}
                        }
                    }
                }
            }
        }
        let runtime_root_refs = runtime_root_specs
            .iter()
            .map(|(module, _, ty, key)| (module, key.as_str(), *ty))
            .chain(runtime_call_root_specs.iter().map(|(module, _, _, _, ty, key)| (module, key.as_str(), *ty)))
            .chain(runtime_trait_root_specs.iter().map(|(module, _, _, _, _, ty, key)| (module, key.as_str(), *ty)))
            .collect::<Vec<_>>();
        let exporter = phalcom_semantic::metadata::MetadataExporter::new(
            analyzed.semantic.store(),
            Some(analyzed.semantic.declarations()),
            Some(analyzed.semantic.callable_signatures().as_ref()),
            None,
            phalcom_type_meta::header::MetadataProfile::RuntimePublic,
        )
        .with_project_universe(&analyzed.project_universe)
        .with_algebraic_semantics(&analyzed.semantic.data_semantics, &analyzed.semantic.enum_semantics)
        .with_associated_static_domains(analyzed.semantic.associated_static_domains.as_ref())
        .with_aliases(analyzed.semantic.type_aliases.as_ref());
        let mut stable_call_root_specs = Vec::with_capacity(runtime_call_root_specs.len());
        let mut exporter = exporter;
        for (module, range, callable, parameter, ty, key) in &runtime_call_root_specs {
            stable_call_root_specs.push((
                module.clone(),
                *range,
                callable.clone(),
                phalcom_semantic::metadata::stable_identity::to_stable_callable_with_context(
                    callable,
                    &phalcom_semantic::metadata::stable_identity::StableIdentityContext::new(&analyzed.project_universe),
                ),
                *parameter,
                exporter
                    .export_type_parameter(*parameter)
                    .map_err(|e| ProgramCompileError::Io(format!("metadata export error: {e}")))?,
                *ty,
                key.clone(),
            ));
        }
        let mut stable_trait_root_specs = Vec::with_capacity(runtime_trait_root_specs.len());
        for (module, range, source_impl, requirement, parameter, ty, key) in &runtime_trait_root_specs {
            stable_trait_root_specs.push((
                module.clone(),
                *range,
                source_impl.clone(),
                requirement.clone(),
                exporter
                    .export_type_parameter(*parameter)
                    .map_err(|e| ProgramCompileError::Io(format!("metadata export error: {e}")))?,
                *ty,
                key.clone(),
            ));
        }
        let metadata_bundle = Arc::new(
            exporter
                .build_bundle(&runtime_root_refs)
                .map_err(|e| ProgramCompileError::Io(format!("metadata export error: {e}")))?,
        );
        let mut runtime_type_roots = BTreeMap::<ModuleId, BTreeMap<SourceRange, RuntimeTypeRef>>::new();
        for ((module, range, _, _), root) in runtime_root_specs.iter().zip(metadata_bundle.runtime_roots.iter()) {
            runtime_type_roots.entry(module.clone()).or_default().insert(
                *range,
                RuntimeTypeRef::Base {
                    pool: MetadataPoolId(0),
                    node: root.form,
                },
            );
        }
        let mut runtime_call_environment_recipes = BTreeMap::<ModuleId, BTreeMap<SourceRange, Arc<RuntimeCallEnvironmentRecipe>>>::new();
        let mut conditional_call_environment_recipes =
            BTreeMap::<ModuleId, BTreeMap<(SourceRange, phalcom_semantic::identity::CallableId), Arc<RuntimeCallEnvironmentRecipe>>>::new();
        let mut conditional_call_keys = BTreeSet::<(ModuleId, SourceRange, phalcom_semantic::identity::CallableId)>::new();
        let mut call_bindings = BTreeMap::<
            (ModuleId, SourceRange, phalcom_semantic::identity::CallableId),
            Vec<(phalcom_type_meta::StableTypeParameterRef, RuntimeTypeRecipe)>,
        >::new();
        let mut call_type_recipes = BTreeMap::<
            (
                ModuleId,
                SourceRange,
                phalcom_semantic::identity::CallableId,
                phalcom_semantic::types::id::TypeParameterId,
            ),
            (phalcom_type_meta::StableTypeParameterRef, RuntimeTypeRecipe),
        >::new();
        for ((module, range, callable, stable_callable, parameter, stable_parameter, ty, key), root) in stable_call_root_specs
            .iter()
            .zip(metadata_bundle.runtime_roots.iter().skip(runtime_root_specs.len()))
        {
            let recipe = if phalcom_semantic::checker::associated::contains_any_type_parameter(&analyzed.semantic.store, *ty) {
                RuntimeTypeRecipe::Template(RuntimeTypeRef::Base {
                    pool: MetadataPoolId(0),
                    node: root.form,
                })
            } else {
                RuntimeTypeRecipe::Closed(RuntimeTypeRef::Base {
                    pool: MetadataPoolId(0),
                    node: root.form,
                })
            };
            call_type_recipes.insert((module.clone(), *range, callable.clone(), *parameter), (stable_parameter.clone(), recipe));
            if key.starts_with("call-environment-") {
                call_bindings
                    .entry((module.clone(), *range, callable.clone()))
                    .or_default()
                    .push((stable_parameter.clone(), recipe));
            }
            let _ = stable_callable;
        }
        let mut runtime_trait_environment_bindings = BTreeMap::<ModuleId, super::semantic_lowering::RuntimeTraitEnvironmentBindings>::new();
        for ((module, range, source_impl, requirement, stable_parameter, ty, _key), root) in stable_trait_root_specs.iter().zip(
            metadata_bundle
                .runtime_roots
                .iter()
                .skip(runtime_root_specs.len() + runtime_call_root_specs.len()),
        ) {
            let recipe = if phalcom_semantic::checker::associated::contains_any_type_parameter(&analyzed.semantic.store, *ty) {
                RuntimeTypeRecipe::Template(RuntimeTypeRef::Base {
                    pool: MetadataPoolId(0),
                    node: root.form,
                })
            } else {
                RuntimeTypeRecipe::Closed(RuntimeTypeRef::Base {
                    pool: MetadataPoolId(0),
                    node: root.form,
                })
            };
            let bindings = runtime_trait_environment_bindings
                .entry(module.clone())
                .or_default()
                .entry((*range, source_impl.clone(), requirement.clone()))
                .or_insert_with(|| Box::new([]));
            let mut updated = bindings.to_vec();
            updated.push((stable_parameter.clone(), recipe));
            updated.sort_by(|(left, _), (right, _)| left.cmp(right));
            *bindings = updated.into_boxed_slice();
        }
        let mut call_conformance_slots =
            BTreeMap::<(ModuleId, SourceRange, phalcom_semantic::identity::CallableId), Vec<crate::typing::RuntimeConformanceSlotRecipe>>::new();
        let default_requirement_slots = super::semantic_lowering::trait_default_requirement_slots(&analyzed.semantic)
            .map_err(|error| ProgramCompileError::GenericEvidenceLayout(error.to_string()))?;

        for (callable, analysis) in analyzed.semantic.callable_analyses.iter() {
            if !analyzed.runtime_modules.contains(callable.owner.module()) {
                continue;
            }
            let caller_owner_sig = match &callable.owner {
                phalcom_semantic::identity::CallableOwnerId::Declaration(decl) => {
                    if callable.side == phalcom_semantic::identity::DispatchSide::Instance {
                        analyzed
                            .semantic
                            .declarations
                            .generic_signature(decl)
                            .or_else(|| analyzed.semantic.trait_headers.get(decl).and_then(|h| h.generic_signature.as_ref()))
                    } else {
                        None
                    }
                }
                phalcom_semantic::identity::CallableOwnerId::Variant(var) => {
                    if callable.side == phalcom_semantic::identity::DispatchSide::Instance {
                        analyzed.semantic.declarations.generic_signature(&var.owner)
                    } else {
                        None
                    }
                }
                phalcom_semantic::identity::CallableOwnerId::Conformance(impl_id) => analyzed
                    .semantic
                    .conformance_witness_plans
                    .get(impl_id)
                    .and_then(|plan| plan.generic_signature.as_ref())
                    .or_else(|| {
                        analyzed
                            .semantic
                            .derived_conformance_rules
                            .iter()
                            .find(|r| &r.impl_id == impl_id)
                            .map(|r| &r.generic_signature)
                    }),
            };
            let mut caller_evidence = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::empty();
            if let Some(owner_sig) = caller_owner_sig {
                let owner_env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                    &owner_sig.constraints,
                    owner_sig.owner.clone(),
                    &analyzed.semantic.trait_surfaces,
                )
                .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                caller_evidence = caller_evidence
                    .try_compose(&owner_env, &analyzed.semantic.trait_surfaces)
                    .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
            }
            if let Some(impl_signature) = inherent_impl_generic_signature(&analyzed.semantic, callable, None) {
                let impl_env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                    &impl_signature.constraints,
                    impl_signature.owner.clone(),
                    &analyzed.semantic.trait_surfaces,
                )
                .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                caller_evidence = caller_evidence
                    .try_compose(&impl_env, &analyzed.semantic.trait_surfaces)
                    .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
            }
            if let Some(sig) = analyzed.semantic.callable_signatures.get(callable) {
                if let Some(generics) = &sig.generics {
                    let callable_env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                        &generics.constraints,
                        generics.owner.clone(),
                        &analyzed.semantic.trait_surfaces,
                    )
                    .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                    caller_evidence = caller_evidence
                        .try_compose(&callable_env, &analyzed.semantic.trait_surfaces)
                        .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                }
            }

            let mut conditional_applications = Vec::new();
            for expression in analysis.expressions.values() {
                let mut capabilities = Vec::new();
                if let Some(capability) = expression.conditional_capability.as_ref() {
                    capabilities.push(capability);
                }
                for (reference_id, resolution) in analysis.callable_reference_resolutions.iter() {
                    if analysis
                        .expressions
                        .get(reference_id)
                        .is_none_or(|reference| reference.range != expression.range)
                    {
                        continue;
                    }
                    if let phalcom_semantic::checker::associated::CallableReferenceResolutionKind::BoundFamily { members, .. } = &resolution.kind {
                        for capability in members.iter().filter_map(|member| member.conditional_capability.as_ref()) {
                            if !capabilities
                                .iter()
                                .any(|existing: &&phalcom_semantic::dispatch::ConditionalSelectedCallCapability| existing.selection == capability.selection)
                            {
                                capabilities.push(capability);
                            }
                        }
                    }
                }
                let specialization = expression.call_specialization.as_ref();
                if capabilities.is_empty() {
                    if specialization.is_some() {
                        conditional_applications.push((expression, specialization, None));
                    }
                } else {
                    for capability in capabilities {
                        conditional_call_keys.insert((callable.owner.module().clone(), expression.range, capability.selection.callable.clone()));
                        conditional_applications.push((expression, specialization, Some(capability)));
                    }
                }
            }
            for (expression, specialization, capability) in conditional_applications {
                let callee = specialization
                    .map(|specialization| &specialization.callable)
                    .or_else(|| capability.map(|capability| &capability.selection.callable))
                    .expect("a specialization or conditional capability was checked above");
                let callee_owner_sig = match &callee.owner {
                    phalcom_semantic::identity::CallableOwnerId::Declaration(decl) => {
                        if callee.side == phalcom_semantic::identity::DispatchSide::Instance {
                            analyzed
                                .semantic
                                .declarations
                                .generic_signature(decl)
                                .or_else(|| analyzed.semantic.trait_headers.get(decl).and_then(|h| h.generic_signature.as_ref()))
                        } else {
                            None
                        }
                    }
                    phalcom_semantic::identity::CallableOwnerId::Variant(var) => {
                        if callee.side == phalcom_semantic::identity::DispatchSide::Instance {
                            analyzed.semantic.declarations.generic_signature(&var.owner)
                        } else {
                            None
                        }
                    }
                    phalcom_semantic::identity::CallableOwnerId::Conformance(impl_id) => analyzed
                        .semantic
                        .conformance_witness_plans
                        .get(impl_id)
                        .and_then(|plan| plan.generic_signature.as_ref())
                        .or_else(|| {
                            analyzed
                                .semantic
                                .derived_conformance_rules
                                .iter()
                                .find(|r| &r.impl_id == impl_id)
                                .map(|r| &r.generic_signature)
                        }),
                };
                let mut callee_evidence = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::empty();
                if let Some(owner_sig) = callee_owner_sig {
                    let owner_env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                        &owner_sig.constraints,
                        owner_sig.owner.clone(),
                        &analyzed.semantic.trait_surfaces,
                    )
                    .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                    callee_evidence = callee_evidence
                        .try_compose(&owner_env, &analyzed.semantic.trait_surfaces)
                        .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                }
                if let Some(capability) = capability {
                    let provenance = capability.applicability.conformance_provenance.as_deref().ok_or_else(|| {
                        ProgramCompileError::GenericEvidenceLayout(format!(
                            "selected conditional impl {:?} has no published proof provenance",
                            capability.selection.impl_id
                        ))
                    })?;
                    let impl_domain = inherent_impl_domain_for_callable(&analyzed.semantic, callee, Some(&capability.selection.impl_id)).ok_or_else(|| {
                        ProgramCompileError::GenericEvidenceLayout(format!(
                            "selected conditional impl {:?} does not match the source impl of callable {:?}",
                            capability.selection.impl_id, callee
                        ))
                    })?;
                    let signature_obligations = impl_domain
                        .constraints
                        .iter()
                        .filter(|constraint| matches!(constraint, phalcom_semantic::types::parameter::GenericConstraint::Conforms { .. }))
                        .count();
                    if signature_obligations != provenance.len() {
                        return Err(ProgramCompileError::GenericEvidenceLayout(format!(
                            "selected conditional impl {:?} publishes {} proof sources for {} conformance slots",
                            capability.selection.impl_id,
                            provenance.len(),
                            signature_obligations
                        )));
                    }
                    if let Some(impl_signature) = impl_domain.generic_signature.as_ref() {
                        let impl_environment = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                            &impl_domain.constraints,
                            impl_signature.owner.clone(),
                            &analyzed.semantic.trait_surfaces,
                        )
                        .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                        callee_evidence = callee_evidence
                            .try_compose(&impl_environment, &analyzed.semantic.trait_surfaces)
                            .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                    } else if !impl_domain.constraints.is_empty() {
                        return Err(ProgramCompileError::GenericEvidenceLayout(format!(
                            "selected conditional impl {:?} has constraints but no generic signature",
                            capability.selection.impl_id
                        )));
                    }
                }
                if let Some(sig) = analyzed.semantic.callable_signatures.get(callee) {
                    if let Some(generics) = &sig.generics {
                        let callable_env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                            &generics.constraints,
                            generics.owner.clone(),
                            &analyzed.semantic.trait_surfaces,
                        )
                        .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                        callee_evidence = callee_evidence
                            .try_compose(&callable_env, &analyzed.semantic.trait_surfaces)
                            .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                    }
                }

                let layout_len = callee_evidence.layout().len();
                let mut slots: Vec<Option<crate::typing::RuntimeConformanceSlotRecipe>> = vec![None; layout_len];
                let mut callee_instantiation = phalcom_semantic::types::instantiation::GenericInstantiation::default();
                if let Some(specialization) = specialization {
                    for &(parameter, ty) in specialization.bindings.iter() {
                        callee_instantiation.bind_type(parameter, ty);
                    }
                }
                if let Some(capability) = capability {
                    for (&parameter, &ty) in &capability.applicability.bindings {
                        callee_instantiation.bind_type(parameter, ty);
                    }
                }
                let mut materialization_store = (*analyzed.semantic.store).clone();
                for (slot_key, slot_idx) in callee_evidence.layout().slots.iter() {
                    let idx = *slot_idx as usize;
                    let instantiated_subject = match &slot_key.conformance.subject {
                        phalcom_semantic::types::parameter::TypeTerm::Canonical(ty) => phalcom_semantic::types::instantiation::materialize_type(
                            &mut materialization_store,
                            *ty,
                            &callee_instantiation,
                            phalcom_semantic::types::instantiation::RowMaterializationMode::PreserveUnboundStableTail,
                        )
                        .map(phalcom_semantic::types::parameter::TypeTerm::Canonical)
                        .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("conformance subject instantiation failed: {error:?}")))?,
                        _ => slot_key.conformance.subject.clone(),
                    };
                    let instantiated_trait_ref = phalcom_semantic::traits::TraitRef::new(
                        slot_key.conformance.trait_ref.declaration.clone(),
                        slot_key
                            .conformance
                            .trait_ref
                            .arguments
                            .iter()
                            .map(|&arg| {
                                phalcom_semantic::types::instantiation::materialize_type(
                                    &mut materialization_store,
                                    arg,
                                    &callee_instantiation,
                                    phalcom_semantic::types::instantiation::RowMaterializationMode::PreserveUnboundStableTail,
                                )
                                .map_err(|error| {
                                    ProgramCompileError::GenericEvidenceLayout(format!("conformance trait argument instantiation failed: {error:?}"))
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                    let instantiated_conformance = phalcom_semantic::generic_evidence::GenericConformanceKey::new(instantiated_subject, instantiated_trait_ref);
                    let binding = specialization
                        .into_iter()
                        .flat_map(|specialization| specialization.conformances.iter())
                        .find(|binding| binding.callee_conformance == instantiated_conformance)
                        .or_else(|| {
                            capability
                                .into_iter()
                                .flat_map(|capability| capability.applicability.conformance_provenance.iter().flatten())
                                .find(|binding| binding.callee_conformance == instantiated_conformance)
                        });
                    if let Some(binding) = binding {
                        match &binding.source {
                            phalcom_semantic::checker::analysis::GenericCallConformanceSource::Exact { evidence } => {
                                if let Some(selection) = evidence.requirements.get(&slot_key.requirement) {
                                    let selector = analyzed
                                        .semantic
                                        .trait_surfaces
                                        .get(&slot_key.conformance.trait_ref.declaration)
                                        .and_then(|s| s.get(&slot_key.requirement))
                                        .map(|m| m.requirement.selector.clone())
                                        .unwrap_or_else(|| slot_key.requirement.selector.clone());
                                    let callable = match selection {
                                        phalcom_semantic::impls::RequirementSelectionTemplate::ConformanceCallable { callable }
                                        | phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { callable }
                                        | phalcom_semantic::impls::RequirementSelectionTemplate::InherentCallable { callable, .. } => Some(callable.clone()),
                                        phalcom_semantic::impls::RequirementSelectionTemplate::ConditionalInherent { candidate, .. } => {
                                            Some(candidate.callable.clone())
                                        }
                                        phalcom_semantic::impls::RequirementSelectionTemplate::DataComponent { .. } => None,
                                    };
                                    if let Some(callable) = callable {
                                        if idx < slots.len() {
                                            let nested_slots =
                                                if matches!(selection, phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { .. }) {
                                                    evidence
                                                        .requirements
                                                        .iter()
                                                        .filter_map(|(requirement, selected)| {
                                                            let callable = phalcom_semantic::trait_dispatch::selected_callable(selected)?;
                                                            let selector = analyzed
                                                                .semantic
                                                                .trait_surfaces
                                                                .get(&slot_key.conformance.trait_ref.declaration)
                                                                .and_then(|surface| surface.get(requirement))
                                                                .map(|member| member.requirement.selector.clone())
                                                                .unwrap_or_else(|| callable.selector.clone());
                                                            default_requirement_slots.get(requirement).copied().map(|slot| {
                                                                (
                                                                    slot,
                                                                    crate::typing::RuntimeConformanceSlotRecipe::Selection {
                                                                        callable,
                                                                        selector,
                                                                        nested_slots: Box::new([]),
                                                                    },
                                                                )
                                                            })
                                                        })
                                                        .collect::<Box<[(u16, crate::typing::RuntimeConformanceSlotRecipe)]>>()
                                                } else {
                                                    Box::new([])
                                                };
                                            slots[idx] = Some(crate::typing::RuntimeConformanceSlotRecipe::Selection {
                                                callable,
                                                selector,
                                                nested_slots,
                                            });
                                        }
                                    }
                                }
                            }
                            phalcom_semantic::checker::analysis::GenericCallConformanceSource::Forwarded { caller_conformance } => {
                                let caller_slot_key =
                                    phalcom_semantic::generic_evidence::GenericEvidenceSlotKey::new(caller_conformance.clone(), slot_key.requirement.clone());
                                if let Some(caller_slot) = caller_evidence.slot_for(&caller_slot_key) {
                                    if idx < slots.len() {
                                        slots[idx] = Some(crate::typing::RuntimeConformanceSlotRecipe::Forwarded(caller_slot));
                                    }
                                }
                            }
                            phalcom_semantic::checker::analysis::GenericCallConformanceSource::Derived { evidence } => {
                                if let Some(selection) = evidence.requirements().get(&slot_key.requirement) {
                                    let selector = analyzed
                                        .semantic
                                        .trait_surfaces
                                        .get(&slot_key.conformance.trait_ref.declaration)
                                        .and_then(|s| s.get(&slot_key.requirement))
                                        .map(|m| m.requirement.selector.clone())
                                        .unwrap_or_else(|| slot_key.requirement.selector.clone());
                                    let (callable, is_trait_default) = match selection {
                                        phalcom_semantic::impls::RequirementSelectionTemplate::ConformanceCallable { callable } => {
                                            (Some(callable.clone()), false)
                                        }
                                        phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { callable } => (Some(callable.clone()), true),
                                        phalcom_semantic::impls::RequirementSelectionTemplate::InherentCallable { callable, .. } => {
                                            (Some(callable.clone()), false)
                                        }
                                        phalcom_semantic::impls::RequirementSelectionTemplate::ConditionalInherent { candidate, .. } => {
                                            (Some(candidate.callable.clone()), false)
                                        }
                                        phalcom_semantic::impls::RequirementSelectionTemplate::DataComponent { .. } => (None, false),
                                    };
                                    if let Some(callable) = callable {
                                        let mut type_bindings = Vec::new();
                                        for &param in evidence.impl_bindings.bindings().keys() {
                                            if let Some((stable_param, recipe)) =
                                                call_type_recipes.get(&(callable.owner.module().clone(), expression.range, callee.clone(), param))
                                            {
                                                type_bindings.push((stable_param.clone(), *recipe));
                                            }
                                        }
                                        type_bindings.sort_by(|(left, _), (right, _)| left.cmp(right));

                                        let mut nested_slots = Vec::new();
                                        if is_trait_default {
                                            let callee_generic_evidence_len = analyzed
                                                .semantic
                                                .trait_headers
                                                .get(&slot_key.conformance.trait_ref.declaration)
                                                .and_then(|h| h.generic_signature.as_ref())
                                                .and_then(|sig| {
                                                    phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                                                        &sig.constraints,
                                                        sig.owner.clone(),
                                                        &analyzed.semantic.trait_surfaces,
                                                    )
                                                    .ok()
                                                })
                                                .map_or(0, |env| env.len() as u16);
                                            // A selected default executes its own compiled
                                            // requirement slots. Populate that callee layout
                                            // from the already-proven sibling selections of
                                            // this derived conformance; antecedent evidence is
                                            // populated below from nested proofs.
                                            for (requirement, default_selection) in evidence.requirements().iter() {
                                                let default_slot = default_requirement_slots.get(requirement).copied().ok_or_else(|| {
                                                    ProgramCompileError::GenericEvidenceLayout(format!("missing default requirement slot for {requirement:?}"))
                                                })?;
                                                let default_callable = match default_selection {
                                                    phalcom_semantic::impls::RequirementSelectionTemplate::ConformanceCallable { callable }
                                                    | phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { callable }
                                                    | phalcom_semantic::impls::RequirementSelectionTemplate::InherentCallable { callable, .. } => {
                                                        Some(callable.clone())
                                                    }
                                                    phalcom_semantic::impls::RequirementSelectionTemplate::ConditionalInherent { candidate, .. } => {
                                                        Some(candidate.callable.clone())
                                                    }
                                                    phalcom_semantic::impls::RequirementSelectionTemplate::DataComponent { .. } => None,
                                                };
                                                if let Some(default_callable) = default_callable {
                                                    let default_selector = analyzed
                                                        .semantic
                                                        .trait_surfaces
                                                        .get(&slot_key.conformance.trait_ref.declaration)
                                                        .and_then(|surface| surface.get(requirement))
                                                        .map(|member| member.requirement.selector.clone())
                                                        .unwrap_or_else(|| default_callable.selector.clone());
                                                    nested_slots.push((
                                                        callee_generic_evidence_len + default_slot,
                                                        crate::typing::RuntimeConformanceSlotRecipe::Selection {
                                                            callable: default_callable,
                                                            selector: default_selector,
                                                            nested_slots: Box::new([]),
                                                        },
                                                    ));
                                                }
                                            }
                                        }
                                        if !evidence.nested.is_empty() {
                                            let nested_signature = match &evidence.origin {
                                                phalcom_semantic::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source } => analyzed
                                                    .semantic
                                                    .conformance_witness_plans
                                                    .get(&exact_source.source_impl)
                                                    .and_then(|plan| plan.generic_signature.as_ref()),
                                                phalcom_semantic::generic_evidence::DerivedConformanceOrigin::GuardedRule { rule, .. } => {
                                                    Some(&rule.generic_signature)
                                                }
                                            }
                                            .ok_or_else(|| {
                                                ProgramCompileError::GenericEvidenceLayout(format!(
                                                    "derived conformance {:?} has no published generic signature for nested evidence layout",
                                                    evidence.origin
                                                ))
                                            })?;
                                            if nested_signature.constraints.len() != evidence.nested.len() {
                                                return Err(ProgramCompileError::GenericEvidenceLayout(format!(
                                                    "derived conformance nested proof count {} does not match source constraint count {}",
                                                    evidence.nested.len(),
                                                    nested_signature.constraints.len()
                                                )));
                                            }
                                            let nested_layout = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                                                &nested_signature.constraints,
                                                nested_signature.owner.clone(),
                                                &analyzed.semantic.trait_surfaces,
                                            )
                                            .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;

                                            // Nested proofs retain source-constraint order, including
                                            // Equivalent/Subtype placeholders. Build the runtime
                                            // environment by the selected impl's canonical layout
                                            // identities rather than compacting or reordering proofs.
                                            for (constraint, nested_item) in nested_signature.constraints.iter().zip(evidence.nested.iter()) {
                                                match (constraint, nested_item) {
                                                    (
                                                        phalcom_semantic::types::parameter::GenericConstraint::Conforms { subject, trait_ref },
                                                        phalcom_semantic::generic_evidence::NestedGenericEvidence::Conformance(nested_conformance),
                                                    ) => {
                                                        let conformance_key =
                                                            phalcom_semantic::generic_evidence::GenericConformanceKey::new(subject.clone(), trait_ref.clone());
                                                        for (slot_key, target_slot) in nested_layout.layout().slots.iter() {
                                                            if slot_key.conformance != conformance_key {
                                                                continue;
                                                            }
                                                            let nested_recipe = match nested_conformance {
                                                                phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(exact_ev) => {
                                                                    let selection = exact_ev.requirements.get(&slot_key.requirement).ok_or_else(|| {
                                                                        ProgramCompileError::GenericEvidenceLayout(format!(
                                                                            "exact nested conformance {:?} lacks selected requirement {:?}",
                                                                            exact_ev.exact_trait_ref, slot_key.requirement
                                                                        ))
                                                                    })?;
                                                                    let nested_callable = match selection {
                                                                        phalcom_semantic::impls::RequirementSelectionTemplate::ConformanceCallable {
                                                                            callable,
                                                                        }
                                                                        | phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { callable }
                                                                        | phalcom_semantic::impls::RequirementSelectionTemplate::InherentCallable {
                                                                            callable,
                                                                            ..
                                                                        } => Some(callable.clone()),
                                                                        phalcom_semantic::impls::RequirementSelectionTemplate::ConditionalInherent {
                                                                            candidate,
                                                                            ..
                                                                        } => Some(candidate.callable.clone()),
                                                                        phalcom_semantic::impls::RequirementSelectionTemplate::DataComponent { .. } => None,
                                                                    };
                                                                    if let Some(nested_callable) = nested_callable {
                                                                        let nested_selector = analyzed
                                                                            .semantic
                                                                            .trait_surfaces
                                                                            .get(&slot_key.conformance.trait_ref.declaration)
                                                                            .and_then(|surface| surface.get(&slot_key.requirement))
                                                                            .map(|member| member.requirement.selector.clone())
                                                                            .unwrap_or_else(|| nested_callable.selector.clone());
                                                                        Some(crate::typing::RuntimeConformanceSlotRecipe::Selection {
                                                                            callable: nested_callable,
                                                                            selector: nested_selector,
                                                                            nested_slots: Box::new([]),
                                                                        })
                                                                    } else {
                                                                        None
                                                                    }
                                                                }
                                                                phalcom_semantic::generic_evidence::GenericConformanceEvidence::Assumed(assumed) => {
                                                                    let caller_slot_key = phalcom_semantic::generic_evidence::GenericEvidenceSlotKey::new(
                                                                        assumed.key.clone(),
                                                                        slot_key.requirement.clone(),
                                                                    );
                                                                    let caller_slot = caller_evidence.slot_for(&caller_slot_key).ok_or_else(|| {
                                                                        ProgramCompileError::GenericEvidenceLayout(format!(
                                                                            "assumed nested conformance {:?} has no caller slot for requirement {:?}",
                                                                            assumed.key, slot_key.requirement
                                                                        ))
                                                                    })?;
                                                                    Some(crate::typing::RuntimeConformanceSlotRecipe::Forwarded(caller_slot))
                                                                }
                                                                phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(nested_derived) => {
                                                                    let selection =
                                                                        nested_derived.requirements().get(&slot_key.requirement).ok_or_else(|| {
                                                                            ProgramCompileError::GenericEvidenceLayout(format!(
                                                                                "derived nested conformance lacks selected requirement {:?}",
                                                                                slot_key.requirement
                                                                            ))
                                                                        })?;
                                                                    let nested_callable = match selection {
                                                                        phalcom_semantic::impls::RequirementSelectionTemplate::ConformanceCallable {
                                                                            callable,
                                                                        }
                                                                        | phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { callable }
                                                                        | phalcom_semantic::impls::RequirementSelectionTemplate::InherentCallable {
                                                                            callable,
                                                                            ..
                                                                        } => Some(callable.clone()),
                                                                        phalcom_semantic::impls::RequirementSelectionTemplate::ConditionalInherent {
                                                                            candidate,
                                                                            ..
                                                                        } => Some(candidate.callable.clone()),
                                                                        phalcom_semantic::impls::RequirementSelectionTemplate::DataComponent { .. } => None,
                                                                    };
                                                                    if let Some(nested_callable) = nested_callable {
                                                                        let nested_selector = analyzed
                                                                            .semantic
                                                                            .trait_surfaces
                                                                            .get(&slot_key.conformance.trait_ref.declaration)
                                                                            .and_then(|surface| surface.get(&slot_key.requirement))
                                                                            .map(|member| member.requirement.selector.clone())
                                                                            .unwrap_or_else(|| nested_callable.selector.clone());
                                                                        Some(crate::typing::RuntimeConformanceSlotRecipe::Selection {
                                                                            callable: nested_callable,
                                                                            selector: nested_selector,
                                                                            nested_slots: Box::new([]),
                                                                        })
                                                                    } else {
                                                                        None
                                                                    }
                                                                }
                                                                phalcom_semantic::generic_evidence::GenericConformanceEvidence::CompilerDerived(_) => None,
                                                            };
                                                            if let Some(nested_recipe) = nested_recipe {
                                                                nested_slots.push((*target_slot, nested_recipe));
                                                            }
                                                        }
                                                    }
                                                    (
                                                        phalcom_semantic::types::parameter::GenericConstraint::Equivalent { .. },
                                                        phalcom_semantic::generic_evidence::NestedGenericEvidence::Equivalent,
                                                    )
                                                    | (
                                                        phalcom_semantic::types::parameter::GenericConstraint::Subtype { .. },
                                                        phalcom_semantic::generic_evidence::NestedGenericEvidence::Subtype,
                                                    ) => {}
                                                    _ => {
                                                        return Err(ProgramCompileError::GenericEvidenceLayout(
                                                            "derived nested proof does not align with its source constraint".into(),
                                                        ));
                                                    }
                                                }
                                            }
                                        }

                                        if idx < slots.len() {
                                            slots[idx] = Some(crate::typing::RuntimeConformanceSlotRecipe::DerivedConditional {
                                                callable,
                                                selector,
                                                type_bindings: type_bindings.into_boxed_slice(),
                                                nested_slots: nested_slots.into_boxed_slice(),
                                            });
                                        }
                                    }
                                }
                            }
                            phalcom_semantic::checker::analysis::GenericCallConformanceSource::CompilerDerived { .. } => {}
                        }
                    }
                }
                let resolved_slots = slots
                    .into_iter()
                    .enumerate()
                    .map(|(slot, recipe)| {
                        recipe.ok_or_else(|| {
                            ProgramCompileError::GenericEvidenceLayout(format!(
                                "missing runtime conformance recipe for semantic slot {slot} at call to {callee:?}"
                            ))
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                call_conformance_slots.insert((callable.owner.module().clone(), expression.range, callee.clone()), resolved_slots);
            }
        }

        let mut all_call_keys = std::collections::BTreeSet::new();
        for key in call_bindings.keys() {
            all_call_keys.insert(key.clone());
        }
        for key in call_conformance_slots.keys() {
            all_call_keys.insert(key.clone());
        }

        for (module, range, callee_id) in all_call_keys {
            let mut bindings = call_bindings.remove(&(module.clone(), range, callee_id.clone())).unwrap_or_default();
            bindings.sort_by(|(left, _), (right, _)| left.cmp(right));
            let stable_callable = phalcom_semantic::metadata::stable_identity::to_stable_callable_with_context(
                &callee_id,
                &phalcom_semantic::metadata::stable_identity::StableIdentityContext::new(&analyzed.project_universe),
            );
            let conformance_slots = call_conformance_slots
                .remove(&(module.clone(), range, callee_id.clone()))
                .unwrap_or_default()
                .into_boxed_slice();
            let recipe = Arc::new(RuntimeCallEnvironmentRecipe {
                callable: stable_callable,
                bindings: bindings.into_boxed_slice(),
                conformance_slots,
            });
            if conditional_call_keys.contains(&(module.clone(), range, callee_id.clone())) {
                conditional_call_environment_recipes
                    .entry(module.clone())
                    .or_default()
                    .insert((range, callee_id.clone()), recipe.clone());
            }
            runtime_call_environment_recipes.entry(module).or_default().insert(range, recipe);
        }
        let mut modules = BTreeMap::new();
        for (id, linked_module) in &analyzed.linked.modules {
            if !analyzed.runtime_modules.contains(id) {
                continue;
            }
            let (source, source_text) = if let Some(parsed_unit) = analyzed.sources.get(id) {
                (parsed_unit.source.clone(), Some(parsed_unit.text.clone()))
            } else {
                (None, None)
            };
            let mut lowering = super::semantic_lowering::build_module_lowering_semantics_with_runtime_types_calls_and_conditional_calls(
                id,
                &analyzed.semantic,
                &analyzed.project_universe,
                runtime_type_roots.get(id).unwrap_or(&BTreeMap::new()),
                runtime_call_environment_recipes.get(id).unwrap_or(&BTreeMap::new()),
                conditional_call_environment_recipes.get(id).unwrap_or(&BTreeMap::new()),
            )
            .map_err(|e| ProgramCompileError::Io(format!("lowering projection error in {id}: {e}")))?;
            if let Some(environments) = runtime_trait_environment_bindings.get(id) {
                super::semantic_lowering::attach_runtime_trait_environments(&mut lowering, environments);
            }
            modules.insert(id.clone(), compile_module(id.clone(), linked_module, source, source_text, Arc::new(lowering)));
        }

        Ok(CompiledProgram {
            project_universe: analyzed.project_universe.clone(),
            linked: analyzed.linked.clone(),
            modules,
            entry: analyzed.entry.clone(),
            initialization_order: analyzed
                .linked
                .initialization_order
                .iter()
                .filter(|id| analyzed.runtime_modules.contains(*id))
                .cloned()
                .collect(),
            semantic_metadata: Some(metadata_bundle),
        })
    }

    /// Compiles an entry selection by analyzing it first and compiling the analyzed result.
    pub fn compile_entry_selection(entry: EntrySelection) -> Result<CompiledProgram, ProgramCompileError> {
        let analyzed = ProgramAnalyzer::analyze_entry_selection(entry)?;
        Self::compile_analyzed(&analyzed)
    }
}

fn relative_path_to_module_path(rel_path: &Path) -> Result<ModulePath, ProgramCompileError> {
    let mut components = Vec::new();
    if let Some(parent) = rel_path.parent() {
        for comp in parent.components() {
            if let Component::Normal(os_str) = comp {
                let s = os_str.to_str().unwrap_or_default();
                if !s.is_empty() {
                    components.push(ModuleComponent::from_kebab(s).map_err(|e| ProgramCompileError::Io(e.to_string()))?);
                }
            }
        }
    }
    let file_stem = rel_path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
    if file_stem != "package" && !file_stem.is_empty() {
        components.push(ModuleComponent::from_kebab(file_stem).map_err(|e| ProgramCompileError::Io(e.to_string()))?);
    }
    Ok(ModulePath::from_components(components))
}

fn compile_module(
    id: ModuleId,
    module: &LinkedModule,
    source: Option<SourceLocation>,
    source_text: Option<Arc<str>>,
    lowering: Arc<super::semantic_lowering::ModuleLoweringSemantics>,
) -> CompiledModule {
    let plan = ModuleMaterializationPlan::empty(module);
    CompiledModule {
        id,
        kind: module.interface.kind,
        source,
        source_text,
        interface: Arc::new(module.interface.clone()),
        linked_reads: module.linked_reads.clone(),
        plan,
        lowering,
    }
}

#[cfg(test)]
mod tests {
    use super::{ProgramCompileError, RuntimeMetadataTypeOwnerCollector, SourceRange, contextual_receiver_read_requirement};
    use crate::typing::RuntimeTypeRecipe;
    use phalcom_modules::{
        LinkedReadSpec, ModuleComponent, ModuleId, ModulePath, RuntimeDependencyReason, RuntimeDependencyRequirement, RuntimeRequirementProducer,
        RuntimeRequirementProduct, RuntimeRequirementProvenance, RuntimeRequirementTarget, RuntimeRequirementTiming, SymbolId,
    };
    use phalcom_semantic::identity::DeclarationId;
    use phalcom_semantic::types::id::StaticDomainId;
    use phalcom_semantic::types::kind::KindData;
    use phalcom_semantic::types::store::{TupleTypeElement, TypeStore};
    use phalcom_semantic::types::type_lambda::ScopedTypeData;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn compiler_error_keeps_semantic_publication_failure_typed() {
        let error: ProgramCompileError = phalcom_semantic::SemanticUpdateFailure::Failed("injected update failure".into()).into();
        assert!(
            matches!(error, ProgramCompileError::SemanticUpdate(phalcom_semantic::SemanticUpdateFailure::Failed(message)) if message == "injected update failure")
        );
    }

    #[test]
    fn canonical_range_lower_option_closes_number_at_is_none_call_edge() {
        let canonical = crate::modules::canonical_universe_program().expect("canonical Universe should build");
        let range_id = ModuleId::universe(ModulePath::from_components(vec![
            ModuleComponent::from_identifier("collections").expect("collections component"),
            ModuleComponent::from_identifier("range").expect("range component"),
        ]));
        let source = canonical
            .source_index()
            .unit(&range_id)
            .map(|module| module.text.clone())
            .expect("canonical Range source text");
        let source = source.as_ref();
        let lower_offset = source.find("lowerOpt = _range.rawLower").expect("RangeIterator lower binding");
        let is_none_offset = source[lower_offset..]
            .find("lowerOpt.isNone")
            .map(|offset| lower_offset + offset)
            .expect("RangeIterator lower isNone observation");
        let is_none_end = is_none_offset + "lowerOpt.isNone".len();
        let module = canonical.program().modules.get(&range_id).expect("canonical Range module artifact");
        let (range, recipe) = module
            .lowering
            .call_environment_recipes
            .iter()
            .find(|(range, recipe)| range.start >= is_none_offset && range.end <= is_none_end && recipe.callable.selector.as_ref() == "isNone")
            .expect("RangeIterator lowerOpt.isNone call recipe");
        // Both the nominal Option parameter and its accepted implementation
        // parameter now carry the same closed type at the call edge.
        assert!(
            matches!(
                recipe.bindings.as_ref(),
                [(_, RuntimeTypeRecipe::Closed(left)), (_, RuntimeTypeRecipe::Closed(right))] if left == right
            ),
            "RangeIterator lowerOpt.isNone must close nominal and implementation parameters consistently at {range:?}, got {recipe:?}"
        );
    }

    #[test]
    fn runtime_metadata_type_roots_retain_nested_nominal_owners() {
        let owner = ModuleId::resolved(
            phalcom_modules::identity::ResolvedProjectId::from_raw(77),
            ModulePath::from_components(vec![ModuleComponent::from_identifier("types").expect("valid module name")]),
        );
        let declaration = DeclarationId::new(owner.clone(), "External".into());
        let mut store = TypeStore::new();
        let external = store.nominal_type(declaration);
        let tuple = store.tuple(Box::new([TupleTypeElement::unlabeled(external)]));
        let static_domain_constructors = BTreeMap::new();
        let associated_domain_constructors = BTreeMap::new();

        let mut collector = RuntimeMetadataTypeOwnerCollector::new(&store, &static_domain_constructors, &associated_domain_constructors);
        collector.add_type(tuple);

        assert_eq!(collector.owners, BTreeSet::from([owner]));
    }

    #[test]
    fn runtime_metadata_type_roots_follow_lambda_kinds_and_current_domain_products() {
        let domain_owner = ModuleId::resolved(
            phalcom_modules::identity::ResolvedProjectId::from_raw(78),
            ModulePath::from_components(vec![ModuleComponent::from_identifier("domain").expect("valid module name")]),
        );
        let payload_owner = ModuleId::resolved(
            phalcom_modules::identity::ResolvedProjectId::from_raw(79),
            ModulePath::from_components(vec![ModuleComponent::from_identifier("payload").expect("valid module name")]),
        );
        let domain_declaration = DeclarationId::new(domain_owner.clone(), "Domain".into());
        let payload_declaration = DeclarationId::new(payload_owner.clone(), "Payload".into());
        let mut store = TypeStore::new();
        let domain = store.nominal_static_domain(domain_declaration);
        let payload_domain = store.nominal_static_domain(payload_declaration);
        let domain_kind = store.static_domain_kind(domain);
        let payload_kind = store.static_domain_kind(payload_domain);
        let constructor = store.intern_static_constructor(
            domain,
            DeclarationId::new(domain_owner.clone(), "Domain".into()),
            "Some",
            Box::new([payload_kind]),
        );
        let lambda_kind = store.arrow_kind(Box::new([domain_kind]), domain_kind);
        let body_type = store.unit();
        let body = store.arena_mut().intern_scoped(ScopedTypeData::Free(body_type));
        let lambda = store.lambda(Box::new([lambda_kind]), body, lambda_kind);
        let static_domain_constructors = BTreeMap::from([(domain, vec![constructor].into_boxed_slice())]);
        let associated_domain_constructors = BTreeMap::new();

        let mut collector = RuntimeMetadataTypeOwnerCollector::new(&store, &static_domain_constructors, &associated_domain_constructors);
        collector.add_type(lambda);

        assert_eq!(collector.owners, BTreeSet::from([domain_owner, payload_owner]));
        assert!(matches!(store.get_kind(lambda_kind), KindData::Arrow { .. }));
    }

    #[test]
    fn runtime_metadata_type_roots_ignore_stale_store_constructors() {
        let domain_owner = ModuleId::resolved(
            phalcom_modules::identity::ResolvedProjectId::from_raw(80),
            ModulePath::from_components(vec![ModuleComponent::from_identifier("current").expect("valid module name")]),
        );
        let stale_owner = ModuleId::resolved(
            phalcom_modules::identity::ResolvedProjectId::from_raw(81),
            ModulePath::from_components(vec![ModuleComponent::from_identifier("stale").expect("valid module name")]),
        );
        let declaration = DeclarationId::new(domain_owner.clone(), "Current".into());
        let mut store = TypeStore::new();
        let domain: StaticDomainId = store.nominal_static_domain(declaration);
        let stale_payload = store.nominal_type(DeclarationId::new(stale_owner, "HistoricalPayload".into()));
        let stale_kind = store.kind_of(stale_payload);
        store.intern_static_constructor(
            domain,
            DeclarationId::new(domain_owner.clone(), "Current".into()),
            "OldCase",
            Box::new([stale_kind]),
        );
        let root = store.associated_static_domain_form(domain);
        let static_domain_constructors = BTreeMap::new();
        let associated_domain_constructors = BTreeMap::new();

        let mut collector = RuntimeMetadataTypeOwnerCollector::new(&store, &static_domain_constructors, &associated_domain_constructors);
        collector.add_type(root);

        assert_eq!(collector.owners, BTreeSet::from([domain_owner]));
    }

    #[test]
    fn contextual_realization_receiver_projects_canonical_runtime_read() {
        let importer = ModuleId::resolved(
            phalcom_modules::identity::ResolvedProjectId::from_raw(82),
            ModulePath::from_components(vec![ModuleComponent::from_identifier("consumer").expect("valid module name")]),
        );
        let owner = ModuleId::resolved(
            phalcom_modules::identity::ResolvedProjectId::from_raw(83),
            ModulePath::from_components(vec![ModuleComponent::from_identifier("scalar").expect("valid module name")]),
        );
        let declaration = DeclarationId::new(owner.clone(), "Scalar".into());
        let mut store = TypeStore::new();
        let target = store.nominal_type(declaration);

        let requirement = contextual_receiver_read_requirement(&importer, target, SourceRange::default(), &store)
            .expect("external contextual receiver must produce a runtime read");

        assert_eq!(
            requirement,
            RuntimeDependencyRequirement {
                importer: importer.clone(),
                dependency: owner.clone(),
                range: SourceRange::default(),
                reason: RuntimeDependencyReason::RuntimeDeclarationReference,
                read_target: Some(LinkedReadSpec::Binding(SymbolId {
                    module: owner.clone(),
                    name: "Scalar".into(),
                })),
                provenance: RuntimeRequirementProvenance {
                    consumer: importer.clone(),
                    producer: RuntimeRequirementProducer::ContextualReceiverRead,
                    target: RuntimeRequirementTarget::Binding {
                        module: owner.clone(),
                        name: "Scalar".into(),
                    },
                    // A cross-module declaration binding is materialized by
                    // source initialization, not guaranteed at the native floor.
                    product: RuntimeRequirementProduct::SourceInitializedValue,
                    timing: RuntimeRequirementTiming::DuringModuleInitializer,
                },
            }
        );
        assert!(
            contextual_receiver_read_requirement(&owner, target, SourceRange::default(), &store).is_none(),
            "a local contextual receiver uses its own module global"
        );
    }
}
