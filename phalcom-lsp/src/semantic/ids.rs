use std::collections::BTreeMap;
use std::fmt;

#[allow(unused_imports)]
pub use phalcom_modules::identity::{ModuleComponent, ModuleId as SemanticModuleId, ModulePath, ResolvedProjectId};
use tower_lsp::lsp_types::Url;

/// Stable URI namespace for source-authored core declarations.
pub const CORE_MODULE_URI: &str = "phalcom://core";

/// Identity of one source module.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModuleId(String);

impl ModuleId {
    /// Creates an identity from an LSP document URI.
    pub fn from_uri(uri: &Url) -> Self {
        Self(uri.to_string())
    }

    /// Creates an identity from an already-normalized URI string.
    pub fn new(uri: impl Into<String>) -> Self {
        Self(uri.into())
    }

    /// Returns the canonical URI string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Document-to-ModuleId mapping for LSP document seam.
#[derive(Clone, Debug, Default)]
pub struct DocumentModuleMap {
    /// URI-to-semantic identity mapping for open documents.
    pub by_uri: BTreeMap<Url, SemanticModuleId>,
    /// Semantic identity-to-URI reverse mapping for editor surfaces.
    pub by_module: BTreeMap<SemanticModuleId, Url>,
    /// LSP-facing module keys used by the legacy semantic tables.
    ///
    /// These keys are derived once at the document boundary. Request and
    /// analysis code must use this map instead of treating a URI as a module
    /// identity.
    pub lsp_by_uri: BTreeMap<Url, ModuleId>,
    /// Reverse index: LSP-facing key to URI.
    pub uri_by_lsp: BTreeMap<ModuleId, Url>,
}

impl DocumentModuleMap {
    /// Creates an empty document identity map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Associates one document URI with one semantic module identity.
    pub fn insert(&mut self, uri: Url, module: SemanticModuleId) {
        // Enforce bijection: remove previous associations if present
        if let Some(old_mod) = self.by_uri.remove(&uri) {
            self.by_module.remove(&old_mod);
        }
        if let Some(old_uri) = self.by_module.remove(&module) {
            self.by_uri.remove(&old_uri);
            if let Some(old_lsp) = self.lsp_by_uri.remove(&old_uri) {
                self.uri_by_lsp.remove(&old_lsp);
            }
        }
        if let Some(old_lsp) = self.lsp_by_uri.remove(&uri) {
            self.uri_by_lsp.remove(&old_lsp);
        }

        let lsp_module = ModuleId::new(module.to_string());
        self.by_uri.insert(uri.clone(), module.clone());
        self.by_module.insert(module, uri.clone());
        self.lsp_by_uri.insert(uri.clone(), lsp_module.clone());
        self.uri_by_lsp.insert(lsp_module, uri);
    }

    /// Looks up semantic identity for a document URI.
    pub fn get_by_uri(&self, uri: &Url) -> Option<&SemanticModuleId> {
        self.by_uri.get(uri)
    }

    /// Looks up document URI for a semantic module identity.
    pub fn get_by_module(&self, module: &SemanticModuleId) -> Option<&Url> {
        self.by_module.get(module)
    }

    /// Returns the LSP-facing key assigned to one document.
    pub fn lsp_for_uri(&self, uri: &Url) -> Option<&ModuleId> {
        self.lsp_by_uri.get(uri)
    }

    /// Returns the semantic identity associated with one LSP-facing key.
    pub fn semantic_for_lsp(&self, module: &ModuleId) -> Option<&SemanticModuleId> {
        self.uri_by_lsp.get(module).and_then(|uri| self.by_uri.get(uri))
    }

    /// Returns the document URI associated with one LSP-facing key.
    pub fn uri_for_lsp(&self, module: &ModuleId) -> Option<&Url> {
        self.uri_by_lsp.get(module)
    }

    /// Ensures an editor document has a stable LSP key.
    ///
    /// Project-backed documents are inserted through [`Self::insert`] by the
    /// shared resolver. Standalone editor documents retain a boundary-only
    /// fallback key until project discovery supplies semantic identity.
    pub fn ensure_lsp_for_uri(&mut self, uri: &Url) -> ModuleId {
        if let Some(lsp_mod) = self.lsp_by_uri.get(uri) {
            return lsp_mod.clone();
        }
        let lsp_mod = ModuleId::new(uri.to_string());
        self.lsp_by_uri.insert(uri.clone(), lsp_mod.clone());
        self.uri_by_lsp.insert(lsp_mod.clone(), uri.clone());
        lsp_mod
    }

    /// Removes one document mapping and returns its LSP-facing key.
    pub fn remove_uri(&mut self, uri: &Url) -> Option<ModuleId> {
        if let Some(module) = self.by_uri.remove(uri) {
            self.by_module.remove(&module);
        }
        let lsp_module = self.lsp_by_uri.remove(uri);
        if let Some(ref lsp) = lsp_module {
            self.uri_by_lsp.remove(lsp);
        }
        lsp_module
    }
}

/// Identity of a class inside one module.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ClassId {
    /// Module that declares the class.
    pub module: ModuleId,
    /// Class name as written in source.
    pub name: String,
}

impl ClassId {
    /// Creates a module-qualified class identity.
    pub fn new(module: ModuleId, name: impl Into<String>) -> Self {
        Self { module, name: name.into() }
    }
}

/// Identity of a callable member.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CallableId {
    /// Class that owns the callable.
    pub owner: ClassId,
    /// Canonical comma-form selector.
    pub selector: String,
    /// Dispatch side on which the callable is installed.
    pub side: DispatchSide,
}

/// Identity of one field in a class and dispatch side.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FieldId {
    /// Class that owns the field.
    pub owner: ClassId,
    /// Source or implementation field name.
    pub name: String,
    /// Storage side on which the field lives.
    pub side: DispatchSide,
}

/// Dispatch side of a class member.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DispatchSide {
    /// Instance-side dispatch.
    Instance,
    /// Class-side dispatch.
    Class,
}
