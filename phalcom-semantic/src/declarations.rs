//! Canonical declaration type forms and metadata table.

use crate::identity::DeclarationId;
use crate::type_alias::TypeAliasInfo;
use crate::types::annotation::{KindResolution, resolve_kind_syntax};
use crate::types::id::{KindId, TypeId};
use crate::types::parameter::{GenericBinderHeader, GenericParameterMode, GenericSignature, TypeParameterData, TypeParameterOwner};
use crate::types::store::TypeStore;
pub use crate::types::type_member::IntrinsicTypeOwner;
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
    /// Bare/source declaration identity used by existing nominal consumers.
    pub form: TypeId,
    pub class_object_type: TypeId,
    pub kind: KindId,
    /// Semantic application behavior and runtime/native ownership are sibling
    /// projections. Intrinsic application never has to infer its behavior from
    /// the runtime owner selected by native import.
    pub binding: DeclarationTypeBinding,
    pub generic_binder_header: Option<GenericBinderHeader>,
    pub generic_signature: Option<GenericSignature>,
    pub supertype_template: Option<GenericSupertypeTemplate>,
}

/// Semantic type-application behavior for a canonical declaration.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DeclarationSemanticForm {
    /// Use the declaration's ordinary TypeId form and existing application rules.
    #[default]
    Ordinary,
    /// Use the closed intrinsic structural application owned by this declaration.
    Intrinsic(IntrinsicTypeApplication),
}

/// Compiler-owned application behavior exposed through an intrinsic declaration.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IntrinsicTypeApplication {
    /// A complete type pack applied through the existing TupleFromPacks family.
    Tuple,
    /// A RecordRow identity applied through the existing Record representation.
    Record,
}

/// Existing runtime authority projected by an intrinsic structural data declaration.
/// The declaration ID is only the authored source anchor; VM classes and structural
/// TypeStore values remain owned by their existing subsystems.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IntrinsicRuntimeOwner {
    Tuple,
    Record,
}

/// The two projections retained by one declaration type record.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct DeclarationTypeBinding {
    /// Semantic application behavior. The base TypeId remains in `DeclarationTypeInfo::form`.
    pub semantic_form: DeclarationSemanticForm,
    /// Existing VM/native owner, when this declaration has one.
    pub runtime_owner: Option<IntrinsicRuntimeOwner>,
}

