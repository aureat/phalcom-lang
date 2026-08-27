from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"anchor not found in {path}: {old[:100]!r}")
    p.write_text(text.replace(old, new, 1))

editor = "phalcom-semantic/src/editor.rs"
replace_once(
    editor,
    "use phalcom_common::selector::Selector;",
    "use phalcom_common::selector::{Selector, SelectorBase, SelectorKind, SelectorSlot};",
)
replace_once(
    editor,
    """/// One visible lexical symbol and its canonical target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisibleSymbol {""",
    """/// Structural prefix of a call being written. This is intentionally
/// protocol-neutral: syntax recovery supplies only slots already present in
/// source, while semantic candidate selection remains compiler-owned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartialCallPattern {
    pub base: SelectorBase,
    pub kind: SelectorKind,
    pub written_slots: Arc<[SelectorSlot]>,
}

impl PartialCallPattern {
    pub fn from_selector_prefix(selector: &Selector) -> Self {
        Self {
            base: selector.base.clone(),
            kind: selector.kind,
            written_slots: Arc::from(selector.slots.to_vec()),
        }
    }
}

/// One visible lexical symbol and its canonical target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisibleSymbol {""",
)
replace_once(
    editor,
    """    /// Returns compiler-owned lexical bindings visible at a source position.
    pub fn visible_symbols_at(&self, module: &ModuleId, offset: usize) -> Vec<VisibleSymbol> {""",
    """    /// Returns canonical callable candidates compatible with the structural
    /// prefix already written at an incomplete call site. Exact dispatch for
    /// each candidate selector is rechecked from every receiver alternative,
    /// so overridden superclass members cannot become accidental candidates.
    pub fn callable_candidates(
        &self,
        receiver: &ResolvedReceiver,
        pattern: &PartialCallPattern,
        access: &AccessContext,
    ) -> Vec<CallableId> {
        let mut candidates = self
            .members_for_receiver(receiver, access)
            .into_iter()
            .filter_map(|member| match member.target {
                EditorMemberTarget::Callable(callable)
                    if callable.selector.base == pattern.base
                        && callable.selector.kind == pattern.kind
                        && callable.selector.slots.len() >= pattern.written_slots.len()
                        && callable
                            .selector
                            .slots
                            .iter()
                            .zip(pattern.written_slots.iter())
                            .all(|(candidate, written)| candidate == written) =>
                {
                    Some(callable)
                }
                _ => None,
            })
            .filter(|callable| {
                receiver.alternatives.iter().any(|alternative| {
                    let side = match alternative.mode {
                        ReceiverMode::Instance => crate::identity::DispatchSide::Instance,
                        ReceiverMode::Class => crate::identity::DispatchSide::Class,
                    };
                    self.snapshot
                        .dispatch
                        .resolve_callable_id(
                            self.snapshot.hierarchy.as_ref(),
                            &alternative.declaration,
                            side,
                            &callable.selector,
                        )
                        .as_ref()
                        == Some(callable)
                })
            })
            .collect::<Vec<_>>();
        candidates.sort();
        candidates.dedup();
        candidates
    }

    /// Returns compiler-owned lexical bindings visible at a source position.
    pub fn visible_symbols_at(&self, module: &ModuleId, offset: usize) -> Vec<VisibleSymbol> {""",
)

lib = "phalcom-semantic/src/lib.rs"
replace_once(
    lib,
    "pub use editor::{AccessContext, EditorMember, EditorMemberTarget, EditorSemanticQuery, ReceiverAlternative, ReceiverMode, ResolvedReceiver, VisibleSymbol};",
    "pub use editor::{\n    AccessContext, EditorMember, EditorMemberTarget, EditorSemanticQuery, PartialCallPattern, ReceiverAlternative, ReceiverMode, ResolvedReceiver,\n    VisibleSymbol,\n};",
)

