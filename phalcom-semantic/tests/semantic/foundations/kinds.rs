use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{AssociatedStaticDomainDeclarationId, DeclarationId};
use phalcom_semantic::types::application::TypeApplicationError;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::kind::KindData;
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::store::{StaticArgument, StaticDataFormationError, TypeData, TypeStore};
use phalcom_semantic::types::substitution::{TypeSubstitution, TypeSubstitutionError};
use std::sync::Arc;

fn test_decl(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), name.into())
}

#[test]
fn kind_application_returns_residual_arrow_kind() {
    let mut store = TypeStore::new();
    let map_kind = store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);

    let residual = store.apply_kind(map_kind, &[KindId::TYPE]).unwrap();

    assert_eq!(
        store.get_kind(residual),
        &KindData::Arrow {
            parameters: vec![KindId::TYPE].into_boxed_slice(),
            result: KindId::TYPE,
        }
    );
}

#[test]
fn nested_type_application_canonicalizes() {
    let mut store = TypeStore::new();
    let map_decl = test_decl("Map");
    let string_decl = test_decl("String");
    let int_decl = test_decl("Int");

    let map_kind = store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let map = store.nominal_form(map_decl, map_kind);
    let string = store.nominal_type(string_decl);
    let int = store.nominal_type(int_decl);

    let partially = store.apply_type_form(map, &[string]).unwrap();
    let nested = store.apply_type_form(partially, &[int]).unwrap();
    let direct = store.apply_type_form(map, &[string, int]).unwrap();

    assert_eq!(nested, direct);
    assert_eq!(store.kind_of(direct), KindId::TYPE);
}

#[test]
fn heterogeneous_static_application_preserves_domain_terms_and_flattens() {
    let mut store = TypeStore::new();
    let shape_decl = test_decl("Shape");
    let shape_domain = store.nominal_static_domain(shape_decl.clone());
    let shape_kind = store.static_domain_kind(shape_domain);
    let width = store.const_int(3);
    let vector = store.intern_static_constructor(shape_domain, shape_decl, "Vector", vec![store.kind_of(width)].into_boxed_slice());
    let shape_vector = store
        .static_data(shape_domain, vector, vec![StaticArgument::Term(width)].into_boxed_slice())
        .expect("Shape::Vector(3)");

    let tensor_kind = store.arrow_kind(vec![KindId::TYPE, shape_kind].into_boxed_slice(), KindId::TYPE);
    let tensor = store.nominal_form(test_decl("Tensor"), tensor_kind);
    let float = store.nominal_type(test_decl("Float"));

    let partial = store.apply_static_form(tensor, &[StaticArgument::Term(float)]).expect("Tensor<Float, _>");
    let nested = store
        .apply_static_form(partial, &[StaticArgument::Term(shape_vector)])
        .expect("Tensor<Float, Shape::Vector(3)>");
    let direct = store
        .apply_static_form(tensor, &[StaticArgument::Term(float), StaticArgument::Term(shape_vector)])
        .expect("direct heterogeneous application");

    assert_eq!(nested, direct);
    assert!(matches!(store.get(direct), TypeData::Applied { arguments, .. } if arguments.as_ref() == [
        StaticArgument::Term(float),
        StaticArgument::Term(shape_vector),
    ]));
    assert_eq!(store.kind_of(direct), KindId::TYPE);
}

#[test]
fn heterogeneous_static_application_rejects_wrong_domain_without_coercion() {
    let mut store = TypeStore::new();
    let shape = store.nominal_static_domain(test_decl("Shape"));
    let endian = store.nominal_static_domain(test_decl("Endian"));
    let shape_kind = store.static_domain_kind(shape);
    let endian_kind = store.static_domain_kind(endian);
    let integer_kind = store.arrow_kind(vec![KindId::TYPE, shape_kind].into_boxed_slice(), KindId::TYPE);
    let integer = store.nominal_form(test_decl("Integer"), integer_kind);
    let bits = store.nominal_type(test_decl("Bits"));
    let endian_term = store.intern_static_constructor(endian, test_decl("Endian"), "Big", Box::new([]));
    let endian_big = store.static_data(endian, endian_term, Box::new([])).expect("Endian::Big");

    assert!(matches!(
        store.apply_static_form(integer, &[StaticArgument::Term(bits), StaticArgument::Term(endian_big)]),
        Err(TypeApplicationError::ArgumentKindMismatch { index: 1, expected, actual }) if expected == shape_kind && actual == endian_kind
    ));
}

