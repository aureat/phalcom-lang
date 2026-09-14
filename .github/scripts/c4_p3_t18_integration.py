from pathlib import Path

path = Path('phalcom-core/tests/core/language/traits.rs')
text = path.read_text()
anchor = '''#[test]
fn trait_default_executes_abstract_getter_satisfied_by_data_component() {'''
insert = r'''#[test]
fn generic_source_conformance_executes_for_multiple_exact_targets() {
    let source = r#"
trait Echo<T> { echo(_ value: T) -> T }
class Value<T> { @constructor new() {} }
impl<T> Echo<T> for Value<T> {
  echo(_ value: T) -> T { value }
}
class Caller {
  intEcho(_ receiver: Value<Int>) -> Int { receiver.echo(7) }
  stringEcho(_ receiver: Value<String>) -> String { receiver.echo("seven") }
}
let caller = Caller.new()
let intValue: Value<Int> = Value.new()
let stringValue: Value<String> = Value.new()
let intResult = caller.intEcho(intValue)
let stringResult = caller.stringEcho(stringValue)
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("generic source conformance compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("generic source conformance executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "intResult").expect("int result").to_string(&vm), "7");
    assert_eq!(named(&vm, module, "stringResult").expect("string result").to_string(&vm), "seven");
}

#[test]
fn generic_trait_default_preserves_runtime_type_environment_with_conformance_environment() {
    let source = r#"
trait Pipeline<T> {
  identity(_ value: T) -> T
  through(_ value: T) -> T { self.identity(value) }
}
class Box<T> { @constructor new() {} }
impl<T> Pipeline<T> for Box<T> {
  identity(_ value: T) -> T { value }
}
class Caller {
  intRun(_ receiver: Box<Int>) -> Int { receiver.through(42) }
  stringRun(_ receiver: Box<String>) -> String { receiver.through("typed") }
}
let caller = Caller.new()
let intBox: Box<Int> = Box.new()
let stringBox: Box<String> = Box.new()
let intResult = caller.intRun(intBox)
let stringResult = caller.stringRun(stringBox)
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("generic trait environment source compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("generic trait environment source executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "intResult").expect("int result").to_string(&vm), "42");
    assert_eq!(named(&vm, module, "stringResult").expect("string result").to_string(&vm), "typed");
}

#[test]
fn exact_enum_case_conformance_executes_only_for_exact_case() {
    let source = r#"
trait Printable { print -> String }
enum Result<T> { Ok(T), Error(String) }
impl Printable for Result<Int>::Ok(_) {
  print -> String { "ok-case" }
}
class Caller {
  run(_ value: Result<Int>::Ok(_)) -> String { value.print }
}
let result = Caller.new().run(Result<Int>::Ok(42))
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("exact-case conformance compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("exact-case conformance executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "ok-case");
}

#[test]
fn exact_generic_trait_bound_reference_keeps_specialized_selection() {
    let source = r#"
trait Tagged { tag -> String }
class Value<T> { @constructor new() {} }
impl Tagged for Value<Int> { tag -> String { "int-ref" } }
impl Tagged for Value<String> { tag -> String { "string-ref" } }
class Caller {
  intTag(_ value: Value<Int>) -> String { let f = &value.tag; f() }
  stringTag(_ value: Value<String>) -> String { let f = &value.tag; f() }
}
let caller = Caller.new()
let intValue: Value<Int> = Value.new()
let stringValue: Value<String> = Value.new()
let intResult = caller.intTag(intValue)
let stringResult = caller.stringTag(stringValue)
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("exact generic trait references compile");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("exact generic trait references execute");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "intResult").expect("int result").to_string(&vm), "int-ref");
    assert_eq!(named(&vm, module, "stringResult").expect("string result").to_string(&vm), "string-ref");
}

#[test]
fn trait_default_executes_abstract_getter_satisfied_by_data_component() {'''
count = text.count(anchor)
if count != 1:
    raise SystemExit(f'anchor count: {count}')
path.write_text(text.replace(anchor, insert, 1))
