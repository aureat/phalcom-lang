use phalcom_core::{value::Value, vm::VM};
fn main() {
    let mut vm = VM::new_native();
    for (label, receiver, selector, arg) in [
        ("negative float remainder", Value::float(-5.0), "%(_)", Value::float(3.0)),
        ("negative divisor float remainder", Value::float(5.0), "%(_)", Value::float(-3.0)),
        ("mixed exact quotient", Value::int(9_007_199_254_740_993), "~/(_)", Value::float(1.0)),
        ("integer quotient control", Value::int(9_007_199_254_740_993), "~/(_)", Value::int(1)),
    ] {
        let sym = vm.get_or_intern(selector);
        println!("{label}: {:?}", vm.send_dynamic(receiver, sym, &[arg]));
    }
}
