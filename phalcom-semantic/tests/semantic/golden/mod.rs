//! Plan 3 golden semantic composition fixtures.
//!
//! These tests are ordinary modules of the canonical `semantic` integration
//! target and share the same assertion/fixture infrastructure as the focused
//! semantic capability suite.

use crate::semantic::support::{Fixture, WorkspaceFixture, assert_source_contract, binding, known, union};
use phalcom_ast::parse;
use phalcom_semantic::checker::BindingConsistency;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceStatus;
use std::path::Path;

const GOLDEN_01: &str = include_str!("../../golden/generic_self_chain.ph");
const GOLDEN_02: &str = include_str!("../../golden/flow_pattern_publication.ph");
const GOLDEN_03: &str = include_str!("../../golden/iterator_chain.ph");
const GOLDEN_04: &str = include_str!("../../golden/family_callable.ph");
const GOLDEN_05: &str = include_str!("../../golden/type_lambda_constraints.ph");
const GOLDEN_07: &str = include_str!("../../golden/unknown_authority.ph");
const GOLDEN_08: &str = include_str!("../../golden/variance_recovery.ph");
const GOLDEN_09: &str = include_str!("../../golden/closure_flow.ph");
const GOLDEN_10: &str = include_str!("../../golden/mixed_pipeline.ph");

fn assert_parses(name: &str, source: &str) {
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "{name} must parse: {:#?}", parsed.errors);
}

fn workspace_sources() -> [(&'static str, &'static str); 4] {
    [
        ("app/model.ph", include_str!("../../golden/workspace_chain/app/model.ph")),
        ("app/repository.ph", include_str!("../../golden/workspace_chain/app/repository.ph")),
        ("app/service.ph", include_str!("../../golden/workspace_chain/app/service.ph")),
        ("app/controller.ph", include_str!("../../golden/workspace_chain/app/controller.ph")),
    ]
}

#[test]
fn golden_01_program_parses() {
    assert_parses("GOLDEN-01", GOLDEN_01);
}

#[test]
#[ignore = "semantic gate waits for generic inheritance and nested Self specialization"]
fn golden_01_generic_self_chain() {
    let f = Fixture::new(GOLDEN_01);
    let animal = f.ty("Animal");
    let cat = f.ty("Cat");
    let box_animal = f.ty("Box");
    let service = f.callable("Service", "makeCat", DispatchSide::Class);
    let probe = f.callable("Probe", "run", DispatchSide::Class);
    let wrap = f.callable_id("Maker", "wrap", DispatchSide::Instance);
    let echo = f.callable_id("AnimalMaker", "echo", DispatchSide::Instance);
    let boxed = f.callable_id("SelfNode", "boxed", DispatchSide::Instance);
    let value = f.callable_id("Box", "value", DispatchSide::Instance);

    f.assert_subtype(cat, animal);
    assert_eq!(wrap.owner.name.as_ref(), "Maker");
    assert_eq!(echo.owner.name.as_ref(), "AnimalMaker");
    assert_eq!(boxed.owner.name.as_ref(), "SelfNode");
    assert_eq!(value.owner.name.as_ref(), "Box");
    f.assert_binding_expectation(probe, "animals", binding().declared(box_animal).current(known(box_animal)));
    assert!(f.binding(probe, "nodeBox").current.ty().is_some());
    assert!(f.binding(probe, "node").current.ty().is_some());
    assert_ne!(f.binding(probe, "animals").binding, f.binding(probe, "animal").binding);
    assert!(service.dependencies.len() >= 2, "call chain should retain dependencies: {service:#?}");
    assert!(probe.dependencies.len() >= 2, "Probe should retain published dependencies: {probe:#?}");
    f.assert_no_error_diagnostics();
}

