//! C3 trait compiler/runtime boundary tests.

use super::vm_support;
use phalcom_core::modules::compile::{EntrySelection, ProgramAnalyzer, ProgramCompiler};
use phalcom_core::value::Value;
use std::sync::Arc;

#[test]
fn random_facade_client_has_complete_source_trait_evidence() {
    let source = "from universe.random import Xoshiro256PlusPlus\nlet rng = Xoshiro256PlusPlus.seed(42)\nlet result = rng.nextBool\n";
    let mut session = phalcom_core::modules::compile_session::ProgramCompilationSession::standalone().unwrap();
    let program = session.compile_fixture(std::path::Path::new("random-client.ph"), source).unwrap();
    let lowering = &program.modules[&program.entry].lowering;
    assert!(
        lowering
            .trait_invocations
            .values()
            .any(|spec| spec.selection.requirement.selector.encode() == "nextBool"),
        "client closure includes source conformance"
    );
}

#[test]
fn random_secure_runtime_contract_is_not_a_generator() {
    let source = r#"
from universe.random import SecureRandom, EntropyError
let secure = SecureRandom.system
let zero = secure.bytes(0)
let one = secure.bytes(1)
let mut caught = false
try { secure.bytes(-1) } on ArgumentError error { caught = true }
let result = zero.size + one.size
"#;
    let mut session = phalcom_core::modules::compile_session::ProgramCompilationSession::standalone().unwrap();
    let program = session
        .compile_fixture(std::path::Path::new("secure-random.ph"), source)
        .expect("secure source compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("secure source executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").unwrap().to_string(&vm), "1");
    assert_eq!(named(&vm, module, "caught").unwrap().to_string(&vm), "true");
}

#[test]
fn random_rejects_secure_conformance_hidden_children_and_wrong_output_types() {
    for (source, expected) in [
        (
            "from universe.random import SecureRandom, Generator\nclass Consumer { @class use<G>(_ generator: G) -> Int where G |= Generator { generator.nextU64 } }\nConsumer.use(SecureRandom.system)",
            "genericconstraintunsatisfied",
        ),
        ("import universe.random.xoshiro256pp\n", "moduleexposurerejected"),
        (
            "from universe.random import Xoshiro256PlusPlus, Bernoulli\nlet wrong: Int = Bernoulli.new(0.5).sample(using: Xoshiro256PlusPlus.seed(42))",
            "assignable",
        ),
        (
            "from universe.random import Xoshiro256PlusPlus\nXoshiro256PlusPlus.seed(42).nextInt(below: 1.0)",
            "argument",
        ),
    ] {
        let mut session = phalcom_core::modules::compile_session::ProgramCompilationSession::standalone().unwrap();
        let error = session
            .compile_fixture(std::path::Path::new("random-negative.ph"), source)
            .expect_err("invalid random client must fail");
        assert!(error.to_string().to_lowercase().contains(expected), "{error}");
    }
}

#[test]
fn random_sampling_preserves_numeric_policy_without_candidate_overshoot() {
    let source = r#"
from universe.random import Generator
class Script {
  mut _index: Int
  @constructor new() { _index = 0 }
  nextU64 -> Int {
    _index = _index + 1
    if _index % 2 == 1 { return 0xffffffffffffffff }
    17
  }
}
impl Generator for Script {}
let result = Script.new().nextInt(below: 1 << 69)
"#;
    let mut session = phalcom_core::modules::compile_session::ProgramCompilationSession::standalone().unwrap();
    let program = session.compile_fixture(std::path::Path::new("random-policy.ph"), source).unwrap();
    let mut vm = vm_support::universe_vm();
    vm.numeric_policy.max_integer_bits = Some(70);
    vm.run_compiled(&program)
        .expect("masked first chunk never builds an avoidable 128-bit candidate");
    let module = vm.module_registry.get(&program.entry).unwrap().object;
    assert_eq!(named(&vm, module, "result").unwrap().to_string(&vm), "571849066284996100113");

    let mut session = phalcom_core::modules::compile_session::ProgramCompilationSession::standalone().unwrap();
    let oversized = session
        .compile_fixture(
            std::path::Path::new("random-limit.ph"),
            "from universe.random import Xoshiro256PlusPlus\nlet result = Xoshiro256PlusPlus.seed(42).nextInt(below: 1 << 140)\n",
        )
        .unwrap();
    vm.numeric_policy.max_integer_bits = Some(128);
    let error = vm.run_compiled(&oversized).expect_err("oversized bound preserves ordinary numeric failure");
    assert!(error.to_string().contains("Numeric limit exceeded"), "{error}");
}

fn named(vm: &phalcom_core::vm::VM, module: phalcom_core::heap::ObjRef, name: &str) -> Option<Value> {
    let symbol = vm.interner.find(name)?;
    vm.heap.module(module).get(symbol)
}

#[test]
fn generic_trait_witness_transports_method_binders_exact_and_forwarded_evidence() {
    let source = r#"
trait Supplies {
  value -> Int
  label -> Int { self.value }
}
class Left {}
class Right {}
impl Supplies for Left { value -> Int { 3 } }
impl Supplies for Right { value -> Int { 7 } }
trait Combine {
  combine<A, B>(_ left: A, _ right: B) -> Int
    where A |= Supplies, B |= Supplies
}
class Mixer {}
impl Combine for Mixer {
  combine<X, Y>(_ left: X, _ right: Y) -> Int
    where Y |= Supplies, X |= Supplies
  {
    let leftValues = List<X>.new()
    leftValues.append(left)
    System.gc
    left.label * 10 + right.label + leftValues.size - 1
  }
}
class Caller {
  @class exact(_ mixer: Mixer) -> Int { mixer.combine(Left.new(), Right.new()) }
  @class forwarded<P, Q>(_ mixer: Mixer, _ left: P, _ right: Q) -> Int
    where P |= Supplies, Q |= Supplies
  { mixer.combine(left, right) }
}
let top = Mixer.new().combine(Left.new(), Right.new())
let member = Caller.exact(Mixer.new())
let forwarded = Caller.forwarded(Mixer.new(), Right.new(), Left.new())
"#;
    let analyzed = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("generic trait source analyzes");
    assert!(!analyzed.semantic.has_errors(), "{:#?}", analyzed.semantic.diagnostics);
    let program = ProgramCompiler::compile_analyzed(&analyzed).expect("canonical witness specialization compiles");
    let lowering = &program.modules[&program.entry].lowering;
    let selections: Vec<_> = lowering
        .trait_invocations
        .values()
        .filter(|spec| spec.selection.signature.selector.encode() == "combine(_,_)")
        .collect();
    assert_eq!(selections.len(), 3);
    for spec in selections {
        let recipe = spec
            .call_environment_recipe
            .as_ref()
            .expect("selected witness retains method call-entry recipe");
        assert_eq!(recipe.bindings.len(), 2);
        assert_eq!(recipe.conformance_slots.len(), 4);
        let selected = spec.selection.callable.as_ref().expect("source witness");
        let signature = analyzed.semantic.callable_signatures.get(selected).expect("canonical witness signature");
        let generic = signature.generics.as_ref().expect("witness method binders");
        for (parameter, _) in recipe.bindings.iter() {
            assert_eq!(
                parameter.owner,
                phalcom_type_meta::generic::StableTypeParameterOwnerRef::Callable(recipe.callable.clone())
            );
        }
        assert_eq!(generic.parameters.len(), recipe.bindings.len());
    }
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("exact and forwarded witnesses execute with method evidence");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "top").unwrap().to_string(&vm), "37");
    assert_eq!(named(&vm, module, "member").unwrap().to_string(&vm), "37");
    assert_eq!(named(&vm, module, "forwarded").unwrap().to_string(&vm), "73");
}

