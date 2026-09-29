//! Cold/incremental parity for the LANG007.C3.P4 T12.6 fingerprint edit matrix.

use super::support::single_module_input;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::snapshot::SemanticSnapshot;
use phalcom_semantic::SemanticWorkspaceSession;
use std::sync::Arc;

fn module(name: &str) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(12_060),
        ModulePath::from_components(vec![ModuleComponent::from_identifier(name).expect("valid module")]),
    )
}

fn diagnostic_summary(snapshot: &SemanticSnapshot) -> Vec<(String, String, phalcom_common::range::SourceRange)> {
    snapshot
        .all_diagnostics()
        .map(|diagnostic| (format!("{:?}", diagnostic.code), diagnostic.message.clone(), diagnostic.primary_range))
        .collect()
}

fn alias_summary(snapshot: &SemanticSnapshot, declaration: &DeclarationId) -> (String, String, Option<String>) {
    let alias = snapshot.type_aliases.get(declaration).unwrap_or_else(|| {
        panic!(
            "published alias {declaration:?}; aliases={:?}; diagnostics={:?}; sources={:?}; resolved_imports={:?}",
            snapshot.type_aliases.iter().map(|(id, _)| id).collect::<Vec<_>>(),
            diagnostic_summary(snapshot),
            snapshot.sources.iter().map(|(module, source)| (module, source.program.statements.iter().filter_map(|statement| {
                if let phalcom_ast::ast::Statement::TypeAlias(alias) = statement {
                    Some((alias.name.as_str(), format!("{:?}", alias.body.expr)))
                } else {
                    None
                }
            }).collect::<Vec<_>>())).collect::<Vec<_>>(),
            snapshot.module_products.resolved_imports
        )
    });
    let signature = alias.generic_signature.as_ref().map(|signature| {
        format!(
            "modes={:?}; kinds={:?}; shapes={:?}",
            signature.parameter_modes, signature.parameter_kinds, signature.parameter_kind_shapes
        )
    });
    (
        alias.kind_shape.to_string(),
        snapshot.store.format_type(alias.form),
        signature,
    )
}

#[test]
fn t12_6_expansion_kind_mode_and_fixed_prefix_edits_match_cold_analysis() {
    let module = module("t12_6_pack_edits");
    let revisions = [
        "type Shape<*Ps: TypePack> = (Int, *Ps)\n",
        "type Shape<**Ps: LabeledTypePack> = (id: Int, **Ps)\n",
        "type Shape<***Ps: CompleteTypePack> = (Int, ***Ps)\n",
        "type Shape<**Fs: RecordFields> = { id: Int, **Fs }\n",
        "type Shape<**Fs: RecordFields> = { id: Int, tag: String, **Fs }\n",
        "type Shape<**Fs: RecordFields> = { **Fs }\n",
    ];
    let mut session = SemanticWorkspaceSession::new();
    for (index, source) in revisions.iter().enumerate() {
        let update = session.update(single_module_input(module.clone(), source, index as u64 + 1));
        assert!(
            !update.snapshot.has_errors(),
            "revision {index} must be valid: {:?}",
            update.snapshot.diagnostics
        );
        let cold = SemanticWorkspaceSession::new().update(single_module_input(module.clone(), source, 1));
        assert_eq!(
            diagnostic_summary(&update.snapshot),
            diagnostic_summary(&cold.snapshot),
            "revision {index} diagnostics must match cold analysis"
        );
        let declaration = DeclarationId::new(module.clone(), "Shape".into());
        assert_eq!(
            alias_summary(&update.snapshot, &declaration),
            alias_summary(&cold.snapshot, &declaration),
            "revision {index} generic shape must match cold analysis"
        );
    }
}

fn data_shape_summary(snapshot: &SemanticSnapshot, owner: &DeclarationId) -> Vec<(String, String, String)> {
    snapshot
        .data_semantics
        .data_info(owner)
        .expect("published data shape")
        .shape_template
        .contributions
        .iter()
        .map(|contribution| match contribution {
            phalcom_semantic::data_semantics::DataProductContribution::Component(component) => (
                "component".into(),
                component.local_name.to_string(),
                component
                    .declared_type
                    .known_term()
                    .and_then(|term| match term {
                        phalcom_semantic::types::parameter::TypeTerm::Canonical(ty) => Some(*ty),
                        _ => None,
                    })
                    .map(|ty| snapshot.store.format_type(ty))
                    .unwrap_or_else(|| "<noncanonical>".into()),
            ),
            phalcom_semantic::data_semantics::DataProductContribution::Expansion(expansion) => (
                "expansion".into(),
                format!("{:?}", expansion.mode),
                format!("{:?}", expansion.operand),
            ),
        })
        .collect()
}

