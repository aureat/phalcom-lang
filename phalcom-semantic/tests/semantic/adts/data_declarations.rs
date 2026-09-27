use std::sync::Arc;

use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::data_semantics::DataShape;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use crate::semantic::support::Fixture;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn data_declaration_publishes_nominal_components() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Point(x: Int, y: Int)
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));

    let point_decl = DeclarationId::new(module, "Point".into());
    let data_info = analysis.snapshot.data_semantics.data_info(&point_decl);
    assert!(data_info.is_some(), "data product for Point should be published");
    let info = data_info.unwrap();
    assert_eq!(info.owner.name.as_ref(), "Point");
    assert_eq!(info.components.len(), 2);
    assert_eq!(info.components[0].local_name.as_ref(), "x");
    assert_eq!(info.components[0].id.index, 0);
    assert_eq!(info.components[1].local_name.as_ref(), "y");
    assert_eq!(info.components[1].id.index, 1);
    assert!(matches!(info.shape, DataShape::Tuple));
    assert_eq!(info.constructor.parameters.len(), 2);
}

#[test]
fn data_record_declaration_publishes_named_components() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data User {
    name: String,
    age: Int,
}

"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));

    let user_decl = DeclarationId::new(module, "User".into());
    let data_info = analysis.snapshot.data_semantics.data_info(&user_decl);
    assert!(data_info.is_some(), "data product for User should be published");
    let info = data_info.unwrap();
    assert_eq!(info.components.len(), 2);
    assert_eq!(info.components[0].local_name.as_ref(), "name");
    assert_eq!(info.components[1].local_name.as_ref(), "age");
    assert!(matches!(info.shape, DataShape::Record));
}

#[test]
fn data_record_materialization_rejects_duplicate_fixed_and_expanded_labels() {
    use phalcom_semantic::data_semantics::{DataProductShapeMaterializationError, DataProductShapeState, materialize_data_product_shape};
    use phalcom_semantic::types::row::{RecordRowField, RecordRowTail};
    use phalcom_semantic::types::store::StaticArgument;

    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Envelope<**Fs: RecordFields> {
    id: Int,
    **Fs
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let declaration = DeclarationId::new(module, "Envelope".into());
    let info = analysis
        .snapshot
        .data_semantics
        .data_info(&declaration)
        .expect("Envelope data product")
        .as_ref()
        .clone();
    let field_type = info.components[0].declared_type.canonical_type().expect("fixed field type");
    let mut store = analysis.snapshot.store.as_ref().clone();

    let materialize_row = |store: &mut phalcom_semantic::types::store::TypeStore, fields: Vec<RecordRowField>| {
        let row = store.record_row_checked(fields, RecordRowTail::Closed).expect("well-formed materialized row");
        let specialized = store
            .apply_static_form(info.root_form, &[StaticArgument::RecordRow(row)])
            .expect("specialize Envelope with RecordFields row");
        materialize_data_product_shape(store, &info, specialized)
    };

    let concrete = materialize_row(
        &mut store,
        vec![RecordRowField {
            name: "extra".into(),
            ty: field_type,
        }],
    )
    .expect("non-conflicting row materializes");
    assert_eq!(concrete.state, DataProductShapeState::Concrete);
    assert_eq!(concrete.components.len(), 2);
    assert_eq!(concrete.components[0].label.as_deref(), Some("id"));
    assert_eq!(concrete.components[1].label.as_deref(), Some("extra"));

    let duplicate = materialize_row(
        &mut store,
        vec![
            RecordRowField {
                name: "id".into(),
                ty: field_type,
            },
            RecordRowField {
                name: "extra".into(),
                ty: field_type,
            },
        ],
    );
    assert!(
        matches!(duplicate, Err(DataProductShapeMaterializationError::DuplicateRecordField { ref field }) if field.as_ref() == "id"),
        "duplicate fixed/expanded label should fail precisely, got {duplicate:#?}"
    );
}

#[test]
fn data_record_materialization_preserves_symbolic_open_row_shape() {
    use phalcom_semantic::data_semantics::{DataProductShapeState, materialize_data_product_shape};
    use phalcom_semantic::types::id::KindId;
    use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
    use phalcom_semantic::types::row::{RecordRowField, RecordRowTail};
    use phalcom_semantic::types::store::StaticArgument;

    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Envelope<**Fs: RecordFields> {
    id: Int,
    **Fs
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let declaration = DeclarationId::new(module.clone(), "Envelope".into());
    let info = analysis
        .snapshot
        .data_semantics
        .data_info(&declaration)
        .expect("Envelope data product")
        .as_ref()
        .clone();
    let mut store = analysis.snapshot.store.as_ref().clone();
    let fixed_type = info.components[0].declared_type.canonical_type().expect("fixed field type");
    let open_tail = store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(DeclarationId::new(module, "OpenTail".into())),
        0,
        "Tail",
        KindId::RECORD_ROW,
    ));
    let open_row = store
        .record_row_checked(
            vec![RecordRowField {
                name: "name".into(),
                ty: fixed_type,
            }],
            RecordRowTail::Parameter(open_tail),
        )
        .expect("open RecordFields row");
    let specialized = store
        .apply_static_form(info.root_form, &[StaticArgument::RecordRow(open_row)])
        .expect("specialize Envelope with the open row");
    let (owner, arguments) = store.applied_nominal_static_parts(specialized).expect("nominal applied Envelope");
    assert_eq!(owner, info.owner);
    assert_eq!(arguments.as_slice(), &[StaticArgument::RecordRow(open_row)]);

    let shape = materialize_data_product_shape(&mut store, &info, specialized).expect("materialize symbolic shape");
    assert_eq!(shape.state, DataProductShapeState::Symbolic);
    assert_eq!(shape.components.len(), 2, "the unresolved tail is not treated as an empty concrete row");
    assert_eq!(shape.components[0].label.as_deref(), Some("id"));
    assert_eq!(shape.components[0].component, Some(info.components[0].id.clone()));
    assert_eq!(shape.components[0].ty, Some(fixed_type));
    assert_eq!(shape.components[1].label.as_deref(), Some("name"));
    assert_eq!(shape.components[1].component, None);
    assert_eq!(shape.components[1].ty, Some(fixed_type));
    assert_eq!(store.record_row(open_row).tail, RecordRowTail::Parameter(open_tail));
}

