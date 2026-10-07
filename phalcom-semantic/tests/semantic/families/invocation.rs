use std::sync::Arc;

use crate::semantic::support::WorkspaceFixture;
use phalcom_common::selector::{SelectorKind, SelectorSlot};
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::InterfaceBuilder;
use phalcom_modules::linker::ModuleLinker;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::checker::{CallableReferenceResolutionKind, FamilyApplicationResolution, FamilyApplicationSelection};
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, InvocationTargetId};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::types::denotation::SemanticDenotation;
use phalcom_semantic::types::store::TypeData;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;

#[test]
fn associated_equality_generic_family_routes_forward_caller_evidence() {
    let client_source = r#"
from .facade import snapshot
from .helpers import Iterable
from .helpers import Floats, Helpers
import .helpers as helpers
class Client {
  @class run<I>(_ input: I) -> Float where I |= Iterable<Item = Float> {
    let local = snapshot
    let a = local(input)
    let b = helpers.snapshot(input)
    let c = snapshot(input)
    c
  }
}
let concrete = Floats.new()
let direct = Helpers.snapshot(concrete)
let localFamily = &Helpers.snapshot(_)
let localResult = localFamily(concrete)
let importedResult = snapshot(concrete)
"#;
    let fixture = WorkspaceFixture::new()
        .module(
            "helpers",
            r#"
trait Iterable { type Item }
class Helpers {
  @class snapshot<J>(_ value: J) -> Float where J |= Iterable<Item = Float> { 0.0 }
  @class local<I>(_ input: I) -> Float where I |= Iterable<Item = Float> {
    let family = &Helpers.snapshot(_)
    let values = family(input)
    values
  }
}
let snapshot = &Helpers.snapshot(_)
class Floats { @constructor new() {} }
impl Iterable for Floats { type Item = Float }
export snapshot, Iterable, Floats, Helpers
"#,
        )
        .module("facade", "from .helpers import snapshot\nexport snapshot\n")
        .module("client", client_source)
        .entry("client")
        .analyze();
    let diagnostics = fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>();
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let client_calls = fixture
        .analysis
        .snapshot
        .callable_analyses
        .iter()
        .filter(|(callable, _)| callable.owner.module() == fixture.module("client"));
    let mut checked = 0;
    for (_, analysis) in client_calls {
        for expression in analysis.expressions.values() {
            let text = &client_source[expression.range.start..expression.range.end];
            if ["local(input)", "helpers.snapshot(input)", "snapshot(input)"].contains(&text) {
                let specialization = expression
                    .call_specialization
                    .as_ref()
                    .unwrap_or_else(|| panic!("missing specialization on {text}"));
                assert_eq!(specialization.bindings.len(), 1);
                assert!(matches!(
                    specialization.conformances[0].source,
                    phalcom_semantic::checker::analysis::GenericCallConformanceSource::Forwarded { .. }
                ));
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 3);
}

#[test]
fn exported_generic_identity_family_keeps_its_binder() {
    let fixture = WorkspaceFixture::new()
        .module(
            "helpers",
            r#"
class Helpers { @class identity<T>(_ value: T) -> T { value } }
let identity = &Helpers.identity(_)
export identity
"#,
        )
        .module("facade", "from .helpers import identity\nexport identity\n")
        .module(
            "client",
            r#"
from .facade import identity
import .helpers as helpers
class Client {
  @class run<U>(_ input: U) -> U {
    let local = identity
    let a = local(input)
    let b = helpers.identity(input)
    a
  }
}
let concrete = identity(1)
"#,
        )
        .entry("client")
        .analyze();
    let diagnostics = fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>();
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn module_value_exports_preserve_family_type_for_selective_and_qualified_calls() {
    let fixture = WorkspaceFixture::new()
        .module(
            "math",
            r#"
class Kernel {
  @class sqrt(_ value: Int) -> Int { value }
}
let sqrt = &Kernel.sqrt(_)
export sqrt
"#,
        )
        .module(
            "facade",
            r#"
from .math import sqrt as square_root
export square_root
"#,
        )
        .module(
            "client",
            r#"
from .facade import square_root as root
import .facade as math
let selective_ok = root(1)
let qualified_ok = math.square_root(1)
let selective_wrong = root("wrong")
let qualified_wrong = math.square_root("wrong")

class Nested {
  @class selective_wrong() { root("wrong") }
  @class qualified_wrong() { math.square_root("wrong") }
}
"#,
        )
        .entry("client")
        .analyze();

    let math_symbol = phalcom_modules::linker::SymbolId {
        module: fixture.module("math").clone(),
        name: "sqrt".into(),
    };
    let math_fact = fixture
        .analysis
        .snapshot
        .module_static_binding(&math_symbol)
        .expect("canonical exported math binding");
    let math_type = math_fact.state.current.ty().expect("exported Family type");
    assert!(matches!(fixture.analysis.snapshot.store.get(math_type), TypeData::Family(_)));

    let client_main = CallableId::new(
        DeclarationId::new(fixture.module("client").clone(), "<main>".into()),
        phalcom_common::selector::Selector::getter("<main>").expect("main selector"),
        DispatchSide::Instance,
    );
    let client = fixture.analysis.snapshot.callable_analyses.get(&client_main).expect("client analysis");
    let imported = client
        .bindings
        .values()
        .find(|binding| binding.name == "root")
        .expect("selective import binding");
    let imported_type = imported.current.ty().expect("imported Family type");
    assert!(matches!(fixture.analysis.snapshot.store.get(imported_type), TypeData::Family(_)));

    let argument_mismatches = fixture
        .analysis
        .snapshot
        .all_diagnostics()
        .filter(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::ArgumentMismatch)
        .count();
    assert_eq!(
        argument_mismatches, 4,
        "selective and qualified calls in top-level and member bodies must use the Family callable contract"
    );
}

#[test]
fn imported_family_member_body_revalidates_when_exported_contract_changes() {
    let math_source_v1 = r#"
class Kernel {
  @class sqrt(_ value: Int) -> Int { value }
}
let sqrt = &Kernel.sqrt(_)
export sqrt
"#;
    let math_source_v2 = r#"
class Kernel {
  @class sqrt(_ value: String) -> String { value }
}
let sqrt = &Kernel.sqrt(_)
export sqrt
"#;
    let client_source = r#"
from .math import sqrt as root
class Client {
  @class run() { root("wrong") }
}
"#;
    let math = incremental_module_id("math");
    let client = incremental_module_id("client");
    let mut session = SemanticWorkspaceSession::new();

    let first = session.update(module_value_input(&math, math_source_v1, &client, client_source, 1));
    let body = CallableId::new(
        DeclarationId::new(client.clone(), "Client".into()),
        phalcom_common::selector::Selector::method("run", []).expect("run selector"),
        DispatchSide::Class,
    );
    let body_key = QueryKey::CallableBody(body);
    assert_eq!(
        first
            .snapshot
            .all_diagnostics()
            .filter(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::ArgumentMismatch)
            .count(),
        1,
        "Int Family contract rejects the string argument in the member body"
    );
    assert!(
        session
            .db()
            .index()
            .dependencies_of(&body_key)
            .is_some_and(|dependencies| { dependencies.iter().any(|edge| edge.dependency == QueryKey::ModuleStaticBindings(math.clone())) })
    );
    let first_revision = first.snapshot.id.revision();

    let second = session.update(module_value_input(&math, math_source_v2, &client, client_source, 2));
    let second_revision = second.snapshot.id.revision();
    assert_ne!(first_revision, second_revision);
    assert_eq!(
        second
            .snapshot
            .all_diagnostics()
            .filter(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::ArgumentMismatch)
            .count(),
        0,
        "the changed String Family contract accepts the same member-body call"
    );
    assert_eq!(session.db().query_state(&body_key).and_then(|state| state.revision()), Some(second_revision));

    let mut cold = SemanticWorkspaceSession::new();
    let cold_result = cold.update(module_value_input(&math, math_source_v2, &client, client_source, 2));
    assert_eq!(
        second.snapshot.diagnostics, cold_result.snapshot.diagnostics,
        "incremental and cold diagnostics agree"
    );
}

fn incremental_module_id(name: &str) -> ModuleId {
    let project = ResolvedProjectId::from_raw(93);
    let path = ModulePath::from_components(vec![ModuleComponent::from_identifier(name).expect("module name")]);
    ModuleId::resolved(project, path)
}

fn module_value_input(math: &ModuleId, math_source: &str, client: &ModuleId, client_source: &str, generation: u64) -> SemanticWorkspaceInput {
    let sources = [(math.clone(), math_source), (client.clone(), client_source)]
        .into_iter()
        .map(|(module, source)| {
            let parsed = phalcom_ast::parse(source, 0);
            assert!(parsed.errors.is_empty(), "fixture parse errors: {:#?}", parsed.errors);
            let program = Arc::new(parsed.program);
            let interface = InterfaceBuilder::build(module.clone(), ModuleKind::Module, &program).expect("module interface");
            (
                module.clone(),
                Arc::new(ParsedModuleUnit::new(module, ModuleKind::Module, None, Arc::from(source), program)),
                interface,
            )
        })
        .collect::<Vec<_>>();
    let interfaces = sources.iter().map(|(module, _, interface)| (module.clone(), interface.clone())).collect();
    let source_units = sources.into_iter().map(|(module, unit, _)| (module, unit)).collect();
    let resolved = BTreeMap::from([
        ((client.clone(), ".math".to_owned()), math.clone()),
        ((client.clone(), "math".to_owned()), math.clone()),
    ]);
    let linked = Arc::new(
        ModuleLinker::new(Arc::new(phalcom_modules::project::ProjectUniverse::new()), interfaces)
            .link_component_interfaces_tolerant(client.clone(), &resolved)
            .program,
    );
    SemanticWorkspaceInput::new(linked, source_units, generation)
}
#[test]
fn immediate_family_call_publishes_static_application_resolution() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
data Weird {
  @variant Marker
  @variant Marker()
  @variant Marker(_ value: Int)
}

class Probe {
  @class run() { (&Weird::Marker...)(1) }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    let probe = DeclarationId::new(module, "Probe".into());
    let resolution = find_application(&analysis, &source, &probe, "run", "(&Weird::Marker...)(1)");

    let FamilyApplicationSelection::Static { operation, target, .. } = &resolution.selection else {
        panic!("expected static family application, got {:?}", resolution.selection);
    };
    assert_eq!(operation.kind, SelectorKind::Method);
    assert_eq!(operation.slots.as_ref(), [SelectorSlot::Positional]);
    assert!(matches!(target, Some(InvocationTargetId::VariantConstructor(_))));
}

#[test]
fn stored_family_call_publishes_static_application_resolution() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
data Weird {
  @variant Marker
  @variant Marker()
  @variant Marker(_ value: Int)
}

class Probe {
  @class run() {
    let make = &Weird::Marker...;
    make(1)
  }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    let probe = DeclarationId::new(module, "Probe".into());
    let resolution = find_application(&analysis, &source, &probe, "run", "make(1)");

    let FamilyApplicationSelection::Static { operation, target, .. } = &resolution.selection else {
        panic!("expected static family application, got {:?}", resolution.selection);
    };
    assert_eq!(operation.kind, SelectorKind::Method);
    assert_eq!(operation.slots.as_ref(), [SelectorSlot::Positional]);
    assert!(matches!(target, Some(InvocationTargetId::VariantConstructor(_))));
}

#[test]
fn stored_family_dynamic_pack_publishes_frozen_candidates() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
data Weird {
  @variant Marker
  @variant Marker()
  @variant Marker(_ value: Int)
}

class Probe {
  @class run() {
    let make = &Weird::Marker...;
    let args = [1];
    make(*args)
  }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    let probe = DeclarationId::new(module, "Probe".into());
    let resolution = find_application(&analysis, &source, &probe, "run", "make(*args)");

    let FamilyApplicationSelection::Dynamic { candidates, .. } = &resolution.selection else {
        panic!("expected dynamic family application, got {:?}", resolution.selection);
    };
    assert_eq!(candidates.len(), 2);
    assert!(candidates.iter().all(|candidate| candidate.operation.kind == SelectorKind::Method));
    assert!(candidates.iter().all(|candidate| candidate.target.is_some()));
    assert!(candidates[0].operation.slots.is_empty());
    assert_eq!(candidates[1].operation.slots.as_ref(), [SelectorSlot::Positional]);
    assert!(matches!(
        &candidates[0].target,
        Some(InvocationTargetId::VariantConstructor(constructor)) if constructor.variant.selector.slots.is_empty()
    ));
    assert!(matches!(
        &candidates[1].target,
        Some(InvocationTargetId::VariantConstructor(constructor))
            if constructor.variant.selector.slots.as_ref() == [SelectorSlot::Positional]
    ));
}

#[test]
fn generic_family_failure_does_not_publish_fallback_result() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Box<T> {
  convert<U>() -> U { 42 }
}

class Probe {
  @class
  run(_ box: Box<Int>) {
    let family = &box.convert...;
    let value = family()
  }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    let probe = DeclarationId::new(module, "Probe".into());
    let callable_id = CallableId::new(
        probe,
        phalcom_common::selector::Selector::method("run", [SelectorSlot::Positional]).expect("callable selector"),
        DispatchSide::Class,
    );
    let callable = analysis.snapshot.callable_analyses.get(&callable_id).expect("callable analysis");
    let family_capture = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some("&box.convert..."))
        .expect("generic family capture expression");
    assert!(
        family_capture.knowledge.is_known(),
        "generic family capture must retain a callable family, got {:?}",
        family_capture.knowledge
    );

    let expression = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some("family()"))
        .expect("family application expression");

    assert!(
        expression.knowledge.ty().is_none(),
        "expected failed generic application, got {:?}",
        expression.knowledge
    );
    assert!(
        !callable.family_applications.contains_key(&expression.id),
        "failed generic application must not publish fallback family result: {:?}",
        callable.family_applications.get(&expression.id)
    );
}

#[test]
fn generic_behavioral_family_recovers_canonical_signature_for_expected_result() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Box<T> {
  convert<U>() -> U { 42 }
}

class Probe {
  @class
  run(_ box: Box<Int>) {
    let family = &box.convert...;
    let value: Int = family();
  }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    let probe = DeclarationId::new(module, "Probe".into());
    let callable_id = CallableId::new(
        probe,
        phalcom_common::selector::Selector::method("run", [SelectorSlot::Positional]).expect("callable selector"),
        DispatchSide::Class,
    );
    let callable = analysis.snapshot.callable_analyses.get(&callable_id).expect("callable analysis");
    let expression = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some("family()"))
        .expect("family application expression");
    let result_type = expression.knowledge.ty().expect("expected generic family result");
    assert_eq!(analysis.snapshot.store.format_type(result_type), "Int");
}

