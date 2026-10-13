use crate::declarations::DeclarationTypeTable;
use crate::identity::{CallableId, CallableOwnerId, DeclarationId, FieldId, ModuleId};
use crate::signature::{CallableSignatureTable, FieldSignatureTable};
use crate::type_alias::TypeAliasTable;
use crate::types::id::TypeId;
use super::export::MetadataExporter;
use phalcom_type_meta::header::MetadataProfile;
use std::collections::{BTreeSet, VecDeque};

/// Authoritative explicit root set for metadata reachability analysis.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MetadataRootSet {
    pub runtime_types: BTreeSet<TypeId>,
    pub declarations: BTreeSet<DeclarationId>,
    pub aliases: BTreeSet<DeclarationId>,
    pub callables: BTreeSet<CallableId>,
    pub fields: BTreeSet<FieldId>,
}

impl MetadataRootSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.runtime_types.is_empty()
            && self.declarations.is_empty()
            && self.aliases.is_empty()
            && self.callables.is_empty()
            && self.fields.is_empty()
    }

    pub fn len(&self) -> usize {
        self.runtime_types.len()
            + self.declarations.len()
            + self.aliases.len()
            + self.callables.len()
            + self.fields.len()
    }

    pub fn insert_runtime_type(&mut self, ty: TypeId) -> bool {
        self.runtime_types.insert(ty)
    }

    pub fn insert_declaration(&mut self, decl: DeclarationId) -> bool {
        self.declarations.insert(decl)
    }

    pub fn insert_alias(&mut self, alias: DeclarationId) -> bool {
        self.aliases.insert(alias)
    }

    pub fn insert_callable(&mut self, callable: CallableId) -> bool {
        self.callables.insert(callable)
    }

    pub fn insert_field(&mut self, field: FieldId) -> bool {
        self.fields.insert(field)
    }
}

/// Profile and explicit roots guiding metadata retention.
#[derive(Clone, Copy, Debug)]
pub struct MetadataRetentionRequest<'a> {
    pub profile: MetadataProfile,
    pub roots: &'a MetadataRootSet,
    pub member_surfaces: Option<&'a crate::parent_map::ParentMap<DeclarationId, crate::surface::DeclarationSurface>>,
}

impl<'a> MetadataRetentionRequest<'a> {
    pub fn new(profile: MetadataProfile, roots: &'a MetadataRootSet) -> Self {
        Self { profile, roots, member_surfaces: None }
    }

    /// Uses canonical published visibility when retaining declaration members.
    pub fn with_member_surfaces(mut self, surfaces: &'a crate::parent_map::ParentMap<DeclarationId, crate::surface::DeclarationSurface>) -> Self {
        self.member_surfaces = Some(surfaces);
        self
    }
}

/// Complete retained top-level semantic entity set computed by reachability analysis.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MetadataReachabilityClosure {
    pub declarations: BTreeSet<DeclarationId>,
    pub aliases: BTreeSet<DeclarationId>,
    pub callables: BTreeSet<CallableId>,
    pub fields: BTreeSet<FieldId>,
    pub runtime_types: BTreeSet<TypeId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum WorkItem {
    Declaration(DeclarationId),
    Alias(DeclarationId),
    Callable(CallableId),
    Field(FieldId),
}

/// Telemetry metrics produced during reachability closure computation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReachabilityMetrics {
    pub roots_selected: usize,
    pub entities_visited: usize,
    pub dependency_edges_traversed: usize,
    pub declarations_retained: usize,
    pub declarations_rejected_by_profile: usize,
    pub aliases_retained: usize,
    pub callables_retained: usize,
    pub fields_retained: usize,
    pub canonical_declarations_elided: usize,
    pub canonical_callables_elided: usize,
    pub canonical_fields_elided: usize,
}

