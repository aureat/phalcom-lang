//! Source-owned complex arithmetic and principal-branch numerical regressions.

use phalcom_core::heap::ObjRef;
use phalcom_core::modules::compile::{CompiledProgram, EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::fs;
use tempfile::TempDir;

#[path = "../../reference/math_complex_reference.rs"]
mod reference;

fn compile(source: &str) -> (TempDir, CompiledProgram) {
    let temp = TempDir::new().expect("temporary package");
    fs::write(temp.path().join("package.ph"), "expose .main\n").expect("manifest");
    fs::write(temp.path().join("main.ph"), source).expect("source");
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Package(temp.path().to_path_buf())).expect("complex package compiles");
    (temp, program)
}

fn run(source: &str) -> (VM, ObjRef) {
    let (_temp, program) = compile(source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("complex package executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    (vm, module)
}

fn global(vm: &VM, module: ObjRef, name: &str) -> Value {
    vm.heap
        .module(module)
        .get(vm.interner.find(name).expect("binding symbol"))
        .expect("binding value")
}

fn assert_true(vm: &VM, module: ObjRef, names: &[&str]) {
    for name in names {
        assert_eq!(global(vm, module, name), Value::bool(true), "{name}");
    }
}

#[test]
fn complex_int_arithmetic_preserves_exact_components_and_generic_identity() {
    let (mut vm, module) = run(r#"
from universe.numbers import Complex
let z = Complex.new(3, 4)
let direct = Complex(3, 4)
let rectangular = z.real == 3 and z.imag == 4
let constructed = direct === z
let sum = z + Complex(1, -2)
let difference = z - Complex(1, -2)
let product = z * Complex(1, -2)
let addition = sum === Complex(4, 2)
let subtraction = difference === Complex(2, 6)
let multiplication = product === Complex(11, -2)
let negation = (-z) === Complex(-3, -4)
let conjugation = z.conjugate === Complex(3, -4)
let squared = z.normSquared == 25
let huge = Complex(10 ** 100, 10 ** 100) * Complex(2, -2)
let exact = huge.real == 4 * (10 ** 100) and huge.imag == 0
let separated = not (Complex(3, 4) === Complex(3.0, 4.0))
"#);
    vm.force_gc();
    assert_true(
        &vm,
        module,
        &[
            "rectangular",
            "constructed",
            "addition",
            "subtraction",
            "multiplication",
            "negation",
            "conjugation",
            "squared",
            "exact",
            "separated",
        ],
    );
}

#[test]
fn complex_third_party_components_use_selected_operation_capabilities() {
    let (vm, module) = run(r#"
from universe.numbers import Complex, Addable, Subtractable, Multipliable, Negatable
data Scalar(_ value: Int)
impl Scalar {
  +(_ other: Scalar) -> Scalar { Scalar(value + other.value) }
  -(_ other: Scalar) -> Scalar { Scalar(value - other.value) }
  *(_ other: Scalar) -> Scalar { Scalar(value * other.value) }
  - -> Scalar { Scalar(-value) }
}
impl Addable for Scalar {}
impl Subtractable for Scalar {}
impl Multipliable for Scalar {}
impl Negatable for Scalar {}
let z = Complex(Scalar(3), Scalar(4))
let square = z * z
let exact = square.real.value == -7 and square.imag.value == 24
let conjugated = z.conjugate.imag.value == -4
let sum = z + z
let added = sum.real.value == 6 and sum.imag.value == 8
"#);
    assert_true(&vm, module, &["exact", "conjugated", "added"]);
}

#[test]
fn complex_float_equality_keeps_nan_and_signed_zero_semantics() {
    let (vm, module) = run(r#"
from universe.numbers import Complex
let a = Complex(0.0, 1.0)
let b = Complex(-0.0, 1.0)
let numeric = a == b
let exact = not (a === b)
let hashes = a.hash == b.hash
let nan = Complex(0.0 / 0.0, 1.0)
let unequal = not (nan == nan)
let component = b.real === -0.0
"#);
    assert_true(&vm, module, &["numeric", "exact", "hashes", "unequal", "component"]);
}

#[test]
fn complex_unsupported_operations_have_no_proven_capability_and_fail_at_runtime() {
    for expression in [
        "Complex(true, false) + Complex(false, true)",
        "Complex(1, 2) / Complex(3, 4)",
        "Complex(1, 2).compare(Complex(3, 4))",
    ] {
        let temp = TempDir::new().expect("temporary negative package");
        fs::write(temp.path().join("package.ph"), "expose .main\n").unwrap();
        fs::write(
            temp.path().join("main.ph"),
            format!("from universe.numbers import Complex\nlet result = {expression}\n"),
        )
        .unwrap();
        let program = ProgramCompiler::compile_entry_selection(EntrySelection::Package(temp.path().to_path_buf()))
            .expect("gradual missing sends compile without fabricated applicability");
        let lowering = &program.modules[&program.entry].lowering;
        assert!(lowering.conditional_invocations.is_empty(), "{expression}: unavailable conditional capability");
        assert!(lowering.trait_invocations.is_empty(), "{expression}: unavailable trait capability");
        assert!(lowering.trait_requirement_invocations.is_empty(), "{expression}: unavailable requirement slot");
        let mut vm = VM::new();
        let error = vm.run_compiled(&program).expect_err("unavailable operations fail through ordinary dispatch");
        let diagnostic = format!("{error:?}");
        let expected = if expression.contains(".compare") {
            "does not understand"
        } else {
            "UnsupportedOperation"
        };
        assert!(
            diagnostic.contains(expected),
            "{expression}: expected ordinary unavailable-operation failure: {diagnostic}"
        );
    }
}

fn assert_component(actual: f64, expected: f64, context: &str) {
    if expected.is_nan() {
        assert!(actual.is_nan(), "{context}: expected NaN, got {actual:?}");
    } else if expected == 0.0 || expected.is_infinite() {
        assert_eq!(actual.to_bits(), expected.to_bits(), "{context}");
    } else {
        let tolerance = expected.abs() * 8.0e-15 + f64::from_bits(4);
        assert!((actual - expected).abs() <= tolerance, "{context}: expected {expected:?}, got {actual:?}");
    }
}

fn float_source(value: f64) -> String {
    if value.is_nan() {
        "(0.0 / 0.0)".into()
    } else if value == f64::INFINITY {
        "(1.0 / 0.0)".into()
    } else if value == f64::NEG_INFINITY {
        "(-1.0 / 0.0)".into()
    } else {
        format!("({value:?})")
    }
}

fn assert_complex(vm: &VM, module: ObjRef, name: &str, expected: (f64, f64)) {
    let value = global(vm, module, name);
    for (slot, expected) in [expected.0, expected.1].into_iter().enumerate() {
        let actual = vm
            .get_data_component(value, slot as u32)
            .expect("complex component")
            .as_float()
            .expect("Float component");
        assert_component(actual, expected, &format!("{name}[{slot}]"));
    }
}

#[test]
fn complex_float_division_scales_large_subnormal_and_unbalanced_operands() {
    let (mut vm, module) = run(r#"
from universe.numbers import Complex
let ordinary = Complex(3.0, 4.0) / Complex(1.0, -2.0)
let huge = Complex(1.0e308, 1.0e308) / Complex(1.0e308, 1.0e308)
let tiny = Complex(5.0e-324, 5.0e-324) / Complex(5.0e-324, 5.0e-324)
let unbalanced = Complex(1.0e308, 1.0e-308) / Complex(1.0e-308, 0.0)
let mixed = Complex(1.0e308, 1.0e-308) / Complex(0.0, 1.0e-308)
let zero = Complex(0.0, 0.0) / Complex(0.0, 0.0)
let pole = Complex(2.0, -3.0) / Complex(-0.0, 0.0)
let infinity = Complex(2.0, -3.0) / Complex(1.0 / 0.0, 0.0)
let exact = ordinary === Complex(-1.0, 2.0)
"#);
    vm.force_gc();
    for (name, expected) in [
        ("ordinary", (-1.0, 2.0)),
        ("huge", (1.0, 0.0)),
        ("tiny", (1.0, 0.0)),
        ("unbalanced", (f64::INFINITY, 1.0)),
        ("mixed", (1.0, f64::NEG_INFINITY)),
        ("zero", (f64::NAN, f64::NAN)),
        ("pole", (f64::NEG_INFINITY, f64::INFINITY)),
        ("infinity", (0.0, -0.0)),
    ] {
        assert_complex(&vm, module, name, expected);
    }
    assert_true(&vm, module, &["exact"]);
}

#[test]
fn complex_float_magnitude_phase_polar_and_explicit_conversion_preserve_signs() {
    let (mut vm, module) = run(r#"
from universe.numbers import Complex
let z = Complex(3.0, 4.0)
let magnitude = z.abs
let angle = z.arg
let huge = Complex(1.0e308, 1.0e308).abs
let positive_side = Complex(-1.0, 0.0).arg
let negative_side = Complex(-1.0, -0.0).arg
let polar = Complex.fromPolar(radius: z.abs, angle: z.arg)
let negative_zero = Complex.fromPolar(radius: 2.0, angle: -0.0)
let converted = Complex(3, -4).toFloat
let explicit = converted === Complex(3.0, -4.0)
let finite = z.isFinite
let nan = Complex(0.0 / 0.0, 1.0).isNaN
let infinite = Complex(0.0 / 0.0, 1.0 / 0.0).isInfinite
"#);
    vm.force_gc();
    for (name, expected) in [
        ("magnitude", 5.0),
        ("angle", 0.9272952180016122),
        ("huge", 1.4142135623730951e308),
        ("positive_side", std::f64::consts::PI),
        ("negative_side", -std::f64::consts::PI),
    ] {
        assert_component(global(&vm, module, name).as_float().unwrap(), expected, name);
    }
    assert_complex(&vm, module, "polar", (3.0, 4.0));
    assert_complex(&vm, module, "negative_zero", (2.0, -0.0));
    assert_true(&vm, module, &["explicit", "finite", "nan", "infinite"]);
}

#[test]
fn complex_elementary_families_match_independent_finite_references() {
    let mut source = String::from("from universe.numbers import Complex\nimport universe.math.complex as math\nlet z = Complex(0.75, -1.25)\n");
    for (name, _, _) in reference::ELEMENTARY {
        source.push_str(&format!("let result_{name} = math.{name}(z)\n"));
    }
    source.push_str("let root = math.sqrt\nlet escaped = root(Complex(3.0, 4.0))\n");
    let (mut vm, module) = run(&source);
    vm.force_gc();
    for &(name, real, imag) in reference::ELEMENTARY {
        assert_complex(&vm, module, &format!("result_{name}"), (real, imag));
    }
    assert_complex(&vm, module, "escaped", (2.0, 1.0));
}

#[test]
fn complex_elementary_scaled_and_branch_edge_discriminators() {
    let (vm, module) = run(r#"
from universe.numbers import Complex
import universe.math.complex as math
let root_upper = math.sqrt(Complex(-1.0, 0.0))
let root_lower = math.sqrt(Complex(-1.0, -0.0))
let log_upper = math.log(Complex(-1.0, 0.0))
let log_lower = math.log(Complex(-1.0, -0.0))
let root_large = math.sqrt(Complex(1.0e308, 1.0e308))
let root_tiny = math.sqrt(Complex(5.0e-324, 5.0e-324))
let log_large = math.log(Complex(1.0e308, 1.0e308))
let log_zero = math.log(Complex(-0.0, -0.0))
let near_zero = math.log1p(Complex(1.0e-20, 2.0e-20))
let log1p_lower = math.log1p(Complex(-2.0, -0.0))
let acosh_near = math.acosh(Complex(1.0000000000000002, 1.0e-20))
let atanh_pole = math.atanh(Complex(1.0, -0.0))
let asinh_cut = math.asinh(Complex(-0.0, 2.0))
let sqrt_infinite = math.sqrt(Complex(0.0 / 0.0, -1.0 / 0.0))
let exp_infinite = math.exp(Complex(-1.0 / 0.0, -0.0))
let tanh_infinite = math.tanh(Complex(1.0 / 0.0, 0.75))
"#);
    let pi = std::f64::consts::PI;
    for (name, expected) in [
        ("root_upper", (0.0, 1.0)),
        ("root_lower", (0.0, -1.0)),
        ("log_upper", (0.0, pi)),
        ("log_lower", (0.0, -pi)),
        ("root_large", (1.09868411346781e154, 4.5508986056222734e153)),
        ("root_tiny", (2.4421097261308304e-162, 1.0115549693666347e-162)),
        ("log_large", (709.542782232446, pi / 4.0)),
        ("log_zero", (f64::NEG_INFINITY, -pi)),
        ("near_zero", (1.0e-20, 2.0e-20)),
        ("log1p_lower", (0.0, -pi)),
        ("acosh_near", (2.1073424260789764e-8, 4.745313280009497e-13)),
        ("atanh_pole", (f64::INFINITY, -0.0)),
        ("asinh_cut", (-1.3169578969248168, pi / 2.0)),
        ("sqrt_infinite", (f64::INFINITY, f64::NEG_INFINITY)),
        ("exp_infinite", (0.0, -0.0)),
        ("tanh_infinite", (1.0, 0.0)),
    ] {
        assert_complex(&vm, module, name, expected);
    }
}

#[test]
fn complex_scaled_elementary_functions_match_high_precision_references() {
    let mut source = String::from("from universe.numbers import Complex\nimport universe.math.complex as math\n");
    for (index, &(name, x, y, _, _)) in reference::SCALED.iter().enumerate() {
        source.push_str(&format!("let result{index} = math.{name}(Complex({}, {}))\n", float_source(x), float_source(y)));
    }
    let (vm, module) = run(&source);
    for (index, &(_, _, _, real, imag)) in reference::SCALED.iter().enumerate() {
        assert_complex(&vm, module, &format!("result{index}"), (real, imag));
    }
}

#[test]
fn complex_elementary_finite_matrix_matches_independent_cut_and_small_references() {
    let mut source = String::from("from universe.numbers import Complex\nimport universe.math.complex as math\n");
    for (index, &(name, x, y, _, _)) in reference::FINITE_MATRIX.iter().enumerate() {
        source.push_str(&format!("let result{index} = math.{name}(Complex({}, {}))\n", float_source(x), float_source(y)));
    }
    let (vm, module) = run(&source);
    for (index, &(_, _, _, real, imag)) in reference::FINITE_MATRIX.iter().enumerate() {
        assert_complex(&vm, module, &format!("result{index}"), (real, imag));
    }
}

#[test]
fn complex_nonfinite_and_signed_cut_cases_follow_the_backend_contract() {
    let inf = f64::INFINITY;
    let nan = f64::NAN;
    let pi = std::f64::consts::PI;
    let cases = [
        ("sqrt", -0.0, -0.0, 0.0, -0.0),
        ("exp", -0.0, -0.0, 1.0, -0.0),
        ("log1p", -0.0, -0.0, -0.0, -0.0),
        ("sin", -0.0, -0.0, -0.0, -0.0),
        ("cos", -0.0, -0.0, 1.0, -0.0),
        ("tan", -0.0, -0.0, -0.0, -0.0),
        ("asin", -0.0, -0.0, -0.0, -0.0),
        ("acos", -0.0, -0.0, pi / 2.0, 0.0),
        ("atan", -0.0, -0.0, -0.0, -0.0),
        ("sinh", -0.0, -0.0, -0.0, -0.0),
        ("cosh", -0.0, -0.0, 1.0, 0.0),
        ("tanh", -0.0, -0.0, -0.0, -0.0),
        ("asinh", -0.0, -0.0, -0.0, -0.0),
        ("acosh", -0.0, -0.0, 0.0, -pi / 2.0),
        ("atanh", -0.0, -0.0, -0.0, -0.0),
        ("sqrt", -inf, nan, nan, inf),
        ("sqrt", inf, -0.0, inf, -0.0),
        ("exp", nan, -0.0, nan, -0.0),
        ("log", -inf, -0.0, inf, -pi),
        ("log1p", -inf, -0.0, inf, -pi),
        ("log1p", nan, 0.0, nan, nan),
        ("sin", nan, -0.0, nan, -0.0),
        ("sin", -0.0, nan, -0.0, nan),
        ("cos", nan, -0.0, nan, 0.0),
        ("tan", -0.0, nan, -0.0, nan),
        ("asin", -0.0, nan, -0.0, nan),
        ("acos", -0.0, nan, pi / 2.0, nan),
        ("atan", -inf, -0.0, -pi / 2.0, -0.0),
        ("sinh", nan, -0.0, nan, -0.0),
        ("sinh", -0.0, nan, 0.0, nan),
        ("cosh", -inf, -0.0, inf, 0.0),
        ("tanh", -0.0, nan, -0.0, nan),
        ("asinh", -inf, -inf, -inf, -pi / 4.0),
        ("asinh", nan, -0.0, nan, -0.0),
        ("acosh", -0.0, nan, nan, pi / 2.0),
        ("atanh", -0.0, nan, -0.0, nan),
        ("asin", 2.0, -0.0, pi / 2.0, -1.3169578969248168),
        ("acos", 2.0, -0.0, 0.0, 1.3169578969248168),
        ("atan", -0.0, 2.0, -pi / 2.0, 0.5493061443340549),
        ("asinh", -0.0, -2.0, -1.3169578969248168, -pi / 2.0),
        ("acosh", -2.0, -0.0, 1.3169578969248168, -pi),
        ("atanh", 2.0, -0.0, 0.5493061443340549, -pi / 2.0),
        ("asin", -2.0, 0.0, -pi / 2.0, 1.3169578969248168),
        ("acos", -2.0, 0.0, pi, -1.3169578969248168),
        ("atan", 0.0, -2.0, pi / 2.0, -0.5493061443340549),
        ("asinh", 0.0, -2.0, 1.3169578969248168, -pi / 2.0),
        ("acosh", -2.0, 0.0, 1.3169578969248168, pi),
        ("atanh", -2.0, 0.0, -0.5493061443340549, pi / 2.0),
        ("asin", 1.0, -0.0, pi / 2.0, -0.0),
        ("acos", 1.0, -0.0, 0.0, 0.0),
        ("acosh", 1.0, -0.0, 0.0, -0.0),
        ("asinh", -0.0, 1.0, -0.0, pi / 2.0),
        ("atanh", -1.0, 0.0, -inf, 0.0),
        ("atan", -0.0, 1.0, -0.0, inf),
        ("log1p", -1.0, -0.0, -inf, -0.0),
    ];
    let mut source = String::from("from universe.numbers import Complex\nimport universe.math.complex as math\n");
    for (index, &(name, x, y, _, _)) in cases.iter().enumerate() {
        source.push_str(&format!("let result{index} = math.{name}(Complex({}, {}))\n", float_source(x), float_source(y)));
    }
    let (vm, module) = run(&source);
    let mut failures = Vec::new();
    for (index, &(name, _, _, real, imag)) in cases.iter().enumerate() {
        let value = global(&vm, module, &format!("result{index}"));
        for (slot, expected) in [real, imag].into_iter().enumerate() {
            let actual = vm.get_data_component(value, slot as u32).unwrap().as_float().unwrap();
            let matches = if expected.is_nan() {
                actual.is_nan()
            } else if expected == 0.0 || expected.is_infinite() {
                actual.to_bits() == expected.to_bits()
            } else {
                (actual - expected).abs() <= expected.abs() * 8e-15
            };
            if !matches {
                failures.push(format!("case {index} {name}[{slot}]: {actual:?}, expected {expected:?}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn complex_integer_and_principal_powers_preserve_exactness_and_zero_endpoints() {
    let (mut vm, module) = run(r#"
from universe.numbers import Complex
import universe.math.complex as math
let exact = (Complex(1, 1) ** 10) === Complex(0, 32)
let zero_exponent = (Complex(0, 0) ** 0) === Complex(1, 0)
let arbitrary_exponent = (Complex(0, 1) ** (2 ** 70)) === Complex(1, 0)
let huge_actual = Complex(10 ** 50, 0) ** 2
let huge_expected = Complex(10 ** 100, 0)
let huge = huge_actual === huge_expected
let huge_numeric = huge_actual == huge_expected
let huge_real_same = huge_actual.real === huge_expected.real
let mut negative_rejected = false
try { Complex(1, 1) ** -1 } catch e { negative_rejected = e.is(ArgumentError) }
let inverse = Complex(1.0, 1.0) ** -2
let captured = math.powInt
let captured_result = captured(Complex(1.0, 1.0), 10)
let nan_identity = captured(Complex(0.0 / 0.0, 1.0 / 0.0), 0)
let positive = Complex(1.0, 1.0) ** 3
let captured_pow = math.pow
let principal = math.pow(Complex(0.75, -1.25), Complex(0.5, 0.25))
let captured_principal = captured_pow(Complex(0.75, -1.25), Complex(0.5, 0.25))
let upper = math.pow(Complex(-1.0, 0.0), Complex(0.5, 0.0))
let lower = math.pow(Complex(-1.0, -0.0), Complex(0.5, 0.0))
let zero_zero = math.pow(Complex(0.0, 0.0), Complex(0.0, 0.0))
let zero_positive = math.pow(Complex(0.0, 0.0), Complex(2.0, 0.0))
let zero_negative = math.pow(Complex(0.0, 0.0), Complex(-2.0, 0.0))
let zero_complex = math.pow(Complex(0.0, 0.0), Complex(1.0, 1.0))
let zero_nan = math.pow(Complex(0.0, 0.0), Complex(0.0 / 0.0, 0.0))
"#);
    vm.force_gc();
    assert_eq!(
        global(&vm, module, "huge_numeric"),
        Value::bool(true),
        "huge integer arithmetic is numerically exact"
    );
    assert_eq!(
        global(&vm, module, "huge_real_same"),
        Value::bool(true),
        "independent LargeInt components retain semantic exactness"
    );
    assert_true(&vm, module, &["exact", "zero_exponent", "arbitrary_exponent", "huge", "negative_rejected"]);
    assert_complex(&vm, module, "inverse", (0.0, -0.5));
    assert_complex(&vm, module, "positive", (-2.0, 2.0));
    assert_complex(&vm, module, "nan_identity", (1.0, 0.0));
    assert_complex(&vm, module, "captured_result", (0.0, 32.0));
    assert_complex(&vm, module, "principal", (1.4257283425539067, -0.638345370227498));
    assert_complex(&vm, module, "captured_principal", (1.4257283425539067, -0.638345370227498));
    for (name, expected) in [
        ("upper", (std::f64::consts::FRAC_PI_2.cos(), 1.0)),
        ("lower", (std::f64::consts::FRAC_PI_2.cos(), -1.0)),
        ("zero_zero", (1.0, 0.0)),
        ("zero_positive", (0.0, 0.0)),
        ("zero_negative", (f64::INFINITY, 0.0)),
        ("zero_complex", (f64::NAN, f64::NAN)),
        ("zero_nan", (f64::NAN, f64::NAN)),
    ] {
        assert_complex(&vm, module, name, expected);
    }
}
