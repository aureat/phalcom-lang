from pathlib import Path


def replace_once(path: str, old: str, new: str, label: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label} anchor count: {count}")
    file.write_text(text.replace(old, new, 1))


# ConformanceIndex keeps query_exact exact. Ordinary trait discovery instead
# specializes one already-bucketed source conformance from the exact target.
replace_once(
    "phalcom-semantic/src/impls.rs",
    """    /// Returns pairs of eligible source heads whose joint target and trait
    /// patterns overlap. No precedence or specialization rule is applied.""",
    """    /// Instantiates one already-selected source conformance from an exact
    /// target. This is the bounded discovery seam used by ordinary trait
    /// dispatch when the source head itself is generic: target matching binds
    /// impl parameters first, then those bindings specialize the source TraitRef.
    ///
    /// Unlike `query_exact`, this does not accept an already-exact TraitRef and
    /// does not scan other conformances. The caller must have selected `impl_id`
    /// through the trait-dispatch family bucket.
    pub fn instantiate_source_for_target(
        &self,
        store: &mut TypeStore,
        impl_id: &ImplId,
        target: TypeId,
    ) -> Option<ConformanceHeadMatch> {
        let contribution = self.contributions.get(impl_id)?;
        if !contribution.is_lookup_eligible() {
            return None;
        }

        let mut bindings = HashMap::new();
        if !match_impl_domain_head(store, contribution.target_head, target, impl_id, &mut bindings) {
            return None;
        }

        let substitution = bindings.iter().fold(TypeSubstitution::new(), |mut substitution, (&parameter, &ty)| {
            substitution.bind(parameter, ty);
            substitution
        });
        let exact_trait_ref = TraitRef::new(
            contribution.trait_ref.declaration.clone(),
            contribution
                .trait_ref
                .arguments
                .iter()
                .map(|&argument| substitution.apply(store, argument))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );

        Some(ConformanceHeadMatch {
            impl_id: impl_id.clone(),
            exact_target: target,
            exact_trait_ref,
            impl_bindings: bindings,
        })
    }

    /// Returns pairs of eligible source heads whose joint target and trait
    /// patterns overlap. No precedence or specialization rule is applied.""",
    "source conformance target specialization",
)

replace_once(
    "phalcom-semantic/src/trait_dispatch.rs",
    """    for contribution in index.candidates_for(&target_family, selector, side) {
        let Some(source) = conformance_index.get(&contribution.impl_id) else {
            continue;
        };
        let matches = conformance_index.query_exact(store, receiver, &source.trait_ref);
        let Some(head) = matches.into_iter().find(|head| head.impl_id == contribution.impl_id) else {
            continue;
        };""",
    """    for contribution in index.candidates_for(&target_family, selector, side) {
        let Some(head) = conformance_index.instantiate_source_for_target(store, &contribution.impl_id, receiver) else {
            continue;
        };""",
    "trait dispatch generic source discovery",
)

# Semantic regression: a generic source conformance must become an exact
# TraitRef from the exact receiver before P2 evidence resolution.
replace_once(
    "phalcom-semantic/tests/semantic/impls/queries.rs",
    "use phalcom_common::selector::Selector;",
    "use phalcom_common::selector::{Selector, SelectorSlot};",
    "selector slot import",
)
replace_once(
    "phalcom-semantic/tests/semantic/impls/queries.rs",
    """#[test]
fn p1_generic_specialization_matrix_and_iterable_head_stay_at_head_level() {""",
    """#[test]
fn trait_dispatch_specializes_generic_source_conformance_from_exact_target() {
    let module = test_module();
    let source = \"trait Echo<T> { echo(_ value: T) -> T }\\nclass Value<T> {}\\nclass Marker {}\\nimpl<T> Echo<T> for Value<T> { echo(_ value: T) -> T { value } }\\n\";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), \"diagnostics: {:?}\", output.snapshot.diagnostics);

    let value_decl = DeclarationId::new(module.clone(), \"Value\".into());
    let marker_decl = DeclarationId::new(module.clone(), \"Marker\".into());
    let value_form = output.snapshot.declarations.form(&value_decl).expect(\"Value form\");
    let marker_form = output.snapshot.declarations.form(&marker_decl).expect(\"Marker form\");
    let mut store = (*output.snapshot.store).clone();
    let value_marker = store.apply_type_form(value_form, &[marker_form]).expect(\"Value<Marker>\");
    let selector = Selector::method(\"echo\", vec![SelectorSlot::Positional]).expect(\"echo selector\");

    let resolution = phalcom_semantic::trait_dispatch::resolve_trait_evidenced_candidates(
        &output.snapshot.trait_dispatch_index,
        &output.snapshot.conformance_index,
        &output.snapshot.conformance_witness_plans,
        &output.snapshot.trait_surfaces,
        &output.snapshot.declarations,
        &mut store,
        &output.snapshot.hierarchy,
        value_marker,
        &selector,
        DispatchSide::Instance,
    );
    let phalcom_semantic::trait_dispatch::TraitDispatchResolution::Found(selection) = resolution else {
        panic!(\"generic source conformance must resolve to exact trait evidence: {resolution:?}\");
    };
    assert_eq!(selection.exact_target, value_marker);
    assert_eq!(selection.exact_trait_ref.declaration, DeclarationId::new(module, \"Echo\".into()));
    assert_eq!(selection.exact_trait_ref.arguments.as_ref(), &[marker_form]);
}

#[test]
fn p1_generic_specialization_matrix_and_iterable_head_stay_at_head_level() {""",
    "generic source semantic regression",
)
