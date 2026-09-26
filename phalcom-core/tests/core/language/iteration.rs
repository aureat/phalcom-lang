use super::vm_support;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use std::sync::Arc;

#[test]
fn tuple_iterator_preserves_specialized_element_type_through_both_protocols() {
    let source = r#"
let tuple: Tuple<***(Int, String)> = (1, "value")
let element: Tuple<***(Int, String)>::Element = tuple[0]
let elementFromUnion: Int | String = element
let next: Option<Int | String> = tuple.iterator.next
let iterated = 0
for value in tuple {
  let protocolItem: Int | String = value
  iterated = iterated + 1
}
let result = next.isSome and iterated == 2
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("specialized Tuple iterator types compile");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("specialized Tuple iterator executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "true");
}

#[test]
fn list_for_uses_fresh_stateful_canonical_iterator() {
let list_for_uses_fresh_stateful_canonical_iterator_source = r#"
const values = [1, 2, 3]
const first = values.iterator
let firstOne = first.next
let firstTwo = first.next
let firstThree = first.next
let firstDone = first.next
let firstDoneAgain = first.next
const fresh = values.iterator
let freshOne = fresh.next
let sum = 0
for value in values {
  sum = sum + value
}
let result = sum == 6 and firstOne.isSome and firstTwo.isSome and firstThree.isSome and firstDone.isNone and firstDoneAgain.isNone and freshOne.isSome
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(list_for_uses_fresh_stateful_canonical_iterator_source))).expect("canonical List iteration compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("canonical List iteration executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "true");
}

#[test]
fn canonical_list_for_preserves_break_and_continue_control() {
    let source = r#"
let sum = 0
for value in [1, 2, 3, 4] {
  if (value == 2) { continue }
  sum = sum + value
  if (value == 3) { break }
}
let result = sum
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("canonical List control compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("canonical List control executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "4");
}

#[test]
fn canonical_parallel_lanes_advance_in_lockstep() {
    let source = r#"
let total = 0
for left at leftIndex in [1, 2], right at rightIndex in [10, 20] {
  total = total + left + right + leftIndex + rightIndex
}
let result = total
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("canonical parallel iteration compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("canonical parallel iteration executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "35");
}

#[test]
fn canonical_parallel_lanes_raise_on_length_mismatch() {
    let source = r#"
for left in [1, 2], right in [10] {
  let ignored = left + right
}
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("canonical strict zip compiles");
    let mut vm = vm_support::universe_vm();
    let error = vm.run_compiled(&program).expect_err("mismatched canonical lanes must raise");
    let rendered = format!("{error:?}");
    assert!(
        rendered.contains("strict zip lanes ended at different times"),
        "unexpected strict zip error: {rendered}"
    );
}

#[test]
fn canonical_single_lane_preserves_pattern_and_ordinal_bindings() {
    let source = r#"
let total = 0
for (left, right) at index in [(1, 10), (2, 20)] {
  total = total + left + right + index
}
let result = total
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("canonical pattern iteration compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("canonical pattern iteration executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "34");
}

