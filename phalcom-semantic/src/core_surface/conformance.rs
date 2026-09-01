//! Cross-layer conformance validation for native surfaces.

use crate::declarations::{DeclarationTypeTable, lower_kind_spec};
use crate::identity::{DeclarationId, ModuleId};
use crate::types::native::resolve_native_type_form;
use crate::types::store::TypeStore;
use phalcom_ast::ast::Statement;
use phalcom_modules::{ModuleComponent, ModulePath, UniverseSourceProvider};
use phalcom_native_meta::TypeExprSpec;
use phalcom_native_meta::universe::{UniverseBindingKind, UniverseKey, UNIVERSE_BINDINGS, UNIVERSE_TYPE_FORMS};
use phalcom_native_surface::NATIVE_SURFACES;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct ConformanceReport {
    pub total_surfaces: usize,
    pub resolved_surfaces: usize,
    pub failures: Vec<String>,
}

fn source_module_id(key: UniverseKey) -> ModuleId {
    let components = key
        .source_path()
        .iter()
        .map(|component| ModuleComponent::from_identifier(component).expect("canonical Universe component"))
        .collect::<Vec<_>>();
    ModuleId::universe(ModulePath::from_components(components))
}

fn source_nominal_arity(provider: &UniverseSourceProvider, key: UniverseKey) -> Result<Option<usize>, String> {
    let module = source_module_id(key);
    let parsed = provider
        .load_parsed(&module)
        .map_err(|error| format!("failed to load canonical source module {module}: {error}"))?;

    for statement in &parsed.program.statements {
        match statement {
            Statement::Class(class_def) if class_def.name == key.name() => return Ok(Some(class_def.generic_parameters.len())),
            Statement::Enum(enum_def) if enum_def.name == key.name() => return Ok(Some(enum_def.generic_parameters.len())),
            _ => {}
        }
    }

    Ok(None)
}

fn validate_source_declaration_conformance(store: &mut TypeStore, declarations: &DeclarationTypeTable, failures: &mut Vec<String>) {
    let provider = UniverseSourceProvider::new();

    for binding in UNIVERSE_BINDINGS {
        let module = source_module_id(binding.key);
        let declaration = DeclarationId::new(module.clone(), binding.key.name().into());
        let source_arity = match source_nominal_arity(&provider, binding.key) {
            Ok(arity) => arity,
            Err(error) => {
                failures.push(format!("{:?}: {error}", binding.key));
                continue;
            }
        };

        let Some(source_arity) = source_arity else {
            if binding.kind != UniverseBindingKind::RuntimeSupportClass {
                failures.push(format!(
                    "{:?}: native binding claims ordinary declaration, but canonical source {}::{} has no class/enum declaration",
                    binding.key,
                    module,
                    binding.key.name()
                ));
            }
            continue;
        };

        let Some(info) = declarations.get(&declaration) else {
            failures.push(format!(
                "{:?}: canonical source declaration {} is absent from semantic declaration table",
                binding.key, declaration
            ));
            continue;
        };

        let semantic_arity = info.generic_signature.as_ref().map_or(0, |signature| signature.parameters.len());
        if semantic_arity != source_arity {
            failures.push(format!(
                "{:?}: source generic arity {source_arity} disagrees with semantic declaration arity {semantic_arity}",
                binding.key
            ));
        }

        let native_form = UNIVERSE_TYPE_FORMS.iter().find(|spec| spec.owner == binding.key);
        let native_arity = native_form.map_or(0, |spec| spec.parameters.len());
        if native_arity != source_arity {
            failures.push(format!(
                "{:?}: native type-form generic arity {native_arity} disagrees with canonical source arity {source_arity}",
                binding.key
            ));
            continue;
        }

        if let (Some(native_form), Some(signature)) = (native_form, info.generic_signature.as_ref()) {
            for (index, (native_parameter, semantic_parameter)) in native_form.parameters.iter().zip(signature.parameters.iter()).enumerate() {
                let native_kind = lower_kind_spec(store, &native_parameter.kind);
                let semantic_kind = store.type_parameter(*semantic_parameter).kind;
                if native_kind != semantic_kind {
                    failures.push(format!(
                        "{:?}: generic parameter {index} native kind {:?} disagrees with source-derived semantic kind {:?}",
                        binding.key, native_kind, semantic_kind
                    ));
                }
            }
        }
    }
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

    validate_source_declaration_conformance(store, declarations, &mut report.failures);

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
