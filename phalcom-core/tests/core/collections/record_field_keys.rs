//! Runtime Record FieldKey construction, lookup, and ordered-label boundary.

use super::vm_support;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::product::{RecordProductShape, RecordProductShapeError, finish_tuple_value};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_semantic::field_key::FieldKey;
use phalcom_semantic::scalar::StaticInt;
use std::sync::Arc;

fn global(vm: &phalcom_core::vm::VM, module: phalcom_core::heap::ObjRef, name: &str) -> Value {
    let symbol = vm.interner.find(name).expect("global name is interned");
    vm.heap.module(module).get(symbol).expect("global exists")
}

fn send0(vm: &mut VM, receiver: Value, selector: &str) -> Value {
    let selector = vm.get_or_intern(selector);
    vm.send_dynamic(receiver, selector, &[]).unwrap_or_else(|error| panic!("send failed: {error}"))
}

fn send1(vm: &mut VM, receiver: Value, selector: &str, arg: Value) -> Value {
    let selector = vm.get_or_intern(selector);
    vm.send_dynamic(receiver, selector, &[arg])
        .unwrap_or_else(|error| panic!("send failed: {error}"))
}

fn tuple(vm: &mut VM, values: Vec<Value>) -> Value {
    finish_tuple_value(vm, values, Vec::new()).expect("tuple construction")
}

#[test]
fn construction_fixed_and_dynamic_keys_preserve_content_and_presentation_order() {
    let source = r#"
let symbol_key = #same
let huge = 1234567890123456789012345678901234567890
let record = { [symbol_key]: 10, ["same"]: 20, [42]: 30, [true]: 40, [huge]: 50 }
"#;
    let (vm, module) = vm_support::run_inline(source).expect("mixed-key Record program compiles and runs");
    let record = global(&vm, module, "record");
    let object = record.as_obj().expect("non-empty Record is heap-backed");
    let view = vm.record_view(object).expect("Record view");

    let huge = FieldKey::int(StaticInt::new(
        "1234567890123456789012345678901234567890"
            .parse::<num_bigint::BigInt>()
            .expect("valid test integer"),
    ));
    assert_eq!(
        view.entries(),
        vec![
            (FieldKey::symbol("same"), Value::int(10)),
            (FieldKey::string("same"), Value::int(20)),
            (FieldKey::int(42.into()), Value::int(30)),
            (FieldKey::bool(true), Value::int(40)),
            (huge.clone(), Value::int(50)),
        ],
        "presentation order must match source encounter order and retain Symbol/String distinction"
    );
    assert_eq!(
        view.logical_keys(),
        &[
            FieldKey::symbol("same"),
            FieldKey::string("same"),
            FieldKey::int(42.into()),
            huge.clone(),
            FieldKey::bool(true),
        ],
        "storage keys use canonical FieldKey order"
    );
    assert_eq!(view.get(&FieldKey::symbol("same")), Some(Value::int(10)));
    assert_eq!(view.get(&FieldKey::string("same")), Some(Value::int(20)));
    assert_eq!(view.get(&FieldKey::int(42.into())), Some(Value::int(30)));
    assert_eq!(view.get(&huge), Some(Value::int(50)));
    assert_eq!(view.get(&FieldKey::bool(true)), Some(Value::int(40)));
}

#[test]
fn construction_empty_record_remains_immediate() {
    let (vm, module) = vm_support::run_inline("let record = {}\n").expect("empty Record compiles and runs");
    let record = global(&vm, module, "record");
    assert!(record.is_empty_record());
    assert!(vm.record_view_value(record).expect("empty Record view").is_empty());
}

