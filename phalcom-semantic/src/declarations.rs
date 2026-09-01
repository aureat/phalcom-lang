//! Canonical declaration type forms and metadata table.

use crate::identity::DeclarationId;
use crate::type_alias::TypeAliasInfo;
use crate::types::annotation::{KindResolution, resolve_kind_syntax};
use crate::types::id::{KindId, TypeId};
use crate::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner};
use crate::types::store::TypeStore;
use phalcom_ast::ast::{GenericParameterSyntax, Statement};
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

fn source_module_id(path: &[&str]) -> ModuleId {
    let components = path
        .iter()
        .map(|component| ModuleComponent::from_identifier(component).expect("canonical Universe component"))
        .collect::<Vec<_>>();
    ModuleId::universe(ModulePath::from_components(components))
}

fn source_parameter_kind(store: &mut TypeStore, parameter: &GenericParameterSyntax) -> KindId {
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

    for (index, parameter) in parameters.iter().enumerate() {
        let kind = source_parameter_kind(store, parameter);
        let parameter_id = store.intern_type_parameter(TypeParameterData::new(
            TypeParameterOwner::Declaration(declaration.clone()),
            index as u32,
            parameter.name.clone(),
            kind,
        ));
        parameter_ids.push(parameter_id);
        parameter_kinds.push(kind);
    }

    let (form, kind, generic_signature) = if parameter_ids.is_empty() {
        (store.nominal_type(declaration.clone()), KindId::TYPE, None)
    } else {
        let kind = store.arrow_kind(parameter_kinds.into_boxed_slice(), KindId::TYPE);
        let form = store.nominal_form(declaration.clone(), kind);
        let signature = GenericSignature::new(TypeParameterOwner::Declaration(declaration.clone()), parameter_ids.into_boxed_slice());
        (form, kind, Some(signature))
    };

    table.insert(DeclarationTypeInfo {
        class_object_type: store.class_object_type(declaration.clone()),
        declaration,
        form,
        kind,
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
                    insert_source_nominal(&mut table, store, declaration, &class_def.generic_parameters);
                }
                Statement::Enum(enum_def) => {
                    source_nominals.insert((module.clone(), enum_def.name.clone()));
                    let declaration = source_declaration_identity(&module, &enum_def.name, universe_resolver);
                    insert_source_nominal(&mut table, store, declaration, &enum_def.generic_parameters);
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
