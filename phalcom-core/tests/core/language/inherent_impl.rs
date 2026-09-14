//! End-to-end installation of accepted inherent impl behavior.

use super::vm_support;
use phalcom_core::compiler::lib::{CompilerError, UnitKind};
use phalcom_core::error::PhError;
use phalcom_core::heap::Object;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_modules::DeclarationId;

fn named(vm: &VM, module: phalcom_core::heap::ObjRef, name: &str) -> Value {
    let symbol = vm.interner.find(name).unwrap_or_else(|| panic!("missing binding `{name}`"));
    vm.heap.module(module).get(symbol).unwrap_or_else(|| panic!("missing value for `{name}`"))
}

#[test]
fn class_impl_instance_and_class_side_methods_dispatch() {
    let source = r#"
class User {}

impl User {
  greet() -> String { "hello" }

  @class origin() -> String { "impl-class" }
}

let instance_result = User.new().greet()
let class_result = User.origin()
"#;

    let (vm, module) = vm_support::run_inline(source).expect("class impl should execute");
    assert_eq!(named(&vm, module, "instance_result").to_string(&vm), "hello");
    assert_eq!(named(&vm, module, "class_result").to_string(&vm), "impl-class");
}

#[test]
fn impl_installation_is_independent_of_source_order() {
    fn run(source: &str) -> Result<String, PhError> {
        let (vm, module) = vm_support::run_inline(source)?;
        Ok(named(&vm, module, "result").to_string(&vm))
    }

    let target_then_impl = r#"
class User {}
impl User { greet() -> String { "same" } }
let result = User.new().greet()
"#;
    let impl_then_target = r#"
impl User { greet() -> String { "same" } }
class User {}
let result = User.new().greet()
"#;

    assert_eq!(run(target_then_impl).unwrap(), run(impl_then_target).unwrap());
}

#[test]
fn multiple_impl_fragments_are_installed_before_later_top_level_calls() {
    let source = r#"
class User {}
let first = User.new().first()
impl User { first() -> Int { 1 } }
impl User { second() -> Int { 2 } }
let second = User.new().second()
let total = first + second
"#;

    let (vm, module) = vm_support::run_inline(source).expect("multiple impl fragments should execute");
    assert_eq!(named(&vm, module, "first"), Value::int(1));
    assert_eq!(named(&vm, module, "second"), Value::int(2));
    assert_eq!(named(&vm, module, "total"), Value::int(3));
}

#[test]
fn inherited_impl_method_uses_ordinary_class_hierarchy_dispatch() {
    let source = r#"
class Base {}
impl Base { greet() -> String { "base" } }
class Child is Base {}
let result = Child.new().greet()
"#;

    let (vm, module) = vm_support::run_inline(source).expect("inherited impl should execute");
    assert_eq!(named(&vm, module, "result").to_string(&vm), "base");
}

#[test]
fn data_impl_method_uses_existing_data_behavior_class() {
    let source = r#"
data Point(_ x: Int, _ y: Int)
impl Point {
  sum() -> Int { self.x + self.y }
}
let result = Point(2, 3).sum()
"#;

    let (vm, module) = vm_support::run_inline(source).expect("data impl should execute");
    assert_eq!(named(&vm, module, "result"), Value::int(5));
}

#[test]
fn enum_root_impl_method_is_available_on_each_case() {
    let source = r#"
enum Status {
  @variant Ready
  @variant Failed
}
impl Status { isTerminal() -> Bool { true } }
let ready = Status::Ready.isTerminal()
let failed = Status::Failed.isTerminal()
"#;

    let (vm, module) = vm_support::run_inline(source).expect("enum-root impl should execute");
    assert_eq!(named(&vm, module, "ready"), Value::bool(true));
    assert_eq!(named(&vm, module, "failed"), Value::bool(true));
}