#[test]
fn generic_associated_variant_family_recovers_constructor_signature() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
data Option<T> {
  @variant Some(_ value: T) -> Option<T>
}

class Probe {
  @class
  run() {
    let family = &Option<Int>::Some...;
    let value = family(1);
  }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    let probe = DeclarationId::new(module, "Probe".into());
    let callable_id = CallableId::new(
        probe,
        phalcom_common::selector::Selector::method("run", []).expect("callable selector"),
        DispatchSide::Class,
    );
    let callable = analysis.snapshot.callable_analyses.get(&callable_id).expect("callable analysis");
    let expression = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some("family(1)"))
        .expect("family application expression");
    let result_type = expression.knowledge.ty().expect("associated family result");
    assert!(matches!(
        analysis.snapshot.store.get(result_type),
        phalcom_semantic::types::store::TypeData::ExactCase { .. }
    ));

    let resolution = callable.family_applications.get(&expression.id).expect("family application resolution");
    let FamilyApplicationSelection::Static { target, .. } = &resolution.selection else {
        panic!("expected static family application, got {:?}", resolution.selection);
    };
    assert!(matches!(target, Some(InvocationTargetId::VariantConstructor(_))));
}

#[test]
fn family_wrong_shape_reports_associated_call_shape() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
data Weird {
  @variant Marker(_ value: Int)
}

