# Semantic products and reflection

> Sources: semantic-analyzer chapters 08–13; SEMA005
> Raw: [semantic specification and program snapshot](../raw/semantic/2026-09-08-semantic-sources.md)
> Updated: 2026-09-08

Semantic products are immutable projections with stable identities, source attachments, status, type knowledge, and dependency fingerprints. Reflection and editor presentation consume these products; they do not become a second semantic authority.

## Product families

Callable signatures, declaration surfaces, expression types, source occurrences, runtime type roots, diagnostics, and advisory shape contributions each have a defined owner. A projection may omit unsupported detail, but it must not invent a stronger fact than its source product provides.

## Reflection boundary

Runtime type metadata is a published projection of semantic identity and type shape. Reflection capabilities are negotiated separately from formal inference. [Semantic type metadata](../type-system/semantic-type-metadata.md) carries the transport form; [native canonical surface](../native/canonical-surface.md) supplies host declarations.

## Status

SEMA005 is in progress and partial. Proposed program records are linked from the implementation index rather than presented as shipped features.