#[test]
fn expansion_field_projection_preserves_assumed_receiver_evidence() {
    use phalcom_semantic::checker::AssociatedResolutionKind;
    use phalcom_semantic::types::evidence::EvidenceStatus;

    let fixture = Fixture::new(
        r#"
data Envelope<**Fs: RecordFields> {
    **Fs
}

class Probe {
    read<T>(_ value: Envelope<**{ name: T }>) {
        value.name
    }
}
"#,
    );
    fixture.assert_no_error_diagnostics();

    let callable = fixture.callable("Probe", "read", phalcom_semantic::identity::DispatchSide::Instance);
    let projection = fixture.expression(callable, "value.name");
    let projected_type = projection.knowledge.ty().expect("exact expanded field type");
    assert_eq!(projection.knowledge.status(), Some(EvidenceStatus::Assumed));

    let resolution = callable.associated_resolutions.get(&projection.id).expect("expanded field resolution");
    let slot = match resolution.kind {
        AssociatedResolutionKind::DataExpansionField { logical_index, value_type } => {
            assert_eq!(value_type, projected_type);
            logical_index
        }
        ref other => panic!("expected exact expansion field, got {other:?}"),
    };
    assert_eq!(slot, 0, "the one-field exact row has logical slot zero");

}

#[test]
fn data_record_constructor_rejects_expected_row_repeating_fixed_label() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Envelope<**Fs: RecordFields> {
    id: Int,
    **Fs
}

class Probe {
    @class
    make() -> Envelope<**{ id: Int }> {
        Envelope { id: 1 }
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let diagnostics = analysis.snapshot.diagnostics_for(&module).expect("diagnostics for module");
    let constructor_text = "Envelope { id: 1 }";
    let constructor_start = source.find(constructor_text).expect("constructor source span");
    let constructor_end = constructor_start + constructor_text.len();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::GenericInferenceConflict
                && diagnostic.primary_range.start < constructor_end
                && constructor_start < diagnostic.primary_range.end
        }),
        "expected the constructor's fixed-field/expanded-row conflict to be diagnosed, got: {diagnostics:#?}"
    );
}

#[test]
fn data_constructor_visibility_is_published_in_the_canonical_product() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data PublicPoint(x: Int)
data PrivatePoint private(x: Int)
data ModulePoint module(x: Int)
data PackagePoint package(x: Int)
data InternalPoint internal(x: Int)
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));

    for (name, visibility) in [
        ("PublicPoint", "Public"),
        ("PrivatePoint", "Private"),
        ("ModulePoint", "Module"),
        ("PackagePoint", "Package"),
        ("InternalPoint", "Internal"),
    ] {
        let declaration = DeclarationId::new(module.clone(), name.into());
        let info = analysis.snapshot.data_semantics.data_info(&declaration).expect("data product");
        let debug = format!("{:?}", info.constructor);
        assert!(
            debug.contains(&format!("visibility: {visibility}")),
            "constructor product omitted {visibility}: {debug}"
        );
    }
}

#[test]
fn inaccessible_private_data_constructor_is_rejected_at_direct_construction() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Secret private(_ value: Int)

class Caller {
    @class make() -> Secret {
        Secret(42)
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diagnostics = analysis.snapshot.diagnostics_for(&module).expect("diagnostics for inaccessible construction");
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::MemberInaccessible),
        "private data construction should be rejected, got: {diagnostics:#?}"
    );
    assert!(
        !diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::AnnotationUnresolved),
        "existing inaccessible constructor must not look unresolved"
    );
    assert!(
        !diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedMemberMissing),
        "direct constructor must not look like a missing associated member"
    );
}

#[test]
fn private_data_constructor_is_available_to_its_owner() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Secret private(_ value: Int)

impl Secret {
    make(_ value: Int) -> Secret {
        Secret(value)
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diagnostics = analysis.snapshot.diagnostics_for(&module);
    assert!(
        diagnostics.is_none_or(|items| !items.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::MemberInaccessible)),
        "constructor owner should be allowed to construct Secret, got: {diagnostics:#?}"
    );
}

#[test]
fn generic_private_data_constructor_stays_restricted_after_specialization() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Box<T> private(_ value: T)

class Caller {
    @class make() -> Box<Int> {
        Box(42)
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    assert!(
        analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::MemberInaccessible),
        "generic private construction should remain restricted: {:?}",
        analysis.snapshot.diagnostics
    );
}

#[test]
fn nullary_private_data_constructor_is_rejected_before_singleton_lowering() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Marker private()

class Caller {
    @class make() -> Marker {
        Marker()
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    assert!(
        analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::MemberInaccessible),
        "nullary private construction should be rejected: {:?}",
        analysis.snapshot.diagnostics
    );
}

#[test]
fn inaccessible_private_record_constructor_is_rejected_at_direct_construction() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Secret private {
    value: Int,
}

