//! Incremental invalidation for contextual algebraic realization capabilities.

use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::{GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, SymbolId};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::checker::ContextualRealization;
use phalcom_semantic::db::{CancellationToken, QueryBudget};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::{SemanticWorkspaceSession, SemanticWorkspaceUpdate};
use phalcom_semantic::snapshot::SemanticSnapshot;
use std::sync::Arc;

fn module() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(310),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").expect("module component")]),
    )
}

fn source(my_type_conformance: bool, other_type_conformance: bool) -> String {
    let mut source = String::from(
        r#"
from universe.numbers.algebra import BasisRealizable, Basis, I

data MyType(_ real: Int, _ imag: Int)
data OtherType(_ real: Int, _ imag: Int)
class Probe {
  @class build() -> MyType { 1i }
}
"#,
    );
    if my_type_conformance {
        source.push_str(
            r#"
impl BasisRealizable<Basis<I>> for MyType {
  type Scalar = Int
  @class along(_ coefficient: Int, _ basis: Basis<I>) -> MyType { MyType(coefficient, 0) }
}
"#,
        );
    }
    if other_type_conformance {
        source.push_str(
            r#"
impl BasisRealizable<Basis<I>> for OtherType {
  type Scalar = Int
  @class along(_ coefficient: Int, _ basis: Basis<I>) -> OtherType { OtherType(coefficient, 0) }
}
"#,
        );
    }
    source
}

fn input(module: ModuleId, source: &str, generation: u64) -> phalcom_semantic::workspace::SemanticWorkspaceInput {
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "fixture parse errors: {:#?}\n{source}", parsed.errors);
    let program = Arc::new(parsed.program);
    let globals = phalcom_modules::interface::InterfaceBuilder::build(module.clone(), ModuleKind::Module, &program)
        .expect("fixture interface")
        .declarations
        .keys()
        .enumerate()
        .map(|(index, name)| (name.clone().into_boxed_str(), GlobalBindingId(index as u32)))
        .collect();
    let algebra = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("numbers").expect("module component"),
        ModuleComponent::from_identifier("algebra").expect("module component"),
    ]));
    let algebra_source = include_str!("../../../../phalcom-core/core/universe/src/numbers/algebra.ph");
    let algebra_parsed = phalcom_ast::parse(algebra_source, 0);
    assert!(algebra_parsed.errors.is_empty(), "canonical algebra module parse errors: {:#?}", algebra_parsed.errors);
    let algebra_program = Arc::new(algebra_parsed.program);
    let algebra_globals = phalcom_modules::interface::InterfaceBuilder::build(algebra.clone(), ModuleKind::Module, &algebra_program)
        .expect("canonical algebra interface")
        .declarations
        .keys()
        .enumerate()
        .map(|(index, name)| (name.clone().into_boxed_str(), GlobalBindingId(index as u32)))
        .collect();
    let export = |name: &str| {
        (
            name.into(),
            LinkedExport {
                public_name: name.into(),
                target: LinkedExportTarget::Binding(SymbolId {
                    module: algebra.clone(),
                    name: name.into(),
                }),
                range: phalcom_common::range::SourceRange::default(),
            },
        )
    };
    let algebra_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: algebra.clone(),
            kind: ModuleKind::Module,
            exports: ["BasisRealizable", "Basis", "I"].into_iter().map(export).collect(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: algebra_globals,
            ..ModuleBindingLayout::default()
        },
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let client = LinkedModule {
        interface: LinkedModuleInterface {
            module: module.clone(),
            kind: ModuleKind::Module,
            exports: Default::default(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: globals,
            imports: [
                ("BasisRealizable".into(), ImportBindingId(0)),
                ("Basis".into(), ImportBindingId(1)),
                ("I".into(), ImportBindingId(2)),
            ]
            .into_iter()
            .collect(),
        },
        linked_reads: ["BasisRealizable", "Basis", "I"]
            .into_iter()
            .map(|name| LinkedReadSpec::Binding(SymbolId {
                module: algebra.clone(),
                name: name.into(),
            }))
            .collect(),
        runtime_dependencies: vec![algebra.clone()],
    };
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules: [(algebra.clone(), algebra_module), (module.clone(), client)].into_iter().collect(),
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module.clone(),
        initialization_order: vec![algebra.clone(), module.clone()],
    });
    let parsed_unit = Arc::new(phalcom_semantic::source::ParsedModuleUnit::new(
        module.clone(),
        ModuleKind::Module,
        None,
        Arc::from(source),
        program,
    ));
    let algebra_unit = Arc::new(phalcom_semantic::source::ParsedModuleUnit::new(
        algebra.clone(),
        ModuleKind::Module,
        None,
        Arc::from(algebra_source),
        algebra_program,
    ));
    phalcom_semantic::workspace::SemanticWorkspaceInput::new(
        linked,
        [(algebra, algebra_unit), (module, parsed_unit)].into_iter().collect(),
        generation,
    )
}

