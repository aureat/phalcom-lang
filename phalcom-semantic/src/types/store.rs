//! Canonical Type Store with interning and normalization.

use super::application::TypeApplicationError;
use super::family::{FamilyMemberType, FamilyMemberTypeKind, FamilyType, FamilyTypeError, FamilyTypeId};
use super::id::{KindId, ProperTypeId, RecordRowId, TypeId, TypeLambdaId, TypeParameterId, TypeStoreId, VariantTypeId};
use super::kind::{KindApplicationError, KindData};
use super::parameter::{SelfTypeTerm, TypeParameterData, TypeParameterOwner};
use super::row::{RecordRowData, RecordRowField, RecordRowTail};
use super::type_lambda::{BetaReductionError, BetaResult, TypeLambdaArena};
use super::variance::Variance;
use crate::identity::{DeclarationId, VariantId};
use phalcom_common::selector::{SelectorKind, SelectorSlot};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TYPE_STORE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TupleTypeElement {
    pub label: Option<Box<str>>,
    pub ty: TypeId,
}

pub type RecordTypeField = RecordRowField;

use phalcom_ast::ast::RestMode;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CallableParameterType {
    pub label: Option<Box<str>>,
    pub ty: TypeId,
    pub rest: RestMode,
}

impl CallableParameterType {
    pub fn new(ty: TypeId) -> Self {
        Self {
            label: None,
            ty,
            rest: RestMode::None,
        }
    }