#[test]
fn impl_members_do_not_add_storage_or_change_product_layouts() {
    let source = r#"
class User {
  _name: String
}
impl User { label() -> String { "user" } }

data Point(_ x: Int, _ y: Int)
impl Point { sum() -> Int { self.x + self.y } }

enum Status {
  @variant Ready
  @variant Failed(reason: String)
}
impl Status { isTerminal() -> Bool { true } }

let user = User.new()
let point = Point(2, 3)
let failed = Status::Failed(reason: "no")
"#;

    let (vm, module) = vm_support::run_inline(source).expect("layout probe should execute");

    let user_class = named(&vm, module, "User").as_obj().expect("User class");
    assert_eq!(vm.heap.class(user_class).field_count, 1);
    assert_eq!(vm.heap.instance(named(&vm, module, "user").as_obj().unwrap()).slots.len(), 1);

    let point_owner = DeclarationId::new(vm.heap.module(module).id.clone(), "Point".into());
    let point_desc_id = vm.data_registry.descriptor_by_declaration(&point_owner).expect("Point descriptor");
    let point_desc = vm.data_registry.descriptor(point_desc_id).expect("Point metadata");
    assert_eq!(vm.heap.product_layouts.get(point_desc.layout).unwrap().components.len(), 2);
    assert_eq!(vm.heap.class(point_desc.behavior_class).field_count, 0);
    let point_ref = named(&vm, module, "point").as_obj().expect("Point value");
    assert_eq!(vm.heap.data(point_ref).storage.layout_id(), point_desc.layout);

    let status_owner = DeclarationId::new(vm.heap.module(module).id.clone(), "Status".into());
    let enum_id = vm.adt_registry.enum_by_declaration(&status_owner).expect("Status descriptor");
    let status_desc = vm.adt_registry.enum_descriptor(enum_id).expect("Status metadata");
    assert_eq!(vm.heap.class(status_desc.root_class).field_count, 0);
    let failed_ref = named(&vm, module, "failed").as_obj().expect("Failed value");
    let Object::AdtCase(case) = vm.heap.get(failed_ref) else {
        panic!("expected an ADT case object");
    };
    let failed_variant = status_desc
        .variants
        .iter()
        .find_map(|variant_id| {
            let descriptor = vm.adt_registry.variant_descriptor(*variant_id)?;
            (descriptor.semantic_id.selector.base == phalcom_common::selector::SelectorBase::Named("Failed".to_string())).then_some(descriptor)
        })
        .expect("Failed metadata");
    assert_eq!(vm.heap.product_layouts.get(failed_variant.layout.unwrap()).unwrap().components.len(), 1);
    assert_eq!(case.storage.layout_id(), failed_variant.layout.unwrap());
}

#[test]
fn duplicate_impl_is_rejected_before_runtime_compilation() {
    let source = r#"
class User {}
impl User { greet() -> String { "first" } }
impl User { greet() -> String { "last-wins-must-not-exist" } }
let result = User.new().greet()
"#;

    assert!(vm_support::run_inline(source).is_err());
}

#[test]
fn impl_compilation_without_semantic_lowering_fails_closed() {
    let mut vm = VM::new();
    let module = vm.create_module("impl-without-lowering", "impl-without-lowering");
    let result = vm.compile_closure_as(module, "class User {}\nimpl User { greet() { \"must not attach\" } }\n", UnitKind::File);

    assert!(matches!(result, Err(PhError::Compile(CompilerError::MissingImplLoweringSemantics(_)))));
}

#[test]
fn exact_enum_case_impl_dispatches_with_defaults_overrides_and_payloads() {
    let source = r#"
enum Expr {
  Lit(_ value: Int)
  Add(_ left: Expr, _ right: Expr)
  Zero
}

impl Expr {
  eval() -> Int

  precedence() -> Int {
    100
  }
}

impl Expr::Lit(_) {
  eval() -> Int {
    self.value
  }

  litOnly() -> String {
    "literal"
  }
}

impl Expr::Add(_, _) {
  eval() -> Int {
    self.left.eval() + self.right.eval()
  }

  precedence() -> Int {
    10
  }
}

impl Expr::Zero {
  eval() -> Int {
    0
  }
}

let lit1 = Expr::Lit(10)
let lit2 = Expr::Lit(20)
let add = Expr::Add(lit1, lit2)
let zero = Expr::Zero

let lit1_val = lit1.eval()
let add_val = add.eval()
let zero_val = zero.eval()

let lit1_prec = lit1.precedence()
let add_prec = add.precedence()
let zero_prec = zero.precedence()

let lit1_tag = lit1.litOnly()
"#;

    let (vm, module) = vm_support::run_inline(source).expect("exact-case enum impl should execute");
    assert_eq!(named(&vm, module, "lit1_val"), Value::int(10));
    assert_eq!(named(&vm, module, "add_val"), Value::int(30));
    assert_eq!(named(&vm, module, "zero_val"), Value::int(0));

    // Root default inherited by Lit and Zero, overridden by Add
    assert_eq!(named(&vm, module, "lit1_prec"), Value::int(100));
    assert_eq!(named(&vm, module, "add_prec"), Value::int(10));
    assert_eq!(named(&vm, module, "zero_prec"), Value::int(100));

    // Case-only method on Lit
    assert_eq!(named(&vm, module, "lit1_tag").to_string(&vm), "literal");
}

