# LANG007.C4.1 — Incremental callable-body reuse corrective follow-up

## Status

```text
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

## Reproducer

The clean-HEAD test

```text
impls::queries::adding_impl_member_does_not_invalidate_another_target_or_its_caller
```

failed at the `Arc::ptr_eq` assertion for the unchanged `Caller.run` body.
`Alpha`'s surface remained equal, but adding `Beta.extra` rebuilt the caller.

## Root cause

`callable_body_input_fingerprint_with_formal_inputs` included coarse workspace
inputs in every callable-body identity. In particular, workspace-wide source
resolution and aggregate conformance state changed when an unrelated impl
member was added. The body query also computed its initial fingerprint before
resolving the canonical signature, while downstream revalidation supplied a
different direct signature representation.

That made an unchanged body appear to have new direct inputs even though its
recorded semantic dependencies and callable contract were unchanged.

## Repair

- Move callable-body input fingerprint construction after canonical signature
  resolution so the query uses one lifecycle consistently.
- Remove workspace-wide source-resolution and aggregate conformance state from
  ordinary body direct identity; exact semantic query dependencies remain the
  invalidation authority.
- Keep body-relative inputs for covering/self-specialized contexts and retain
  lifecycle facts only when they actually belong to the callable owner.
- Add diagnostic context to the two Arc-reuse assertions so future failures
  expose update statistics and query keys.

## Verification

Passed:

```text
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls::queries -- --nocapture
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls::bodies -- --nocapture
git diff --check
```

Results:

- `impls::queries`: 60 passed
- `impls::bodies`: 17 passed
- the reproduced regression passes

The existing repository deprecation warnings remain outside this corrective
change. The broad workspace formatting gate was not used; the modified Rust
files were formatted directly.
