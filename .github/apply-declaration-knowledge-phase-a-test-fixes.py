from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text()
    if old not in text:
        if new in text:
            return
        raise SystemExit(f"anchor missing in {path}: {old[:120]!r}")
    if text.count(old) != 1:
        raise SystemExit(f"anchor not unique in {path}: {text.count(old)}")
    target.write_text(text.replace(old, new, 1))


# Fingerprint fixture now spells declaration requirement explicitly.
replace_once(
    "phalcom-semantic/tests/semantic/incremental/fingerprints.rs",
    "        return_type: TypeTerm::Canonical(TypeId(1)),",
    '''        declared_return: phalcom_semantic::DeclaredTypeFact::known(\n            TypeTerm::Canonical(TypeId(1)),\n            phalcom_semantic::DeclaredTypeBasis::SourceAnnotation,\n        ),\n        inferred_return: None,''',
)

# Presentation fixture uses canonical parameter identity and declared-type facts.
replace_once(
    "phalcom-semantic/tests/semantic/integration/presentation.rs",
    '''        parameters: vec![CallableParameterSemantic::new(0, "value", int.into())].into_boxed_slice(),\n        return_type: int.into(),''',
    '''        parameters: vec![CallableParameterSemantic::new(\n            phalcom_semantic::CallableParameterId::new(callable.clone(), 0),\n            "value",\n            phalcom_semantic::DeclaredTypeFact::known(\n                phalcom_semantic::types::TypeTerm::Canonical(int),\n                phalcom_semantic::DeclaredTypeBasis::SourceAnnotation,\n            ),\n        )]\n        .into_boxed_slice(),\n        declared_return: phalcom_semantic::DeclaredTypeFact::known(\n            phalcom_semantic::types::TypeTerm::Canonical(int),\n            phalcom_semantic::DeclaredTypeBasis::SourceAnnotation,\n        ),\n        inferred_return: None,''',
)

# Inferred factory result is now explicitly the published body result, separate from declaration constraint.
replace_once(
    "phalcom-semantic/tests/constructor_factory_probe.rs",
    '''    assert_eq!(\n        signature.return_type,\n        TypeTerm::Canonical(cell_ty),\n        "inferred factory signature must return CellNum"\n    );''',
    '''    assert_eq!(\n        signature.published_return_term(),\n        Some(TypeTerm::Canonical(cell_ty)),\n        "inferred factory signature must return CellNum"\n    );''',
)

print("phase A semantic tests migrated")