#[test]
fn t12_6_data_fixed_components_around_expansion_match_cold_analysis() {
    let module = module("t12_6_data_shape");
    let revisions = [
        "data Envelope<**Fs: RecordFields> { id: Int, **Fs }\n",
        "data Envelope<**Fs: RecordFields> { id: Int, tag: String, **Fs }\n",
        "data Envelope<**Fs: RecordFields> { **Fs }\n",
    ];
    let owner = DeclarationId::new(module.clone(), "Envelope".into());
    let mut session = SemanticWorkspaceSession::new();
    let mut previous_fingerprint = None;
    for (index, source) in revisions.iter().enumerate() {
        let update = session.update(single_module_input(module.clone(), source, index as u64 + 1));
        assert!(!update.snapshot.has_errors(), "revision {index}: {:?}", update.snapshot.diagnostics);
        let cold = SemanticWorkspaceSession::new().update(single_module_input(module.clone(), source, 1));
        assert_eq!(diagnostic_summary(&update.snapshot), diagnostic_summary(&cold.snapshot));
        let incremental_shape = data_shape_summary(&update.snapshot, &owner);
        assert_eq!(incremental_shape, data_shape_summary(&cold.snapshot, &owner));

        let fingerprint = session
            .db()
            .ready_product_fingerprint(&QueryKey::DataDeclaration(owner.clone()))
            .expect("data declaration product fingerprint");
        if let Some(previous) = previous_fingerprint {
            assert_ne!(previous, fingerprint, "each fixed-shape edit must invalidate the data product");
        }
        previous_fingerprint = Some(fingerprint);
    }
}

fn type_member_summary(snapshot: &SemanticSnapshot, owner: &DeclarationId) -> Vec<(String, String, String)> {
    snapshot
        .type_member_declarations
        .iter()
        .filter(|(id, _)| &id.owner == owner)
        .map(|(id, product)| {
            (
                id.selector.name().to_string(),
                format!("{:?}", product.signature.result_domain),
                format!("{:?}", product.signature.intrinsic),
            )
        })
        .collect()
}

fn assert_universe_member_edit_parity(path: &[&str], owner_name: &str, source_a: &str, source_b: &str, member_name: &str) {
    let module = ModuleId::universe(ModulePath::from_components(
        path.iter()
            .map(|part| ModuleComponent::from_identifier(part).expect("valid path component"))
            .collect::<Vec<_>>(),
    ));
    let owner = DeclarationId::new(module.clone(), owner_name.into());
    let mut incremental = SemanticWorkspaceSession::new();
    let initial = incremental.update(single_module_input(module.clone(), source_a, 1));
    let updated = incremental.update(single_module_input(module.clone(), source_b, 2));
    let cold = SemanticWorkspaceSession::new().update(single_module_input(module.clone(), source_b, 1));
    assert_eq!(diagnostic_summary(&updated.snapshot), diagnostic_summary(&cold.snapshot));
    assert_eq!(
        type_member_summary(&updated.snapshot, &owner),
        type_member_summary(&cold.snapshot, &owner),
        "edited intrinsic member product must match cold source analysis"
    );
    assert!(
        initial.snapshot.has_errors() || !updated.snapshot.has_errors() || !cold.snapshot.has_errors(),
        "the source edit must publish either a member or a matching diagnostic"
    );
    assert!(
        updated.snapshot.all_diagnostics().any(|diagnostic| diagnostic.message.contains(member_name))
            || updated.snapshot.type_member_declarations.keys().any(|id| id.selector.name() == member_name),
        "the edited member must be represented by a product or its diagnostic"
    );
}

#[test]
fn t12_6_tuple_and_record_intrinsic_member_result_kind_edits_match_cold() {
    let tuple_a = r#"
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data CompleteTypePack
intrinsic data Tuple<***Es: CompleteTypePack>(***Es)
impl<***Es: CompleteTypePack> Tuple<***Es> { intrinsic type Element: Type }
"#;
    let tuple_b = tuple_a.replace("Element: Type", "Element: CompleteTypePack");
    assert_universe_member_edit_parity(&["collections", "tuple"], "Tuple", tuple_a, &tuple_b, "Element");

    let record_a = r#"
intrinsic type data RecordFields
intrinsic data Record<**Fs: RecordFields> { **Fs }
impl<**Fs: RecordFields> Record<**Fs> { intrinsic type Value: Type }
"#;
    let record_b = record_a.replace("Value: Type", "Value: RecordFields");
    assert_universe_member_edit_parity(&["collections", "record"], "Record", record_a, &record_b, "Value");
}

