//! Type environment, specialization views, and lazy member projection.

use super::id::{RecordRowId, TypeId, TypeParameterId};
use super::instantiation::{GenericInstantiation, RowMaterializationMode, TypeMaterializationError, materialize_type};
use super::row::RecordRowField;
use super::store::{CallableParameterType, CallableType, TupleTypeElement, TypeData, TypeStore};
use super::substitution::TypeSubstitution;
use crate::checker::analysis_map::{AnalysisMap, FrozenAnalysisMapMutation};
use crate::checker::analysis_set::{AnalysisSet, FrozenAnalysisSetMutation};
use crate::identity::CallableId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrozenTypeEnvironmentMutation {
    Bindings,
    DynamicBindings,
    RowBindings,
}

/// An environment mapping type parameters and `Self` bindings for lazy specialization.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TypeEnvironment {
    pub bindings: AnalysisMap<TypeParameterId, TypeId>,
    pub dynamic_bindings: AnalysisSet<TypeParameterId>,
    pub row_bindings: AnalysisMap<TypeParameterId, RecordRowId>,
    pub self_binding: Option<TypeId>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self::default()
    }

    fn ensure_source_mutable(&self) -> Result<(), FrozenTypeEnvironmentMutation> {
        if self.bindings.is_frozen() {
            return Err(FrozenTypeEnvironmentMutation::Bindings);
        }
        if self.dynamic_bindings.is_frozen() {
            return Err(FrozenTypeEnvironmentMutation::DynamicBindings);
        }
        if self.row_bindings.is_frozen() {
            return Err(FrozenTypeEnvironmentMutation::RowBindings);
        }
        Ok(())
    }

    pub fn bind_param(&mut self, param: TypeParameterId, ty: TypeId) -> Result<(), FrozenTypeEnvironmentMutation> {
        self.ensure_source_mutable()?;
        self.dynamic_bindings
            .remove_source(&param)
            .map_err(|FrozenAnalysisSetMutation| FrozenTypeEnvironmentMutation::DynamicBindings)?;
        self.row_bindings
            .remove_source(&param)
            .map_err(|FrozenAnalysisMapMutation| FrozenTypeEnvironmentMutation::RowBindings)?;
        self.bindings
            .insert_source(param, ty)
            .map_err(|FrozenAnalysisMapMutation| FrozenTypeEnvironmentMutation::Bindings)?;
        Ok(())
    }

    pub fn bind_dynamic(&mut self, param: TypeParameterId) -> Result<(), FrozenTypeEnvironmentMutation> {
        self.ensure_source_mutable()?;
        self.bindings
            .remove_source(&param)
            .map_err(|FrozenAnalysisMapMutation| FrozenTypeEnvironmentMutation::Bindings)?;
        self.row_bindings
            .remove_source(&param)
            .map_err(|FrozenAnalysisMapMutation| FrozenTypeEnvironmentMutation::RowBindings)?;
        self.dynamic_bindings
            .insert_source(param)
            .map_err(|FrozenAnalysisSetMutation| FrozenTypeEnvironmentMutation::DynamicBindings)?;
        Ok(())
    }

    pub fn is_dynamic(&self, param: TypeParameterId) -> bool {
        self.dynamic_bindings.contains(&param)
    }

    pub fn bind_row(&mut self, param: TypeParameterId, row: RecordRowId) -> Result<(), FrozenTypeEnvironmentMutation> {
        self.ensure_source_mutable()?;
        self.bindings
            .remove_source(&param)
            .map_err(|FrozenAnalysisMapMutation| FrozenTypeEnvironmentMutation::Bindings)?;
        self.dynamic_bindings
            .remove_source(&param)
            .map_err(|FrozenAnalysisSetMutation| FrozenTypeEnvironmentMutation::DynamicBindings)?;
        self.row_bindings
            .insert_source(param, row)
            .map_err(|FrozenAnalysisMapMutation| FrozenTypeEnvironmentMutation::RowBindings)?;
        Ok(())
    }

    /// Returns the one canonical argument category bound to `param`, if any.
    pub fn argument(&self, param: TypeParameterId) -> Option<super::store::StaticArgument> {
        self.bindings
            .get(&param)
            .copied()
            .map(super::store::StaticArgument::Term)
            .or_else(|| self.dynamic_bindings.contains(&param).then_some(super::store::StaticArgument::DynamicType))
            .or_else(|| self.row_bindings.get(&param).copied().map(super::store::StaticArgument::RecordRow))
    }

    pub fn bind_self(&mut self, ty: TypeId) -> Result<(), FrozenTypeEnvironmentMutation> {
        self.ensure_source_mutable()?;
        self.self_binding = Some(ty);
        Ok(())
    }

    pub fn get_param(&self, param: TypeParameterId) -> Option<TypeId> {
        self.bindings.get(&param).copied()
    }

    pub fn get_row(&self, param: TypeParameterId) -> Option<RecordRowId> {
        self.row_bindings.get(&param).copied()
    }

    pub fn get_self(&self) -> Option<TypeId> {
        self.self_binding
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty() && self.dynamic_bindings.is_empty() && self.row_bindings.is_empty() && self.self_binding.is_none()
    }

    pub fn to_substitution(&self) -> TypeSubstitution {
        let mut subst = TypeSubstitution::new();
        for (&p, &t) in &self.bindings {
            subst.bind(p, t);
        }
        subst
    }

    pub fn to_generic_instantiation(&self) -> GenericInstantiation {
        let mut instantiation = GenericInstantiation::default();
        for (&parameter, &ty) in &self.bindings {
            instantiation.bind_type(parameter, ty);
        }
        for (&parameter, &row) in &self.row_bindings {
            instantiation.bind_row(parameter, row);
        }
        for &parameter in &self.dynamic_bindings {
            instantiation.bind_dynamic(parameter);
        }
        instantiation
    }
}

