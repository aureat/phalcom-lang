# AUD-T3 — Source-level nested covering forms remain under-tested

## Classification

- **Program:** LANG007
- **Checkpoint:** LANG007.C4.1 corrective closure
- **Severity:** Medium; end-to-end verification gap
- **Status:** No current implementation defect demonstrated
- **Current owner:** covering-body source fixtures and semantic integration tests

## Issue

The low-level `Self` specialization and RecordRow carrier tests are broad. The
source-level covering-body tests exercise a smaller set of forms. The audit did
not find a compact end-to-end matrix proving the complete pipeline for nested
types:

```text
parse
  → declaration/signature formation
  → declaration-canonical publication
  → inverse covering materialization
  → owner-relative Self specialization
  → source-body checking
  → body dependency/publication
```

The missing combinations include, where supported by the current source
language:

- `Option<Box<T>>` or equivalent nested nominal application;
- callable types containing `T` or `Self`;
- tuple containing `Self`;
- record containing `Self`;
- associated projection containing `Self`;
- type-member projection containing `Self`;
- pack or row carrier containing `Self`.

This is a certification gap, not evidence that the recursive helper is wrong.

## Why source-level coverage matters

Unit tests can prove that `specialize_self_type` handles a constructed
`TypeData` value. They do not prove that the parser, resolver, signature
formation, covering substitution, and body checker construct and transport the
same form.

A source pipeline can fail by:

- resolving `Self` under the wrong owner before specialization;
- dropping a nested type while canonicalizing a signature;
- failing to reconstruct a row or projection carrier during inverse covering;
- publishing a correct top-level type while leaving an old nested parameter;
- checking the body against a different type than the one used by diagnostics;
- recording a body dependency against the wrong definition product.

## Evidence in the live tree

- `types/substitution.rs` owns recursive `Self` specialization and generic
  constraint mapping.
- `impls.rs` owns covering materialization and body context construction.
- `session.rs` supplies the body-relative context during initial analysis and
  downstream revalidation.
- `tests/semantic/impls/bodies.rs` contains covering, shadowing, row, receiver,
  and incremental body tests, but not a systematic nested carrier matrix.
- `tests/semantic/foundations/record_row_materialization.rs` and related
  foundation tests cover low-level row behavior.

## Correct architectural approach

Add a small source fixture matrix in the owning semantic test area. Each case
should vary one carrier while holding the ownership pattern constant:

```text
data Box<T>(...)
impl<I> Box<I> {
    method(...) -> <carrier involving Self or I> { ... }
}
```

The tests must inspect both:

1. the published declaration-canonical callable signature; and
2. the body analysis under the source-relative impl coordinate system.

Do not add syntax solely to satisfy this matrix. First classify each desired
form as parser-supported, signature-supported but body-unsupported, or not yet
part of the source language. Unsupported forms belong in a staged lead with a
precise blocker.

## Testing strategy

For each supported carrier:

- parse with no syntax errors;
- assert the update has no unexpected diagnostics;
- locate the exact callable definition by `CallableId`;
- inspect its published signature for declaration-owned coordinates;
- inspect the body analysis for the expected nested type;
- compare a covering permutation (`Pair<B,A>`) with an identity mapping;
- include a negative binder-confusion case where a callable-local `U` must not
  become the impl's `T`.

At least one test should mutate only the covering target while leaving the body
text fixed, then compare the changed body product and unchanged public contract.

## Investigation leads

- Start from existing `test_covering_class_impl_body_specializes_bare_self`,
  row materialization tests, and callable-shadowing tests.
- Inspect the AST syntax for callable types, record rows, packs, associated
  projections, and type-member projections before adding fixtures.
- Reuse the exact source-body provenance assertion `(ImplId,
  source_member_index)`.
- Keep the matrix small: one positive and one negative fixture per carrier is
  more useful than a large suite with overlapping assertions.

## Acceptance condition

The issue is closed when all currently supported nested carriers have a source
fixture through body checking and publication, unsupported forms are explicitly
classified, and the matrix compares covering and identity mappings without
weakening existing binder or coordinate assertions.