#[test]
fn trait_declaration_is_compile_time_only_and_surrounding_code_executes() {
    let source = r#"
trait Display {
  render() -> String { "trait" }
}

let result = "surrounding"
"#;
    let mut vm = vm_support::universe_vm();
    let class_count = vm.classes.len();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("trait source compiles");
    vm.run_compiled(&program).expect("trait-containing module executes");
    let entry_id = program.initialization_order.last().expect("entry module");
    let module = vm.module_registry.get(entry_id).expect("entry module registered").object;

    assert_eq!(vm.classes.len(), class_count, "trait must not allocate a runtime class");
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "surrounding");
    assert!(named(&vm, module, "Display").is_none(), "trait must not bind an ordinary runtime value");
}

#[test]
fn trait_default_is_not_injected_into_an_unrelated_runtime_class() {
    let source = r#"
trait Display {
  render() -> String { "trait" }
}

class Other {
  render() -> String { "other" }
}

let result = Other.new().render()
"#;
    let (vm, module) = vm_support::run_inline(source).expect("trait and unrelated class execute");
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "other");
}

#[test]
fn trait_evidenced_default_and_witness_execute_without_target_installation() {
    let source = r#"
trait Identified {
  name -> String
  label -> String { self.name }
}

class User {}
impl Identified for User { name -> String { "user" } }
class Caller { run(_ user: User) -> String { user.label } }
let result = Caller.new().run(User.new())
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("trait evidence compiles");
    let lowering = &program.modules[&program.entry].lowering;
    assert!(!lowering.trait_invocations.is_empty(), "ordinary trait call has lowering evidence");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("trait evidence executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "user");
    let user = named(&vm, module, "User").expect("User binding");
    let user_class = user.as_obj().expect("User class");
    let label = vm.interner.find("label").expect("label symbol");
    assert!(vm.heap.class(user_class).get_method(label).is_none(), "trait default stays detached");
}

#[test]
fn ordinary_trait_lowering_preserves_exact_and_derived_dispatch_evidence() {
    let source = r#"
trait ExactTag { value -> Int { 1 } }
class ExactReceiver {}
impl ExactTag for ExactReceiver {}

class Caller {
  inspect(_ exact: ExactReceiver) -> Int {
    let exactValue = exact.value
    let receiver = [1, 2].iterator
    let list = receiver.toList
    exactValue + list.size
  }
}
"#;
    let analyzed = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("canonical Universe source should analyze");
    assert!(!analyzed.semantic.has_errors(), "semantic diagnostics: {:#?}", analyzed.semantic.diagnostics);

    let find_selection = |occurrence: &str| {
        analyzed
            .semantic
            .callable_analyses
            .iter()
            .filter(|(callable, _)| callable.owner.module() == &analyzed.entry)
            .flat_map(|(_, analysis)| analysis.expressions.values())
            .find_map(|expression| {
                if source.get(expression.range.start..expression.range.end)? != occurrence {
                    return None;
                }
                match expression.trait_dispatch.as_ref()? {
                    phalcom_semantic::trait_dispatch::TraitDispatchSite::Evidenced(selection) => Some((expression.range, selection)),
                    _ => None,
                }
            })
            .unwrap_or_else(|| panic!("missing evidenced trait dispatch at {occurrence}"))
    };

    let (exact_range, exact_selection) = find_selection("exact.value");
    assert!(matches!(
        &exact_selection.evidence,
        phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(_)
    ));
    let (derived_range, derived_selection) = find_selection("receiver.toList");
    let phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(derived_evidence) = &derived_selection.evidence else {
        panic!("expected guarded Derived semantic evidence, got {:#?}", derived_selection.evidence);
    };
    assert!(matches!(
        &derived_evidence.origin,
        phalcom_semantic::generic_evidence::DerivedConformanceOrigin::GuardedRule { .. }
    ));

    let compiled = ProgramCompiler::compile_analyzed(&analyzed).expect("selected trait dispatches lower");
    let lowering = &compiled.modules[&analyzed.entry].lowering;
    for (range, expected_selection) in [(exact_range, exact_selection), (derived_range, derived_selection)] {
        let spec = lowering
            .trait_invocations
            .iter()
            .find_map(|(site, spec)| {
                (site.range == range && site.kind == phalcom_core::modules::semantic_lowering::LoweringSiteKind::TraitInvoke).then_some(spec)
            })
            .unwrap_or_else(|| panic!("missing TraitInvocationSpec at {range:?}"));
        assert_eq!(
            &spec.selection, expected_selection,
            "lowering must preserve the complete canonical selection and evidence"
        );
    }
}

#[test]
fn trait_evidenced_bound_reference_executes_detached_default() {
    let source = r#"
trait Identified {
  name -> String
  label -> String { self.name }
}

class User {}
impl Identified for User { name -> String { "user" } }
class Caller { run(_ user: User) -> String { let f = &user.label; f() } }
let result = Caller.new().run(User.new())
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("trait bound reference compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("trait bound reference executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "user");
}

#[test]
fn trait_evidenced_bound_reference_executes_conformance_witness() {
    let source = r#"
trait Marked { marker -> String }
class User {}
impl Marked for User { marker -> String { "witness-user" } }
class Caller { run(_ user: User) -> String { let f = &user.marker; f() } }
let result = Caller.new().run(User.new())
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("trait witness bound reference compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("trait witness bound reference executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "witness-user");
    let user = named(&vm, module, "User").expect("User binding").as_obj().expect("User class");
    let marker = vm.interner.find("marker").expect("marker symbol");
    assert!(
        vm.heap.class(user).get_method(marker).is_none(),
        "conformance witness must remain detached from the target class"
    );
}

#[test]
fn nested_trait_defaults_preserve_conformance_environment() {
    let source = r#"
trait Identified {
  name -> String
  label -> String { self.name }
  decorated -> String { "[\(self.label)]" }
}

class User {}
impl Identified for User { name -> String { "user" } }
class Caller { run(_ user: User) -> String { user.decorated } }
let result = Caller.new().run(User.new())
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("nested trait defaults compile");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("nested trait defaults execute");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "[user]");
}