/// A lazy view of a `TypeId` under a specialization `TypeEnvironment`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeView {
    pub root: TypeId,
    pub environment: TypeEnvironment,
}

impl TypeView {
    pub fn new(root: TypeId, environment: TypeEnvironment) -> Self {
        Self { root, environment }
    }

    pub fn identity(root: TypeId) -> Self {
        Self {
            root,
            environment: TypeEnvironment::new(),
        }
    }

    /// Materializes this lazy view into a canonical `TypeId` in `store`.
    pub fn materialize(&self, store: &mut TypeStore) -> TypeId {
        if self.environment.is_empty() {
            return self.root;
        }
        materialize_view(store, self.root, &self.environment)
    }

    /// Materializes this view into knowledge-bearing `TypeKnowledge`.
    pub fn materialize_knowledge(&self, store: &mut TypeStore) -> crate::types::evidence::TypeKnowledge {
        if self.environment.is_empty() {
            return crate::types::evidence::TypeKnowledge::assumed(self.root, crate::types::evidence::EvidenceOrigin::CallableSignature);
        }
        if let TypeData::Parameter(param) = store.get(self.root) {
            if self.environment.is_dynamic(*param) {
                return crate::types::evidence::TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape);
            }
        }
        let materialized = self.materialize(store);
        crate::types::evidence::TypeKnowledge::assumed(materialized, crate::types::evidence::EvidenceOrigin::CallableSignature)
    }

    pub fn materialize_checked(&self, store: &mut TypeStore, row_mode: RowMaterializationMode) -> Result<TypeId, TypeMaterializationError> {
        let instantiation = self.environment.to_generic_instantiation();
        materialize_type(store, self.root, &instantiation, row_mode)
    }
}