impl DeclarationTypeBinding {
    /// Ordinary declaration binding with no special application or runtime projection.
    pub const fn ordinary() -> Self {
        Self {
            semantic_form: DeclarationSemanticForm::Ordinary,
            runtime_owner: None,
        }
    }
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
        self.into_type_info_with_binding(supertype_template, DeclarationTypeBinding::ordinary())
    }

    pub fn into_type_info_with_binding(
        self,
        supertype_template: Option<GenericSupertypeTemplate>,
        binding: DeclarationTypeBinding,
    ) -> DeclarationTypeInfo {
        DeclarationTypeInfo {
            declaration: self.declaration,
            form: self.form,
            class_object_type: self.class_object_type,
            kind: self.kind,
            binding,
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

    /// Replaces the complete declaration product and all independently
    /// registered projections for its identity.
    pub fn replace_authoritative(&mut self, info: DeclarationTypeInfo) {
        self.store_authoritative(info);
    }

    /// Compatibility alias for `replace_authoritative`.
    #[inline]
    pub fn insert(&mut self, info: DeclarationTypeInfo) {
        self.replace_authoritative(info);
    }

    /// Refreshes a declaration shell while preserving already registered
    /// intrinsic semantic/runtime projections when the new shell contains only
    /// their ordinary defaults.
    pub fn refresh_shell_preserving_registered_projections(&mut self, mut info: DeclarationTypeInfo) {
        let static_domain = self.intrinsic_static_domains.get(&info.declaration).copied();
        let type_owner = self.intrinsic_type_owners.get(&info.declaration).copied();
        if let Some(existing) = self.entries.get(&info.declaration) {
            if info.binding.semantic_form == DeclarationSemanticForm::Ordinary {
                info.binding.semantic_form = existing.binding.semantic_form;
            }
            if info.binding.runtime_owner.is_none() {
                info.binding.runtime_owner = existing.binding.runtime_owner;
            }
        }

        let preserve_type_owner = info.binding.runtime_owner.is_none().then_some(type_owner).flatten();
        let declaration = info.declaration.clone();
        self.store_authoritative(info);
        if let Some(kind) = static_domain {
            self.intrinsic_static_domains.insert(declaration.clone(), kind);
        }
        if let Some(owner) = preserve_type_owner {
            self.intrinsic_type_owners.insert(declaration, owner);
        }
    }

    fn store_authoritative(&mut self, info: DeclarationTypeInfo) {
        self.intrinsic_static_domains.remove(&info.declaration);
        self.intrinsic_type_owners.remove(&info.declaration);
        if let Some(owner) = info.binding.runtime_owner {
            let type_owner = match owner {
                IntrinsicRuntimeOwner::Tuple => Some(IntrinsicTypeOwner::Tuple),
                IntrinsicRuntimeOwner::Record => Some(IntrinsicTypeOwner::Record),
            };
            if let Some(type_owner) = type_owner {
                self.intrinsic_type_owners.insert(info.declaration.clone(), type_owner);
            }
        }
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

    /// Registers an authored runtime facade against its existing platform owner.
    /// This association does not allocate a nominal data type or runtime class.
    /// Records only the declaration's semantic type-application behavior.
    pub fn register_intrinsic_semantic_form(&mut self, declaration: &DeclarationId, semantic_form: DeclarationSemanticForm) -> bool {
        let Some(info) = self.entries.get_mut(declaration) else {
            return false;
        };
        info.binding.semantic_form = semantic_form;
        self.intrinsic_static_domains.remove(declaration);
        true
    }

    /// Adds the existing runtime/native owner without changing semantic form.
    pub fn register_intrinsic_runtime_owner(&mut self, declaration: &DeclarationId, owner: IntrinsicRuntimeOwner) -> bool {
        let Some(info) = self.entries.get_mut(declaration) else {
            return false;
        };
        info.binding.runtime_owner = Some(owner);
        let type_owner = match owner {
            IntrinsicRuntimeOwner::Tuple => Some(IntrinsicTypeOwner::Tuple),
            IntrinsicRuntimeOwner::Record => Some(IntrinsicTypeOwner::Record),
        };
        if let Some(type_owner) = type_owner {
            self.intrinsic_type_owners.insert(declaration.clone(), type_owner);
        }
        true
    }

    pub fn intrinsic_runtime_owner(&self, declaration: &DeclarationId) -> Option<IntrinsicRuntimeOwner> {
        self.entries.get(declaration).and_then(|info| info.binding.runtime_owner)
    }

    pub fn intrinsic_runtime_declarations(&self) -> impl Iterator<Item = (&DeclarationId, IntrinsicRuntimeOwner)> {
        self.entries
            .iter()
            .filter_map(|(declaration, info)| info.binding.runtime_owner.map(|owner| (declaration, owner)))
    }

    pub fn semantic_form(&self, declaration: &DeclarationId) -> Option<DeclarationSemanticForm> {
        self.entries.get(declaration).map(|info| info.binding.semantic_form)
    }

    pub fn intrinsic_type_owner(&self, declaration: &DeclarationId) -> Option<IntrinsicTypeOwner> {
        self.intrinsic_type_owners.get(declaration).copied()
    }

    pub fn intrinsic_type_owner_declaration(&self, owner: IntrinsicTypeOwner) -> Option<DeclarationId> {
        self.intrinsic_type_owners
            .iter()
            .find_map(|(declaration, candidate)| (*candidate == owner).then(|| declaration.clone()))
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

    /// Returns the proper runtime class projection used by native surface import.
    /// Source type formation may use the same declaration's structural form.
    pub fn runtime_form(&self, store: &mut TypeStore, declaration: &DeclarationId) -> Option<TypeId> {
        self.get(declaration)?;
        self.intrinsic_runtime_owner(declaration)
            .map(|_| store.nominal_type(declaration.clone()))
            .or_else(|| self.form(declaration))
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
    if let Some(syntax) = parameter.kind.as_ref() {
        match resolve_kind_syntax(store, syntax) {
            KindResolution::Ready(kind) => return kind,
            other => panic!(
                "canonical Universe generic parameter {} has non-ready kind during bootstrap: {other:?}",
                parameter.name
            ),
        }
    }
    match parameter.mode {
        Some(ExpansionMode::Positional) => KindId::TYPE_PACK,
        // `**` defaults to RecordFields. LabeledTypePack remains available as
        // an explicit kind annotation under the same expansion mode.
        Some(ExpansionMode::Labeled) => KindId::RECORD_ROW,
        Some(ExpansionMode::Complete) => KindId::COMPLETE_TYPE_PACK,
        None => KindId::TYPE,
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

    table.replace_authoritative(DeclarationTypeInfo {
        class_object_type: store.class_object_type(declaration.clone()),
        declaration,
        form,
        kind,
        binding: DeclarationTypeBinding::ordinary(),
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
                    insert_source_nominal(&mut table, store, declaration.clone(), &class_def.generic_parameters);
                    if let Some(binding) = intrinsic_binding_for_source(&module, &class_def.name) {
                        table.register_intrinsic_semantic_form(&declaration, binding.semantic_form);
                        if let Some(owner) = binding.runtime_owner {
                            table.register_intrinsic_runtime_owner(&declaration, owner);
                        }
                    }
                    let tuple_module = source_module_id(&["collections", "tuple"]);
                    let record_module = source_module_id(&["collections", "record"]);
                    match (module.eq(&tuple_module), module.eq(&record_module), class_def.name.as_str()) {
                        (true, _, "Tuple") => table.register_intrinsic_type_owner(declaration.clone(), IntrinsicTypeOwner::Tuple),
                        (_, true, "Record") => table.register_intrinsic_type_owner(declaration.clone(), IntrinsicTypeOwner::Record),
                        _ => {}
                    }
                }
                Statement::Data(data_def) => {
                    let declaration = source_declaration_identity(&module, &data_def.name, universe_resolver);
                    if data_def.implementation == phalcom_common::modifier::ImplementationKind::Intrinsic {
                        if data_def.phase == phalcom_ast::ast::AlgebraicPhase::TypeOnly {
                            if let Some(kind) = intrinsic_static_facade_kind(&module, &data_def.name) {
                                table.register_intrinsic_static_domain(declaration, kind);
                            }
                        } else if let Some(binding) = intrinsic_binding_for_source(&module, &data_def.name) {
                            source_nominals.insert((module.clone(), data_def.name.clone()));
                            insert_source_nominal(&mut table, store, declaration.clone(), &data_def.generic_parameters);
                            let semantic_registered = table.register_intrinsic_semantic_form(&declaration, binding.semantic_form);
                            let owner_registered = binding
                                .runtime_owner
                                .is_some_and(|owner| table.register_intrinsic_runtime_owner(&declaration, owner));
                            debug_assert!(semantic_registered && owner_registered, "intrinsic data shell projections were just registered");
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
                source_nominals.contains(&(source_owner.clone(), binding.key.name().to_string()))
                    || table.intrinsic_runtime_owner(&universe_resolver(binding.key)).is_some(),
                "ordinary native Universe binding {:?} has no canonical source declaration at {}::{}",
                binding.key,
                source_owner,
                binding.key.name()
            );
        }
    }

    table
}

pub(crate) fn intrinsic_binding_for_source(module: &ModuleId, name: &str) -> Option<DeclarationTypeBinding> {
    let tuple_module = source_module_id(&["collections", "tuple"]);
    let record_module = source_module_id(&["collections", "record"]);
    match (module, name) {
        (module, "Tuple") if module == &tuple_module => Some(DeclarationTypeBinding {
            semantic_form: DeclarationSemanticForm::Intrinsic(IntrinsicTypeApplication::Tuple),
            runtime_owner: Some(IntrinsicRuntimeOwner::Tuple),
        }),
        (module, "Record") if module == &record_module => Some(DeclarationTypeBinding {
            semantic_form: DeclarationSemanticForm::Intrinsic(IntrinsicTypeApplication::Record),
            runtime_owner: Some(IntrinsicRuntimeOwner::Record),
        }),
        _ => None,
    }
}

fn intrinsic_static_facade_kind(module: &ModuleId, name: &str) -> Option<KindId> {
    let tuple_module = source_module_id(&["collections", "tuple"]);
    let record_module = source_module_id(&["collections", "record"]);
    match (module, name) {
        (module, "TypePack") if module == &tuple_module => Some(KindId::TYPE_PACK),
        (module, "LabeledTypePack") if module == &tuple_module => Some(KindId::LABELED_TYPE_PACK),
        (module, "CompleteTypePack") if module == &tuple_module => Some(KindId::COMPLETE_TYPE_PACK),
        (module, "RecordFields") if module == &record_module => Some(KindId::RECORD_ROW),
        _ => None,
    }
}
