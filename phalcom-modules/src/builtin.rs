//! Provider-backed builtin project interfaces and canonical virtual source identity.

use crate::error::{ModuleLoadError, ModuleResolutionError};
use crate::identity::{ModuleId, ModulePath, SourceId, universe_module_uri};
use crate::interface::UnlinkedModuleInterface;
use crate::source::ModuleKind;
#[cfg(feature = "test-support")]
use std::collections::BTreeMap;
#[cfg(feature = "test-support")]
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct UniverseNodeSpec {
    pub path: &'static [&'static str],
    pub kind: ModuleKind,
    #[allow(dead_code)]
    pub children: &'static [&'static str],
}

include!(concat!(env!("OUT_DIR"), "/universe_catalog.rs"));

/// Toolchain-owned source/interface authority for the Universe package.
#[derive(Clone, Debug, Default)]
pub struct UniverseSourceProvider {
    #[cfg(feature = "test-support")]
    test_source_overrides: Arc<BTreeMap<ModuleId, Arc<str>>>,
}

impl UniverseSourceProvider {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a test-only provider that overrides source text for one canonical
    /// Universe module while preserving its existing module/source identity.
    ///
    /// This is a fixture seam, not a production source customization API.
    #[cfg(feature = "test-support")]
    pub fn with_test_source_override(module: ModuleId, text: Arc<str>) -> Result<Self, ModuleLoadError> {
        let provider = Self {
            test_source_overrides: Arc::new(BTreeMap::from([(module.clone(), text)])),
        };
        provider.validate_id(&module)?;
        Ok(provider)
    }

    pub fn is_canonical(&self) -> bool {
        !self.has_test_source_overrides()
    }

    pub(crate) fn has_test_source_overrides(&self) -> bool {
        #[cfg(feature = "test-support")]
        {
            !self.test_source_overrides.is_empty()
        }
        #[cfg(not(feature = "test-support"))]
        {
            false
        }
    }

    /// Canonical virtual identity used by diagnostics/LSP for builtin documents.
    pub fn source_id(&self, id: &ModuleId) -> Result<SourceId, ModuleLoadError> {
        self.validate_id(id)?;
        let uri = universe_module_uri(id).ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("{id} is not a Universe module")))?;
        Ok(SourceId(uri.into()))
    }

    /// Loads immutable public interface of builtin node derived from canonical source.
    pub fn load_interface(&self, id: &ModuleId) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
        self.validate_id(id)?;
        crate::builtin_interface::BuiltinInterfaceBuilder::build(self, id)
    }

    /// Loads the parsed module unit for a builtin node.
    pub fn load_parsed(&self, id: &ModuleId) -> Result<std::sync::Arc<crate::source::ParsedModuleUnit>, ModuleLoadError> {
        self.validate_id(id)?;
        crate::builtin_interface::BuiltinInterfaceBuilder::load_parsed(self, id)
    }

    /// Returns the embedded source text for a canonical Universe module.
    pub fn source_text(&self, id: &ModuleId) -> Result<std::sync::Arc<str>, ModuleLoadError> {
        use std::sync::Arc;
        if std::env::var_os("PHALCOM_DENY_SOURCE_BOOTSTRAP").is_some() {
            panic!("UniverseSourceProvider::source_text attempted while source bootstrap is denied by PHALCOM_DENY_SOURCE_BOOTSTRAP");
        }
        self.validate_id(id)?;
        let _node = self
            .node_spec(&id.path)
            .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("Universe module {id} is not part of the Universe graph")))?;

        #[cfg(feature = "test-support")]
        if let Some(text) = self.test_source_overrides.get(id) {
            return Ok(text.clone());
        }

        let components = id.path.components();
        let content = embedded_source(&components.iter().map(|component| component.as_str()).collect::<Vec<_>>())
            .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("builtin source for {id} not found")))?;
        Ok(Arc::from(content))
    }

    /// Returns whether a canonical module path exists in this builtin graph.
    pub fn contains(&self, path: &ModulePath) -> bool {
        self.node_spec(path).is_some()
    }

    /// Returns the source-unit kind for a canonical builtin path.
    pub fn kind(&self, path: &ModulePath) -> Option<ModuleKind> {
        self.node_spec(path).map(|node| node.kind)
    }

    fn node_spec(&self, path: &ModulePath) -> Option<&'static UniverseNodeSpec> {
        let expected = path.components().iter().map(|component| component.as_str()).collect::<Vec<_>>();
        self.nodes().iter().find(|node| node.path == expected.as_slice())
    }

    pub fn nodes(&self) -> &'static [UniverseNodeSpec] {
        UNIVERSE_NODES
    }

    fn validate_id(&self, id: &ModuleId) -> Result<(), ModuleLoadError> {
        if !id.project.is_universe() {
            return Err(ModuleResolutionError::ModuleNotFound(format!("{id} does not belong to Universe package")).into());
        }
        Ok(())
    }
}
