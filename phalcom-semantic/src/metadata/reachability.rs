use crate::declarations::DeclarationTypeTable;
use crate::identity::{CallableId, CallableOwnerId, DeclarationId, FieldId, ModuleId};
use crate::signature::{CallableSignatureTable, FieldSignatureTable};
use crate::type_alias::TypeAliasTable;
use crate::types::id::TypeId;
use crate::types::store::{AnonymousTupleRemainder, StaticArgument, TypeData, TypeStore};
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetadataRetentionRequest<'a> {
    pub profile: MetadataProfile,
    pub roots: &'a MetadataRootSet,
}

impl<'a> MetadataRetentionRequest<'a> {
    pub fn new(profile: MetadataProfile, roots: &'a MetadataRootSet) -> Self {
        Self { profile, roots }
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
    Type(TypeId),
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

/// Computes the reachability closure from explicit roots and retention policy.
pub fn compute_reachability_closure<F>(
    request: &MetadataRetentionRequest<'_>,
    store: &TypeStore,
    declarations: Option<&DeclarationTypeTable>,
    callables: Option<&CallableSignatureTable>,
    fields: Option<&FieldSignatureTable>,
    aliases: Option<&TypeAliasTable>,
    is_external_module: F,
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

    // 1. Seed roots
    for &ty in &request.roots.runtime_types {
        if closure.runtime_types.insert(ty) {
            worklist.push_back(WorkItem::Type(ty));
        }
    }

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
                        if closure.runtime_types.insert(info.form) {
                            worklist.push_back(WorkItem::Type(info.form));
                        }

                        if let Some(ref tmpl) = info.supertype_template {
                            metrics.dependency_edges_traversed += 1;
                            if closure.runtime_types.insert(tmpl.supertype) {
                                worklist.push_back(WorkItem::Type(tmpl.supertype));
                            }
                        }

                        if let Some(ref sig) = info.generic_signature {
                            enqueue_generic_signature(sig, &mut closure, &mut worklist, &mut metrics);
                        }

                        // Profile-visible owned callables and fields
                        if request.profile != MetadataProfile::RuntimeMinimal {
                            if let Some(call_table) = callables {
                                for (cid, _) in call_table.iter_declaration_owner(&decl_id) {
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
                                if closure.runtime_types.insert(ty) {
                                    worklist.push_back(WorkItem::Type(ty));
                                }
                            }
                        }

                        if let Some(ty) = sig.declared_return.canonical_type() {
                            metrics.dependency_edges_traversed += 1;
                            if closure.runtime_types.insert(ty) {
                                worklist.push_back(WorkItem::Type(ty));
                            }
                        }

                        if let Some(ref inferred) = sig.inferred_return {
                            if let Some(ty) = inferred.ty() {
                                metrics.dependency_edges_traversed += 1;
                                if closure.runtime_types.insert(ty) {
                                    worklist.push_back(WorkItem::Type(ty));
                                }
                            }
                        }

                        if let Some(ref generics) = sig.generics {
                            enqueue_generic_signature(generics, &mut closure, &mut worklist, &mut metrics);
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
                            if closure.runtime_types.insert(ty) {
                                worklist.push_back(WorkItem::Type(ty));
                            }
                        }
                    }
                }
            }

            WorkItem::Alias(alias_id) => {
                if let Some(alias_table) = aliases {
                    if let Some(info) = alias_table.get(&alias_id) {
                        metrics.dependency_edges_traversed += 1;
                        if closure.runtime_types.insert(info.form) {
                            worklist.push_back(WorkItem::Type(info.form));
                        }

                        if let Some(ref sig) = info.generic_signature {
                            enqueue_generic_signature(sig, &mut closure, &mut worklist, &mut metrics);
                        }
                    }
                }
            }

