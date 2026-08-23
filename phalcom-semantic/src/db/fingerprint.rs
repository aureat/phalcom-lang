//! Deterministic semantic input and product fingerprint hashing (Spec 04.5 / Wave 5).

use crate::checker::analysis::CallableAnalysis;
use crate::db::key::{InputFingerprint, ProductFingerprint};
use crate::identity::{CallableId, DeclarationId, ModuleId};
use crate::signature::CallableSemanticSignature;
use crate::surface::DeclarationSurface;
use crate::types::store::TypeStore;
use phalcom_ast::ast::Statement;
use phalcom_common::range::SourceRange;
use phalcom_modules::interface::{ImportSurface, LinkedExportTarget, LinkedModuleInterface, UnlinkedModuleInterface};
use phalcom_modules::source::ModuleKind;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Computes input fingerprint for a parsed module query.
pub fn parsed_module_input_fingerprint(module: &ModuleId, kind: ModuleKind, source: &str) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    module.hash(&mut hasher);
    format!("{kind:?}").hash(&mut hasher);
    source.as_bytes().hash(&mut hasher);
    InputFingerprint::new(hasher.finish())
}

/// Computes product fingerprint for an unlinked module interface.
pub fn unlinked_interface_product_fingerprint(interface: &UnlinkedModuleInterface) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    interface.id.hash(&mut hasher);
    format!("{:?}", interface.kind).hash(&mut hasher);
    for import in &interface.imports {
        match import {
            ImportSurface::Module(m) => {
                0u8.hash(&mut hasher);
                format!("{:?}", m.path).hash(&mut hasher);
                format!("{:?}", m.alias).hash(&mut hasher);
            }
            ImportSurface::Selective(s) => {
                1u8.hash(&mut hasher);
                format!("{:?}", s.path).hash(&mut hasher);
                for item in &s.items {
                    item.name.hash(&mut hasher);
                    format!("{:?}", item.alias).hash(&mut hasher);
                }
            }
            ImportSurface::ReExport(r) => {
                2u8.hash(&mut hasher);
                format!("{:?}", r.path).hash(&mut hasher);
                for item in &r.items {
                    item.local_or_remote_name.hash(&mut hasher);
                    format!("{:?}", item.alias).hash(&mut hasher);
                }
            }
        }
    }
    for (name, export) in &interface.exports {
        name.hash(&mut hasher);
        export.exported_name.hash(&mut hasher);
        export.internal_name.hash(&mut hasher);
    }
    for child in &interface.exposed_children {
        child.hash(&mut hasher);
    }
    ProductFingerprint::new(hasher.finish())
}

/// Computes product fingerprint for a linked module interface.
pub fn linked_interface_product_fingerprint(interface: &LinkedModuleInterface) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    interface.module.hash(&mut hasher);
    format!("{:?}", interface.kind).hash(&mut hasher);
    for (name, export) in &interface.exports {
        name.hash(&mut hasher);
        export.public_name.hash(&mut hasher);
        match &export.target {
            LinkedExportTarget::Binding(sym) => {
                0u8.hash(&mut hasher);
                sym.module.hash(&mut hasher);
                sym.name.hash(&mut hasher);
            }
            LinkedExportTarget::Module(mod_id) => {
                1u8.hash(&mut hasher);
                mod_id.hash(&mut hasher);
            }
        }
    }
    ProductFingerprint::new(hasher.finish())
}

/// Computes product fingerprint for a declaration surface.
pub fn declaration_surface_product_fingerprint(surface: &DeclarationSurface) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    surface.id.hash(&mut hasher);
    // Instance fields
    let mut inst_fields: Vec<_> = surface.instance.fields.iter().collect();
    inst_fields.sort_by_key(|(name, _)| *name);
    for (name, ty) in inst_fields {
        name.hash(&mut hasher);
        ty.hash(&mut hasher);
    }
    // Class fields
    let mut class_fields: Vec<_> = surface.class.fields.iter().collect();
    class_fields.sort_by_key(|(name, _)| *name);
    for (name, ty) in class_fields {
        name.hash(&mut hasher);
        ty.hash(&mut hasher);
    }
    // Instance callable signatures
    let mut inst_callables: Vec<_> = surface.instance.callable_signatures.iter().collect();
    inst_callables.sort_by_key(|(sel, _)| (*sel).clone());
    for (sel, sig) in inst_callables {
        sel.hash(&mut hasher);
        sig.return_type.hash(&mut hasher);
        for p in &sig.parameters {
            p.local_name.hash(&mut hasher);
            p.external_label.hash(&mut hasher);
            format!("{:?}", p.rest).hash(&mut hasher);
            p.ty.hash(&mut hasher);
        }
    }
    // Class callable signatures
    let mut class_callables: Vec<_> = surface.class.callable_signatures.iter().collect();
    class_callables.sort_by_key(|(sel, _)| (*sel).clone());
    for (sel, sig) in class_callables {
        sel.hash(&mut hasher);
        sig.return_type.hash(&mut hasher);
        for p in &sig.parameters {
            p.local_name.hash(&mut hasher);
            p.external_label.hash(&mut hasher);
            format!("{:?}", p.rest).hash(&mut hasher);
            p.ty.hash(&mut hasher);
        }
    }
    ProductFingerprint::new(hasher.finish())
}

