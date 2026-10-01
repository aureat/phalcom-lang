use super::vm_support;
use phalcom_core::modules::compile::{EntrySelection, ProgramAnalyzer, ProgramCompiler};
use std::sync::Arc;

#[test]
fn range_iterator_take_infers_and_publishes_specialized_constructor_result() {
    let analyzed =
        ProgramAnalyzer::analyze_entry_selection(EntrySelection::Inline(Arc::from("let marker = 1"))).expect("canonical Universe source should analyze");
    let (take_callable, take_analysis) = analyzed
        .semantic
        .callable_analyses
        .iter()
        .find(|(callable, _)| {
            callable.try_declaration_owner().is_some_and(|owner| owner.name == "RangeIterator".into()) && callable.selector.encode() == "take(_)"
        })
        .expect("canonical RangeIterator.take body analysis");
    let constructor_call = take_analysis
        .expressions
        .values()
        .find(|expression| {
            expression.callable.as_ref().is_some_and(|callable| {
                callable
                    .try_declaration_owner()
                    .is_some_and(|owner| owner.module.path.to_string() == "collections.iterator_laws" && owner.name == "TakeIterator".into())
                    && callable.selector.encode() == "new(_,_)"
            })
        })
        .expect("RangeIterator.take must resolve TakeIterator.new through its source import");
    assert_eq!(constructor_call.status, phalcom_semantic::checker::analysis::AnalysisStatus::Ready);
    let constructor_callable = constructor_call.callable.as_ref().expect("resolved constructor identity");
    let specialization = constructor_call.call_specialization.as_ref().expect("generic constructor specialization");
    assert_eq!(&specialization.callable, constructor_callable);
    assert_eq!(specialization.bindings.len(), 1);
    let (_, iterator_type) = specialization.bindings[0];
    assert_eq!(analyzed.semantic.store.format_type(iterator_type), "RangeIterator");

    let constructor_result = constructor_call.knowledge.ty().expect("known specialized constructor result");
    assert_eq!(analyzed.semantic.store.format_type(constructor_result), "TakeIterator<RangeIterator>");
    let [normal_return] = &take_analysis.exits.normal_returns[..] else {
        panic!("expected exactly one normal return: {:#?}", take_analysis.exits.normal_returns);
    };
    assert_eq!(normal_return.status, phalcom_semantic::checker::analysis::AnalysisStatus::Ready);
    assert_eq!(normal_return.knowledge.ty(), Some(constructor_result));
    assert_eq!(normal_return.direct_result_dependency.as_ref(), Some(constructor_callable));
    let signature = analyzed
        .semantic
        .callable_signatures
        .get_for_body(take_callable)
        .expect("canonical RangeIterator.take signature");
    assert_eq!(
        signature.inferred_return.as_ref().and_then(phalcom_semantic::TypeKnowledge::ty),
        Some(constructor_result)
    );
}

#[test]
fn natural_range_iter_and_take_stages_are_known() {
    let source = r#"
class Caller {
  inspect -> Int {
    System.print((0..).iter.take(10).toList.size)
    0
  }
}
"#;
    let analyzed = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("canonical Universe source should analyze");
    assert!(
        analyzed
            .semantic
            .diagnostics
            .get(&analyzed.entry)
            .is_none_or(|diagnostics| diagnostics.is_empty()),
        "the inline entry itself should remain diagnostic-free"
    );
    let (iter_call, iter_selection) = analyzed
        .semantic
        .callable_analyses
        .iter()
        .filter(|(callable, _)| callable.owner.module() == &analyzed.entry)
        .flat_map(|(_, analysis)| analysis.expressions.values())
        .find_map(|expression| {
            (source.get(expression.range.start..expression.range.end) == Some("(0..).iter"))
                .then_some(expression)
                .and_then(|expression| match expression.trait_dispatch.as_ref()? {
                    phalcom_semantic::trait_dispatch::TraitDispatchSite::Evidenced(selection) => Some((expression, selection)),
                    _ => None,
                })
        })
        .expect("the range iterator call has an evidenced Iterable::iter selection");
    assert_eq!(iter_call.status, phalcom_semantic::checker::analysis::AnalysisStatus::Ready);
    let iter_result = iter_call.knowledge.ty().expect("Iterable::iter has a known result");
    assert_eq!(analyzed.semantic.store.format_type(iter_result), "RangeIterator");
    assert_eq!(iter_selection.requirement.owner.name, "Iterable".into());
    assert_eq!(iter_selection.requirement.selector.encode(), "iter");
    assert!(matches!(
        iter_selection.evidence,
        phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(_)
    ));
    let take_call = analyzed
        .semantic
        .callable_analyses
        .iter()
        .filter(|(callable, _)| callable.owner.module() == &analyzed.entry)
        .flat_map(|(_, analysis)| analysis.expressions.values())
        .find(|expression| {
            expression.callable.as_ref().is_some_and(|callable| {
                callable
                    .try_declaration_owner()
                    .is_some_and(|owner| owner.module.path.to_string() == "collections.range" && owner.name == "RangeIterator".into())
                    && callable.selector.encode() == "take(_)"
            })
        })
        .expect("natural range chain selects the inherent RangeIterator.take method");
    assert_eq!(take_call.status, phalcom_semantic::checker::analysis::AnalysisStatus::Ready);
    assert!(take_call.trait_dispatch.is_none(), "RangeIterator.take is an ordinary inherent selection");
    assert!(
        take_call.call_specialization.is_none(),
        "RangeIterator.take has no method-generic specialization"
    );
    let take_result = take_call.knowledge.ty().expect("RangeIterator.take has a known result");
    assert_eq!(analyzed.semantic.store.format_type(take_result), "TakeIterator<RangeIterator>");
    let take_callable = take_call.callable.as_ref().expect("canonical inherent callable identity");
    let take_signature = analyzed
        .semantic
        .callable_signatures
        .get_for_body(take_callable)
        .expect("canonical RangeIterator.take signature");
    assert_eq!(
        take_signature.inferred_return.as_ref().and_then(phalcom_semantic::TypeKnowledge::ty),
        Some(take_result)
    );
    assert!(matches!(
        take_signature.parameter_declared_type_at(0).map(|fact| &fact.state),
        Some(phalcom_semantic::declaration_type::DeclaredTypeState::Unknown(
            phalcom_semantic::types::evidence::UnknownReason::NoTypeEvidence
        ))
    ));
}

