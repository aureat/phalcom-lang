use crate::semantic::support::Fixture;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::checker::{ContextualRealization, TypedExpression};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::trait_dispatch::TraitDispatchSite;
use phalcom_semantic::types::denotation::SemanticDenotation;
use phalcom_semantic::types::parameter::TypeTerm;
use phalcom_semantic::{SemanticAnalysis, session::SemanticWorkspaceSession};
use std::sync::Arc;

fn analyze_algebra_source(source: &str) -> (SemanticAnalysis, ModuleId, Arc<str>) {
    let root = tempfile::tempdir().expect("workspace tempdir");
    std::fs::write(root.path().join("package.ph"), "").expect("workspace package marker");
    let path = root.path().join("main.ph");
    let location = SourceLocation {
        source_id: SourceId(path.to_string_lossy().into_owned().into()),
        display_path: path,
    };
    let source: Arc<str> = Arc::from(source);
    let publication = SemanticWorkspaceSession::new()
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: location.clone(),
            text: Arc::clone(&source),
            revision: SourceRevision(1),
            recovered_program: None,
        }])
        .expect("canonical Universe workspace analysis");
    let module = publication
        .snapshot
        .module_for_source(&location.source_id)
        .expect("user module identity")
        .clone();
    (
        SemanticAnalysis {
            snapshot: publication.snapshot,
        },
        module,
        source,
    )
}

fn expression_for<'a>(
    analysis: &'a SemanticAnalysis,
    module: &ModuleId,
    source: &str,
    method: &str,
    expression: &str,
) -> &'a phalcom_semantic::checker::ExpressionAnalysis {
    let callable = CallableId::new(
        DeclarationId::new(module.clone(), "Probe".into()),
        Selector::method(method, []).expect("method selector"),
        DispatchSide::Class,
    );
    analysis
        .snapshot
        .callable_analyses
        .get(&callable)
        .unwrap_or_else(|| panic!("missing callable analysis for {callable:?}"))
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some(expression))
        .unwrap_or_else(|| panic!("missing expression `{expression}` in {callable:?}"))
}

fn assert_generic_target(analysis: &SemanticAnalysis, module: &ModuleId, method: &str, source: &str, literal: &str, scalar: &str) {
    let expression = expression_for(analysis, module, source, method, literal);
    let realization = expression
        .contextual_realization
        .as_deref()
        .unwrap_or_else(|| panic!("contextual realization product missing: {expression:#?}"));
    let (declaration, arguments) = analysis
        .snapshot
        .store
        .applied_nominal_parts(realization.target_type)
        .expect("realization target must be an applied nominal type");
    assert_eq!(declaration, DeclarationId::new(module.clone(), "Strange".into()));
    assert_eq!(arguments.len(), 2);
    assert_eq!(
        analysis.snapshot.store.nominal_origin_declaration(arguments[0]),
        Some(&DeclarationId::new(module.clone(), "Tag".into()))
    );
    assert_eq!(
        analysis
            .snapshot
            .store
            .nominal_origin_declaration(arguments[1])
            .map(|declaration| declaration.name.as_ref()),
        Some(scalar)
    );
}

fn strange_realization_source(value_body: &str, include_conformance: bool) -> String {
    let mut source = String::from(
        r#"
from universe.numbers import BasisRealizable, Basis, I

data Strange<Marker, S>(_ real: S, _ imag: S)
data Tag()

"#,
    );
    if include_conformance {
        source.push_str(
            r#"
impl<Marker, S> BasisRealizable<Basis<I>> for Strange<Marker, S> {
  type Scalar = S
  @class along(_ coefficient: S, _ basis: Basis<I>) -> Strange<Marker, S> { Strange(coefficient, coefficient) }
}

"#,
        );
    }
    source.push_str(&format!("class Probe {{ @class value() {{ {value_body} }} }}\n"));
    source
}