#[test]
fn applied_set_and_map_constructors_execute_as_ordinary_class_object_sends() {
    let source = r#"
let values = Set<Int>.new()
let mapping = Map<Int, String>.new()
let result = values.size == 0 and mapping.size == 0
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("applied Set and Map construction compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("applied Set and Map construction execute");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "true");
}

#[test]
fn applied_box_constructor_executes_as_its_nominal_class_object() {
    let source = r#"
class Box<T> {
  @constructor
  new() {}
}
let value = Box<Int>.new()
let result = value.is(Box)
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("applied Box construction compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("applied Box construction executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "true");
}

#[test]
fn canonical_set_iterator_is_fresh_sticky_and_ordered() {
let source = r#"
const values = Set<Int>.new()
values.add(3).add(1).add(2)
const it = values.iterator
let directFirst = it.next.isSome
let directSecond = it.next.isSome
let directThird = it.next.isSome
let done = it.next
let doneAgain = it.next
const fresh = values.iterator
let freshFirst = fresh.next.isSome
let sum = 0
let count = 0
for value in values {
  sum = sum + value
  count = count + 1
}
let result = sum == 6 and count == 3 and directFirst and directSecond and directThird and done.isNone and doneAgain.isNone and freshFirst
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("canonical Set iteration compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("canonical Set iteration executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "true");
}

#[test]
fn canonical_bytes_and_string_views_iterate_their_stateful_surfaces() {
    let source = r#"
const bytes = Bytes.fromList([7, 8, 9])
const bytesIterator = bytes.iterator
let directFirst = bytesIterator.next.isSome
let directSecond = bytesIterator.next.isSome
let directThird = bytesIterator.next.isSome
let byteDone = bytesIterator.next
const text = "a€"
const byteView = text.bytes.iterator
const codePointView = text.codePoints.iterator
let viewFirst = byteView.next.isSome
let viewSecond = byteView.next.isSome
let viewThird = byteView.next.isSome
let viewFourth = byteView.next.isSome
let viewDone = byteView.next
let codePointFirst = codePointView.next.isSome
let codePointSecond = codePointView.next.isSome
let codePointDone = codePointView.next
let byteSum = 0
for value in bytes { byteSum = byteSum + value }
let result = byteSum == 24 and directFirst and directSecond and directThird and byteDone.isNone and text.bytes.size == 4 and viewFirst and viewSecond and viewThird and viewFourth and viewDone.isNone and text.codePoints.size == 2 and codePointFirst and codePointSecond and codePointDone.isNone
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("canonical Bytes and String iteration compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("canonical Bytes and String iteration executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "true");
}

#[test]
fn canonical_range_for_publishes_semantic_iteration_lane() {
    let source = r#"
let sum = 0
for value in (1..4) { sum = sum + value }
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("canonical Range iteration compiles");
    assert_eq!(
        program.modules.len(),
        1,
        "semantic support modules must not be rematerialized into the Universe VM"
    );
    assert_eq!(program.initialization_order, vec![program.entry.clone()]);
    assert_eq!(
        program.modules[&program.entry].lowering.for_iteration_lanes.len(),
        1,
        "Range for-loop must use the semantic canonical Iterator/Iterable lane"
    );
}

#[test]
fn canonical_map_iterator_and_views_yield_labeled_entries() {
    let source = r#"
let values: Map<String, Int> = Map.new()
values.at("a", put: 10).at("b", put: 20)
let entries = values.iterator
let directFirst = entries.next.isSome
let directSecond = entries.next.isSome
let done = entries.next
let keyView = values.keys.iterator
let valueView = values.values.iterator
let entryView = values.entries.iterator
let keyFirst = keyView.next.isSome
let valueFirst = valueView.next.isSome
let viewFirst = entryView.next.isSome
let count = 0
let keySum = 0
let valueSum = 0
let entrySum = 0
for entry in values {
  count = count + 1
  if (entry.key == "a") { keySum = keySum + 1 }
  valueSum = valueSum + entry.value
}
for key in values.keys { if (key == "a") { keySum = keySum + 10 } }
for value in values.values { valueSum = valueSum + value }
for entry in values.entries { entrySum = entrySum + entry.value }
let result = count == 2 and directFirst and directSecond and done.isNone and keyFirst and valueFirst and viewFirst and keySum == 11 and valueSum == 60 and entrySum == 30
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("canonical Map iteration compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("canonical Map iteration executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "true");
}

#[test]
fn canonical_range_iterator_is_stateful_and_respects_bounds() {
    let source = r#"
let exclusive = (1..4).iterator
let inclusive = (1..=3).iterator
let directFirst = exclusive.next.isSome
let directSecond = exclusive.next.isSome
let directThird = exclusive.next.isSome
let directDone = exclusive.next
let inclusiveFirst = inclusive.next.isSome
let inclusiveSecond = inclusive.next.isSome
let inclusiveThird = inclusive.next.isSome
let inclusiveDone = inclusive.next
let exclusiveSum = 0
for value in (1..4) { exclusiveSum = exclusiveSum + value }
let inclusiveSum = 0
for value in (1..=3) { inclusiveSum = inclusiveSum + value }
let result = exclusiveSum == 6 and inclusiveSum == 6 and directFirst and directSecond and directThird and directDone.isNone and inclusiveFirst and inclusiveSecond and inclusiveThird and inclusiveDone.isNone
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("canonical Range iteration compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("canonical Range iteration executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "true");
}

#[test]
fn canonical_set_iterator_direct_path_is_fresh_sticky_and_ordered() {
    let source = r#"
let values = Set<Int>.new()
values.add(3).add(1).add(2)
let it = values.iterator
let directFirst = it.next.isSome
let directSecond = it.next.isSome
let directThird = it.next.isSome
let done = it.next
let doneAgain = it.next
let fresh = values.iterator
let freshFirst = fresh.next.isSome
let result = directFirst and directSecond and directThird and done.isNone and doneAgain.isNone and freshFirst
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("direct canonical Set iteration compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("direct canonical Set iteration executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "true");
}

#[test]
fn canonical_map_iterator_direct_path_exposes_entries_and_views() {
    let source = r#"
let values: Map<String, Int> = Map.new()
values.at("a", put: 10).at("b", put: 20)
let entries = values.iterator
let directFirst = entries.next.isSome
let directSecond = entries.next.isSome
let done = entries.next
let keyView = values.keys.iterator
let valueView = values.values.iterator
let entryView = values.entries.iterator
let keyFirst = keyView.next.isSome
let valueFirst = valueView.next.isSome
let viewFirst = entryView.next.isSome
let result = directFirst and directSecond and done.isNone and keyFirst and valueFirst and viewFirst
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("direct canonical Map iteration compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("direct canonical Map iteration executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "true");
}