    pub fn with_label(mut self, label: impl Into<Box<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_rest(mut self, rest: RestMode) -> Self {
        self.rest = rest;
        self
    }

    pub fn is_rest(&self) -> bool {
        self.rest != RestMode::None
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CallableType {
    pub parameters: Box<[CallableParameterType]>,
    pub return_type: TypeId,
}

/// Structural canonical type definition.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeData {
    /// Canonical bottom type (inhabited by no values).
    Never,
    /// Canonical unit type (inhabited by single unit value).
    Unit,
    /// Proper static value type of a runtime class object (internal semantic representation).
    ClassObject { declaration: DeclarationId },
    /// Canonical nominal class declaration type or constructor form.
    Nominal { declaration: DeclarationId },
    /// Generic type application (e.g. `List<Int>`).
    Applied { origin: TypeId, arguments: Box<[TypeId]> },
    /// Exact static enum case type (e.g. `ExactCase<Option::Some(_), Option<Int>>`).
    ExactCase { variant: VariantTypeId, enum_type: TypeId },
    /// Flat, deduplicated, sorted union of two or more distinct types.
    Union(Box<[TypeId]>),
    /// Tuple type.
    Tuple(Box<[TupleTypeElement]>),
    /// Record / structural property map backed by a canonical record row.
    Record(RecordRowId),
    /// Callable / block signature.
    Callable(CallableType),
    /// First-class structural associated member family.
    Family(FamilyTypeId),
    /// Type variable parameter in generic declaration.
    Parameter(TypeParameterId),
    /// First-class type lambda form.
    Lambda(TypeLambdaId),
    /// Owner-relative `Self` type term.
    SelfType(SelfTypeTerm),
}

/// Central store for canonical type interning, hash-consing, and kind assignments.
#[derive(Clone, Debug)]
pub struct TypeStore {
    id: TypeStoreId,
    types: Vec<TypeData>,
    type_to_id: HashMap<(TypeData, KindId), TypeId>,
    kinds: Vec<KindData>,
    kind_to_id: HashMap<KindData, KindId>,
    type_kinds: Vec<KindId>,
    type_parameters: Vec<TypeParameterData>,
    parameter_to_id: HashMap<(TypeParameterOwner, u32), TypeParameterId>,
    parameter_variances: HashMap<(DeclarationId, u32), Variance>,
    lambda_arena: TypeLambdaArena,
    row_arena: Vec<RecordRowData>,
    row_interner: HashMap<RecordRowData, RecordRowId>,
    family_arena: Vec<FamilyType>,
    family_interner: HashMap<FamilyType, FamilyTypeId>,
    variant_identities: Vec<VariantId>,
    variant_identity_to_id: HashMap<VariantId, VariantTypeId>,

    never_id: TypeId,
    unit_id: TypeId,
}

impl Default for TypeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeStore {
    pub fn new() -> Self {
        let mut store = Self {
            id: TypeStoreId(NEXT_TYPE_STORE_ID.fetch_add(1, Ordering::Relaxed)),
            types: Vec::new(),
            type_to_id: HashMap::new(),
            kinds: Vec::new(),
            kind_to_id: HashMap::new(),
            type_kinds: Vec::new(),
            type_parameters: Vec::new(),
            parameter_to_id: HashMap::new(),
            parameter_variances: HashMap::new(),
            lambda_arena: TypeLambdaArena::new(),
            row_arena: Vec::new(),
            row_interner: HashMap::new(),
            family_arena: Vec::new(),
            family_interner: HashMap::new(),
            variant_identities: Vec::new(),
            variant_identity_to_id: HashMap::new(),
            never_id: TypeId::DUMMY,
            unit_id: TypeId::DUMMY,
        };

        // Kind::Type is KindId(0)
        let type_kind = store.intern_kind(KindData::Type);
        assert_eq!(type_kind, KindId::TYPE);
        let record_row_kind = store.intern_kind(KindData::RecordRow);
        assert_eq!(record_row_kind, KindId::RECORD_ROW);

        store.never_id = store.intern_with_kind(TypeData::Never, KindId::TYPE);
        store.unit_id = store.intern_with_kind(TypeData::Unit, KindId::TYPE);

        store
    }

    pub fn id(&self) -> TypeStoreId {
        self.id
    }

    pub fn type_count(&self) -> usize {
        self.types.len()
    }

    pub fn proper_type(&self, id: TypeId) -> Result<ProperTypeId, KindId> {
        let kind = self.kind_of(id);
        if kind == KindId::TYPE {
            Ok(ProperTypeId(id))
        } else {
            Err(kind)
        }
    }

    pub fn intern_type_parameter(&mut self, data: TypeParameterData) -> TypeParameterId {
        let key = (data.owner.clone(), data.index);
        if let TypeParameterOwner::Declaration(ref decl) = data.owner {
            self.parameter_variances.insert((decl.clone(), data.index), data.variance);
        }

        if let Some(&id) = self.parameter_to_id.get(&key) {
            let existing = &self.type_parameters[id.index()];
            let same_semantics = existing.name == data.name && existing.kind == data.kind && existing.variance == data.variance;

            if same_semantics {
                // Source provenance is revision-local presentation data, not semantic
                // identity. Refresh it in the live store while retained snapshot clones
                // continue to preserve the provenance from their own revision.
                self.type_parameters[id.index()].source = data.source;
                return id;
            }
        }

        // `(owner, index)` identifies the current binder slot, not an eternal arena
        // object. If the slot's semantic meaning changes, allocate a new parameter
        // version rather than mutating the old one. Cached products and retained
        // snapshots may still reference the previous ID and must keep its denotation.
        let id = TypeParameterId(self.type_parameters.len() as u32);
        self.type_parameters.push(data);
        self.parameter_to_id.insert(key, id);
        id
    }

    pub fn set_parameter_variance(&mut self, decl: DeclarationId, index: u32, variance: Variance) {
        self.parameter_variances.insert((decl, index), variance);
    }

    pub fn get_parameter_variance(&self, decl: &DeclarationId, index: u32) -> Option<Variance> {
        self.parameter_variances.get(&(decl.clone(), index)).copied()
    }

    pub fn find_type_parameter_id(&self, owner: &TypeParameterOwner, index: u32) -> Option<TypeParameterId> {
        self.parameter_to_id.get(&(owner.clone(), index)).copied()
    }

    #[inline]
    pub fn arena(&self) -> &TypeLambdaArena {
        &self.lambda_arena
    }

    #[inline]
    pub fn arena_mut(&mut self) -> &mut TypeLambdaArena {
        &mut self.lambda_arena
    }

    /// Interns a type lambda form into the store.
    pub fn lambda(&mut self, parameter_kinds: Box<[KindId]>, body: super::id::ScopedTypeId, result_kind: KindId) -> TypeId {
        let lambda_id = self.lambda_arena.intern_lambda(parameter_kinds, body, result_kind, None);
        self.type_lambda(lambda_id)
    }

    /// Interns an existing `TypeLambdaId` into the `TypeStore` with its computed arrow kind.
    pub fn type_lambda(&mut self, lambda_id: TypeLambdaId) -> TypeId {
        let lambda = self.lambda_arena.get_lambda(lambda_id).clone();
        let kind = if lambda.parameter_kinds.is_empty() {
            lambda.result_kind
        } else {
            self.arrow_kind(lambda.parameter_kinds, lambda.result_kind)
        };
        self.intern_with_kind(TypeData::Lambda(lambda_id), kind)
    }

    /// Interns an owner-relative `Self` type term.
    pub fn self_type(&mut self, term: SelfTypeTerm) -> TypeId {
        self.intern_with_kind(TypeData::SelfType(term), KindId::TYPE)
    }

    #[inline]
    pub fn type_parameter(&self, id: TypeParameterId) -> &TypeParameterData {
        &self.type_parameters[id.index()]
    }

    pub fn parameter_form(&mut self, id: TypeParameterId) -> TypeId {
        let kind = self.type_parameter(id).kind;
        assert_ne!(
            kind,
            KindId::RECORD_ROW,
            "RecordRow-kinded type parameters must never produce TypeData::Parameter"
        );
        self.intern_with_kind(TypeData::Parameter(id), kind)
    }

    pub fn contains_parameter_type(&self, parameter: TypeParameterId) -> bool {
        self.types.iter().any(|ty| matches!(ty, TypeData::Parameter(id) if *id == parameter))
    }

    #[inline]
    pub fn never(&self) -> TypeId {
        self.never_id
    }

    #[inline]
    pub fn unit(&self) -> TypeId {
        self.unit_id
    }

    pub fn intern_kind(&mut self, data: KindData) -> KindId {
        if let Some(&id) = self.kind_to_id.get(&data) {
            return id;
        }
        let id = KindId(self.kinds.len() as u32);
        self.kinds.push(data.clone());
        self.kind_to_id.insert(data, id);
        id
    }

    pub fn get_kind(&self, id: KindId) -> &KindData {
        &self.kinds[id.index()]
    }

    pub fn format_kind(&self, id: KindId) -> String {
        match self.get_kind(id) {
            KindData::Type => "Type".to_string(),
            KindData::RecordRow => "RecordRow".to_string(),
            KindData::Arrow { parameters, result } => {
                let parameters = parameters.iter().map(|&parameter| self.format_kind(parameter)).collect::<Vec<_>>().join(", ");
                format!("({parameters}) -> {}", self.format_kind(*result))
            }
        }
    }

    pub fn arrow_kind(&mut self, parameters: Box<[KindId]>, result: KindId) -> KindId {
        if parameters.is_empty() {
            return result;
        }
        let (params, final_result) = match self.get_kind(result).clone() {
            KindData::Arrow {
                parameters: sub_params,
                result: sub_res,
            } => {
                let mut combined = parameters.to_vec();
                combined.extend_from_slice(&sub_params);
                (combined.into_boxed_slice(), sub_res)
            }
            _ => (parameters, result),
        };
        self.intern_kind(KindData::Arrow {
            parameters: params,
            result: final_result,
        })
    }

    pub fn apply_kind(&mut self, callee: KindId, arguments: &[KindId]) -> Result<KindId, KindApplicationError> {
        if arguments.is_empty() {
            return Ok(callee);
        }

        let callee_data = self.get_kind(callee).clone();
        match callee_data {
            KindData::Type | KindData::RecordRow => Err(KindApplicationError::NotApplicable { kind: callee }),
            KindData::Arrow { parameters, result } => {
                if arguments.len() > parameters.len() {
                    return Err(KindApplicationError::TooManyArguments {
                        supplied: arguments.len(),
                        accepted: parameters.len(),
                    });
                }
                for (i, (&arg, &param)) in arguments.iter().zip(parameters.iter()).enumerate() {
                    if arg != param {
                        return Err(KindApplicationError::ArgumentKindMismatch {
                            index: i,
                            expected: param,
                            actual: arg,
                        });
                    }
                }
                if arguments.len() == parameters.len() {
                    Ok(result)
                } else {
                    let remaining = parameters[arguments.len()..].to_vec().into_boxed_slice();
                    Ok(self.arrow_kind(remaining, result))
                }
            }
        }
    }

    pub fn intern_with_kind(&mut self, data: TypeData, kind: KindId) -> TypeId {
        // Kind is part of a canonical type form's identity. This matters for a
        // persistent store: the same declaration or parameter payload can acquire a
        // different kind in a later semantic revision, and the old TypeId must retain
        // its original denotation for cached products and retained snapshots.
        let key = (data.clone(), kind);
        if let Some(&id) = self.type_to_id.get(&key) {
            return id;
        }

        let id = TypeId(self.types.len() as u32);
        self.types.push(data);
        self.type_kinds.push(kind);
        self.type_to_id.insert(key, id);
        debug_assert_eq!(self.types.len(), self.type_kinds.len());
        id
    }

    #[inline]
    pub fn kind_of(&self, ty: TypeId) -> KindId {
        self.type_kinds[ty.index()]
    }

    #[inline]
    pub fn is_proper_type(&self, form: TypeId) -> bool {
        self.kind_of(form) == KindId::TYPE
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.types.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    #[inline]
    pub fn get(&self, id: TypeId) -> &TypeData {
        &self.types[id.index()]
    }

    pub fn nominal_form(&mut self, declaration: DeclarationId, kind: KindId) -> TypeId {
        self.intern_with_kind(TypeData::Nominal { declaration }, kind)
    }

    pub fn nominal_type(&mut self, declaration: DeclarationId) -> TypeId {
        self.nominal_form(declaration, KindId::TYPE)
    }

    pub fn class_object_type(&mut self, declaration: DeclarationId) -> TypeId {
        self.intern_with_kind(TypeData::ClassObject { declaration }, KindId::TYPE)
    }

    /// Legacy compatibility helper for tests / callers expecting nominal type.
    pub fn nominal(&mut self, declaration: DeclarationId) -> TypeId {
        self.nominal_type(declaration)
    }

    /// Interns a generic applied type with checked kinding and beta-reduction for type lambdas.
    pub fn apply_type_form(&mut self, origin: TypeId, arguments: &[TypeId]) -> Result<TypeId, TypeApplicationError> {
        if arguments.is_empty() {
            return Ok(origin);
        }

        // If origin is a TypeLambda, perform beta-reduction directly
        if let TypeData::Lambda(lambda_id) = self.get(origin).clone() {
            let mut arena = self.lambda_arena.clone();
            let res = arena.beta_reduce(lambda_id, arguments, self);
            self.lambda_arena = arena;
            match res {
                Ok(BetaResult::Canonical(can)) => return Ok(can),
                Ok(BetaResult::ResidualLambda(res_id)) => {
                    return Ok(self.type_lambda(res_id));
                }
                Err(BetaReductionError::TooManyArguments { expected, actual }) => {
                    return Err(TypeApplicationError::TooManyArguments {
                        supplied: actual,
                        accepted: expected,
                    });
                }
                Err(BetaReductionError::KindMismatch {
                    parameter_index,
                    expected,
                    actual,
                }) => {
                    return Err(TypeApplicationError::ArgumentKindMismatch {
                        index: parameter_index as usize,
                        expected,
                        actual,
                    });
                }
                Err(BetaReductionError::UnboundVariable { .. }) => return Err(TypeApplicationError::MalformedLambda),
                Err(BetaReductionError::Application(error)) => return Err(error),
            }
        }

        let origin_kind = self.kind_of(origin);
        let arg_kinds: Vec<KindId> = arguments.iter().map(|&a| self.kind_of(a)).collect();

        let residual_kind = match self.apply_kind(origin_kind, &arg_kinds) {
            Ok(k) => k,
            Err(KindApplicationError::NotApplicable { kind }) => {
                return Err(TypeApplicationError::NotAConstructor { origin, kind });
            }
            Err(KindApplicationError::TooManyArguments { supplied, accepted }) => {
                return Err(TypeApplicationError::TooManyArguments { supplied, accepted });
            }
            Err(KindApplicationError::ArgumentKindMismatch { index, expected, actual }) => {
                return Err(TypeApplicationError::ArgumentKindMismatch { index, expected, actual });
            }
        };

        let (final_origin, final_args) = match self.get(origin).clone() {
            TypeData::Applied {
                origin: base,
                arguments: old_args,
            } => {
                let mut combined = old_args.to_vec();
                combined.extend_from_slice(arguments);
                (base, combined.into_boxed_slice())
            }
            _ => (origin, arguments.to_vec().into_boxed_slice()),
        };

        Ok(self.intern_with_kind(
            TypeData::Applied {
                origin: final_origin,
                arguments: final_args,
            },
            residual_kind,
        ))
    }

    /// Interns a tuple type.
    pub fn tuple(&mut self, elements: Box<[TupleTypeElement]>) -> TypeId {
        for elem in elements.iter() {
            debug_assert!(self.is_proper_type(elem.ty), "tuple element must be a proper type");
        }
        self.intern_with_kind(TypeData::Tuple(elements), KindId::TYPE)
    }

    /// Interns a record row into the store.
    pub fn intern_record_row(&mut self, data: RecordRowData) -> RecordRowId {
        if let Some(&id) = self.row_interner.get(&data) {
            return id;
        }
        let id = RecordRowId(self.row_arena.len() as u32);
        self.row_arena.push(data.clone());
        self.row_interner.insert(data, id);
        id
    }

    /// Accesses a record row by its ID.
    #[inline]
    pub fn record_row(&self, id: RecordRowId) -> &RecordRowData {
        &self.row_arena[id.index()]
    }

    /// Looks up an already-interned record row.
    pub fn find_record_row(&self, data: &RecordRowData) -> Option<RecordRowId> {
        self.row_interner.get(data).copied()
    }

    /// Returns the number of interned record rows.
    #[inline]
    pub fn record_row_count(&self) -> usize {
        self.row_arena.len()
    }

    /// Interns a record type backed by a canonical closed row.
    pub fn record(&mut self, fields: Box<[RecordRowField]>) -> TypeId {
        for field in fields.iter() {
            debug_assert!(self.is_proper_type(field.ty), "record field must be a proper type");
        }
        let mut sorted_fields = fields.into_vec();
        sorted_fields.sort_by(|a, b| a.name.cmp(&b.name));
        for i in 1..sorted_fields.len() {
            assert_ne!(sorted_fields[i - 1].name, sorted_fields[i].name, "duplicate record field");
        }
        let row_id = self.intern_record_row(RecordRowData {
            fields: sorted_fields.into_boxed_slice(),
            tail: RecordRowTail::Closed,
        });
        self.intern_with_kind(TypeData::Record(row_id), KindId::TYPE)
    }

    /// Interns a record type from an already-interned row.
    pub fn record_type(&mut self, row_id: RecordRowId) -> TypeId {
        self.intern_with_kind(TypeData::Record(row_id), KindId::TYPE)
    }

    /// Interns a callable type.
    pub fn callable(&mut self, callable: CallableType) -> TypeId {
        for param in callable.parameters.iter() {
            debug_assert!(self.is_proper_type(param.ty), "callable parameter must be a proper type");
        }
        debug_assert!(self.is_proper_type(callable.return_type), "callable return type must be a proper type");
        self.intern_with_kind(TypeData::Callable(callable), KindId::TYPE)
    }

    /// Interns a structural associated family type.
    pub fn family_type(&mut self, members: impl IntoIterator<Item = FamilyMemberType>) -> Result<TypeId, FamilyTypeError> {
        let member_vec: Vec<FamilyMemberType> = members.into_iter().collect();
        for member in &member_vec {
            if member.member_kind == FamilyMemberTypeKind::Callable && !matches!(self.get(member.ty), TypeData::Callable(_)) {
                return Err(FamilyTypeError::CallableMemberNotCallable {
                    operation: member.operation.clone(),
                    ty: member.ty,
                });
            }
        }

        let mut sorted_members = member_vec;
        sorted_members.sort_by(|a, b| a.operation.cmp(&b.operation));

        let mut deduped: Vec<FamilyMemberType> = Vec::with_capacity(sorted_members.len());
        for member in sorted_members {
            if let Some(last) = deduped.last() {
                if last.operation == member.operation {
                    if last == &member {
                        continue;
                    } else {
                        return Err(FamilyTypeError::DuplicateOperationShape { operation: member.operation });
                    }
                }
            }
            deduped.push(member);
        }

        let family = FamilyType::new(deduped.into_boxed_slice());
        let family_id = if let Some(&id) = self.family_interner.get(&family) {
            id
        } else {
            let id = FamilyTypeId::new(self.family_arena.len() as u32);
            self.family_arena.push(family.clone());
            self.family_interner.insert(family, id);
            id
        };

        Ok(self.intern_with_kind(TypeData::Family(family_id), KindId::TYPE))
    }

    #[inline]
    pub fn get_family(&self, id: FamilyTypeId) -> &FamilyType {
        &self.family_arena[id.index()]
    }

    pub fn family_count(&self) -> usize {
        self.family_arena.len()
    }

    /// Interns a `List<T>` applied type.
    pub fn list_of(&mut self, list_form: TypeId, element: TypeId) -> Result<TypeId, TypeApplicationError> {
        self.apply_type_form(list_form, &[element])
    }

    /// Interns a `Map<K, V>` applied type.
    pub fn map_of(&mut self, map_form: TypeId, key: TypeId, value: TypeId) -> Result<TypeId, TypeApplicationError> {
        self.apply_type_form(map_form, &[key, value])
    }

    /// Interns a `Set<T>` applied type.
    pub fn set_of(&mut self, set_form: TypeId, element: TypeId) -> Result<TypeId, TypeApplicationError> {
        self.apply_type_form(set_form, &[element])
    }

    /// Normalizes and interns a union type.
    ///
    /// Rules:
    /// 1. Flatten nested unions.
    /// 2. Remove duplicates.
    /// 3. Remove `Never` when other members exist.
    /// 4. Sort canonically by TypeId.
    /// 5. Zero members -> `Never`.
    /// 6. One member -> member TypeId.
    pub fn union(&mut self, members: &[TypeId]) -> TypeId {
        for &m in members {
            debug_assert!(self.is_proper_type(m), "union member must be a proper type");
        }
        let mut flattened = Vec::new();
        self.collect_union_members(members, &mut flattened);

        // Deduplicate and sort
        flattened.sort_unstable_by_key(|t| t.0);
        flattened.dedup();

        // Remove Never if other types present
        if flattened.len() > 1 {
            flattened.retain(|&t| t != self.never_id);
        }

        match flattened.len() {
            0 => self.never_id,
            1 => flattened[0],
            _ => self.intern_with_kind(TypeData::Union(flattened.into_boxed_slice()), KindId::TYPE),
        }
    }

    fn collect_union_members(&self, members: &[TypeId], out: &mut Vec<TypeId>) {
        for &m in members {
            match self.get(m) {
                TypeData::Union(nested) => self.collect_union_members(nested, out),
                _ => out.push(m),
            }
        }
    }

    /// Formats a canonical TypeId into human-readable type syntax.
    pub fn format_type(&self, ty: TypeId) -> String {
        match self.get(ty) {
            TypeData::Never => "Never".to_string(),
            TypeData::Unit => "Unit".to_string(),
            TypeData::ClassObject { declaration } => format!("class {}", declaration.name),
            TypeData::Nominal { declaration } => declaration.name.to_string(),
            TypeData::Applied { origin, arguments } => {
                let orig_str = self.format_type(*origin);
                let args_str = arguments.iter().map(|&arg| self.format_type(arg)).collect::<Vec<_>>().join(", ");
                format!("{orig_str}<{args_str}>")
            }
            TypeData::Union(types) => types.iter().map(|&t| self.format_type(t)).collect::<Vec<_>>().join(" | "),
            TypeData::Tuple(elements) => {
                let elems = elements
                    .iter()
                    .map(|elem| {
                        let t_str = self.format_type(elem.ty);
                        if let Some(ref l) = elem.label {
                            format!("{l}: {t_str}")
                        } else {
                            t_str
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({elems})")
            }
            TypeData::Record(row_id) => {
                let row = &self.row_arena[row_id.index()];
                let fields = row
                    .fields
                    .iter()
                    .map(|f| format!("{}: {}", f.name, self.format_type(f.ty)))
                    .collect::<Vec<_>>()
                    .join(", ");
                match row.tail {
                    RecordRowTail::Closed => format!("{{{fields}}}"),
                    RecordRowTail::Parameter(p) => {
                        let p_name = &self.type_parameters[p.index()].name;
                        if fields.is_empty() {
                            format!("{{..{p_name}}}")
                        } else {
                            format!("{{{fields}, ..{p_name}}}")
                        }
                    }
                }
            }
            TypeData::Callable(callable) => {
                let params = callable
                    .parameters
                    .iter()
                    .map(|p| {
                        let t_str = self.format_type(p.ty);
                        let prefix = match p.rest {
                            RestMode::None => "",
                            RestMode::Positional => "...",
                            RestMode::Labeled => "...#",
                            RestMode::Complete => "...*",
                        };
                        if let Some(ref l) = p.label {
                            format!("{prefix}{l}: {t_str}")
                        } else {
                            format!("{prefix}{t_str}")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret = self.format_type(callable.return_type);
                format!("({params}) -> {ret}")
            }
            TypeData::Family(fid) => {
                let family = &self.family_arena[fid.index()];
                let member_strs = family
                    .members
                    .iter()
                    .map(|m| {
                        let kind_str = match m.operation.kind {
                            SelectorKind::Method => "method",
                            SelectorKind::Getter => "getter",
                            SelectorKind::Setter => "setter",
                            SelectorKind::SubscriptGet => "subscript_get",
                            SelectorKind::SubscriptSet => "subscript_set",
                        };
                        let mut slots_str = String::new();
                        if m.operation.kind == SelectorKind::Method {
                            let slots = m
                                .operation
                                .slots
                                .iter()
                                .map(|s| match s {
                                    SelectorSlot::Positional => "_",
                                    SelectorSlot::Label(l) => l.as_str(),
                                })
                                .collect::<Vec<_>>()
                                .join(", ");
                            slots_str = format!("({slots})");
                        }
                        let t_str = self.format_type(m.ty);
                        format!("{kind_str}{slots_str}: {t_str}")
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                format!("family {{{member_strs}}}")
            }
            TypeData::ExactCase { variant, enum_type } => {
                let variant_id = self.variant_identity(*variant);
                let enum_str = self.format_type(*enum_type);
                format!("ExactCase<{}::{}, {enum_str}>", variant_id.owner.name, variant_id.selector.encode())
            }
            TypeData::Parameter(param_id) => self.type_parameters[param_id.index()].name.to_string(),
            TypeData::Lambda(_) => "[TypeLambda]".to_string(),
            TypeData::SelfType(_) => "Self".to_string(),
        }
    }

    /// Interns a [`VariantId`] into a compact store-relative [`VariantTypeId`].
    pub fn intern_variant_identity(&mut self, variant: VariantId) -> VariantTypeId {
        if let Some(&id) = self.variant_identity_to_id.get(&variant) {
            return id;
        }
        let id = VariantTypeId::from_index(self.variant_identities.len());
        self.variant_identities.push(variant.clone());
        self.variant_identity_to_id.insert(variant, id);
        id
    }

    /// Returns the stable [`VariantId`] corresponding to a [`VariantTypeId`].
    pub fn variant_identity(&self, id: VariantTypeId) -> &VariantId {
        &self.variant_identities[id.index()]
    }

    /// Returns nominal origin declaration of `ty` if it is a nominal type or applied generic nominal type.
    pub fn nominal_origin_declaration(&self, ty: TypeId) -> Option<&DeclarationId> {
        match self.get(ty) {
            TypeData::Nominal { declaration } => Some(declaration),
            TypeData::Applied { origin, .. } => self.nominal_origin_declaration(*origin),
            TypeData::ExactCase { enum_type, .. } => self.nominal_origin_declaration(*enum_type),
            _ => None,
        }
    }

    /// Decomposes a nominal or applied nominal type into `(declaration, type_arguments)`.
    pub fn applied_nominal_parts(&self, ty: TypeId) -> Option<(DeclarationId, Vec<TypeId>)> {
        match self.get(ty) {
            TypeData::Nominal { declaration } => Some((declaration.clone(), Vec::new())),
            TypeData::Applied { origin, arguments } => {
                let decl = self.nominal_origin_declaration(*origin)?;
                Some((decl.clone(), arguments.to_vec()))
            }
            TypeData::ExactCase { enum_type, .. } => self.applied_nominal_parts(*enum_type),
            _ => None,
        }
    }

    /// Validates and interns a canonical exact static case type `ExactCase(variant, enum_type)`.
    pub fn exact_case_type(&mut self, variant: &VariantId, mut enum_type: TypeId) -> Result<TypeId, ExactCaseTypeError> {
        while let TypeData::ExactCase { enum_type: inner, .. } = self.get(enum_type) {
            enum_type = *inner;
        }
        let enum_kind = self.kind_of(enum_type);
        if enum_kind != KindId::TYPE {
            return Err(ExactCaseTypeError::EnumTypeMalformed);
        }
        let Some(origin) = self.nominal_origin_declaration(enum_type) else {
            return Err(ExactCaseTypeError::NominalOriginMissing);
        };
        if *origin != variant.owner {
            return Err(ExactCaseTypeError::WrongOwner {
                expected: variant.owner.clone(),
                got: origin.clone(),
            });
        }
        let variant_type_id = self.intern_variant_identity(variant.clone());
        Ok(self.intern_with_kind(
            TypeData::ExactCase {
                variant: variant_type_id,
                enum_type,
            },
            KindId::TYPE,
        ))
    }

    /// Checks if a type contains a specific type parameter.
    pub fn contains_type_parameter(&self, ty: TypeId, target: TypeParameterId) -> bool {
        match self.get(ty) {
            TypeData::Parameter(p) => *p == target,
            TypeData::Applied { origin, arguments } => {
                self.contains_type_parameter(*origin, target) || arguments.iter().any(|&a| self.contains_type_parameter(a, target))
            }
            TypeData::Union(members) => members.iter().any(|&m| self.contains_type_parameter(m, target)),
            TypeData::Tuple(elems) => elems.iter().any(|e| self.contains_type_parameter(e.ty, target)),
            TypeData::Record(row_id) => {
                let row = self.record_row(*row_id);
                row.fields.iter().any(|f| self.contains_type_parameter(f.ty, target))
            }
            TypeData::Callable(call) => {
                call.parameters.iter().any(|p| self.contains_type_parameter(p.ty, target)) || self.contains_type_parameter(call.return_type, target)
            }
            TypeData::ExactCase { enum_type, .. } => self.contains_type_parameter(*enum_type, target),
            _ => false,
        }
    }

    /// Formats TypeKnowledge into human-readable type syntax.
    pub fn format_knowledge(&self, knowledge: &crate::types::evidence::TypeKnowledge) -> String {
        match knowledge {
            crate::types::evidence::TypeKnowledge::Known(ev) => self.format_type(ev.ty()),
            crate::types::evidence::TypeKnowledge::Dynamic(_) => "Dynamic".to_string(),
            crate::types::evidence::TypeKnowledge::Unknown(_) => "Unknown".to_string(),
        }
    }
}

/// Error returned when constructing an invalid exact-case static type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExactCaseTypeError {
    EnumTypeMalformed,
    NominalOriginMissing,
    WrongOwner { expected: DeclarationId, got: DeclarationId },
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_modules::identity::ModuleId;

    fn test_decl(name: &str) -> DeclarationId {
        let module = ModuleId::universe_root();
        DeclarationId::new(module, name.into())
    }

    #[test]
    fn nominal_interning_is_canonical() {
        let mut store = TypeStore::new();
        let decl = test_decl("User");
        let t1 = store.nominal(decl.clone());
        let t2 = store.nominal(decl);
        assert_eq!(t1, t2);
    }

    #[test]
    fn union_normalization_flattens_deduplicates_and_sorts() {
        let mut store = TypeStore::new();
        let d_a = test_decl("A");
        let d_b = test_decl("B");
        let d_c = test_decl("C");
        let t_a = store.nominal(d_a);
        let t_b = store.nominal(d_b);
        let t_c = store.nominal(d_c);

        let u1 = store.union(&[t_a, t_b]);
        let u2 = store.union(&[t_b, t_a]);
        assert_eq!(u1, u2, "union order invariant");

        let u3 = store.union(&[u1, t_c]);
        let u4 = store.union(&[t_c, t_a, t_b, t_a]);
        assert_eq!(u3, u4, "nested and duplicated union invariant");

        let with_never = store.union(&[t_a, store.never()]);
        assert_eq!(with_never, t_a, "never removed from nonempty union");

        let empty_union = store.union(&[]);
        assert_eq!(empty_union, store.never(), "empty union is never");
    }
}
