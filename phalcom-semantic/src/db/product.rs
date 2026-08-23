//! Typed semantic query product wrappers and accessors (Spec 04.5 / Wave 5).

use crate::checker::analysis::CallableAnalysis;
use crate::db::state::QueryValue;
use crate::diagnostic::SemanticDiagnostic;
use crate::source::ParsedModuleUnit;
use crate::surface::DeclarationSurface;
use phalcom_modules::interface::{LinkedModuleInterface, UnlinkedModuleInterface};
use std::sync::Arc;

/// Strongly-typed wrapper around semantic database product variants.
#[derive(Clone, Debug)]
pub enum SemanticProduct {
    ParsedModule(Arc<ParsedModuleUnit>),
    UnlinkedInterface(Arc<UnlinkedModuleInterface>),
    LinkedInterface(Arc<LinkedModuleInterface>),
    DeclarationSurface(Arc<DeclarationSurface>),
    CallableBody(Arc<CallableAnalysis>),
    ModuleDiagnostics(Arc<[SemanticDiagnostic]>),
}

impl SemanticProduct {
    pub fn as_parsed_module(&self) -> Option<&Arc<ParsedModuleUnit>> {
        match self {
            Self::ParsedModule(unit) => Some(unit),
            _ => None,
        }
    }

    pub fn as_unlinked_interface(&self) -> Option<&Arc<UnlinkedModuleInterface>> {
        match self {
            Self::UnlinkedInterface(interface) => Some(interface),
            _ => None,
        }
    }

    pub fn as_linked_interface(&self) -> Option<&Arc<LinkedModuleInterface>> {
        match self {
            Self::LinkedInterface(interface) => Some(interface),
            _ => None,
        }
    }

    pub fn as_declaration_surface(&self) -> Option<&Arc<DeclarationSurface>> {
        match self {
            Self::DeclarationSurface(surface) => Some(surface),
            _ => None,
        }
    }

    pub fn as_callable_body(&self) -> Option<&Arc<CallableAnalysis>> {
        match self {
            Self::CallableBody(body) => Some(body),
            _ => None,
        }
    }

    pub fn as_module_diagnostics(&self) -> Option<&Arc<[SemanticDiagnostic]>> {
        match self {
            Self::ModuleDiagnostics(diags) => Some(diags),
            _ => None,
        }
    }

    /// Converts typed product into type-erased `QueryValue`.
    pub fn to_query_value(&self) -> QueryValue {
        // The typed product is retained by `SemanticDb::products`; this small
        // discriminator keeps the erased state lossless enough for generic
        // query instrumentation without pretending bytes are the product.
        let kind = match self {
            Self::ParsedModule(_) => b"parsed-module".as_slice(),
            Self::UnlinkedInterface(_) => b"unlinked-interface".as_slice(),
            Self::LinkedInterface(_) => b"linked-interface".as_slice(),
            Self::DeclarationSurface(_) => b"declaration-surface".as_slice(),
            Self::CallableBody(_) => b"callable-body".as_slice(),
            Self::ModuleDiagnostics(_) => b"module-diagnostics".as_slice(),
        };
        QueryValue::from_bytes(kind)
    }
}