backend = "phalcom-lsp/src/backend.rs"
replace_once(
    backend,
    """fn compiler_signature_for_call<'a>(
    compiler: &'a phalcom_semantic::SemanticSnapshot,
    module: &phalcom_modules::ModuleId,
    site: &signature_help::CallSite,
) -> Option<&'a phalcom_semantic::CallableSemanticSignature> {
    let callable = compiler
        .formal_fact_at(module, site.name_range.start)
        .and_then(|fact| match &fact.fact {
            phalcom_semantic::FormalFactRef::Callable(callable) | phalcom_semantic::FormalFactRef::Expression { callable, .. } => Some(callable),
            phalcom_semantic::FormalFactRef::Binding { .. } => None,
        })
        .or_else(|| {
            compiler
                .occurrence_at(module, site.name_range.start)
                .and_then(|occurrence| match occurrence.target {
                    Some(phalcom_semantic::SemanticTargetId::Callable(callable)) => Some(callable),
                    _ => None,
                })
        })?;
    compiler.callable_signatures().get(callable)
}
""",
    """fn compiler_signature_for_call<'a>(
    compiler: &'a phalcom_semantic::SemanticSnapshot,
    module: &phalcom_modules::ModuleId,
    site: &signature_help::CallSite,
) -> Option<&'a phalcom_semantic::CallableSemanticSignature> {
    let exact = compiler
        .formal_fact_at(module, site.name_range.start)
        .and_then(|fact| match &fact.fact {
            phalcom_semantic::FormalFactRef::Callable(callable) | phalcom_semantic::FormalFactRef::Expression { callable, .. } => Some(callable.clone()),
            phalcom_semantic::FormalFactRef::Binding { .. } => None,
        })
        .or_else(|| match compiler.editor().target_at(module, site.name_range.start) {
            Some(phalcom_semantic::SemanticTargetId::Callable(callable)) => Some(callable),
            _ => None,
        });
    if let Some(callable) = exact {
        return compiler.callable_signatures().get(&callable);
    }

    let receiver_range = site.receiver_range?;
    let receiver = compiler.editor().resolve_receiver_at(module, receiver_range)?;
    let selector = phalcom_common::selector::Selector::try_decode_exact(&site.selector).ok()?;
    let pattern = phalcom_semantic::PartialCallPattern::from_selector_prefix(&selector);
    let access = compiler.editor().access_context_at(module, site.name_range.start);
    let candidates = compiler.editor().callable_candidates(&receiver, &pattern, &access);
    let [callable] = candidates.as_slice() else {
        return None;
    };
    compiler.callable_signatures().get(callable)
}
""",
)

probe = Path("phalcom-semantic/tests/constructor_factory_probe.rs")
text = probe.read_text()
marker = "\nfn builtin_annotation_snapshot()"
if marker not in text:
    raise SystemExit("semantic test insertion marker missing")
new_test = r'''

#[test]
fn partial_call_candidates_are_selected_by_canonical_receiver_surface() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
class Service {
  compute(_ x: Int, label y: Int) -> Int { x }
  run() { self.compute(1) }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let snapshot = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program)).snapshot;
    let service = DeclarationId::new(module.clone(), "Service".into());
    let compute = CallableId::new(
        service,
        Selector::method(
            "compute",
            vec![SelectorSlot::Positional, SelectorSlot::Label("label".to_string())],
        )
        .unwrap(),
        DispatchSide::Instance,
    );
    let receiver_start = source.find("self.compute").expect("self receiver");
    let receiver = snapshot
        .editor()
        .resolve_receiver_at(
            &module,
            SourceRange {
                start: receiver_start,
                end: receiver_start + "self".len(),
            },
        )
        .expect("self receiver");
    let access = snapshot.editor().access_context_at(&module, receiver_start);
    let prefix = Selector::method("compute", Vec::new()).unwrap();
    let candidates = snapshot.editor().callable_candidates(
        &receiver,
        &phalcom_semantic::PartialCallPattern::from_selector_prefix(&prefix),
        &access,
    );
    assert_eq!(candidates, vec![compute], "empty written slot prefix must select the receiver's compatible canonical method");
}
'''
probe.write_text(text.replace(marker, new_test + marker, 1))

print("partial call signature query candidate applied")
