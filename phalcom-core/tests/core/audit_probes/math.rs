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

fn compile_numerics_package(source: &str) -> (TempDir, phalcom_core::modules::compile::CompiledProgram) {
    let temp = TempDir::new().expect("temporary numerics client directory");
    let source_root = temp.path().join("src");
    fs::create_dir(&source_root).expect("numerics client source directory");
    fs::write(source_root.join("package.ph"), "expose .main\n").expect("numerics client package");
    let main = source_root.join("main.ph");
    fs::write(&main, source).expect("numerics client source");
    let numerics = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../packages/numerics");
    fs::write(
        temp.path().join("project.toml"),
        format!(
            "[project]\nname = \"numerics-tests\"\nnamespace = \"numerics_tests\"\nsource = \"src\"\n\n[dependencies]\nnumerics = {{ path = {:?} }}\n",
            numerics.to_str().unwrap()
        ),
    )
    .expect("numerics client manifest");
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Module(main)).expect("numerics client compiles");
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

type UnaryBackend = (&'static str, fn(f64) -> f64);

const UNARY_BACKENDS: [UnaryBackend; 21] = [
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
    {
        let (declarations, expression, limit) = ("", "math.isqrt(81)", 4);
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

#[test]
fn log_sum_exp_preserves_extreme_values_and_small_corrections() {
    assert_float_result("math.logSumExp([])", f64::NEG_INFINITY);
    assert_float_result("math.logSumExp([-1.0 / 0.0, -1.0 / 0.0])", f64::NEG_INFINITY);
    assert_float_result("math.logSumExp([1.0 / 0.0, 0.0 / 0.0])", f64::NAN);
    assert_float_result("math.logSumExp([0.0 / 0.0, 1.0 / 0.0])", f64::NAN);
    assert_float_result("math.logSumExp([1.0 / 0.0, 2.0])", f64::INFINITY);
    assert_float_result("math.logSumExp([1000.0, 1000.0])", 1000.0 + 2.0_f64.ln());
    assert_float_result("math.logSumExp([-1000.0, -1000.0])", -1000.0 + 2.0_f64.ln());
    assert_float_result("math.logSumExp([0.0, -40.0])", (-40.0_f64).exp().ln_1p());
    assert_float_result("math.logSumExp([42.0, -1.0 / 0.0])", 42.0);
}

#[test]
fn exact_combinatorics_handles_boundaries_and_arbitrary_precision() {
    for (expression, expected) in [
        ("math.factorial(0)", "1"),
        ("math.factorial(30)", "265252859812191058636308480000000"),
        ("math.choose(100, 50)", "100891344545564193334812497256"),
        ("math.choose(5, -1)", "0"),
        ("math.choose(5, 6)", "0"),
        ("math.choose(0, 0)", "1"),
        ("math.fallingFactorial(5, 3)", "60"),
        ("math.fallingFactorial(3, 4)", "0"),
        ("math.risingFactorial(5, 3)", "210"),
        ("math.risingFactorial(0, 0)", "1"),
        ("math.risingFactorial(0, 100000000000000000000)", "0"),
    ] {
        let (vm, result) = run_expression(expression);
        assert_eq!(as_big_int(&vm, result), expected.parse::<BigInt>().unwrap(), "{expression}");
    }
}

#[test]
fn exact_combinatorics_rejects_invalid_domains_and_honors_resource_limits() {
    for expression in [
        "math.factorial(-1)",
        "math.choose(-1, 0)",
        "math.fallingFactorial(1, -1)",
        "math.risingFactorial(-1, 0)",
    ] {
        let (_temp, program) = compile_package(&format!("import universe.math as math\n{expression}\n"));
        let mut vm = VM::new();
        let error = vm.run_compiled(&program).expect_err("negative domain must raise");
        let RuntimeError::Raise { error, .. } = runtime_error(&error) else {
            panic!("expected ArgumentError")
        };
        assert_eq!(vm.heap.class(error.class(&vm)).name, "ArgumentError");
    }
    let (_temp, program) = compile_package("import universe.math as math\nlet result = math.choose(100, 50)\n");
    let mut vm = VM::new();
    vm.numeric_policy.max_integer_bits = Some(32);
    let error = vm.run_compiled(&program).expect_err("exact result must respect bit budget");
    assert_numeric_limit(&mut vm, &error);
}

#[test]
fn special_functions_match_independent_high_precision_references() {
    let mut source = String::from("import numerics.special as special\n");
    let mut references = Vec::new();
    for row in include_str!("../../fixtures/math_special_reference.tsv")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let mut columns = row.split('\t');
        let expression = columns.next().unwrap();
        let expected: f64 = columns.next().unwrap().parse().unwrap();
        let tolerance: f64 = columns.next().unwrap().parse().unwrap();
        source.push_str(&format!("let result{} = {expression}\n", references.len()));
        references.push((expression, expected, tolerance));
    }
    assert!(references.len() > 100, "reference fixture must select real cases");
    let (_temp, program) = compile_numerics_package(&source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("special-function reference matrix executes");
    let module = vm.module_registry.get(&program.entry).unwrap().object;
    for (i, (expression, expected, tolerance)) in references.iter().enumerate() {
        let actual = read_global(&vm, module, &format!("result{i}")).as_float().expect("Float result");
        if expected.is_infinite() {
            assert_eq!(actual, *expected, "{expression}");
        } else {
            let scale = if expression.contains(".log") && !expression.contains(".logErfc") {
                expected.abs().max(1.0)
            } else {
                expected.abs()
            };
            let budget = tolerance * scale;
            assert!(
                (actual - expected).abs() <= budget,
                "{expression}: actual={actual:.17e}, reference={expected:.17e}, budget={budget:.3e}"
            );
        }
    }
}

#[test]
fn probability_helpers_normalize_extremes_without_replaying_or_mutating_inputs() {
    let source = r#"
import numerics.probability as p
import universe.math as math
let weights = [1.0e308, 1.0e308, 0.0]
let normalized = p.normalizeWeights(weights.iterator)
let first = normalized.at(0)
let second = normalized.at(1)
let zero = normalized.at(2)
let original = weights.at(0)
let iterator = [1.0e308, 1.0e308].iterator
let logs = p.normalizedLogWeights(iterator)
let logFirst = logs.at(0)
let exhausted = iterator.next.isNone
let impossible = p.normalizedLogWeights([-1.0 / 0.0, 0.0]).at(0)
let smallLog = p.normalizedLogWeights([0.0, -1000.0]).at(1)
let tiny = p.normalizeLogWeights([0.0, -1000.0]).at(1)
let hugeSpread = p.normalizeLogWeights([1.0e308, -1.0e308]).at(1)
let logitZero = p.logit(0.0)
let logitOne = p.logit(1.0)
let logitHalf = p.logit(0.5)
let oddsOne = p.odds(1.0)
let oddsZero = p.odds(-0.0)
let negative = p.invLogit(-1000.0)
let positive = p.invLogit(1000.0)
let middle = p.invLogit(0.0)
let reductionIterator = [1000.0, 1000.0].iterator
let reduction = math.logSumExp(reductionIterator)
let reductionExhausted = reductionIterator.next.isNone
"#;
    let (_temp, program) = compile_numerics_package(source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("probability helper client executes");
    let module = vm.module_registry.get(&program.entry).unwrap().object;
    for (name, expected) in [
        ("first", 0.5),
        ("second", 0.5),
        ("zero", 0.0),
        ("original", 1e308),
        ("logFirst", -2.0_f64.ln()),
        ("impossible", f64::NEG_INFINITY),
        ("smallLog", -1000.0),
        ("tiny", 0.0),
        ("hugeSpread", 0.0),
        ("logitZero", f64::NEG_INFINITY),
        ("logitOne", f64::INFINITY),
        ("logitHalf", 0.0),
        ("oddsOne", f64::INFINITY),
        ("oddsZero", 0.0),
        ("negative", 0.0),
        ("positive", 1.0),
        ("middle", 0.5),
        ("reduction", 1000.0 + 2.0_f64.ln()),
    ] {
        let actual = read_global(&vm, module, name).as_float().unwrap();
        assert_eq!(actual.to_bits(), expected.to_bits(), "{name}");
    }
    assert_eq!(read_global(&vm, module, "exhausted"), Value::bool(true));
    assert_eq!(read_global(&vm, module, "reductionExhausted"), Value::bool(true));
}

#[test]
fn probability_helpers_preserve_distinct_error_categories() {
    for (expression, class) in [
        ("p.normalizeWeights([0.0, 0.0])", "UndefinedNormalizationError"),
        ("p.normalizeLogWeights([-1.0 / 0.0])", "UndefinedNormalizationError"),
        ("p.normalizeWeights([-1.0])", "ArgumentError"),
        ("p.normalizedLogWeights([1.0 / 0.0])", "ArgumentError"),
        ("p.normalizedLogWeights([1.0e308, -1.0e308])", "NumericalRangeError"),
        ("p.logit(1.1)", "ArgumentError"),
    ] {
        let (_temp, program) = compile_numerics_package(&format!("import numerics.probability as p\n{expression}\n"));
        let mut vm = VM::new();
        let failure = vm.run_compiled(&program).expect_err("invalid probability operation must raise");
        let RuntimeError::Raise { error, .. } = runtime_error(&failure) else {
            panic!("expected raised {class}: {failure}")
        };
        assert_eq!(vm.heap.class(error.class(&vm)).name, class, "{expression}");
    }
}

#[test]
fn special_functions_reject_invalid_domains_and_unrepresentable_log_results() {
    for (expression, class) in [
        ("special.Gamma.log(0.0)", "ArgumentError"),
        ("special.Gamma.p(2.0, -1.0)", "ArgumentError"),
        ("special.Beta.regularized(1.1, 2.0, 3.0)", "ArgumentError"),
        ("special.Normal.quantile(0.0 / 0.0)", "ArgumentError"),
        ("special.Normal.survival(0.0 / 0.0)", "ArgumentError"),
        ("special.Normal.logSurvival(0.0 / 0.0)", "ArgumentError"),
        ("special.erf(0.0 / 0.0)", "ArgumentError"),
        ("special.Gamma.logQ(1000001.0, 1.0)", "NumericalRangeError"),
        ("special.logErfc(1.0e308)", "NumericalRangeError"),
        ("special.logChoose(9007199254740992, 1)", "NumericalRangeError"),
        ("special.Combinatorics.logChoose(9007199254740992, 1)", "NumericalRangeError"),
    ] {
        let (_temp, program) = compile_numerics_package(&format!("import numerics.special as special\n{expression}\n"));
        let mut vm = VM::new();
        let failure = vm.run_compiled(&program).expect_err("invalid special-function operation must raise");
        let RuntimeError::Raise { error, .. } = runtime_error(&failure) else {
            panic!("expected raised {class}: {failure}")
        };
        assert_eq!(vm.heap.class(error.class(&vm)).name, class, "{expression}");
    }
}

#[test]
fn special_function_families_public_facade_and_imports() {
    let source = r#"
from numerics.special import (
  Gamma,
  Beta,
  Normal,
  Combinatorics,
  erf,
  erfc,
  logErfc,
  logFactorial,
  logChoose,
  NumericalRangeError,
  NumericalEvaluationError
)

let gammaLog = Gamma.log(1.0)
let gammaP = Gamma.p(1.0, 1.0)
let gammaQ = Gamma.q(1.0, 1.0)
let gammaLogP = Gamma.logP(1.0, 1.0)
let gammaLogQ = Gamma.logQ(1.0, 1.0)
let gammaInvP = Gamma.inverseP(1.0, 0.5)

let betaLog = Beta.log(1.0, 1.0)
let betaReg = Beta.regularized(0.5, 1.0, 1.0)
let betaComp = Beta.complement(0.5, 1.0, 1.0)
let betaLogReg = Beta.logRegularized(0.5, 1.0, 1.0)
let betaLogComp = Beta.logComplement(0.5, 1.0, 1.0)
let betaInv = Beta.inverse(0.5, 1.0, 1.0)

let normalQuantile = Normal.quantile(0.5)
let normalCdf = Normal.cdf(0.0)
let normalSurvival = Normal.survival(0.0)
let normalLogCdf = Normal.logCdf(0.0)
let normalLogSurvival = Normal.logSurvival(0.0)

let erfZero = erf(0.0)
let erfOne = erf(1.0)
let erfNegOne = erf(-1.0)
let erfcZero = erfc(0.0)
let logErfcZero = logErfc(0.0)

let combLogFact = Combinatorics.logFactorial(5)
let combLogChoose = Combinatorics.logChoose(5, 2)
let flatLogFact = logFactorial(5)
let flatLogChoose = logChoose(5, 2)
"#;

    let (_temp, program) = compile_numerics_package(source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("special function structured facade executes");
    let module = vm.module_registry.get(&program.entry).unwrap().object;

    assert_eq!(read_global(&vm, module, "gammaLog").as_float().unwrap(), 0.0);
    let p_val = read_global(&vm, module, "gammaP").as_float().unwrap();
    let q_val = read_global(&vm, module, "gammaQ").as_float().unwrap();
    assert!((p_val + q_val - 1.0).abs() < 1e-15);
    assert!((read_global(&vm, module, "gammaLogQ").as_float().unwrap() - -1.0).abs() < 1e-14);
    let inv_p = read_global(&vm, module, "gammaInvP").as_float().unwrap();
    assert!((inv_p - 2.0_f64.ln()).abs() < 1e-12);

    assert!(read_global(&vm, module, "betaLog").as_float().unwrap().abs() < 1e-14);
    assert!((read_global(&vm, module, "betaReg").as_float().unwrap() - 0.5).abs() < 1e-14);
    assert!((read_global(&vm, module, "betaComp").as_float().unwrap() - 0.5).abs() < 1e-14);
    assert!((read_global(&vm, module, "betaLogReg").as_float().unwrap() - -2.0_f64.ln()).abs() < 1e-14);
    assert!((read_global(&vm, module, "betaLogComp").as_float().unwrap() - -2.0_f64.ln()).abs() < 1e-14);
    assert!((read_global(&vm, module, "betaInv").as_float().unwrap() - 0.5).abs() < 1e-10);

    assert_eq!(read_global(&vm, module, "normalQuantile").as_float().unwrap(), 0.0);
    assert_eq!(read_global(&vm, module, "normalCdf").as_float().unwrap(), 0.5);
    assert_eq!(read_global(&vm, module, "normalSurvival").as_float().unwrap(), 0.5);
    assert!((read_global(&vm, module, "normalLogCdf").as_float().unwrap() - -2.0_f64.ln()).abs() < 1e-15);
    assert!((read_global(&vm, module, "normalLogSurvival").as_float().unwrap() - -2.0_f64.ln()).abs() < 1e-15);

    assert_eq!(read_global(&vm, module, "erfZero").as_float().unwrap(), 0.0);
    let erf_one = read_global(&vm, module, "erfOne").as_float().unwrap();
    let erf_neg_one = read_global(&vm, module, "erfNegOne").as_float().unwrap();
    assert!((erf_one + erf_neg_one).abs() < 1e-15, "erf must be odd");
    assert!((erf_one - 0.8427007929497149).abs() < 1e-12);
    assert_eq!(read_global(&vm, module, "erfcZero").as_float().unwrap(), 1.0);
    assert_eq!(read_global(&vm, module, "logErfcZero").as_float().unwrap(), 0.0);

    let expected_fact5 = 120.0_f64.ln();
    let expected_choose5_2 = 10.0_f64.ln();
    assert!((read_global(&vm, module, "combLogFact").as_float().unwrap() - expected_fact5).abs() < 1e-14);
    assert!((read_global(&vm, module, "flatLogFact").as_float().unwrap() - expected_fact5).abs() < 1e-14);
    assert!((read_global(&vm, module, "combLogChoose").as_float().unwrap() - expected_choose5_2).abs() < 1e-14);
    assert!((read_global(&vm, module, "flatLogChoose").as_float().unwrap() - expected_choose5_2).abs() < 1e-14);
}

#[test]
fn float_representation_methods_and_stable_reductions_cover_boundaries() {
    let source = r#"
import universe.math as math

let fused = 1.000000007450580596923828125.fma(0.999999992549419403076171875, -1.0)
let copiedNegative = 1.0.copySign(-0.0)
let negativeZero = 0.0.copySign(-1.0)
let smallestSubnormal = 1.0.scalePow2(-1074)
let underflow = 1.0.scalePow2(-1075)
let overflow = 1.0.scalePow2(1024)
let logAdd = math.logAddExp(1000.0, 1000.0)
let logAddNaN = math.logAddExp(0.0 / 0.0, 1.0 / 0.0)
let logOneMinus = math.log1mexp(-0.6931471805599453)
let meanOption = math.meanOption([1.0e308, -1.0e308].iterator)
let mean = meanOption.match(some: |value| { value }, none: || { -1.0 })
let empty = List<Float>.new()
let emptyMeanIsNone = math.meanOption(empty.iterator).isNone
"#;
    let (_temp, program) = compile_package(source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("Float boundary client executes");
    let module = vm.module_registry.get(&program.entry).unwrap().object;

    for (name, expected) in [
        ("fused", -2.0_f64.powi(-54)),
        ("copiedNegative", -1.0),
        ("negativeZero", -0.0),
        ("smallestSubnormal", f64::from_bits(1)),
        ("underflow", 0.0),
        ("overflow", f64::INFINITY),
        ("logAdd", 1000.0 + 2.0_f64.ln()),
        ("logOneMinus", -2.0_f64.ln()),
        ("mean", 0.0),
    ] {
        let actual = read_global(&vm, module, name).as_float().unwrap();
        assert_eq!(actual.to_bits(), expected.to_bits(), "{name}");
    }
    assert!(read_global(&vm, module, "logAddNaN").as_float().unwrap().is_nan());
    assert_eq!(read_global(&vm, module, "emptyMeanIsNone"), Value::bool(true));
}

#[test]
fn rational_negative_inverse_uses_the_normalized_constructor() {
    let source = r#"
from universe.numbers import Rational

let value = Rational.new(-2, 3)
let reciprocal = value.inverse
let expected = Rational.new(-3, 2)
let exact = reciprocal === expected
let productIsOne = (value * reciprocal) === Rational.fromInt(1)
"#;
    let (_temp, program) = compile_package(source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("negative Rational inverse executes");
    let module = vm.module_registry.get(&program.entry).unwrap().object;
    assert_eq!(read_global(&vm, module, "exact"), Value::bool(true));
    assert_eq!(read_global(&vm, module, "productIsOne"), Value::bool(true));
}

#[test]
fn generic_method_infers_shared_callback_result_for_option_and_plain_calls() {
    let source = r#"
class Probe {
  @class optionCheck(_ value: Option<Int>) -> Bool {
    return value.match(
      some: |item| { item > 0 },
      none: || { false }
    )
  }

  @class callbackCheck() -> Bool {
    return Probe.sameResult(
      1,
      |item| { item > 0 },
      || { false }
    )
  }

  @class sameResult<T, R>(_ value: T, _ some: (T) -> R, _ none: () -> R) -> R {
    some.call(value)
  }
}

let optionResult = Probe.optionCheck(Option<Int>::Some(1))
let callbackResult = Probe.callbackCheck()
"#;
    let (_temp, program) = compile_package(source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("generic callback result inference executes");
    let module = vm.module_registry.get(&program.entry).unwrap().object;
    assert_eq!(read_global(&vm, module, "optionResult"), Value::bool(true));
    assert_eq!(read_global(&vm, module, "callbackResult"), Value::bool(true));
}

#[test]
fn generic_plain_method_infers_shared_callback_result() {
    let source = r#"
class Probe {
  @class callbackCheck() -> Bool {
    return Probe.sameResult(
      1,
      |item| { item > 0 },
      || { false }
    )
  }

  @class sameResult<T, R>(_ value: T, _ some: (T) -> R, _ none: () -> R) -> R {
    some.call(value)
  }
}

let callbackResult = Probe.callbackCheck()
"#;
    let (_temp, _program) = compile_package(source);
}

#[test]
fn generic_iterable_specializes_typed_and_literal_inputs() {
    let source = r#"
from universe.collections.iterator_laws import Iterable

class Probe {
  @class accept<I>(_ input: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  { 1.0 }
}

let values = List<Float>.new()
values.append(1.0)
let typed = Probe.accept(values)
let iterator = Probe.accept(values.iterator)
let literal = Probe.accept([1.0])
let literalIterator = Probe.accept([1.0].iterator)
"#;
    let (_temp, program) = compile_package(source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("generic Iterable calls execute");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    for name in ["typed", "iterator", "literal", "literalIterator"] {
        assert_eq!(read_global(&vm, module, name).as_float(), Some(1.0), "{name}");
    }
}

#[test]
fn public_statistics_family_applies_its_recipe_after_live_member_selection() {
    let source = r#"
import numerics.statistics as stats

let values = List<Float>.new()
values.append(1.0)
values.append(3.0)
let mean = stats.mean(values.iterator)
"#;
    let (_temp, program) = compile_numerics_package(source);
    let entry_module = program.modules.get(&program.entry).expect("entry module");
    let (call_range, recipe) = entry_module
        .lowering
        .call_environment_recipes
        .iter()
        .find(|(_, recipe)| recipe.callable.selector.as_ref() == "mean(_)")
        .expect("generic recipe for the public mean call");
    let family_application = entry_module
        .lowering
        .family_applications
        .iter()
        .find(|(site, _)| site.range == *call_range)
        .map(|(_, spec)| spec)
        .expect("mean recipe is associated with its Family application");
    assert!(matches!(
        family_application,
        phalcom_core::modules::semantic_lowering::FamilyApplicationLoweringSpec::Static {
            kind: phalcom_semantic::checker::associated::FamilyApplicationKind::Behavioral,
            ..
        } | phalcom_core::modules::semantic_lowering::FamilyApplicationLoweringSpec::DynamicPack {
            kind: phalcom_semantic::checker::associated::FamilyApplicationKind::Behavioral,
            ..
        }
    ));
    let owner = recipe.callable.owner.try_declaration().expect("declaration-owned public method");
    assert_eq!(owner.path.last().map(AsRef::as_ref), Some("Descriptive"));

    let mut vm = VM::new();
    vm.run_compiled(&program)
        .expect("public Family call executes with its selected generic evidence");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(read_global(&vm, module, "mean").as_float(), Some(2.0));

    let mut mismatched = program.clone();
    let mut lowering = (*mismatched.modules[&mismatched.entry].lowering).clone();
    let mismatched_recipe = lowering.call_environment_recipes.get_mut(call_range).expect("mean call recipe");
    let mismatched_recipe = std::sync::Arc::make_mut(mismatched_recipe);
    mismatched_recipe.callable.side = match mismatched_recipe.callable.side {
        phalcom_type_meta::identity::StableDispatchSide::Instance => phalcom_type_meta::identity::StableDispatchSide::Class,
        phalcom_type_meta::identity::StableDispatchSide::Class => phalcom_type_meta::identity::StableDispatchSide::Instance,
    };
    mismatched.modules.get_mut(&mismatched.entry).expect("entry module").lowering = std::sync::Arc::new(lowering);
    let mut mismatched_vm = VM::new();
    let failure = mismatched_vm
        .run_compiled(&mismatched)
        .expect_err("same-selector but wrong callable identity must fail closed");
    assert!(
        format!("{}", runtime_error(&failure)).contains("canonical callable identity mismatch"),
        "unexpected failure: {failure}"
    );
}

#[test]
fn numerics_modules_compile_and_run_as_one_public_client() {
    let source = r#"
import numerics as numerics
import numerics.special as special
import numerics.statistics as stats
import numerics.probability as probability
import numerics.probability.bayes as bayes
import numerics.probability.monte_carlo as monte
from universe.random import Xoshiro256PlusPlus

let gammaLog = special.Gamma.log(5.0)
let mean = stats.mean([1.0, 2.0, 3.0].iterator)
let stream = [1.0, 2.0, 3.0].iterator
let streamMean = stats.mean(stream)
let streamExhausted = stream.next.isNone
let exactMean = stats.meanExact([1, 2, 3].iterator)
let exactMeanFloat = exactMean.toFloat

let normal = probability.Normal.new(mean: 0.0, stddev: 1.0)
let normalMedian = normal.quantile(0.5)
let gamma = probability.Gamma.new(shape: 1.0, rate: 1.0)
let gammaLogDensity = gamma.logPdf(1.0)
let gammaCdf = gamma.cdf(0.5)
let gammaMedian = gamma.quantile(0.5)
let beta = probability.Beta.new(alpha: 2.0, beta: 3.0)
let betaLogDensity = beta.logPdf(0.5)
let betaCdf = beta.cdf(0.5)
let betaMedian = beta.quantile(0.5)
let betaMedianCdf = beta.cdf(betaMedian)
let binomial = probability.Binomial.new(trials: 4, probability: 0.5)
let binomialCdf = binomial.cdf(1)
let poisson = probability.Poisson.new(rate: 2.0)
let poissonCdf = poisson.cdf(1)

let bernoulli = probability.Bernoulli.new(0.25)
let bernoulliMean = bernoulli.mean
let bernoulliVariance = bernoulli.variance
let bernoulliSupport = probability.bernoulliSupport(bernoulli)
let supportHasFalse = bernoulliSupport.contains(false)
let supportHasTrue = bernoulliSupport.contains(true)

let prior = probability.Beta.new(alpha: 1.0, beta: 1.0)
let counts = bayes.BernoulliCounts.fromBools([true, true, false].iterator)
let posterior = bayes.betaBernoulli(prior: prior, counts: counts)
let posteriorAlpha = posterior.alpha
let posteriorBeta = posterior.beta
let logEvidence = bayes.logBernoulliSequenceEvidence(prior: prior, counts: counts)

let mut callbackCount = 0
let generator = Xoshiro256PlusPlus.seed(17)
let estimate = monte.expectation(
  bernoulli,
  count: 16,
  using: generator,
  evaluate: |value| {
    callbackCount = callbackCount + 1
    if value { 1.0 } else { 0.0 }
  }
)
let estimateCount = estimate.count

let sharedStatisticError = numerics.UndefinedStatisticError === stats.UndefinedStatisticError and stats.UndefinedStatisticError === probability.UndefinedStatisticError
"#;
    let (_temp, program) = compile_numerics_package(source);
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("integrated numerics client executes");
    let module = vm.module_registry.get(&program.entry).unwrap().object;

    for (name, expected) in [
        ("gammaLog", 24.0_f64.ln()),
        ("mean", 2.0),
        ("streamMean", 2.0),
        ("exactMeanFloat", 2.0),
        ("normalMedian", 0.0),
        ("gammaLogDensity", -1.0),
        ("gammaCdf", 1.0 - (-0.5_f64).exp()),
        ("gammaMedian", 2.0_f64.ln()),
        ("betaLogDensity", 0.4054651081081644),
        ("betaCdf", 0.6875),
        ("binomialCdf", 0.3125),
        ("poissonCdf", 3.0 * (-2.0_f64).exp()),
        ("bernoulliMean", 0.25),
        ("bernoulliVariance", 0.1875),
        ("posteriorAlpha", 3.0),
        ("posteriorBeta", 2.0),
        ("logEvidence", -12.0_f64.ln()),
    ] {
        let actual = read_global(&vm, module, name).as_float().unwrap();
        assert!((actual - expected).abs() <= 1e-12 * expected.abs().max(1.0), "{name}: {actual} != {expected}");
    }

    assert_eq!(read_global(&vm, module, "streamExhausted"), Value::bool(true));
    assert_eq!(read_global(&vm, module, "supportHasFalse"), Value::bool(true));
    assert_eq!(read_global(&vm, module, "supportHasTrue"), Value::bool(true));
    assert_eq!(read_global(&vm, module, "callbackCount").as_int(), Some(16));
    assert_eq!(read_global(&vm, module, "estimateCount").as_int(), Some(16));
    assert!((read_global(&vm, module, "betaMedianCdf").as_float().unwrap() - 0.5).abs() <= 1e-12);
    assert_eq!(read_global(&vm, module, "sharedStatisticError"), Value::bool(true));
}

#[test]
fn statistics_mean_preserves_empty_input_error() {
    let source = r#"
import numerics.statistics as stats
let empty = List<Float>.new()
stats.mean(empty.iterator)
"#;
    let (_temp, program) = compile_numerics_package(source);
    let mut vm = VM::new();
    let failure = vm.run_compiled(&program).expect_err("empty mean must raise");
    let RuntimeError::Raise { error, .. } = runtime_error(&failure) else {
        panic!("expected EmptyInputError: {failure}")
    };
    assert_eq!(vm.heap.class(error.class(&vm)).name, "EmptyInputError");
}
