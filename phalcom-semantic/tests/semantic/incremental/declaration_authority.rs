//! Architectural ownership regressions for canonical declaration knowledge.

#[test]
fn callable_signature_query_never_reconstructs_semantics_from_dispatch_surface() {
    let query = include_str!("../../../src/db/query.rs");
    assert!(
        !query.contains("semantic_signature_from_surface"),
        "CallableSignature must be computed from declaration-owned semantic facts; DeclarationSurface/dispatch is a projection, never an input authority"
    );
}

#[test]
fn advisory_and_return_refresh_read_canonical_signatures_not_dispatch_surfaces() {
    let session = include_str!("../../../src/session.rs");
    assert!(
        !session.contains("dispatch.get_surface(&callable.owner)?.get_callable(callable.side, &callable.selector)?"),
        "advisory formal return knowledge must come from CallableSignatureTable, not DeclarationSurface"
    );
    assert!(
        !session.contains("let surface = dispatch.surfaces().get(&callable.owner)?;"),
        "inferred-return refresh must decide canonical knowledge from CallableSignatureTable before projecting to dispatch"
    );
    assert!(
        session.contains("callable_signatures.get_for_body"),
        "constructor bodies and ordinary bodies must resolve their declaration-owned callable signature through the canonical table"
    );
}
