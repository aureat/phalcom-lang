//! Canonical source-visible prelude type policy.

use crate::identity::DeclarationId;
use phalcom_modules::builtin::UniverseSourceProvider;
use phalcom_modules::builtin_interface::UniverseSourceDeclarationCatalog;
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

static CANONICAL_UNIVERSE_PRELUDE: OnceLock<Arc<PreludeTypeMap>> = OnceLock::new();

/// Canonical source-visible type names supplied implicitly by the Phalcom
/// prelude.
///
/// Prelude membership comes only from explicit `UNIVERSE_BINDINGS.prelude`
/// policy. Each entry is additionally proven to exist in the canonical
/// Universe source catalog and points directly at that source-owned
/// `DeclarationId`; no synthetic root/prelude declaration is manufactured.
#[derive(Clone, Debug, Default)]
pub struct PreludeTypeMap {
    entries: BTreeMap<Box<str>, DeclarationId>,
}

impl PreludeTypeMap {
    /// Builds the canonical prelude from explicit native policy plus canonical
    /// source identity.
    pub fn canonical_universe() -> Self {
        let provider = UniverseSourceProvider::new();
        let catalog = UniverseSourceDeclarationCatalog::build(&provider)
            .expect("canonical Universe source declaration catalog must build");
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

        Self { entries }
    }

    /// Returns the single process-stable canonical prelude map.
    ///
    /// The map contains only stable source declaration identities; it carries
    /// no store-local `TypeId` or workspace mutation state, so sharing it is
    /// safe and prevents resolver/editor policy drift.
    pub fn shared_canonical_universe() -> Arc<Self> {
        CANONICAL_UNIVERSE_PRELUDE
            .get_or_init(|| Arc::new(Self::canonical_universe()))
            .clone()
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
