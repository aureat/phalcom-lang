# PDR-0031 — Range slicing uses normalized bounds with two native collection seams

- Status: Accepted
- Date: 2026-08-08
- Amends: ADR-0019's frozen primitive floor and ADR-0039's collection-container admission
- Related: [C.3 implementation specification](../work/pending/collections/C.3-range-slicing-and-list-replacement.md), [floor census](../spec/current/core/floor-census.md), [PDR-0011](0011-admit-bytes-native-octet-buffer.md)

## Context

C.3 adds Range reads for List, Tuple, and Bytes plus variable-length List replacement. Bound
interpretation is derivable in `core.ph` from Range's existing optional-bound observations, but
two operations cross the representation boundary: rebuilding a Tuple while preserving its
labeled suffix, and changing a List's vector length without replacing its receiver object.

Copying range normalization into every collection would make negative, inclusive, and clamped
semantics drift. Reconstructing Tuple slices through List would discard labels. Expressing List
splice through `set_` and `push_` cannot shrink or grow an interior span while retaining identity.

## Decision

1. `Range#sliceBounds_(size)` lives in `core.ph`. It validates finite integral compatibility
   coordinates, distinguishes omitted endpoints from supplied `None`, handles negative and
   inclusive bounds, and clamps to finite consumer size.
2. Add exactly two bindings to the floor: `Tuple#slice_(_,_)` and
   `List#replaceSlice_(_,_,_)`. The census moves 158 to 160 bindings and 129 to 131 native
   functions.
3. `Tuple#slice_` receives only canonical half-open bounds and finalizes through `finish_tuple`,
   preserving selected labels and normalizing an empty result to Unit.
4. `List#replaceSlice_` accepts a List replacement only. It snapshots replacement elements before
   borrowing the destination mutably, so self-replacement is stable.

## Consequences

- List and Bytes range reads stay derived; Bytes reuses its existing `slice_` bulk primitive.
- Source slice assignment remains strict through `List#replace(...).unwrap`; its expression
  result is still handled by C.1's assignment lowering.
- The cost is two permanent floor bindings and a temporary List-only replacement restriction.
- This does not define Range traversal, descending ranges, stepped slices, generic iterable
  replacement, or zero-copy views. Those remain future work and cannot reinterpret C.3's local
  reversed-interval-as-empty rule.

## Alternatives rejected

- **A Range-aware primitive on every collection:** rejected because it duplicates normalization
  and embeds source-level slice policy below the floor.
- **Tuple reconstruction through List:** rejected because it loses label identity and bypasses
  product finalization.
- **Arbitrary Iterable replacement:** rejected because it forces boundedness, materialization,
  and re-entrant iteration policy before Spec E.
- **No List splice primitive:** rejected because fixed-width `set_`/`push_` cannot express an
  identity-preserving variable-length replacement.
