//! Generic associated-family specialization ownership.

use super::super::support::analyze_adt;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::types::store::TypeData;

#[test]
fn generic_constructor_metadata_keeps_owner_and_payload_specialization() {
    let case = analyze_adt(
        r#"
enum Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}
"#,
    );
    let some = case.variant("Option", Selector::method("Some", [SelectorSlot::Positional]).expect("selector"));
    assert_eq!(some.id.owner.name.as_ref(), "Option");
    assert_eq!(some.fields.len(), 1);
    assert!(matches!(case.analysis.snapshot.store.get(some.result_type_template), TypeData::Applied { .. }));
}
