from pathlib import Path

path = Path('phalcom-core/tests/core/language/traits.rs')
text = path.read_text()
anchor = '''#[test]
fn trait_default_executes_abstract_getter_satisfied_by_data_component() {'''
insert = r'''#[test]
fn recursive_trait_default_reuses_conformance_environment() {
    let source = r#"
trait Recursive {
  countdown(_ n: Int) -> Int {
    return (n == 0).ifTrue(
      || { 0 },
      ifFalse: || { self.countdown(n - 1) }
    )
  }
}
class User {}
impl Recursive for User {}
class Caller { run(_ user: User) -> Int { user.countdown(3) } }
let result = Caller.new().run(User.new())
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("recursive trait default compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("recursive trait default executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "0");
}

#[test]
fn trait_default_conformance_environment_survives_forced_gc() {
    let source = r#"
trait Identified {
  name -> String
  label -> String {
    System.gc
    self.name
  }
}
class User {}
impl Identified for User { name -> String { "gc-user" } }
class Caller { run(_ user: User) -> String { user.label } }
let result = Caller.new().run(User.new())
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("GC trait default compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("GC trait default executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "gc-user");
}

#[test]
fn trait_default_executes_abstract_getter_satisfied_by_data_component() {'''
count = text.count(anchor)
if count != 1:
    raise SystemExit(f'anchor count: {count}')
path.write_text(text.replace(anchor, insert, 1))