#[test]
fn associated_static_domain_declaration_identity_survives_owner_specialization() {
    let mut store = TypeStore::new();
    let declaration = AssociatedStaticDomainDeclarationId::new(test_decl("Container"), "State");
    let int = store.nominal_type(test_decl("Int"));
    let string = store.nominal_type(test_decl("String"));

    let int_domain = store.associated_static_domain(declaration.clone(), vec![StaticArgument::Term(int)].into_boxed_slice());
    let same_int_domain = store.associated_static_domain(declaration.clone(), vec![StaticArgument::Term(int)].into_boxed_slice());
    let string_domain = store.associated_static_domain(declaration.clone(), vec![StaticArgument::Term(string)].into_boxed_slice());

    assert_eq!(int_domain, same_int_domain);
    assert_ne!(int_domain, string_domain);
    assert_eq!(declaration.owner, test_decl("Container"));
    assert_eq!(declaration.name.as_ref(), "State");

    let row_owner = test_decl("RowOwner");
    let owner_kind = store.arrow_kind(vec![KindId::TYPE, KindId::RECORD_ROW].into_boxed_slice(), KindId::TYPE);
    let owner_form = store.nominal_form(row_owner.clone(), owner_kind);
    let row = store
        .record_row_checked(Vec::new(), phalcom_semantic::types::row::RecordRowTail::Closed)
        .expect("closed record row");
    let applied = store
        .apply_static_form(owner_form, &[StaticArgument::Term(int), StaticArgument::RecordRow(row)])
        .expect("heterogeneous nominal application");
    assert_eq!(
        store.applied_nominal_static_parts(applied),
        Some((row_owner, vec![StaticArgument::Term(int), StaticArgument::RecordRow(row)]))
    );
}

#[test]
fn associated_static_domain_owner_preserves_mixed_term_row_and_constant_arguments() {
    use phalcom_semantic::types::row::RecordRowTail;
    use phalcom_semantic::types::store::StaticDomainData;

    let mut store = TypeStore::new();
    let owner = test_decl("Container");
    let declaration = AssociatedStaticDomainDeclarationId::new(owner.clone(), "State");
    let int = store.nominal_type(test_decl("Int"));
    let count = store.const_int(3);
    let row = store.record_row_checked(Vec::new(), RecordRowTail::Closed).expect("closed owner row argument");
    let owner_arguments = vec![StaticArgument::Term(int), StaticArgument::RecordRow(row), StaticArgument::Term(count)].into_boxed_slice();

    let domain = store.associated_static_domain(declaration.clone(), owner_arguments.clone());
    let same_domain = store.associated_static_domain(declaration.clone(), owner_arguments.clone());
    assert_eq!(domain, same_domain, "canonical owner environments reuse the domain identity");
    assert!(matches!(
        store.static_domain(domain),
        Some(StaticDomainData::Associated { declaration: current, owner_arguments: current_arguments })
            if current == &declaration && current_arguments == &owner_arguments
    ));

    let domain_form = store.associated_static_domain_form(domain);
    assert_eq!(store.kind_of(domain_form), store.static_domain_kind(domain));
    let constructor = store.intern_static_constructor(domain, owner, "Ready", Box::new([]));
    let ready = store.static_data(domain, constructor, Box::new([])).expect("associated case");
    assert!(matches!(store.get(ready), TypeData::StaticData(term) if term.domain == domain));
}

#[test]
fn checked_substitution_reports_invalid_heterogeneous_application() {
    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(test_decl("Owner"));
    let parameter = store.intern_type_parameter(TypeParameterData::new(owner, 0, "T", KindId::TYPE));
    let parameter_form = store.parameter_form(parameter);
    let list_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let list = store.nominal_form(test_decl("List"), list_kind);
    let applied = store.apply_type_form(list, &[parameter_form]).expect("List<T>");
    let mut substitution = TypeSubstitution::new();
    substitution.bind(parameter, list);

    assert!(matches!(
        substitution.try_apply(&mut store, applied),
        Err(TypeSubstitutionError::Application(TypeApplicationError::ArgumentKindMismatch { index: 0, .. }))
    ));
}

