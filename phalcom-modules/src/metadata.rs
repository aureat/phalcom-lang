//! Inert module and package metadata attributes (`@!`).

use crate::error::InterfaceError;
use crate::source::ModuleKind;
use phalcom_ast::ast::{MetadataLiteral, ModuleMetadataAttribute as AstModuleMetadataAttribute};
use phalcom_common::range::SourceRange;
use std::fmt;

/// Semantic target of a module/package header attribute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataTarget {
    Module,
    Package,
    Project,
}

impl fmt::Display for MetadataTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Module => write!(f, "module"),
            Self::Package => write!(f, "package"),
            Self::Project => write!(f, "project"),
        }
    }
}

/// A validated, inert module or package metadata attribute.
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleMetadataAttribute {
    pub target: MetadataTarget,
    pub name: String,
    pub arguments: Vec<MetadataLiteral>,
    pub range: SourceRange,
}

/// Collection of metadata attributes attached to a module interface.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModuleMetadata {
    pub attributes: Vec<ModuleMetadataAttribute>,
}

impl ModuleMetadata {
    /// Validates and constructs `ModuleMetadata` from AST attributes for the given file kind.
    pub fn from_ast(ast_attrs: &[AstModuleMetadataAttribute], kind: ModuleKind) -> Result<Self, InterfaceError> {
        let target = match kind {
            ModuleKind::Package => MetadataTarget::Package,
            ModuleKind::Module => MetadataTarget::Module,
        };

        let mut attributes = Vec::new();
        for attr in ast_attrs {
            // Unknown metadata is data, not semantics. Standardized semantic
            // attributes must be registered explicitly rather than inferred from spelling.
            attributes.push(ModuleMetadataAttribute {
                target,
                name: attr.name.clone(),
                arguments: attr.arguments.clone(),
                range: attr.range,
            });
        }

        Ok(Self { attributes })
    }
}
