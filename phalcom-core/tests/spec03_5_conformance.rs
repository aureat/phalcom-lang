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
fn test_runtime_native_method_reflection() {
    let (result, vm) = eval_source(
        r#"
        let m = true.methodFor(#not)
        let res = [m.isNative, m.isIntrinsic, m.implementationKind == #native]
        "#,
        "res",
    )
    .expect("eval succeeds");

    assert_eq!(result.to_string(&vm), "[true, true, true]");
}

#[test]
fn test_runtime_source_method_reflection() {
    let (result, vm) = eval_source(
        r#"
        class Greeter {
            sayHello() { "hello" }
        }
        let g = Greeter.new()
        let m = g.methodFor(#"sayHello()")
        let res = [m.isNative, m.isIntrinsic, m.implementationKind == #source]
        "#,
        "res",
    )
    .expect("eval succeeds");

    assert_eq!(result.to_string(&vm), "[false, false, true]");
}
