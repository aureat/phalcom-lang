//! Runtime probes for contextual algebraic basis values and native suffix literals.

use phalcom_core::heap::ObjRef;
use phalcom_core::modules::compile::{CompiledProgram, EntrySelection, ProgramCompiler};
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

#[test]
fn native_imaginary_literals_and_basis_products_are_runtime_values() {
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
    assert_eq!(global(&vm, module, "projection"), Value::bool(true));
}
