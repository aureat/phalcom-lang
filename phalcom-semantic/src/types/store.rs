//! Canonical Type Store with interning and normalization.

use super::application::TypeApplicationError;
use super::id::{InferVarId, KindId, TypeId, TypeParameterId};
use super::kind::{KindApplicationError, KindData};
use super::parameter::{TypeParameterData, TypeParameterOwner};
use crate::identity::DeclarationId;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TupleTypeElement {
    pub label: Option<Box<str>>,
    pub ty: TypeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordTypeField {
    pub name: Box<str>,
    pub ty: TypeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CallableParameterType {
    pub label: Option<Box<str>>,
    pub ty: TypeId,
    pub rest: bool,
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
    /// Flat, deduplicated, sorted union of two or more distinct types.
    Union(Box<[TypeId]>),
    /// Tuple type.
    Tuple(Box<[TupleTypeElement]>),
    /// Record / structural property map.
    Record(Box<[RecordTypeField]>),
    /// Callable / block signature.
    Callable(CallableType),
    /// Type variable parameter in generic declaration.
    Parameter(TypeParameterId),
    /// Type inference variable.
    Infer(InferVarId),
}

/// Central store for canonical type interning, hash-consing, and kind assignments.
#[derive(Clone, Debug)]
pub struct TypeStore {
    types: Vec<TypeData>,
    type_to_id: HashMap<TypeData, TypeId>,
    kinds: Vec<KindData>,
    kind_to_id: HashMap<KindData, KindId>,
    type_kinds: Vec<KindId>,
    type_parameters: Vec<TypeParameterData>,
    parameter_to_id: HashMap<(TypeParameterOwner, u16), TypeParameterId>,

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
            types: Vec::new(),
            type_to_id: HashMap::new(),
            kinds: Vec::new(),
            kind_to_id: HashMap::new(),
            type_kinds: Vec::new(),
            type_parameters: Vec::new(),
            parameter_to_id: HashMap::new(),
            never_id: TypeId::DUMMY,
            unit_id: TypeId::DUMMY,
        };

        // Kind::Type is KindId(0)
        let type_kind = store.intern_kind(KindData::Type);
        assert_eq!(type_kind, KindId::TYPE);

        store.never_id = store.intern_with_kind(TypeData::Never, KindId::TYPE);
        store.unit_id = store.intern_with_kind(TypeData::Unit, KindId::TYPE);

        store
    }

    pub fn intern_type_parameter(&mut self, data: TypeParameterData) -> TypeParameterId {
        let key = (data.owner.clone(), data.index);
        if let Some(&id) = self.parameter_to_id.get(&key) {
            return id;
        }
        let id = TypeParameterId(self.type_parameters.len() as u32);
        self.type_parameters.push(data);
        self.parameter_to_id.insert(key, id);
        id
    }

    #[inline]
    pub fn type_parameter(&self, id: TypeParameterId) -> &TypeParameterData {
        &self.type_parameters[id.index()]
    }

    pub fn parameter_form(&mut self, id: TypeParameterId) -> TypeId {
        let kind = self.type_parameter(id).kind;
        self.intern_with_kind(TypeData::Parameter(id), kind)
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

    pub fn apply_kind(
        &mut self,
        callee: KindId,
        arguments: &[KindId],
    ) -> Result<KindId, KindApplicationError> {
        if arguments.is_empty() {
            return Ok(callee);
        }

        let callee_data = self.get_kind(callee).clone();
        match callee_data {
            KindData::Type => Err(KindApplicationError::NotApplicable { kind: callee }),
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
        if let Some(&id) = self.type_to_id.get(&data) {
            debug_assert_eq!(self.type_kinds[id.index()], kind);
            return id;
        }

        let id = TypeId(self.types.len() as u32);
        self.types.push(data.clone());
        self.type_kinds.push(kind);
        self.type_to_id.insert(data, id);
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

    /// Interns a generic applied type with checked kinding.
    pub fn apply_type_form(
        &mut self,
        origin: TypeId,
        arguments: &[TypeId],
    ) -> Result<TypeId, TypeApplicationError> {
        if arguments.is_empty() {
            return Ok(origin);
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
            Err(KindApplicationError::ArgumentKindMismatch {
                index,
                expected,
                actual,
            }) => {
                return Err(TypeApplicationError::ArgumentKindMismatch {
                    index,
                    expected,
                    actual,
                });
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
            debug_assert!(
                self.is_proper_type(elem.ty),
                "tuple element must be a proper type"
            );
        }
        self.intern_with_kind(TypeData::Tuple(elements), KindId::TYPE)
    }

    /// Interns a record type.
    pub fn record(&mut self, fields: Box<[RecordTypeField]>) -> TypeId {
        for field in fields.iter() {
            debug_assert!(
                self.is_proper_type(field.ty),
                "record field must be a proper type"
            );
        }
        self.intern_with_kind(TypeData::Record(fields), KindId::TYPE)
    }

    /// Interns a callable type.
    pub fn callable(&mut self, callable: CallableType) -> TypeId {
        for param in callable.parameters.iter() {
            debug_assert!(
                self.is_proper_type(param.ty),
                "callable parameter must be a proper type"
            );
        }
        debug_assert!(
            self.is_proper_type(callable.return_type),
            "callable return type must be a proper type"
        );
        self.intern_with_kind(TypeData::Callable(callable), KindId::TYPE)
    }

    /// Interns an inference type variable.
    pub fn infer(&mut self, var: InferVarId) -> TypeId {
        self.intern_with_kind(TypeData::Infer(var), KindId::TYPE)
    }

    /// Interns a `List<T>` applied type.
    pub fn list_of(
        &mut self,
        list_form: TypeId,
        element: TypeId,
    ) -> Result<TypeId, TypeApplicationError> {
        self.apply_type_form(list_form, &[element])
    }

    /// Interns a `Map<K, V>` applied type.
    pub fn map_of(
        &mut self,
        map_form: TypeId,
        key: TypeId,
        value: TypeId,
    ) -> Result<TypeId, TypeApplicationError> {
        self.apply_type_form(map_form, &[key, value])
    }

    /// Interns a `Set<T>` applied type.
    pub fn set_of(
        &mut self,
        set_form: TypeId,
        element: TypeId,
    ) -> Result<TypeId, TypeApplicationError> {
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
            debug_assert!(
                self.is_proper_type(m),
                "union member must be a proper type"
            );
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
            _ => self.intern_with_kind(
                TypeData::Union(flattened.into_boxed_slice()),
                KindId::TYPE,
            ),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_modules::identity::ModuleId;

    fn test_decl(name: &str) -> DeclarationId {
        let module = ModuleId::core();
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