#[test]
#[ignore = "semantic gate waits for component-wise product joins across nominal branch results"]
fn golden_02_flow_pattern_publication() {
    assert_parses("GOLDEN-02", GOLDEN_02);
    let f = Fixture::new(GOLDEN_02);
    let cat = f.ty("Cat");
    let dog = f.ty("Dog");
    let int = f.ty("Int");
    let service = f.callable("Service", "choose", DispatchSide::Class);
    let probe = f.callable("Probe", "run", DispatchSide::Class);
    let pair_callable = f.callable_id("Factory", "pair", DispatchSide::Class);

    let pair = f.binding(service, "pair").current.ty().expect("pair type");
    let phalcom_semantic::types::store::TypeData::Tuple(elements) = f.analysis.snapshot.store.get(pair) else {
        panic!("Factory.pair should publish tuple, got {:?}", f.analysis.snapshot.store.get(pair));
    };
    f.assert_type(elements[0].ty, union([cat.into(), dog.into()]));
    assert_eq!(elements[1].ty, int);
    f.assert_binding_type(service, "result", pair);
    f.assert_type(f.binding(probe, "animal").current.ty().expect("animal type"), union([cat.into(), dog.into()]));
    f.assert_binding_type(probe, "count", int);
    f.assert_expression_call_target(f.expression(service, "Factory.pair(flag)"), &pair_callable);
    assert_ne!(f.binding(service, "animal").binding, f.binding(service, "count").binding);
    assert!(f.binding(service, "result").current.status().is_some());
    f.assert_callable_dependency(service, &pair_callable);
    assert!(service.dependencies.iter().any(|dependency| dependency.owner.name.as_ref() == "Factory"));
    f.assert_no_error_diagnostics();
}

#[test]
fn golden_03_program_parses() {
    assert_parses("GOLDEN-03", GOLDEN_03);
}

#[test]
#[ignore = "semantic gate follows iteration protocol stabilization"]
fn golden_03_iterator_chain() {
    let f = Fixture::new(GOLDEN_03);
    let string = f.ty("String");
    let int = f.ty("Int");
    let service = f.callable("Service", "collect", DispatchSide::Class);
    let probe = f.callable("Probe", "run", DispatchSide::Class);
    let decode = f.callable_id("Decoder", "decode", DispatchSide::Class);

    f.assert_binding_type(service, "entry", f.ty("Entry"));
    f.assert_binding_type(service, "key", string);
    f.assert_binding_type(service, "value", int);
    f.assert_binding_type(service, "last", int);
    f.assert_binding_type(probe, "last", int);
    f.assert_expression_call_target(f.expression(service, "Decoder.decode(entry.value())"), &decode);
    f.assert_expression_ready(f.expression(service, "Decoder.decode(entry.value())"));
    assert!(f.binding(service, "last").mutable);
    assert!(f.binding(probe, "result").current.ty().is_some());
    assert!(f.binding(probe, "status").current.ty().is_some());
    assert!(!service.semantic_dependencies.is_empty());
    f.assert_no_error_diagnostics();
}

#[test]
fn golden_04_program_parses() {
    assert_parses("GOLDEN-04", GOLDEN_04);
}

#[test]
#[ignore = "semantic gate waits for formal Family activation and overloaded callable routes"]
fn golden_04_family_callable() {
    let f = Fixture::new(GOLDEN_04);
    let string = f.ty("String");
    let service = f.callable("Service", "run", DispatchSide::Class);
    let router = f.callable("Router", "use", DispatchSide::Class);
    let use_callable = f.callable_id("Router", "use", DispatchSide::Class);

    assert!(f.binding(service, "exact").current.ty().is_some());
    assert!(f.binding(service, "pattern").current.ty().is_some());
    f.assert_binding_type(service, "a", string);
    f.assert_binding_type(service, "b", string);
    f.assert_binding_type(service, "c", string);
    f.assert_expression_call_target(f.expression(service, "Router.use(exact, 42)"), &use_callable);
    f.assert_expression_ready(f.expression(service, "pattern(value: \"x\")"));
    f.assert_expression_ready(f.expression(service, "pattern()"));
    assert!(router.dependencies.is_empty());
    assert!(service.dependencies.iter().any(|dependency| dependency.owner.name.as_ref() == "Router"));
    f.assert_no_error_diagnostics();
}

#[test]
#[ignore = "parser prerequisite: source type-lambda declaration syntax is not accepted yet"]
fn golden_05_program_parses() {
    assert_parses("GOLDEN-05", GOLDEN_05);
}