class Probe {
  @class
  run() { Weird::Marker(1, 2) }
}
"#,
    );
    let analysis = analyze_source(module, source);
    assert!(
        analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::AssociatedCallShapeMissing)
    );
}

#[test]
fn behavioral_family_source_preserves_pattern_storage_and_dispatch_side() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Service {
  @class
  make(_ value: Int) -> Int { value }

  take(_ value: Int) -> Int { value }
}

class Probe {
  @class
  run(_ service: Service) {
    let instance = &service.take...;
    let class_side = &Service.make...;
    let instance_result = instance(1)
    let class_result = class_side(1)
  }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    assert!(!analysis.snapshot.has_errors(), "diagnostics: {:?}", analysis.snapshot.diagnostics);
    let probe = DeclarationId::new(module, "Probe".into());
    let run_id = CallableId::new(
        probe,
        phalcom_common::selector::Selector::method("run", [SelectorSlot::Positional]).expect("run selector"),
        DispatchSide::Class,
    );
    let callable = analysis.snapshot.callable_analyses.get(&run_id).expect("run analysis");

    let capture_for = |text: &str| {
        callable
            .expressions
            .values()
            .find(|expression| source.get(expression.range.start..expression.range.end) == Some(text))
            .unwrap_or_else(|| panic!("missing family capture {text}"))
    };
    for (text, expected_side) in [("&service.take...", DispatchSide::Instance), ("&Service.make...", DispatchSide::Class)] {
        let capture = capture_for(text);
        let resolution = callable.callable_reference_resolutions.get(&capture.id).expect("family resolution");
        let CallableReferenceResolutionKind::BoundFamily { members, .. } = &resolution.kind else {
            panic!("expected behavioral family resolution for {text}, got {:?}", resolution.kind);
        };
        assert_eq!(members.len(), 1, "one exact source method should populate one family member");
        assert!(matches!(&members[0].target, InvocationTargetId::Behavioral(callable) if callable.side == expected_side));
    }