#[test]
fn explicit_user_algebra_resolves_from_associated_scalar_type() {
    let source = r#"
from universe.numbers import BasisRealizable, Basis, I, i

data Strange<Marker, S>(_ real: S, _ imag: S)
data Tag()
impl<Marker, S> BasisRealizable<Basis<I>> for Strange<Marker, S> {
  type Scalar = S
  @class along(_ coefficient: S, _ basis: Basis<I>) -> Strange<Marker, S> { Strange(coefficient, coefficient) }
}
class Probe {
  @class intValue() -> Strange<Tag, Int> { i }
  @class floatValue() -> Strange<Tag, Float> { 2.0i }
}
"#;
    let (analysis, module, source) = analyze_algebra_source(source);
    assert!(
        !analysis.snapshot.has_errors(),
        "unexpected diagnostics: {:?}",
        analysis.snapshot.all_diagnostics().collect::<Vec<_>>()
    );
    assert_generic_target(&analysis, &module, "intValue", &source, "i", "Int");
    assert_generic_target(&analysis, &module, "floatValue", &source, "2.0i", "Float");
}

#[test]
fn integer_imaginary_literal_does_not_realize_against_float_scalar() {
    let source = r#"
from universe.numbers import BasisRealizable, Basis, I

data Strange<Marker, S>(_ real: S, _ imag: S)
data Tag()
impl<Marker, S> BasisRealizable<Basis<I>> for Strange<Marker, S> {
  type Scalar = S
  @class along(_ coefficient: S, _ basis: Basis<I>) -> Strange<Marker, S> { Strange(coefficient, coefficient) }
}
class Probe {
  @class floatValue() -> Strange<Tag, Float> { 2i }
}
"#;
    let (analysis, module, source) = analyze_algebra_source(source);
    let expression = expression_for(&analysis, &module, &source, "floatValue", "2i");
    assert!(
        expression.contextual_realization.is_none(),
        "Int coefficient must not be reinterpreted as Float"
    );
    assert!(analysis.snapshot.has_errors(), "the explicit Float target rejects the Int literal");
}

#[test]
fn inferred_imaginary_literal_keeps_canonical_default_without_user_conformance() {
    let source = strange_realization_source("2i", false);
    let (analysis, module, source) = analyze_algebra_source(&source);
    assert!(
        !analysis.snapshot.has_errors(),
        "unexpected diagnostics: {:?}",
        analysis.snapshot.all_diagnostics().collect::<Vec<_>>()
    );
    let expression = expression_for(&analysis, &module, &source, "value", "2i");
    let realization = expression
        .contextual_realization
        .as_deref()
        .unwrap_or_else(|| panic!("canonical realization missing: {expression:#?}"));
    let (declaration, arguments) = analysis
        .snapshot
        .store
        .applied_nominal_parts(realization.target_type)
        .expect("canonical target application");
    assert_eq!(declaration.name.as_ref(), "Complex");
    assert_eq!(arguments.len(), 1);
    assert_eq!(
        analysis
            .snapshot
            .store
            .nominal_origin_declaration(arguments[0])
            .map(|declaration| declaration.name.as_ref()),
        Some("Int")
    );
}

#[test]
fn importing_user_conformance_does_not_widen_inferred_default() {
    let source = strange_realization_source("2i", true);
    let (analysis, module, source) = analyze_algebra_source(&source);
    assert!(
        !analysis.snapshot.has_errors(),
        "unexpected diagnostics: {:?}",
        analysis.snapshot.all_diagnostics().collect::<Vec<_>>()
    );
    let expression = expression_for(&analysis, &module, &source, "value", "2i");
    let realization = expression
        .contextual_realization
        .as_deref()
        .unwrap_or_else(|| panic!("canonical realization missing: {expression:#?}"));
    let (declaration, arguments) = analysis
        .snapshot
        .store
        .applied_nominal_parts(realization.target_type)
        .expect("canonical target application");
    assert_eq!(declaration.name.as_ref(), "Complex");
    assert_eq!(arguments.len(), 1);
    assert_eq!(
        analysis
            .snapshot
            .store
            .nominal_origin_declaration(arguments[0])
            .map(|declaration| declaration.name.as_ref()),
        Some("Int")
    );
}

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
            let analysis = phalcom_semantic::analyze_single_module(module, source.clone(), program).expect("semantic test analysis should succeed");
            assert!(
                !analysis.snapshot.has_errors(),
                "unexpected diagnostics: {:?}",
                analysis.snapshot.all_diagnostics().collect::<Vec<_>>()
            );
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
