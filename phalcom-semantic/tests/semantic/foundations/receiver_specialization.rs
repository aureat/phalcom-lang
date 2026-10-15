//! SC-2 owner-relative receiver specialization laws.

use crate::semantic::support::{Fixture, applied};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::explain::ExplanationStep;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::parameter::TypeParameterOwner;
use phalcom_semantic::types::specialization::specialize_receiver_to_owner;
use phalcom_semantic::types::store::StaticArgument;
use phalcom_semantic::checker::context::CheckerControl;

#[test]
fn direct_generic_receiver_specializes_member_contract() {
    let fixture = Fixture::new(
        r#"
class Box<T> {
  value(_ value: T) -> T { value }
}

class Probe {
  @class
  run(_ box: Box<Int>) {
    let result = box.value(1)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(fixture.binding(run, "result").current.ty(), Some(fixture.ty("Int")));
    assert_eq!(fixture.binding(run, "result").current.status(), Some(phalcom_semantic::EvidenceStatus::Assumed));
}

#[test]
fn transformed_single_hop_receiver_projects_owner_argument() {
    let fixture = Fixture::new(
        r#"
class Parent<T> {
  value() -> T { 1 }
}

class Child<T> is Parent<List<T>> {}

class Probe {
  @class
  run(_ child: Child<Int>) {
    let result = child.value()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let result = fixture.binding(run, "result");
    fixture.assert_type(result.current.ty().expect("projected result"), applied("List", [fixture.ty("Int").into()]));
    fixture.assert_trace_has(run, fixture.expression(run, "child.value()"), |step| {
        matches!(
            step,
            ExplanationStep::CallableSelection {
                declaring_owner,
                specialization_path,
                ..
            } if declaring_owner.name.as_ref() == "Parent"
                && specialization_path.len() == 2
                && specialization_path[0].name.as_ref() == "Child"
                && specialization_path[1].name.as_ref() == "Parent"
        )
    });
}

#[test]
fn class_and_method_generics_specialize_together() {
    let fixture = Fixture::new(
        r#"
class Pairer<T> {
  pair<U>(_ value: U) -> (T, U) { (1, value) }
}

class Probe {
  @class
  run(_ pairer: Pairer<Int>) {
    let result = pairer.pair("text")
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    fixture.assert_tuple_types(
        fixture.binding(run, "result").current.ty().expect("generic pair result"),
        &[fixture.ty("Int"), fixture.ty("String")],
    );
}

#[test]
fn multi_hop_receiver_projects_each_generic_supertype_template() {
    let fixture = Fixture::new(
        r#"
class Base<T> {
  value() -> T { 1 }
}

class Middle<T> is Base<Option<T>> {}
class Leaf<T> is Middle<T> {}

class Probe {
  @class
  run(_ leaf: Leaf<Int>) {
    let result = leaf.value()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let result = fixture.binding(run, "result");
    fixture.assert_type(result.current.ty().expect("projected result"), applied("Option", [fixture.ty("Int").into()]));
}

#[test]
fn receiver_specialization_carries_method_generic_constraints() {
    let fixture = Fixture::new(
        r#"
class Animal {}
class Cat is Animal { @constructor new() {} }

class Holder<T> {
  convert<U>(_ value: U) -> U where U <: T { value }
}

class Probe {
  @class
  run(_ holder: Holder<Animal>, _ cat: Cat) {
    let result = holder.convert(cat)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let result = fixture.binding(run, "result");
    assert_eq!(result.current.ty(), Some(fixture.ty("Cat")), "{result:#?}");
    assert!(matches!(fixture.expression(run, "holder.convert(cat)").status, AnalysisStatus::Ready));
}

#[test]
fn self_inherited_member_uses_actual_receiver_type() {
    let fixture = Fixture::new(
        r#"
class Box<T> {}
class Parent {
  wrap() -> Box<Self> { 1 }
}
class Child is Parent {}

class Probe {
  @class
  run(_ child: Child) {
    let result = child.wrap()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let result = fixture.binding(run, "result");
    fixture.assert_type(result.current.ty().expect("projected result"), applied("Box", [fixture.ty("Child").into()]));
}

#[test]
fn explicit_dynamic_owner_argument_is_preserved_by_receiver_specialization() {
    fn owner_argument(source_type: &str, dynamic: bool) -> (Fixture, phalcom_semantic::types::environment::TypeEnvironment) {
        let fixture = Fixture::new_allowing_internal_incidents(&format!(
            "class Box<T> {{ ping() -> Box<T> {{ self }} }}\n\
             class Probe {{ @class run(_ box: Box<{source_type}>) {{ box.ping() }} }}\n"
        ));
        let run = fixture.callable("Probe", "run", DispatchSide::Class);
        let call = fixture.expression(run, "box.ping()");
        let selected = fixture.callable_id("Box", "ping", DispatchSide::Instance);
        assert_eq!(call.callable.as_ref(), Some(&selected), "receiver must select Box.ping");

        let receiver = fixture
            .binding(run, "box")
            .current
            .ty()
            .expect("annotated receiver type");
        let owner = fixture.decl("Box");
        let (_, arguments) = fixture.analysis.snapshot.store.applied_nominal_static_parts(receiver).expect("applied Box receiver");
        assert_eq!(arguments.as_slice(), &[if dynamic {
            StaticArgument::DynamicType
        } else {
            StaticArgument::Term(fixture.ty("Int"))
        }]);

        let parameter = fixture
            .analysis
            .snapshot
            .store
            .find_type_parameter_id(&TypeParameterOwner::Declaration(owner.clone()), 0)
            .expect("Box owner parameter");
        let parameter_data = fixture.analysis.snapshot.store.type_parameter(parameter);
        assert_eq!(parameter_data.owner, TypeParameterOwner::Declaration(owner.clone()));
        assert_eq!(parameter_data.index, 0);
        assert_eq!(parameter_data.kind, phalcom_semantic::types::id::KindId::TYPE);

        let mut store = (*fixture.analysis.snapshot.store).clone();
        let specialization = specialize_receiver_to_owner(
            &mut store,
            &*fixture.analysis.snapshot.hierarchy,
            receiver,
            &owner,
            &CheckerControl::default(),
        )
        .expect("exact Box receiver specialization");
        let environment = specialization.environment;
        assert_eq!(environment.get_param(parameter), (!dynamic).then_some(fixture.ty("Int")));
        assert_eq!(environment.is_dynamic(parameter), dynamic);
        assert_eq!(environment.get_row(parameter), None);
        (fixture, environment)
    }

    let (dynamic_fixture, _) = owner_argument("Dynamic", true);
    dynamic_fixture.assert_no_internal_incidents();
    let run = dynamic_fixture.callable("Probe", "run", DispatchSide::Class);
    let call = dynamic_fixture.expression(run, "box.ping()");
    let parameter = dynamic_fixture
        .analysis
        .snapshot
        .store
        .find_type_parameter_id(&TypeParameterOwner::Declaration(dynamic_fixture.decl("Box")), 0)
        .expect("Box owner parameter");
    assert_eq!(
        call.call_specialization
            .as_ref()
            .and_then(|value| value.bindings.iter().find(|(id, _)| *id == parameter).map(|(_, arg)| *arg)),
        Some(StaticArgument::DynamicType)
    );
    let mut store = (*dynamic_fixture.analysis.snapshot.store).clone();
    let box_decl = dynamic_fixture.decl("Box");
    let box_parameter = store
        .find_type_parameter_id(&TypeParameterOwner::Declaration(box_decl.clone()), 0)
        .expect("Box<T> owner parameter");
    let parameter_form = store.parameter_form(box_parameter);
    let ping = dynamic_fixture.callable_id("Box", "ping", DispatchSide::Instance);
    let nested_parameter = dynamic_fixture
        .analysis
        .snapshot
        .callable_signatures
        .get(&ping)
        .expect("Box.ping signature")
        .declared_return
        .canonical_type()
        .expect("proper nested Box<T> return type");
    let mut instantiation = phalcom_semantic::types::instantiation::GenericInstantiation::default();
    instantiation.bind_dynamic(box_parameter);
    assert_eq!(
        phalcom_semantic::types::instantiation::materialize_type(
            &mut store,
            parameter_form,
            &instantiation,
            phalcom_semantic::types::instantiation::RowMaterializationMode::PreserveUnboundStableTail,
        ),
        Err(phalcom_semantic::types::instantiation::TypeMaterializationError::DynamicBoundary(box_parameter))
    );
    let nested = phalcom_semantic::types::instantiation::materialize_type(
        &mut store,
        nested_parameter,
        &instantiation,
        phalcom_semantic::types::instantiation::RowMaterializationMode::PreserveUnboundStableTail,
    )
    .expect("nested type application can carry Dynamic");
    assert_eq!(
        store.applied_nominal_static_parts(nested).expect("materialized Box<Dynamic>").1,
        vec![StaticArgument::DynamicType]
    );

    let (int_fixture, _) = owner_argument("Int", false);
    int_fixture.assert_no_internal_incidents();
    let run = int_fixture.callable("Probe", "run", DispatchSide::Class);
    let call = int_fixture.expression(run, "box.ping()");
    let parameter = int_fixture
        .analysis
        .snapshot
        .store
        .find_type_parameter_id(&TypeParameterOwner::Declaration(int_fixture.decl("Box")), 0)
        .expect("Box owner parameter");
    assert_eq!(
        call.call_specialization
            .as_ref()
            .and_then(|value| value.bindings.iter().find(|(id, _)| *id == parameter).map(|(_, arg)| *arg)),
        Some(StaticArgument::Term(int_fixture.ty("Int")))
    );
}