#[test]
#[ignore = "parser prerequisite: source type-lambda syntax must land before constraint semantics can run"]
fn golden_05_type_lambda_constraints() {
    let _ = Fixture::new(GOLDEN_05);
}

#[test]
fn golden_06_workspace_programs_parse() {
    for (name, source) in workspace_sources() {
        assert_parses(name, source);
    }
}

#[test]
#[ignore = "semantic gate waits for linked multi-module export/import publication"]
fn golden_06_workspace_chain() {
    let mut workspace = WorkspaceFixture::new().entry("app.controller");
    for (name, source) in workspace_sources() {
        let module = match name {
            "app/model.ph" => "app.model",
            "app/repository.ph" => "app.repository",
            "app/service.ph" => "app.service",
            "app/controller.ph" => "app.controller",
            _ => unreachable!(),
        };
        workspace = workspace.module(module, source);
    }
    let analyzed = workspace.analyze();
    assert!(analyzed.analysis.snapshot.sources.len() >= 4);
    assert!(analyzed.analysis.snapshot.module_products.linked.len() >= 4);
    assert!(analyzed.analysis.snapshot.surfaces.contains_key(&analyzed.decl("app.model", "Entity")));
    assert!(analyzed.analysis.snapshot.surfaces.contains_key(&analyzed.decl("app.model", "User")));
    assert!(
        analyzed
            .analysis
            .snapshot
            .surfaces
            .contains_key(&analyzed.decl("app.repository", "UserRepository"))
    );
    assert!(analyzed.analysis.snapshot.surfaces.contains_key(&analyzed.decl("app.service", "UserService")));
    assert!(analyzed.analysis.snapshot.surfaces.contains_key(&analyzed.decl("app.controller", "Controller")));
    assert_ne!(analyzed.module("app.model"), analyzed.module("app.repository"));
    assert_ne!(analyzed.module("app.repository"), analyzed.module("app.service"));
    assert_ne!(analyzed.module("app.service"), analyzed.module("app.controller"));
    assert!(analyzed.analysis.snapshot.module_products.resolved_imports.len() >= 2);
}

#[test]
fn golden_07_unknown_authority() {
    assert_parses("GOLDEN-07", GOLDEN_07);
    let f = Fixture::new(GOLDEN_07);
    let cell_num = f.ty("CellNum");
    let int = f.ty("Int");
    let run = f.callable("Service", "run", DispatchSide::Class);
    let known_callable = f.callable_id("Factory", "known", DispatchSide::Class);
    let value_callable = f.callable_id("CellNum", "value", DispatchSide::Instance);

    let certain = f.binding(run, "certain");
    assert_source_contract(certain, cell_num);
    assert_eq!(certain.current.status(), Some(EvidenceStatus::Established));

    let uncertain = f.binding(run, "uncertain");
    assert_source_contract(uncertain, cell_num);
    assert_ne!(uncertain.current.status(), Some(EvidenceStatus::Established));

    f.assert_expression_call_target(f.expression(run, "Factory.known()"), &known_callable);
    f.assert_expression_call_target(f.expression(run, "certain.value()"), &value_callable);
    f.assert_expression_established(f.expression(run, "certain.value()"), int);
    let selected = f.binding(run, "selected");
    assert!(selected.current.ty().is_none());
    assert_ne!(selected.current.status(), Some(EvidenceStatus::Established));
    assert_ne!(certain.binding, uncertain.binding);
    assert!(run.dependencies.len() >= 2);
    f.assert_binding_type(run, "independent", int);
    f.assert_no_error_diagnostics();
}