#[test]
fn lookup_surface_accepts_each_field_key_kind_and_returns_none_for_missing_key() {
    let source = r#"
let record = { [#same]: 10, ["same"]: 20, [42]: 30, [true]: 40 }
let by_symbol = record.get(#same)
let by_string = record.get("same")
let by_int = record.get(42)
let by_bool = record.get(true)
let missing = record.get(false)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Record.get accepts all FieldKey kinds");
    for (name, value) in [
        ("by_symbol", Value::int(10)),
        ("by_string", Value::int(20)),
        ("by_int", Value::int(30)),
        ("by_bool", Value::int(40)),
    ] {
        let expected = value.wrap_some().expect("integer wraps in Some");
        assert_eq!(global(&vm, module, name), expected, "lookup result `{name}`");
    }
    assert!(global(&vm, module, "missing").is_none());
}

#[test]
fn lookup_key_at_observes_each_field_key_kind() {
    let source = r#"
let record = { [#symbol]: 1, ["string"]: 2, [9]: 3, [false]: 4 }
let symbol_key = record.keyAt(0)
let string_key = record.keyAt(1)
let int_key = record.keyAt(2)
let bool_key = record.keyAt(3)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Record.keyAt supports heterogeneous keys");
    let symbol_key = global(&vm, module, "symbol_key");
    assert_eq!(symbol_key.as_symbol().ok().map(|symbol| vm.resolve_symbol(symbol)), Some("symbol"));
    let string_key = global(&vm, module, "string_key");
    assert_eq!(
        string_key.as_obj().and_then(|id| vm.heap.as_string(id)).map(|string| string.as_str()),
        Some("string")
    );
    assert_eq!(global(&vm, module, "int_key").as_int(), Some(9));
    assert_eq!(global(&vm, module, "bool_key").as_bool(), Some(false));
}

#[test]
fn construction_computed_duplicate_key_fails() {
    let error = match vm_support::run_inline("let key = 7\nlet record = { [key]: 1, [7]: 2 }\n") {
        Ok(_) => panic!("equal computed FieldKeys must be rejected"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("duplicate Record"), "unexpected error: {error}");
}

#[test]
fn evaluation_order_rejects_invalid_key_before_field_value() {
    let source = r#"
let trace: List<Int> = []
class Probe {
  @class
  badKey(_ value: Dynamic) -> Dynamic { value }
  @class
  sideEffect() -> Int { trace.append(1); 2 }
}
let record = { [Probe.badKey(1.5)]: Probe.sideEffect() }
"#;
    let (mut vm, program, _closure) = vm_support::compile_inline(source).expect("dynamic-key source compiles");
    let error = vm.run_compiled(&program).expect_err("Float key fails the runtime FieldKey guard");
    assert!(
        error.to_string().contains("Record field key must be Symbol, String, Int, or Bool"),
        "unexpected error: {error}"
    );
    let entry = program.initialization_order.last().expect("entry module");
    let module = vm.module_registry.get(entry).expect("partially initialized entry module").object;
    let trace = global(&vm, module, "trace");
    let trace_object = trace.as_obj().expect("trace list");
    assert!(
        vm.heap.list(trace_object).elements().is_empty(),
        "field value expression must not run after key validation fails"
    );
}

#[test]
fn evaluation_order_rejects_duplicate_dynamic_key_before_field_value() {
    let source = r#"
let trace: List<Int> = []
class Probe {
  @class
  dynamicKey(_ value: Dynamic) -> Dynamic { value }
  @class
  sideEffect() -> Int { trace.append(1); 2 }
}
let record = { [Probe.dynamicKey(7)]: 1, [7]: Probe.sideEffect() }
"#;
    let (mut vm, program, _closure) = vm_support::compile_inline(source).expect("dynamic duplicate-key source compiles");
    let error = match vm.run_compiled(&program) {
        Ok(()) => panic!("duplicate computed FieldKey must fail"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("duplicate Record field `7`"), "unexpected error: {error}");
    let entry = program.initialization_order.last().expect("entry module");
    let module = vm.module_registry.get(entry).expect("partially initialized entry module").object;
    let trace = global(&vm, module, "trace");
    let trace_object = trace.as_obj().expect("trace list");
    assert!(
        vm.heap.list(trace_object).elements().is_empty(),
        "duplicate key must be rejected before its field value expression runs"
    );
}

#[test]
fn construction_lowering_carries_exact_semantic_field_keys() {
    let source = "const record = { second: 2, first: 1 }\n";
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("Record program compiles");
    let lowering = &program.modules.get(&program.entry).expect("entry module").lowering;
    let spec = lowering
        .anonymous_products
        .values()
        .find(|spec| {
            matches!(
                spec.kind,
                phalcom_core::modules::semantic_lowering::AnonymousProductConstructionKind::Record { .. }
            )
        })
        .expect("Record lowering spec");
    let phalcom_core::modules::semantic_lowering::AnonymousProductConstructionKind::Record { logical_keys, .. } = &spec.kind else {
        unreachable!("filtered Record spec")
    };
    assert_eq!(logical_keys.as_ref(), &[FieldKey::symbol("first"), FieldKey::symbol("second"),]);
}

#[test]
fn lookup_large_mixed_shape_uses_canonical_binary_search_coordinates() {
    let count_per_kind = 3_333;
    let mut presentation = Vec::with_capacity(count_per_kind * 3 + 2);
    for i in (0..count_per_kind).rev() {
        presentation.push(FieldKey::int((i as i64).into()));
        presentation.push(FieldKey::string(Arc::<str>::from(format!("key-{i:05}"))));
        presentation.push(FieldKey::symbol(Arc::<str>::from(format!("key-{i:05}"))));
    }
    // Bool has two values, so add the complete distinct Bool domain once.
    presentation.push(FieldKey::bool(false));
    presentation.push(FieldKey::bool(true));

    let shape = RecordProductShape::from_presentation_keys(presentation).expect("10k mixed FieldKey shape");
    assert_eq!(shape.len(), count_per_kind * 3 + 2);
    for i in [0, 1, 41, 1_249, 2_499, 3_332] {
        let symbol = FieldKey::symbol(format!("key-{i:05}"));
        let string = FieldKey::string(format!("key-{i:05}"));
        let integer = FieldKey::int((i as i64).into());
        for key in [&symbol, &string, &integer] {
            let mut comparisons = 0;
            assert!(
                shape
                    .lookup_logical_index_by(|candidate| {
                        comparisons += 1;
                        candidate.canonical_cmp(key)
                    })
                    .is_some()
            );
            assert!(comparisons <= 15, "10k lookup must stay logarithmic, used {comparisons} comparisons");
        }
    }
    let mut comparisons = 0;
    assert!(
        shape
            .lookup_logical_index_by(|candidate| {
                comparisons += 1;
                candidate.canonical_cmp(&FieldKey::bool(true))
            })
            .is_some()
    );
    assert!(comparisons <= 15, "10k Bool lookup must stay logarithmic");
}

#[test]
fn construction_rejects_duplicate_keys_by_canonical_comparison() {
    let duplicate = RecordProductShape::from_presentation_keys(vec![FieldKey::symbol("same"), FieldKey::symbol("same")]);
    assert!(matches!(duplicate, Err(RecordProductShapeError::DuplicateKey(_))));
}

#[test]
fn construction_rejects_malformed_shape_permutation() {
    let error = RecordProductShape::new(
        vec![FieldKey::symbol("a"), FieldKey::symbol("b")].into_boxed_slice(),
        vec![0, 0].into_boxed_slice(),
    )
    .expect_err("duplicate presentation mapping must fail closed");
    assert_eq!(error, RecordProductShapeError::InvalidPermutation);
}

#[test]
fn typing_overlay_reflection_preserves_heterogeneous_field_keys() {
    let mut vm = VM::new();
    let typing_class = vm.universe.typing_classes.get("Typing").expect("Typing class");
    let debug = Value::symbol(vm.get_or_intern("debug"));
    let context_known = send1(&mut vm, Value::obj(typing_class), "contextFor(_)", debug);
    let context = send0(&mut vm, context_known, "value");
    let int_type = Value::obj(vm.universe.classes.int_class);

    let fields = [
        (Value::symbol(vm.get_or_intern("same")), int_type),
        (vm.alloc_string_value("same".to_owned()), int_type),
        (Value::int(42), int_type),
        (Value::bool(true), int_type),
    ]
    .into_iter()
    .map(|(key, ty)| tuple(&mut vm, vec![key, ty]))
    .collect();
    let fields = tuple(&mut vm, fields);
    let record_known = send1(&mut vm, context, "recordOf(_)", fields);
    let record_type = send0(&mut vm, record_known, "value");

    assert_eq!(send0(&mut vm, record_type, "fieldCount").as_int(), Some(4));
    let symbol_key = send1(&mut vm, record_type, "fieldKeyAt(_)", Value::int(0));
    assert_eq!(symbol_key.as_symbol().ok().map(|symbol| vm.resolve_symbol(symbol)), Some("same"));
    let string_key = send1(&mut vm, record_type, "fieldKeyAt(_)", Value::int(1));
    assert_eq!(
        string_key.as_obj().and_then(|id| vm.heap.as_string(id)).map(|string| string.as_str()),
        Some("same")
    );
    assert_eq!(send1(&mut vm, record_type, "fieldKeyAt(_)", Value::int(2)).as_int(), Some(42));
    assert_eq!(send1(&mut vm, record_type, "fieldKeyAt(_)", Value::int(3)).as_bool(), Some(true));

    let display = send0(&mut vm, record_type, "display");
    let display = vm.heap.as_string(display.as_obj().expect("display String")).expect("display text").as_str();
    assert!(display.contains("#same"), "Symbol key should remain visibly distinct: {display}");
    assert!(display.contains("\"same\""), "String key should remain visibly distinct: {display}");
}

#[test]
fn equality_hash_uses_canonical_tagged_keys_and_ignores_presentation_order() {
    let source = r#"
let left = { first: 1, second: 2, third: 3 }
let reordered = { third: 3, first: 1, second: 2 }
let reordered_equal = left == reordered
let reordered_hash_equal = left.hash == reordered.hash
let symbol_key = { [#same]: 1 }
let string_key = { ["same"]: 1 }
let symbol_string_differ = symbol_key != string_key
let integer_key = { [1]: 1 }
let boolean_key = { [true]: 1 }
let integer_boolean_differ = integer_key != boolean_key
let value_differ = left != { first: 1, second: 9, third: 3 }
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Record equality/hash source runs");
    for name in [
        "reordered_equal",
        "reordered_hash_equal",
        "symbol_string_differ",
        "integer_boolean_differ",
        "value_differ",
    ] {
        assert_eq!(global(&vm, module, name).as_bool(), Some(true), "{name}");
    }
}

#[test]
fn equality_hash_handles_large_reordered_record_linearly() {
    let fields = (0..256).map(|index| format!("field_{index:03}: {index}")).collect::<Vec<_>>();
    let mut reversed = fields.clone();
    reversed.reverse();
    let source = format!(
        "let left = {{ {} }}\nlet right = {{ {} }}\nlet same = left == right\nlet same_hash = left.hash == right.hash\n",
        fields.join(", "),
        reversed.join(", "),
    );
    let (vm, module) = vm_support::run_inline(&source).expect("large Record equality/hash runs");
    assert_eq!(global(&vm, module, "same").as_bool(), Some(true));
    assert_eq!(global(&vm, module, "same_hash").as_bool(), Some(true));
}

#[test]
fn spread_record_preserves_heterogeneous_keys_and_presentation_order() {
    let source = r#"
let source = { [#same]: 1, ["same"]: 2, [7]: 3, [true]: 4 }
let target = { before: 0, **source, after: 5 }
"#;
    let (vm, module) = vm_support::run_inline(source).expect("heterogeneous Record spread runs");
    let target = global(&vm, module, "target").as_obj().expect("nonempty Record object");
    assert_eq!(
        vm.record_view(target).expect("Record view").entries(),
        vec![
            (FieldKey::symbol("before"), Value::int(0)),
            (FieldKey::symbol("same"), Value::int(1)),
            (FieldKey::string("same"), Value::int(2)),
            (FieldKey::int(7.into()), Value::int(3)),
            (FieldKey::bool(true), Value::int(4)),
            (FieldKey::symbol("after"), Value::int(5)),
        ]
    );
}

#[test]
fn spread_tuple_and_pack_labeled_lanes_become_symbol_field_keys() {
    let source = r#"
let from_tuple = { **(first: 1, second: 2) }
let from_pack = { **Pack(third: 3, fourth: 4) }
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Tuple/Pack labels spread into Records");
    for (name, keys) in [
        ("from_tuple", vec![FieldKey::symbol("first"), FieldKey::symbol("second")]),
        ("from_pack", vec![FieldKey::symbol("third"), FieldKey::symbol("fourth")]),
    ] {
        let object = global(&vm, module, name).as_obj().expect("nonempty Record object");
        let entries = vm.record_view(object).expect("Record view").entries();
        assert_eq!(entries.iter().map(|(key, _)| key.clone()).collect::<Vec<_>>(), keys, "{name}");
    }
}

#[test]
fn spread_all_symbol_record_into_call_preserves_ordered_labels() {
    let source = r#"
class Receiver {
  collect(**labels) { return labels }
}
let record = { right: 2, left: 1 }
let labels = Receiver.new().collect(**record)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("all-Symbol Record call spread runs");
    let labels = global(&vm, module, "labels").as_obj().expect("labeled call result Pack");
    let view = vm.pack_view(labels).expect("Pack view");
    assert_eq!(
        view.labels().iter().map(|label| vm.resolve_symbol(*label)).collect::<Vec<_>>(),
        ["right", "left"]
    );
    assert_eq!(view.labeled_values(), [Value::int(2), Value::int(1)]);
}

#[test]
fn spread_known_non_symbol_record_into_ordered_lanes_is_rejected_semantically() {
    let source = r#"
class Receiver {
  collect(**labels) { return labels }
}
let record = { ["same"]: 1 }
let labels = Receiver.new().collect(**record)
"#;
    let error = match vm_support::run_inline(source) {
        Ok(_) => panic!("known String key cannot enter an ordered call lane"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("cannot enter a Symbol-only ordered labeled lane"),
        "unexpected error: {error}"
    );

    for source in [
        "let record = {[\"same\"]: 1}\nlet result = Pack(**record)\n",
        "let record = {[\"same\"]: 1}\nlet result = (**record,)\n",
    ] {
        let error = match vm_support::run_inline(source) {
            Ok(_) => panic!("known String key cannot enter an ordered product lane"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("cannot enter a Symbol-only ordered labeled lane"),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn spread_dynamic_non_symbol_record_into_call_fails_at_runtime_without_coercion() {
    let source = r#"
class Receiver {
  collect(**labels) { return labels }
}
class Builder {
  @class
  fromKey(_ key: Dynamic) -> Dynamic { {[key]: 1} }
}
let record = Builder.fromKey("same")
let labels = Receiver.new().collect(**record)
"#;
    let error = match vm_support::run_inline(source) {
        Ok(_) => panic!("runtime String key cannot become an ordered call label"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("cannot enter a Symbol-only labeled lane"),
        "unexpected error: {error}"
    );
}

#[test]
fn spread_duplicate_prevents_later_record_field_value() {
    let source = r#"
let trace: List<Int> = []
class Builder {
  @class
  make(_ key: Dynamic) -> Dynamic { {[key]: 1} }
}
class Probe {
  @class
  sideEffect() -> Int { trace.append(1); 2 }
}
let spread = Builder.make(#same)
let result = { **spread, same: Probe.sideEffect() }
"#;
    let (mut vm, program, _closure) = vm_support::compile_inline(source).expect("dynamic Record spread compiles");
    let error = vm.run_compiled(&program).expect_err("duplicate Record key fails during spread assembly");
    assert!(error.to_string().contains("duplicate Record field `#same`"), "unexpected error: {error}");
    let entry = program.initialization_order.last().expect("entry module");
    let module = vm.module_registry.get(entry).expect("partially initialized entry module").object;
    let trace = global(&vm, module, "trace").as_obj().expect("trace list");
    assert!(
        vm.heap.list(trace).elements().is_empty(),
        "field value expression must not run after a spread introduces its duplicate key"
    );
}