#[test]
fn tuple_iterator_preserves_specialized_element_type_through_both_protocols() {
    let source = r#"
let tuple: Tuple<***(Int, String)> = (1, "value")
let element: Tuple<***(Int, String)>::Element = tuple[0]
let elementFromUnion: Int | String = element
let next: Option<Tuple<***(Int, String)>::Element> = tuple.iterator.next
let iterated = 0
for value in tuple {
  let protocolItem: Tuple<***(Int, String)>::Element = value
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
fn unbounded_take_to_list_keeps_guarded_iterable_dispatch_evidence() {
    let source = r#"
class Caller {
  inspect -> Int {
    System.print((0..).iter.take(10).toList.size)
    0
  }
}
"#;
    let analyzed = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("canonical Universe source should analyze");
    assert!(!analyzed.semantic.has_errors(), "semantic diagnostics: {:#?}", analyzed.semantic.diagnostics);
    let (_callable, _callable_analysis, _text, selection) = analyzed
        .semantic
        .callable_analyses
        .iter()
        .filter(|(callable, _)| callable.owner.module() == &analyzed.entry)
        .find_map(|(callable, analysis)| {
            analysis.expressions.values().find_map(|expression| {
                let text = source.get(expression.range.start..expression.range.end)?;
                if !text.contains(".take(10).toList") {
                    return None;
                }
                let phalcom_semantic::trait_dispatch::TraitDispatchSite::Evidenced(selection) = expression.trait_dispatch.as_ref()? else {
                    return None;
                };
                if selection.requirement.selector.encode() != "toList" {
                    return None;
                }
                Some((callable, analysis, text, selection))
            })
        })
        .expect("the actual take(10).toList expression must publish evidenced trait dispatch");
    assert_eq!(selection.requirement.owner.name, "Iterable".into());
    let phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(derived) = &selection.evidence else {
        panic!("expected guarded Derived evidence for Iterable::toList, got {:#?}", selection.evidence);
    };
    assert!(matches!(
        &derived.origin,
        phalcom_semantic::generic_evidence::DerivedConformanceOrigin::GuardedRule { .. }
    ));
}

#[test]
fn known_list_iterator_source_receiver_gets_guarded_iterable_to_list_evidence() {
    let source = r#"
class Caller {
  inspect -> Int {
    let receiver = [1, 2].iterator
    receiver.toList.size
    0
  }
}
"#;
    let analyzed = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("canonical Universe source should analyze");
    assert!(!analyzed.semantic.has_errors(), "semantic diagnostics: {:#?}", analyzed.semantic.diagnostics);

    let (callable, callable_analysis, text, selection) = analyzed
        .semantic
        .callable_analyses
        .iter()
        .filter(|(callable, _)| callable.owner.module() == &analyzed.entry)
        .find_map(|(callable, analysis)| {
            analysis.expressions.values().find_map(|expression| {
                let text = source.get(expression.range.start..expression.range.end)?;
                if text != "receiver.toList" {
                    return None;
                }
                let phalcom_semantic::trait_dispatch::TraitDispatchSite::Evidenced(selection) = expression.trait_dispatch.as_ref()? else {
                    return None;
                };
                Some((callable, analysis, text, selection))
            })
        })
        .expect("the actual receiver.toList expression must publish evidenced trait dispatch");

    assert_eq!(selection.requirement.selector.encode(), "toList");
    assert_eq!(selection.requirement.owner.name, "Iterable".into());
    assert!(matches!(
        selection.selection,
        phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { .. }
    ));

    let phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(derived) = &selection.evidence else {
        panic!("expected canonical guarded Derived evidence, got {:#?}", selection.evidence);
    };
    let phalcom_semantic::generic_evidence::DerivedConformanceOrigin::GuardedRule {
        rule,
        exact_subject,
        exact_trait_ref,
    } = &derived.origin
    else {
        panic!("expected the guarded Iterator => Iterable rule, got {:#?}", derived.origin);
    };
    assert_eq!(rule.impl_id, selection.source_impl);
    assert_eq!(exact_trait_ref.declaration, selection.requirement.owner);
    assert_eq!(*exact_subject, selection.exact_target);
    assert!(matches!(
        derived.nested.as_ref(),
        [phalcom_semantic::generic_evidence::NestedGenericEvidence::Conformance(
            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(_)
        )]
    ));

    let dependencies = &callable_analysis.semantic_dependencies;
    assert!(
        dependencies.contains(&phalcom_semantic::checker::analysis::SemanticDependency::ConformanceDispatch(
            rule.impl_id.clone()
        ))
    );
    assert!(dependencies.contains(&phalcom_semantic::checker::analysis::SemanticDependency::TraitSurface(
        exact_trait_ref.declaration.clone()
    )));
    let nested_exact = match derived.nested.as_ref() {
        [
            phalcom_semantic::generic_evidence::NestedGenericEvidence::Conformance(phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(
                exact,
            )),
        ] => exact,
        other => panic!("expected exactly one nested exact Iterator proof, got {other:#?}"),
    };
    assert!(
        dependencies.contains(&phalcom_semantic::checker::analysis::SemanticDependency::ConformanceDispatch(
            nested_exact.source_impl.clone()
        ))
    );
    assert!(dependencies.contains(&phalcom_semantic::checker::analysis::SemanticDependency::TraitSurface(
        nested_exact.exact_trait_ref.declaration.clone()
    )));

    assert_eq!(text, "receiver.toList");
    let receiver_origin = match analyzed.semantic.store.get(selection.exact_target) {
        phalcom_semantic::types::TypeData::Applied { origin, .. } => *origin,
        other => panic!("expected a known applied ListIterator receiver, got {other:?}"),
    };
    let phalcom_semantic::types::TypeData::Nominal {
        declaration: receiver_declaration,
    } = analyzed.semantic.store.get(receiver_origin)
    else {
        panic!(
            "expected a nominal ListIterator origin, got {:#?}",
            analyzed.semantic.store.get(receiver_origin)
        );
    };
    assert_eq!(receiver_declaration.name, "ListIterator".into());
    assert!(callable.owner.module() == &analyzed.entry);
}

#[test]
fn known_non_iterator_source_receiver_does_not_get_derived_iterable_dispatch() {
    let source = r#"
class Caller {
  inspect -> Int {
    let receiver: Int = 1
    receiver.toList
    0
  }
}
"#;
    let analyzed = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("canonical Universe source should analyze");
    let expression = analyzed
        .semantic
        .callable_analyses
        .iter()
        .filter(|(callable, _)| callable.owner.module() == &analyzed.entry)
        .flat_map(|(_, analysis)| analysis.expressions.values())
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("receiver.toList"))
        .expect("the negative-control source expression is analyzed");
    assert!(
        !matches!(
            expression.trait_dispatch.as_ref(),
            Some(phalcom_semantic::trait_dispatch::TraitDispatchSite::Evidenced(selection))
                if selection.requirement.owner.name == "Iterable".into()
                    && selection.requirement.selector.encode() == "toList"
                    && matches!(selection.evidence, phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(_))
        ),
        "an ineligible known Int receiver must not acquire Derived Iterable::toList evidence: {expression:#?}"
    );
}

#[test]
fn iterator_take_publishes_known_constructor_result_with_untyped_count_formal() {
    let analyzed =
        ProgramAnalyzer::analyze_entry_selection(EntrySelection::Inline(Arc::from("let marker = 1"))).expect("canonical Universe source should analyze");

    let (take_callable, take_analysis) = analyzed
        .semantic
        .callable_analyses
        .iter()
        .find(|(callable, _)| callable.try_declaration_owner().is_some_and(|owner| owner.name == "Iterator".into()) && callable.selector.encode() == "take(_)")
        .expect("canonical Iterator.take body analysis");
    let constructor_call = take_analysis
        .expressions
        .values()
        .find(|expression| {
            expression.callable.as_ref().is_some_and(|callable| {
                callable.try_declaration_owner().is_some_and(|owner| owner.name == "TakeIterator".into()) && callable.selector.encode() == "new(_,_)"
            })
        })
        .expect("TakeIterator.new call in canonical Iterator.take body");
    assert!(
        matches!(constructor_call.knowledge, phalcom_semantic::TypeKnowledge::Known(_)),
        "known upstream should specialize TakeIterator: {constructor_call:#?}"
    );
    assert_eq!(
        constructor_call.status,
        phalcom_semantic::checker::analysis::AnalysisStatus::Ready,
        "untyped count formal must not manufacture a blocker: {constructor_call:#?}"
    );
    let constructor_result = constructor_call.knowledge.ty().expect("known constructor result");
    let phalcom_semantic::types::TypeData::Applied { origin, .. } = analyzed.semantic.store.get(constructor_result) else {
        panic!(
            "expected specialized TakeIterator application, got {:?}",
            analyzed.semantic.store.get(constructor_result)
        );
    };
    let phalcom_semantic::types::TypeData::Nominal { declaration } = analyzed.semantic.store.get(*origin) else {
        panic!("expected nominal TakeIterator origin, got {:?}", analyzed.semantic.store.get(*origin));
    };
    assert_eq!(declaration.name, "TakeIterator".into());

    let signature = analyzed
        .semantic
        .callable_signatures
        .get_for_body(take_callable)
        .expect("canonical Iterator.take signature");
    assert_eq!(
        signature.inferred_return.as_ref().and_then(phalcom_semantic::TypeKnowledge::ty),
        Some(constructor_result),
        "normal return publication should reuse the specialized constructor result: {signature:#?}"
    );
}

#[test]
fn tuple_and_record_raw_observers_keep_exact_types_and_public_options() {
    let source = r#"
let tuple: Tuple<***(Int, String)> = (7, "tail")
let first: Option<Int> = tuple.at(0)
let last: Option<String> = tuple.at(-1)
let firstMethod: Option<Int> = tuple.first
let lastMethod: Option<String> = tuple.last
let outOfRange: Option<Int> = tuple.at(2)
let indexed: String = tuple[1]
let emptyTuple = ()

let record: Record<**{ count: Int }> = { count: 7 }
let present: Option<Int> = record.get(#count)
let missing: Option<Int> = record.get(#absent)

let firstMatches = first.match(some: |value| { value == 7 }, none: || { false })
let lastMatches = last.match(some: |value| { value == "tail" }, none: || { false })
let firstMethodMatches = firstMethod.match(some: |value| { value == 7 }, none: || { false })
let lastMethodMatches = lastMethod.match(some: |value| { value == "tail" }, none: || { false })
let presentMatches = present.match(some: |value| { value == 7 }, none: || { false })
let result = firstMatches and lastMatches and firstMethodMatches and lastMethodMatches and outOfRange.isNone and indexed == "tail" and emptyTuple.first.isNone and emptyTuple.last.isNone and presentMatches and missing.isNone
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("exact Tuple/Record observer types compile");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("public Tuple/Record optional behavior executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "true");
}

#[test]
fn runtime_shaped_tuple_slice_and_zip_stay_behind_dynamic_boundary() {
    let source = r#"
let tuple: Tuple<***(Int, String, Bool)> = (1, "middle", true)
let slice = tuple[1..3]
let slicePreservesRuntimeShape = slice.size == 2 and slice[0] == "middle" and slice[1] == true

let zipped = ([1, 2], ["a", "b"]).zipped.iterator
let firstPair = zipped.next.match(
  some: |pair| { pair.size == 2 and pair[0] == 1 and pair[1] == "a" },
  none: || { false }
)
let secondPair = zipped.next.match(
  some: |pair| { pair.size == 2 and pair[0] == 2 and pair[1] == "b" },
  none: || { false }
)
let exhausted = zipped.next.isNone
let result = slicePreservesRuntimeShape and firstPair and secondPair and exhausted
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source)))
        .expect("runtime-shaped Tuple operations compile through their dynamic boundary");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("runtime-shaped Tuple operations execute");
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
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(list_for_uses_fresh_stateful_canonical_iterator_source)))
        .expect("canonical List iteration compiles");
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
fn iterable_to_map_preserves_all_distinct_entries() {
    let source = r#"
const entries = [
  Entry.new("a", 11),
  Entry.new("b", 22),
  Entry.new("c", 33),
  Entry.new("d", 44)
]
const map = entries.toMap.unwrap
let result = map.size == 4 and map["a"] == 11 and map["b"] == 22 and map["c"] == 33 and map["d"] == 44
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("toMap source compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("toMap preserves each entry");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    let symbol = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "true");
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