            WorkItem::Type(ty) => {
                if ty.index() < store.type_count() {
                    match store.get(ty) {
                        TypeData::Nominal { declaration } => {
                            if request.profile != MetadataProfile::RuntimeMinimal {
                                metrics.dependency_edges_traversed += 1;
                                if !is_external_module(&declaration.module) {
                                    if closure.declarations.insert(declaration.clone()) {
                                        worklist.push_back(WorkItem::Declaration(declaration.clone()));
                                    }
                                } else {
                                    metrics.canonical_declarations_elided += 1;
                                }
                            }
                        }
                        TypeData::Applied { origin, arguments } => {
                            metrics.dependency_edges_traversed += 1;
                            if closure.runtime_types.insert(*origin) {
                                worklist.push_back(WorkItem::Type(*origin));
                            }
                            for arg in arguments.iter() {
                                if let StaticArgument::Term(arg_ty) = arg {
                                    metrics.dependency_edges_traversed += 1;
                                    if closure.runtime_types.insert(*arg_ty) {
                                        worklist.push_back(WorkItem::Type(*arg_ty));
                                    }
                                }
                            }
                        }
                        TypeData::Union(members) => {
                            for &m in members.iter() {
                                metrics.dependency_edges_traversed += 1;
                                if closure.runtime_types.insert(m) {
                                    worklist.push_back(WorkItem::Type(m));
                                }
                            }
                        }
                        TypeData::Tuple(tuple_ty) => {
                            for el in tuple_ty.elements.iter() {
                                metrics.dependency_edges_traversed += 1;
                                if closure.runtime_types.insert(el.ty) {
                                    worklist.push_back(WorkItem::Type(el.ty));
                                }
                            }
                            if let AnonymousTupleRemainder::Typed(t) = tuple_ty.positional_remainder {
                                metrics.dependency_edges_traversed += 1;
                                if closure.runtime_types.insert(t) {
                                    worklist.push_back(WorkItem::Type(t));
                                }
                            }
                            if let AnonymousTupleRemainder::Typed(t) = tuple_ty.labeled_remainder {
                                metrics.dependency_edges_traversed += 1;
                                if closure.runtime_types.insert(t) {
                                    worklist.push_back(WorkItem::Type(t));
                                }
                            }
                        }
                        _ => {}
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
    worklist: &mut VecDeque<WorkItem>,
    metrics: &mut ReachabilityMetrics,
) {
    for constraint in sig.constraints.iter() {
        match constraint {
            crate::types::parameter::GenericConstraint::Subtype { lower, upper } => {
                if let crate::types::parameter::TypeTerm::Canonical(ty) = lower {
                    metrics.dependency_edges_traversed += 1;
                    if closure.runtime_types.insert(*ty) {
                        worklist.push_back(WorkItem::Type(*ty));
                    }
                }
                if let crate::types::parameter::TypeTerm::Canonical(ty) = upper {
                    metrics.dependency_edges_traversed += 1;
                    if closure.runtime_types.insert(*ty) {
                        worklist.push_back(WorkItem::Type(*ty));
                    }
                }
            }
            crate::types::parameter::GenericConstraint::Equivalent { left, right } => {
                if let crate::types::parameter::TypeTerm::Canonical(ty) = left {
                    metrics.dependency_edges_traversed += 1;
                    if closure.runtime_types.insert(*ty) {
                        worklist.push_back(WorkItem::Type(*ty));
                    }
                }
                if let crate::types::parameter::TypeTerm::Canonical(ty) = right {
                    metrics.dependency_edges_traversed += 1;
                    if closure.runtime_types.insert(*ty) {
                        worklist.push_back(WorkItem::Type(*ty));
                    }
                }
            }
            crate::types::parameter::GenericConstraint::Conforms { subject, trait_ref } => {
                if let crate::types::parameter::TypeTerm::Canonical(ty) = subject {
                    metrics.dependency_edges_traversed += 1;
                    if closure.runtime_types.insert(*ty) {
                        worklist.push_back(WorkItem::Type(*ty));
                    }
                }
                for &arg in trait_ref.arguments.iter() {
                    metrics.dependency_edges_traversed += 1;
                    if closure.runtime_types.insert(arg) {
                        worklist.push_back(WorkItem::Type(arg));
                    }
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
        let (closure_min, metrics_min) = compute_reachability_closure(
            &req_minimal,
            &store,
            Some(&decl_table),
            Some(&call_table),
            Some(&field_table),
            None,
            |m| m.project.is_universe(),
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
        let (closure_pub, metrics_pub) = compute_reachability_closure(
            &req_pub,
            &store,
            Some(&decl_table),
            Some(&call_table),
            Some(&field_table),
            None,
            |m| m.project.is_universe(),
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
        let store = TypeStore::new();
        let canonical_mod = ModuleId::universe_root();
        let canonical_decl = DeclarationId::new(canonical_mod.clone(), "Int".into());

        let mut roots = MetadataRootSet::new();
        roots.insert_declaration(canonical_decl.clone());

        let req = MetadataRetentionRequest::new(MetadataProfile::RuntimePublic, &roots);
        let (closure, metrics) = compute_reachability_closure(
            &req,
            &store,
            None,
            None,
            None,
            None,
            |m| m.project.is_universe(),
        );

        assert_eq!(closure.declarations.len(), 0);
        assert_eq!(metrics.canonical_declarations_elided, 1);
    }
}
