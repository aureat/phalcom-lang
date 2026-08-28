from pathlib import Path
import re


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old in text:
        return text.replace(old, new, 1)
    if new in text:
        return text
    raise SystemExit(f"{label} shape changed")


# First run the existing Phase 4 authority transformation with a structural
# postcondition instead of the stale raw occurrence count.
phase4_path = Path(".github/scripts/canonical_callable_phase4.py")
phase4 = phase4_path.read_text()
old_guard = '''if text.count("callable_signatures.get_for_body") < 4:
    raise SystemExit("canonical body-signature lookup not installed at all intended authority sites")
'''
new_guard = '''required_after = {
    "field lifecycle constructor authority": "callable_signatures\\n                                                            .get_for_body(&analysis.callable)",
    "advisory formal return authority": "let fact = callable_signatures\\n            .get_for_body(&analysis.callable)",
    "inferred return authority": "let signature = callable_signatures.get_for_body(callable)?;",
    "body recheck authority": "let declared_signature = callable_signatures\\n                        .get_for_body(&callable)",
}
for label, snippet in required_after.items():
    if snippet not in text:
        raise SystemExit(f"missing canonical authority site: {label}")
'''
phase4 = replace_once(phase4, old_guard, new_guard, "Phase 4 structural guard")
phase4_path.write_text(phase4)
exec(compile(phase4, str(phase4_path), "exec"), {"__name__": "__main__"})

# The root allocator declaration was historically synthesized only into
# dispatch. Canonical authority therefore needs the declaration retained in the
# signature table and dispatch rebuilt as a projection of that declaration.
sig_path = Path("phalcom-semantic/src/checker/declaration_signature.rs")
sig = sig_path.read_text()
anchor = "pub(crate) fn semantic_signature_for_member(ctx: &mut CheckingContext<'_>, owner: &DeclarationId, member: &ClassMember) -> Option<CallableSemanticSignature> {\n"
helper = '''/// Canonical declaration for the root `Class.new()` allocator behavior.
///
/// The member is inherited by class objects through ordinary instance-side
/// dispatch on `Class`; its constructor result is receiver-specialized `Self`.
/// Standalone checker contexts project this declaration into dispatch, while
/// workspace sessions also retain it in `CallableSignatureTable`.
pub(crate) fn canonical_core_class_new_signature(store: &mut crate::types::store::TypeStore) -> CallableSemanticSignature {
    let owner = DeclarationId::new(crate::identity::ModuleId::core(), "Class".into());
    let selector = Selector::method("new", Vec::new()).expect("root Class.new selector must be valid");
    let callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);
    let self_type = store.self_type(crate::types::parameter::SelfTypeTerm {
        owner: owner.clone(),
        side: DispatchSide::Instance,
        role: crate::types::parameter::SelfRole::InstanceType,
    });
    let knowledge = TypeKnowledge::established(self_type, EvidenceOrigin::ConstructorSemantics);
    CallableSemanticSignature {
        callable,
        owner,
        side: DispatchSide::Instance,
        selector,
        generics: None,
        parameters: Vec::<CallableParameterSemantic>::new().into_boxed_slice(),
        declared_return: DeclaredTypeFact::from_knowledge_with_basis(&knowledge, DeclaredTypeBasis::ConstructorSemantics),
        inferred_return: None,
        source: None,
        implementation: phalcom_native_meta::ImplementationKind::Generated,
        native_id: None,
        effects: phalcom_native_meta::EffectSpec::Unknown,
        raises: phalcom_native_meta::RaisesSpec::Unknown,
        flow: phalcom_native_meta::ReturnFlowSpec::Value,
        lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
    }
}

''' + anchor
sig = replace_once(sig, anchor, helper, "core Class.new canonical helper")
sig_path.write_text(sig)

ctx_path = Path("phalcom-semantic/src/checker/context.rs")
ctx = ctx_path.read_text()
new_block = '''    let canonical_new = crate::checker::declaration_signature::canonical_core_class_new_signature(store);
    if class_surface.instance.get_callable(&canonical_new.selector).is_none() {
        class_surface.add_callable(
            DispatchSide::Instance,
            crate::checker::declaration_signature::project_semantic_signature(&canonical_new),
        );
    }
'''
if "canonical_core_class_new_signature(store)" not in ctx:
    pattern = re.compile(
        r'    if let Ok\(selector\) = Selector::method\("new", Vec::new\(\)\)\n'
        r'        && class_surface\.instance\.get_callable\(&selector\)\.is_none\(\)\n'
        r'    \{\n.*?'
        r'        class_surface\.add_callable\(DispatchSide::Instance, signature\);\n'
        r'    \}\n',
        re.S,
    )
    ctx, count = pattern.subn(new_block, ctx, count=1)
    if count != 1:
        raise SystemExit(f"core Class.new dispatch projection match count={count}")
ctx_path.write_text(ctx)

session_path = Path("phalcom-semantic/src/session.rs")
session = session_path.read_text()
old_seed = '''        let mut base_callable_signatures = CallableSignatureTable::new();
        for (_, signature) in native_report.callable_signatures {
            base_callable_signatures.insert(signature);
        }
'''
new_seed = old_seed + '''        let core_class_new = crate::checker::declaration_signature::canonical_core_class_new_signature(&mut store);
        if base_callable_signatures.get(&core_class_new.callable).is_none() {
            base_callable_signatures.insert(core_class_new);
        }
'''
session = replace_once(session, old_seed, new_seed, "workspace core Class.new canonical seed")
session = session.replace(", advisory_fact_from_formal,", ",")
session = session.replace("use crate::types::evidence::TypeKnowledge;\n", "")
session_path.write_text(session)

for forbidden in (
    'dispatch.get_surface(&callable.owner)?.get_callable(callable.side, &callable.selector)?',
    'let surface = dispatch.surfaces().get(&callable.owner)?;',
):
    if forbidden in session:
        raise SystemExit(f"reverse callable authority remains after Phase 4: {forbidden}")

if "canonical_core_class_new_signature(&mut store)" not in session:
    raise SystemExit("root Class.new is not retained in canonical callable signatures")