fn update(session: &mut SemanticWorkspaceSession, module: ModuleId, source: &str, generation: u64) -> SemanticWorkspaceUpdate {
    session.update_with_budget_and_cancel(input(module, source, generation), QueryBudget::default(), &CancellationToken::new()).unwrap_or_else(
        |error| {
            let blocked = session
                .db()
                .query_keys()
                .filter_map(|key| {
                    session
                        .db()
                        .query_state(key)
                        .filter(|state| matches!(state, phalcom_semantic::db::QueryState::Blocked { .. }))
                        .map(|state| (key, state))
                })
                .collect::<Vec<_>>();
            panic!("semantic update failed at generation {generation}: {error:?}; blocked queries={blocked:#?}\n{source}")
        },
    )
}

fn build_callable(module: &ModuleId) -> CallableId {
    CallableId::new(
        DeclarationId::new(module.clone(), "Probe".into()),
        Selector::method("build", []).expect("build selector"),
        DispatchSide::Class,
    )
}

fn build_analysis<'a>(snapshot: &'a SemanticSnapshot, module: &ModuleId) -> &'a Arc<phalcom_semantic::checker::CallableAnalysis> {
    snapshot
        .callable_analyses
        .get(&build_callable(module))
        .expect("Probe.build analysis")
}

fn realization<'a>(snapshot: &'a SemanticSnapshot, module: &ModuleId, source: &str) -> Option<&'a ContextualRealization> {
    build_analysis(snapshot, module)
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("1i"))
        .expect("imaginary literal expression")
        .contextual_realization
        .as_deref()
}

#[test]
fn add_conformance_invalidates_negative_dispatch_bucket() {
    let module = module();
    let source_without = source(false, false);
    let source_with = source(true, false);
    let parsed = phalcom_ast::parse(&source_with, 0);
    assert!(parsed.errors.is_empty(), "fixture parse errors: {:#?}\n{}", parsed.errors, source_with);
    let mut session = SemanticWorkspaceSession::new();

    let first = update(&mut session, module.clone(), &source_without, 1);
    assert!(
        realization(&first.snapshot, &module, &source_without).is_none(),
        "a basis literal without a conformance must not publish a realization"
    );

    let updated = update(&mut session, module.clone(), &source_with, 2);
    let incremental = realization(&updated.snapshot, &module, &source_with).unwrap_or_else(|| {
        panic!(
            "new conformance enables realization; diagnostics={:#?}; conformance count={}; buckets={:#?}",
            updated.snapshot.all_diagnostics().collect::<Vec<_>>(),
            updated.snapshot.conformance_index.iter().count(),
            updated.snapshot.trait_dispatch_index.buckets(),
        )
    });
    assert_eq!(
        updated.snapshot.store.nominal_origin_declaration(incremental.target_type),
        Some(&DeclarationId::new(module.clone(), "MyType".into()))
    );

    let cold = update(&mut SemanticWorkspaceSession::new(), module.clone(), &source_with, 1);
    assert_eq!(
        realization(&cold.snapshot, &module, &source_with),
        Some(incremental),
        "incremental realization must agree with analysis from a fresh session"
    );
}

#[test]
fn remove_conformance_invalidates_positive_dispatch_bucket() {
    let module = module();
    let source_with = source(true, false);
    let source_without = source(false, false);
    let mut session = SemanticWorkspaceSession::new();

    let first = update(&mut session, module.clone(), &source_with, 1);
    assert!(realization(&first.snapshot, &module, &source_with).is_some(), "initial conformance must enable realization");

    let updated = update(&mut session, module.clone(), &source_without, 2);
    assert!(
        realization(&updated.snapshot, &module, &source_without).is_none(),
        "removing the only matching conformance must remove the realization"
    );

    let cold = update(&mut SemanticWorkspaceSession::new(), module.clone(), &source_without, 1);
    assert_eq!(
        realization(&cold.snapshot, &module, &source_without),
        realization(&updated.snapshot, &module, &source_without),
        "incremental absence must agree with analysis from a fresh session"
    );
}

#[test]
fn unrelated_conformance_keeps_realization_consumer_reusable() {
    let module = module();
    let source_before = source(true, false);
    let source_after = source(true, true);
    let mut session = SemanticWorkspaceSession::new();

    let first = update(&mut session, module.clone(), &source_before, 1);
    let callable = build_callable(&module);
    first.snapshot.callable_analyses.get(&callable).expect("initial consumer analysis");
    assert!(realization(&first.snapshot, &module, &source_before).is_some());

    let updated = update(&mut session, module.clone(), &source_after, 2);
    updated.snapshot.callable_analyses.get(&callable).expect("updated consumer analysis");
    assert!(
        !updated
            .recomputed
            .contains(&phalcom_semantic::db::QueryKey::CallableBody(callable.clone())),
        "a conformance in another target-family bucket must not re-execute this consumer; recomputed={:?}",
        updated.recomputed
    );
    assert!(realization(&updated.snapshot, &module, &source_after).is_some());
}