#[test]
fn wrong_kind_parameter_fails() {
    let mut store = TypeStore::new();
    let list_decl = test_decl("List");
    let map_decl = test_decl("Map");

    let list_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let map_kind = store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);

    let list = store.nominal_form(list_decl, list_kind);
    let map = store.nominal_form(map_decl, map_kind);

    let err = store.apply_type_form(list, &[map]).unwrap_err();
    assert!(matches!(
        err,
        TypeApplicationError::ArgumentKindMismatch {
            index: 0,
            expected: KindId::TYPE,
            actual,
        } if actual == map_kind
    ));
}

#[test]
fn too_many_arguments_fails() {
    let mut store = TypeStore::new();
    let list_decl = test_decl("List");
    let int_decl = test_decl("Int");

    let list_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let list = store.nominal_form(list_decl, list_kind);
    let int = store.nominal_type(int_decl);

    let err = store.apply_type_form(list, &[int, int]).unwrap_err();
    assert_eq!(err, TypeApplicationError::TooManyArguments { supplied: 2, accepted: 1 });
}

#[test]
fn applying_proper_int_as_constructor_fails() {
    let mut store = TypeStore::new();
    let int_decl = test_decl("Int");
    let int = store.nominal_type(int_decl);

    let err = store.apply_type_form(int, &[int]).unwrap_err();
    assert_eq!(
        err,
        TypeApplicationError::NotAConstructor {
            origin: int,
            kind: KindId::TYPE,
        }
    );
}

#[test]
fn class_object_distinct_from_nominal() {
    let mut store = TypeStore::new();
    let int_decl = test_decl("Int");

    let nominal_int = store.nominal_type(int_decl.clone());
    let class_obj_int = store.class_object_type(int_decl);

    assert_ne!(nominal_int, class_obj_int);
    assert_eq!(store.kind_of(nominal_int), KindId::TYPE);
    assert_eq!(store.kind_of(class_obj_int), KindId::TYPE);
    assert!(store.is_proper_type(nominal_int));
    assert!(store.is_proper_type(class_obj_int));
}

#[test]
fn every_interned_type_has_explicit_kind() {
    let store = TypeStore::new();
    assert_eq!(store.kind_of(store.never()), KindId::TYPE);
    assert_eq!(store.kind_of(store.unit()), KindId::TYPE);
    assert!(store.is_proper_type(store.never()));
    assert!(store.is_proper_type(store.unit()));
}

#[test]
fn static_domains_and_const_ints_are_distinct_from_proper_types() {
    let mut store = TypeStore::new();
    let width = store.nominal_static_domain(test_decl("Width"));
    let height = store.nominal_static_domain(test_decl("Height"));
    assert_ne!(width, height);

    let width_kind = store.static_domain_kind(width);
    let const_int = store.const_int(3);
    assert_eq!(store.kind_of(const_int), store.static_domain_kind(store.const_int_domain()));
    assert!(matches!(store.get_kind(width_kind), KindData::StaticDomain(_)));
    assert_eq!(store.proper_type(const_int), Err(store.kind_of(const_int)));
}

