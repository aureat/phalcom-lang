# `Range`

`Range` is an immutable, lazy numeric interval. It stores optional lower and
upper bounds and whether the upper bound is included rather than eagerly
allocating members. Construction is explicit through
`Range.new(lower, upper, upperInclusive)`; either bound may be absent. Membership
uses the supplied numeric bounds, while iteration requires an integer lower
bound and, when present, an integer upper bound.

`contains`/`includes` test membership. `size` requires both bounds and returns
zero when the lower bound exceeds the upper bound; otherwise it is the numeric
difference between bounds, plus one when the upper bound is inclusive.
`at(index)` requires a lower bound and finite size; it accepts negative indices
relative to the end. Iteration produces ascending unit steps; a lower bound
greater than the upper bound is an error rather than a descending traversal.
`take` and iterator combinators can bound traversal without materializing the
whole range. `toList` is the explicit materialization operation on the
iterator.

The operators `a..b` and `a...b` are reserved but inactive. Their committed
future meanings are inclusive and exclusive upper bound respectively, but the
syntax is not part of the active source contract. Until activated, callers use
the Range constructors. A Range is immutable and value-equal; it is eligible
for value hashing under the shared collection rules.

An unbounded range has no finite `size`; an unbounded lower bound cannot be
indexed or iterated. An unbounded upper range can be infinite, so callers should
bound traversal with `take` before materializing it.