class Caller {
    @class make() -> Secret {
        Secret { value: 42 }
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diagnostics = analysis
        .snapshot
        .diagnostics_for(&module)
        .expect("diagnostics for inaccessible record construction");
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::MemberInaccessible),
        "private record construction should be rejected, got: {diagnostics:#?}"
    );
}

#[test]
fn data_constructor_and_component_projection_in_body() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Point(x: Int, y: Int)

class Geometry {
    @class test() -> Int {
        let p = Point(x: 10, y: 20);
        p.x
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diags = analysis.snapshot.diagnostics_for(&module);
    assert!(diags.is_none() || diags.unwrap().is_empty(), "diagnostics: {:#?}", diags);

    let geom_decl = DeclarationId::new(module, "Geometry".into());
    let test_callable = CallableId::new(geom_decl, Selector::method("test", []).unwrap(), DispatchSide::Class);
    let callable_analysis = analysis.snapshot.callable_analyses.get(&test_callable);
    assert!(callable_analysis.is_some(), "CallableAnalysis for Geometry.test should exist");
}

#[test]
fn data_component_assignment_is_rejected() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Point(x: Int, y: Int)

class Geometry {
    @class test() {
        let p = Point(x: 10, y: 20);
        p.x = 30;
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diags = analysis.snapshot.diagnostics_for(&module).expect("diagnostics for module");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::DataComponentImmutable),
        "should emit DataComponentImmutable diagnostic, got: {:#?}",
        diags
    );
}

#[test]
fn data_duplicate_component_is_rejected() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Duplicate(x: Int, x: String)
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diags = analysis.snapshot.diagnostics_for(&module).expect("diagnostics for module");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::DataDuplicateComponent),
        "should emit DataDuplicateComponent diagnostic, got: {:#?}",
        diags
    );
}

#[test]
fn data_generic_box_specialization() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Box<T>(_ value: T)

class Container {
    @class unbox() -> Int {
        let b = Box(42);
        b.value
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diags = analysis.snapshot.diagnostics_for(&module);
    assert!(diags.is_none() || diags.unwrap().is_empty(), "diagnostics: {:#?}", diags);
}

#[test]
fn projection_free_complete_pack_data_constructor_infers_pack() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data PackBox<***Ps: CompleteTypePack>(***Ps)

class Runner {
    @class make() -> PackBox<***(Int, String)> {
        PackBox(1, "value")
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diagnostics = analysis.snapshot.diagnostics_for(&module);
    assert!(diagnostics.as_ref().is_none_or(|diagnostics| diagnostics.is_empty()), "diagnostics: {diagnostics:#?}");
}

#[test]
fn complete_pack_data_constructor_infers_nonzero_pack_without_expected_result() {
    use phalcom_semantic::types::store::{StaticArgument, TypeData};

    let fixture = Fixture::new(
        r#"
data PackBox<***Ps: CompleteTypePack>(***Ps)

class Probe {
    @class exercise() {
        let single = PackBox(1)
        let many = PackBox(1, "value")
        let labeled = PackBox(1, labeled: true)
        many
    }
}
"#,
    );
    fixture.assert_no_error_diagnostics();
    let exercise = fixture.callable("Probe", "exercise", DispatchSide::Class);
    let int = fixture.ty("Int");
    let string = fixture.ty("String");
    let bool = fixture.ty("Bool");

    for (source, expected_positionals, expected_labeled) in [
        ("PackBox(1)", vec![int], vec![]),
        ("PackBox(1, \"value\")", vec![int, string], vec![]),
        ("PackBox(1, labeled: true)", vec![int], vec![("labeled", bool)]),
    ] {
        let expression = fixture.expression(exercise, source);
        let result = expression.knowledge.ty().unwrap_or_else(|| {
            panic!(
                "missing inferred result for {source}: expression={expression:#?}; diagnostics={:#?}",
                fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>()
            )
        });
        let TypeData::Applied { arguments, .. } = fixture.analysis.snapshot.store.get(result) else {
            panic!("constructor result must retain user nominal Applied identity: {:#?}", expression);
        };
        let [StaticArgument::Term(complete_pack)] = arguments.as_ref() else {
            panic!("constructor must publish one CompleteTypePack argument: {arguments:#?}");
        };
        let TypeData::CompleteTypePack { positionals, labeled } = fixture.analysis.snapshot.store.get(*complete_pack) else {
            panic!(
                "constructor argument must be a CompleteTypePack: {:?}",
                fixture.analysis.snapshot.store.get(*complete_pack)
            );
        };
        let TypeData::TypePack(positionals) = fixture.analysis.snapshot.store.get(*positionals) else {
            panic!("positional lane must be TypePack");
        };
        assert_eq!(
            positionals.as_ref(),
            expected_positionals.as_slice(),
            "wrong inferred positional lane for {source}"
        );
        let TypeData::LabeledTypePack(labeled) = fixture.analysis.snapshot.store.get(*labeled) else {
            panic!("labeled lane must be LabeledTypePack");
        };
        assert_eq!(labeled.len(), expected_labeled.len(), "wrong labeled lane length for {source}");
        for (actual, (label, ty)) in labeled.iter().zip(expected_labeled) {
            assert_eq!(actual.label.as_ref(), label, "wrong labeled lane label for {source}");
            assert_eq!(actual.element, ty, "wrong labeled lane type for {source}");
        }
    }
}

#[test]
fn complete_pack_data_constructor_infers_exact_empty_pack_without_expected_result() {
    use phalcom_semantic::types::evidence::EvidenceStatus;
    use phalcom_semantic::types::store::{StaticArgument, TypeData};

    let fixture = Fixture::new(
        r#"
data PackBox<***Ps: CompleteTypePack>(***Ps)

class Probe {
    @class exercise() {
        PackBox()
    }
}
"#,
    );
    fixture.assert_no_error_diagnostics();
    let exercise = fixture.callable("Probe", "exercise", DispatchSide::Class);
    let expression = fixture.expression(exercise, "PackBox()");
    let result = expression.knowledge.ty().unwrap_or_else(|| {
        panic!(
            "zero constructor must infer nominal PackBox result: expression={expression:#?}; diagnostics={:#?}",
            fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>()
        )
    });
    let TypeData::Applied { arguments, .. } = fixture.analysis.snapshot.store.get(result) else {
        panic!("zero constructor result must retain nominal Applied identity: {expression:#?}");
    };
    let [StaticArgument::Term(complete)] = arguments.as_ref() else {
        panic!("zero constructor must publish one CompleteTypePack argument: {arguments:#?}");
    };
    let TypeData::CompleteTypePack { positionals, labeled } = fixture.analysis.snapshot.store.get(*complete) else {
        panic!("zero constructor argument must be a CompleteTypePack");
    };
    assert!(matches!(fixture.analysis.snapshot.store.get(*positionals), TypeData::TypePack(elements) if elements.is_empty()));
    assert!(matches!(fixture.analysis.snapshot.store.get(*labeled), TypeData::LabeledTypePack(elements) if elements.is_empty()));
    assert_eq!(expression.knowledge.status(), Some(EvidenceStatus::Established));
}

#[test]
fn complete_pack_data_constructor_empty_result_matches_cold_analysis() {
    use phalcom_semantic::types::evidence::EvidenceStatus;
    use phalcom_semantic::types::store::{StaticArgument, TypeData};

    let module = test_module();
    let initial_source = "data PackBox<***Ps: CompleteTypePack>(***Ps)\n";
    let source = r#"
data PackBox<***Ps: CompleteTypePack>(***Ps)

class Probe {
    @class exercise() {
        PackBox()
    }
}
"#;
    let mut session = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let _initial = session.update(crate::semantic::incremental::support::single_module_input(module.clone(), initial_source, 1));
    let incremental = session.update(crate::semantic::incremental::support::single_module_input(module.clone(), source, 2));
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let cold = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed.program));

