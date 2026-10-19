//! Alpha-normalized Type Lambda calculus and scoped bound representation.

use super::id::{KindId, ScopedTypeId, TypeId, TypeLambdaId, TypeParameterId};
use super::overlay::PersistentHashMapOverlay;
use super::parameter::GenericParameterMode;
use super::row::{RecordRowFormationError, RecordRowTail};
use super::store::{PackFormationError, StaticArgument, TypeData, TypeStore};
use super::substitution::TypeSubstitution;
use crate::diagnostic::SemanticSourceSpan;
use crate::field_key::FieldKey;
use im::Vector;

/// Representation of scoped types inside a type lambda body arena.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
    /// Scoped open record type with a capture-safe stable row tail.
    OpenRecord(ScopedOpenRecord),
    /// Row-valued generic argument. Unlike `Record`/`OpenRecord`, this node has
    /// kind `RecordFields` and is never a proper type on its own.
    RecordRow(ScopedRecordRow),
    /// Scoped callable type.
    Callable(ScopedCallableType),
    /// Proper Pack type whose element and remainder constraints may mention
    /// lambda-bound variables.
    Pack(ScopedPackType),
    /// Ordered positional pack whose elements may mention bound lambda variables.
    TypePack(Box<[ScopedTypeId]>),
    /// Ordered labeled pack whose elements may mention bound lambda variables.
    LabeledTypePack(Box<[ScopedLabeledTypePackElement]>),
    /// Exact two-lane pack product in a scoped body.
    CompleteTypePack { positionals: ScopedTypeId, labeled: ScopedTypeId },
    /// Symbolic positional pack composition in a scoped body.
    TypePackConcat(Box<[ScopedTypeId]>),
    /// Symbolic labeled pack composition in a scoped body.
    LabeledTypePackConcat(Box<[ScopedTypeId]>),
    /// Tuple formation from scoped symbolic lanes.
    TupleFromPacks { positionals: ScopedTypeId, labeled: ScopedTypeId },
    /// Pack formation from scoped symbolic lanes.
    PackFromPacks { positionals: ScopedTypeId, labeled: ScopedTypeId },
    /// Callable formation from scoped symbolic lanes and return type.
    CallableFromPacks {
        positionals: ScopedTypeId,
        labeled: ScopedTypeId,
        return_type: ScopedTypeId,
    },
    /// Lane projection retained while its complete-pack subject is bound.
    PackProjection { subject: ScopedTypeId, lane: ScopedPackProjectionLane },
    /// Nested lambda reference.
    Lambda(TypeLambdaId),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopedTupleElement {
    pub label: Option<Box<str>>,
    pub ty: ScopedTypeId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ScopedPackRemainder {
    None,
    Unconstrained,
    Typed(ScopedTypeId),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopedPackType {
    pub elements: Box<[ScopedTupleElement]>,
    pub positional_remainder: ScopedPackRemainder,
    pub labeled_remainder: ScopedPackRemainder,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopedLabeledTypePackElement {
    pub label: Box<str>,
    pub element: ScopedTypeId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ScopedPackProjectionLane {
    Positionals,
    Labeled,
    Complete,
}

pub type ScopedAnonymousRecordRemainder = super::row::AnonymousRecordRemainder<ScopedTypeId>;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopedRecordField {
    pub key: FieldKey,
    pub ty: ScopedTypeId,
    pub presence: super::row::StructuralMemberPresence,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ScopedRecordTail {
    Bound { depth: u32, index: u32 },
    FreeParameter(TypeParameterId),
    Anonymous(ScopedAnonymousRecordRemainder),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopedOpenRecord {
    pub fields: Box<[ScopedRecordField]>,
    pub tail: ScopedRecordTail,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ScopedRecordRowTail {
    Closed,
    Anonymous(ScopedAnonymousRecordRemainder),
    Bound { depth: u32, index: u32 },
    FreeParameter(TypeParameterId),
    Projection(TypeId),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopedRecordRow {
    pub fields: Box<[ScopedRecordField]>,
    pub tail: ScopedRecordRowTail,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopedCallableParameter {
    pub label: Option<Box<str>>,
    pub ty: ScopedTypeId,
    pub rest: bool,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopedCallableType {
    pub parameters: Box<[ScopedCallableParameter]>,
    pub return_type: ScopedTypeId,
}

/// Canonical alpha-normalized type lambda definition.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeLambdaData {
    pub parameter_kinds: Box<[KindId]>,
    pub parameter_modes: Box<[GenericParameterMode]>,
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

use std::sync::Arc;

/// Frozen immutable prefix for type-lambda arena loaded from a compiler artifact.
#[derive(Debug, Eq, PartialEq)]
pub struct FrozenTypeLambdaPrefix {
    pub scoped_types: Box<[ScopedTypeData]>,
    pub scoped_to_id: Box<[ScopedTypeId]>,
    pub lambdas: Box<[TypeLambdaData]>,
    pub lambda_to_id: Box<[TypeLambdaId]>,
    pub provenance: Box<[TypeLambdaProvenance]>,
}

impl FrozenTypeLambdaPrefix {
    pub fn find_scoped(&self, key: &ScopedTypeData) -> Option<ScopedTypeId> {
        let idx = self.scoped_to_id.binary_search_by(|&id| self.scoped_types[id.index()].cmp(key)).ok()?;
        Some(self.scoped_to_id[idx])
    }

    pub fn find_lambda(&self, key: &TypeLambdaData) -> Option<TypeLambdaId> {
        let idx = self.lambda_to_id.binary_search_by(|&id| self.lambdas[id.index()].cmp(key)).ok()?;
        Some(self.lambda_to_id[idx])
    }
}

/// Arena managing scoped types and alpha-normalized type lambdas.
#[derive(Clone, Debug, Default)]
pub struct TypeLambdaArena {
    frozen: Option<Arc<FrozenTypeLambdaPrefix>>,
    scoped_types: Vector<ScopedTypeData>,
    scoped_to_id: PersistentHashMapOverlay<ScopedTypeData, ScopedTypeId>,
    lambdas: Vector<TypeLambdaData>,
    lambda_to_id: PersistentHashMapOverlay<TypeLambdaData, TypeLambdaId>,
    provenance: Vector<TypeLambdaProvenance>,
}

impl TypeLambdaArena {
    /// Bounded persistence census, excluding presentation provenance.
    pub(crate) fn artifact_counts(&self) -> (usize, usize) {
        let frozen_scoped = self.frozen.as_ref().map_or(0, |f| f.scoped_types.len());
        let frozen_lambdas = self.frozen.as_ref().map_or(0, |f| f.lambdas.len());
        (frozen_scoped + self.scoped_types.len(), frozen_lambdas + self.lambdas.len())
    }

    pub(crate) fn extends_prefix(&self, prefix: &Self) -> bool {
        let (self_scoped, self_lambdas) = self.artifact_counts();
        let (prefix_scoped, prefix_lambdas) = prefix.artifact_counts();
        self_scoped >= prefix_scoped
            && self_lambdas >= prefix_lambdas
            && (0..prefix_scoped).all(|i| self.get_scoped(ScopedTypeId::from_index(i)) == prefix.get_scoped(ScopedTypeId::from_index(i)))
            && (0..prefix_lambdas).all(|i| self.get_lambda(TypeLambdaId::from_index(i)) == prefix.get_lambda(TypeLambdaId::from_index(i)))
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_frozen(frozen: Arc<FrozenTypeLambdaPrefix>) -> Self {
        Self {
            frozen: Some(frozen),
            scoped_types: Vector::new(),
            scoped_to_id: PersistentHashMapOverlay::default(),
            lambdas: Vector::new(),
            lambda_to_id: PersistentHashMapOverlay::default(),
            provenance: Vector::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_backing_with(&self, other: &Self) -> bool {
        let frozen_share = match (&self.frozen, &other.frozen) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        frozen_share
            && (self.scoped_types.is_empty() && other.scoped_types.is_empty() || self.scoped_types.ptr_eq(&other.scoped_types))
            && self.scoped_to_id.ptr_eq(&other.scoped_to_id)
            && (self.lambdas.is_empty() && other.lambdas.is_empty() || self.lambdas.ptr_eq(&other.lambdas))
            && self.lambda_to_id.ptr_eq(&other.lambda_to_id)
            && (self.provenance.is_empty() && other.provenance.is_empty() || self.provenance.ptr_eq(&other.provenance))
    }

    pub fn intern_scoped(&mut self, data: ScopedTypeData) -> ScopedTypeId {
        if let Some(&id) = self.scoped_to_id.get(&data) {
            return id;
        }
        if let Some(frozen) = &self.frozen {
            if let Some(id) = frozen.find_scoped(&data) {
                return id;
            }
        }
        let frozen_len = self.frozen.as_ref().map_or(0, |f| f.scoped_types.len());
        let id = ScopedTypeId::from_index(frozen_len + self.scoped_types.len());
        self.scoped_types.push_back(data.clone());
        self.scoped_to_id.insert(data, id);
        id
    }

    pub fn get_scoped(&self, id: ScopedTypeId) -> &ScopedTypeData {
        let index = id.index();
        if let Some(frozen) = &self.frozen {
            if index < frozen.scoped_types.len() {
                return &frozen.scoped_types[index];
            }
            &self.scoped_types[index - frozen.scoped_types.len()]
        } else {
            &self.scoped_types[index]
        }
    }

    pub fn intern_lambda(
        &mut self,
        parameter_kinds: Box<[KindId]>,
        body: ScopedTypeId,
        result_kind: KindId,
        prov: Option<TypeLambdaProvenance>,
    ) -> TypeLambdaId {
        let parameter_modes = vec![GenericParameterMode::Single; parameter_kinds.len()].into_boxed_slice();
        self.intern_lambda_with_modes(parameter_kinds, parameter_modes, body, result_kind, prov)
    }

    /// Interns a lambda while preserving each source binder's application mode.
    pub fn intern_lambda_with_modes(
        &mut self,
        parameter_kinds: Box<[KindId]>,
        parameter_modes: Box<[GenericParameterMode]>,
        body: ScopedTypeId,
        result_kind: KindId,
        prov: Option<TypeLambdaProvenance>,
    ) -> TypeLambdaId {
        assert_eq!(
            parameter_kinds.len(),
            parameter_modes.len(),
            "type-lambda parameter kinds and modes must remain aligned"
        );
        let data = TypeLambdaData {
            parameter_kinds,
            parameter_modes,
            body,
            result_kind,
        };
        if let Some(&id) = self.lambda_to_id.get(&data) {
            return id;
        }
        if let Some(frozen) = &self.frozen {
            if let Some(id) = frozen.find_lambda(&data) {
                return id;
            }
        }
        let frozen_len = self.frozen.as_ref().map_or(0, |f| f.lambdas.len());
        let id = TypeLambdaId::from_index(frozen_len + self.lambdas.len());
        self.lambdas.push_back(data.clone());
        self.lambda_to_id.insert(data, id);
        self.provenance.push_back(prov.unwrap_or_default());
        id
    }

    pub fn get_lambda(&self, id: TypeLambdaId) -> &TypeLambdaData {
        let index = id.index();
        if let Some(frozen) = &self.frozen {
            if index < frozen.lambdas.len() {
                return &frozen.lambdas[index];
            }
            &self.lambdas[index - frozen.lambdas.len()]
        } else {
            &self.lambdas[index]
        }
    }

    pub fn get_provenance(&self, id: TypeLambdaId) -> Option<&TypeLambdaProvenance> {
        let index = id.index();
        if let Some(frozen) = &self.frozen {
            if index < frozen.provenance.len() {
                return Some(&frozen.provenance[index]);
            }
            self.provenance.get(index - frozen.provenance.len())
        } else {
            self.provenance.get(index)
        }
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
            ScopedTypeData::OpenRecord(record) => {
                matches!(record.tail, ScopedRecordTail::Bound { depth, .. } if depth >= current_depth)
                    || match record.tail {
                        ScopedRecordTail::Anonymous(ScopedAnonymousRecordRemainder::Unconstrained { key_type }) => self.has_free_bound(key_type, current_depth),
                        ScopedRecordTail::Anonymous(ScopedAnonymousRecordRemainder::Typed { key_type, value_type }) => {
                            self.has_free_bound(key_type, current_depth) || self.has_free_bound(value_type, current_depth)
                        }
                        _ => false,
                    }
                    || record.fields.iter().any(|f| self.has_free_bound(f.ty, current_depth))
            }
            ScopedTypeData::RecordRow(row) => {
                matches!(row.tail, ScopedRecordRowTail::Bound { depth, .. } if depth >= current_depth)
                    || match row.tail {
                        ScopedRecordRowTail::Anonymous(ScopedAnonymousRecordRemainder::Unconstrained { key_type }) => {
                            self.has_free_bound(key_type, current_depth)
                        }
                        ScopedRecordRowTail::Anonymous(ScopedAnonymousRecordRemainder::Typed { key_type, value_type }) => {
                            self.has_free_bound(key_type, current_depth) || self.has_free_bound(value_type, current_depth)
                        }
                        _ => false,
                    }
                    || row.fields.iter().any(|field| self.has_free_bound(field.ty, current_depth))
            }
            ScopedTypeData::Callable(call) => {
                call.parameters.iter().any(|p| self.has_free_bound(p.ty, current_depth)) || self.has_free_bound(call.return_type, current_depth)
            }
            ScopedTypeData::Pack(pack) => {
                pack.elements.iter().any(|element| self.has_free_bound(element.ty, current_depth))
                    || match pack.positional_remainder {
                        ScopedPackRemainder::Typed(ty) => self.has_free_bound(ty, current_depth),
                        ScopedPackRemainder::None | ScopedPackRemainder::Unconstrained => false,
                    }
                    || match pack.labeled_remainder {
                        ScopedPackRemainder::Typed(ty) => self.has_free_bound(ty, current_depth),
                        ScopedPackRemainder::None | ScopedPackRemainder::Unconstrained => false,
                    }
            }
            ScopedTypeData::TypePack(elements) | ScopedTypeData::TypePackConcat(elements) => {
                elements.iter().any(|&element| self.has_free_bound(element, current_depth))
            }
            ScopedTypeData::LabeledTypePack(elements) => elements.iter().any(|element| self.has_free_bound(element.element, current_depth)),
            ScopedTypeData::LabeledTypePackConcat(segments) => segments.iter().any(|&segment| self.has_free_bound(segment, current_depth)),
            ScopedTypeData::CompleteTypePack { positionals, labeled }
            | ScopedTypeData::TupleFromPacks { positionals, labeled }
            | ScopedTypeData::PackFromPacks { positionals, labeled } => {
                self.has_free_bound(*positionals, current_depth) || self.has_free_bound(*labeled, current_depth)
            }
            ScopedTypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                self.has_free_bound(*positionals, current_depth)
                    || self.has_free_bound(*labeled, current_depth)
                    || self.has_free_bound(*return_type, current_depth)
            }
            ScopedTypeData::PackProjection { subject, .. } => self.has_free_bound(*subject, current_depth),
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
            ScopedTypeData::OpenRecord(record) => {
                for f in record.fields.iter() {
                    self.collect_free_types(f.ty, out);
                }
            }
            ScopedTypeData::RecordRow(row) => {
                for field in row.fields.iter() {
                    self.collect_free_types(field.ty, out);
                }
                if let ScopedRecordRowTail::Projection(projection) = row.tail
                    && !out.contains(&projection)
                {
                    out.push(projection);
                }
            }
            ScopedTypeData::Callable(call) => {
                for p in call.parameters.iter() {
                    self.collect_free_types(p.ty, out);
                }
                self.collect_free_types(call.return_type, out);
            }
            ScopedTypeData::Pack(pack) => {
                for element in pack.elements.iter() {
                    self.collect_free_types(element.ty, out);
                }
                if let ScopedPackRemainder::Typed(ty) = pack.positional_remainder {
                    self.collect_free_types(ty, out);
                }
                if let ScopedPackRemainder::Typed(ty) = pack.labeled_remainder {
                    self.collect_free_types(ty, out);
                }
            }
            ScopedTypeData::TypePack(elements) | ScopedTypeData::TypePackConcat(elements) => {
                for &element in elements.iter() {
                    self.collect_free_types(element, out);
                }
            }
            ScopedTypeData::LabeledTypePack(elements) => {
                for element in elements.iter() {
                    self.collect_free_types(element.element, out);
                }
            }
            ScopedTypeData::LabeledTypePackConcat(segments) => {
                for &segment in segments.iter() {
                    self.collect_free_types(segment, out);
                }
            }
            ScopedTypeData::CompleteTypePack { positionals, labeled }
            | ScopedTypeData::TupleFromPacks { positionals, labeled }
            | ScopedTypeData::PackFromPacks { positionals, labeled } => {
                self.collect_free_types(*positionals, out);
                self.collect_free_types(*labeled, out);
            }
            ScopedTypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                self.collect_free_types(*positionals, out);
                self.collect_free_types(*labeled, out);
                self.collect_free_types(*return_type, out);
            }
            ScopedTypeData::PackProjection { subject, .. } => self.collect_free_types(*subject, out),
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
            let substituted = self.subst_scoped_to_canonical(lambda.body, 0, args, store)?;
            Ok(BetaResult::Canonical(substituted))
        } else {
            let residual_kinds = lambda.parameter_kinds[args.len()..].to_vec().into_boxed_slice();
            let residual_modes = lambda.parameter_modes[args.len()..].to_vec().into_boxed_slice();
            let shifted_body = self.subst_scoped_partial(lambda.body, 0, args, store);
            let residual_id = self.intern_lambda_with_modes(residual_kinds, residual_modes, shifted_body, lambda.result_kind, None);
            Ok(BetaResult::ResidualLambda(residual_id))
        }
    }

    /// Substitutes canonical free types captured by a lambda while preserving
    /// its bound variables. Receiver specialization uses this when a generic
    /// superclass passes a constructor lambda that closes over an owner
    /// parameter, for example `Either<E, X>`.
    pub fn substitute_free_types(&mut self, lambda_id: TypeLambdaId, substitution: &TypeSubstitution, store: &mut TypeStore) -> TypeLambdaId {
        let lambda = self.get_lambda(lambda_id).clone();
        let body = self.substitute_free_scoped(lambda.body, substitution, store);
        self.intern_lambda_with_modes(lambda.parameter_kinds, lambda.parameter_modes, body, lambda.result_kind, None)
    }

    fn substitute_free_scoped(&mut self, scoped: ScopedTypeId, substitution: &TypeSubstitution, store: &mut TypeStore) -> ScopedTypeId {
        match self.get_scoped(scoped).clone() {
            ScopedTypeData::Bound { .. } => scoped,
            ScopedTypeData::Free(ty) => {
                let replacement = substitution.apply_with_arena(store, self, ty);
                self.intern_scoped(ScopedTypeData::Free(replacement))
            }
            ScopedTypeData::Applied { origin, arguments } => {
                let origin = self.substitute_free_scoped(origin, substitution, store);
                let arguments = arguments
                    .iter()
                    .map(|&argument| self.substitute_free_scoped(argument, substitution, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                self.intern_scoped(ScopedTypeData::Applied { origin, arguments })
            }
            ScopedTypeData::Union(members) => {
                let members = members
                    .iter()
                    .map(|&member| self.substitute_free_scoped(member, substitution, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                self.intern_scoped(ScopedTypeData::Union(members))
            }
            ScopedTypeData::Tuple(elements) => {
                let elements = elements
                    .iter()
                    .map(|element| ScopedTupleElement {
                        label: element.label.clone(),
                        ty: self.substitute_free_scoped(element.ty, substitution, store),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                self.intern_scoped(ScopedTypeData::Tuple(elements))
            }
            ScopedTypeData::Record(fields) => {
                let fields = fields
                    .iter()
                    .map(|field| ScopedRecordField {
                        key: field.key.clone(),
                        ty: self.substitute_free_scoped(field.ty, substitution, store),
                        presence: field.presence,
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                self.intern_scoped(ScopedTypeData::Record(fields))
            }
            ScopedTypeData::OpenRecord(record) => {
                let fields = record
                    .fields
                    .iter()
                    .map(|field| ScopedRecordField {
                        key: field.key.clone(),
                        ty: self.substitute_free_scoped(field.ty, substitution, store),
                        presence: field.presence,
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                let tail = match record.tail {
                    ScopedRecordTail::Anonymous(ScopedAnonymousRecordRemainder::Unconstrained { key_type }) => {
                        ScopedRecordTail::Anonymous(ScopedAnonymousRecordRemainder::Unconstrained {
                            key_type: self.substitute_free_scoped(key_type, substitution, store),
                        })
                    }
                    ScopedRecordTail::Anonymous(ScopedAnonymousRecordRemainder::Typed { key_type, value_type }) => {
                        ScopedRecordTail::Anonymous(ScopedAnonymousRecordRemainder::Typed {
                            key_type: self.substitute_free_scoped(key_type, substitution, store),
                            value_type: self.substitute_free_scoped(value_type, substitution, store),
                        })
                    }
                    other => other,
                };
                self.intern_scoped(ScopedTypeData::OpenRecord(ScopedOpenRecord { fields, tail }))
            }
            ScopedTypeData::RecordRow(row) => {
                let fields = row
                    .fields
                    .iter()
                    .map(|field| ScopedRecordField {
                        key: field.key.clone(),
                        ty: self.substitute_free_scoped(field.ty, substitution, store),
                        presence: field.presence,
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                let tail = match row.tail {
                    ScopedRecordRowTail::Projection(projection) => ScopedRecordRowTail::Projection(substitution.apply_with_arena(store, self, projection)),
                    ScopedRecordRowTail::Anonymous(ScopedAnonymousRecordRemainder::Unconstrained { key_type }) => {
                        ScopedRecordRowTail::Anonymous(ScopedAnonymousRecordRemainder::Unconstrained {
                            key_type: self.substitute_free_scoped(key_type, substitution, store),
                        })
                    }
                    ScopedRecordRowTail::Anonymous(ScopedAnonymousRecordRemainder::Typed { key_type, value_type }) => {
                        ScopedRecordRowTail::Anonymous(ScopedAnonymousRecordRemainder::Typed {
                            key_type: self.substitute_free_scoped(key_type, substitution, store),
                            value_type: self.substitute_free_scoped(value_type, substitution, store),
                        })
                    }
                    tail => tail,
                };
                self.intern_scoped(ScopedTypeData::RecordRow(ScopedRecordRow { fields, tail }))
            }
            ScopedTypeData::Callable(callable) => {
                let parameters = callable
                    .parameters
                    .iter()
                    .map(|parameter| ScopedCallableParameter {
                        label: parameter.label.clone(),
                        ty: self.substitute_free_scoped(parameter.ty, substitution, store),
                        rest: parameter.rest,
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                let return_type = self.substitute_free_scoped(callable.return_type, substitution, store);
                self.intern_scoped(ScopedTypeData::Callable(ScopedCallableType { parameters, return_type }))
            }
            ScopedTypeData::Pack(pack) => {
                let elements = pack
                    .elements
                    .iter()
                    .map(|element| ScopedTupleElement {
                        label: element.label.clone(),
                        ty: self.substitute_free_scoped(element.ty, substitution, store),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                let mut substitute_remainder = |arena: &mut Self, remainder| match remainder {
                    ScopedPackRemainder::Typed(ty) => ScopedPackRemainder::Typed(arena.substitute_free_scoped(ty, substitution, store)),
                    remainder => remainder,
                };
                let positional_remainder = substitute_remainder(self, pack.positional_remainder);
                let labeled_remainder = substitute_remainder(self, pack.labeled_remainder);
                self.intern_scoped(ScopedTypeData::Pack(ScopedPackType {
                    elements,
                    positional_remainder,
                    labeled_remainder,
                }))
            }
            ScopedTypeData::TypePack(elements) => {
                let elements = elements
                    .iter()
                    .map(|&element| self.substitute_free_scoped(element, substitution, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                self.intern_scoped(ScopedTypeData::TypePack(elements))
            }
            ScopedTypeData::LabeledTypePack(elements) => {
                let elements = elements
                    .iter()
                    .map(|element| ScopedLabeledTypePackElement {
                        label: element.label.clone(),
                        element: self.substitute_free_scoped(element.element, substitution, store),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                self.intern_scoped(ScopedTypeData::LabeledTypePack(elements))
            }
            ScopedTypeData::CompleteTypePack { positionals, labeled } => {
                let positionals = self.substitute_free_scoped(positionals, substitution, store);
                let labeled = self.substitute_free_scoped(labeled, substitution, store);
                self.intern_scoped(ScopedTypeData::CompleteTypePack { positionals, labeled })
            }
            ScopedTypeData::TypePackConcat(segments) => {
                let segments = segments
                    .iter()
                    .map(|&segment| self.substitute_free_scoped(segment, substitution, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                self.intern_scoped(ScopedTypeData::TypePackConcat(segments))
            }
            ScopedTypeData::LabeledTypePackConcat(segments) => {
                let segments = segments
                    .iter()
                    .map(|&segment| self.substitute_free_scoped(segment, substitution, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                self.intern_scoped(ScopedTypeData::LabeledTypePackConcat(segments))
            }
            ScopedTypeData::TupleFromPacks { positionals, labeled } => {
                let positionals = self.substitute_free_scoped(positionals, substitution, store);
                let labeled = self.substitute_free_scoped(labeled, substitution, store);
                self.intern_scoped(ScopedTypeData::TupleFromPacks { positionals, labeled })
            }
            ScopedTypeData::PackFromPacks { positionals, labeled } => {
                let positionals = self.substitute_free_scoped(positionals, substitution, store);
                let labeled = self.substitute_free_scoped(labeled, substitution, store);
                self.intern_scoped(ScopedTypeData::PackFromPacks { positionals, labeled })
            }
            ScopedTypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                let positionals = self.substitute_free_scoped(positionals, substitution, store);
                let labeled = self.substitute_free_scoped(labeled, substitution, store);
                let return_type = self.substitute_free_scoped(return_type, substitution, store);
                self.intern_scoped(ScopedTypeData::CallableFromPacks {
                    positionals,
                    labeled,
                    return_type,
                })
            }
            ScopedTypeData::PackProjection { subject, lane } => {
                let subject = self.substitute_free_scoped(subject, substitution, store);
                self.intern_scoped(ScopedTypeData::PackProjection { subject, lane })
            }
            ScopedTypeData::Lambda(nested_id) => {
                let nested = self.get_lambda(nested_id).clone();
                let body = self.substitute_free_scoped(nested.body, substitution, store);
                let nested_id = self.intern_lambda_with_modes(nested.parameter_kinds, nested.parameter_modes, body, nested.result_kind, None);
                self.intern_scoped(ScopedTypeData::Lambda(nested_id))
            }
        }
    }

    fn subst_scoped_to_canonical(&mut self, scoped: ScopedTypeId, depth: u32, args: &[TypeId], store: &mut TypeStore) -> Result<TypeId, BetaReductionError> {
        match self.get_scoped(scoped).clone() {
            ScopedTypeData::Bound { depth: d, index: idx } => {
                if d == depth && (idx as usize) < args.len() {
                    Ok(args[idx as usize])
                } else {
                    Err(BetaReductionError::UnboundVariable { depth: d, index: idx })
                }
            }
            ScopedTypeData::Free(ty) => Ok(ty),
            ScopedTypeData::Applied { origin, arguments } => {
                let can_origin = self.subst_scoped_to_canonical(origin, depth, args, store)?;
                let can_args: Vec<StaticArgument> = arguments
                    .iter()
                    .map(|&argument| self.subst_scoped_to_static_argument(argument, depth, args, store))
                    .collect::<Result<_, _>>()?;
                store
                    .apply_static_form_with_arena(self, can_origin, &can_args)
                    .map_err(BetaReductionError::Application)
            }
            ScopedTypeData::Union(members) => {
                let can_members: Vec<TypeId> = members
                    .iter()
                    .map(|&m| self.subst_scoped_to_canonical(m, depth, args, store))
                    .collect::<Result<_, _>>()?;
                Ok(store.union(&can_members))
            }
            ScopedTypeData::Tuple(elems) => {
                let can_elems: Vec<super::store::TupleTypeElement> = elems
                    .iter()
                    .map(|e| {
                        Ok(super::store::TupleTypeElement {
                            label: e.label.clone(),
                            ty: self.subst_scoped_to_canonical(e.ty, depth, args, store)?,
                        })
                    })
                    .collect::<Result<_, BetaReductionError>>()?;
                Ok(store.tuple(can_elems.into_boxed_slice()))
            }
            ScopedTypeData::Record(fields) => {
                let can_fields: Vec<super::store::RecordTypeField> = fields
                    .iter()
                    .map(|f| {
                        Ok(super::store::RecordTypeField {
                            key: f.key.clone(),
                            ty: self.subst_scoped_to_canonical(f.ty, depth, args, store)?,
                            presence: f.presence,
                        })
                    })
                    .collect::<Result<_, BetaReductionError>>()?;
                Ok(store
                    .record_row_type_checked(can_fields, RecordRowTail::Closed)
                    .map_err(BetaReductionError::RecordRowFormation)?)
            }
            ScopedTypeData::OpenRecord(record) => {
                let can_fields = record
                    .fields
                    .iter()
                    .map(|field| {
                        Ok(super::store::RecordTypeField {
                            key: field.key.clone(),
                            ty: self.subst_scoped_to_canonical(field.ty, depth, args, store)?,
                            presence: field.presence,
                        })
                    })
                    .collect::<Result<Vec<_>, BetaReductionError>>()?;
                let tail = match record.tail {
                    ScopedRecordTail::FreeParameter(parameter) => RecordRowTail::Parameter(parameter),
                    ScopedRecordTail::Anonymous(ScopedAnonymousRecordRemainder::Unconstrained { key_type }) => {
                        let k = self.subst_scoped_to_canonical(key_type, depth, args, store)?;
                        RecordRowTail::Anonymous(super::row::AnonymousRecordRemainder::Unconstrained { key_type: k })
                    }
                    ScopedRecordTail::Anonymous(ScopedAnonymousRecordRemainder::Typed { key_type, value_type }) => {
                        let k = self.subst_scoped_to_canonical(key_type, depth, args, store)?;
                        let v = self.subst_scoped_to_canonical(value_type, depth, args, store)?;
                        RecordRowTail::Anonymous(super::row::AnonymousRecordRemainder::Typed { key_type: k, value_type: v })
                    }
                    ScopedRecordTail::Bound { .. } => return Err(BetaReductionError::UnsupportedRecordRowArgument),
                };
                store.record_row_type_checked(can_fields, tail).map_err(BetaReductionError::RecordRowFormation)
            }
            ScopedTypeData::RecordRow(_) => Err(BetaReductionError::UnsupportedRecordRowArgument),
            ScopedTypeData::Callable(call) => {
                let can_params: Vec<super::store::CallableParameterType> = call
                    .parameters
                    .iter()
                    .map(|p| {
                        Ok(super::store::CallableParameterType {
                            label: p.label.clone(),
                            ty: self.subst_scoped_to_canonical(p.ty, depth, args, store)?,
                            rest: if p.rest {
                                phalcom_ast::ast::RestMode::Positional
                            } else {
                                phalcom_ast::ast::RestMode::None
                            },
                        })
                    })
                    .collect::<Result<_, BetaReductionError>>()?;
                let can_ret = self.subst_scoped_to_canonical(call.return_type, depth, args, store)?;
                Ok(store.callable(super::store::CallableType {
                    parameters: can_params.into_boxed_slice(),
                    return_type: can_ret,
                }))
            }
            ScopedTypeData::Pack(pack) => {
                let elements = pack
                    .elements
                    .iter()
                    .map(|element| {
                        Ok(super::store::PackTypeElement {
                            label: element.label.clone(),
                            ty: self.subst_scoped_to_canonical(element.ty, depth, args, store)?,
                        })
                    })
                    .collect::<Result<Vec<_>, BetaReductionError>>()?
                    .into_boxed_slice();
                let positional_remainder = match pack.positional_remainder {
                    ScopedPackRemainder::None => super::store::AnonymousTupleRemainder::None,
                    ScopedPackRemainder::Unconstrained => super::store::AnonymousTupleRemainder::Unconstrained,
                    ScopedPackRemainder::Typed(ty) => super::store::AnonymousTupleRemainder::Typed(self.subst_scoped_to_canonical(ty, depth, args, store)?),
                };
                let labeled_remainder = match pack.labeled_remainder {
                    ScopedPackRemainder::None => super::store::AnonymousTupleRemainder::None,
                    ScopedPackRemainder::Unconstrained => super::store::AnonymousTupleRemainder::Unconstrained,
                    ScopedPackRemainder::Typed(ty) => super::store::AnonymousTupleRemainder::Typed(self.subst_scoped_to_canonical(ty, depth, args, store)?),
                };
                store
                    .open_pack(elements, positional_remainder, labeled_remainder)
                    .map_err(BetaReductionError::PackFormation)
            }
            ScopedTypeData::TypePack(elements) => {
                let elements = elements
                    .iter()
                    .map(|&element| self.subst_scoped_to_canonical(element, depth, args, store))
                    .collect::<Result<Vec<_>, _>>()?;
                store.type_pack(elements.into_boxed_slice()).map_err(BetaReductionError::PackFormation)
            }
            ScopedTypeData::LabeledTypePack(elements) => {
                let elements = elements
                    .iter()
                    .map(|element| {
                        Ok(super::store::LabeledTypePackElement {
                            label: element.label.clone(),
                            element: self.subst_scoped_to_canonical(element.element, depth, args, store)?,
                        })
                    })
                    .collect::<Result<Vec<_>, BetaReductionError>>()?;
                store.labeled_type_pack(elements.into_boxed_slice()).map_err(BetaReductionError::PackFormation)
            }
            ScopedTypeData::CompleteTypePack { positionals, labeled } => {
                let positionals = self.subst_scoped_to_canonical(positionals, depth, args, store)?;
                let labeled = self.subst_scoped_to_canonical(labeled, depth, args, store)?;
                store.complete_type_pack(positionals, labeled).map_err(BetaReductionError::PackFormation)
            }
            ScopedTypeData::TypePackConcat(segments) => {
                let segments = segments
                    .iter()
                    .map(|&segment| self.subst_scoped_to_canonical(segment, depth, args, store))
                    .collect::<Result<Vec<_>, _>>()?;
                store.type_pack_concat(&segments).map_err(BetaReductionError::PackFormation)
            }
            ScopedTypeData::LabeledTypePackConcat(segments) => {
                let segments = segments
                    .iter()
                    .map(|&segment| self.subst_scoped_to_canonical(segment, depth, args, store))
                    .collect::<Result<Vec<_>, _>>()?;
                store.labeled_type_pack_concat(&segments).map_err(BetaReductionError::PackFormation)
            }
            ScopedTypeData::TupleFromPacks { positionals, labeled } => {
                let positionals = self.subst_scoped_to_canonical(positionals, depth, args, store)?;
                let labeled = self.subst_scoped_to_canonical(labeled, depth, args, store)?;
                store.tuple_from_packs(positionals, labeled).map_err(BetaReductionError::PackFormation)
            }
            ScopedTypeData::PackFromPacks { positionals, labeled } => {
                let positionals = self.subst_scoped_to_canonical(positionals, depth, args, store)?;
                let labeled = self.subst_scoped_to_canonical(labeled, depth, args, store)?;
                store.pack_from_packs(positionals, labeled).map_err(BetaReductionError::PackFormation)
            }
            ScopedTypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                let positionals = self.subst_scoped_to_canonical(positionals, depth, args, store)?;
                let labeled = self.subst_scoped_to_canonical(labeled, depth, args, store)?;
                let return_type = self.subst_scoped_to_canonical(return_type, depth, args, store)?;
                store
                    .callable_from_packs(positionals, labeled, return_type)
                    .map_err(BetaReductionError::PackFormation)
            }
            ScopedTypeData::PackProjection { subject, lane } => {
                let subject = self.subst_scoped_to_canonical(subject, depth, args, store)?;
                match lane {
                    ScopedPackProjectionLane::Positionals => store.project_positionals(subject),
                    ScopedPackProjectionLane::Labeled => store.project_labeled(subject),
                    ScopedPackProjectionLane::Complete => store.project_complete(subject),
                }
                .map_err(BetaReductionError::PackFormation)
            }
            ScopedTypeData::Lambda(lid) => {
                let nested = self.get_lambda(lid).clone();
                // Inner binders remain scoped while outer arguments are
                // substituted. Converting through `subst_scoped_to_canonical`
                // would incorrectly reject those bound de Bruijn nodes as
                // unbound once the enclosing lambda is saturated.
                let scoped_body = self.subst_scoped_partial(nested.body, depth + 1, args, store);
                let parameter_kinds = nested.parameter_kinds.clone();
                let new_lid = self.intern_lambda_with_modes(parameter_kinds.clone(), nested.parameter_modes, scoped_body, nested.result_kind, None);
                // Keep the new lambda ID in this same active arena. The
                // enclosing TypeStore boundary restores this arena only after
                // all recursive canonical applications have completed.
                let kind = if parameter_kinds.is_empty() {
                    nested.result_kind
                } else {
                    store.arrow_kind(parameter_kinds, nested.result_kind)
                };
                Ok(store.intern_with_kind(TypeData::Lambda(new_lid), kind))
            }
        }
    }

    fn subst_scoped_to_static_argument(
        &mut self,
        scoped: ScopedTypeId,
        depth: u32,
        args: &[TypeId],
        store: &mut TypeStore,
    ) -> Result<StaticArgument, BetaReductionError> {
        let ScopedTypeData::RecordRow(row) = self.get_scoped(scoped).clone() else {
            return self.subst_scoped_to_canonical(scoped, depth, args, store).map(StaticArgument::Term);
        };

        let fields = row
            .fields
            .iter()
            .map(|field| {
                Ok(super::store::RecordTypeField {
                    key: field.key.clone(),
                    ty: self.subst_scoped_to_canonical(field.ty, depth, args, store)?,
                    presence: field.presence,
                })
            })
            .collect::<Result<Vec<_>, BetaReductionError>>()?;
        let tail = match row.tail {
            ScopedRecordRowTail::Closed => RecordRowTail::Closed,
            ScopedRecordRowTail::Bound { .. } => return Err(BetaReductionError::UnsupportedRecordRowArgument),
            ScopedRecordRowTail::FreeParameter(parameter) => RecordRowTail::Parameter(parameter),
            ScopedRecordRowTail::Projection(projection) => RecordRowTail::Projection(projection),
            ScopedRecordRowTail::Anonymous(ScopedAnonymousRecordRemainder::Unconstrained { key_type }) => {
                let k = self.subst_scoped_to_canonical(key_type, depth, args, store)?;
                RecordRowTail::Anonymous(super::row::AnonymousRecordRemainder::Unconstrained { key_type: k })
            }
            ScopedRecordRowTail::Anonymous(ScopedAnonymousRecordRemainder::Typed { key_type, value_type }) => {
                let k = self.subst_scoped_to_canonical(key_type, depth, args, store)?;
                let v = self.subst_scoped_to_canonical(value_type, depth, args, store)?;
                RecordRowTail::Anonymous(super::row::AnonymousRecordRemainder::Typed { key_type: k, value_type: v })
            }
        };
        let row = store.record_row_checked(fields, tail).map_err(BetaReductionError::RecordRowFormation)?;
        Ok(StaticArgument::RecordRow(row))
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
                        key: f.key.clone(),
                        ty: self.subst_scoped_partial(f.ty, depth, args, store),
                        presence: f.presence,
                    })
                    .collect();
                self.intern_scoped(ScopedTypeData::Record(s_fields.into_boxed_slice()))
            }
            ScopedTypeData::OpenRecord(record) => {
                let fields = record
                    .fields
                    .iter()
                    .map(|field| ScopedRecordField {
                        key: field.key.clone(),
                        ty: self.subst_scoped_partial(field.ty, depth, args, store),
                        presence: field.presence,
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                let tail = match record.tail {
                    ScopedRecordTail::Anonymous(ScopedAnonymousRecordRemainder::Unconstrained { key_type }) => {
                        ScopedRecordTail::Anonymous(ScopedAnonymousRecordRemainder::Unconstrained {
                            key_type: self.subst_scoped_partial(key_type, depth, args, store),
                        })
                    }
                    ScopedRecordTail::Anonymous(ScopedAnonymousRecordRemainder::Typed { key_type, value_type }) => {
                        ScopedRecordTail::Anonymous(ScopedAnonymousRecordRemainder::Typed {
                            key_type: self.subst_scoped_partial(key_type, depth, args, store),
                            value_type: self.subst_scoped_partial(value_type, depth, args, store),
                        })
                    }
                    other => other,
                };
                self.intern_scoped(ScopedTypeData::OpenRecord(ScopedOpenRecord { fields, tail }))
            }
            ScopedTypeData::RecordRow(row) => {
                let fields = row
                    .fields
                    .iter()
                    .map(|field| ScopedRecordField {
                        key: field.key.clone(),
                        ty: self.subst_scoped_partial(field.ty, depth, args, store),
                        presence: field.presence,
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                let tail = match row.tail {
                    ScopedRecordRowTail::Bound { depth: bound_depth, index } if bound_depth == depth && index >= args.len() as u32 => {
                        ScopedRecordRowTail::Bound {
                            depth: bound_depth,
                            index: index - args.len() as u32,
                        }
                    }
                    ScopedRecordRowTail::Anonymous(ScopedAnonymousRecordRemainder::Unconstrained { key_type }) => {
                        ScopedRecordRowTail::Anonymous(ScopedAnonymousRecordRemainder::Unconstrained {
                            key_type: self.subst_scoped_partial(key_type, depth, args, store),
                        })
                    }
                    ScopedRecordRowTail::Anonymous(ScopedAnonymousRecordRemainder::Typed { key_type, value_type }) => {
                        ScopedRecordRowTail::Anonymous(ScopedAnonymousRecordRemainder::Typed {
                            key_type: self.subst_scoped_partial(key_type, depth, args, store),
                            value_type: self.subst_scoped_partial(value_type, depth, args, store),
                        })
                    }
                    tail => tail,
                };
                self.intern_scoped(ScopedTypeData::RecordRow(ScopedRecordRow { fields, tail }))
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
            ScopedTypeData::Pack(pack) => {
                let elements = pack
                    .elements
                    .iter()
                    .map(|element| ScopedTupleElement {
                        label: element.label.clone(),
                        ty: self.subst_scoped_partial(element.ty, depth, args, store),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                let positional_remainder = match pack.positional_remainder {
                    ScopedPackRemainder::Typed(ty) => ScopedPackRemainder::Typed(self.subst_scoped_partial(ty, depth, args, store)),
                    remainder => remainder,
                };
                let labeled_remainder = match pack.labeled_remainder {
                    ScopedPackRemainder::Typed(ty) => ScopedPackRemainder::Typed(self.subst_scoped_partial(ty, depth, args, store)),
                    remainder => remainder,
                };
                self.intern_scoped(ScopedTypeData::Pack(ScopedPackType {
                    elements,
                    positional_remainder,
                    labeled_remainder,
                }))
            }
            ScopedTypeData::TypePack(elements) => {
                let elements = elements
                    .iter()
                    .map(|&element| self.subst_scoped_partial(element, depth, args, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                self.intern_scoped(ScopedTypeData::TypePack(elements))
            }
            ScopedTypeData::LabeledTypePack(elements) => {
                let elements = elements
                    .iter()
                    .map(|element| ScopedLabeledTypePackElement {
                        label: element.label.clone(),
                        element: self.subst_scoped_partial(element.element, depth, args, store),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                self.intern_scoped(ScopedTypeData::LabeledTypePack(elements))
            }
            ScopedTypeData::CompleteTypePack { positionals, labeled } => {
                let positionals = self.subst_scoped_partial(positionals, depth, args, store);
                let labeled = self.subst_scoped_partial(labeled, depth, args, store);
                self.intern_scoped(ScopedTypeData::CompleteTypePack { positionals, labeled })
            }
            ScopedTypeData::TypePackConcat(segments) => {
                let segments = segments
                    .iter()
                    .map(|&segment| self.subst_scoped_partial(segment, depth, args, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                self.intern_scoped(ScopedTypeData::TypePackConcat(segments))
            }
            ScopedTypeData::LabeledTypePackConcat(segments) => {
                let segments = segments
                    .iter()
                    .map(|&segment| self.subst_scoped_partial(segment, depth, args, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                self.intern_scoped(ScopedTypeData::LabeledTypePackConcat(segments))
            }
            ScopedTypeData::TupleFromPacks { positionals, labeled } => {
                let positionals = self.subst_scoped_partial(positionals, depth, args, store);
                let labeled = self.subst_scoped_partial(labeled, depth, args, store);
                self.intern_scoped(ScopedTypeData::TupleFromPacks { positionals, labeled })
            }
            ScopedTypeData::PackFromPacks { positionals, labeled } => {
                let positionals = self.subst_scoped_partial(positionals, depth, args, store);
                let labeled = self.subst_scoped_partial(labeled, depth, args, store);
                self.intern_scoped(ScopedTypeData::PackFromPacks { positionals, labeled })
            }
            ScopedTypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                let positionals = self.subst_scoped_partial(positionals, depth, args, store);
                let labeled = self.subst_scoped_partial(labeled, depth, args, store);
                let return_type = self.subst_scoped_partial(return_type, depth, args, store);
                self.intern_scoped(ScopedTypeData::CallableFromPacks {
                    positionals,
                    labeled,
                    return_type,
                })
            }
            ScopedTypeData::PackProjection { subject, lane } => {
                let subject = self.subst_scoped_partial(subject, depth, args, store);
                self.intern_scoped(ScopedTypeData::PackProjection { subject, lane })
            }
            ScopedTypeData::Lambda(lid) => {
                let nested = self.get_lambda(lid).clone();
                let s_body = self.subst_scoped_partial(nested.body, depth + 1, args, store);
                let new_lid = self.intern_lambda_with_modes(nested.parameter_kinds, nested.parameter_modes, s_body, nested.result_kind, None);
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

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum BetaReductionError {
    #[error("too many type-lambda arguments: supplied {actual}, expected at most {expected}")]
    TooManyArguments { expected: usize, actual: usize },
    #[error("type-lambda argument kind mismatch at parameter {parameter_index}")]
    KindMismatch { parameter_index: u32, expected: KindId, actual: KindId },
    #[error("type-lambda body contains an unbound variable at depth {depth}, index {index}")]
    UnboundVariable { depth: u32, index: u32 },
    #[error("type-lambda application cannot supply a RecordFields argument")]
    UnsupportedRecordRowArgument,
    #[error(transparent)]
    RecordRowFormation(#[from] RecordRowFormationError),
    #[error(transparent)]
    PackFormation(#[from] PackFormationError),
    #[error(transparent)]
    Application(#[from] super::application::TypeApplicationError),
}

#[cfg(test)]
mod persistent_root_tests {
    use super::{ScopedTypeData, TypeLambdaArena};
    use crate::types::id::TypeId;
    use std::ptr;

    #[test]
    fn cloned_type_lambda_arena_shares_existing_nodes_and_isolates_candidate_interning() {
        let mut committed = TypeLambdaArena::new();
        let first = committed.intern_scoped(ScopedTypeData::Bound { depth: 0, index: 0 });
        for index in 1..96 {
            committed.intern_scoped(ScopedTypeData::Bound { depth: 0, index });
        }
        let first_node_address = committed.get_scoped(first) as *const _;
        let mut candidate = committed.clone();

        assert!(committed.scoped_types.ptr_eq(&candidate.scoped_types));
        assert!(committed.scoped_to_id.ptr_eq(&candidate.scoped_to_id));
        let added = candidate.intern_scoped(ScopedTypeData::Free(TypeId::DUMMY));

        assert_eq!(committed.get_scoped(first), candidate.get_scoped(first));
        assert!(ptr::eq(first_node_address, candidate.get_scoped(first) as *const _));
        assert_ne!(added, first);
        assert_eq!(committed.scoped_types.len() + 1, candidate.scoped_types.len());
        assert!(committed.lambda_to_id.ptr_eq(&candidate.lambda_to_id));
    }
}