/// Computes the complete entity closure using exporter-owned type recursion.
///
/// The exporter discovers nominal declarations in recursively reachable type
/// forms; this function applies profile and external-identity policy to those
/// dependencies and repeats until the retained entity set stabilizes.
pub fn compute_reachability_closure<F>(
    request: &MetadataRetentionRequest<'_>,
    declarations: Option<&DeclarationTypeTable>,
    callables: Option<&CallableSignatureTable>,
    fields: Option<&FieldSignatureTable>,
    aliases: Option<&TypeAliasTable>,
    dependency_exporter: &mut MetadataExporter<'_>,
    is_external_module: F,
) -> (MetadataReachabilityClosure, ReachabilityMetrics)
where
    F: Fn(&ModuleId) -> bool,
{
    let mut roots = request.roots.clone();
    let mut accumulated_metrics = ReachabilityMetrics::default();

    loop {
        let round_request = MetadataRetentionRequest {
            profile: request.profile,
            roots: &roots,
            member_surfaces: request.member_surfaces,
        };
        let (closure, round_metrics) = compute_entity_closure(
            &round_request,
            declarations,
            callables,
            fields,
            aliases,
            &is_external_module,
        );

        accumulated_metrics.entities_visited += round_metrics.entities_visited;
        accumulated_metrics.dependency_edges_traversed += round_metrics.dependency_edges_traversed;
        accumulated_metrics.declarations_rejected_by_profile += round_metrics.declarations_rejected_by_profile;
        accumulated_metrics.canonical_declarations_elided += round_metrics.canonical_declarations_elided;
        accumulated_metrics.canonical_callables_elided += round_metrics.canonical_callables_elided;
        accumulated_metrics.canonical_fields_elided += round_metrics.canonical_fields_elided;

        let mut expanded = false;
        if request.profile != MetadataProfile::RuntimeMinimal {
            for declaration in dependency_exporter.discover_nominal_declaration_dependencies(&closure.runtime_types) {
                if !closure.declarations.contains(&declaration) {
                    expanded |= roots.insert_declaration(declaration);
                }
            }
        }

        if expanded {
            continue;
        }

        accumulated_metrics.roots_selected = roots.len();
        accumulated_metrics.declarations_retained = closure.declarations.len();
        accumulated_metrics.aliases_retained = closure.aliases.len();
        accumulated_metrics.callables_retained = closure.callables.len();
        accumulated_metrics.fields_retained = closure.fields.len();
        return (closure, accumulated_metrics);
    }
}

