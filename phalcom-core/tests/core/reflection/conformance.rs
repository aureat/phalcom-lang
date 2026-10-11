use phalcom_core::heap::Object;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

fn eval_source(src: &str, var_name: &str) -> Result<(Value, VM), String> {
    let mut vm = VM::new();
    let main = vm.create_module("main", "<spec03_5 test>");
    vm.interpret_source(main, src).map_err(|error| error.to_string())?;
    let symbol = vm.interner.intern(var_name);
    let Object::Module(module) = vm.heap.get(main) else {
        return Err("main module is not a module".to_string());
    };
    let val = module.get(symbol).ok_or_else(|| format!("variable `{var_name}` not found"))?;
    Ok((val, vm))
}

#[test]
fn native_method_reflection_reports_native_metadata() {
    let (result, vm) = eval_source(
        r#"
        let m = true.methodFor(#not)
        let res = <Bool>[m.isNative, m.isIntrinsic, m.implementationKind == #native]
        "#,
        "res",
    )
    .expect("eval succeeds");

    assert_eq!(result.to_string(&vm), "Array([true, true, true])");
}

#[test]
fn source_method_reflection_reports_source_metadata() {
    let (result, vm) = eval_source(
        r#"
        class Greeter {
            sayHello() { "hello" }
        }
        let g = Greeter.new()
        let m = g.methodFor(#"sayHello()")
        let res = <Bool>[m.isNative, m.isIntrinsic, m.implementationKind == #phalcom]
        "#,
        "res",
    )
    .expect("eval succeeds");

    assert_eq!(result.to_string(&vm), "Array([false, false, true])");
}

#[test]
fn foreign_declaration_fails_at_explicit_binding_boundary() {
    let mut vm = VM::new();
    let main = vm.create_module("main", "<foreign test>");
    let error = vm
        .interpret_source(main, "class ForeignBox { foreign external() -> Int }\n")
        .expect_err("foreign declaration must not silently become a missing body");

    assert!(error.to_string().contains("foreign.binding_unimplemented"), "unexpected error: {error}");
}
