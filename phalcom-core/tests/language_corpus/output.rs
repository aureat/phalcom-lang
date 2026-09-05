use phalcom_core::vm::{BufferedOutput, VM};

fn run(source: &str) -> Vec<u8> {
    let sink = BufferedOutput::new();
    let handle = sink.handle();
    let mut vm = VM::new_native_with_output(Box::new(sink));
    let module = vm.create_module("output-test", "<output-test>");
    let closure = vm.compile_closure(module, source).expect("output fixture compiles");
    vm.run_in_module(module, closure).expect("output fixture runs");
    handle.bytes()
}

#[test]
fn buffered_vm_captures_exact_print_bytes() {
    assert_eq!(run("System.print(1)"), b"1\n");
}

#[test]
fn display_override_stays_on_fixture_sink() {
    assert_eq!(
        run("class Label { @constructor new() {} toString { \"nested\" } }\nSystem.print(Label.new())"),
        b"nested\n"
    );
}

#[test]
fn separate_vms_have_independent_output_buffers() {
    let sink_a = BufferedOutput::new();
    let handle_a = sink_a.handle();
    let mut vm_a = VM::new_native_with_output(Box::new(sink_a));
    let module_a = vm_a.create_module("output-a", "<output-a>");
    let closure_a = vm_a.compile_closure(module_a, "System.print(1)").expect("fixture A compiles");
    vm_a.run_in_module(module_a, closure_a).expect("fixture A runs");

    let sink_b = BufferedOutput::new();
    let handle_b = sink_b.handle();
    let mut vm_b = VM::new_native_with_output(Box::new(sink_b));
    let module_b = vm_b.create_module("output-b", "<output-b>");
    let closure_b = vm_b.compile_closure(module_b, "System.print(2)").expect("fixture B compiles");
    vm_b.run_in_module(module_b, closure_b).expect("fixture B runs");

    assert_eq!(handle_a.bytes(), b"1\n");
    assert_eq!(handle_b.bytes(), b"2\n");
}