#[test]
#[ignore = "semantic gate waits for call-summary refutation recovery through generic variance"]
fn golden_08_variance_recovery() {
    assert_parses("GOLDEN-08", GOLDEN_08);
    let f = Fixture::new(GOLDEN_08);
    let animal = f.ty("Animal");
    let int = f.ty("Int");
    let run = f.callable("Service", "run", DispatchSide::Class);
    let bad = f.binding(run, "bad");

    assert!(f.binding(run, "producer").declared_type().is_some());
    assert!(f.binding(run, "producer").current.ty().is_some());
    assert!(f.binding(run, "animal").current.ty().is_some());
    assert!(bad.declared_type().is_some());
    assert!(
        matches!(bad.consistency, BindingConsistency::Refuted { .. }),
        "bad contract must be refuted: {bad:#?}"
    );
    assert!(f.binding(run, "independent").current.ty().is_some());
    f.assert_subtype(animal, animal);
    assert!(run.dependencies.len() >= 2);
    f.assert_binding_type(run, "independent", int);
    f.assert_diagnostic(DiagnosticCode::BindingInitializerMismatch, 1);
    f.assert_only_error_codes(&[DiagnosticCode::BindingInitializerMismatch]);
}

#[test]
fn golden_09_program_parses() {
    assert_parses("GOLDEN-09", GOLDEN_09);
}

#[test]
#[ignore = "semantic gate waits for closure contextual typing and capture publication"]
fn golden_09_closure_flow() {
    let f = Fixture::new(GOLDEN_09);
    let int = f.ty("Int");
    let service = f.callable("Service", "run", DispatchSide::Class);
    let apply = f.callable_id("Apply", "apply", DispatchSide::Class);

    f.assert_binding_established(service, "base", int);
    assert!(f.binding(service, "transform").current.ty().is_some());
    f.assert_binding_established(service, "result", int);
    f.assert_binding_established(service, "stillBase", int);
    f.assert_expression_call_target(f.expression(service, "Apply.apply(42, with: transform)"), &apply);
    f.assert_expression_ready(f.expression(service, "Apply.apply(42, with: transform)"));
    assert_eq!(f.bindings_named(service, "base").len(), 1);
    assert_ne!(f.binding(service, "base").binding, f.binding(service, "transform").binding);
    assert!(service.dependencies.iter().any(|dependency| dependency.owner.name.as_ref() == "Apply"));
    f.assert_no_error_diagnostics();
}

#[test]
#[ignore = "semantic gate waits for structural-record argument recovery and single-diagnostic ownership"]
fn golden_10_mixed_pipeline() {
    assert_parses("GOLDEN-10", GOLDEN_10);
    let f = Fixture::new(GOLDEN_10);
    let string = f.ty("String");
    let int = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let pipeline = f.callable_id("Pipeline", "fetch", DispatchSide::Class);
    let presenter = f.callable_id("Presenter", "present", DispatchSide::Class);

    let record = f.binding(run, "record").current.ty().expect("published record type");
    f.assert_record(record);
    let bad = f.binding(run, "bad");
    assert!(
        matches!(bad.consistency, BindingConsistency::Refuted { .. }),
        "record/string contract must be refuted: {bad:#?}"
    );
    f.assert_binding_type(run, "label", string);
    f.assert_binding_type(run, "count", int);
    f.assert_binding_type(run, "independent", int);
    let presented = f.binding(run, "presented").current.ty().expect("presented tuple");
    f.assert_tuple_types(presented, &[string, int]);
    f.assert_expression_call_target(f.expression(run, "Pipeline.fetch(repo, fallback)"), &pipeline);
    f.assert_expression_call_target(f.expression(run, "Presenter.present(record)"), &presenter);
    assert!(run.dependencies.len() >= 3);
    assert!(bad.current.ty().is_some());
    f.assert_diagnostic(DiagnosticCode::BindingInitializerMismatch, 1);
    f.assert_only_error_codes(&[DiagnosticCode::BindingInitializerMismatch]);
}

#[test]
fn golden_sources_are_part_of_canonical_semantic_target() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let golden_dir = manifest_dir.join("tests/golden");
    assert!(golden_dir.is_dir());
    for file in [
        "generic_self_chain.ph",
        "flow_pattern_publication.ph",
        "iterator_chain.ph",
        "family_callable.ph",
        "type_lambda_constraints.ph",
        "unknown_authority.ph",
        "variance_recovery.ph",
        "closure_flow.ph",
        "mixed_pipeline.ph",
    ] {
        assert!(golden_dir.join(file).is_file(), "missing golden source {file}");
    }
}
