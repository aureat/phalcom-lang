//! Incremental tests for Pack types, remainders, static definitions,
//! exact product invalidation, and warm/cold agreement.

use super::support::single_module_input;
use phalcom_common::selector::SelectorBase;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::db::fingerprint::{callable_signature_product_fingerprint, static_definition_product_fingerprint};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::snapshot::SemanticSnapshot;
use phalcom_semantic::static_constant::StaticConstantId;
use phalcom_semantic::static_eval::StaticEvalOutcome;
use phalcom_semantic::static_value::StaticValueData;

fn module(raw: u32) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(raw),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("pack_incr").expect("valid module component")]),
    )
}

#[test]
fn checked_static_pack_edit_recomputes_constant_and_matches_cold_analysis() {
    let module = module(407);
    let before = "type MakePack<N: Int>: Pack<***: Int> { Pack(N, other: 9, named: 7) }\n\
                  const Result: Pack<***: Int> = MakePack<3>\n";
    let after = "type MakePack<N: Int>: Pack<***: Int> { Pack(N, other: 9, named: 8) }\n\
                 const Result: Pack<***: Int> = MakePack<3>\n";
    let mut warm = SemanticWorkspaceSession::new();
    let initial = warm.update(single_module_input(module.clone(), before, 1)).expect("initial analysis");
    assert!(!initial.snapshot.has_errors(), "initial Pack must check: {:?}", initial.snapshot.diagnostics);
    let edited = warm.update(single_module_input(module.clone(), after, 2)).expect("incremental analysis");
    assert!(!edited.snapshot.has_errors(), "edited Pack must check: {:?}", edited.snapshot.diagnostics);
    let mut cold = SemanticWorkspaceSession::new();
    let rebuilt = cold.update(single_module_input(module.clone(), after, 1)).expect("cold analysis");
    assert_eq!(edited.snapshot.diagnostics, rebuilt.snapshot.diagnostics, "warm and cold Pack diagnostics must agree");

    let id = StaticConstantId::new(module, "Result");
    let pack = |snapshot: &SemanticSnapshot| {
        let product = snapshot.static_constant(&id).expect("dependent Pack constant");
        let StaticEvalOutcome::Reduced(value) = product.outcome else { panic!("Pack constant must reduce: {:?}", product.outcome); };
        let StaticValueData::Pack(pack) = snapshot.static_values().get(value).expect("Pack value") else { panic!("Pack value expected"); };
        let positionals = snapshot.static_values().pack_positionals(*pack).unwrap();
        let labeled = snapshot.static_values().pack_labeled(*pack).unwrap();
        assert_eq!(positionals.len(), 1);
        labeled.iter().map(|(label, value)| {
            (snapshot.static_values().get_symbol(*label).unwrap().to_owned(), snapshot.static_values().get(*value).unwrap().clone())
        }).collect::<Vec<_>>()
    };
    let first = pack(&initial.snapshot);
    let current = pack(&edited.snapshot);
    assert_ne!(first, current, "editing an exact Pack component must refresh the dependent constant");
    assert_eq!(current, pack(&rebuilt.snapshot), "incremental Pack value must agree with cold analysis");
    assert_eq!(current.iter().map(|(label, _)| label.as_str()).collect::<Vec<_>>(), vec!["other", "named"]);
}

fn callable_id(snapshot: &SemanticSnapshot, module: &ModuleId, owner: &str, name: &str, side: DispatchSide) -> CallableId {
    let owner = DeclarationId::new(module.clone(), owner.into());
    snapshot
        .callable_signatures
        .iter()
        .map(|(callable, _)| callable)
        .find(|callable| callable.owner == owner && callable.side == side && matches!(&callable.selector.base, SelectorBase::Named(base) if base == name))
        .cloned()
        .unwrap_or_else(|| panic!("missing callable {owner:?}.{name} on {side:?}"))
}

