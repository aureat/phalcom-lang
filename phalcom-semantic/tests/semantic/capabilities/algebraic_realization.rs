use crate::semantic::support::Fixture;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::trait_dispatch::TraitDispatchSite;
use phalcom_semantic::types::denotation::SemanticDenotation;
use phalcom_semantic::types::parameter::TypeTerm;

#[test]
fn assumed_generic_class_side_zero_and_one_preserve_type_form_and_self() {
    let fixture = Fixture::new(
        r#"
trait Zero {
  @class zero -> Self
}

trait One {
  @class one -> Self
}

class Probe {
  @class makeZero<T>() -> T where T |= Zero { T.zero }
  @class makeOne<T>() -> T where T |= One { T.one }
}
"#,
    );
    assert!(
        !fixture.analysis.snapshot.has_errors(),
        "unexpected diagnostics: {:?}",
        fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>()
    );

    for (method, expression_text, trait_name, selector) in [("makeZero", "T.zero", "Zero", "zero"), ("makeOne", "T.one", "One", "one")] {
        let callable = fixture.callable("Probe", method, DispatchSide::Class);
        let receiver = fixture.expression(callable, "T");
        let Some(SemanticDenotation::TypeForm(receiver_type)) = receiver.denotation.as_ref() else {
            panic!("{method} receiver must retain a type-form denotation: {receiver:#?}");
        };
        let result = fixture.expression(callable, expression_text);
        assert_eq!(result.knowledge.ty(), Some(*receiver_type), "class-side Self must be the conforming subject");
        let Some(TraitDispatchSite::Assumed(selection)) = result.trait_dispatch.as_ref() else {
            panic!("{method} must select assumed generic trait evidence: {:#?}", result.trait_dispatch);
        };
        assert_eq!(selection.requirement.owner, fixture.decl(trait_name));
        assert_eq!(
            selection.requirement.selector.base,
            phalcom_common::selector::SelectorBase::Named(selector.into())
        );
        assert_eq!(selection.requirement.side, DispatchSide::Class);
        assert_eq!(selection.conformance.subject, TypeTerm::Canonical(*receiver_type));
        assert_eq!(selection.conformance.trait_ref.declaration, fixture.decl(trait_name));
    }
}

#[test]
fn instance_dispatch_cannot_consume_class_side_zero() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Zero {
  @class zero -> Self
}

class Probe {
  @class invalid<T>(_ value: T) -> T where T |= Zero { value.zero }
}
"#,
    );
    let callable = fixture.callable("Probe", "invalid", DispatchSide::Class);
    let expression = fixture.expression(callable, "value.zero");
    assert!(
        !matches!(expression.trait_dispatch, Some(TraitDispatchSite::Assumed(_))),
        "wrong-side dispatch must fail closed: {expression:#?}"
    );
}

#[test]
fn instance_side_zero_cannot_satisfy_generic_type_form_dispatch() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Zero {
  zero -> Self
}

class Probe {
  @class invalid<T>() -> T where T |= Zero { T.zero }
}
"#,
    );
    let callable = fixture.callable("Probe", "invalid", DispatchSide::Class);
    let expression = fixture.expression(callable, "T.zero");
    assert!(
        !matches!(expression.trait_dispatch, Some(TraitDispatchSite::Assumed(_))),
        "wrong-side dispatch must fail closed: {expression:#?}"
    );
}