/// Computes product fingerprint for a callable semantic signature.
pub fn callable_signature_product_fingerprint(sig: &CallableSemanticSignature) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    sig.callable.hash(&mut hasher);
    sig.owner.hash(&mut hasher);
    sig.side.hash(&mut hasher);
    sig.selector.hash(&mut hasher);
    sig.return_type.hash(&mut hasher);
    for p in sig.parameters.iter() {
        p.index.hash(&mut hasher);
        p.local_name.hash(&mut hasher);
        p.external_label.hash(&mut hasher);
        format!("{:?}", p.rest).hash(&mut hasher);
        p.ty.hash(&mut hasher);
    }
    format!("{:?}", sig.implementation).hash(&mut hasher);
    ProductFingerprint::new(hasher.finish())
}

/// Computes product fingerprint for a hierarchy edge.
pub fn hierarchy_edge_product_fingerprint(class_decl: &DeclarationId, super_decl: &Option<DeclarationId>) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    class_decl.hash(&mut hasher);
    super_decl.hash(&mut hasher);
    ProductFingerprint::new(hasher.finish())
}

/// Computes input fingerprint for a callable body query.
pub fn callable_body_input_fingerprint(
    callable: &CallableId,
    body: &[Statement],
    body_range: SourceRange,
    store: &TypeStore,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    callable.hash(&mut hasher);
    body_range.start.hash(&mut hasher);
    body_range.end.hash(&mut hasher);
    store.id().hash(&mut hasher);
    for statement in body {
        format!("{statement:?}").hash(&mut hasher);
    }
    InputFingerprint::new(hasher.finish())
}

/// Computes product fingerprint for a callable body analysis result.
pub fn callable_body_product_fingerprint(analysis: &CallableAnalysis) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    analysis.callable.hash(&mut hasher);
    analysis.body_range.start.hash(&mut hasher);
    analysis.body_range.end.hash(&mut hasher);
    for (expr_id, expr) in &analysis.expressions {
        expr_id.hash(&mut hasher);
        expr.range.start.hash(&mut hasher);
        expr.range.end.hash(&mut hasher);
        expr.knowledge.hash(&mut hasher);
    }
    for (bind_id, _bind) in &analysis.bindings {
        bind_id.hash(&mut hasher);
    }
    for d in analysis.diagnostics.iter() {
        d.code.hash(&mut hasher);
        d.severity.hash(&mut hasher);
        d.primary_range.start.hash(&mut hasher);
        d.primary_range.end.hash(&mut hasher);
    }
    for dep in analysis.semantic_dependencies.iter() {
        dep.hash(&mut hasher);
    }
    ProductFingerprint::new(hasher.finish())
}

/// Computes product fingerprint for resolved imports.
pub fn resolved_imports_product_fingerprint(product: &crate::module_product::ResolvedImportsProduct) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    product.module.hash(&mut hasher);
    for (path, target) in &product.imports {
        path.hash(&mut hasher);
        target.hash(&mut hasher);
    }
    for (err, range) in &product.unresolved_diagnostics {
        err.hash(&mut hasher);
        range.start.hash(&mut hasher);
        range.end.hash(&mut hasher);
    }
    ProductFingerprint::new(hasher.finish())
}

/// Computes product fingerprint for module diagnostics.
pub fn module_diagnostics_product_fingerprint(module: &ModuleId, diagnostics: &[crate::diagnostic::SemanticDiagnostic]) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    module.hash(&mut hasher);
    for d in diagnostics {
        d.code.hash(&mut hasher);
        d.severity.hash(&mut hasher);
        d.primary_range.start.hash(&mut hasher);
        d.primary_range.end.hash(&mut hasher);
        d.message.hash(&mut hasher);
    }
    ProductFingerprint::new(hasher.finish())
}
