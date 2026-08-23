//! Typed semantic query product wrappers and accessors (Spec 04.5 / Wave 5).

use crate::checker::analysis::CallableAnalysis;
use crate::db::state::QueryValue;
use crate::diagnostic::SemanticDiagnostic;
use crate::hierarchy_product::HierarchyEdgeProduct;
use crate::module_product::ResolvedImportsProduct;
use crate::signature::CallableSemanticSignature;
use crate::source::ParsedModuleUnit;
use crate::surface::DeclarationSurface;
use phalcom_modules::interface::{LinkedModuleInterface, UnlinkedModuleInterface};
use phalcom_modules::linker::LinkedProgram;
use std::sync::Arc;

/// Strongly-typed wrapper around semantic database product variants.
#[derive(Clone, Debug)]
pub enum SemanticProduct {
    ParsedModule(Arc<ParsedModuleUnit>),
    UnlinkedInterface(Arc<UnlinkedModuleInterface>),
    ResolvedImports(Arc<ResolvedImportsProduct>),
    LinkedInterface(Arc<LinkedModuleInterface>),
    DeclarationSurface(Arc<DeclarationSurface>),
    HierarchyEdge(Arc<HierarchyEdgeProduct>),
    CallableSignature(Arc<CallableSemanticSignature>),
    CallableBody(Arc<CallableAnalysis>),
    ModuleDiagnostics(Arc<[SemanticDiagnostic]>),
    SemanticComponent(Arc<LinkedProgram>),
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

    pub fn as_resolved_imports(&self) -> Option<&Arc<ResolvedImportsProduct>> {
        match self {
            Self::ResolvedImports(product) => Some(product),
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

    pub fn as_hierarchy_edge(&self) -> Option<&Arc<HierarchyEdgeProduct>> {
        match self {
            Self::HierarchyEdge(edge) => Some(edge),
            _ => None,
        }
    }

    pub fn as_callable_signature(&self) -> Option<&Arc<CallableSemanticSignature>> {
        match self {
            Self::CallableSignature(sig) => Some(sig),
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

    pub fn as_semantic_component(&self) -> Option<&Arc<LinkedProgram>> {
        match self {
            Self::SemanticComponent(program) => Some(program),
            _ => None,
        }
    }

    /// Converts typed product into type-erased `QueryValue`.
    pub fn to_query_value(&self) -> QueryValue {
        let kind = match self {
            Self::ParsedModule(_) => b"parsed-module".as_slice(),
            Self::UnlinkedInterface(_) => b"unlinked-interface".as_slice(),
            Self::ResolvedImports(_) => b"resolved-imports".as_slice(),
            Self::LinkedInterface(_) => b"linked-interface".as_slice(),
            Self::DeclarationSurface(_) => b"declaration-surface".as_slice(),
            Self::HierarchyEdge(_) => b"hierarchy-edge".as_slice(),
            Self::CallableSignature(_) => b"callable-signature".as_slice(),
            Self::CallableBody(_) => b"callable-body".as_slice(),
            Self::ModuleDiagnostics(_) => b"module-diagnostics".as_slice(),
            Self::SemanticComponent(_) => b"semantic-component".as_slice(),
        };
        QueryValue::from_bytes(kind)
    }
}
