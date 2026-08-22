//! Canonical declaration type forms and metadata table.

use crate::identity::DeclarationId;
use crate::types::id::{KindId, TypeId};
use crate::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner};
use crate::types::store::TypeStore;
use phalcom_native_meta::types::{KindSpec, UniverseTypeFormSpec};
use phalcom_native_meta::universe::{UNIVERSE_BINDINGS, UNIVERSE_TYPE_FORMS, UniverseKey};
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationTypeInfo {
    pub declaration: DeclarationId,
    pub form: TypeId,
    pub class_object_type: TypeId,
    pub kind: KindId,
    pub generic_signature: Option<GenericSignature>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DeclarationTypeTable {
    entries: HashMap<DeclarationId, DeclarationTypeInfo>,
}

impl DeclarationTypeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, info: DeclarationTypeInfo) {
        self.entries.insert(info.declaration.clone(), info);
    }

    pub fn get(&self, declaration: &DeclarationId) -> Option<&DeclarationTypeInfo> {
        self.entries.get(declaration)
    }

    pub fn form(&self, declaration: &DeclarationId) -> Option<TypeId> {
        self.entries.get(declaration).map(|info| info.form)
    }

    pub fn class_object_type(&self, declaration: &DeclarationId) -> Option<TypeId> {
        self.entries
            .get(declaration)
            .map(|info| info.class_object_type)
    }

    pub fn kind(&self, declaration: &DeclarationId) -> Option<KindId> {
        self.entries.get(declaration).map(|info| info.kind)
    }

    pub fn generic_signature(&self, declaration: &DeclarationId) -> Option<&GenericSignature> {
        self.entries
            .get(declaration)
            .and_then(|info| info.generic_signature.as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DeclarationId, &DeclarationTypeInfo)> {
        self.entries.iter()
    }
}

/// Helper function to lower a `KindSpec` into a canonical `KindId`.
pub fn lower_kind_spec(store: &mut TypeStore, spec: &KindSpec) -> KindId {
    match spec {
        KindSpec::Type => KindId::TYPE,
        KindSpec::Arrow { parameters, result } => {
            let param_kinds: Vec<KindId> = parameters
                .iter()
                .map(|p| lower_kind_spec(store, p))
                .collect();
            let res_kind = lower_kind_spec(store, result);
            store.arrow_kind(param_kinds.into_boxed_slice(), res_kind)
        }
    }
}

/// Bootstraps canonical declaration type forms for all core universe classes.
pub fn bootstrap_universe_declarations(
    store: &mut TypeStore,
    universe_resolver: &dyn Fn(UniverseKey) -> DeclarationId,
) -> DeclarationTypeTable {
    let mut table = DeclarationTypeTable::new();

    let mut generic_specs: HashMap<UniverseKey, &UniverseTypeFormSpec> = HashMap::new();
    for spec in UNIVERSE_TYPE_FORMS {
        generic_specs.insert(spec.owner, spec);
    }

    for binding in UNIVERSE_BINDINGS {
        let key = binding.key;
        let decl = universe_resolver(key);

        if let Some(spec) = generic_specs.get(&key) {
            let mut param_ids = Vec::new();
            let mut param_kinds = Vec::new();

            for (idx, p) in spec.parameters.iter().enumerate() {
                let p_kind = lower_kind_spec(store, &p.kind);
                let param_id = store.intern_type_parameter(TypeParameterData {
                    owner: TypeParameterOwner::Declaration(decl.clone()),
                    index: idx as u16,
                    name: p.name.into(),
                    kind: p_kind,
                });
                param_ids.push(param_id);
                param_kinds.push(p_kind);
            }

            let decl_kind = store.arrow_kind(param_kinds.into_boxed_slice(), KindId::TYPE);
            let form = store.nominal_form(decl.clone(), decl_kind);
            let class_obj_type = store.class_object_type(decl.clone());

            table.insert(DeclarationTypeInfo {
                declaration: decl.clone(),
                form,
                class_object_type: class_obj_type,
                kind: decl_kind,
                generic_signature: Some(GenericSignature {
                    owner: TypeParameterOwner::Declaration(decl),
                    parameters: param_ids.into_boxed_slice(),
                }),
            });
        } else {
            let form = store.nominal_type(decl.clone());
            let class_obj_type = store.class_object_type(decl.clone());

            table.insert(DeclarationTypeInfo {
                declaration: decl,
                form,
                class_object_type: class_obj_type,
                kind: KindId::TYPE,
                generic_signature: None,
            });
        }
    }

    table
}