/// Computes one entity-only closure round. Type-form children are intentionally
/// left to `MetadataExporter`; this helper never inspects `TypeData`.
fn compute_entity_closure<F>(
    request: &MetadataRetentionRequest<'_>,
    declarations: Option<&DeclarationTypeTable>,
    callables: Option<&CallableSignatureTable>,
    fields: Option<&FieldSignatureTable>,
    aliases: Option<&TypeAliasTable>,
    is_external_module: &F,
) -> (MetadataReachabilityClosure, ReachabilityMetrics)
where
    F: Fn(&ModuleId) -> bool,
{
    let mut closure = MetadataReachabilityClosure::default();
    let mut metrics = ReachabilityMetrics {
        roots_selected: request.roots.len(),
        ..Default::default()
    };

    let mut worklist = VecDeque::<WorkItem>::new();

    // Runtime type roots are retained here, while recursive type-form
    // dependencies are discovered by MetadataExporter during export.
    closure.runtime_types.extend(request.roots.runtime_types.iter().copied());

    for decl in &request.roots.declarations {
        if is_external_module(&decl.module) {
            metrics.canonical_declarations_elided += 1;
        } else if closure.declarations.insert(decl.clone()) {
            worklist.push_back(WorkItem::Declaration(decl.clone()));
        }
    }

    for alias in &request.roots.aliases {
        if is_external_module(&alias.module) {
            // external alias elided
        } else if closure.aliases.insert(alias.clone()) {
            worklist.push_back(WorkItem::Alias(alias.clone()));
        }
    }

    for callable in &request.roots.callables {
        if is_external_module(callable.owner.module()) {
            metrics.canonical_callables_elided += 1;
        } else if closure.callables.insert(callable.clone()) {
            worklist.push_back(WorkItem::Callable(callable.clone()));
        }
    }

    for field in &request.roots.fields {
        if is_external_module(&field.owner.module) {
            metrics.canonical_fields_elided += 1;
        } else if closure.fields.insert(field.clone()) {
            worklist.push_back(WorkItem::Field(field.clone()));
        }
    }

    // 2. Fixed-point worklist expansion
    while let Some(item) = worklist.pop_front() {
        metrics.entities_visited += 1;
        match item {
            WorkItem::Declaration(decl_id) => {
                if let Some(decls) = declarations {
                    if let Some(info) = decls.get(&decl_id) {
                        metrics.dependency_edges_traversed += 1;
                        closure.runtime_types.insert(info.form);

                        if let Some(ref tmpl) = info.supertype_template {
                            metrics.dependency_edges_traversed += 1;
                            closure.runtime_types.insert(tmpl.supertype);
                        }

                        if let Some(ref sig) = info.generic_signature {
                            enqueue_generic_signature(sig, &mut closure, &mut metrics);
                        }

                        // Profile-visible owned callables and fields
                        if request.profile != MetadataProfile::RuntimeMinimal {
                            if let Some(call_table) = callables {
                                for (cid, _) in call_table.iter_declaration_owner(&decl_id) {
                                    if request.member_surfaces.is_some_and(|surfaces| surfaces.get(&decl_id)
                                        .and_then(|surface| surface.surface(cid.side).callable_visibility.get(&cid.selector)) != Some(&crate::surface::MemberVisibility::Public)) { continue; }
                                    metrics.dependency_edges_traversed += 1;
                                    if !is_external_module(cid.owner.module()) {
                                        if closure.callables.insert(cid.clone()) {
                                            worklist.push_back(WorkItem::Callable(cid.clone()));
                                        }
                                    } else {
                                        metrics.canonical_callables_elided += 1;
                                    }
                                }
                            }

                            if let Some(field_table) = fields {
                                for (fid, _) in field_table.iter_owner(&decl_id) {
                                    if request.member_surfaces.is_some_and(|surfaces| surfaces.get(&decl_id)
                                        .and_then(|surface| surface.surface(fid.side).field_visibility.get(fid.name.as_ref())) != Some(&crate::surface::MemberVisibility::Public)) { continue; }
                                    metrics.dependency_edges_traversed += 1;
                                    if !is_external_module(&fid.owner.module) {
                                        if closure.fields.insert(fid.clone()) {
                                            worklist.push_back(WorkItem::Field(fid.clone()));
                                        }
                                    } else {
                                        metrics.canonical_fields_elided += 1;
                                    }
                                }
                            }
                        } else {
                            // Under RuntimeMinimal, declaration does not pull in all methods/fields
                            metrics.declarations_rejected_by_profile += 1;
                        }
                    }
                }
            }

            WorkItem::Callable(callable_id) => {
                if let CallableOwnerId::Declaration(ref owner_decl) = callable_id.owner {
                    metrics.dependency_edges_traversed += 1;
                    if !is_external_module(&owner_decl.module) {
                        if closure.declarations.insert(owner_decl.clone()) {
                            worklist.push_back(WorkItem::Declaration(owner_decl.clone()));
                        }
                    } else {
                        metrics.canonical_declarations_elided += 1;
                    }
                }

                if let Some(call_table) = callables {
                    if let Some(sig) = call_table.get(&callable_id) {
                        for p in sig.parameters.iter() {
                            if let Some(ty) = p.declared_type.canonical_type() {
                                metrics.dependency_edges_traversed += 1;
                                closure.runtime_types.insert(ty);
                            }
                        }

                        if let Some(ty) = sig.declared_return.canonical_type() {
                            metrics.dependency_edges_traversed += 1;
                            closure.runtime_types.insert(ty);
                        }

                        if let Some(ref inferred) = sig.inferred_return {
                            if let Some(ty) = inferred.ty() {
                                metrics.dependency_edges_traversed += 1;
                                closure.runtime_types.insert(ty);
                            }
                        }

                        if let Some(ref generics) = sig.generics {
                            enqueue_generic_signature(generics, &mut closure, &mut metrics);
                        }
                    }
                }
            }

            WorkItem::Field(field_id) => {
                metrics.dependency_edges_traversed += 1;
                if !is_external_module(&field_id.owner.module) {
                    if closure.declarations.insert(field_id.owner.clone()) {
                        worklist.push_back(WorkItem::Declaration(field_id.owner.clone()));
                    }
                } else {
                    metrics.canonical_declarations_elided += 1;
                }

                if let Some(field_table) = fields {
                    if let Some(sig) = field_table.get(&field_id) {
                        if let Some(ty) = sig.declared_type.canonical_type() {
                            metrics.dependency_edges_traversed += 1;
                            closure.runtime_types.insert(ty);
                        }
                    }
                }
            }

            WorkItem::Alias(alias_id) => {
                if let Some(alias_table) = aliases {
                    if let Some(info) = alias_table.get(&alias_id) {
                        metrics.dependency_edges_traversed += 1;
                        closure.runtime_types.insert(info.form);

                        if let Some(ref sig) = info.generic_signature {
                            enqueue_generic_signature(sig, &mut closure, &mut metrics);
                        }
                    }
                }
            }

        }
    }

    metrics.declarations_retained = closure.declarations.len();
    metrics.aliases_retained = closure.aliases.len();
    metrics.callables_retained = closure.callables.len();
    metrics.fields_retained = closure.fields.len();

    (closure, metrics)
}