    let summarize = |snapshot: &phalcom_semantic::snapshot::SemanticSnapshot| {
        let callable = CallableId::new(
            DeclarationId::new(module.clone(), "Probe".into()),
            Selector::method("exercise", []).expect("exercise selector"),
            DispatchSide::Class,
        );
        let expression = snapshot
            .callable_analyses
            .get(&callable)
            .expect("Probe.exercise analysis")
            .expressions
            .values()
            .find(|expression| source.get(expression.range.start..expression.range.end) == Some("PackBox()"))
            .expect("PackBox constructor expression");
        let result = expression.knowledge.ty().expect("inferred constructor result");
        let TypeData::Applied { arguments, .. } = snapshot.store.get(result) else {
            panic!("constructor result must retain nominal Applied identity: {expression:#?}");
        };
        let [StaticArgument::Term(complete)] = arguments.as_ref() else {
            panic!("constructor must publish one CompleteTypePack argument: {arguments:#?}");
        };
        let TypeData::CompleteTypePack { positionals, labeled } = snapshot.store.get(*complete) else {
            panic!("constructor argument must be CompleteTypePack");
        };
        assert!(matches!(snapshot.store.get(*positionals), TypeData::TypePack(elements) if elements.is_empty()));
        assert!(matches!(snapshot.store.get(*labeled), TypeData::LabeledTypePack(elements) if elements.is_empty()));
        assert_eq!(expression.knowledge.status(), Some(EvidenceStatus::Established));
        snapshot.store.format_type(result)
    };

    assert_eq!(summarize(&cold.snapshot), summarize(&incremental.snapshot));
}

#[test]
fn record_row_constructor_result_matches_cold_analysis() {
    use phalcom_semantic::types::evidence::EvidenceStatus;
    use phalcom_semantic::types::store::{StaticArgument, TypeData};

    let module = test_module();
    let initial_source = "data Envelope<**Fs: RecordFields> { id: Int, **Fs }\n";
    let source = r#"
data Envelope<**Fs: RecordFields> {
    id: Int,
    **Fs
}

class Probe {
    @class exercise() {
        Envelope { id: 1, name: "Ada", age: 42 }
    }
}
"#;
    let mut session = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let _initial = session.update(crate::semantic::incremental::support::single_module_input(module.clone(), initial_source, 1));
    let incremental = session.update(crate::semantic::incremental::support::single_module_input(module.clone(), source, 2));
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let cold = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed.program));

    let summarize = |snapshot: &phalcom_semantic::snapshot::SemanticSnapshot| {
        let callable = CallableId::new(
            DeclarationId::new(module.clone(), "Probe".into()),
            Selector::method("exercise", []).expect("exercise selector"),
            DispatchSide::Class,
        );
        let expression = snapshot
            .callable_analyses
            .get(&callable)
            .expect("Probe.exercise analysis")
            .expressions
            .values()
            .find(|expression| source.get(expression.range.start..expression.range.end) == Some("Envelope { id: 1, name: \"Ada\", age: 42 }"))
            .expect("Envelope constructor expression");
        let result = expression.knowledge.ty().expect("inferred constructor result");
        let TypeData::Applied { arguments, .. } = snapshot.store.get(result) else {
            panic!("constructor result must retain nominal Applied identity: {expression:#?}");
        };
        let [StaticArgument::RecordRow(row)] = arguments.as_ref() else {
            panic!("constructor must retain one exact RecordRow argument: {arguments:#?}");
        };
        let fields = snapshot
            .store
            .record_row(*row)
            .fields
            .iter()
            .map(|field| (field.name.to_string(), snapshot.store.format_type(field.ty)))
            .collect::<Vec<_>>();
        assert_eq!(fields.iter().map(|(label, _)| label.as_str()).collect::<Vec<_>>(), ["age", "name"]);
        assert_eq!(expression.knowledge.status(), Some(EvidenceStatus::Established));
        (snapshot.store.format_type(result), fields)
    };

    assert_eq!(summarize(&cold.snapshot), summarize(&incremental.snapshot));
}