#[test]
fn trait_default_executes_abstract_getter_satisfied_by_inherent_member() {
    let source = r#"
trait Named {
  name -> String
  label -> String { "<\(self.name)>" }
}

class User {
  name -> String { "inherent-user" }
}
impl Named for User {}
class Caller { run(_ user: User) -> String { user.label } }
let result = Caller.new().run(User.new())
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("inherent trait witness compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("inherent trait witness executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "<inherent-user>");
}

#[test]
fn trait_default_executes_abstract_getter_satisfied_by_inherited_member() {
    let source = r#"
trait Named {
  name -> String
  label -> String { "<\(self.name)>" }
}

class Entity {
  name -> String { "inherited-user" }
}
class User is Entity {}
impl Named for User {}
class Caller { run(_ user: User) -> String { user.label } }
let result = Caller.new().run(User.new())
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("inherited trait witness compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("inherited trait witness executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "<inherited-user>");
}

#[test]
fn generic_trait_default_dispatches_nested_abstract_method_calls() {
    let source = r#"
trait Scalable<T> {
  scale(_ value: T) -> T
  twice(_ value: T) -> T { self.scale(self.scale(value)) }
}

class Doubler {}
impl Scalable<Int> for Doubler {
  scale(_ value: Int) -> Int { value * 2 }
}
class Caller { run(_ value: Doubler) -> Int { value.twice(3) } }
let result = Caller.new().run(Doubler.new())
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("generic trait default compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("generic trait default executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "12");
}

