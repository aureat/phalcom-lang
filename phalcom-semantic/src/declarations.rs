//! Canonical declaration type forms and metadata table.

use crate::identity::DeclarationId;
use crate::type_alias::TypeAliasInfo;
use crate::types::annotation::{KindResolution, resolve_kind_syntax};
use crate::types::id::{KindId, TypeId};
use crate::types::parameter::{GenericBinderHeader, GenericParameterMode, GenericSignature, TypeParameterData, TypeParameterOwner};
use crate::types::store::TypeStore;
use crate::types::type_member::IntrinsicTypeOwner;
use phalcom_ast::ast::{ExpansionMode, GenericParameterSyntax, Statement};
use phalcom_modules::{ModuleComponent, ModuleId, ModulePath, UniverseSourceProvider};
use phalcom_native_meta::types::KindSpec;
use phalcom_native_meta::universe::{UNIVERSE_BINDINGS, UniverseBindingKind, UniverseKey};
use std::collections::{HashMap, HashSet};

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
    pub generic_binder_header: Option<GenericBinderHeader>,
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
    pub generic_binder_header: Option<GenericBinderHeader>,
    pub generic_signature: Option<GenericSignature>,
}

impl NominalDeclarationHeader {
    pub fn from_signature(store: &mut TypeStore, declaration: DeclarationId, generic_signature: Option<GenericSignature>) -> Self {
        Self::from_signature_with_result_kind(store, declaration, generic_signature, KindId::TYPE)
    }

    pub fn from_signature_with_result_kind(
        store: &mut TypeStore,
        declaration: DeclarationId,
        generic_signature: Option<GenericSignature>,
        result_kind: KindId,
    ) -> Self {
        let generic_binder_header = generic_signature
            .as_ref()
            .map(|signature| GenericBinderHeader::from_signature(store, signature));
        let kind = generic_signature.as_ref().map_or(KindId::TYPE, |signature| {
            let parameter_kinds = signature
                .parameters
                .iter()
                .map(|&parameter| store.type_parameter(parameter).kind)
                .collect::<Vec<_>>();
            store.arrow_kind(parameter_kinds.into_boxed_slice(), result_kind)
        });
        let kind = if generic_signature.is_some() { kind } else { result_kind };
        let form = store.nominal_form(declaration.clone(), kind);
        let class_object_type = store.class_object_type(declaration.clone());
        Self {
            declaration,
            form,
            class_object_type,
            kind,
            generic_binder_header,
            generic_signature,
        }
    }