#[test]
fn pack_element_type_edit_invalidates_exact_dependent_product_and_reuses_unrelated() {
    let module = module(401);
    let mut session = SemanticWorkspaceSession::new();

    let source_a = r#"
type Payload = Pack<*: Int>

class Unrelated {
  @class id(_ x: Int) -> Int { x }
}

class Consumer {
  @class consume(_ p: Payload) -> Int { 42 }
}
"#;

    let update_a = session.update(single_module_input(module.clone(), source_a, 1)).expect("semantic test update should succeed");
    assert!(!update_a.snapshot.has_errors(), "initial source must be valid");

    let unrelated_id = callable_id(&update_a.snapshot, &module, "Unrelated", "id", DispatchSide::Class);
    let consumer_id = callable_id(&update_a.snapshot, &module, "Consumer", "consume", DispatchSide::Class);

    let sig_unrelated_a = update_a.snapshot.callable_signatures.get(&unrelated_id).expect("unrelated sig A");
    let sig_consumer_a = update_a.snapshot.callable_signatures.get(&consumer_id).expect("consumer sig A");
    let fp_unrelated_a = callable_signature_product_fingerprint(sig_unrelated_a);
    let fp_consumer_a = callable_signature_product_fingerprint(sig_consumer_a);

    // Edit Payload: Pack<*: Int> -> Pack<*: String>
    let source_b = r#"
type Payload = Pack<*: String>

class Unrelated {
  @class id(_ x: Int) -> Int { x }
}

class Consumer {
  @class consume(_ p: Payload) -> Int { 42 }
}
"#;

    let update_b = session.update(single_module_input(module.clone(), source_b, 2)).expect("semantic test update should succeed");
    assert!(!update_b.snapshot.has_errors(), "updated source must be valid");

    let sig_unrelated_b = update_b.snapshot.callable_signatures.get(&unrelated_id).expect("unrelated sig B");
    let sig_consumer_b = update_b.snapshot.callable_signatures.get(&consumer_id).expect("consumer sig B");
    let fp_unrelated_b = callable_signature_product_fingerprint(sig_unrelated_b);
    let fp_consumer_b = callable_signature_product_fingerprint(sig_consumer_b);

    // Exact dependent product signature changed
    assert_ne!(
        fp_consumer_a, fp_consumer_b,
        "editing Pack element type must invalidate dependent signature fingerprint"
    );

    // Unrelated declaration signature is preserved
    assert_eq!(fp_unrelated_a, fp_unrelated_b, "unrelated declaration signature must be preserved");

    // Cold vs warm check on source_b
    let mut cold_session = SemanticWorkspaceSession::new();
    let cold_update = cold_session.update(single_module_input(module.clone(), source_b, 1)).expect("semantic test update should succeed");
    let cold_sig_consumer = cold_update.snapshot.callable_signatures.get(&consumer_id).expect("cold consumer sig");
    let warm_param = sig_consumer_b.parameters[0].declared_type.canonical_type().unwrap();
    let cold_param = cold_sig_consumer.parameters[0].declared_type.canonical_type().unwrap();
    assert_eq!(
        update_b.snapshot.store.format_type(warm_param),
        cold_update.snapshot.store.format_type(cold_param),
        "warm and cold Pack parameter types must agree"
    );
}

#[test]
fn labeled_remainder_type_edit_invalidates_exact_dependent_product() {
    let module = module(402);
    let mut session = SemanticWorkspaceSession::new();

    let source_a = r#"
type LabeledPayload = Pack<**: Int>

class Consumer {
  @class consume(_ p: LabeledPayload) -> Int { 42 }
}
"#;

    let update_a = session.update(single_module_input(module.clone(), source_a, 1)).expect("semantic test update should succeed");
    assert!(!update_a.snapshot.has_errors(), "initial source must be valid");
    let consumer_id = callable_id(&update_a.snapshot, &module, "Consumer", "consume", DispatchSide::Class);
    let sig_consumer_a = update_a.snapshot.callable_signatures.get(&consumer_id).expect("consumer sig A");
    let fp_consumer_a = callable_signature_product_fingerprint(sig_consumer_a);

    // Edit labeled remainder: Int -> String
    let source_b = r#"
type LabeledPayload = Pack<**: String>

class Consumer {
  @class consume(_ p: LabeledPayload) -> Int { 42 }
}
"#;

    let update_b = session.update(single_module_input(module.clone(), source_b, 2)).expect("semantic test update should succeed");
    assert!(!update_b.snapshot.has_errors(), "updated source must be valid");
    let sig_consumer_b = update_b.snapshot.callable_signatures.get(&consumer_id).expect("consumer sig B");
    let fp_consumer_b = callable_signature_product_fingerprint(sig_consumer_b);

    assert_ne!(
        fp_consumer_a, fp_consumer_b,
        "editing labeled remainder in Pack must invalidate dependent signature fingerprint"
    );
}