#[test]
fn complete_pack_data_constructor_preserves_fixed_prefix_order_without_expected_result() {
    use phalcom_semantic::data_semantics::{DataProductShapeState, materialize_data_product_shape};
    use phalcom_semantic::types::store::{StaticArgument, TypeData};

    let fixture = Fixture::new(
        r#"
data Prefixed<***Ps: CompleteTypePack>(_ prefix: Int, ***Ps)

class Probe {
    @class exercise() {
        let value = Prefixed(7, "tail", true)
        value
    }
}
"#,
    );
    fixture.assert_no_error_diagnostics();
    let exercise = fixture.callable("Probe", "exercise", DispatchSide::Class);
    let expression = fixture.expression(exercise, "Prefixed(7, \"tail\", true)");
    let result = expression.knowledge.ty().expect("inferred nominal constructor result");
    let declaration = fixture.decl("Prefixed");
    assert_eq!(
        fixture.analysis.snapshot.store.nominal_origin_declaration(result),
        Some(&declaration)
    );
    let TypeData::Applied { arguments, .. } = fixture.analysis.snapshot.store.get(result) else {
        panic!("constructor result must preserve nominal Applied identity: {expression:#?}");
    };
    let [StaticArgument::Term(complete_pack)] = arguments.as_ref() else {
        panic!("expected one CompleteTypePack argument, got {arguments:#?}");
    };
    let TypeData::CompleteTypePack { positionals, labeled } = fixture.analysis.snapshot.store.get(*complete_pack) else {
        panic!("expected CompleteTypePack argument");
    };
    let TypeData::TypePack(pack_lanes) = fixture.analysis.snapshot.store.get(*positionals) else {
        panic!("expected positional TypePack lane");
    };
    assert_eq!(
        pack_lanes.as_ref(),
        &[fixture.ty("String"), fixture.ty("Bool")]
    );
    assert!(matches!(
        fixture.analysis.snapshot.store.get(*labeled),
        TypeData::LabeledTypePack(elements) if elements.is_empty()
    ));

    let info = fixture
        .analysis
        .snapshot
        .data_semantics
        .data_info(&declaration)
        .expect("Prefixed data product");
    let mut store = fixture.analysis.snapshot.store.as_ref().clone();
    let shape =
        materialize_data_product_shape(&mut store, info, result).expect("materialize inferred product shape");
    assert_eq!(shape.state, DataProductShapeState::Concrete);
    let ordered_types = shape.components.iter().map(|component| component.ty).collect::<Vec<_>>();
    assert_eq!(
        ordered_types,
        vec![Some(fixture.ty("Int")), Some(fixture.ty("String")), Some(fixture.ty("Bool"))]
    );
    assert!(shape.components[0].component.is_some(), "fixed prefix retains declaration component identity");
    assert!(
        shape.components[1..].iter().all(|component| component.component.is_none()),
        "expanded lanes do not invent component IDs"
    );
}

#[test]
fn record_row_constructor_infers_fixed_and_residual_fields_without_expected_result() {
    use phalcom_semantic::data_semantics::{DataProductShapeState, materialize_data_product_shape};
    use phalcom_semantic::types::row::RecordRowTail;
    use phalcom_semantic::types::store::{StaticArgument, TypeData};

    let fixture = Fixture::new(
        r#"
data Envelope<**Fs: RecordFields> {
    id: Int,
    **Fs
}

class Probe {
    @class exercise() {
        let value = Envelope { id: 1, name: "Ada", age: 42 }
        value
    }
}
"#,
    );
    fixture.assert_no_error_diagnostics();
    let exercise = fixture.callable("Probe", "exercise", DispatchSide::Class);
    let expression = fixture.expression(exercise, "Envelope { id: 1, name: \"Ada\", age: 42 }");
    let result = expression.knowledge.ty().unwrap_or_else(|| {
        panic!(
            "constructor result lacks inferred nominal type: {expression:#?}; diagnostics={:#?}",
            fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>()
        )
    });
    let declaration = fixture.decl("Envelope");
    assert_eq!(fixture.analysis.snapshot.store.nominal_origin_declaration(result), Some(&declaration));
    let TypeData::Applied { arguments, .. } = fixture.analysis.snapshot.store.get(result) else {
        panic!("constructor result must be nominal Applied: {expression:#?}");
    };
    let [StaticArgument::RecordRow(row_id)] = arguments.as_ref() else {
        panic!("Envelope must retain one exact RecordRow argument, got {arguments:#?}");
    };
    let row = fixture.analysis.snapshot.store.record_row(*row_id);
    assert_eq!(row.tail, RecordRowTail::Closed);
    assert_eq!(
        expression.knowledge.status(),
        Some(phalcom_semantic::types::evidence::EvidenceStatus::Established)
    );
    assert_eq!(
        row.fields.iter().map(|field| (field.name.as_ref(), field.ty)).collect::<Vec<_>>(),
        vec![("age", fixture.ty("Int")), ("name", fixture.ty("String"))]
    );

    let info = fixture.analysis.snapshot.data_semantics.data_info(&declaration).expect("Envelope data product");
    let mut store = fixture.analysis.snapshot.store.as_ref().clone();
    let shape = materialize_data_product_shape(&mut store, info, result).expect("materialize inferred product shape");
    assert_eq!(shape.state, DataProductShapeState::Concrete);
    assert_eq!(
        shape.components.iter().map(|component| component.label.as_deref()).collect::<Vec<_>>(),
        vec![Some("id"), Some("age"), Some("name")]
    );
    assert_eq!(
        shape.components.iter().map(|component| component.ty).collect::<Vec<_>>(),
        vec![Some(fixture.ty("Int")), Some(fixture.ty("Int")), Some(fixture.ty("String"))]
    );
    assert!(shape.components[0].component.is_some(), "fixed id retains declaration component identity");
    assert!(
        shape.components[1..].iter().all(|component| component.component.is_none()),
        "expanded RecordRow fields do not invent component IDs"
    );
}

