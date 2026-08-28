//! Typed semantic query product wrappers and accessors (Spec 04.5 / Wave 5).

use crate::advisory::{AdvisoryCallableSummary, AdvisoryModuleProduct};
use crate::checker::analysis::CallableAnalysis;
use crate::db::state::QueryValue;
use crate::declarations::DeclarationTypeInfo;
use crate::diagnostic::SemanticDiagnostic;
use crate::hierarchy_product::HierarchyEdgeProduct;
use crate::module_product::ResolvedImportsProduct;
use crate::signature::{CallableSemanticSignature, FieldSemanticSignature};
use crate::source::ParsedModuleUnit;
use crate::source_index::{CallableSourceAttachment, ModuleSourceIndex};
use crate::surface::DeclarationSurface;
use phalcom_modules::interface::{LinkedModuleInterface, UnlinkedModuleInterface};
use phalcom_modules::linker::LinkedProgram;
use std::sync::Arc;

/// Declaration-surface query payload.
///
/// The semantic surface is the dependency-visible product. Diagnostics are retained
/// alongside it so the query that resolves member annotations also owns the resulting
/// user-facing errors without making diagnostic-only changes invalidate body consumers.
#[derive(Clone, Debug)]
pub struct DeclarationSurfaceProduct {
    /// Canonical member surface consumed by dispatch and callable queries.
    pub surface: Arc<DeclarationSurface>,
    /// Annotation diagnostics produced while resolving the surface.
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}

impl DeclarationSurfaceProduct {
    /// Creates a declaration-surface query product from its semantic surface and diagnostics.
    pub fn new(surface: Arc<DeclarationSurface>, diagnostics: Arc<[SemanticDiagnostic]>) -> Self {
        Self { surface, diagnostics }
    }
}

/// Strongly-typed wrapper around semantic database product variants.
#[derive(Clone, Debug)]
pub enum SemanticProduct {
    ParsedModule(Arc<ParsedModuleUnit>),
    UnlinkedInterface(Arc<UnlinkedModuleInterface>),
    ResolvedImports(Arc<ResolvedImportsProduct>),
    LinkedInterface(Arc<LinkedModuleInterface>),
    DeclarationShell(Arc<DeclarationTypeInfo>),
    DeclarationSurface(Arc<DeclarationSurfaceProduct>),
    HierarchyEdge(Arc<HierarchyEdgeProduct>),
    CallableSignature(Arc<CallableSemanticSignature>),
    FieldSignature(Arc<FieldSemanticSignature>),
    CallableBody(Arc<CallableAnalysis>),
    SourceStructure(Arc<ModuleSourceIndex>),
    SourceFormalAttachment(Arc<CallableSourceAttachment>),
    AdvisoryCallable(Arc<AdvisoryCallableSummary>),
    AdvisoryModule(Arc<AdvisoryModuleProduct>),
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

    /// Returns canonical declaration type metadata published as a shell product.
    pub fn as_declaration_shell(&self) -> Option<&Arc<DeclarationTypeInfo>> {
        match self {
            Self::DeclarationShell(info) => Some(info),
            _ => None,
        }
    }

    pub fn as_declaration_surface(&self) -> Option<&Arc<DeclarationSurface>> {
        match self {
            Self::DeclarationSurface(product) => Some(&product.surface),
            _ => None,
        }
    }

    /// Returns declaration-surface diagnostics retained by the query product.
    pub fn as_declaration_surface_diagnostics(&self) -> Option<&Arc<[SemanticDiagnostic]>> {
        match self {
            Self::DeclarationSurface(product) => Some(&product.diagnostics),
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

    pub fn as_field_signature(&self) -> Option<&Arc<FieldSemanticSignature>> {
        match self {
            Self::FieldSignature(sig) => Some(sig),
            _ => None,
        }
    }

    pub fn as_callable_body(&self) -> Option<&Arc<CallableAnalysis>> {
        match self {
            Self::CallableBody(body) => Some(body),
            _ => None,
        }
    }

    pub fn as_source_structure(&self) -> Option<&Arc<ModuleSourceIndex>> {
        match self {
            Self::SourceStructure(product) => Some(product),
            _ => None,
        }
    }

    pub fn as_source_formal_attachment(&self) -> Option<&Arc<CallableSourceAttachment>> {
        match self {
            Self::SourceFormalAttachment(product) => Some(product),
            _ => None,
        }
    }

    pub fn as_advisory_callable(&self) -> Option<&Arc<AdvisoryCallableSummary>> {
        match self {
            Self::AdvisoryCallable(product) => Some(product),
            _ => None,
        }
    }

    pub fn as_advisory_module(&self) -> Option<&Arc<AdvisoryModuleProduct>> {
        match self {
            Self::AdvisoryModule(product) => Some(product),
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
            Self::DeclarationShell(_) => b"declaration-shell".as_slice(),
            Self::DeclarationSurface(_) => b"declaration-surface".as_slice(),
            Self::HierarchyEdge(_) => b"hierarchy-edge".as_slice(),
            Self::CallableSignature(_) => b"callable-signature".as_slice(),
            Self::FieldSignature(_) => b"field-signature".as_slice(),
            Self::CallableBody(_) => b"callable-body".as_slice(),
            Self::SourceStructure(_) => b"source-structure".as_slice(),
            Self::SourceFormalAttachment(_) => b"source-formal-attachment".as_slice(),
            Self::AdvisoryCallable(_) => b"advisory-callable".as_slice(),
            Self::AdvisoryModule(_) => b"advisory-module".as_slice(),
            Self::ModuleDiagnostics(_) => b"module-diagnostics".as_slice(),
            Self::SemanticComponent(_) => b"semantic-component".as_slice(),
        };
        QueryValue::from_bytes(kind)
    }
}