#[test]
fn static_parameter_mode_edit_invalidates_definition_and_application() {
    let module = module(403);
    let mut session = SemanticWorkspaceSession::new();

    let source_a = r#"
type Worker<*Ts: TypePack> = Pack<*Ts>

class Consumer {
  @class use(_ p: Worker<*(Int, String)>) -> Int { 1 }
}

class Unrelated {
  @class keep() -> Int { 99 }
}
"#;

    let update_a = session.update(single_module_input(module.clone(), source_a, 1)).expect("semantic test update should succeed");
    assert!(!update_a.snapshot.has_errors(), "source A must be valid");

    let worker_id = DeclarationId::new(module.clone(), "Worker".into());
    let def_a = update_a.snapshot.static_definition(&worker_id).expect("worker def A");
    let def_fp_a = static_definition_product_fingerprint(def_a);

    let consumer_id = callable_id(&update_a.snapshot, &module, "Consumer", "use", DispatchSide::Class);
    let sig_a = update_a.snapshot.callable_signatures.get(&consumer_id).expect("consumer sig A");
    let consumer_fp_a = callable_signature_product_fingerprint(sig_a);

    let unrelated_id = callable_id(&update_a.snapshot, &module, "Unrelated", "keep", DispatchSide::Class);
    let sig_unrelated_a = update_a.snapshot.callable_signatures.get(&unrelated_id).expect("unrelated sig A");
    let unrelated_fp_a = callable_signature_product_fingerprint(sig_unrelated_a);

    // Edit mode of Ts from * (positional pack) to ** (labeled pack) and update application
    let source_b = r#"
type Worker<**Ts: LabeledTypePack> = Pack<**Ts>

class Consumer {
  @class use(_ p: Worker<**(x: Int, y: String)>) -> Int { 1 }
}

class Unrelated {
  @class keep() -> Int { 99 }
}
"#;

    let update_b = session.update(single_module_input(module.clone(), source_b, 2)).expect("semantic test update should succeed");
    assert!(!update_b.snapshot.has_errors(), "source B must be valid");

    let def_b = update_b.snapshot.static_definition(&worker_id).expect("worker def B");
    let def_fp_b = static_definition_product_fingerprint(def_b);

    let sig_b = update_b.snapshot.callable_signatures.get(&consumer_id).expect("consumer sig B");
    let consumer_fp_b = callable_signature_product_fingerprint(sig_b);

    let sig_unrelated_b = update_b.snapshot.callable_signatures.get(&unrelated_id).expect("unrelated sig B");
    let unrelated_fp_b = callable_signature_product_fingerprint(sig_unrelated_b);

    assert_ne!(def_fp_a, def_fp_b, "StaticParameter mode edit must invalidate definition product");
    assert_ne!(consumer_fp_a, consumer_fp_b, "StaticParameter mode edit must invalidate application product");
    assert_eq!(unrelated_fp_a, unrelated_fp_b, "unrelated declaration must remain reused");
}