#[test]
fn record_row_constructor_preserves_assumed_generic_field_evidence() {
    use phalcom_semantic::types::evidence::EvidenceStatus;
    use phalcom_semantic::types::store::StaticArgument;

    let fixture = Fixture::new(
        r#"
class Number {}

data Envelope<**Fs: RecordFields> {
    id: Int,
    **Fs
}

class Probe {
    @class
    make<T>(value: T) where T <: Number {
        let wrapped = Envelope { id: 1, name: value }
        wrapped
    }
}
"#,
    );
    let make = fixture.callable("Probe", "make", DispatchSide::Class);
    let parameter = &fixture.binding(make, "value").current;
    assert_eq!(parameter.status(), Some(EvidenceStatus::Assumed));
    let parameter_ty = parameter.ty().expect("generic parameter has a canonical type");
    let constructor = fixture.expression(make, "Envelope { id: 1, name: value }");
    let constructed_ty = constructor
        .knowledge
        .ty()
        .unwrap_or_else(|| panic!("generic-field constructor lost nominal result: {constructor:#?}"));
    let phalcom_semantic::types::store::TypeData::Applied { arguments, .. } = fixture.analysis.snapshot.store.get(constructed_ty) else {
        panic!("constructor result must remain nominal Applied: {constructor:#?}");
    };
    let [StaticArgument::RecordRow(row)] = arguments.as_ref() else {
        panic!("constructor must retain one inferred RecordRow, got {arguments:#?}");
    };
    assert_eq!(fixture.analysis.snapshot.store.record_row(*row).find_field("name"), Some(parameter_ty));
    assert_eq!(constructor.knowledge.status(), Some(EvidenceStatus::Assumed));
    assert_ne!(constructor.knowledge.status(), Some(EvidenceStatus::Established));
}

#[test]
fn record_row_constructor_meets_mixed_established_and_assumed_fields() {
    use phalcom_semantic::types::evidence::EvidenceStatus;

    let fixture = Fixture::new(
        r#"
data Envelope<**Fs: RecordFields> {
    id: Int,
    **Fs
}

class Probe {
    @class
    make<T>(value: T) {
        Envelope { id: 1, name: value, age: 42 }
    }
}
"#,
    );
    let make = fixture.callable("Probe", "make", DispatchSide::Class);
    let constructor = fixture.expression(make, "Envelope { id: 1, name: value, age: 42 }");
    assert!(
        constructor.knowledge.ty().is_some(),
        "nominal result survives mixed row evidence: {constructor:#?}"
    );
    assert_eq!(constructor.knowledge.status(), Some(EvidenceStatus::Assumed));
}

#[test]
fn record_row_constructor_publishes_exact_empty_row_as_established() {
    use phalcom_semantic::types::evidence::EvidenceStatus;
    use phalcom_semantic::types::row::RecordRowTail;
    use phalcom_semantic::types::store::{StaticArgument, TypeData};

    let fixture = Fixture::new(
        r#"
data Envelope<**Fs: RecordFields> {
    id: Int,
    **Fs
}

class Probe {
    @class
    make() {
        Envelope { id: 1 }
    }
}
"#,
    );
    let make = fixture.callable("Probe", "make", DispatchSide::Class);
    let constructor = fixture.expression(make, "Envelope { id: 1 }");
    let result = constructor.knowledge.ty().expect("empty exact row should publish nominal result");
    let TypeData::Applied { arguments, .. } = fixture.analysis.snapshot.store.get(result) else {
        panic!("expected nominal constructor result: {constructor:#?}");
    };
    let [StaticArgument::RecordRow(row)] = arguments.as_ref() else {
        panic!("expected one empty row argument: {arguments:#?}");
    };
    assert!(fixture.analysis.snapshot.store.record_row(*row).fields.is_empty());
    assert_eq!(fixture.analysis.snapshot.store.record_row(*row).tail, RecordRowTail::Closed);
    assert_eq!(constructor.knowledge.status(), Some(EvidenceStatus::Established));
}

#[test]
fn record_row_constructor_preserves_dynamic_residual_field_evidence() {
    use phalcom_semantic::types::evidence::{DynamicReason, TypeKnowledge};

    let fixture = Fixture::new(
        r#"
data Envelope<**Fs: RecordFields> {
    id: Int,
    **Fs
}

class Probe {
    @class
    make(_ dynamic: Dynamic) {
        Envelope { id: 1, value: dynamic }
    }
}
"#,
    );
    let make = fixture.callable("Probe", "make", DispatchSide::Class);
    let constructor = fixture.expression(make, "Envelope { id: 1, value: dynamic }");
    assert!(
        matches!(constructor.knowledge, TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape)),
        "dynamic residual-field evidence must not be replaced by row-solver uncertainty: {constructor:#?}"
    );
}

