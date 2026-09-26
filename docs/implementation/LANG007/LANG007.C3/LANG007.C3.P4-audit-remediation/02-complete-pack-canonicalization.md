# 02 — Complete-pack canonicalization across source identity

## Finding and accepted law

The audited normalization law is:

```text
Complete(Positionals(P), Labeled(P)) ≡ P
```

The round trip must be independent of whether the two projections were created
internally or selected through the published source-backed intrinsic members.
Source declaration provenance remains part of the semantic product; it is not
discarded to make normalization succeed.

## Owning layer and identity checks

The correction stays in `TypeStore::canonical_complete_pack_subject`, the
existing complete-pack normalization owner. It recognizes only the canonical
intrinsic `CompleteTypePack::Positionals` and
`CompleteTypePack::Labeled` operations, including their canonical
`TypeMemberDeclarationId`s when source-backed. The pair must have the expected
`TypePack` and `LabeledTypePack` result kinds, zero arguments, and the same
subject of `CompleteTypePack` kind.

Selector spelling by itself is not authority. Unrelated user-owned members
named `Positionals` or `Labeled` do not trigger the rule, and distinct subjects
remain distinct. Canonical source declarations are preserved on the resulting
source-visible projections.

The focused discriminators cover:

- internal + internal lane round trip;
- source-backed + source-backed lane round trip;
- canonical mixed internal/source formation;
- differing subjects;
- unrelated user-owned same-name members;
- retained canonical source member declaration identity.

## Verification

[Actions run 36264589395](https://github.com/aureat/phalcom-lang/actions/runs/36264589395)
confirmed selection and passed exactly these focused tests:

| Test | Result |
|---|---|
| `complete_pack_reassembles_only_its_own_intrinsic_lanes` | 1 passed; 118 filtered |
| `source_visible_complete_pack_lane_projections_keep_canonical_member_declarations` | 1 passed; 1710 filtered |

The source-visible discriminator deliberately constructs/uses the published
product IDs at the `TypeStore` boundary. It proves source-backed identity is
retained by canonicalization; it does **not** prove the annotation resolver
accepts a kind-only `TypeData::Parameter` fixture. Earlier fixture attempts
through that route were superseded by the passing targeted test. No broad suite
or release gate was run for this item.

## State and next action

This closes the complete-pack source-identity finding only. P4 remains
`IN_PROGRESS / PARTIAL / UNVERIFIED`, and the full audit/release gates remain
open. The next audit item is to investigate and classify the reported runtime
regressions against the named predecessor revisions, then classify remaining
semantic failures before resuming the unfinished T4/T5 work.
