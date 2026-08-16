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
}

impl DocumentModuleMap {
    /// Creates an empty document identity map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Associates one document URI with one semantic module identity.
    pub fn insert(&mut self, uri: Url, module: SemanticModuleId) {
        self.by_uri.insert(uri.clone(), module.clone());
        self.by_module.insert(module, uri);
    }

    /// Looks up semantic identity for a document URI.
    pub fn get_by_uri(&self, uri: &Url) -> Option<&SemanticModuleId> {
        self.by_uri.get(uri)
    }

    /// Looks up document URI for a semantic module identity.
    pub fn get_by_module(&self, module: &SemanticModuleId) -> Option<&Url> {
        self.by_module.get(module)
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
