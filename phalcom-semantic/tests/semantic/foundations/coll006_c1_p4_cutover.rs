use phalcom_semantic::checker::SequenceLiteralFamily;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::linker::SymbolId;
use phalcom_semantic::module_bindings::ModuleBindingPhase;
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::static_constant::StaticConstantId;

use crate::semantic::support::Fixture;
use crate::semantic::incremental::support::single_module_input;

#[test]
fn bare_runtime_sequence_literal_defaults_to_array() {
    let f = Fixture::new(
        r#"
class Container {
    @class test() {
        let values = [1, 2, 3]
    }
}
"#,
    );
    f.assert_no_error_diagnostics();

    let callable = f.callable("Container", "test", DispatchSide::Class);
    let binding = callable
        .bindings
        .values()
        .find(|binding| binding.name == "values")
        .expect("binding for values");
    let binding_type = binding.current.ty().expect("known Array type");
    let (declaration, arguments) = f
        .analysis
        .snapshot
        .store
        .applied_nominal_parts(binding_type)
        .expect("applied Array type");
    assert_eq!(declaration, f.decl("Array"));
    assert_eq!(arguments, [f.ty("Int")]);

    let resolution = callable.sequence_literals.values().next().expect("checked sequence literal");
    assert_eq!(resolution.family, SequenceLiteralFamily::Array);
    assert_eq!(resolution.result_type, binding_type);
}

#[test]
fn ambiguous_trait_dispatch_does_not_use_runtime_bracket_default_to_select_a_candidate() {
    let f = Fixture::new_allowing_internal_incidents(
        r#"
trait ListChoice {
    choose(_ values: List<Int>) -> Int { 1 }
}
trait VectorChoice {
    choose(_ values: Vector<Int, 1>) -> Int { 2 }
}
class User {}
impl ListChoice for User {}
impl VectorChoice for User {}
class Caller {
    read(_ user: User) { user.choose([1]) }
}
"#,
    );

    assert_eq!(f.diagnostics(DiagnosticCode::TraitDispatchAmbiguous).len(), 1);
    let callable = f.callable("Caller", "read", DispatchSide::Instance);
    let dispatch = callable
        .expressions
        .values()
        .find(|expression| expression.trait_dispatch_candidates.as_ref().is_some_and(|candidates| candidates.len() == 2))
        .expect("ambiguous expression retains both trait candidates");
    assert!(dispatch.trait_dispatch.is_none(), "ambiguous dispatch must not publish a selected target");

    let literal = callable
        .sequence_literals
        .values()
        .next()
        .expect("unbound argument is still checked under the runtime phase");
    assert_eq!(literal.family, SequenceLiteralFamily::Array);
}

#[test]
fn top_level_const_and_let_bindings_use_their_phase_default_families() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(608),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("cutover_phase").unwrap()]),
    );
    let symbol = SymbolId {
        module: module.clone(),
        name: "values".into(),
    };

    for (generation, source, expected_phase, expected_type, has_static_product) in [
        (
            1,
            "const values = [1, 2]\n",
            ModuleBindingPhase::StaticConstant,
            "Vector<Int, 2#const>",
            true,
        ),
        (2, "let values = [1, 2]\n", ModuleBindingPhase::Runtime, "Array<Int>", false),
    ] {
        let mut session = SemanticWorkspaceSession::new();
        let update = session.update(single_module_input(module.clone(), source, generation)).expect("semantic test update should succeed");
        assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

        let binding = update.snapshot.module_static_binding(&symbol).expect("canonical module binding");
        assert_eq!(binding.phase, expected_phase);
        assert_eq!(
            update.snapshot.store.format_type(binding.state.current.ty().expect("binding type")),
            expected_type
        );
        let constant = update
            .snapshot
            .static_constant(&StaticConstantId::new(module.clone(), "values"));
        assert_eq!(constant.is_some(), has_static_product);

        if let Some(constant) = constant {
            let resolution = constant.checked.sequence_literals.values().next().expect("checked static bracket");
            assert_eq!(resolution.family, SequenceLiteralFamily::Vector);
        }
    }
}