fn enqueue_generic_signature(
    sig: &crate::types::parameter::GenericSignature,
    closure: &mut MetadataReachabilityClosure,
    metrics: &mut ReachabilityMetrics,
) {
    for constraint in sig.constraints.iter() {
        match constraint {
            crate::types::parameter::GenericConstraint::Subtype { lower, upper } => {
                if let crate::types::parameter::TypeTerm::Canonical(ty) = lower {
                    metrics.dependency_edges_traversed += 1;
                    closure.runtime_types.insert(*ty);
                }
                if let crate::types::parameter::TypeTerm::Canonical(ty) = upper {
                    metrics.dependency_edges_traversed += 1;
                    closure.runtime_types.insert(*ty);
                }
            }
            crate::types::parameter::GenericConstraint::Equivalent { left, right } => {
                if let crate::types::parameter::TypeTerm::Canonical(ty) = left {
                    metrics.dependency_edges_traversed += 1;
                    closure.runtime_types.insert(*ty);
                }
                if let crate::types::parameter::TypeTerm::Canonical(ty) = right {
                    metrics.dependency_edges_traversed += 1;
                    closure.runtime_types.insert(*ty);
                }
            }
            crate::types::parameter::GenericConstraint::Conforms { subject, trait_ref } => {
                if let crate::types::parameter::TypeTerm::Canonical(ty) = subject {
                    metrics.dependency_edges_traversed += 1;
                    closure.runtime_types.insert(*ty);
                }
                for &arg in trait_ref.arguments.iter() {
                    metrics.dependency_edges_traversed += 1;
                    closure.runtime_types.insert(arg);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
    use crate::declarations::{DeclarationTypeTable, NominalDeclarationHeader};
    use crate::signature::{CallableSemanticSignature, FieldSemanticSignature, ReturnContractValidation};
    use crate::types::evidence::EvidenceStatus;
    use crate::types::parameter::TypeTerm;
    use crate::types::store::TypeStore;
    use phalcom_common::modifier::ImplementationKind;
    use phalcom_common::selector::Selector;
    use phalcom_native_meta::{EffectSpec, NativeLifecycleSpec, RaisesSpec, ReturnFlowSpec};

    #[test]
    fn metadata_root_set_tracks_cardinalities() {
        let mut roots = MetadataRootSet::new();
        assert!(roots.is_empty());
        assert_eq!(roots.len(), 0);

        roots.insert_runtime_type(TypeId(1));
        let module = ModuleId::universe_root();
        let decl = DeclarationId::new(module.clone(), "Test".into());
        roots.insert_declaration(decl.clone());

        assert!(!roots.is_empty());
        assert_eq!(roots.len(), 2);
    }

    #[test]
    fn reachability_closure_minimal_vs_public_retention() {
        let mut store = TypeStore::new();
        let int_ty = TypeId(1);
        let local_mod = ModuleId::resolved(
            phalcom_modules::identity::ResolvedProjectId::from_raw(1),
            phalcom_modules::identity::ModulePath::root(),
        );

        let local_decl = DeclarationId::new(local_mod.clone(), "Widget".into());
        let selector = Selector::method("render", vec![]).unwrap();
        let callable = CallableId::new(local_decl.clone(), selector.clone(), crate::identity::DispatchSide::Instance);
        let field_id = FieldId::new(local_decl.clone(), "name", crate::identity::DispatchSide::Instance);

        let mut decl_table = DeclarationTypeTable::new();
        decl_table.insert(NominalDeclarationHeader::from_signature(&mut store, local_decl.clone(), None).into_type_info(None));

        let mut call_table = CallableSignatureTable::new();
        call_table.insert(CallableSemanticSignature {
            callable: callable.clone(),
            owner: local_decl.clone(),
            side: crate::identity::DispatchSide::Instance,
            selector: selector.clone(),
            generics: None,
            parameters: Box::new([]),
            declared_return: DeclaredTypeFact::known(TypeTerm::Canonical(int_ty), DeclaredTypeBasis::SourceAnnotation),
            return_validation: ReturnContractValidation::Satisfied(EvidenceStatus::Established),
            inferred_return: None,
            source: None,
            implementation: ImplementationKind::Phalcom,
            native_id: None,
            effects: EffectSpec::Unknown,
            raises: RaisesSpec::Unknown,
            flow: ReturnFlowSpec::Value,
            lifecycle: NativeLifecycleSpec::UNKNOWN,
        });

        let mut field_table = FieldSignatureTable::new();
        field_table.insert(FieldSemanticSignature {
            field: field_id.clone(),
            owner: local_decl.clone(),
            side: crate::identity::DispatchSide::Instance,
            name: "name".into(),
            mutable: false,
            declared_type: DeclaredTypeFact::known(TypeTerm::Canonical(int_ty), DeclaredTypeBasis::DeclarationSemantics),
            source: None,
        });

        // 1. RuntimeMinimal: root only contains local_decl, should NOT pull in render or name
        let mut minimal_roots = MetadataRootSet::new();
        minimal_roots.insert_declaration(local_decl.clone());

        let req_minimal = MetadataRetentionRequest::new(MetadataProfile::RuntimeMinimal, &minimal_roots);
        let (closure_min, metrics_min) = compute_entity_closure(
            &req_minimal,
            Some(&decl_table),
            Some(&call_table),
            Some(&field_table),
            None,
            &|m: &ModuleId| m.project.is_universe(),
        );

        assert_eq!(closure_min.declarations.len(), 1);
        assert_eq!(closure_min.callables.len(), 0);
        assert_eq!(closure_min.fields.len(), 0);
        assert!(closure_min.declarations.contains(&local_decl));
        assert_eq!(metrics_min.declarations_rejected_by_profile, 1);

        // 2. RuntimePublic: root contains local_decl, SHOULD pull in owned callables and fields
        let mut public_roots = MetadataRootSet::new();
        public_roots.insert_declaration(local_decl.clone());

        let req_pub = MetadataRetentionRequest::new(MetadataProfile::RuntimePublic, &public_roots);
        let (closure_pub, metrics_pub) = compute_entity_closure(
            &req_pub,
            Some(&decl_table),
            Some(&call_table),
            Some(&field_table),
            None,
            &|m: &ModuleId| m.project.is_universe(),
        );

        assert_eq!(closure_pub.declarations.len(), 1);
        assert_eq!(closure_pub.callables.len(), 1);
        assert_eq!(closure_pub.fields.len(), 1);
        assert!(closure_pub.callables.contains(&callable));
        assert!(closure_pub.fields.contains(&field_id));
        assert_eq!(metrics_pub.canonical_declarations_elided, 0);
    }

    #[test]
    fn reachability_closure_elides_canonical_authorities() {
        let canonical_mod = ModuleId::universe_root();
        let canonical_decl = DeclarationId::new(canonical_mod.clone(), "Int".into());

        let mut roots = MetadataRootSet::new();
        roots.insert_declaration(canonical_decl.clone());

        let req = MetadataRetentionRequest::new(MetadataProfile::RuntimePublic, &roots);
        let (closure, metrics) = compute_entity_closure(
            &req,
            None,
            None,
            None,
            None,
            &|m: &ModuleId| m.project.is_universe(),
        );

        assert_eq!(closure.declarations.len(), 0);
        assert_eq!(metrics.canonical_declarations_elided, 1);
    }

    #[test]
    fn reachability_closure_discovers_nested_nominal_dependencies() {
        use crate::field_key::FieldKey;
        use crate::traits::{AssociatedTypeRequirementId, TraitRef};
        use crate::types::row::RecordRowField;
        use crate::types::store::{CallableParameterType, CallableType};

        let mut store = TypeStore::new();
        let mut alloc = phalcom_modules::identity::SyntheticProjectIdAllocator;
        let local_mod = ModuleId::synthetic(
            alloc.allocate(),
            phalcom_modules::identity::ModulePath::root(),
        );
        let external_mod = ModuleId::universe_root();

        let decl_record = DeclarationId::new(local_mod.clone(), "RecordChild".into());
        let decl_callable = DeclarationId::new(local_mod.clone(), "CallableChild".into());
        let decl_pack = DeclarationId::new(local_mod.clone(), "PackChild".into());
        let decl_proj_subj = DeclarationId::new(local_mod.clone(), "ProjSubjectChild".into());
        let decl_proj_arg = DeclarationId::new(local_mod.clone(), "ProjArgChild".into());
        let decl_external = DeclarationId::new(external_mod.clone(), "ExternalChild".into());
        let decl_rolled_back = DeclarationId::new(local_mod.clone(), "RolledBackChild".into());

        let ty_record = store.nominal(decl_record.clone());
        let ty_callable = store.nominal(decl_callable.clone());
        let ty_pack = store.nominal(decl_pack.clone());
        let ty_proj_subj = store.nominal(decl_proj_subj.clone());
        let ty_proj_arg = store.nominal(decl_proj_arg.clone());
        let ty_external = store.nominal(decl_external.clone());
        let ty_rolled_back = store.nominal(decl_rolled_back.clone());

        let mut decl_table = DeclarationTypeTable::new();
        decl_table.insert(NominalDeclarationHeader::from_signature(&mut store, decl_record.clone(), None).into_type_info(None));
        decl_table.insert(NominalDeclarationHeader::from_signature(&mut store, decl_callable.clone(), None).into_type_info(None));
        decl_table.insert(NominalDeclarationHeader::from_signature(&mut store, decl_pack.clone(), None).into_type_info(None));
        decl_table.insert(NominalDeclarationHeader::from_signature(&mut store, decl_proj_subj.clone(), None).into_type_info(None));
        decl_table.insert(NominalDeclarationHeader::from_signature(&mut store, decl_proj_arg.clone(), None).into_type_info(None));
        decl_table.insert(NominalDeclarationHeader::from_signature(&mut store, decl_rolled_back.clone(), None).into_type_info(None));

        // 1. Nested in Record (including external module nominal)
        let record_ty = store.record(Box::new([
            RecordRowField::symbol_required("field_a", ty_record),
            RecordRowField::symbol_required("field_ext", ty_external),
        ]));

        // 2. Nested in Callable
        let callable_ty = store.callable(CallableType {
            parameters: Box::new([CallableParameterType::new(ty_callable)]),
            return_type: store.unit(),
        });

        // 3. Nested in TypePack
        let pack_ty = store.type_pack(Box::new([ty_pack])).unwrap();

        // 4. Nested in AssociatedProjection (both subject and argument)
        let trait_decl = DeclarationId::new(local_mod.clone(), "TraitDef".into());
        let proj_ty = store.associated_projection(
            ty_proj_subj,
            TraitRef::new(trait_decl.clone(), vec![ty_proj_arg].into_boxed_slice()),
            AssociatedTypeRequirementId::new(trait_decl, 0),
        );

        // 5. Unexportable form: Record with non-symbol field key that should roll back its dependencies
        let failing_record_ty = store.record(Box::new([
            RecordRowField::symbol_required("valid_field", ty_rolled_back),
            RecordRowField::required(FieldKey::string("invalid_non_symbol_key"), store.unit()),
        ]));

        // Root set contains only these composite type forms
        let mut roots = MetadataRootSet::new();
        roots.insert_runtime_type(record_ty);
        roots.insert_runtime_type(callable_ty);
        roots.insert_runtime_type(pack_ty);
        roots.insert_runtime_type(proj_ty);
        roots.insert_runtime_type(failing_record_ty);

        // Test 1: RuntimePublic profile - all valid nested local nominals discovered, external elided, failing rolled back
        let req_pub = MetadataRetentionRequest::new(MetadataProfile::RuntimePublic, &roots);
        let mut exporter_pub = MetadataExporter::new(
            &store,
            Some(&decl_table),
            None,
            None,
            MetadataProfile::RuntimePublic,
        );

        let (closure_pub, metrics_pub) = compute_reachability_closure(
            &req_pub,
            Some(&decl_table),
            None,
            None,
            None,
            &mut exporter_pub,
            |m| m.project.is_universe(),
        );

        assert!(closure_pub.declarations.contains(&decl_record), "Record child nominal must be discovered");
        assert!(closure_pub.declarations.contains(&decl_callable), "Callable child nominal must be discovered");
        assert!(closure_pub.declarations.contains(&decl_pack), "Pack child nominal must be discovered");
        assert!(closure_pub.declarations.contains(&decl_proj_subj), "Projection subject child nominal must be discovered");
        assert!(closure_pub.declarations.contains(&decl_proj_arg), "Projection argument child nominal must be discovered");
        assert!(!closure_pub.declarations.contains(&decl_external), "External child nominal must be elided from retained declarations");
        assert_eq!(metrics_pub.canonical_declarations_elided, 1, "External child nominal must be counted in elided metric");
        assert!(!closure_pub.declarations.contains(&decl_rolled_back), "Failed type root dependencies must be rolled back");

        // Test 2: RuntimeMinimal profile - nested nominal declarations are not retained into declarations closure
        let req_min = MetadataRetentionRequest::new(MetadataProfile::RuntimeMinimal, &roots);
        let mut exporter_min = MetadataExporter::new(
            &store,
            Some(&decl_table),
            None,
            None,
            MetadataProfile::RuntimeMinimal,
        );

        let (closure_min, _) = compute_reachability_closure(
            &req_min,
            Some(&decl_table),
            None,
            None,
            None,
            &mut exporter_min,
            |m| m.project.is_universe(),
        );

        assert!(!closure_min.declarations.contains(&decl_record));
        assert!(!closure_min.declarations.contains(&decl_callable));
        assert!(!closure_min.declarations.contains(&decl_pack));
        assert!(!closure_min.declarations.contains(&decl_proj_subj));
        assert!(!closure_min.declarations.contains(&decl_proj_arg));
    }
}
