//! Canonical source-visible prelude type policy.

use crate::identity::DeclarationId;
use phalcom_modules::builtin::UniverseSourceProvider;
use phalcom_modules::builtin_interface::UniverseSourceDeclarationCatalog;
use phalcom_modules::{ModuleComponent, ModuleId, ModulePath};
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

static CANONICAL_UNIVERSE_PRELUDE: OnceLock<Arc<PreludeTypeMap>> = OnceLock::new();

/// Canonical source-visible type names supplied implicitly by the Phalcom
/// prelude.
///
/// Prelude membership comes from explicit `UNIVERSE_BINDINGS.prelude` policy,
/// plus the canonical source-backed Unit alias exception. Both are represented
/// only in this `PreludeTypeMap`; no separate alias catalog or synthetic
/// root/prelude declaration is manufactured.
#[derive(Clone, Debug, Default)]
pub struct PreludeTypeMap {
    entries: BTreeMap<Box<str>, DeclarationId>,
}

impl PreludeTypeMap {
    /// Builds the canonical prelude from explicit native policy plus canonical
    /// source identity.
    pub fn canonical_universe() -> Self {
        if let Some(prelude) = CANONICAL_UNIVERSE_PRELUDE.get() {
            return (**prelude).clone();
        }
        Self::build_source_canonical_universe()
    }

    /// Directly builds the canonical prelude by parsing Universe source files,
    /// bypassing the cached `OnceLock`.
    pub fn build_source_canonical_universe() -> Self {
        let provider = UniverseSourceProvider::new();
        let catalog = UniverseSourceDeclarationCatalog::build(&provider).expect("canonical Universe source declaration catalog must build");
        let mut entries = BTreeMap::new();

        for binding in phalcom_native_meta::UNIVERSE_BINDINGS.iter().filter(|binding| binding.prelude) {
            // A prelude flag alone is not declaration authority. This lookup
            // rejects stale/native-only rows while deliberately admitting
            // source-authored declarations such as Unit even if older native
            // metadata still classifies their runtime attachment specially.
            let Ok((module, name)) = catalog.declaration_for(binding.key) else {
                continue;
            };
            entries.insert(binding.name.into(), DeclarationId::new(module, name.into()));
        }

        // Unit is the source alias `Tuple<>`, not a runtime-support class. Keep
        // its implicit source name visible through this canonical catalog.
        let unit_module = ModuleId::universe(ModulePath::from_components(vec![
            ModuleComponent::from_identifier("option").expect("canonical Universe component"),
            ModuleComponent::from_identifier("unit").expect("canonical Universe component"),
        ]));
        assert!(
            catalog.contains_declaration(&unit_module, "Unit"),
            "canonical Unit alias must exist in Universe source"
        );
        entries.insert("Unit".into(), DeclarationId::new(unit_module, "Unit".into()));

        let duration_module = ModuleId::universe(ModulePath::from_components(vec![
            ModuleComponent::from_identifier("time").expect("canonical Universe component"),
            ModuleComponent::from_identifier("duration").expect("canonical Universe component"),
        ]));
        entries.insert("Duration".into(), DeclarationId::new(duration_module, "Duration".into()));
        let timestamp_module = ModuleId::universe(ModulePath::from_components(vec![
            ModuleComponent::from_identifier("time").expect("canonical Universe component"),
            ModuleComponent::from_identifier("timestamp").expect("canonical Universe component"),
        ]));
        entries.insert("Timestamp".into(), DeclarationId::new(timestamp_module, "Timestamp".into()));

        let duplicate_key_error_module = ModuleId::universe(ModulePath::from_components(vec![
            ModuleComponent::from_identifier("errors").expect("canonical Universe component"),
            ModuleComponent::from_identifier("argument").expect("canonical Universe component"),
        ]));
        entries.insert(
            "DuplicateKeyError".into(),
            DeclarationId::new(duplicate_key_error_module, "DuplicateKeyError".into()),
        );

        let proof_module = crate::core_surface::universe_module(&["core", "proof"]);
        entries.insert("Equivalent".into(), DeclarationId::new(proof_module.clone(), "Equivalent".into()));
        entries.insert("Subtype".into(), DeclarationId::new(proof_module.clone(), "Subtype".into()));
        entries.insert("Conforms".into(), DeclarationId::new(proof_module, "Conforms".into()));

        Self { entries }
    }

