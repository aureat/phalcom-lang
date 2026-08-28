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

#[test]
fn native_import_builds_canonical_signature_before_dispatch_projection() {
    let native = include_str!("../../../src/types/native.rs");
    assert!(
        !native.contains("surface.get_callable(side, &callable_id.selector)"),
        "native metadata must build CallableSemanticSignature directly; it must never reconstruct declaration truth by reading the dispatch surface it just projected"
    );
    assert!(
        native.contains("project_semantic_signature(&canonical_signature)"),
        "native dispatch must be a projection of the canonical native callable signature"
    );
}

#[test]
fn formal_field_lookup_never_reads_type_from_dispatch_surface() {
    let context = include_str!("../../../src/checker/context.rs");
    assert!(
        !context.contains("surface.get_field(side, name)"),
        "formal field lookup must consume FieldSignatureTable; DeclarationSurface may describe structural membership but must not be field type authority"
    );
    assert!(
        context.contains("field_signatures"),
        "checker context must receive canonical field declaration knowledge rather than reconstructing it from dispatch"
    );
}

#[test]
fn field_signature_is_a_first_class_incremental_product() {
    let key = include_str!("../../../src/db/key.rs");
    let product = include_str!("../../../src/db/product.rs");
    let query = include_str!("../../../src/db/query.rs");
    let analysis = include_str!("../../../src/checker/analysis.rs");

    assert!(
        key.contains("FieldSignature(FieldId)"),
        "canonical field declaration knowledge needs its own QueryKey rather than borrowing DeclarationSurface invalidation"
    );
    assert!(
        product.contains("FieldSignature(Arc<FieldSemanticSignature>)"),
        "the semantic DB must publish the same canonical FieldSemanticSignature consumed by formal analysis"
    );
    assert!(
        query.contains("query_field_signature"),
        "field signatures must be computed as declaration-owned query products"
    );
    assert!(
        analysis.contains("FieldSignature(FieldId)"),
        "body analyses must be able to record exact canonical field dependencies"
    );
}
