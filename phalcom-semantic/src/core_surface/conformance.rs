//! Cross-layer conformance validation for native surfaces.

use crate::declarations::{DeclarationTypeTable, lower_kind_spec};
use crate::identity::{DeclarationId, ModuleId};
use crate::types::native::resolve_native_type_form;
use crate::types::store::TypeStore;
use phalcom_ast::ast::Statement;
use phalcom_modules::{ModuleComponent, ModulePath, UniverseSourceProvider};
use phalcom_native_meta::TypeExprSpec;
use phalcom_native_meta::universe::{UNIVERSE_BINDINGS, UNIVERSE_CLASS_RELATIONS, UNIVERSE_TYPE_FORMS, UniverseBindingKind, UniverseKey};
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceNominalKind {
    Class,
    Enum,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceNominal {
    kind: SourceNominalKind,
    arity: usize,
    superclass: Option<String>,
    native: bool,
}

fn source_nominal(provider: &UniverseSourceProvider, key: UniverseKey) -> Result<Option<SourceNominal>, String> {
    let module = source_module_id(key);
    let parsed = provider
        .load_parsed(&module)
        .map_err(|error| format!("failed to load canonical source module {module}: {error}"))?;

    for statement in &parsed.program.statements {
        match statement {
            Statement::Class(class_def) if class_def.name == key.name() => {
                return Ok(Some(SourceNominal {
                    kind: SourceNominalKind::Class,
                    arity: class_def.generic_parameters.len(),
                    superclass: class_def.superclass_ref().map(|reference| reference.leaf_name().to_owned()),
                    native: class_def.attributes.iter().any(|attribute| attribute.name == "native"),
                }));
            }
            Statement::Enum(enum_def) if enum_def.name == key.name() => {
                return Ok(Some(SourceNominal {
                    kind: SourceNominalKind::Enum,
                    arity: enum_def.generic_parameters.len(),
                    superclass: None,
                    native: enum_def.attributes.iter().any(|attribute| attribute.name == "native"),
                }));
            }
            _ => {}
        }
    }

    Ok(None)
}

fn validate_source_declaration_conformance(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn crate::types::annotation::TypeResolver,
    current_module: &ModuleId,
    failures: &mut Vec<String>,
) {
    let provider = UniverseSourceProvider::new();

    for binding in UNIVERSE_BINDINGS {
        let module = source_module_id(binding.key);
        let declaration = DeclarationId::new(module.clone(), binding.key.name().into());
        let source = match source_nominal(&provider, binding.key) {
            Ok(nominal) => nominal,
            Err(error) => {
                failures.push(format!("{:?}: {error}", binding.key));
                continue;
            }
        };

        let Some(source) = source else {
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

        let expected_kind = match binding.key {
            UniverseKey::Option | UniverseKey::Result | UniverseKey::Ordering => SourceNominalKind::Enum,
            _ => SourceNominalKind::Class,
        };
        if source.kind != expected_kind {
            failures.push(format!(
                "{:?}: canonical source declaration kind {:?} disagrees with expected {:?}",
                binding.key, source.kind, expected_kind
            ));
        }
        if binding.kind == UniverseBindingKind::Class && !source.native {
            failures.push(format!("{:?}: canonical source declaration is not marked @native", binding.key));
        }
        if source.kind == SourceNominalKind::Class {
            let expected_superclass = UNIVERSE_CLASS_RELATIONS
                .iter()
                .find(|relation| relation.class == binding.key)
                .and_then(|relation| relation.superclass)
                .map(|superclass| superclass.name().to_owned());
            if source.superclass != expected_superclass {
                failures.push(format!(
                    "{:?}: source superclass {:?} disagrees with canonical runtime relation {:?}",
                    binding.key, source.superclass, expected_superclass
                ));
            }
        }

        let Some(info) = declarations.get(&declaration) else {
            failures.push(format!(
                "{:?}: canonical source declaration {:?} is absent from semantic declaration table",
                binding.key, declaration
            ));
            continue;
        };

        // The active resolver must agree with the source-owned declaration for
        // ordinary native rows. Runtime-support rows can intentionally have no
        // source-facing resolver entry.
        if binding.kind == UniverseBindingKind::Class {
            let resolved = resolver.resolve_type_name(&module, binding.key.name(), &[]);
            if resolved.as_ref() != Some(&declaration) {
                failures.push(format!(
                    "{:?}: active resolver associates source declaration with {:?}, expected {:?} (checking from {current_module})",
                    binding.key, resolved, declaration
                ));
            }
        }

        let semantic_arity = info.generic_signature.as_ref().map_or(0, |signature| signature.parameters.len());
        if semantic_arity != source.arity {
            failures.push(format!(
                "{:?}: source generic arity {} disagrees with semantic declaration arity {semantic_arity}",
                binding.key, source.arity
            ));
        }

        let native_form = UNIVERSE_TYPE_FORMS.iter().find(|spec| spec.owner == binding.key);
        let native_arity = native_form.map_or(0, |spec| spec.parameters.len());
        if native_arity != source.arity {
            failures.push(format!(
                "{:?}: native type-form generic arity {native_arity} disagrees with canonical source arity {}",
                binding.key, source.arity
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
    resolver: &dyn crate::types::annotation::TypeResolver,
    current_module: &ModuleId,
) -> ConformanceReport {
    let mut report = ConformanceReport::default();
    if let Err(failures) = phalcom_native_surface::validate_native_surface_catalog(NATIVE_SURFACES) {
        report.failures.extend(failures);
    }

    validate_source_declaration_conformance(store, declarations, resolver, current_module, &mut report.failures);

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
