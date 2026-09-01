//! Canonical declaration type forms and metadata table.

use crate::identity::DeclarationId;
use crate::type_alias::TypeAliasInfo;
use crate::types::id::{KindId, TypeId};
use crate::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner};
use crate::types::store::TypeStore;
use phalcom_native_meta::types::{KindSpec, UniverseTypeFormSpec};
use phalcom_native_meta::universe::{UniverseKey, UNIVERSE_BINDINGS, UNIVERSE_TYPE_FORMS};
use std::collections::HashMap;

/// Generic supertype template: records static generic supertype (e.g. `Names<T> is Sequence<Option<T>>`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericSupertypeTemplate {
    pub declaration: DeclarationId,
    pub supertype: TypeId,
    pub structural_form: Option<Box<str>>,
}

impl GenericSupertypeTemplate {
    pub fn from_type(store: &TypeStore, declaration: DeclarationId, supertype: TypeId) -> Self {
        Self {
            declaration,
            structural_form: Some(store.format_type(supertype).into_boxed_str()),
            supertype,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationTypeInfo {
    pub declaration: DeclarationId,
    pub form: TypeId,
    pub class_object_type: TypeId,
    pub kind: KindId,
    pub generic_signature: Option<GenericSignature>,
    pub supertype_template: Option<GenericSupertypeTemplate>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeDeclarationShell {
    Nominal(DeclarationTypeInfo),
    Alias(TypeAliasInfo),
}

impl TypeDeclarationShell {
    pub fn declaration(&self) -> &DeclarationId {
        match self {
            Self::Nominal(info) => &info.declaration,
            Self::Alias(info) => &info.declaration,
        }
    }
}

/// Final declaration header assembled from one validated generic signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NominalDeclarationHeader {
    pub declaration: DeclarationId,
    pub form: TypeId,
    pub class_object_type: TypeId,
    pub kind: KindId,
    pub generic_signature: Option<GenericSignature>,
}

impl NominalDeclarationHeader {
    pub fn from_signature(store: &mut TypeStore, declaration: DeclarationId, generic_signature: Option<GenericSignature>) -> Self {
        let kind = generic_signature.as_ref().map_or(KindId::TYPE, |signature| {
            let parameter_kinds = signature
                .parameters
                .iter()
                .map(|&parameter| store.type_parameter(parameter).kind)
                .collect::<Vec<_>>();
            store.arrow_kind(parameter_kinds.into_boxed_slice(), KindId::TYPE)
        });
        let form = store.nominal_form(declaration.clone(), kind);
        let class_object_type = store.class_object_type(declaration.clone());
        Self {
            declaration,
            form,
            class_object_type,
            kind,
            generic_signature,
        }
    }

    pub fn into_type_info(self, supertype_template: Option<GenericSupertypeTemplate>) -> DeclarationTypeInfo {
        DeclarationTypeInfo {
            declaration: self.declaration,
            form: self.form,
            class_object_type: self.class_object_type,
            kind: self.kind,
            generic_signature: self.generic_signature,
            supertype_template,
        }
    }
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

    pub fn remove(&mut self, declaration: &DeclarationId) -> Option<DeclarationTypeInfo> {
        self.entries.remove(declaration)
    }

    pub fn get(&self, declaration: &DeclarationId) -> Option<&DeclarationTypeInfo> {
        self.entries.get(declaration)
    }

    pub fn form(&self, declaration: &DeclarationId) -> Option<TypeId> {
        self.entries.get(declaration).map(|info| info.form)
    }

    pub fn class_object_type(&self, declaration: &DeclarationId) -> Option<TypeId> {
        self.entries.get(declaration).map(|info| info.class_object_type)
    }

    pub fn kind(&self, declaration: &DeclarationId) -> Option<KindId> {
        self.entries.get(declaration).map(|info| info.kind)
    }

    pub fn generic_signature(&self, declaration: &DeclarationId) -> Option<&GenericSignature> {
        self.entries.get(declaration).and_then(|info| info.generic_signature.as_ref())
    }

    pub fn supertype_template(&self, declaration: &DeclarationId) -> Option<&GenericSupertypeTemplate> {
        self.entries.get(declaration).and_then(|info| info.supertype_template.as_ref())
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
            let param_kinds: Vec<KindId> = parameters.iter().map(|p| lower_kind_spec(store, p)).collect();
            let res_kind = lower_kind_spec(store, result);
            store.arrow_kind(param_kinds.into_boxed_slice(), res_kind)
        }
    }
}

/// Bootstraps canonical declaration type forms for all core universe classes.
pub fn bootstrap_universe_declarations(store: &mut TypeStore, universe_resolver: &dyn Fn(UniverseKey) -> DeclarationId) -> DeclarationTypeTable {
    let mut table = DeclarationTypeTable::new();

    let mut generic_specs: HashMap<UniverseKey, &UniverseTypeFormSpec> = HashMap::new();
    for spec in UNIVERSE_TYPE_FORMS {
        generic_specs.insert(spec.owner, spec);
    }

    for binding in UNIVERSE_BINDINGS {
        if binding.kind == phalcom_native_meta::universe::UniverseBindingKind::RuntimeSupportClass {
            continue;
        }

        let key = binding.key;
        let decl = universe_resolver(key);

        if let Some(spec) = generic_specs.get(&key) {
            let mut param_ids = Vec::new();
            let mut param_kinds = Vec::new();

            for (idx, p) in spec.parameters.iter().enumerate() {
                let p_kind = lower_kind_spec(store, &p.kind);
                let param_id = store.intern_type_parameter(TypeParameterData::new(
                    TypeParameterOwner::Declaration(decl.clone()),
                    idx as u32,
                    p.name,
                    p_kind,
                ));
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
                generic_signature: Some(GenericSignature::new(TypeParameterOwner::Declaration(decl), param_ids.into_boxed_slice())),
                supertype_template: None,
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
                supertype_template: None,
            });
        }
    }

    table
}