    for (text, expected_side) in [("instance(1)", DispatchSide::Instance), ("class_side(1)", DispatchSide::Class)] {
        let application = callable
            .expressions
            .values()
            .find(|expression| source.get(expression.range.start..expression.range.end) == Some(text))
            .expect("family application expression");
        let resolution = callable.family_applications.get(&application.id).expect("family application resolution");
        let FamilyApplicationSelection::Static { target, .. } = &resolution.selection else {
            panic!("expected static family invocation for {text}, got {:?}", resolution.selection);
        };
        assert!(matches!(target, Some(InvocationTargetId::Behavioral(callable)) if callable.side == expected_side));
    }
}

#[test]
fn structurally_equal_behavioral_families_retain_distinct_denotations_and_targets() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Left {
  @class
  make(_ value: Int) -> Int { value }
}
class Right {
  @class
  make(_ value: Int) -> Int { value }
}
class Probe {
  @class
  run() {
    let left = &Left.make...;
    let right = &Right.make...;
  }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    assert!(!analysis.snapshot.has_errors(), "diagnostics: {:?}", analysis.snapshot.diagnostics);
    let probe = DeclarationId::new(module, "Probe".into());
    let run_id = CallableId::new(
        probe,
        phalcom_common::selector::Selector::method("run", []).expect("run selector"),
        DispatchSide::Class,
    );
    let callable = analysis.snapshot.callable_analyses.get(&run_id).expect("run analysis");
    let capture = |text: &str| {
        callable
            .expressions
            .values()
            .find(|expression| source.get(expression.range.start..expression.range.end) == Some(text))
            .unwrap_or_else(|| panic!("missing family capture {text}"))
    };
    let left = capture("&Left.make...");
    let right = capture("&Right.make...");
    assert_eq!(
        left.knowledge.ty(),
        right.knowledge.ty(),
        "same callable shape should intern one structural family type"
    );
    assert_ne!(left.denotation, right.denotation, "receiver declaration remains family provenance");

