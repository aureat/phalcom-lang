# `Set<T>`

`Set<T>` is a mutable collection of unique members. `Set.new()` creates an
empty set. Membership uses the member's hash and the keyed-collection
equivalence relation; inserting an equivalent value does not create a
duplicate. For numeric members, see
[`../numeric-types.md`](../numeric-types.md).

`add(value)`, `remove(value)`, and `contains`/`includes` mutate or inspect the
set. `size` reports unique members. `at(index)` selects the member at its
encounter-order position. Iteration is deterministic and follows insertion
order. Set equality is structural by size and membership, independent of
encounter order.

Because Set is mutable, it cannot be a `Map` key or a member of another Set.
Set membership operations must preserve the language's
hash/equality consistency rules and tolerate user-defined hash/equality
re-entry.

The `#{...}` sigil is reserved but inactive. The active construction form is
`Set.new()` followed by `add`; the empty set is not `#{}` today. See
[`../collection-literals.md`](../collection-literals.md).

This specification does not prescribe the underlying hash-set representation.