fn callable_body_summary(snapshot: &SemanticSnapshot, source: &str, callable: &CallableId) -> Vec<(String, Option<String>, String)> {
    let analysis = snapshot.callable_analyses.get(callable).expect("callable analysis");
    let mut expressions = analysis
        .expressions
        .values()
        .filter_map(|expression| {
            let text = source.get(expression.range.start..expression.range.end)?.trim().to_owned();
            let ty = expression.knowledge.ty().map(|ty| snapshot.store.format_type(ty));
            Some((text, ty, format!("{:?}", expression.knowledge.status())))
        })
        .collect::<Vec<_>>();
    expressions.sort();
    expressions
}

#[test]
fn t12_6_record_literal_field_and_spread_edit_matches_cold_analysis() {
    let module = module("t12_6_record_body");
    let source_a = r#"
class Probe {
  @class build() {
    let value = { name: "Ada", age: 42 }
    value
  }
}
"#;
    let source_b = r#"
class Probe {
  @class build() {
    let extra = { age: "42", active: true }
    let value = { name: "Ada", **extra }
    value
  }
}
"#;
    let callable = CallableId::new(
        DeclarationId::new(module.clone(), "Probe".into()),
        Selector::method("build", []).expect("build selector"),
        DispatchSide::Class,
    );
    let body_key = QueryKey::CallableBody(callable.clone());
    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental.update(single_module_input(module.clone(), source_a, 1));
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    let first_analysis = first.snapshot.callable_analyses.get(&callable).expect("initial body").clone();
    let first_fp = incremental.db().ready_product_fingerprint(&body_key).expect("initial body fingerprint");
    let updated = incremental.update(single_module_input(module.clone(), source_b, 2));
    let cold = SemanticWorkspaceSession::new().update(single_module_input(module.clone(), source_b, 1));
    assert_eq!(diagnostic_summary(&updated.snapshot), diagnostic_summary(&cold.snapshot));
    assert_eq!(
        callable_body_summary(&updated.snapshot, source_b, &callable),
        callable_body_summary(&cold.snapshot, source_b, &callable)
    );
    assert!(!Arc::ptr_eq(&first_analysis, updated.snapshot.callable_analyses.get(&callable).expect("updated body")));
    assert_ne!(first_fp, incremental.db().ready_product_fingerprint(&body_key).expect("updated body fingerprint"));
    assert!(incremental.db().revision_recomputed_keys().any(|key| key == &body_key));
}

#[test]
fn t12_6_unit_empty_record_alias_consumer_edit_matches_cold_analysis() {
    let root = tempfile::tempdir().expect("temporary semantic project");
    std::fs::write(root.path().join("package.ph"), "").expect("package marker");
    let provider_path = root.path().join("provider.ph");
    let consumer_path = root.path().join("consumer.ph");
    let location = |path: &std::path::Path| SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path.to_path_buf(),
    };
    let provider = location(&provider_path);
    let consumer = location(&consumer_path);
    let source_a = r#"
type Zero = Unit
export Zero
"#;
    let source_b = r#"
from universe.collections.record import EmptyRecord
type Zero = EmptyRecord
export Zero
"#;
    let consumer_source = r#"