    /// Builds the canonical prelude by validating candidate declarations against a predicate.
    pub fn from_declarations<F>(mut contains_decl: F) -> Self
    where
        F: FnMut(&DeclarationId) -> bool,
    {
        let mut entries = BTreeMap::new();

        for binding in phalcom_native_meta::UNIVERSE_BINDINGS.iter().filter(|binding| binding.prelude) {
            let path: Vec<crate::identity::ModuleComponent> = binding
                .key
                .source_path()
                .iter()
                .map(|component| crate::identity::ModuleComponent::from_identifier(component).expect("canonical Universe component"))
                .collect();
            let module = ModuleId::universe(crate::identity::ModulePath::from_components(path));
            let name = binding.key.name();
            let decl = DeclarationId::new(module, name.into());
            if contains_decl(&decl) {
                entries.insert(binding.name.into(), decl);
            }
        }

        let exceptional_entries = [
            (
                "Unit",
                DeclarationId::new(
                    ModuleId::universe(ModulePath::from_components(vec![
                        ModuleComponent::from_identifier("option").expect("canonical Universe component"),
                        ModuleComponent::from_identifier("unit").expect("canonical Universe component"),
                    ])),
                    "Unit".into(),
                ),
            ),
            (
                "Duration",
                DeclarationId::new(
                    ModuleId::universe(ModulePath::from_components(vec![
                        ModuleComponent::from_identifier("time").expect("canonical Universe component"),
                        ModuleComponent::from_identifier("duration").expect("canonical Universe component"),
                    ])),
                    "Duration".into(),
                ),
            ),
            (
                "Timestamp",
                DeclarationId::new(
                    ModuleId::universe(ModulePath::from_components(vec![
                        ModuleComponent::from_identifier("time").expect("canonical Universe component"),
                        ModuleComponent::from_identifier("timestamp").expect("canonical Universe component"),
                    ])),
                    "Timestamp".into(),
                ),
            ),
            (
                "DuplicateKeyError",
                DeclarationId::new(
                    ModuleId::universe(ModulePath::from_components(vec![
                        ModuleComponent::from_identifier("errors").expect("canonical Universe component"),
                        ModuleComponent::from_identifier("argument").expect("canonical Universe component"),
                    ])),
                    "DuplicateKeyError".into(),
                ),
            ),
            (
                "Equivalent",
                DeclarationId::new(crate::core_surface::universe_module(&["core", "proof"]), "Equivalent".into()),
            ),
            (
                "Subtype",
                DeclarationId::new(crate::core_surface::universe_module(&["core", "proof"]), "Subtype".into()),
            ),
            (
                "Conforms",
                DeclarationId::new(crate::core_surface::universe_module(&["core", "proof"]), "Conforms".into()),
            ),
        ];

        for (name, decl) in &exceptional_entries {
            if !contains_decl(decl) {
                panic!("Mandatory prelude declaration {:?} ({:?}) not found in semantic authority", name, decl);
            }
            entries.insert((*name).into(), decl.clone());
        }

        Self { entries }
    }

    /// Initializes the process-stable canonical prelude map from an authoritative snapshot.
    pub fn initialize_from_snapshot(snapshot: &crate::snapshot::SemanticSnapshot) {
        let snapshot_prelude = Arc::new(Self::from_declarations(|decl| {
            snapshot.declarations.contains(decl)
                || snapshot.type_aliases.contains_key(decl)
                || snapshot.trait_headers.contains(decl)
                || crate::checker::proof_predicate::is_proof_predicate_declaration(decl).is_some()
        }));
        let existing = CANONICAL_UNIVERSE_PRELUDE.get_or_init(|| snapshot_prelude.clone());
        if existing.entries != snapshot_prelude.entries {
            panic!(
                "PreludeTypeMap: incompatible initialization — \
                 entries_only_in_global={:?}, entries_only_in_snapshot={:?}",
                existing
                    .entries
                    .keys()
                    .filter(|k| !snapshot_prelude.entries.contains_key(*k))
                    .collect::<Vec<_>>(),
                snapshot_prelude
                    .entries
                    .keys()
                    .filter(|k| !existing.entries.contains_key(*k))
                    .collect::<Vec<_>>(),
            );
        }
    }

    /// Returns the single process-stable canonical prelude map.
    ///
    /// The map contains only stable source declaration identities; it carries
    /// no store-local `TypeId` or workspace mutation state, so sharing it is
    /// safe and prevents resolver/editor policy drift.
    pub fn shared_canonical_universe() -> Arc<Self> {
        CANONICAL_UNIVERSE_PRELUDE.get_or_init(|| Arc::new(Self::canonical_universe())).clone()
    }

    pub fn entries(&self) -> &BTreeMap<Box<str>, DeclarationId> {
        &self.entries
    }

    pub fn get(&self, name: &str) -> Option<&DeclarationId> {
        self.entries.get(name)
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &DeclarationId)> {
        self.entries.iter().map(|(name, declaration)| (name.as_ref(), declaration))
    }
}
