//! Analysis-local rigid type variables.
//!
//! Rigid variables are deliberately absent from [`TypeStore`]. They exist only
//! while one semantic query is opening a constructor-local existential and can
//! therefore never become durable type metadata by accident.

use super::id::{KindId, RigidScopeId, RigidTypeVariableId, TypeId};
use super::row::RecordRowTail;
use super::store::{CallableParameterType, CallableType, RecordTypeField, TupleTypeElement, TypeData, TypeStore};
use crate::identity::VariantId;
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RigidOrigin {
    VariantParameter {
        variant: VariantId,
        parameter: super::id::TypeParameterId,
    },
    Synthetic,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RigidTypeVariable {
    pub id: RigidTypeVariableId,
    pub scope: RigidScopeId,
    pub kind: KindId,
    pub origin: RigidOrigin,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct RigidScope {
    parent: Option<RigidScopeId>,
}

/// Monotonic query-local rigid allocator and scope tree.
#[derive(Clone, Debug, Default)]
pub struct RigidArena {
    scopes: Vec<RigidScope>,
    variables: Vec<RigidTypeVariable>,
}

impl RigidArena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fresh_scope(&mut self, parent: Option<RigidScopeId>) -> RigidScopeId {
        let id = RigidScopeId::from_index(self.scopes.len());
        self.scopes.push(RigidScope { parent });
        id
    }

    pub fn fresh(&mut self, scope: RigidScopeId, kind: KindId, origin: RigidOrigin) -> RigidTypeVariableId {
        let id = RigidTypeVariableId::from_index(self.variables.len());
        self.variables.push(RigidTypeVariable { id, scope, kind, origin });
        id
    }

    pub fn variable(&self, id: RigidTypeVariableId) -> Option<&RigidTypeVariable> {
        self.variables.get(id.index())
    }

    pub fn scope_contains(&self, outer: RigidScopeId, inner: RigidScopeId) -> bool {
        let mut current = Some(inner);
        while let Some(scope) = current {
            if scope == outer {
                return true;
            }
            current = self.scopes.get(scope.index()).and_then(|scope| scope.parent);
        }
        false
    }

    pub fn variable_in_scope(&self, id: RigidTypeVariableId, scope: RigidScopeId) -> bool {
        self.variable(id).is_some_and(|variable| self.scope_contains(scope, variable.scope))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LocalTupleElement {
    pub label: Option<Box<str>>,
    pub ty: LocalType,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LocalRecordField {
    pub name: Box<str>,
    pub ty: LocalType,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LocalCallableParameter {
    pub label: Option<Box<str>>,
    pub ty: LocalType,
    pub rest: phalcom_ast::ast::RestMode,
}

/// A local type expression that may contain canonical types and scoped rigids.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum LocalType {
    Canonical(TypeId),
    Rigid(RigidTypeVariableId),
    Applied {
        origin: Box<LocalType>,
        arguments: Box<[LocalType]>,
    },
    ExactCase {
        variant: VariantId,
        enum_type: Box<LocalType>,
    },
    Union(Box<[LocalType]>),
    Tuple(Box<[LocalTupleElement]>),
    Record(Box<[LocalRecordField]>),
    Callable {
        parameters: Box<[LocalCallableParameter]>,
        return_type: Box<LocalType>,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum LocalConstraint {
    Subtype { lower: LocalType, upper: LocalType },
    Equivalent { left: LocalType, right: LocalType },
}

impl LocalType {
    pub fn rigid(id: RigidTypeVariableId) -> Self {
        Self::Rigid(id)
    }

    pub fn free_rigids(&self) -> BTreeSet<RigidTypeVariableId> {
        let mut result = BTreeSet::new();
        self.collect_free_rigids(&mut result);
        result
    }

    pub fn contains_rigid_from_scope(&self, arena: &RigidArena, scope: RigidScopeId) -> bool {
        self.free_rigids().iter().any(|id| arena.variable_in_scope(*id, scope))
    }

    pub fn collect_free_rigids(&self, result: &mut BTreeSet<RigidTypeVariableId>) {
        match self {
            Self::Rigid(id) => {
                result.insert(*id);
            }
            Self::Applied { origin, arguments } => {
                origin.collect_free_rigids(result);
                for argument in arguments.iter() {
                    argument.collect_free_rigids(result);
                }
            }
            Self::ExactCase { enum_type, .. } => enum_type.collect_free_rigids(result),
            Self::Union(members) => members.iter().for_each(|member| member.collect_free_rigids(result)),
            Self::Tuple(elements) => elements.iter().for_each(|element| element.ty.collect_free_rigids(result)),
            Self::Record(fields) => fields.iter().for_each(|field| field.ty.collect_free_rigids(result)),
            Self::Callable { parameters, return_type } => {
                parameters.iter().for_each(|parameter| parameter.ty.collect_free_rigids(result));
                return_type.collect_free_rigids(result);
            }
            Self::Canonical(_) => {}
        }
    }

    /// Compares local types modulo one-to-one rigid renaming.
    pub fn alpha_equivalent(&self, other: &Self) -> bool {
        fn compare(
            left: &LocalType,
            right: &LocalType,
            mapping: &mut BTreeMap<RigidTypeVariableId, RigidTypeVariableId>,
            reverse: &mut BTreeMap<RigidTypeVariableId, RigidTypeVariableId>,
        ) -> bool {
            match (left, right) {
                (LocalType::Canonical(left), LocalType::Canonical(right)) => left == right,
                (LocalType::Rigid(left), LocalType::Rigid(right)) => {
                    if let Some(mapped) = mapping.get(left) {
                        return mapped == right;
                    }
                    if reverse.contains_key(right) {
                        return false;
                    }
                    mapping.insert(*left, *right);
                    reverse.insert(*right, *left);
                    true
                }
                (
                    LocalType::Applied {
                        origin: left_origin,
                        arguments: left_arguments,
                    },
                    LocalType::Applied {
                        origin: right_origin,
                        arguments: right_arguments,
                    },
                ) => {
                    left_arguments.len() == right_arguments.len()
                        && compare(left_origin, right_origin, mapping, reverse)
                        && left_arguments
                            .iter()
                            .zip(right_arguments.iter())
                            .all(|(left, right)| compare(left, right, mapping, reverse))
                }
                (
                    LocalType::ExactCase {
                        variant: left_variant,
                        enum_type: left_enum,
                    },
                    LocalType::ExactCase {
                        variant: right_variant,
                        enum_type: right_enum,
                    },
                ) => left_variant == right_variant && compare(left_enum, right_enum, mapping, reverse),
                (LocalType::Union(left), LocalType::Union(right)) => {
                    left.len() == right.len() && left.iter().zip(right.iter()).all(|(left, right)| compare(left, right, mapping, reverse))
                }
                (LocalType::Tuple(left), LocalType::Tuple(right)) => {
                    left.len() == right.len()
                        && left
                            .iter()
                            .zip(right.iter())
                            .all(|(left, right)| left.label == right.label && compare(&left.ty, &right.ty, mapping, reverse))
                }
                (LocalType::Record(left), LocalType::Record(right)) => {
                    left.len() == right.len()
                        && left
                            .iter()
                            .zip(right.iter())
                            .all(|(left, right)| left.name == right.name && compare(&left.ty, &right.ty, mapping, reverse))
                }
                (
                    LocalType::Callable {
                        parameters: left_parameters,
                        return_type: left_return,
                    },
                    LocalType::Callable {
                        parameters: right_parameters,
                        return_type: right_return,
                    },
                ) => {
                    left_parameters.len() == right_parameters.len()
                        && left_parameters
                            .iter()
                            .zip(right_parameters.iter())
                            .all(|(left, right)| left.label == right.label && left.rest == right.rest && compare(&left.ty, &right.ty, mapping, reverse))
                        && compare(left_return, right_return, mapping, reverse)
                }
                _ => false,
            }
        }

        compare(self, other, &mut BTreeMap::new(), &mut BTreeMap::new())
    }

    pub fn materialize(&self, store: &mut TypeStore) -> Result<TypeId, RigidMaterializationError> {
        match self {
            Self::Canonical(ty) => Ok(*ty),
            Self::Rigid(id) => Err(RigidMaterializationError::ContainsRigid(*id)),
            Self::Applied { origin, arguments } => {
                let origin = origin.materialize(store)?;
                let arguments = arguments.iter().map(|argument| argument.materialize(store)).collect::<Result<Vec<_>, _>>()?;
                store
                    .apply_type_form(origin, &arguments)
                    .map_err(|_| RigidMaterializationError::InvalidApplication)
            }
            Self::ExactCase { variant, enum_type } => {
                let enum_type = enum_type.materialize(store)?;
                store
                    .exact_case_type(variant, enum_type)
                    .map_err(|_| RigidMaterializationError::InvalidApplication)
            }
            Self::Union(members) => {
                let members = members.iter().map(|member| member.materialize(store)).collect::<Result<Vec<_>, _>>()?;
                Ok(store.union(&members))
            }
            Self::Tuple(elements) => {
                let mut tuple = Vec::with_capacity(elements.len());
                for element in elements.iter() {
                    tuple.push(TupleTypeElement {
                        label: element.label.clone(),
                        ty: element.ty.materialize(store)?,
                    });
                }
                Ok(store.tuple(tuple.into_boxed_slice()))
            }
            Self::Record(fields) => {
                let mut record = Vec::with_capacity(fields.len());
                for field in fields.iter() {
                    record.push(RecordTypeField {
                        name: field.name.clone(),
                        ty: field.ty.materialize(store)?,
                    });
                }
                store
                    .record_row_type_checked(record, RecordRowTail::Closed)
                    .map_err(|_| RigidMaterializationError::InvalidApplication)
            }
            Self::Callable { parameters, return_type } => {
                let mut callable_parameters = Vec::with_capacity(parameters.len());
                for parameter in parameters.iter() {
                    callable_parameters.push(CallableParameterType {
                        label: parameter.label.clone(),
                        ty: parameter.ty.materialize(store)?,
                        rest: parameter.rest,
                    });
                }
                let return_type = return_type.materialize(store)?;
                Ok(store.callable(CallableType {
                    parameters: callable_parameters.into_boxed_slice(),
                    return_type,
                }))
            }
        }
    }

    /// Replaces selected canonical declaration parameters with local rigids.
    pub fn from_canonical(store: &TypeStore, ty: TypeId, replacements: &HashMap<super::id::TypeParameterId, LocalType>) -> Self {
        match store.get(ty).clone() {
            TypeData::Parameter(parameter) => replacements.get(&parameter).cloned().unwrap_or(Self::Canonical(ty)),
            TypeData::Applied { origin, arguments } => Self::Applied {
                origin: Box::new(Self::from_canonical(store, origin, replacements)),
                arguments: arguments.iter().map(|argument| Self::from_canonical(store, *argument, replacements)).collect(),
            },
            TypeData::ExactCase { variant, enum_type } => Self::ExactCase {
                variant: store.variant_identity(variant).clone(),
                enum_type: Box::new(Self::from_canonical(store, enum_type, replacements)),
            },
            TypeData::Union(members) => Self::Union(members.iter().map(|member| Self::from_canonical(store, *member, replacements)).collect()),
            TypeData::Tuple(elements) => Self::Tuple(
                elements
                    .iter()
                    .map(|element| LocalTupleElement {
                        label: element.label.clone(),
                        ty: Self::from_canonical(store, element.ty, replacements),
                    })
                    .collect(),
            ),
            TypeData::Record(row_id) => Self::Record(
                store
                    .record_row(row_id)
                    .fields
                    .iter()
                    .map(|field| LocalRecordField {
                        name: field.name.clone(),
                        ty: Self::from_canonical(store, field.ty, replacements),
                    })
                    .collect(),
            ),
            TypeData::Callable(callable) => Self::Callable {
                parameters: callable
                    .parameters
                    .iter()
                    .map(|parameter| LocalCallableParameter {
                        label: parameter.label.clone(),
                        ty: Self::from_canonical(store, parameter.ty, replacements),
                        rest: parameter.rest,
                    })
                    .collect(),
                return_type: Box::new(Self::from_canonical(store, callable.return_type, replacements)),
            },
            _ => Self::Canonical(ty),
        }
    }

    /// Replaces complete canonical subterms with local terms before descending
    /// through the canonical representation. This keeps branch-local types
    /// outside `TypeStore` while preserving them through composites.
    pub fn from_canonical_types(store: &TypeStore, ty: TypeId, replacements: &HashMap<TypeId, LocalType>) -> Self {
        if let Some(replacement) = replacements.get(&ty) {
            return replacement.clone();
        }
        match store.get(ty).clone() {
            TypeData::Applied { origin, arguments } => Self::Applied {
                origin: Box::new(Self::from_canonical_types(store, origin, replacements)),
                arguments: arguments
                    .iter()
                    .map(|argument| Self::from_canonical_types(store, *argument, replacements))
                    .collect(),
            },
            TypeData::ExactCase { variant, enum_type } => Self::ExactCase {
                variant: store.variant_identity(variant).clone(),
                enum_type: Box::new(Self::from_canonical_types(store, enum_type, replacements)),
            },
            TypeData::Union(members) => Self::Union(
                members
                    .iter()
                    .map(|member| Self::from_canonical_types(store, *member, replacements))
                    .collect(),
            ),
            TypeData::Tuple(elements) => Self::Tuple(
                elements
                    .iter()
                    .map(|element| LocalTupleElement {
                        label: element.label.clone(),
                        ty: Self::from_canonical_types(store, element.ty, replacements),
                    })
                    .collect(),
            ),
            TypeData::Record(row_id) => Self::Record(
                store
                    .record_row(row_id)
                    .fields
                    .iter()
                    .map(|field| LocalRecordField {
                        name: field.name.clone(),
                        ty: Self::from_canonical_types(store, field.ty, replacements),
                    })
                    .collect(),
            ),
            TypeData::Callable(callable) => Self::Callable {
                parameters: callable
                    .parameters
                    .iter()
                    .map(|parameter| LocalCallableParameter {
                        label: parameter.label.clone(),
                        ty: Self::from_canonical_types(store, parameter.ty, replacements),
                        rest: parameter.rest,
                    })
                    .collect(),
                return_type: Box::new(Self::from_canonical_types(store, callable.return_type, replacements)),
            },
            _ => Self::Canonical(ty),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RigidMaterializationError {
    ContainsRigid(RigidTypeVariableId),
    InvalidApplication,
}

/// A branch-local substitution from constructor binders to rigid types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RigidSubstitution {
    pub scope: RigidScopeId,
    pub bindings: BTreeMap<super::id::TypeParameterId, RigidTypeVariableId>,
}

impl RigidSubstitution {
    pub fn new(scope: RigidScopeId) -> Self {
        Self {
            scope,
            bindings: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, parameter: super::id::TypeParameterId, rigid: RigidTypeVariableId) {
        self.bindings.insert(parameter, rigid);
    }

    pub fn apply(&self, store: &TypeStore, ty: TypeId) -> LocalType {
        let replacements = self
            .bindings
            .iter()
            .map(|(parameter, rigid)| (*parameter, LocalType::Rigid(*rigid)))
            .collect::<HashMap<_, _>>();
        LocalType::from_canonical(store, ty, &replacements)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::id::KindId;
    use crate::types::store::TypeStore;

    #[test]
    fn rigid_scopes_and_kinds_are_stable_inside_one_arena() {
        let mut arena = RigidArena::new();
        let outer = arena.fresh_scope(None);
        let inner = arena.fresh_scope(Some(outer));
        let rigid = arena.fresh(inner, KindId::TYPE, RigidOrigin::Synthetic);
        assert_eq!(arena.variable(rigid).expect("rigid metadata").kind, KindId::TYPE);
        assert!(arena.variable_in_scope(rigid, outer));
        assert!(arena.variable_in_scope(rigid, inner));
    }

    #[test]
    fn composite_free_rigid_walk_and_alpha_equivalence_ignore_raw_ids() {
        let mut arena = RigidArena::new();
        let left_scope = arena.fresh_scope(None);
        let right_scope = arena.fresh_scope(None);
        let left = arena.fresh(left_scope, KindId::TYPE, RigidOrigin::Synthetic);
        let right = arena.fresh(right_scope, KindId::TYPE, RigidOrigin::Synthetic);
        let list = TypeId(0);
        let left_type = LocalType::Applied {
            origin: Box::new(LocalType::Canonical(list)),
            arguments: Box::new([LocalType::Rigid(left)]),
        };
        let right_type = LocalType::Applied {
            origin: Box::new(LocalType::Canonical(list)),
            arguments: Box::new([LocalType::Rigid(right)]),
        };
        assert_eq!(left_type.free_rigids().into_iter().collect::<Vec<_>>(), vec![left]);
        assert!(left_type.alpha_equivalent(&right_type));
    }

    #[test]
    fn rigid_materialization_is_a_hard_publication_barrier() {
        let mut arena = RigidArena::new();
        let scope = arena.fresh_scope(None);
        let rigid = arena.fresh(scope, KindId::TYPE, RigidOrigin::Synthetic);
        let mut store = TypeStore::new();
        assert_eq!(
            LocalType::Rigid(rigid).materialize(&mut store),
            Err(RigidMaterializationError::ContainsRigid(rigid))
        );
    }
}
