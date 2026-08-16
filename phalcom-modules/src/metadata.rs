//! Inert declarative unit-header metadata attributes (`@!`).

use crate::error::InterfaceError;
use crate::source::ModuleKind;
use phalcom_ast::ast::{
    MetadataLiteral, ModuleMetadataAttribute as AstModuleMetadataAttribute,
};
use phalcom_common::range::SourceRange;
use std::fmt;

/// Semantic owner to which a parsed header attribute is attached.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataTarget {
    Project,
    Package,
    Module,
}

impl fmt::Display for MetadataTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Project => write!(f, "project"),
            Self::Package => write!(f, "package"),
            Self::Module => write!(f, "module"),
        }
    }
}

/// A validated, inert unit metadata attribute.
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleMetadataAttribute {
    pub target: MetadataTarget,
    pub name: String,
    pub arguments: Vec<MetadataLiteral>,
    pub range: SourceRange,
}

/// Metadata attached to one unit interface/owner.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModuleMetadata {
    pub attributes: Vec<ModuleMetadataAttribute>,
}

impl ModuleMetadata {
    /// Constructs inert metadata from the AST. Attribute names do not acquire
    /// semantics here: unknown syntactically valid names are retained verbatim.
    pub fn from_ast(
        ast_attrs: &[AstModuleMetadataAttribute],
        kind: ModuleKind,
    ) -> Result<Self, InterfaceError> {
        let target = match kind {
            ModuleKind::Package => MetadataTarget::Package,
            ModuleKind::Module => MetadataTarget::Module,
        };
        Ok(Self::with_target(ast_attrs, target))
    }

    pub fn with_target(
        ast_attrs: &[AstModuleMetadataAttribute],
        target: MetadataTarget,
    ) -> Self {
        let attributes = ast_attrs
            .iter()
            .map(|attr| ModuleMetadataAttribute {
                target,
                name: attr.name.clone(),
                arguments: attr.arguments.clone(),
                range: attr.range,
            })
            .collect();
        Self { attributes }
    }
}
