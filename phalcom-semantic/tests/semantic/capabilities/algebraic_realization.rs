use crate::semantic::support::Fixture;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::checker::{ContextualRealization, TypedExpression};
use phalcom_semantic::trait_dispatch::TraitDispatchSite;
use phalcom_semantic::types::denotation::SemanticDenotation;
use phalcom_semantic::types::parameter::TypeTerm;
use std::sync::Arc;

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

#[test]
fn typed_expression_remains_a_lightweight_recursive_carrier() {
    // Measured at origin/main c2dd635e. Contextual semantic products must
    // remain indirect so extending their evidence cannot enlarge recursive
    // expression-checking frames.
    const MAIN_C2DD635E_SIZE: usize = 744;

    let current = std::mem::size_of::<TypedExpression>();
    let allowance = 2 * std::mem::size_of::<usize>();
    assert!(
        current <= MAIN_C2DD635E_SIZE + allowance,
        "TypedExpression grew beyond its recursive-carrier budget: main={MAIN_C2DD635E_SIZE}, current={current}"
    );
    assert_eq!(
        std::mem::size_of::<Option<Arc<ContextualRealization>>>(),
        std::mem::size_of::<Arc<ContextualRealization>>(),
        "the optional realization product must stay pointer-sized"
    );
}

#[test]
fn ordinary_float_horner_expression_is_stack_safe_without_realization() {
    let _ = phalcom_semantic::CanonicalUniverseSemanticBaseline::build();
    let type_fixture = Fixture::new("class Probe { @class calculate(_ x: Float) -> Float { return x } }\n");
    let float_type = type_fixture.ty("Float");
    let mut expression = "64.0".to_owned();
    for coefficient in (1..64).rev() {
        expression = format!("{coefficient}.0 + x * ({expression})");
    }
    let source = format!("class Probe {{ @class calculate(_ x: Float) -> Float {{ return ({expression}) }} }}\n");
    let source_for_parse = source.clone();
    let parsed = std::thread::Builder::new()
        .name("ordinary-expression-parse-regression".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || phalcom_ast::parse(&source_for_parse, 0))
        .expect("spawn parser setup thread")
        .join()
        .expect("bounded-depth source must parse on the setup stack");
    assert!(parsed.errors.is_empty(), "Horner regression must parse: {:#?}", parsed.errors);
    let module = phalcom_semantic::ModuleId::universe_root();
    let source: Arc<str> = Arc::from(source);
    let program = Arc::new(parsed.program);
    std::thread::Builder::new()
        .name("ordinary-expression-stack-regression".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let analysis = phalcom_semantic::analyze_single_module(module, source.clone(), program);
            assert!(!analysis.snapshot.has_errors(), "unexpected diagnostics: {:?}", analysis.snapshot.all_diagnostics().collect::<Vec<_>>());
            let analyzed = analysis
                .snapshot
                .callable_analyses
                .values()
                .flat_map(|callable| callable.expressions.values())
                .find(|analysis| source.get(analysis.range.start..analysis.range.end) == Some(expression.as_str()))
                .expect("the Horner expression has a published semantic product");
            assert_eq!(analyzed.knowledge.ty(), Some(float_type));
            assert!(
                analysis
                    .snapshot
                    .callable_analyses
                    .values()
                    .flat_map(|callable| callable.expressions.values())
                    .all(|expression| expression.contextual_realization.is_none()),
                "ordinary Float arithmetic must not publish algebraic realization products"
            );
        })
        .expect("spawn bounded-stack semantic regression")
        .join()
        .expect("ordinary nested Float arithmetic must not overflow the checker stack");
}