#[test]
fn record_row_constructor_preserves_unknown_residual_field_evidence() {
    use phalcom_semantic::types::evidence::TypeKnowledge;

    let fixture = Fixture::new(
        r#"
class Number {}

data Envelope<**Fs: RecordFields> {
    id: Int,
    **Fs
}

class Probe {
    @class
    make<T>(value: T) where T <: Number {
        let missing
        Envelope { id: 1, name: value, value: missing }
    }
}
"#,
    );
    let make = fixture.callable("Probe", "make", DispatchSide::Class);
    let constructor = fixture.expression(make, "Envelope { id: 1, name: value, value: missing }");
    assert!(
        matches!(constructor.knowledge, TypeKnowledge::Unknown(_)),
        "unknown residual-field evidence must remain unknown rather than being treated as an exact empty row: {constructor:#?}"
    );
}

#[test]
fn user_complete_pack_data_type_keeps_nominal_identity_distinct_from_tuple() {
    use phalcom_semantic::types::store::{StaticArgument, TypeData};

    let module = test_module();
    let source: Arc<str> = Arc::from("data MyTuple<***Es: CompleteTypePack>(***Es)\n");
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let declaration = DeclarationId::new(module.clone(), "MyTuple".into());
    let info = analysis
        .snapshot
        .data_semantics
        .data_info(&declaration)
        .expect("MyTuple semantic product");
    let mut store = analysis.snapshot.store.as_ref().clone();
    let int = store.nominal_type(DeclarationId::new(module.clone(), "Int".into()));
    let string = store.nominal_type(DeclarationId::new(module, "String".into()));
    let positionals = store.type_pack(vec![int, string].into_boxed_slice()).expect("positional pack");
    let labeled = store.labeled_type_pack(Box::new([])).expect("empty labeled pack");
    let complete = store.complete_type_pack(positionals, labeled).expect("complete pack");

    let nominal = store
        .apply_static_form(info.root_form, &[StaticArgument::Term(complete)])
        .expect("apply user MyTuple to CompleteTypePack");
    let structural = store.tuple_from_packs(positionals, labeled).expect("structural Tuple from same lanes");

    assert!(
        matches!(store.get(nominal), TypeData::Applied { arguments, .. } if arguments.as_ref() == [StaticArgument::Term(complete)]),
        "user data specialization must retain its nominal Applied identity, got {:?}",
        store.get(nominal)
    );
    assert!(matches!(store.get(structural), TypeData::Tuple(elements) if elements.len() == 2));
    assert_ne!(nominal, structural, "user MyTuple must not collapse to structural Tuple");
}

#[test]
fn user_record_row_data_type_keeps_nominal_identity_distinct_from_record() {
    use phalcom_semantic::types::row::{RecordRowField, RecordRowTail};
    use phalcom_semantic::types::store::{StaticArgument, TypeData};

    let module = test_module();
    let source: Arc<str> = Arc::from("data MyRecord<**Fs: RecordFields> { **Fs }\n");
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let declaration = DeclarationId::new(module.clone(), "MyRecord".into());
    let info = analysis
        .snapshot
        .data_semantics
        .data_info(&declaration)
        .expect("MyRecord semantic product");
    let mut store = analysis.snapshot.store.as_ref().clone();
    let int = store.nominal_type(DeclarationId::new(module.clone(), "Int".into()));
    let string = store.nominal_type(DeclarationId::new(module, "String".into()));
    let fields = vec![
        RecordRowField {
            name: "age".into(),
            ty: int,
        },
        RecordRowField {
            name: "name".into(),
            ty: string,
        },
    ];
    let row = store
        .record_row_checked(fields.clone(), RecordRowTail::Closed)
        .expect("heterogeneous RecordRow");

    let nominal = store
        .apply_static_form(info.root_form, &[StaticArgument::RecordRow(row)])
        .expect("apply user MyRecord to RecordRow");
    let structural = store.record(fields.into_boxed_slice());

    assert!(
        matches!(store.get(nominal), TypeData::Applied { arguments, .. } if arguments.as_ref() == [StaticArgument::RecordRow(row)]),
        "user data specialization must retain the exact RecordRow argument, got {:?}",
        store.get(nominal)
    );
    assert!(matches!(store.get(structural), TypeData::Record(found) if *found == row));
    assert_ne!(nominal, structural, "user MyRecord must not collapse to structural Record");
}

#[test]
fn data_record_construction_and_projection() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Person {
    name: String,
    age: Int,
}