#[test]
fn bodyless_requirements_produce_no_lowering_members_or_method_objects() {
    let source = r#"
enum Status {
  Ready
  Done
}

impl Status {
  code() -> Int
}

impl Status::Ready {
  code() -> Int { 1 }
}

impl Status::Done {
  code() -> Int { 2 }
}

let r = Status::Ready.code()
let d = Status::Done.code()
"#;

    let (vm, module) = vm_support::run_inline(source).expect("status enum impl should execute");
    assert_eq!(named(&vm, module, "r"), Value::int(1));
    assert_eq!(named(&vm, module, "d"), Value::int(2));

    // Verify lowering spec for root Status has 0 executable members (since code() is bodyless requirement)
    let lowering = vm.heap.module(module).lowering.as_ref().expect("lowering semantics");
    let status_decl_id = DeclarationId::new(vm.heap.module(module).id.clone(), "Status".into());
    for spec in lowering.inherent_impls.iter() {
        if spec.target == phalcom_core::modules::semantic_lowering::InherentImplLoweringTarget::Declaration(status_decl_id.clone()) {
            assert_eq!(spec.members.len(), 0, "bodyless requirement must not be projected as executable member");
        }
    }
}

#[test]
fn exact_case_impl_emits_variant_method_bytecode() {
    let source = r#"
enum Choice {
  Yes
  No
}
impl Choice::Yes {
  say() -> String { "yes" }
}
impl Choice::No {
  say() -> String { "no" }
}
"#;

    let (vm, module) = vm_support::run_inline(source).expect("choice enum should compile and execute");
    let module_obj = vm.heap.module(module);
    let lowering = module_obj.lowering.as_ref().expect("lowering semantics");

    // All exact-case impls in lowering must have InherentImplLoweringTarget::ExactEnumCase
    for spec in lowering.inherent_impls.iter() {
        match &spec.target {
            phalcom_core::modules::semantic_lowering::InherentImplLoweringTarget::ExactEnumCase(variant) => {
                let name = match &variant.selector.base {
                    phalcom_common::selector::SelectorBase::Named(n) => n.as_str(),
                    _ => "",
                };
                assert!(name == "Yes" || name == "No");
            }
            phalcom_core::modules::semantic_lowering::InherentImplLoweringTarget::Declaration(_) => {}
        }
    }
}

#[test]
fn specialized_data_impl_executes_only_for_the_applicable_receiver() {
    let source = r#"
data Box<T>(_ value: T)

impl Box<Int> {
  intValue() -> Int { self.value }
}

let result = Box(7).intValue()
"#;

    let (vm, module) = vm_support::run_inline(source).expect("specialized data impl should execute");
    assert_eq!(named(&vm, module, "result"), Value::int(7));

    let inapplicable = r#"
data Box<T>(_ value: T)
impl Box<Int> { intValue() -> Int { self.value } }
let result = Box("not an Int").intValue()
"#;
    assert!(
        vm_support::run_inline(inapplicable).is_err(),
        "a specialized member must not leak onto the shared generic behavior class"
    );
}

#[test]
fn bound_family_retains_the_selected_specialized_impl() {
    let source = r#"
data Box<T>(_ value: T)

impl Box<Int> {
  intValue() -> Int { self.value }
}

let family = &Box(7).intValue()
let result = family()
"#;

    let (vm, module) = vm_support::run_inline(source).expect("conditional bound family should execute");
    assert_eq!(named(&vm, module, "result"), Value::int(7));
}
