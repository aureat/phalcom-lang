//! LSP-only transport for configured, discovered, and bundled core source.
//!
//! Core semantic products are built by `phalcom-semantic`; this module only
//! selects and parses source text needed by the worker and virtual documents.

use phalcom_ast::parser::Parse;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower_lsp::lsp_types::Url;

/// Stable URI used for the canonical virtual core module.
pub const CORE_MODULE_URI: &str = "phalcom://core";

/// Source location and text selected for core transport.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreSource {
    /// Explicitly configured physical source.
    Configured { physical_uri: Url, text: Arc<str> },
    /// Workspace-discovered physical source.
    Workspace { physical_uri: Url, text: Arc<str> },
    /// Bundled source assembled from canonical builtin modules.
    Bundled { text: Arc<str> },
}

impl CoreSource {
    /// Selects configured, workspace, or bundled core source in that order.
    pub fn select(configured_path: Option<&Path>, workspace_roots: &[PathBuf]) -> Self {
        if let Some(path) = configured_path
            && let Some(source) = Self::load_from_path(path, true)
        {
            return source;
        }
        for root in workspace_roots {
            if let Some(source) = Self::load_from_path(&root.join("phalcom-core/core/universe/src/package.ph"), false) {
                return source;
            }
            if let Some(source) = Self::load_from_path(&root.join("core/universe/src/package.ph"), false) {
                return source;
            }
        }
        Self::Bundled {
            text: Arc::from(canonical_universe_source()),
        }
    }

    fn load_from_path(path: &Path, configured: bool) -> Option<Self> {
        let text = std::fs::read_to_string(path).ok()?;
        let physical_uri = Url::from_file_path(path.canonicalize().unwrap_or_else(|_| path.to_path_buf())).ok()?;
        let text = Arc::from(text);
        Some(if configured {
            Self::Configured { physical_uri, text }
        } else {
            Self::Workspace { physical_uri, text }
        })
    }

    /// Returns selected source text.
    pub fn text(&self) -> &str {
        match self {
            Self::Configured { text, .. } | Self::Workspace { text, .. } | Self::Bundled { text } => text,
        }
    }

    /// Parses selected source, preserving separate bundled-module preambles.
    pub fn parse(&self) -> Parse {
        match self {
            Self::Bundled { .. } => bundled_parse(),
            Self::Configured { .. } | Self::Workspace { .. } => phalcom_ast::parser::parse(self.text(), 0),
        }
    }

    /// Returns physical URI when selected source came from disk.
    pub fn physical_uri(&self) -> Option<&Url> {
        match self {
            Self::Configured { physical_uri, .. } | Self::Workspace { physical_uri, .. } => Some(physical_uri),
            Self::Bundled { .. } => None,
        }
    }
}

/// Parses all bundled universe modules into one transport program.
pub fn bundled_parse() -> Parse {
    let provider = phalcom_modules::builtin::BuiltinProjectSourceProvider::new(phalcom_modules::identity::BuiltinProject::Universe);
    let mut combined = phalcom_ast::parser::parse("", 0);
    for node in provider.nodes() {
        let path = phalcom_modules::identity::ModulePath::from_components(
            node.path
                .iter()
                .map(|component| phalcom_modules::ModuleComponent::from_identifier(component).expect("valid builtin component"))
                .collect::<Vec<_>>(),
        );
        let module = phalcom_modules::identity::ModuleId::builtin(phalcom_modules::identity::BuiltinProject::Universe, path);
        let source = provider
            .source_text(&module)
            .unwrap_or_else(|error| panic!("failed to load canonical universe source {module}: {error}"));
        let parsed = phalcom_ast::parser::parse(&source, 0);
        combined.program.statements.extend(parsed.program.statements);
        combined.errors.extend(parsed.errors);
    }
    combined
}

fn canonical_universe_source() -> String {
    let provider = phalcom_modules::builtin::BuiltinProjectSourceProvider::new(phalcom_modules::identity::BuiltinProject::Universe);
    let mut combined = String::new();
    for node in provider.nodes() {
        let path = phalcom_modules::identity::ModulePath::from_components(
            node.path
                .iter()
                .map(|component| phalcom_modules::ModuleComponent::from_identifier(component).expect("valid builtin component"))
                .collect::<Vec<_>>(),
        );
        let module = phalcom_modules::identity::ModuleId::builtin(phalcom_modules::identity::BuiltinProject::Universe, path);
        let source = provider
            .source_text(&module)
            .unwrap_or_else(|error| panic!("failed to load canonical universe source {module}: {error}"));
        combined.push_str(&source);
        combined.push('\n');
    }
    combined
}