    let target = |expression: &phalcom_semantic::checker::analysis::ExpressionAnalysis| {
        let SemanticDenotation::BehavioralFamily(denotation) = expression.denotation.as_ref().expect("family denotation") else {
            panic!("expected associated family denotation");
        };
        denotation.members[0].target.clone()
    };
    let left_target = target(left);
    let right_target = target(right);
    assert_ne!(left_target, right_target, "same family shape must retain distinct callable targets");
    assert!(matches!(left_target, InvocationTargetId::Behavioral(ref callable) if callable.owner.declaration().name.as_ref() == "Left"));
    assert!(matches!(right_target, InvocationTargetId::Behavioral(ref callable) if callable.owner.declaration().name.as_ref() == "Right"));
}

fn analyze_source(module: ModuleId, source: Arc<str>) -> phalcom_semantic::workspace::SemanticAnalysis {
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    analyze_single_module(module, source, Arc::new(parsed.program))
}

fn find_application<'a>(
    analysis: &'a phalcom_semantic::workspace::SemanticAnalysis,
    source: &str,
    owner: &DeclarationId,
    callable: &str,
    expression: &str,
) -> &'a FamilyApplicationResolution {
    let callable_id = CallableId::new(
        owner.clone(),
        phalcom_common::selector::Selector::method(callable, []).expect("callable selector"),
        DispatchSide::Class,
    );
    let callable = analysis.snapshot.callable_analyses.get(&callable_id).expect("callable analysis");
    let expression = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some(expression))
        .expect("family application expression");
    callable.family_applications.get(&expression.id).expect("family application resolution")
}
