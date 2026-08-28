//! Architectural ownership regressions for canonical declaration knowledge.

#[test]
fn callable_signature_query_never_reconstructs_semantics_from_dispatch_surface() {
    let query = include_str!("../../../src/db/query.rs");
    assert!(
        !query.contains("semantic_signature_from_surface"),
        "CallableSignature must be computed from declaration-owned semantic facts; DeclarationSurface/dispatch is a projection, never an input authority"
    );
}
