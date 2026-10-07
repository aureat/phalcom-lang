//! Focused correctness probes for the source-owned `universe.math` surface.

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, ToPrimitive, Zero};
use phalcom_core::error::{PhError, RuntimeError};
use phalcom_core::modules::ModuleFailure;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::native::PrimitiveEntry;
use phalcom_core::native::registry::PRIMITIVES;
use phalcom_core::value::{NumericPolicy, Value};
use phalcom_core::vm::VM;
use phalcom_native_meta::{EffectSpec, NativeDispatch};
use std::fs;
use tempfile::TempDir;

fn compile_package(source: &str) -> (TempDir, phalcom_core::modules::compile::CompiledProgram) {
    let temp = TempDir::new().expect("temporary package directory");
    fs::write(temp.path().join("package.ph"), "expose .main\n").expect("package manifest");
    fs::write(temp.path().join("main.ph"), source).expect("package source");
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Package(temp.path().to_path_buf())).expect("package should compile");
    (temp, program)
}

fn run_expression(expression: &str) -> (VM, Value) {
    let source = format!("import universe.math as math\nlet result = {expression}\n");
    let (_temp, program) = compile_package(&source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("math expression should execute");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let result = vm
        .heap
        .module(module)
        .get(vm.interner.find("result").expect("result symbol"))
        .expect("result global");
    (vm, result)
}

fn assert_float_result(expression: &str, expected: f64) {
    let (_vm, result) = run_expression(expression);
    let actual = result.as_float().expect("math result must be Float");
    if expected.is_nan() {
        assert!(actual.is_nan(), "{expression}: expected NaN, got {actual:?}");
    } else {
        assert_eq!(actual.to_bits(), expected.to_bits(), "{expression}");
    }
}

// Compare transport to this run's backend, not to a portable transcendental
// last-bit oracle. NaN payload/sign bits are deliberately excluded.
fn assert_float_cases(cases: &[(String, f64)]) {
    let mut source = String::from("import universe.math as math\n");
    for (index, (expression, _)) in cases.iter().enumerate() {
        source.push_str(&format!("let result{index} = {expression}\n"));
    }
    let (_temp, program) = compile_package(&source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("Float matrix executes");
    let module = vm.module_registry.get(&program.entry).unwrap().object;
    for (index, (expression, expected)) in cases.iter().enumerate() {
        let actual = read_global(&vm, module, &format!("result{index}")).as_float().expect("Float result");
        if expected.is_nan() {
            assert!(actual.is_nan(), "{expression}: expected NaN, got {actual:?}");
        } else {
            assert_eq!(actual.to_bits(), expected.to_bits(), "{expression}");
        }
    }
}

const UNARY_BACKENDS: [(&str, fn(f64) -> f64); 21] = [
    ("sqrt", f64::sqrt),
    ("cbrt", f64::cbrt),
    ("exp", f64::exp),
    ("exp2", f64::exp2),
    ("expm1", f64::exp_m1),
    ("ln", f64::ln),
    ("ln1p", f64::ln_1p),
    ("log2", f64::log2),
    ("log10", f64::log10),
    ("sin", f64::sin),
    ("cos", f64::cos),
    ("tan", f64::tan),
    ("asin", f64::asin),
    ("acos", f64::acos),
    ("atan", f64::atan),
    ("sinh", f64::sinh),
    ("cosh", f64::cosh),
    ("tanh", f64::tanh),
    ("asinh", f64::asinh),
    ("acosh", f64::acosh),
    ("atanh", f64::atanh),
];

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

#[test]
fn every_float_family_matches_backend_across_ieee_and_accuracy_tiers() {
    let inputs = [
        0.0,
        -0.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        f64::from_bits(1),
        -f64::from_bits(1),
        f64::MIN_POSITIVE,
        f64::MAX,
        -1.0,
        1.0,
        f64::from_bits(1.0f64.to_bits() + 1),
        f64::from_bits((-1.0f64).to_bits() + 1),
        0.25,
        -0.25,
        1.25,
        2.25,
        1e-20,
        -1e-20,
    ];
    let mut cases = Vec::new();
    for (name, operation) in UNARY_BACKENDS {
        for input in inputs {
            let input = std::hint::black_box(input);
            cases.push((format!("math.{name}({})", float_source(input)), operation(input)));
        }
    }
    // All quadrants, signed-zero combinations, infinities and NaNs; the
    // asymmetric finite pair discriminates atan2's y/x ordering.
    let binary_inputs = [0.0, -0.0, 1.0, -2.0, f64::INFINITY, f64::NEG_INFINITY, f64::NAN, f64::from_bits(1), f64::MAX];
    for x in binary_inputs {
        for y in binary_inputs {
            cases.push((format!("math.atan2({}, {})", float_source(x), float_source(y)), x.atan2(y)));
            cases.push((format!("math.hypot({}, {})", float_source(x), float_source(y)), x.hypot(y)));
        }
    }
    for value in [0.0, -0.0, -1.0, 1.0, 8.25, f64::INFINITY, f64::NAN, f64::from_bits(1)] {
        for base in [0.0, -2.0, 1.0, 2.5, 0.5, f64::INFINITY, f64::NAN] {
            cases.push((format!("math.log({}, base: {})", float_source(value), float_source(base)), value.log(base)));
        }
    }
    assert_float_cases(&cases);
}

fn read_global(vm: &VM, module: phalcom_core::heap::ObjRef, name: &str) -> Value {
    vm.heap
        .module(module)
        .get(vm.interner.find(name).expect("global symbol"))
        .unwrap_or_else(|| panic!("missing global {name}"))
}

fn as_big_int(vm: &VM, value: Value) -> BigInt {
    if let Some(value) = value.as_int() {
        BigInt::from(value)
    } else {
        let object = value.as_obj().expect("expected Int or LargeInt");
        vm.heap.as_large_int(object).expect("expected LargeInt").clone()
    }
}

fn runtime_error(error: &PhError) -> &RuntimeError {
    match error {
        PhError::Runtime(error) => error,
        PhError::NativeOrigin { source, .. } => runtime_error(source),
        PhError::ModuleInitialization(initialization) => match initialization.failure.as_ref() {
            ModuleFailure::Initializer { cause } => runtime_error(cause),
            other => panic!("unexpected dependency failure: {other:?}"),
        },
        other => panic!("expected runtime error, got {other:?}"),
    }
}

fn assert_numeric_limit(vm: &mut VM, error: &PhError) {
    let RuntimeError::Raise { error, .. } = runtime_error(error) else {
        panic!("expected catchable numeric error")
    };
    let kind = vm.get_or_intern("kind");
    let result = vm.send_dynamic(*error, kind, &[]).expect("numeric error kind");
    assert_eq!(vm.resolve_symbol(result.as_symbol().expect("kind Symbol")), "numericLimit");
}

#[test]
fn integer_math_deterministic_property_matrix() {
    let values = [
        BigInt::from(-84),
        BigInt::from(-30),
        BigInt::from(-1),
        BigInt::zero(),
        BigInt::one(),
        BigInt::from(6),
        BigInt::from(21),
        BigInt::from(84),
        BigInt::from(i64::MIN),
        BigInt::from(i64::MAX),
        (BigInt::one() << 128usize) + BigInt::from(3),
        -((BigInt::one() << 200usize) + BigInt::from(15)),
    ];
    let mut source = String::from("import universe.math as math\n");
    for (index, value) in values.iter().enumerate() {
        source.push_str(&format!("let value{index} = {value}\n"));
    }
    for (a, _) in values.iter().enumerate() {
        for (b, _) in values.iter().enumerate() {
            source.push_str(&format!(
                "let gcd{a}x{b} = math.gcd(value{a}, value{b})\nlet lcm{a}x{b} = math.lcm(value{a}, value{b})\n"
            ));
        }
    }
    let (_temp, program) = compile_package(&source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("integer matrix executes");
    let module = vm.module_registry.get(&program.entry).unwrap().object;
    for (i, a) in values.iter().enumerate() {
        for (j, b) in values.iter().enumerate() {
            let g = as_big_int(&vm, read_global(&vm, module, &format!("gcd{i}x{j}")));
            let l = as_big_int(&vm, read_global(&vm, module, &format!("lcm{i}x{j}")));
            assert_eq!(g, a.gcd(b), "gcd({a}, {b})");
            assert_eq!(l, a.lcm(b), "lcm({a}, {b})");
            assert!(!g.is_negative() && !l.is_negative());
            assert_eq!(g, as_big_int(&vm, read_global(&vm, module, &format!("gcd{j}x{i}"))));
            if !g.is_zero() {
                assert!((a % &g).is_zero() && (b % &g).is_zero());
            }
            assert_eq!(&g * &l, (a * b).abs());
        }
    }
}

fn run_integer_case(a: &BigInt, b: &BigInt, n: &BigInt) -> (VM, phalcom_core::heap::ObjRef) {
    let source = format!(
        "import universe.math as math\nlet a = {a}\nlet b = {b}\nlet n = {n}\nlet gcd = math.gcd(a, b)\nlet swapped = math.gcd(b, a)\nlet lcm = math.lcm(a, b)\nlet root = math.isqrt(n)\n"
    );
    let (_temp, program) = compile_package(&source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("integer math expression should execute");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    (vm, module)
}

#[test]
fn math_constants_have_the_documented_binary64_bits() {
    for (name, bits) in [
        ("pi", (0x4009_21fb_5444_2d18, std::f64::consts::PI.to_bits())),
        ("tau", (0x4019_21fb_5444_2d18, std::f64::consts::TAU.to_bits())),
        ("e", (0x4005_bf0a_8b14_5769, std::f64::consts::E.to_bits())),
        ("ln2", (0x3fe6_2e42_fefa_39ef, std::f64::consts::LN_2.to_bits())),
        ("ln10", (0x4002_6bb1_bbb5_5516, std::f64::consts::LN_10.to_bits())),
        ("sqrt2", (0x3ff6_a09e_667f_3bcd, std::f64::consts::SQRT_2.to_bits())),
    ] {
        let (_vm, result) = run_expression(&format!("math.{name}"));
        let actual = result.as_float().expect("constant must be Float").to_bits();
        assert_eq!(actual, bits.0, "{name} literal bit oracle");
        assert_eq!(actual, bits.1, "{name} standard-library bit oracle");
    }
}

#[test]
fn every_float_family_matches_its_declared_host_operation() {
    let unary = [
        ("sqrt(2.25)", 2.25f64.sqrt()),
        ("cbrt(-8.0)", (-8.0f64).cbrt()),
        ("exp(0.25)", 0.25f64.exp()),
        ("exp2(3.25)", 3.25f64.exp2()),
        ("expm1(0.25)", 0.25f64.exp_m1()),
        ("ln(1.25)", 1.25f64.ln()),
        ("ln1p(0.25)", 0.25f64.ln_1p()),
        ("log2(8.25)", 8.25f64.log2()),
        ("log10(100.25)", 100.25f64.log10()),
        ("sin(0.25)", 0.25f64.sin()),
        ("cos(0.25)", 0.25f64.cos()),
        ("tan(0.25)", 0.25f64.tan()),
        ("asin(0.25)", 0.25f64.asin()),
        ("acos(0.25)", 0.25f64.acos()),
        ("atan(0.25)", 0.25f64.atan()),
        ("sinh(0.25)", 0.25f64.sinh()),
        ("cosh(0.25)", 0.25f64.cosh()),
        ("tanh(0.25)", 0.25f64.tanh()),
        ("asinh(0.25)", 0.25f64.asinh()),
        ("acosh(1.25)", 1.25f64.acosh()),
        ("atanh(0.25)", 0.25f64.atanh()),
    ];
    for (expression, expected) in unary {
        assert_float_result(&format!("math.{expression}"), expected);
    }
    assert_float_result("math.log(8.25, base: 2.5)", 8.25f64.log(2.5));
    assert_float_result("math.atan2(1.0, -2.0)", 1.0f64.atan2(-2.0));
    assert_float_result("math.hypot(3.0, 4.0)", 3.0f64.hypot(4.0));
}

#[test]
fn float_families_preserve_ieee_edges_and_function_domains() {
    for (expression, expected) in [
        ("math.cbrt(-0.0)", (-0.0f64).cbrt()),
        ("math.exp(1.0 / 0.0)", f64::INFINITY),
        ("math.exp2(-1.0 / 0.0)", 0.0),
        ("math.expm1(-1.0 / 0.0)", -1.0),
        ("math.sqrt(-1.0)", f64::NAN),
        ("math.sqrt(1.0 / 0.0)", f64::INFINITY),
        ("math.ln(-1.0)", f64::NAN),
        ("math.ln(0.0)", f64::NEG_INFINITY),
        ("math.ln(-0.0)", f64::NEG_INFINITY),
        ("math.ln(1.0 / 0.0)", f64::INFINITY),
        ("math.ln1p(-1.0)", f64::NEG_INFINITY),
        ("math.log(-1.0, base: 2.0)", f64::NAN),
        ("math.log(0.0, base: 2.0)", f64::NEG_INFINITY),
        ("math.log(1.0 / 0.0, base: 2.0)", f64::INFINITY),
        ("math.log(1.0, base: 1.0)", f64::NAN),
        ("math.log2(1.0 / 0.0)", f64::INFINITY),
        ("math.log10(1.0 / 0.0)", f64::INFINITY),
        ("math.sin(1.0 / 0.0)", f64::NAN),
        ("math.cos(1.0 / 0.0)", f64::NAN),
        ("math.tan(1.0 / 0.0)", f64::NAN),
        ("math.asin(1.01)", f64::NAN),
        ("math.acos(-1.0)", std::f64::consts::PI),
        ("math.atan(1.0 / 0.0)", std::f64::consts::FRAC_PI_2),
        ("math.sinh(1.0 / 0.0)", f64::INFINITY),
        ("math.cosh(-1.0 / 0.0)", f64::INFINITY),
        ("math.tanh(-1.0 / 0.0)", -1.0),
        ("math.asinh(-1.0 / 0.0)", f64::NEG_INFINITY),
        ("math.acosh(0.5)", f64::NAN),
        ("math.atanh(1.1)", f64::NAN),
        ("math.hypot(1.0 / 0.0, 3.0)", f64::INFINITY),
        ("math.sqrt(5e-324)", (5e-324f64).sqrt()),
    ] {
        assert_float_result(expression, expected);
    }

    for (expression, expected) in [
        ("math.sqrt(-0.0)", (-0.0f64).sqrt()),
        ("math.sqrt(0.0)", 0.0f64.sqrt()),
        ("math.atan2(0.0, 0.0)", 0.0f64.atan2(0.0)),
        ("math.atan2(0.0, -0.0)", 0.0f64.atan2(-0.0)),
        ("math.atan2(-0.0, 0.0)", (-0.0f64).atan2(0.0)),
        ("math.atan2(-0.0, -0.0)", (-0.0f64).atan2(-0.0)),
    ] {
        assert_float_result(expression, expected);
    }
    assert_float_result("math.sin(0.0 / 0.0)", f64::NAN);
}

#[test]
fn float_native_descriptors_are_pure() {
    let descriptors = PRIMITIVES
        .iter()
        .filter(|descriptor| descriptor.surface.key.owner == phalcom_native_meta::UniverseKey::FloatMathKernel)
        .collect::<Vec<_>>();
    assert_eq!(descriptors.len(), 24, "all declared Float kernels must be registered");
    let mut vm = VM::new_native();
    let before_handles = vm.heap.iter_handles_for_test();
    let before_policy = vm.numeric_policy;
    for descriptor in descriptors {
        assert_eq!(descriptor.surface.key.side, NativeDispatch::Class);
        assert_eq!(descriptor.surface.effects, EffectSpec::Pure, "{}", descriptor.surface.key.selector);
        let PrimitiveEntry::Value(operation) = descriptor.entry else {
            panic!("Float kernel must use ordinary value ABI")
        };
        let arity = if descriptor.surface.key.selector == "log(_,base)" || descriptor.surface.key.selector.contains("_,_") {
            2
        } else {
            1
        };
        let args = [Value::float(1.25), Value::float(2.5)];
        let first = operation(&mut vm, &Value::unit(), &args[..arity]).unwrap();
        let second = operation(&mut vm, &Value::unit(), &args[..arity]).unwrap();
        assert!(first.as_float().is_some());
        assert_eq!(first.as_float().unwrap().to_bits(), second.as_float().unwrap().to_bits());
        assert_eq!(vm.heap.iter_handles_for_test(), before_handles, "{} allocates", descriptor.surface.key.selector);
        assert_eq!(vm.numeric_policy, before_policy);
    }
}

#[test]
fn integer_math_satisfies_gcd_lcm_and_isqrt_laws_for_large_values() {
    let cases = [
        (BigInt::zero(), BigInt::zero()),
        (BigInt::zero(), BigInt::one()),
        (BigInt::from(84), BigInt::from(30)),
        (BigInt::from(-84), BigInt::from(30)),
        (BigInt::from(84), BigInt::from(-30)),
        (BigInt::from(-84), BigInt::from(-30)),
        (
            BigInt::parse_bytes(b"123456789012345678901234567890", 10).unwrap(),
            BigInt::parse_bytes(b"987654321098765432109876543210", 10).unwrap(),
        ),
        (BigInt::from(i64::MAX), BigInt::from(i64::MIN)),
    ];
    for (a, b) in cases {
        let n = (&a * &a) + BigInt::from(12345);
        let (vm, module) = run_integer_case(&a, &b, &n);
        let gcd = as_big_int(&vm, read_global(&vm, module, "gcd"));
        let swapped = as_big_int(&vm, read_global(&vm, module, "swapped"));
        let lcm = as_big_int(&vm, read_global(&vm, module, "lcm"));
        let root = as_big_int(&vm, read_global(&vm, module, "root"));

        assert_eq!(gcd, a.gcd(&b));
        assert_eq!(swapped, gcd);
        assert!(!gcd.is_negative());
        if !gcd.is_zero() {
            assert_eq!((&a % &gcd), BigInt::zero());
            assert_eq!((&b % &gcd), BigInt::zero());
        }
        let expected_lcm = a.lcm(&b);
        assert_eq!(lcm, expected_lcm);
        if !a.is_zero() && !b.is_zero() {
            assert_eq!(&gcd * &lcm, (a * b).abs());
        } else {
            assert!(lcm.is_zero());
        }
        assert!(root.sign() != num_bigint::Sign::Minus);
        assert!(&root * &root <= n);
        assert!(n < (&root + BigInt::one()) * (&root + BigInt::one()));
    }
}

#[test]
fn integer_math_covers_isqrt_boundaries_and_large_perfect_squares() {
    for k in [
        BigInt::from(1),
        BigInt::from(2),
        BigInt::from(3),
        BigInt::from(63),
        BigInt::from(64),
        BigInt::from(1_i64 << 30),
        BigInt::from(3_037_000_499_i64),
        BigInt::from(3_037_000_500_i64),
        BigInt::one() << 200,
    ] {
        for delta in [-1, 0, 1] {
            let n = &k * &k + BigInt::from(delta);
            if n.is_negative() {
                continue;
            }
            let (vm, module) = run_integer_case(&BigInt::from(21), &BigInt::from(6), &n);
            let root = as_big_int(&vm, read_global(&vm, module, "root"));
            let expected = if delta < 0 { &k - 1 } else { k.clone() };
            assert_eq!(root, expected, "isqrt(k² {delta:+}) for k={k}");
        }
    }

    for n in [BigInt::from(10u8).pow(100), BigInt::from(10u8).pow(1000)] {
        let (vm, module) = run_integer_case(&BigInt::from(21), &BigInt::from(6), &n);
        let root = as_big_int(&vm, read_global(&vm, module, "root"));
        assert!(&root * &root <= n);
        assert!(n < (&root + BigInt::one()) * (&root + BigInt::one()));
    }

    for n in [BigInt::zero(), BigInt::one(), BigInt::from(2), BigInt::from(3), BigInt::from(4)] {
        let (vm, module) = run_integer_case(&BigInt::zero(), &BigInt::zero(), &n);
        let root = as_big_int(&vm, read_global(&vm, module, "root"));
        let expected = match n.to_u8().unwrap() {
            0 => 0,
            1 => 1,
            2 | 3 => 1,
            4 => 2,
            _ => unreachable!(),
        };
        assert_eq!(root, BigInt::from(expected));
    }
}

#[test]
fn isqrt_rejects_negative_exact_inputs_with_argument_error() {
    let source = "import universe.math as math\nmath.isqrt(-1)\n";
    let (_temp, program) = compile_package(source);
    let mut vm = VM::new();
    let error = vm.run_compiled(&program).expect_err("negative isqrt must raise");
    assert!(format!("{error}").contains("isqrt requires a non-negative Int"), "{error}");
    let RuntimeError::Raise { error, .. } = runtime_error(&error) else {
        panic!("expected ArgumentError raise")
    };
    assert_eq!(vm.heap.class(error.class(&vm)).name, "ArgumentError");
}

#[test]
fn integer_math_respects_numeric_limits_in_intermediate_results() {
    let source = "import universe.math as math\nlet a = (1 << 100) - 1\nlet b = (1 << 100) + 1\nlet result = math.lcm(a, b)\n";
    let (_temp, program) = compile_package(source);
    let mut vm = VM::new();
    vm.numeric_policy = NumericPolicy {
        max_source_numeric_digits: Some(100_000),
        max_text_conversion_digits: Some(100_000),
        max_integer_bits: Some(128),
        max_numeric_allocation_bytes: Some(2_097_152),
    };
    let error = vm.run_compiled(&program).expect_err("lcm intermediate must honor the integer-bit limit");
    assert!(format!("{error}").contains("Numeric limit exceeded"), "{error}");
    assert_numeric_limit(&mut vm, &error);
}

#[test]
fn integer_math_keeps_existing_limits_and_divides_before_multiplying() {
    // Inputs are admitted under the ordinary policy, then the runtime budget
    // is reduced. gcd must use checked ordinary negation and isqrt must use
    // checked ordinary shift, rather than bypassing the numeric owner.
    let (mut vm, gcd) = run_expression("math.gcd");
    let negative = phalcom_core::value::normalize_bigint(-(BigInt::one() << 64usize), &mut vm.heap);
    vm.numeric_policy.max_integer_bits = Some(64);
    let call = vm.get_or_intern("call(_,_)");
    let error = vm
        .send_dynamic(gcd, call, &[negative, Value::int(3)])
        .expect_err("gcd normalization uses ordinary checked negation");
    assert_numeric_limit(&mut vm, &error);
    for (declarations, expression, limit) in [("", "math.isqrt(81)", 4)] {
        let (_temp, program) = compile_package(&format!("import universe.math as math\n{declarations}let result = {expression}\n"));
        let mut vm = VM::new();
        vm.numeric_policy.max_integer_bits = Some(limit);
        let error = vm.run_compiled(&program).expect_err("ordinary intermediate limit must be preserved");
        assert_numeric_limit(&mut vm, &error);
    }
    let (_temp, program) = compile_package("import universe.math as math\nlet a = 1 << 100\nlet result = math.lcm(a, a)\n");
    let mut vm = VM::new();
    vm.numeric_policy.max_integer_bits = Some(128);
    vm.run_compiled(&program)
        .expect("divide-before-multiply avoids an unnecessary 201-bit temporary");
    let module = vm.module_registry.get(&program.entry).unwrap().object;
    assert_eq!(as_big_int(&vm, read_global(&vm, module, "result")), BigInt::one() << 100);
}
