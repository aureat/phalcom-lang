//! Tests for trait generic contracts, derived conformance, and associated projection normalization.

use crate::semantic::support::Fixture;
use phalcom_semantic::DiagnosticCode;
use phalcom_semantic::types::parameter::{GenericConstraint, TypeTerm};
use phalcom_semantic::types::store::TypeData;

#[test]
fn trait_contract_where_self_forms() {
    let fixture = Fixture::new(
        r#"
trait Base {
    type Item
}

trait Sub
    where Self |= Base
{
    type Item
}
"#,
    );

    let sub_decl = fixture.decl("Sub");
    let base_decl = fixture.decl("Base");
    let sub_info = fixture.analysis.snapshot.trait_headers.get(&sub_decl).expect("sub trait header");
    let generics = sub_info.generic_signature.as_ref().expect("generics on Sub");

    assert_eq!(generics.constraints.len(), 1);
    match &generics.constraints[0] {
        GenericConstraint::Conforms { subject, trait_ref } => {
            assert!(matches!(subject, TypeTerm::Canonical(ty) if matches!(fixture.analysis.snapshot.store.get(*ty), TypeData::SelfType(_))));
            assert_eq!(trait_ref.declaration, base_decl);
        }
        other => panic!("expected Conforms constraint, got {:?}", other),
    }
}

#[test]
fn trait_contract_where_self_associated_projection_forms() {
    let fixture = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Stream
    where Self::Iterator |= Stepper
{
    type Item
    type Iterator
}
"#,
    );

    let stream_decl = fixture.decl("Stream");
    let stepper_decl = fixture.decl("Stepper");
    let stream_info = fixture.analysis.snapshot.trait_headers.get(&stream_decl).expect("stream trait header");
    let generics = stream_info.generic_signature.as_ref().expect("generics on Stream");

    assert_eq!(generics.constraints.len(), 1);
    match &generics.constraints[0] {
        GenericConstraint::Conforms { subject, trait_ref } => {
            match subject {
                TypeTerm::Canonical(ty) => match fixture.analysis.snapshot.store.get(*ty) {
                    TypeData::AssociatedProjection(proj) => {
                        assert_eq!(proj.trait_ref.declaration, stream_decl);
                        assert_eq!(proj.requirement.owner, stream_decl);
                        assert_eq!(proj.requirement.index, 1); // Iterator is index 1
                        assert!(matches!(fixture.analysis.snapshot.store.get(proj.subject), TypeData::SelfType(_)));
                    }
                    other => panic!("expected AssociatedProjection on Self, got {:?}", other),
                },
                _ => panic!("expected canonical term"),
            }
            assert_eq!(trait_ref.declaration, stepper_decl);
        }
        other => panic!("expected Conforms constraint, got {:?}", other),
    }
}

#[test]
fn trait_contract_qualified_nested_projection_forms() {
    let fixture = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Stream
    where
        Self::Iterator |= Stepper,
        <Self::Iterator as Stepper>::Item == Self::Item
{
    type Item
    type Iterator
}
"#,
    );

    let stream_decl = fixture.decl("Stream");
    let stepper_decl = fixture.decl("Stepper");
    let stream_info = fixture.analysis.snapshot.trait_headers.get(&stream_decl).expect("stream trait header");
    let generics = stream_info.generic_signature.as_ref().expect("generics on Stream");

    assert_eq!(generics.constraints.len(), 2);
    let mut found_conforms = false;
    let mut found_equiv = false;

    for constraint in &generics.constraints {
        match constraint {
            GenericConstraint::Conforms { subject: _, trait_ref } => {
                assert_eq!(trait_ref.declaration, stepper_decl);
                found_conforms = true;
            }
            GenericConstraint::Equivalent { left, right } => {
                // One side should be < <Self as Stream>::Iterator as Stepper>::Item
                // The other side should be <Self as Stream>::Item
                let mut found_nested = false;
                let mut found_stream_item = false;
                for term in [left, right] {
                    if let TypeTerm::Canonical(ty) = term {
                        match fixture.analysis.snapshot.store.get(*ty) {
                            TypeData::AssociatedProjection(proj) => {
                                if proj.trait_ref.declaration == stepper_decl {
                                    assert_eq!(proj.requirement.owner, stepper_decl);
                                    assert_eq!(proj.requirement.index, 0); // Item is index 0 in Stepper
                                    // Subject of stepper projection is <Self as Stream>::Iterator
                                    match fixture.analysis.snapshot.store.get(proj.subject) {
                                        TypeData::AssociatedProjection(inner_proj) => {
                                            assert_eq!(inner_proj.trait_ref.declaration, stream_decl);
                                            assert_eq!(inner_proj.requirement.owner, stream_decl);
                                            assert_eq!(inner_proj.requirement.index, 1); // Iterator is index 1 in Stream
                                            found_nested = true;
                                        }
                                        other => panic!("expected inner AssociatedProjection, got {:?}", other),
                                    }
                                } else if proj.trait_ref.declaration == stream_decl {
                                    assert_eq!(proj.requirement.owner, stream_decl);
                                    assert_eq!(proj.requirement.index, 0); // Item is index 0 in Stream
                                    assert!(matches!(fixture.analysis.snapshot.store.get(proj.subject), TypeData::SelfType(_)));
                                    found_stream_item = true;
                                }
                            }
                            other => panic!("unexpected term: {:?}", other),
                        }
                    }
                }
                assert!(found_nested, "expected nested projection term");
                assert!(found_stream_item, "expected Self::Item projection term");
                assert!(left <= right, "Equality must be canonically oriented");
                found_equiv = true;
            }
            other => panic!("unexpected constraint: {:?}", other),
        }
    }

    assert!(found_conforms);
    assert!(found_equiv);
}

#[test]
fn trait_contract_constraint_order_is_canonical() {
    let fixture1 = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Stream
    where
        Self::Iterator |= Stepper,
        <Self::Iterator as Stepper>::Item == Self::Item
{
    type Item
    type Iterator
}
"#,
    );

    let fixture2 = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Stream
    where
        <Self::Iterator as Stepper>::Item == Self::Item,
        Self::Iterator |= Stepper
{
    type Item
    type Iterator
}
"#,
    );

    let s1 = fixture1.analysis.snapshot.trait_headers.get(&fixture1.decl("Stream")).unwrap();
    let s2 = fixture2.analysis.snapshot.trait_headers.get(&fixture2.decl("Stream")).unwrap();

    let sig1 = s1.generic_signature.as_ref().unwrap();
    let sig2 = s2.generic_signature.as_ref().unwrap();

    assert_eq!(sig1.constraints, sig2.constraints);
}

#[test]
fn trait_contract_unknown_own_associated_projection_diagnoses() {
    let fixture = Fixture::new(
        r#"
trait Stream
    where Self::UnknownType |= Stream
{
    type Item
}
"#,
    );

    let diags = fixture.diagnostics(DiagnosticCode::AssociatedTypeUnknown);
    assert!(
        !diags.is_empty(),
        "expected AssociatedTypeUnknown diagnostic, got: {:?}",
        fixture.analysis.snapshot.diagnostics
    );
}

#[test]
fn trait_contract_does_not_require_associated_declaration_bound_syntax() {
    let fixture = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Iterable
    where
        Self::Iterator |= Stepper,
        <Self::Iterator as Stepper>::Item == Self::Item
{
    type Item
    type Iterator
}
"#,
    );

    let diags = &fixture.analysis.snapshot.diagnostics;
    assert!(diags.is_empty(), "expected clean analysis without inline bound syntax, got: {:?}", diags);
}
