//! Type parameter substitution for generic declarations and applied member views.

use super::id::{KindId, TypeId, TypeParameterId};
use super::row::{RecordRowField, RecordRowTail};
use super::application::TypeApplicationError;
use super::store::{CallableParameterType, CallableType, PackFormationError, StaticDataFormationError, TupleTypeElement, TypeData, TypeStore};
use super::type_lambda::TypeLambdaArena;
use crate::declarations::DeclarationTypeTable;
use std::collections::HashMap;

/// A map of type parameter replacements (`T -> Type`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TypeSubstitution {
    bindings: HashMap<TypeParameterId, TypeId>,
}

/// Errors made explicit when C2 pack terms are re-canonicalized after substitution.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TypeSubstitutionError {
    #[error("pack normalization failed after substitution: {0}")]
    Pack(#[from] PackFormationError),
    #[error("heterogeneous application failed after substitution: {0}")]
    Application(#[from] TypeApplicationError),
    #[error("static data formation failed after substitution: {0}")]
    StaticData(#[from] StaticDataFormationError),
}

impl TypeSubstitution {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bindings(bindings: HashMap<TypeParameterId, TypeId>) -> Self {
        Self { bindings }
    }

    pub fn bindings(&self) -> &HashMap<TypeParameterId, TypeId> {
        &self.bindings
    }

    pub fn bind(&mut self, param: TypeParameterId, ty: TypeId) {
        self.bindings.insert(param, ty);
    }

    pub fn get(&self, param: TypeParameterId) -> Option<TypeId> {
        self.bindings.get(&param).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Applies substitution through C2 pack composition terms without hiding a
    /// duplicate label that becomes concrete during re-canonicalization.
    pub fn try_apply(&self, store: &mut TypeStore, ty: TypeId) -> Result<TypeId, TypeSubstitutionError> {
        if self.is_empty() {
            return Ok(ty);
        }
        store.with_active_lambda_arena(|store, arena| self.try_apply_with_arena(store, arena, ty))
    }

    pub(crate) fn try_apply_with_arena(
        &self,
        store: &mut TypeStore,
        arena: &mut TypeLambdaArena,
        ty: TypeId,
    ) -> Result<TypeId, TypeSubstitutionError> {
        if self.is_empty() {
            return Ok(ty);
        }
        match store.get(ty).clone() {
            TypeData::Parameter(parameter) => Ok(self.bindings.get(&parameter).copied().unwrap_or(ty)),
            TypeData::CompleteTypePack { positionals, labeled } => {
                let positionals = self.try_apply_with_arena(store, arena, positionals)?;
                let labeled = self.try_apply_with_arena(store, arena, labeled)?;
                Ok(store.complete_type_pack(positionals, labeled)?)
            }
            TypeData::TypePackConcat(segments) => {
                let segments = segments.iter().map(|&segment| self.try_apply_with_arena(store, arena, segment)).collect::<Result<Vec<_>, _>>()?;
                Ok(store.type_pack_concat(&segments)?)
            }
            TypeData::LabeledTypePackConcat(segments) => {
                let segments = segments.iter().map(|&segment| self.try_apply_with_arena(store, arena, segment)).collect::<Result<Vec<_>, _>>()?;
                Ok(store.labeled_type_pack_concat(&segments)?)
            }
            TypeData::StaticData(term) => {
                let arguments = term
                    .arguments
                    .iter()
                    .map(|argument| match argument {
                        super::store::StaticArgument::Term(argument) => {
                            Ok(super::store::StaticArgument::Term(self.try_apply_with_arena(store, arena, *argument)?))
                        }
                        super::store::StaticArgument::RecordRow(row) => Ok(super::store::StaticArgument::RecordRow(*row)),
                    })
                    .collect::<Result<Vec<_>, TypeSubstitutionError>>()?
                    .into_boxed_slice();
                Ok(store.static_data(term.domain, term.constructor, arguments)?)
            }
            TypeData::ConstInt(_) => Ok(ty),
            TypeData::TupleFromPacks { positionals, labeled } => {
                let positionals = self.try_apply_with_arena(store, arena, positionals)?;
                let labeled = self.try_apply_with_arena(store, arena, labeled)?;
                Ok(store.tuple_from_packs(positionals, labeled)?)
            }
            TypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                let positionals = self.try_apply_with_arena(store, arena, positionals)?;
                let labeled = self.try_apply_with_arena(store, arena, labeled)?;
                let return_type = self.try_apply_with_arena(store, arena, return_type)?;
                Ok(store.callable_from_packs(positionals, labeled, return_type)?)
            }
            TypeData::Applied { origin, arguments } => {
                let subst_origin = self.try_apply_with_arena(store, arena, origin)?;
                let subst_args = arguments
                    .iter()
                    .map(|argument| match argument {
                        super::store::StaticArgument::Term(argument) => Ok(super::store::StaticArgument::Term(self.try_apply_with_arena(store, arena, *argument)?)),
                        super::store::StaticArgument::RecordRow(row) => Ok(super::store::StaticArgument::RecordRow(self.apply_record_row_with_arena(store, arena, *row))),
                    })
                    .collect::<Result<Vec<_>, TypeSubstitutionError>>()?;
                Ok(store.apply_static_form_with_arena(arena, subst_origin, &subst_args)?)
            }
            TypeData::Tuple(elements) => {
                let subst_elements = elements
                    .iter()
                    .map(|elem| {
                        Ok(TupleTypeElement {
                            label: elem.label.clone(),
                            ty: self.try_apply_with_arena(store, arena, elem.ty)?,
                        })
                    })
                    .collect::<Result<Vec<_>, TypeSubstitutionError>>()?;
                Ok(store.tuple(subst_elements.into_boxed_slice()))
            }
            TypeData::Callable(callable) => {
                let params = callable
                    .parameters
                    .iter()
                    .map(|p| {
                        Ok(CallableParameterType {
                            label: p.label.clone(),
                            ty: self.try_apply_with_arena(store, arena, p.ty)?,
                            rest: p.rest,
                        })
                    })
                    .collect::<Result<Vec<_>, TypeSubstitutionError>>()?;
                let return_type = self.try_apply_with_arena(store, arena, callable.return_type)?;
                Ok(store.callable(CallableType {
                    parameters: params.into_boxed_slice(),
                    return_type,
                }))
            }
            TypeData::TypeMemberProjection(projection)
                if projection.arguments.is_empty() && matches!(projection.subject, super::type_member::ProjectionSubject::Type(_)) =>
            {
                let super::type_member::ProjectionSubject::Type(subject) = projection.subject else {
                    unreachable!()
                };
                let subject = self.try_apply_with_arena(store, arena, subject)?;
                match projection.selector {
                    super::type_member::TypeMemberSelector::Named(name) if name.as_ref() == "Positionals" => Ok(store.project_positionals(subject)?),
                    super::type_member::TypeMemberSelector::Named(name) if name.as_ref() == "Labeled" => Ok(store.project_labeled(subject)?),
                    _ => Ok(self.apply_with_arena(store, arena, ty)),
                }
            }
            _ => Ok(self.apply_with_arena(store, arena, ty)),
        }
    }

    /// Recursively applies this substitution to a `RecordRowId`.
    pub fn apply_record_row(&self, store: &mut TypeStore, row_id: crate::types::id::RecordRowId) -> crate::types::id::RecordRowId {
        if self.is_empty() {
            return row_id;
        }
        store.with_active_lambda_arena(|store, arena| self.apply_record_row_with_arena(store, arena, row_id))
    }

    fn apply_record_row_with_arena(
        &self,
        store: &mut TypeStore,
        arena: &mut TypeLambdaArena,
        row_id: crate::types::id::RecordRowId,
    ) -> crate::types::id::RecordRowId {
        if self.is_empty() {
            return row_id;
        }
        let (fields, tail) = {
            let row = store.record_row(row_id);
            (row.fields.to_vec(), row.tail)
        };
        let subst_fields: Vec<RecordRowField> = fields
            .into_iter()
            .map(|field| RecordRowField {
                name: field.name,
                ty: self.apply_with_arena(store, arena, field.ty),
            })
            .collect();
        let subst_tail = match tail {
            RecordRowTail::Closed => RecordRowTail::Closed,
            RecordRowTail::Parameter(param) => RecordRowTail::Parameter(param),
            RecordRowTail::Projection(proj) => RecordRowTail::Projection(self.apply_with_arena(store, arena, proj)),
        };
        store
            .record_row_checked(subst_fields, subst_tail)
            .expect("type substitution must preserve canonical Record-row invariants")
    }

    /// Recursively applies this substitution to `ty`.
    pub fn apply(&self, store: &mut TypeStore, ty: TypeId) -> TypeId {
        if self.is_empty() {
            return ty;
        }
        store.with_active_lambda_arena(|store, arena| self.apply_with_arena(store, arena, ty))
    }

    pub(crate) fn apply_with_arena(&self, store: &mut TypeStore, arena: &mut TypeLambdaArena, ty: TypeId) -> TypeId {
        if self.is_empty() {
            return ty;
        }

        match store.get(ty).clone() {
            TypeData::Parameter(param_id) => {
                if let Some(&replacement) = self.bindings.get(&param_id) {
                    replacement
                } else {
                    ty
                }
            }
            TypeData::Applied { origin, arguments } => {
                let subst_origin = self.apply_with_arena(store, arena, origin);
                let subst_args = arguments
                    .iter()
                    .map(|argument| match argument {
                        super::store::StaticArgument::Term(argument) => super::store::StaticArgument::Term(self.apply_with_arena(store, arena, *argument)),
                        super::store::StaticArgument::RecordRow(row) => super::store::StaticArgument::RecordRow(self.apply_record_row_with_arena(store, arena, *row)),
                    })
                    .collect::<Vec<_>>();
                store.apply_static_form_with_arena(arena, subst_origin, &subst_args).unwrap_or(ty)
            }
            TypeData::ExactCase { variant, enum_type } => {
                let subst_enum = self.apply_with_arena(store, arena, enum_type);
                let variant_id = store.variant_identity(variant).clone();
                store.exact_case_type(&variant_id, subst_enum).unwrap_or(ty)
            }
            TypeData::Union(members) => {
                let subst_members: Vec<TypeId> = members.iter().map(|&m| self.apply_with_arena(store, arena, m)).collect();
                store.union(&subst_members)
            }
            TypeData::Tuple(elements) => {
                let subst_elements: Vec<TupleTypeElement> = elements
                    .iter()
                    .map(|elem| TupleTypeElement {
                        label: elem.label.clone(),
                        ty: self.apply_with_arena(store, arena, elem.ty),
                    })
                    .collect();
                store.tuple(subst_elements.into_boxed_slice())
            }
            TypeData::Record(row_id) => {
                let subst_row = self.apply_record_row_with_arena(store, arena, row_id);
                store.intern_with_kind(TypeData::Record(subst_row), KindId::TYPE)
            }
            TypeData::Callable(callable) => {
                let params: Vec<CallableParameterType> = callable
                    .parameters
                    .iter()
                    .map(|p| CallableParameterType {
                        label: p.label.clone(),
                        ty: self.apply_with_arena(store, arena, p.ty),
                        rest: p.rest,
                    })
                    .collect();
                let return_type = self.apply_with_arena(store, arena, callable.return_type);
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
                        ty: self.apply_with_arena(store, arena, m.ty),
                    })
                    .collect();
                store.family_type(subst_members).unwrap_or(ty)
            }
            TypeData::SelfType(_) => ty,
            TypeData::AssociatedProjection(projection) => {
                let subject = self.apply_with_arena(store, arena, projection.subject);
                let arguments = projection
                    .trait_ref
                    .arguments
                    .iter()
                    .map(|&argument| self.apply_with_arena(store, arena, argument))
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
                    super::type_member::ProjectionSubject::Type(sub_ty) => super::type_member::ProjectionSubject::Type(self.apply_with_arena(store, arena, *sub_ty)),
                    super::type_member::ProjectionSubject::RecordRow(row_id) => {
                        let subst_row = self.apply_record_row_with_arena(store, arena, *row_id);
                        super::type_member::ProjectionSubject::RecordRow(subst_row)
                    }
                };
                let arguments = projection
                    .arguments
                    .iter()
                    .map(|arg| match arg {
                        super::type_member::TypeMemberArgument::Type(arg_ty) => super::type_member::TypeMemberArgument::Type(self.apply_with_arena(store, arena, *arg_ty)),
                        super::type_member::TypeMemberArgument::Const(c) => super::type_member::TypeMemberArgument::Const(c.clone()),
                        super::type_member::TypeMemberArgument::RecordRow(row_id) => {
                            let subst_row = self.apply_record_row_with_arena(store, arena, *row_id);
                            super::type_member::TypeMemberArgument::RecordRow(subst_row)
                        }
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
            TypeData::Lambda(lambda_id) => {
                if self.is_empty() {
                    ty
                } else {
                    let specialized = arena.substitute_free_types(lambda_id, self, store);
                    let lambda = arena.get_lambda(specialized).clone();
                    let kind = if lambda.parameter_kinds.is_empty() {
                        lambda.result_kind
                    } else {
                        store.arrow_kind(lambda.parameter_kinds, lambda.result_kind)
                    };
                    store.intern_with_kind(TypeData::Lambda(specialized), kind)
                }
            }
            TypeData::TypePack(elements) => {
                let subst_elements: Vec<TypeId> = elements.iter().map(|&elem| self.apply_with_arena(store, arena, elem)).collect();
                store.type_pack(subst_elements.into_boxed_slice()).unwrap_or(ty)
            }
            TypeData::LabeledTypePack(elements) => {
                let subst_elements: Vec<crate::types::store::LabeledTypePackElement> = elements
                    .iter()
                    .map(|elem| crate::types::store::LabeledTypePackElement {
                        label: elem.label.clone(),
                        element: self.apply_with_arena(store, arena, elem.element),
                    })
                    .collect();
                store.labeled_type_pack(subst_elements.into_boxed_slice()).unwrap_or(ty)
            }
            TypeData::TupleFromPacks { positionals, labeled } => {
                let subst_pos = self.apply_with_arena(store, arena, positionals);
                let subst_lab = self.apply_with_arena(store, arena, labeled);
                store.tuple_from_packs(subst_pos, subst_lab).unwrap_or(ty)
            }
            TypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                let subst_pos = self.apply_with_arena(store, arena, positionals);
                let subst_lab = self.apply_with_arena(store, arena, labeled);
                let subst_ret = self.apply_with_arena(store, arena, return_type);
                store.callable_from_packs(subst_pos, subst_lab, subst_ret).unwrap_or(ty)
            }
            TypeData::CompleteTypePack { .. } | TypeData::TypePackConcat(_) | TypeData::LabeledTypePackConcat(_) => {
                // C2 source forms must use `try_apply`; legacy callers retain
                // their existing infallible precondition until P2 migrates them.
                self.try_apply_with_arena(store, arena, ty).unwrap_or(ty)
            }
            TypeData::StaticData(term) => {
                let arguments = term
                    .arguments
                    .iter()
                    .map(|argument| match argument {
                        super::store::StaticArgument::Term(argument) => {
                            super::store::StaticArgument::Term(self.apply_with_arena(store, arena, *argument))
                        }
                        super::store::StaticArgument::RecordRow(row) => super::store::StaticArgument::RecordRow(*row),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                store.static_data(term.domain, term.constructor, arguments).unwrap_or(ty)
            }
            TypeData::ConstInt(_) => ty,
            TypeData::Never | TypeData::Unit | TypeData::Nominal { .. } | TypeData::StaticDomainForm { .. } | TypeData::ClassObject { .. } => ty,
        }
    }
}

/// Builds a substitution from declaration generic signature and applied type arguments.
pub fn substitution_for_applied(declarations: &DeclarationTypeTable, store: &TypeStore, mut applied: TypeId) -> Option<TypeSubstitution> {
    while let TypeData::ExactCase { enum_type, .. } = store.get(applied) {
        applied = *enum_type;
    }

    let (origin, arguments) = match store.get(applied) {
        TypeData::Applied { origin, arguments } => (*origin, arguments),
        _ => return None,
    };

    let mut base = origin;
    while let TypeData::Applied { origin: next, .. } = store.get(base) {
        base = *next;
    }

    if let TypeData::Nominal { declaration } = store.get(base) {
        if let Some(info) = declarations.get(declaration) {
            if let Some(sig) = &info.generic_signature {
                let mut subst = TypeSubstitution::new();
                for (&param_id, argument) in sig.parameters.iter().zip(arguments.iter()) {
                    if let Some(arg_ty) = argument.as_type_id() {
                        subst.bind(param_id, arg_ty);
                    }
                }
                return Some(subst);
            }
        }
    }

    None
}

/// Specializes `Self` type terms within `ty` based on the concrete `receiver` type.
pub fn specialize_self_type(store: &mut TypeStore, declarations: &DeclarationTypeTable, receiver: TypeId, ty: TypeId) -> TypeId {
    match store.get(ty).clone() {
        TypeData::SelfType(term) => match term.role {
            crate::types::parameter::SelfRole::ReceiverValue => receiver,
            crate::types::parameter::SelfRole::InstanceType => match store.get(receiver).clone() {
                TypeData::ClassObject { declaration } => {
                    let Some(form) = declarations.form(&declaration) else { return ty };
                    let Some(signature) = declarations.generic_signature(&declaration) else {
                        return form;
                    };
                    let arguments = signature
                        .parameters
                        .iter()
                        .map(|&parameter| store.parameter_form(parameter))
                        .collect::<Vec<_>>();
                    store.apply_type_form(form, &arguments).unwrap_or(form)
                }
                TypeData::Nominal { .. } | TypeData::Applied { .. } => receiver,
                _ => match declarations.form(&term.owner) {
                    Some(form) => form,
                    None => ty,
                },
            },
        },
        TypeData::Applied { origin, arguments } => {
            let subst_origin = specialize_self_type(store, declarations, receiver, origin);
            let subst_args = arguments
                .iter()
                .map(|argument| match argument {
                    super::store::StaticArgument::Term(argument) => {
                        super::store::StaticArgument::Term(specialize_self_type(store, declarations, receiver, *argument))
                    }
                    super::store::StaticArgument::RecordRow(row) => super::store::StaticArgument::RecordRow(*row),
                })
                .collect::<Vec<_>>();
            store.apply_static_form(subst_origin, &subst_args).unwrap_or(ty)
        }
        TypeData::ExactCase { variant, enum_type } => {
            let subst_enum = specialize_self_type(store, declarations, receiver, enum_type);
            let variant_id = store.variant_identity(variant).clone();
            store.exact_case_type(&variant_id, subst_enum).unwrap_or(ty)
        }
        TypeData::AssociatedProjection(projection) => {
            let subject = specialize_self_type(store, declarations, receiver, projection.subject);
            let arguments = projection
                .trait_ref
                .arguments
                .iter()
                .map(|&argument| specialize_self_type(store, declarations, receiver, argument))
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
                super::type_member::ProjectionSubject::Type(sub_ty) => {
                    super::type_member::ProjectionSubject::Type(specialize_self_type(store, declarations, receiver, *sub_ty))
                }
                super::type_member::ProjectionSubject::RecordRow(row_id) => super::type_member::ProjectionSubject::RecordRow(*row_id),
            };
            let arguments = projection
                .arguments
                .iter()
                .map(|arg| match arg {
                    super::type_member::TypeMemberArgument::Type(arg_ty) => {
                        super::type_member::TypeMemberArgument::Type(specialize_self_type(store, declarations, receiver, *arg_ty))
                    }
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
        TypeData::Union(members) => {
            let subst_members: Vec<TypeId> = members.iter().map(|&m| specialize_self_type(store, declarations, receiver, m)).collect();
            store.union(&subst_members)
        }
        TypeData::Tuple(elements) => {
            let subst_elements: Vec<TupleTypeElement> = elements
                .iter()
                .map(|elem| TupleTypeElement {
                    label: elem.label.clone(),
                    ty: specialize_self_type(store, declarations, receiver, elem.ty),
                })
                .collect();
            store.tuple(subst_elements.into_boxed_slice())
        }
        TypeData::Record(row_id) => {
            let (fields, tail) = {
                let row = store.record_row(row_id);
                (row.fields.to_vec(), row.tail)
            };
            let subst_fields: Vec<RecordRowField> = fields
                .into_iter()
                .map(|field| RecordRowField {
                    name: field.name,
                    ty: specialize_self_type(store, declarations, receiver, field.ty),
                })
                .collect();
            store
                .record_row_type_checked(subst_fields, tail)
                .expect("Self specialization must preserve canonical Record-row invariants")
        }
        TypeData::Callable(callable) => {
            let params: Vec<CallableParameterType> = callable
                .parameters
                .iter()
                .map(|p| CallableParameterType {
                    label: p.label.clone(),
                    ty: specialize_self_type(store, declarations, receiver, p.ty),
                    rest: p.rest,
                })
                .collect();
            let return_type = specialize_self_type(store, declarations, receiver, callable.return_type);
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
                    ty: specialize_self_type(store, declarations, receiver, m.ty),
                })
                .collect();
            store.family_type(subst_members).unwrap_or(ty)
        }
        TypeData::TypePack(elements) => {
            let subst_elements: Vec<TypeId> = elements.iter().map(|&elem| specialize_self_type(store, declarations, receiver, elem)).collect();
            store.type_pack(subst_elements.into_boxed_slice()).unwrap_or(ty)
        }
        TypeData::LabeledTypePack(elements) => {
            let subst_elements: Vec<crate::types::store::LabeledTypePackElement> = elements
                .iter()
                .map(|elem| crate::types::store::LabeledTypePackElement {
                    label: elem.label.clone(),
                    element: specialize_self_type(store, declarations, receiver, elem.element),
                })
                .collect();
            store.labeled_type_pack(subst_elements.into_boxed_slice()).unwrap_or(ty)
        }
        TypeData::TupleFromPacks { positionals, labeled } => {
            let subst_pos = specialize_self_type(store, declarations, receiver, positionals);
            let subst_lab = specialize_self_type(store, declarations, receiver, labeled);
            store.tuple_from_packs(subst_pos, subst_lab).unwrap_or(ty)
        }
        TypeData::CallableFromPacks {
            positionals,
            labeled,
            return_type,
        } => {
            let subst_pos = specialize_self_type(store, declarations, receiver, positionals);
            let subst_lab = specialize_self_type(store, declarations, receiver, labeled);
            let subst_ret = specialize_self_type(store, declarations, receiver, return_type);
            store.callable_from_packs(subst_pos, subst_lab, subst_ret).unwrap_or(ty)
        }
        TypeData::CompleteTypePack { positionals, labeled } => {
            let positionals = specialize_self_type(store, declarations, receiver, positionals);
            let labeled = specialize_self_type(store, declarations, receiver, labeled);
            store.complete_type_pack(positionals, labeled).unwrap_or(ty)
        }
        TypeData::TypePackConcat(segments) => {
            let segments = segments
                .iter()
                .map(|&segment| specialize_self_type(store, declarations, receiver, segment))
                .collect::<Vec<_>>();
            store.type_pack_concat(&segments).unwrap_or(ty)
        }
        TypeData::LabeledTypePackConcat(segments) => {
            let segments = segments
                .iter()
                .map(|&segment| specialize_self_type(store, declarations, receiver, segment))
                .collect::<Vec<_>>();
            store.labeled_type_pack_concat(&segments).unwrap_or(ty)
        }
        TypeData::StaticData(term) => {
            let arguments = term
                .arguments
                .iter()
                .map(|argument| match argument {
                    super::store::StaticArgument::Term(argument) => {
                        super::store::StaticArgument::Term(specialize_self_type(store, declarations, receiver, *argument))
                    }
                    super::store::StaticArgument::RecordRow(row) => super::store::StaticArgument::RecordRow(*row),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice();
            store.static_data(term.domain, term.constructor, arguments).unwrap_or(ty)
        }
        TypeData::ConstInt(_) => ty,
        TypeData::Parameter(_) | TypeData::Lambda(_) | TypeData::Never | TypeData::Unit | TypeData::Nominal { .. } | TypeData::StaticDomainForm { .. } | TypeData::ClassObject { .. } => ty,
    }
}