#[test]
fn specialized_generic_targets_keep_distinct_trait_execution() {
    let source = r#"
trait Tagged { tag -> String }
class Value<T> { @constructor new() {} }
impl Tagged for Value<String> { tag -> String { "string" } }
impl Tagged for Value<Int> { tag -> String { "int" } }
class Caller {
  stringTag(_ value: Value<String>) -> String { value.tag }
  intTag(_ value: Value<Int>) -> String { value.tag }
}
let caller = Caller.new()
let stringValue: Value<String> = Value.new()
let intValue: Value<Int> = Value.new()
let stringResult = caller.stringTag(stringValue)
let intResult = caller.intTag(intValue)
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("specialized trait targets compile");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("specialized trait targets execute");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "stringResult").expect("string result").to_string(&vm), "string");
    assert_eq!(named(&vm, module, "intResult").expect("int result").to_string(&vm), "int");
}

#[test]
fn trait_default_uses_proven_conditional_inherent_witness() {
    let source = r#"
trait Tagged {
  tag -> String { "default" }
  label -> String { self.tag }
}
class NumericBase {}
class Numeric is NumericBase {}
class Value<T> { @constructor new() {} }
impl<T> Value<T> where T <: NumericBase { tag -> String { "number" } }
impl<T> Tagged for Value<T> {}
class Caller { run(_ value: Value<Numeric>) -> String { value.label } }
let value: Value<Numeric> = Value.new()
let result = Caller.new().run(value)
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("conditional inherent trait witness compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("conditional inherent trait witness executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "number");
}

#[test]
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
fn trait_default_executes_abstract_getter_satisfied_by_data_component() {
    let source = r#"
trait Named {
  name -> String
  label -> String { self.name }
}

data Person(name: String)
impl Named for Person {}
class Caller { run(_ person: Person) -> String { person.label } }
let result = Caller.new().run(Person(name: "data-user"))
"#;
    let analyzed = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("data-component trait source analyzes");
    assert!(!analyzed.semantic.has_errors(), "semantic diagnostics: {:#?}", analyzed.semantic.diagnostics);
    let person = phalcom_modules::DeclarationId::new(analyzed.entry.clone(), "Person".into());
    assert!(
        analyzed.semantic.declarations.form(&person).is_some(),
        "semantic snapshot must retain the Person declaration"
    );
    assert!(
        analyzed.semantic.data_semantics.data_info(&person).is_some(),
        "semantic snapshot must retain Person data semantics; declarations: {:#?}",
        analyzed.semantic.declarations
    );
    let program = ProgramCompiler::compile_analyzed(&analyzed).expect("data-component trait witness compiles");
    assert_eq!(
        program.modules[&program.entry].lowering.data_decls.len(),
        1,
        "formal module lowering must retain Person data semantics"
    );
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("data-component trait witness executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "data-user");
}