fn materialize_view(store: &mut TypeStore, ty: TypeId, env: &TypeEnvironment) -> TypeId {
    match store.get(ty).clone() {
        TypeData::Parameter(param) => {
            if let Some(replacement) = env.get_param(param) {
                replacement
            } else {
                ty
            }
        }
        TypeData::SelfType(_term) => {
            if let Some(self_ty) = env.get_self() {
                self_ty
            } else {
                ty
            }
        }
        TypeData::AssociatedProjection(projection) => {
            let subject = materialize_view(store, projection.subject, env);
            let arguments = projection
                .trait_ref
                .arguments
                .iter()
                .map(|&argument| materialize_view(store, argument, env))
                .collect::<Vec<_>>()
                .into_boxed_slice();
            store.associated_projection(
                subject,
                crate::traits::TraitRef::new(projection.trait_ref.declaration, arguments),
                projection.requirement,
            )
        }
        TypeData::TypeMemberProjection(projection) => {
            let kind = store.kind_of(ty);
            let subject = match &projection.subject {
                super::type_member::ProjectionSubject::Type(sub_ty) => super::type_member::ProjectionSubject::Type(materialize_view(store, *sub_ty, env)),
                super::type_member::ProjectionSubject::RecordRow(row_id) => super::type_member::ProjectionSubject::RecordRow(*row_id),
            };
            let arguments = projection
                .arguments
                .iter()
                .map(|arg| match arg {
                    super::type_member::TypeMemberArgument::Type(arg_ty) => super::type_member::TypeMemberArgument::Type(materialize_view(store, *arg_ty, env)),
                    super::type_member::TypeMemberArgument::Const(c) => super::type_member::TypeMemberArgument::Const(c.clone()),
                    super::type_member::TypeMemberArgument::RecordRow(row_id) => super::type_member::TypeMemberArgument::RecordRow(*row_id),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice();
            store.type_member_projection(
                super::type_member::TypeLevelProjection {
                    subject,
                    selector: projection.selector.clone(),
                    arguments,
                    source_declaration: projection.source_declaration.clone(),
                },
                kind,
            )
        }
        TypeData::Applied { origin, arguments } => {
            let subst_origin = materialize_view(store, origin, env);
            let subst_args = arguments
                .iter()
                .map(|argument| match argument {
                    super::store::StaticArgument::Term(arg) => {
                        if let TypeData::Parameter(param) = store.get(*arg) {
                            if env.is_dynamic(*param) {
                                return super::store::StaticArgument::DynamicType;
                            }
                        }
                        super::store::StaticArgument::Term(materialize_view(store, *arg, env))
                    }
                    super::store::StaticArgument::RecordRow(row) => super::store::StaticArgument::RecordRow(*row),
                    super::store::StaticArgument::DynamicType => super::store::StaticArgument::DynamicType,
                })
                .collect::<Vec<_>>();
            store.apply_static_form(subst_origin, &subst_args).unwrap_or(ty)
        }
        TypeData::ExactCase { variant, enum_type } => {
            let subst_enum = materialize_view(store, enum_type, env);
            let variant_id = store.variant_identity(variant).clone();
            store.exact_case_type(&variant_id, subst_enum).unwrap_or(ty)
        }
        TypeData::Union(members) => {
            let subst_members: Vec<TypeId> = members.iter().map(|&m| materialize_view(store, m, env)).collect();
            store.union(&subst_members)
        }
        TypeData::Tuple(elems) => {
            let subst_elems: Vec<TupleTypeElement> = elems
                .elements
                .iter()
                .map(|e| TupleTypeElement {
                    label: e.label.clone(),
                    ty: materialize_view(store, e.ty, env),
                })
                .collect();
            store.tuple(subst_elems.into_boxed_slice())
        }
        TypeData::Record(row_id) => {
            let (fields, tail) = {
                let row = store.record_row(row_id);
                (row.fields.to_vec(), row.tail)
            };
            let subst_fields: Vec<RecordRowField> = fields
                .into_iter()
                .map(|f| RecordRowField {
                    key: f.key,
                    ty: materialize_view(store, f.ty, env),
                    presence: f.presence,
                })
                .collect();
            store
                .record_row_type_checked(subst_fields, tail)
                .expect("lazy type materialization must preserve canonical Record-row invariants")
        }
        TypeData::Callable(call) => {
            let params: Vec<CallableParameterType> = call
                .parameters
                .iter()
                .map(|p| CallableParameterType {
                    label: p.label.clone(),
                    ty: materialize_view(store, p.ty, env),
                    rest: p.rest,
                })
                .collect();
            let return_type = materialize_view(store, call.return_type, env);
            store.callable(CallableType {
                parameters: params.into_boxed_slice(),
                return_type,
            })
        }
        TypeData::Family(fid) => {
            let family = store.get_family(fid).clone();
            let subst_members: Vec<crate::types::family::FamilyMemberType> = family
                .members
                .iter()
                .map(|m| crate::types::family::FamilyMemberType {
                    operation: m.operation.clone(),
                    member_kind: m.member_kind,
                    ty: materialize_view(store, m.ty, env),
                })
                .collect();
            store.family_type(subst_members).unwrap_or(ty)
        }
        TypeData::Lambda(lambda_id) => {
            let substitution = env.to_substitution();
            if substitution.is_empty() {
                ty
            } else {
                store.substitute_type_lambda(lambda_id, &substitution)
            }
        }
        TypeData::TypePack(elements) => {
            let subst_elements: Vec<TypeId> = elements.iter().map(|&elem| materialize_view(store, elem, env)).collect();
            store.type_pack(subst_elements.into_boxed_slice()).unwrap_or(ty)
        }
        TypeData::LabeledTypePack(elements) => {
            let subst_elements: Vec<crate::types::store::LabeledTypePackElement> = elements
                .iter()
                .map(|elem| crate::types::store::LabeledTypePackElement {
                    label: elem.label.clone(),
                    element: materialize_view(store, elem.element, env),
                })
                .collect();
            store.labeled_type_pack(subst_elements.into_boxed_slice()).unwrap_or(ty)
        }
        TypeData::Pack(pack) => {
            let subst_elements: Vec<super::store::PackTypeElement> = pack
                .elements
                .iter()
                .map(|elem| super::store::PackTypeElement {
                    label: elem.label.clone(),
                    ty: materialize_view(store, elem.ty, env),
                })
                .collect();
            let pos_rem = match pack.positional_remainder {
                super::store::AnonymousTupleRemainder::Typed(t) => super::store::AnonymousTupleRemainder::Typed(materialize_view(store, t, env)),
                rem => rem,
            };
            let lab_rem = match pack.labeled_remainder {
                super::store::AnonymousTupleRemainder::Typed(t) => super::store::AnonymousTupleRemainder::Typed(materialize_view(store, t, env)),
                rem => rem,
            };
            store.open_pack(subst_elements.into_boxed_slice(), pos_rem, lab_rem).unwrap_or(ty)
        }
        TypeData::PackFromPacks { positionals, labeled } => {
            let subst_pos = materialize_view(store, positionals, env);
            let subst_lab = materialize_view(store, labeled, env);
            store.pack_from_packs(subst_pos, subst_lab).unwrap_or(ty)
        }
        TypeData::TupleFromPacks { positionals, labeled } => {
            let subst_pos = materialize_view(store, positionals, env);
            let subst_lab = materialize_view(store, labeled, env);
            store.tuple_from_packs(subst_pos, subst_lab).unwrap_or(ty)
        }
        TypeData::CallableFromPacks {
            positionals,
            labeled,
            return_type,
        } => {
            let subst_pos = materialize_view(store, positionals, env);
            let subst_lab = materialize_view(store, labeled, env);
            let subst_ret = materialize_view(store, return_type, env);
            store.callable_from_packs(subst_pos, subst_lab, subst_ret).unwrap_or(ty)
        }
        TypeData::CompleteTypePack { positionals, labeled } => {
            let positionals = materialize_view(store, positionals, env);
            let labeled = materialize_view(store, labeled, env);
            store.complete_type_pack(positionals, labeled).unwrap_or(ty)
        }
        TypeData::TypePackConcat(segments) => {
            let segments = segments.iter().map(|&segment| materialize_view(store, segment, env)).collect::<Vec<_>>();
            store.type_pack_concat(&segments).unwrap_or(ty)
        }
        TypeData::LabeledTypePackConcat(segments) => {
            let segments = segments.iter().map(|&segment| materialize_view(store, segment, env)).collect::<Vec<_>>();
            store.labeled_type_pack_concat(&segments).unwrap_or(ty)
        }
        TypeData::StaticData(term) => {
            let arguments = term
                .arguments
                .iter()
                .map(|argument| match argument {
                    super::store::StaticArgument::Term(argument) => super::store::StaticArgument::Term(materialize_view(store, *argument, env)),
                    super::store::StaticArgument::RecordRow(row) => super::store::StaticArgument::RecordRow(*row),
                    super::store::StaticArgument::DynamicType => super::store::StaticArgument::DynamicType,
                })
                .collect::<Vec<_>>()
                .into_boxed_slice();
            store.static_data(term.domain, term.constructor, arguments).unwrap_or(ty)
        }
        TypeData::ConstInt(_) => ty,
        TypeData::Never | TypeData::Unit | TypeData::Nominal { .. } | TypeData::StaticDomainForm { .. } | TypeData::ClassObject { .. } => ty,
    }
}

/// A specialized view over a callable member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpecializedCallableView {
    pub callable: CallableId,
    pub environment: TypeEnvironment,
}

/// A specialized member view (method or field).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpecializedMemberView {
    pub callable: Option<CallableId>,
    pub environment: TypeEnvironment,
}

