//! Cross-layer conformance validation for native surfaces.

use crate::declarations::DeclarationTypeTable;
use crate::identity::ModuleId;
use crate::types::native::resolve_native_type_form;
use crate::types::store::TypeStore;
use phalcom_native_meta::TypeExprSpec;
use phalcom_native_surface::NATIVE_SURFACES;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct ConformanceReport {
    pub total_surfaces: usize,
    pub resolved_surfaces: usize,
    pub failures: Vec<String>,
}

/// Runs full conformance validation on the canonical native surface catalog.
pub fn validate_native_surface_conformance(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    _resolver: &dyn crate::types::annotation::TypeResolver,
    _current_module: &ModuleId,
) -> ConformanceReport {
    let mut report = ConformanceReport::default();
    if let Err(failures) = phalcom_native_surface::validate_native_surface_catalog(NATIVE_SURFACES) {
        report.failures.extend(failures);
    }
    let universe_resolver = |key: phalcom_native_meta::UniverseKey| -> crate::identity::DeclarationId { crate::core_surface::universe_declaration(key) };
    let empty_params = HashMap::new();

    let resolve_spec = |store: &mut TypeStore, spec: &TypeExprSpec| -> Result<(), String> {
        if matches!(spec, TypeExprSpec::Unknown | TypeExprSpec::SelfType) {
            return Ok(());
        }
        resolve_native_type_form(store, declarations, &empty_params, &universe_resolver, spec)
            .map(|_| ())
            .map_err(|e| e.to_string())
    };

    for record in NATIVE_SURFACES {
        report.total_surfaces += 1;
        let mut failed = false;

        // 1. Resolve return type
        if let Err(e) = resolve_spec(store, record.returns()) {
            report
                .failures
                .push(format!("{:?}.{}: failed to resolve return type: {e}", record.owner(), record.selector()));
            failed = true;
        }

        // 2. Resolve parameter types
        for (i, p) in record.params().positional.iter().enumerate() {
            if let Err(e) = resolve_spec(store, p) {
                report
                    .failures
                    .push(format!("{:?}.{}: failed to resolve param {i}: {e}", record.owner(), record.selector()));
                failed = true;
            }
        }
        for labeled in record.params().labeled {
            if let Err(e) = resolve_spec(store, labeled.ty) {
                report.failures.push(format!(
                    "{:?}.{}: failed to resolve labeled param {}: {e}",
                    record.owner(),
                    record.selector(),
                    labeled.label
                ));
                failed = true;
            }
        }
        if let Some(rest) = record.params().rest.and_then(|rest| rest.ty) {
            if let Err(e) = resolve_spec(store, rest) {
                report
                    .failures
                    .push(format!("{:?}.{}: failed to resolve rest param: {e}", record.owner(), record.selector()));
                failed = true;
            }
        }

        if !failed {
            report.resolved_surfaces += 1;
        }
    }

    report
}
