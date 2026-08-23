//! Alpha-normalized Type Lambda calculus and scoped bound representation.

use super::id::{KindId, ScopedTypeId, TypeId, TypeLambdaId};
use super::store::TypeStore;
use crate::diagnostic::SemanticSourceSpan;
use std::collections::HashMap;

/// Representation of scoped types inside a type lambda body arena.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ScopedTypeData {
    /// Lambda-bound variable; de Bruijn depth and index within the binder layer.
    Bound { depth: u32, index: u32 },
    /// A canonical free type from the enclosing semantic store.
    Free(TypeId),
    /// Application of a scoped type constructor to scoped arguments.
    Applied { origin: ScopedTypeId, arguments: Box<[ScopedTypeId]> },
    /// Flat union of scoped types.
    Union(Box<[ScopedTypeId]>),
    /// Scoped tuple type.
    Tuple(Box<[ScopedTupleElement]>),
    /// Scoped record type.
    Record(Box<[ScopedRecordField]>),
    /// Scoped callable type.
    Callable(ScopedCallableType),
    /// Nested lambda reference.
    Lambda(TypeLambdaId),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ScopedTupleElement {
    pub label: Option<Box<str>>,
    pub ty: ScopedTypeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ScopedRecordField {
    pub name: Box<str>,
    pub ty: ScopedTypeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ScopedCallableParameter {
    pub label: Option<Box<str>>,
    pub ty: ScopedTypeId,
    pub rest: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ScopedCallableType {
    pub parameters: Box<[ScopedCallableParameter]>,
    pub return_type: ScopedTypeId,
}

/// Canonical alpha-normalized type lambda definition.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeLambdaData {
    pub parameter_kinds: Box<[KindId]>,
    pub body: ScopedTypeId,
    pub result_kind: KindId,
}

/// Source provenance for type lambda presentation (excluded from semantic equality).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TypeLambdaProvenance {
    pub parameter_names: Box<[Box<str>]>,
    pub parameter_sources: Box<[SemanticSourceSpan]>,
    pub lambda_source: Option<SemanticSourceSpan>,
}

/// Arena managing scoped types and alpha-normalized type lambdas.
#[derive(Clone, Debug, Default)]
pub struct TypeLambdaArena {
    scoped_types: Vec<ScopedTypeData>,
    scoped_to_id: HashMap<ScopedTypeData, ScopedTypeId>,
    lambdas: Vec<TypeLambdaData>,
    lambda_to_id: HashMap<TypeLambdaData, TypeLambdaId>,
    provenance: Vec<TypeLambdaProvenance>,
}

impl TypeLambdaArena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern_scoped(&mut self, data: ScopedTypeData) -> ScopedTypeId {
        if let Some(&id) = self.scoped_to_id.get(&data) {
            return id;
        }
        let id = ScopedTypeId::from_index(self.scoped_types.len());
        self.scoped_types.push(data.clone());
        self.scoped_to_id.insert(data, id);
        id
    }

    pub fn get_scoped(&self, id: ScopedTypeId) -> &ScopedTypeData {
        &self.scoped_types[id.index()]
    }

    pub fn intern_lambda(
        &mut self,
        parameter_kinds: Box<[KindId]>,
        body: ScopedTypeId,
        result_kind: KindId,
        prov: Option<TypeLambdaProvenance>,
    ) -> TypeLambdaId {
        let data = TypeLambdaData {
            parameter_kinds,
            body,
            result_kind,
        };
        if let Some(&id) = self.lambda_to_id.get(&data) {
            return id;
        }
        let id = TypeLambdaId::from_index(self.lambdas.len());
        self.lambdas.push(data.clone());
        self.lambda_to_id.insert(data, id);
        self.provenance.push(prov.unwrap_or_default());
        id
    }

    pub fn get_lambda(&self, id: TypeLambdaId) -> &TypeLambdaData {
        &self.lambdas[id.index()]
    }

    pub fn get_provenance(&self, id: TypeLambdaId) -> Option<&TypeLambdaProvenance> {
        self.provenance.get(id.index())
    }

    /// Computes the full kind of a type lambda: `Kind(P0) -> Kind(P1) -> ... -> ResultKind`.
    pub fn lambda_kind(&self, id: TypeLambdaId, store: &mut TypeStore) -> KindId {
        let lambda = self.get_lambda(id);
        if lambda.parameter_kinds.is_empty() {
            lambda.result_kind
        } else {
            store.arrow_kind(lambda.parameter_kinds.clone(), lambda.result_kind)
        }
    }

    /// Checks if a scoped type has any free bound variables at or above the given depth.
    pub fn has_free_bound(&self, scoped: ScopedTypeId, current_depth: u32) -> bool {
        match self.get_scoped(scoped) {
            ScopedTypeData::Bound { depth, .. } => *depth >= current_depth,
            ScopedTypeData::Free(_) => false,
            ScopedTypeData::Applied { origin, arguments } => {
                self.has_free_bound(*origin, current_depth) || arguments.iter().any(|&a| self.has_free_bound(a, current_depth))
            }
            ScopedTypeData::Union(members) => members.iter().any(|&m| self.has_free_bound(m, current_depth)),
            ScopedTypeData::Tuple(elems) => elems.iter().any(|e| self.has_free_bound(e.ty, current_depth)),
            ScopedTypeData::Record(fields) => fields.iter().any(|f| self.has_free_bound(f.ty, current_depth)),
            ScopedTypeData::Callable(call) => {
                call.parameters.iter().any(|p| self.has_free_bound(p.ty, current_depth)) || self.has_free_bound(call.return_type, current_depth)
            }
            ScopedTypeData::Lambda(lid) => {
                let inner = self.get_lambda(*lid);
                self.has_free_bound(inner.body, current_depth + 1)
            }
        }
    }

    /// Collects all canonical free `TypeId`s referenced in a lambda body.
    pub fn collect_free_types(&self, scoped: ScopedTypeId, out: &mut Vec<TypeId>) {
        match self.get_scoped(scoped) {
            ScopedTypeData::Bound { .. } => {}
            ScopedTypeData::Free(ty) => {
                if !out.contains(ty) {
                    out.push(*ty);
                }
            }
            ScopedTypeData::Applied { origin, arguments } => {
                self.collect_free_types(*origin, out);
                for &arg in arguments.iter() {
                    self.collect_free_types(arg, out);
                }
            }
            ScopedTypeData::Union(members) => {
                for &m in members.iter() {
                    self.collect_free_types(m, out);
                }
            }
            ScopedTypeData::Tuple(elems) => {
                for e in elems.iter() {
                    self.collect_free_types(e.ty, out);
                }
            }
            ScopedTypeData::Record(fields) => {
                for f in fields.iter() {
                    self.collect_free_types(f.ty, out);
                }
            }
            ScopedTypeData::Callable(call) => {
                for p in call.parameters.iter() {
                    self.collect_free_types(p.ty, out);
                }
                self.collect_free_types(call.return_type, out);
            }
            ScopedTypeData::Lambda(lid) => {
                let inner = self.get_lambda(*lid);
                self.collect_free_types(inner.body, out);
            }
        }
    }

    /// Beta-reduces a TypeLambda applied to concrete `TypeId` arguments.
    /// If fewer arguments are supplied than parameters, returns a residual `TypeLambdaId` or canonical constructor.
    /// If fully applied, converts the scoped body back to a canonical `TypeId` in `store`.
    pub fn beta_reduce(&mut self, lambda_id: TypeLambdaId, args: &[TypeId], store: &mut TypeStore) -> Result<BetaResult, BetaReductionError> {
        let lambda = self.get_lambda(lambda_id).clone();
        if args.len() > lambda.parameter_kinds.len() {
            return Err(BetaReductionError::TooManyArguments {
                expected: lambda.parameter_kinds.len(),
                actual: args.len(),
            });
        }

        // Validate argument kinds against parameter kinds
        for (i, (&arg, &expected_kind)) in args.iter().zip(lambda.parameter_kinds.iter()).enumerate() {
            let actual_kind = store.kind_of(arg);
            if actual_kind != expected_kind {
                return Err(BetaReductionError::KindMismatch {
                    parameter_index: i as u32,
                    expected: expected_kind,
                    actual: actual_kind,
                });
            }
        }

        if args.len() == lambda.parameter_kinds.len() {
            let substituted = self.subst_scoped_to_canonical(lambda.body, 0, args, store);
            Ok(BetaResult::Canonical(substituted))
        } else {
            let residual_kinds = lambda.parameter_kinds[args.len()..].to_vec().into_boxed_slice();
            let shifted_body = self.subst_scoped_partial(lambda.body, 0, args, store);
            let residual_id = self.intern_lambda(residual_kinds, shifted_body, lambda.result_kind, None);
            Ok(BetaResult::ResidualLambda(residual_id))
        }
    }

    fn subst_scoped_to_canonical(&self, scoped: ScopedTypeId, depth: u32, args: &[TypeId], store: &mut TypeStore) -> TypeId {
        match self.get_scoped(scoped).clone() {
            ScopedTypeData::Bound { depth: d, index: idx } => {
                if d == depth && (idx as usize) < args.len() {
                    args[idx as usize]
                } else {
                    store.never()
                }
            }
            ScopedTypeData::Free(ty) => ty,
            ScopedTypeData::Applied { origin, arguments } => {
                let can_origin = self.subst_scoped_to_canonical(origin, depth, args, store);
                let can_args: Vec<TypeId> = arguments.iter().map(|&a| self.subst_scoped_to_canonical(a, depth, args, store)).collect();
                store.apply_type_form(can_origin, &can_args).unwrap_or(store.never())
            }
            ScopedTypeData::Union(members) => {
                let can_members: Vec<TypeId> = members.iter().map(|&m| self.subst_scoped_to_canonical(m, depth, args, store)).collect();
                store.union(&can_members)
            }
            ScopedTypeData::Tuple(elems) => {
                let can_elems: Vec<super::store::TupleTypeElement> = elems
                    .iter()
                    .map(|e| super::store::TupleTypeElement {
                        label: e.label.clone(),
                        ty: self.subst_scoped_to_canonical(e.ty, depth, args, store),
                    })
                    .collect();
                store.tuple(can_elems.into_boxed_slice())
            }
            ScopedTypeData::Record(fields) => {
                let can_fields: Vec<super::store::RecordTypeField> = fields
                    .iter()
                    .map(|f| super::store::RecordTypeField {
                        name: f.name.clone(),
                        ty: self.subst_scoped_to_canonical(f.ty, depth, args, store),
                    })
                    .collect();
                store.record(can_fields.into_boxed_slice())
            }
            ScopedTypeData::Callable(call) => {
                let can_params: Vec<super::store::CallableParameterType> = call
                    .parameters
                    .iter()
                    .map(|p| super::store::CallableParameterType {
                        label: p.label.clone(),
                        ty: self.subst_scoped_to_canonical(p.ty, depth, args, store),
                        rest: p.rest,
                    })
                    .collect();
                let can_ret = self.subst_scoped_to_canonical(call.return_type, depth, args, store);
                store.callable(super::store::CallableType {
                    parameters: can_params.into_boxed_slice(),
                    return_type: can_ret,
                })
            }
            ScopedTypeData::Lambda(lid) => {
                let nested = self.get_lambda(lid).clone();
                let nested_body = self.subst_scoped_to_canonical(nested.body, depth + 1, args, store);
                let scoped_body = store.arena_mut().intern_scoped(ScopedTypeData::Free(nested_body));
                let new_lid = store.arena_mut().intern_lambda(nested.parameter_kinds, scoped_body, nested.result_kind, None);
                store.type_lambda(new_lid)
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn subst_scoped_partial(&mut self, scoped: ScopedTypeId, depth: u32, args: &[TypeId], store: &mut TypeStore) -> ScopedTypeId {
        match self.get_scoped(scoped).clone() {
            ScopedTypeData::Bound { depth: d, index: idx } => {
                if d == depth {
                    let idx_usize = idx as usize;
                    if idx_usize < args.len() {
                        let ty = args[idx_usize];
                        self.intern_scoped(ScopedTypeData::Free(ty))
                    } else {
                        let new_idx = idx - (args.len() as u32);
                        self.intern_scoped(ScopedTypeData::Bound { depth: d, index: new_idx })
                    }
                } else {
                    scoped
                }
            }
            ScopedTypeData::Free(_) => scoped,
            ScopedTypeData::Applied { origin, arguments } => {
                let s_origin = self.subst_scoped_partial(origin, depth, args, store);
                let s_args: Vec<ScopedTypeId> = arguments.iter().map(|&a| self.subst_scoped_partial(a, depth, args, store)).collect();
                self.intern_scoped(ScopedTypeData::Applied {
                    origin: s_origin,
                    arguments: s_args.into_boxed_slice(),
                })
            }
            ScopedTypeData::Union(members) => {
                let s_members: Vec<ScopedTypeId> = members.iter().map(|&m| self.subst_scoped_partial(m, depth, args, store)).collect();
                self.intern_scoped(ScopedTypeData::Union(s_members.into_boxed_slice()))
            }
            ScopedTypeData::Tuple(elems) => {
                let s_elems: Vec<ScopedTupleElement> = elems
                    .iter()
                    .map(|e| ScopedTupleElement {
                        label: e.label.clone(),
                        ty: self.subst_scoped_partial(e.ty, depth, args, store),
                    })
                    .collect();
                self.intern_scoped(ScopedTypeData::Tuple(s_elems.into_boxed_slice()))
            }
            ScopedTypeData::Record(fields) => {
                let s_fields: Vec<ScopedRecordField> = fields
                    .iter()
                    .map(|f| ScopedRecordField {
                        name: f.name.clone(),
                        ty: self.subst_scoped_partial(f.ty, depth, args, store),
                    })
                    .collect();
                self.intern_scoped(ScopedTypeData::Record(s_fields.into_boxed_slice()))
            }
            ScopedTypeData::Callable(call) => {
                let s_params: Vec<ScopedCallableParameter> = call
                    .parameters
                    .iter()
                    .map(|p| ScopedCallableParameter {
                        label: p.label.clone(),
                        ty: self.subst_scoped_partial(p.ty, depth, args, store),
                        rest: p.rest,
                    })
                    .collect();
                let s_ret = self.subst_scoped_partial(call.return_type, depth, args, store);
                self.intern_scoped(ScopedTypeData::Callable(ScopedCallableType {
                    parameters: s_params.into_boxed_slice(),
                    return_type: s_ret,
                }))
            }
            ScopedTypeData::Lambda(lid) => {
                let nested = self.get_lambda(lid).clone();
                let s_body = self.subst_scoped_partial(nested.body, depth + 1, args, store);
                let new_lid = self.intern_lambda(nested.parameter_kinds, s_body, nested.result_kind, None);
                self.intern_scoped(ScopedTypeData::Lambda(new_lid))
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BetaResult {
    Canonical(TypeId),
    ResidualLambda(TypeLambdaId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BetaReductionError {
    TooManyArguments { expected: usize, actual: usize },
    KindMismatch { parameter_index: u32, expected: KindId, actual: KindId },
}