#[test]
fn cold_and_warm_pack_results_agree() {
    let module = module(404);
    let initial = "type MyPack = Pack<*: Int>\nclass Consumer {\n  @class probe(_ p: MyPack) -> Int { 1 }\n}\n";
    let updated = "type MyPack = Pack<*: String>\nclass Consumer {\n  @class probe(_ p: MyPack) -> Int { 1 }\n}\n";

    let mut incremental = SemanticWorkspaceSession::new();
    let _ = incremental.update(single_module_input(module.clone(), initial, 1)).expect("semantic test update should succeed");
    let incremental_final = incremental.update(single_module_input(module.clone(), updated, 2)).expect("semantic test update should succeed");

    let mut cold = SemanticWorkspaceSession::new();
    let cold_final = cold.update(single_module_input(module.clone(), updated, 1)).expect("semantic test update should succeed");

    let alias = DeclarationId::new(module.clone(), "MyPack".into());
    let inc_form = incremental_final.snapshot.type_aliases.form(&alias).expect("incremental alias form");
    let cold_form = cold_final.snapshot.type_aliases.form(&alias).expect("cold alias form");

    assert_eq!(
        incremental_final.snapshot.store.format_type(inc_form),
        cold_final.snapshot.store.format_type(cold_form),
        "incremental and cold Pack forms must have equal structural formatting"
    );
    assert_eq!(
        incremental_final.snapshot.store.kind_of(inc_form),
        cold_final.snapshot.store.kind_of(cold_form),
        "incremental and cold Pack forms must have equal kinds"
    );

    let consumer_id = callable_id(&incremental_final.snapshot, &module, "Consumer", "probe", DispatchSide::Class);
    let inc_sig = incremental_final.snapshot.callable_signatures.get(&consumer_id).expect("inc sig");
    let cold_sig = cold_final.snapshot.callable_signatures.get(&consumer_id).expect("cold sig");
    let inc_param_ty = inc_sig.parameters[0].declared_type.canonical_type().unwrap();
    let cold_param_ty = cold_sig.parameters[0].declared_type.canonical_type().unwrap();
    assert_eq!(
        incremental_final.snapshot.store.format_type(inc_param_ty),
        cold_final.snapshot.store.format_type(cold_param_ty),
        "incremental and cold parameter types must have equal formatted structural types"
    );
}

#[test]
fn mapped_pack_constructor_edit_matches_cold_analysis_and_changes_contract() {
    let module = module(405);
    let initial = r#"
type Wrap<T> = List<T>
data MapCarrier<*Ts> { args: Pack<*(Ts::Map<Wrap>)> }
class MapConsumer { @class accept<*Ts>(_ args: Pack<*(Ts::Map<Wrap>)>) -> Unit { () } }
class Stable { @class value() -> Int { 1 } }
"#;
    let updated = initial.replace("List<T>", "Option<T>");
    let mut incremental = SemanticWorkspaceSession::new();
    let before = incremental.update(single_module_input(module.clone(), initial, 1)).expect("semantic test update should succeed");
    assert!(!before.snapshot.has_errors());
    let warm = incremental.update(single_module_input(module.clone(), &updated, 2)).expect("semantic test update should succeed");
    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session.update(single_module_input(module.clone(), &updated, 1)).expect("semantic test update should succeed");
    assert!(!warm.snapshot.has_errors());
    assert!(!cold.snapshot.has_errors());
    let consumer = callable_id(&warm.snapshot, &module, "MapConsumer", "accept", DispatchSide::Class);
    let parameter = |snapshot: &SemanticSnapshot| {
        let signature = snapshot.callable_signatures.get(&consumer).unwrap();
        // Formatting an abstract lambda prints only [TypeLambda]. Apply the
        // actual Map constructor to a concrete type to compare its semantics.
        use phalcom_semantic::types::store::TypeData;
        use phalcom_semantic::types::type_member::TypeMemberArgument;
        let pack = signature.parameters[0].declared_type.canonical_type().unwrap();
        let TypeData::PackFromPacks { positionals, .. } = snapshot.store.get(pack) else {
            panic!("symbolic Pack contract");
        };
        let TypeData::TypeMemberProjection(projection) = snapshot.store.get(*positionals) else {
            panic!("Map projection");
        };
        let TypeMemberArgument::Type(constructor) = projection.arguments[0] else {
            panic!("unary constructor argument");
        };
        let mut store = snapshot.store.as_ref().clone();
        let int = store.nominal(phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Int));
        let applied = store.apply_type_form(constructor, &[int]).unwrap();
        store.format_type(applied)
    };
    assert_ne!(
        parameter(&before.snapshot),
        parameter(&warm.snapshot),
        "Map constructor changes must invalidate the dependent Pack contract"
    );
    assert_eq!(parameter(&warm.snapshot), parameter(&cold.snapshot));
    let stable = callable_id(&warm.snapshot, &module, "Stable", "value", DispatchSide::Class);
    assert_eq!(
        callable_signature_product_fingerprint(before.snapshot.callable_signatures.get(&stable).unwrap()),
        callable_signature_product_fingerprint(warm.snapshot.callable_signatures.get(&stable).unwrap()),
    );
}