#[test]
fn static_data_is_canonical_and_rejects_wrong_domain_or_payload_kind() {
    let mut store = TypeStore::new();
    let owner = test_decl("Shape");
    let domain = store.nominal_static_domain(owner.clone());
    let other_domain = store.nominal_static_domain(test_decl("OtherShape"));
    let const_int = store.const_int(3);
    let const_kind = store.kind_of(const_int);
    let constructor = store.intern_static_constructor(domain, owner, "Vector", vec![const_kind].into_boxed_slice());

    let first = store
        .static_data(domain, constructor, vec![StaticArgument::Term(const_int)].into_boxed_slice())
        .expect("valid static constructor application");
    let second = store
        .static_data(domain, constructor, vec![StaticArgument::Term(const_int)].into_boxed_slice())
        .expect("same static constructor application");
    assert_eq!(first, second);
    assert!(matches!(store.get(first), TypeData::StaticData(_)));
    assert_eq!(store.proper_type(first), Err(store.kind_of(first)));

    assert!(matches!(
        store.static_data(other_domain, constructor, vec![StaticArgument::Term(const_int)].into_boxed_slice()),
        Err(StaticDataFormationError::DomainMismatch { .. })
    ));
    let runtime_type = store.nominal_type(test_decl("Runtime"));
    assert!(matches!(
        store.static_data(domain, constructor, vec![StaticArgument::Term(runtime_type)].into_boxed_slice()),
        Err(StaticDataFormationError::ArgumentKindMismatch { index: 0, .. })
    ));
}

#[test]
fn source_type_data_publishes_a_static_domain_kind() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from("type data Width(_ value: Int)\n");
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);

    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    assert!(!analysis.snapshot.has_errors(), "semantic diagnostics: {:?}", analysis.snapshot.diagnostics);

    let declaration = DeclarationId::new(module, "Width".into());
    let form = analysis.snapshot.declarations.form(&declaration).expect("type data declaration form");
    assert!(matches!(
        analysis.snapshot.store.get_kind(analysis.snapshot.store.kind_of(form)),
        KindData::StaticDomain(_)
    ));
    assert!(!analysis.snapshot.store.is_proper_type(form));
    assert_eq!(analysis.snapshot.store.static_constructor_count(), 1);
}

#[test]
fn source_generic_binder_resolves_named_static_domain_kind() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from("type data Shape { Scalar }\ndata Tensor<T: Type, S: Shape>()\n");
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);

    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    assert!(!analysis.snapshot.has_errors(), "semantic diagnostics: {:?}", analysis.snapshot.diagnostics);
    let tensor = DeclarationId::new(module, "Tensor".into());
    let signature = analysis.snapshot.declarations.generic_signature(&tensor).expect("Tensor generic signature");
    assert_eq!(analysis.snapshot.store.type_parameter(signature.parameters[0]).kind, KindId::TYPE);
    assert!(matches!(
        analysis
            .snapshot
            .store
            .get_kind(analysis.snapshot.store.type_parameter(signature.parameters[1]).kind),
        KindData::StaticDomain(_)
    ));
}

#[test]
fn source_generic_binder_resolves_associated_static_domain_kind_through_type_form() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        "type data Shape { Scalar }\n\
         class Container<T: Type> { type data State { Ready } }\n\
         data Holder<S: Container<Int>::State>()\n",
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);

    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    assert!(!analysis.snapshot.has_errors(), "semantic diagnostics: {:?}", analysis.snapshot.diagnostics);
    let holder = DeclarationId::new(module, "Holder".into());
    let signature = analysis.snapshot.declarations.generic_signature(&holder).expect("Holder generic signature");
    let kind = analysis.snapshot.store.type_parameter(signature.parameters[0]).kind;
    let phalcom_semantic::types::kind::KindData::StaticDomain(domain) = analysis.snapshot.store.get_kind(kind) else {
        panic!("expected associated static-domain kind, got {}", analysis.snapshot.store.format_kind(kind));
    };
    assert!(matches!(
        analysis.snapshot.store.static_domain(*domain),
        Some(phalcom_semantic::types::store::StaticDomainData::Associated { declaration, owner_arguments })
            if declaration.owner.name.as_ref() == "Container" && owner_arguments.len() == 1
    ));
}

#[test]
fn type_data_case_cannot_be_used_as_a_runtime_value() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        "type data Shape { Scalar }\n\
         class Probe {\n\
           @class run() {\n\
             let value = Shape::Scalar\n\
             value\n\
           }\n\
         }\n",
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let analysis = analyze_single_module(module, source, Arc::new(parsed.program));
    assert!(
        analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| matches!(diagnostic.code, DiagnosticCode::TypeOnlyRuntimeValue)),
        "type-data case must be rejected in runtime value position: {:?}",
        analysis.snapshot.diagnostics
    );
}