    pub fn into_type_info(self, supertype_template: Option<GenericSupertypeTemplate>) -> DeclarationTypeInfo {
        DeclarationTypeInfo {
            declaration: self.declaration,
            form: self.form,
            class_object_type: self.class_object_type,
            kind: self.kind,
            generic_binder_header: self.generic_binder_header,
            generic_signature: self.generic_signature,
            supertype_template,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DeclarationTypeTable {
    entries: HashMap<DeclarationId, DeclarationTypeInfo>,
    intrinsic_static_domains: HashMap<DeclarationId, KindId>,
    intrinsic_type_owners: HashMap<DeclarationId, IntrinsicTypeOwner>,
}

impl DeclarationTypeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, info: DeclarationTypeInfo) {
        self.intrinsic_static_domains.remove(&info.declaration);
        self.intrinsic_type_owners.remove(&info.declaration);
        self.entries.insert(info.declaration.clone(), info);
    }

    /// Registers an authored intrinsic facade against an existing compiler-owned kind.
    /// This deliberately does not create a type form or nominal static domain.
    pub fn register_intrinsic_static_domain(&mut self, declaration: DeclarationId, kind: KindId) {
        self.entries.remove(&declaration);
        let owner = match kind {
            KindId::TYPE_PACK => Some(IntrinsicTypeOwner::TypePack),
            KindId::LABELED_TYPE_PACK => Some(IntrinsicTypeOwner::LabeledTypePack),
            KindId::COMPLETE_TYPE_PACK => Some(IntrinsicTypeOwner::CompleteTypePack),
            KindId::RECORD_ROW => Some(IntrinsicTypeOwner::RecordRow),
            _ => None,
        };
        if let Some(owner) = owner {
            self.intrinsic_type_owners.insert(declaration.clone(), owner);
        }
        self.intrinsic_static_domains.insert(declaration, kind);
    }

    pub fn register_intrinsic_type_owner(&mut self, declaration: DeclarationId, owner: IntrinsicTypeOwner) {
        self.intrinsic_type_owners.insert(declaration, owner);
    }

    pub fn intrinsic_type_owner(&self, declaration: &DeclarationId) -> Option<IntrinsicTypeOwner> {
        self.intrinsic_type_owners.get(declaration).copied()
    }

    pub fn intrinsic_static_domain_kind(&self, declaration: &DeclarationId) -> Option<KindId> {
        self.intrinsic_static_domains.get(declaration).copied()
    }

    pub fn intrinsic_static_domain_declarations(&self) -> impl Iterator<Item = &DeclarationId> {
        self.intrinsic_static_domains.keys()
    }

    pub fn is_intrinsic_static_domain(&self, declaration: &DeclarationId) -> bool {
        self.intrinsic_static_domains.contains_key(declaration)
    }

    pub fn remove(&mut self, declaration: &DeclarationId) -> Option<DeclarationTypeInfo> {
        self.intrinsic_static_domains.remove(declaration);
        self.intrinsic_type_owners.remove(declaration);
        self.entries.remove(declaration)
    }

    pub fn get(&self, declaration: &DeclarationId) -> Option<&DeclarationTypeInfo> {
        self.entries.get(declaration)
    }

    pub fn contains(&self, declaration: &DeclarationId) -> bool {
        self.entries.contains_key(declaration)
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

    pub fn generic_binder_header(&self, declaration: &DeclarationId) -> Option<&GenericBinderHeader> {
        self.entries.get(declaration).and_then(|info| info.generic_binder_header.as_ref())
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

fn source_module_id(path: &[&str]) -> ModuleId {
    let components = path
        .iter()
        .map(|component| ModuleComponent::from_identifier(component).expect("canonical Universe component"))
        .collect::<Vec<_>>();
    ModuleId::universe(ModulePath::from_components(components))
}

fn source_parameter_kind(store: &mut TypeStore, parameter: &GenericParameterSyntax) -> KindId {
    match parameter.mode {
        Some(ExpansionMode::Positional) => return KindId::TYPE_PACK,
        Some(ExpansionMode::Labeled) => return KindId::LABELED_TYPE_PACK,
        Some(ExpansionMode::Complete) => return KindId::COMPLETE_TYPE_PACK,
        None => {}
    }
    match parameter
        .kind
        .as_ref()
        .map_or(KindResolution::Ready(KindId::TYPE), |syntax| resolve_kind_syntax(store, syntax))
    {
        KindResolution::Ready(kind) => kind,
        other => panic!(
            "canonical Universe generic parameter {} has non-ready kind during bootstrap: {other:?}",
            parameter.name
        ),
    }
}

fn insert_source_nominal(table: &mut DeclarationTypeTable, store: &mut TypeStore, declaration: DeclarationId, parameters: &[GenericParameterSyntax]) {
    let mut parameter_ids = Vec::with_capacity(parameters.len());
    let mut parameter_kinds = Vec::with_capacity(parameters.len());

    let mut parameter_modes = Vec::with_capacity(parameters.len());
    for (index, parameter) in parameters.iter().enumerate() {
        let kind = source_parameter_kind(store, parameter);
        let mode = match parameter.mode {
            None => GenericParameterMode::Single,
            Some(ExpansionMode::Positional) => GenericParameterMode::PositionalPack,
            Some(ExpansionMode::Labeled) => GenericParameterMode::LabeledPack,
            Some(ExpansionMode::Complete) => GenericParameterMode::CompletePack,
        };
        let parameter_id = store.intern_type_parameter(
            TypeParameterData::new(TypeParameterOwner::Declaration(declaration.clone()), index as u32, parameter.name.clone(), kind).with_mode(mode),
        );
        parameter_ids.push(parameter_id);
        parameter_kinds.push(kind);
        parameter_modes.push(mode);
    }

    let (form, kind, generic_signature) = if parameter_ids.is_empty() {
        (store.nominal_type(declaration.clone()), KindId::TYPE, None)
    } else {
        let kind = store.arrow_kind(parameter_kinds.into_boxed_slice(), KindId::TYPE);
        let form = store.nominal_form(declaration.clone(), kind);
        let signature = GenericSignature::new(TypeParameterOwner::Declaration(declaration.clone()), parameter_ids.into_boxed_slice())
            .with_parameter_modes(parameter_modes.into_boxed_slice());
        (form, kind, Some(signature))
    };

    table.insert(DeclarationTypeInfo {
        class_object_type: store.class_object_type(declaration.clone()),
        declaration,
        form,
        kind,
        generic_binder_header: generic_signature
            .as_ref()
            .map(|signature| GenericBinderHeader::from_signature(store, signature)),
        generic_signature,
        supertype_template: None,
    });
}

fn source_declaration_identity(module: &ModuleId, name: &str, universe_resolver: &dyn Fn(UniverseKey) -> DeclarationId) -> DeclarationId {
    if let Some(key) = UniverseKey::from_name(name) {
        if source_module_id(key.source_path()) == *module {
            return universe_resolver(key);
        }
    }
    DeclarationId::new(module.clone(), name.into())
}

/// Bootstraps canonical declaration type forms from authoritative Universe source.
///
/// Source owns declaration existence and generic shape. `universe_resolver` remains
/// an identity-injection seam for isolated tests; production passes the canonical
/// source-aware resolver. Native metadata is attachment/conformance data only.
pub fn bootstrap_universe_declarations(store: &mut TypeStore, universe_resolver: &dyn Fn(UniverseKey) -> DeclarationId) -> DeclarationTypeTable {
    let provider = UniverseSourceProvider::new();
    let mut table = DeclarationTypeTable::new();
    let mut source_nominals = HashSet::<(ModuleId, String)>::new();

    for node in provider.nodes() {
        let module = source_module_id(node.path);
        let parsed = provider
            .load_parsed(&module)
            .unwrap_or_else(|error| panic!("failed to load canonical Universe source module {module}: {error}"));

        for statement in &parsed.program.statements {
            match statement {
                Statement::Class(class_def) => {
                    source_nominals.insert((module.clone(), class_def.name.clone()));
                    let declaration = source_declaration_identity(&module, &class_def.name, universe_resolver);
                    let tuple_module = source_module_id(&["collections", "tuple"]);
                    let record_module = source_module_id(&["collections", "record"]);
                    match (module.eq(&tuple_module), module.eq(&record_module), class_def.name.as_str()) {
                        (true, _, "Tuple") => table.register_intrinsic_type_owner(declaration.clone(), IntrinsicTypeOwner::Tuple),
                        (_, true, "Record") => table.register_intrinsic_type_owner(declaration.clone(), IntrinsicTypeOwner::Record),
                        _ => {}
                    }
                    insert_source_nominal(&mut table, store, declaration, &class_def.generic_parameters);
                }
                Statement::Data(data_def) => {
                    let declaration = source_declaration_identity(&module, &data_def.name, universe_resolver);
                    if data_def.implementation == phalcom_common::modifier::ImplementationKind::Intrinsic
                        && matches!(&data_def.shape, phalcom_ast::ast::AlgebraicShapeSyntax::IntrinsicFacade(_))
                    {
                        if let Some(kind) = intrinsic_static_facade_kind(&module, &data_def.name) {
                            table.register_intrinsic_static_domain(declaration, kind);
                        }
                        continue;
                    }
                    source_nominals.insert((module.clone(), data_def.name.clone()));
                    insert_source_nominal(&mut table, store, declaration, &data_def.generic_parameters);
                }
                _ => {}
            }
        }
    }

    // Ordinary native rows must attach to real source declarations. Runtime-support
    // rows may be source-less; when a real declaration exists (for example Unit),
    // the source scan above still creates its semantic shell.
    for binding in UNIVERSE_BINDINGS {
        if binding.kind == UniverseBindingKind::Class {
            let source_owner = source_module_id(binding.key.source_path());
            assert!(
                source_nominals.contains(&(source_owner.clone(), binding.key.name().to_string())),
                "ordinary native Universe binding {:?} has no canonical source declaration at {}::{}",
                binding.key,
                source_owner,
                binding.key.name()
            );
        }
    }

    table
}

fn intrinsic_static_facade_kind(module: &ModuleId, name: &str) -> Option<KindId> {
    let tuple_module = source_module_id(&["collections", "tuple"]);
    let record_module = source_module_id(&["collections", "record"]);
    match (module, name) {
        (module, "TypePack") if module == &tuple_module => Some(KindId::TYPE_PACK),
        (module, "LabeledTypePack") if module == &tuple_module => Some(KindId::LABELED_TYPE_PACK),
        (module, "TupleEntries") if module == &tuple_module => Some(KindId::COMPLETE_TYPE_PACK),
        (module, "RecordRow") if module == &record_module => Some(KindId::RECORD_ROW),
        _ => None,
    }
}