#[cfg(test)]
mod tests {
    use super::{FrozenTypeEnvironmentMutation, TypeEnvironment};
    use crate::checker::analysis_map::AnalysisMap;
    use crate::checker::analysis_set::AnalysisSet;
    use crate::types::id::{RecordRowId, TypeId, TypeParameterId};
    use phalcom_type_meta::allocation::AllocationBudget;

    #[test]
    fn source_environment_bindings_preserve_single_argument_category() {
        let parameter = TypeParameterId(1);
        let mut environment = TypeEnvironment::new();
        environment.bind_param(parameter, TypeId(2)).unwrap();
        assert_eq!(environment.argument(parameter), Some(crate::types::store::StaticArgument::Term(TypeId(2))));
        environment.bind_dynamic(parameter).unwrap();
        assert_eq!(environment.argument(parameter), Some(crate::types::store::StaticArgument::DynamicType));
        environment.bind_row(parameter, RecordRowId(3)).unwrap();
        assert_eq!(
            environment.argument(parameter),
            Some(crate::types::store::StaticArgument::RecordRow(RecordRowId(3)))
        );
    }

    #[test]
    fn a_frozen_nested_environment_collection_rejects_binding_without_partial_mutation() {
        let parameter = TypeParameterId(1);
        let mut allocations = AllocationBudget::new(4096);
        let mut entries = allocations.vector(1).unwrap();
        entries.push(parameter);
        let dynamic_bindings = AnalysisSet::from_budgeted_entries(entries, &mut allocations).unwrap();
        let mut environment = TypeEnvironment {
            bindings: AnalysisMap::from([(parameter, TypeId(2))]),
            dynamic_bindings,
            row_bindings: AnalysisMap::new(),
            self_binding: None,
        };

        assert_eq!(environment.bind_dynamic(parameter), Err(FrozenTypeEnvironmentMutation::DynamicBindings));
        assert_eq!(environment.argument(parameter), Some(crate::types::store::StaticArgument::Term(TypeId(2))));
        assert_eq!(environment.bind_self(TypeId(4)), Err(FrozenTypeEnvironmentMutation::DynamicBindings));
        assert_eq!(environment.get_self(), None);
    }
}
