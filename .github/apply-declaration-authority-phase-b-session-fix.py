from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
path = ROOT / "phalcom-semantic/src/session.rs"
text = path.read_text()

old = '''                    let mut analysis = crate::checker::body::analyze_callable_body_with_fields(
                        callable.clone(),
                        body,
                        range,
                        store,
                        hierarchy,
                        &scoped_resolver,
                        declarations,
                        dispatch,
                        module_id.clone(),
                        budget,
                        cancel,
                        Some(field_lifecycle),
                    );
'''
new = '''                    let signature_id = if callable_signatures.get(&callable).is_some() {
                        Some(callable.clone())
                    } else if callable.side == DispatchSide::Instance {
                        let class_side = CallableId::new(callable.owner.clone(), callable.selector.clone(), DispatchSide::Class);
                        callable_signatures.get(&class_side).is_some().then_some(class_side)
                    } else {
                        None
                    };
                    let declared_signature = signature_id.as_ref().and_then(|signature_id| {
                        callable_signatures
                            .get(signature_id)
                            .map(|signature| (signature_id, signature))
                    });
                    let mut analysis = crate::checker::body::analyze_callable_body_with_fields(
                        callable.clone(),
                        body,
                        range,
                        store,
                        hierarchy,
                        &scoped_resolver,
                        declarations,
                        dispatch,
                        declared_signature,
                        module_id.clone(),
                        budget,
                        cancel,
                        Some(field_lifecycle),
                    );
'''
if old not in text:
    raise SystemExit("fixed-point body reanalysis anchor not found")
path.write_text(text.replace(old, new, 1))
print("phase B fixed-point canonical signature wiring applied")
