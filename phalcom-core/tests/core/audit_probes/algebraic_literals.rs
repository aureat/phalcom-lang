//! Runtime probes for contextual algebraic basis values and native suffix literals.

use phalcom_core::heap::ObjRef;
use phalcom_core::modules::compile::{CompiledProgram, EntrySelection, ProgramAnalyzer, ProgramCompiler};
use phalcom_core::typing::inspect;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::fs;
use tempfile::TempDir;

fn compile(source: &str) -> (TempDir, CompiledProgram) {
    let temp = TempDir::new().expect("temporary package");
    fs::write(temp.path().join("package.ph"), "expose .main\n").expect("manifest");
    fs::write(temp.path().join("main.ph"), source).expect("source");
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Package(temp.path().to_path_buf())).expect("algebraic literal package compiles");
    (temp, program)
}

fn run(source: &str) -> (VM, ObjRef) {
    let (_temp, program) = compile(source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("algebraic literal package executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    (vm, module)
}

fn global(vm: &VM, module: ObjRef, name: &str) -> Value {
    vm.heap
        .module(module)
        .get(vm.interner.find(name).expect("binding symbol"))
        .expect("binding value")
}

fn exact_tuple_type(vm: &VM, value: Value) -> String {
    let descriptor = vm.heap.tuple(value.as_obj().expect("tuple value")).descriptor();
    let exact = vm
        .anonymous_product_descriptors
        .get(descriptor)
        .expect("tuple descriptor")
        .exact_type
        .expect("exact tuple type");
    inspect::display(&vm.runtime_typing_context, &vm.typing_registry, &vm.heap, exact)
}

fn exact_data_type(vm: &VM, value: Value) -> String {
    let descriptor = vm.heap.data(value.as_obj().expect("data value")).descriptor;
    let exact = vm
        .data_registry
        .descriptor(descriptor)
        .expect("data descriptor")
        .exact_type
        .expect("exact data type");
    inspect::display(&vm.runtime_typing_context, &vm.typing_registry, &vm.heap, exact)
}

fn on_algebraic_stack(name: &'static str, body: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name(name.into())
        .stack_size(8 * 1024 * 1024)
        .spawn(body)
        .expect("spawn bounded algebraic realization test")
        .join()
        .expect("algebraic realization test must not overflow an 8 MiB stack");
}

#[test]
fn native_imaginary_literals_and_basis_products_are_runtime_values() {
    std::thread::Builder::new()
        .name("algebraic-literals-runtime".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let (vm, module) = run(r#"
from universe.numbers import Complex, Quaternion, i, j
let typedComplex: Complex<Float> = i
let typedQuaternion: Quaternion<Float> = j
let complex = 2i
let quaternion = 1 + 2i + 3j + 4k
let equivalent = 2i === Complex(0, 2)
let hamilton = (Quaternion(0, 1, 0, 0) * Quaternion(0, 0, 1, 0)) === Quaternion(0, 0, 0, 1)
"#);
            assert_eq!(global(&vm, module, "equivalent"), Value::bool(true));
            assert_eq!(global(&vm, module, "hamilton"), Value::bool(true));
            assert!(global(&vm, module, "typedComplex").is_obj());
            assert!(global(&vm, module, "typedQuaternion").is_obj());
            assert!(global(&vm, module, "complex").is_obj());
            assert!(global(&vm, module, "quaternion").is_obj());
        })
        .expect("spawn algebraic literals runtime thread")
        .join()
        .expect("algebraic literals runtime must not overflow an 8 MiB stack");
}

#[test]
fn class_side_trait_default_runs_at_runtime() {
    std::thread::Builder::new()
        .name("class-side-trait-default-runtime".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let (vm, module) = run(r#"
trait Describable {
  @class describe() -> String { "undescribed" }
}
data Widget(_ value: Int)
impl Describable for Widget {}
let description = Widget.describe()
"#);
            let description = global(&vm, module, "description");
            let text = vm.heap.as_string(description.as_obj().expect("String object")).expect("String value");
            assert_eq!(text.as_str(), "undescribed");
        })
        .expect("spawn class-side trait runtime thread")
        .join()
        .expect("class-side trait default must not overflow an 8 MiB stack");
}

#[test]
fn contextual_realization_preserves_side_effect_order() {
    on_algebraic_stack("algebraic-realization-order", || {
        let (vm, module) = run(r#"
from universe.numbers import Complex, Quaternion, i, j
class Probe {
  @class mut _counter = 0
  @class coefficient() -> Int {
    _counter = _counter + 1
    return _counter
  }
  @class counter { _counter }
}
let x = Probe.coefficient() * i
let once = Probe.counter == 1
let firstTerm: Quaternion<Int> = Probe.coefficient() * i
let secondTerm: Quaternion<Int> = Probe.coefficient() * j
let y = firstTerm + secondTerm
let thrice = Probe.counter == 3
let firstThenSecond = y === Quaternion(0, 2, 3, 0)
"#);
        assert_eq!(global(&vm, module, "once"), Value::bool(true));
        assert_eq!(global(&vm, module, "thrice"), Value::bool(true));
        assert_eq!(
            global(&vm, module, "firstThenSecond"),
            Value::bool(true),
            "the i coefficient must be evaluated before the j coefficient"
        );
        assert_eq!(exact_data_type(&vm, global(&vm, module, "x")), "Complex<Int>");
        assert_eq!(exact_data_type(&vm, global(&vm, module, "y")), "Quaternion<Int>");
    });
}

#[test]
fn scalar_basis_operand_orders_have_the_same_exact_value() {
    on_algebraic_stack("algebraic-realization-operand-order", || {
        let source = r#"
from universe.numbers import Complex, i
let coefficientFirst = 3i
let basisFirst = i * 3
let same = coefficientFirst === basisFirst
"#;
        let (_temp, program) = compile(source);
        let start = source.find("i * 3").expect("basis-first operand order");
        let basis_first_range = phalcom_common::range::SourceRange::new(start, start + "i * 3".len());
        assert!(
            program.modules[&program.entry]
                .lowering
                .contextual_realizations
                .contains_key(&basis_first_range),
            "basis-first product must publish its canonical realization"
        );
        let mut vm = VM::new();
        vm.run_compiled(&program).expect("basis-first realization executes");
        let module = vm.module_registry.get(&program.entry).expect("entry module").object;
        let coefficient_first = global(&vm, module, "coefficientFirst");
        let basis_first = global(&vm, module, "basisFirst");
        assert_eq!(global(&vm, module, "same"), Value::bool(true));
        assert_eq!(exact_data_type(&vm, coefficient_first), "Complex<Int>");
        assert_eq!(exact_data_type(&vm, basis_first), "Complex<Int>");
    });
}

#[test]
fn generic_assumed_receiver_projects_realization_prerequisite_slots() {
    on_algebraic_stack("algebraic-realization-generic-projection", || {
        let source = r#"
from universe.numbers import Complex, i, Zero, One
class ImaginaryFactory {
  @class
  makeImaginary<T>(_ seed: T) -> Complex<T> where T |= Zero, T |= One { i }
}
"#;
        use phalcom_core::modules::semantic_lowering::ForIterationRequirementSpec;
        use phalcom_core::typing::RuntimeConformanceSlotRecipe;
        use phalcom_semantic::checker::analysis::RealizationCapabilitySelection;
        use phalcom_semantic::generic_evidence::{GenericConformanceEvidence, GenericEvidenceEnvironment, GenericEvidenceSlotKey};

        let temp = TempDir::new().expect("temporary package");
        fs::write(temp.path().join("package.ph"), "expose .main\n").expect("manifest");
        fs::write(temp.path().join("main.ph"), source).expect("source");
        let analyzed = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Package(temp.path().to_path_buf())).expect("generic realization analyzes");
        assert!(!analyzed.semantic.has_errors(), "{:#?}", analyzed.semantic.diagnostics);
        let (caller, realization) = analyzed
            .semantic
            .callable_analyses
            .iter()
            .filter(|(callable, _)| callable.owner.module() == &analyzed.entry)
            .find_map(|(callable, analysis)| {
                analysis
                    .expressions
                    .values()
                    .find_map(|expression| expression.contextual_realization.as_ref().map(|realization| (callable, realization)))
            })
            .expect("generic contextual realization");
        let RealizationCapabilitySelection::Evidenced(selection) = &realization.realization else {
            panic!("Complex<T> must have a selected source realization")
        };
        let GenericConformanceEvidence::Derived(evidence) = &selection.evidence else {
            panic!("conditional Complex<T> realization must retain its derived proof")
        };
        let signature = analyzed.semantic.conformance_witness_plans[&selection.source_impl]
            .generic_signature
            .as_ref()
            .expect("impl prerequisites");
        let impl_layout =
            GenericEvidenceEnvironment::try_from_constraints(&signature.constraints, signature.owner.clone(), &analyzed.semantic.trait_surfaces).unwrap();
        let caller_signature = analyzed.semantic.callable_signatures.get(caller).unwrap().generics.as_ref().unwrap();
        let caller_layout =
            GenericEvidenceEnvironment::try_from_constraints(&caller_signature.constraints, caller_signature.owner.clone(), &analyzed.semantic.trait_surfaces)
                .unwrap();
        let program = ProgramCompiler::compile_analyzed(&analyzed).expect("generic realization projects");
        let realization = program.modules[&program.entry]
            .lowering
            .contextual_realizations
            .values()
            .next()
            .expect("generic contextual realization");
        let ForIterationRequirementSpec::Selected(invocation) = &realization.realization else {
            panic!("selected along invocation")
        };
        assert_eq!(
            invocation.selection, *selection,
            "projection must preserve the selected callable and derived proof"
        );
        let recipe = invocation
            .call_environment_recipe
            .as_ref()
            .expect("along must receive its implementation prerequisites");
        assert_eq!(evidence.nested.len(), 2);
        assert_eq!(recipe.conformance_slots.len(), impl_layout.len());
        for (constraint, nested) in signature.constraints.iter().zip(evidence.nested.iter()) {
            let phalcom_semantic::types::parameter::GenericConstraint::Conforms { subject, trait_ref } = constraint else {
                panic!("conformance prerequisite")
            };
            let phalcom_semantic::generic_evidence::NestedGenericEvidence::Conformance(GenericConformanceEvidence::Assumed(assumed)) = nested else {
                panic!("caller-assumed prerequisite")
            };
            let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(subject.clone(), trait_ref.clone());
            for (slot_key, target_slot) in impl_layout.layout().slots.iter().filter(|(slot_key, _)| slot_key.conformance == key) {
                let caller_key = GenericEvidenceSlotKey::new(assumed.key.clone(), slot_key.requirement.clone());
                assert_eq!(
                    recipe.conformance_slots[*target_slot as usize],
                    RuntimeConformanceSlotRecipe::Forwarded(caller_layout.slot_for(&caller_key).unwrap()),
                    "{:?} must keep its canonical impl/caller slot mapping",
                    slot_key.requirement
                );
            }
        }
        assert_eq!(
            impl_layout
                .layout()
                .slots
                .iter()
                .map(|(key, _)| key.requirement.owner.name.as_ref())
                .collect::<std::collections::BTreeSet<_>>(),
            ["One", "Zero"].into_iter().collect()
        );
    });
}

#[test]
fn generic_assumed_receiver_realizes_exact_complex_types() {
    on_algebraic_stack("algebraic-realization-generic", || {
        let (vm, module) = run(r#"
from universe.numbers import Complex, i, Zero, One
class ImaginaryFactory {
  @class
  makeImaginary<T>(_ seed: T) -> Complex<T> where T |= Zero, T |= One { i }
}
let intImag = ImaginaryFactory.makeImaginary(1)
let floatImag = ImaginaryFactory.makeImaginary(1.0)
let intExact = intImag === Complex(0, 1)
let floatExact = floatImag === Complex(0.0, 1.0)
"#);
        assert_eq!(global(&vm, module, "intExact"), Value::bool(true));
        assert_eq!(global(&vm, module, "floatExact"), Value::bool(true));
        assert_eq!(exact_data_type(&vm, global(&vm, module, "intImag")), "Complex<Int>");
        assert_eq!(exact_data_type(&vm, global(&vm, module, "floatImag")), "Complex<Float>");
    });
}

#[test]
fn open_world_real_embedding_keeps_exact_user_target_type() {
    on_algebraic_stack("algebraic-realization-open-world", || {
        let (vm, module) = run(r#"
from universe.numbers import RealEmbeddable, Zero
data Plane<T>(_ x: T, _ y: T)
impl<T> RealEmbeddable for Plane<T> where T |= Zero {
  type Scalar = T
  @class fromReal(_ value: T) -> Plane<T> { Plane(value, T.zero) }
}
let point: Plane<Int> = 5
let expected = Plane<Int>(5, 0)
let exact = point === expected
"#);
        assert_eq!(global(&vm, module, "exact"), Value::bool(true));
        assert_eq!(exact_data_type(&vm, global(&vm, module, "point")), "Plane<Int>");
    });
}

#[test]
fn contextual_realization_survives_collection_pressure() {
    on_algebraic_stack("algebraic-realization-gc", || {
        let source = r#"
from universe.numbers import Complex, i
class Probe {
  @class mut _counter = 0
  @class coefficient() -> Int {
    System.gc
    _counter = _counter + 1
    return _counter
  }
  @class counter { _counter }
}
let result = Probe.coefficient() * i
let counterValue = Probe.counter
"#;
        let (_temp, program) = compile(source);
        let mut vm = VM::new();
        vm.force_gc();
        vm.run_compiled(&program).expect("GC-pressure realization executes");
        let module = vm.module_registry.get(&program.entry).expect("entry module").object;
        vm.force_gc();
        let result = global(&vm, module, "result");
        assert_eq!(exact_data_type(&vm, result), "Complex<Int>");
        assert_eq!(global(&vm, module, "counterValue"), Value::int(1));
    });
}

#[test]
fn basis_values_preserve_nominal_axis_identity_through_aliases() {
    let (mut vm, module) = run(r#"
from universe.numbers import Basis, I, J, K, i as imaginary, j, k
import universe.numbers as numbers
class Identity {
  @class
  of<T>(_ value: T) -> T { value }
}
let unit = imaginary
let explicitI = Basis<I>()
let sameAxis = unit === explicitI
let differentAxes = not (unit === j) and not (j === k)
let explicitAxes = j === Basis<J>() and k === Basis<K>()
let qualifiedAxis = numbers.i === unit
let retainedAxis = Identity.of(unit) === unit
"#);
    vm.force_gc();
    assert_eq!(
        global(&vm, module, "unit"),
        global(&vm, module, "explicitI"),
        "canonical imported basis and direct exact-axis construction must agree",
    );
    for name in ["sameAxis", "differentAxes", "explicitAxes", "qualifiedAxis", "retainedAxis"] {
        assert_eq!(global(&vm, module, name), Value::bool(true), "{name}");
    }
}

#[test]
fn quaternion_constructor_arithmetic_is_exact_and_order_sensitive() {
    let (mut vm, module) = run(r#"
from universe.numbers import Quaternion
let a = Quaternion(1, 2, 3, 4)
let b = Quaternion(5, 6, 7, 8)
let hamilton = (a*b) === Quaternion(-60, 12, 30, 24)
let reverse = (b*a) === Quaternion(-60, 20, 14, 32)
let added = (a+b) === Quaternion(6, 8, 10, 12)
let subtracted = (b-a) === Quaternion(4, 4, 4, 4)
let negated = (-a) === Quaternion(-1, -2, -3, -4)
let conjugated = a.conjugate === Quaternion(1, -2, -3, -4)
let norm = a.normSquared == 30
let associative = ((a*b)*a) === (a*(b*a))
let conjugationLaw = (a*b).conjugate === (b.conjugate*a.conjugate)
let huge = Quaternion(10**100, 0, 0, 0) * Quaternion(2, 0, 0, 0)
let exact = huge.real === 2*(10**100)
"#);
    vm.force_gc();
    for name in [
        "hamilton",
        "reverse",
        "added",
        "subtracted",
        "negated",
        "conjugated",
        "norm",
        "associative",
        "conjugationLaw",
        "exact",
    ] {
        assert_eq!(global(&vm, module, name), Value::bool(true), "{name}");
    }
}

#[test]
fn quaternion_unavailable_capabilities_do_not_create_dispatch_evidence() {
    let (_temp, program) = compile(
        r#"
from universe.numbers import Quaternion
let result = Quaternion(true, false, true, false) + Quaternion(false, true, false, true)
"#,
    );
    let lowering = &program.modules[&program.entry].lowering;
    assert!(lowering.conditional_invocations.is_empty());
    assert!(lowering.trait_invocations.is_empty());
    assert!(lowering.trait_requirement_invocations.is_empty());
    let mut vm = VM::new();
    let error = vm.run_compiled(&program).expect_err("unavailable arithmetic fails through ordinary dispatch");
    assert!(format!("{error:?}").contains("UnsupportedOperation"), "{error:?}");
}

#[test]
fn quaternion_complex_part_preserves_exact_applied_identity() {
    let (mut vm, module) = run(r#"
from universe.numbers import Quaternion, Complex
let projected = Quaternion(1, 2, 3, 4).complexPart
let expectedProjection = Complex(1, 2)
let projection = projected === expectedProjection
"#);
    vm.force_gc();
    let projected = global(&vm, module, "projected");
    let expected = global(&vm, module, "expectedProjection");
    assert_eq!(vm.get_data_component(projected, 0).unwrap(), Value::int(1));
    assert_eq!(vm.get_data_component(projected, 1).unwrap(), Value::int(2));
    let projected_descriptor = vm.heap.data(projected.as_obj().unwrap()).descriptor;
    let expected_descriptor = vm.heap.data(expected.as_obj().unwrap()).descriptor;
    assert_eq!(
        projected_descriptor,
        expected_descriptor,
        "projection: {:?} versus {:?}",
        vm.data_registry.descriptor(projected_descriptor),
        vm.data_registry.descriptor(expected_descriptor),
    );
    assert_eq!(exact_data_type(&vm, projected), "Complex<Int>");
    assert_eq!(global(&vm, module, "projection"), Value::bool(true));
}

#[test]
fn covering_impl_generic_product_preserves_exact_applied_identity() {
    let source = r#"
data Box<T>(_ value: T)
impl<T> Box<T> {
  pair -> (T, Int) { (self.value, 1) }
}
class Maker {
  @class
  pair<T>(_ value: T) -> (T, Int) { (value, 1) }
}
let pair = Box(7).pair
let expected = Maker.pair(7)
let exact = pair === expected
"#;
    let (_temp, program) = compile(source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("covering impl package executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    vm.force_gc();
    let pair = global(&vm, module, "pair");
    let expected = global(&vm, module, "expected");
    assert_eq!(exact_tuple_type(&vm, pair), "(Int, Int)");
    assert_eq!(global(&vm, module, "exact"), Value::bool(true));
    assert_eq!(
        vm.heap.tuple(pair.as_obj().expect("pair tuple")).descriptor(),
        vm.heap.tuple(expected.as_obj().expect("expected tuple")).descriptor(),
        "covering impl generic product must reify the same applied tuple type",
    );
}

#[test]
fn covering_impl_nested_generic_call_preserves_exact_applied_identity() {
    let source = r#"
class Identity {
  @class
  of<T>(_ value: T) -> (T, Int) { (value, 1) }
}
data Box<T>(_ value: T)
impl<T> Box<T> {
  project -> (T, Int) { Identity.of(self.value) }
}
class Maker {
  @class
  pair<T>(_ value: T) -> (T, Int) { (value, 1) }
}
let projected = Box(7).project
let expected = Maker.pair(7)
let exact = projected === expected
"#;
    let (_temp, program) = compile(source);
    let nested_start = source.find("Identity.of(self.value)").expect("nested call");
    let recipe = program.modules[&program.entry]
        .lowering
        .call_environment_recipes
        .iter()
        .find(|(range, _)| range.start == nested_start)
        .map(|(_, recipe)| recipe)
        .expect("checked nested generic call recipe");
    assert!(
        recipe
            .bindings
            .iter()
            .any(|(_, ty)| matches!(ty, phalcom_core::typing::RuntimeTypeRecipe::Template(_))),
        "nested call must receive an impl-relative template type argument",
    );
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("nested covering impl package executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    vm.force_gc();
    let projected = global(&vm, module, "projected");
    let expected = global(&vm, module, "expected");
    assert_eq!(exact_tuple_type(&vm, projected), "(Int, Int)");
    assert_eq!(global(&vm, module, "exact"), Value::bool(true));
    assert_eq!(
        vm.heap.tuple(projected.as_obj().expect("projected tuple")).descriptor(),
        vm.heap.tuple(expected.as_obj().expect("expected tuple")).descriptor(),
        "nested generic call must retain the covering impl's applied type",
    );
}

#[test]
fn covering_impl_forwarded_type_arguments_reify_in_both_coordinates() {
    let (mut vm, module) = run(r#"
data Box<T>(_ value: T)
impl<T> Box<T> {
  project -> Box<T> { Box<T>(self.value) }
}
class Forward {
  @class
  project<T>(_ box: Box<T>) -> Box<T> { box.project }
}
let intProjected = Forward.project(Box(7))
let intExpected = Box(7)
let floatProjected = Forward.project(Box(7.0))
let floatExpected = Box(7.0)
let intExact = intProjected === intExpected
let floatExact = floatProjected === floatExpected
"#);
    vm.force_gc();
    for (projected, expected, exact_type) in [("intProjected", "intExpected", "Box<Int>"), ("floatProjected", "floatExpected", "Box<Float>")] {
        let projected = global(&vm, module, projected);
        let expected = global(&vm, module, expected);
        assert_eq!(exact_data_type(&vm, projected), exact_type);
        assert_eq!(
            vm.heap.data(projected.as_obj().expect("projected data")).descriptor,
            vm.heap.data(expected.as_obj().expect("expected data")).descriptor,
        );
    }
    assert_eq!(global(&vm, module, "intExact"), Value::bool(true));
    assert_eq!(global(&vm, module, "floatExact"), Value::bool(true));
}

#[test]
fn independent_covering_impl_binders_do_not_collide() {
    let (mut vm, module) = run(r#"
data First<T>(_ value: T)
data Second<T>(_ value: T)
impl<T> First<T> {
  wrap -> Second<T> { Second<T>(self.value) }
}
impl<T> Second<T> {
  wrap -> First<T> { First<T>(self.value) }
}
let first = First(7).wrap
let second = Second(7.0).wrap
let intExpected = Second(7)
let floatExpected = First(7.0)
let intExact = first === intExpected
let floatExact = second === floatExpected
"#);
    vm.force_gc();
    for (actual, expected, exact_type) in [("first", "intExpected", "Second<Int>"), ("second", "floatExpected", "First<Float>")] {
        let actual = global(&vm, module, actual);
        let expected = global(&vm, module, expected);
        assert_eq!(exact_data_type(&vm, actual), exact_type);
        assert_eq!(
            vm.heap.data(actual.as_obj().expect("actual data")).descriptor,
            vm.heap.data(expected.as_obj().expect("expected data")).descriptor,
        );
    }
    assert_eq!(global(&vm, module, "intExact"), Value::bool(true));
    assert_eq!(global(&vm, module, "floatExact"), Value::bool(true));
}

#[test]
fn covering_impl_semantic_specialization_stays_declaration_canonical() {
    let source = r#"
data Box<T>(_ value: T)
impl<T> Box<T> {
  wrap -> Box<T> { Box<T>(self.value) }
}
let result = Box(7).wrap
"#;
    let analyzed = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Inline(source.into())).expect("covering impl source analyzes");
    assert!(!analyzed.semantic.has_errors(), "{:#?}", analyzed.semantic.diagnostics);
    let expression = analyzed
        .semantic
        .callable_analyses
        .iter()
        .filter(|(callable, _)| callable.owner.module() == &analyzed.entry)
        .flat_map(|(_, analysis)| analysis.expressions.values())
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("Box(7).wrap"))
        .expect("exact covering impl call");
    let specialization = expression.call_specialization.as_ref().expect("canonical call specialization");
    let [(parameter, ty)] = &specialization.bindings[..] else {
        panic!("one declaration-owned binding expected: {:?}", specialization.bindings)
    };
    assert!(
        matches!(
            &analyzed.semantic.store.type_parameter(*parameter).owner,
            phalcom_semantic::types::parameter::TypeParameterOwner::Declaration(_)
        ),
        "semantic specialization must not gain an impl-owned execution alias",
    );
    assert_eq!(analyzed.semantic.store.format_type(*ty), "Int");

    let program = ProgramCompiler::compile_analyzed(&analyzed).expect("call environment projection");
    let recipe = program.modules[&program.entry]
        .lowering
        .call_environment_recipes
        .get(&expression.range)
        .expect("executable covering impl recipe");
    assert_eq!(recipe.bindings.len(), 2, "callee must see both declaration and source-body coordinates");
    assert_ne!(recipe.bindings[0].0.owner, recipe.bindings[1].0.owner);
    assert_eq!(recipe.bindings[0].1, recipe.bindings[1].1);
}