from .provider import Zero
class Consumer {
  @class identity(_ value: Zero) -> Zero { value }
}
export Consumer
"#;
    std::fs::write(&provider_path, source_a).expect("initial alias source file");
    std::fs::write(&consumer_path, consumer_source).expect("consumer source file");
    for source in [source_a, source_b, consumer_source] {
        let parsed = phalcom_ast::parse(source, 0);
        assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    }
    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: provider.clone(),
                text: Arc::from(source_a),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: consumer.clone(),
                text: Arc::from(consumer_source),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("initial canonical-Universe consumer publication");
    assert!(!first.snapshot.has_errors(), "Unit consumer diagnostics: {:?}", first.snapshot.diagnostics);
    let owner = first
        .snapshot
        .callable_signatures
        .iter()
        .map(|(id, _)| id)
        .find(|id| id.declaration_owner().name.as_ref() == "Consumer")
        .map(|id| id.declaration_owner().clone())
        .unwrap_or_else(|| {
            let source_module = first.snapshot.sources.keys().find(|module| module.path.to_string().contains("consumer"));
            let local_aliases = first
                .snapshot
                .type_aliases
                .iter()
                .filter(|(id, _)| id.name.as_ref() == "Zero")
                .map(|(id, _)| id)
                .collect::<Vec<_>>();
            let local_declarations = source_module.map(|module| {
                ["Zero", "Consumer"]
                    .into_iter()
                    .map(|name| first.snapshot.declarations.get(&DeclarationId::new(module.clone(), name.into())).is_some())
                    .collect::<Vec<_>>()
            });
            panic!(
                "overlay source produced no Consumer signature; source ids={:?}; source paths={:?}; Zero aliases={local_aliases:#?}; local declaration presence [Zero, Consumer]={local_declarations:?}; diagnostics={:?}",
                first.snapshot.sources.keys().collect::<Vec<_>>(),
                first
                    .snapshot
                    .sources
                    .values()
                    .filter_map(|source| source.source.as_ref().map(|source| source.display_path.clone()))
                    .collect::<Vec<_>>(),
                first.snapshot.diagnostics
            )
        });
    let provider_module = first
        .snapshot
        .module_for_display_path(&provider_path)
        .expect("provider module identity")
        .clone();
    let alias = DeclarationId::new(provider_module.clone(), "Zero".into());
    let callable = CallableId::new(
        owner,
        Selector::method("identity", [phalcom_common::selector::SelectorSlot::Positional]).expect("identity selector"),
        DispatchSide::Class,
    );
    let first_alias = first.snapshot.type_aliases.form(&alias).expect("initial source alias Zero");
    assert_eq!(first_alias, first.snapshot.store.unit(), "Unit alias resolves to canonical zero-product Tuple");
    let first_signature = first.snapshot.callable_signatures.get(&callable).expect("initial Consumer signature");
    let first_return = first_signature.declared_return.canonical_type().expect("initial return type");
    let updated = incremental
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: provider.clone(),
            text: Arc::from(source_b),
            revision: SourceRevision(2),
            recovered_program: None,
        }])
        .expect("updated canonical-Universe consumer publication");
    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: provider,
                text: Arc::from(source_b),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: consumer,
                text: Arc::from(consumer_source),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("cold canonical-Universe consumer publication");
    assert!(
        updated.snapshot.type_aliases.get(&alias).is_some(),
        "updated Zero alias absent; sources={:?}; diagnostics={:?}",
        updated.snapshot.sources.keys().collect::<Vec<_>>(),
        diagnostic_summary(&updated.snapshot)
    );
    assert!(
        cold.snapshot.type_aliases.get(&alias).is_some(),
        "cold Zero alias absent; sources={:?}; diagnostics={:?}",
        cold.snapshot.sources.keys().collect::<Vec<_>>(),
        diagnostic_summary(&cold.snapshot)
    );
    assert_eq!(diagnostic_summary(&updated.snapshot), diagnostic_summary(&cold.snapshot));
    assert_eq!(
        alias_summary(&updated.snapshot, &alias),
        alias_summary(&cold.snapshot, &alias),
        "updated Unit/EmptyRecord alias consumer must match cold semantics"
    );
    let updated_signature = updated.snapshot.callable_signatures.get(&callable).expect("updated consumer signature");
    let cold_signature = cold.snapshot.callable_signatures.get(&callable).expect("cold consumer signature");
    let updated_return = updated_signature.declared_return.canonical_type().expect("updated return type");
    let cold_return = cold_signature.declared_return.canonical_type().expect("cold return type");
    let mut updated_store = (*updated.snapshot.store).clone();
    let canonical_empty_record = updated_store.record(Box::new([]));
    assert_eq!(
        updated.snapshot.type_aliases.form(&alias),
        Some(canonical_empty_record),
        "EmptyRecord alias resolves to canonical zero-field Record"
    );
    let mut cold_store = (*cold.snapshot.store).clone();
    let cold_empty_record = cold_store.record(Box::new([]));
    assert_eq!(
        cold.snapshot.store.format_type(cold.snapshot.type_aliases.form(&alias).expect("cold Zero alias")),
        cold_store.format_type(cold_empty_record),
        "cold alias also resolves to canonical zero-field Record"
    );
    assert_eq!(updated.snapshot.store.format_type(updated_return), cold.snapshot.store.format_type(cold_return));
    assert_ne!(first.snapshot.store.format_type(first_return), updated.snapshot.store.format_type(updated_return));
}