class Directory {
    @class check() -> String {
        let p = Person { name: "Alice", age: 30 };
        p.name
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diags = analysis.snapshot.diagnostics_for(&module);
    assert!(diags.is_none() || diags.unwrap().is_empty(), "diagnostics: {:#?}", diags);
}

#[test]
fn data_phantom_applications_remain_distinct() {
    use phalcom_semantic::types::id::KindId;
    use phalcom_semantic::types::relation::{MapTypeHierarchy, is_subtype};
    use phalcom_semantic::types::store::TypeStore;

    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let module = test_module();

    let tagged_decl = DeclarationId::new(module.clone(), "Tagged".into());
    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let str_decl = DeclarationId::new(module, "String".into());

    let int_ty = store.nominal_type(int_decl);
    let str_ty = store.nominal_type(str_decl);

    let tagged_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let tagged_root = store.nominal_form(tagged_decl, tagged_kind);
    let tagged_int = store.apply_type_form(tagged_root, &[int_ty]).expect("apply tagged int");
    let tagged_str = store.apply_type_form(tagged_root, &[str_ty]).expect("apply tagged str");

    assert_ne!(tagged_int, tagged_str);
    assert!(is_subtype(&mut store, &hier, tagged_int, tagged_int));
    assert!(is_subtype(&mut store, &hier, tagged_str, tagged_str));
    assert!(!is_subtype(&mut store, &hier, tagged_int, tagged_str));
    assert!(!is_subtype(&mut store, &hier, tagged_str, tagged_int));
}

#[test]
fn data_declaration_cold_incremental_equivalence() {
    let module = test_module();
    let source = "data Point(x: Int, y: Int)\nclass Geometry { @class test() -> Int { let p = Point(x: 10, y: 20); p.x } }\n";
    let parsed1 = phalcom_ast::parse(source, 0);
    assert!(parsed1.errors.is_empty(), "parse errors: {:#?}", parsed1.errors);
    let cold_analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed1.program));

    let mut session = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let input = crate::semantic::incremental::support::single_module_input(module.clone(), source, 1);
    let update = session.update(input);

    let cold_point = cold_analysis
        .snapshot
        .data_semantics
        .data_info(&DeclarationId::new(module.clone(), "Point".into()));
    let incr_point = update.snapshot.data_semantics.data_info(&DeclarationId::new(module.clone(), "Point".into()));

    assert!(cold_point.is_some());
    assert!(incr_point.is_some());
    assert_eq!(cold_point.unwrap().components.len(), incr_point.unwrap().components.len());
}

#[test]
fn class_cannot_inherit_from_data() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Point(x: Int, y: Int)
class StrangePoint is Point {}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diags = analysis.snapshot.diagnostics_for(&module).expect("diagnostics expected");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::DataUsedAsSuperclass),
        "expected DataUsedAsSuperclass diagnostic, got: {:#?}",
        diags
    );
}

#[test]
fn class_cannot_inherit_from_data_incremental() {
    let module = test_module();
    let mut session = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let source1 = "class Base {}\nclass Derived is Base {}\n";
    let update1 = session.update(crate::semantic::incremental::support::single_module_input(module.clone(), source1, 1));
    let diags1 = update1.snapshot.diagnostics_for(&module);
    assert!(diags1.is_none_or(|d| d.is_empty()));

    let source2 = "data Base(x: Int)\nclass Derived is Base {}\n";
    let update2 = session.update(crate::semantic::incremental::support::single_module_input(module.clone(), source2, 2));
    let diags2 = update2.snapshot.diagnostics_for(&module).expect("diagnostics expected in update 2");
    assert!(
        diags2.iter().any(|d| d.code == DiagnosticCode::DataUsedAsSuperclass),
        "expected DataUsedAsSuperclass diagnostic after Base became data, got: {:#?}",
        diags2
    );
}

#[test]
fn data_cross_module_record_construction_order() {
    let module_a = ModuleId::resolved(
        ResolvedProjectId::from_raw(42),
        ModulePath::from_components(vec![phalcom_modules::identity::ModuleComponent::from_identifier("schema").unwrap()]),
    );
    let module_b = ModuleId::resolved(
        ResolvedProjectId::from_raw(42),
        ModulePath::from_components(vec![phalcom_modules::identity::ModuleComponent::from_identifier("app").unwrap()]),
    );

    let source_a = "data User { name: String, age: Int }\nexport User\n";
    let source_b = "import schema.User\nclass App { @class make() -> User { User(age: 30, name: \"Alice\") } }\n";

    let mut session = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let update = session.update(crate::semantic::incremental::support::multi_module_input(
        vec![(module_a.clone(), source_a.into()), (module_b.clone(), source_b.into())],
        1,
    ));

    let user_decl = DeclarationId::new(module_a, "User".into());
    let user_info = update.snapshot.data_semantics.data_info(&user_decl);
    assert!(user_info.is_some(), "data product for User should exist");
    let info = user_info.unwrap();
    assert_eq!(info.components.len(), 2);
    assert_eq!(info.components[0].local_name.as_ref(), "name");
    assert_eq!(info.components[1].local_name.as_ref(), "age");
}

#[test]
fn data_incremental_imported_component_type_mutation() {
    let module_a = ModuleId::resolved(
        ResolvedProjectId::from_raw(42),
        ModulePath::from_components(vec![phalcom_modules::identity::ModuleComponent::from_identifier("model").unwrap()]),
    );
    let module_b = ModuleId::resolved(
        ResolvedProjectId::from_raw(42),
        ModulePath::from_components(vec![phalcom_modules::identity::ModuleComponent::from_identifier("client").unwrap()]),
    );

    let source_a_v1 = "data Container(x: Int)\n";
    let source_b = "data Other(y: Int)\n";

    let mut session = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let update1 = session.update(crate::semantic::incremental::support::multi_module_input(
        vec![(module_a.clone(), source_a_v1.into()), (module_b.clone(), source_b.into())],
        1,
    ));
    let container_decl = DeclarationId::new(module_a.clone(), "Container".into());
    let info1 = update1.snapshot.data_semantics.data_info(&container_decl).unwrap();
    assert_eq!(info1.components.len(), 1);

    // Now change Container.x from Int to String in module A
    let source_a_v2 = "data Container(x: String)\n";
    let update2 = session.update(crate::semantic::incremental::support::multi_module_input(
        vec![(module_a.clone(), source_a_v2.into()), (module_b.clone(), source_b.into())],
        2,
    ));

    // Data declaration product in module A must reflect String
    let container_info = update2.snapshot.data_semantics.data_info(&container_decl).unwrap();
    assert_eq!(container_info.components.len(), 1);
    assert_ne!(info1, container_info);
}
