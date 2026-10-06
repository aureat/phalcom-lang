/// Ordered pair used by `zip` sequencing.
///
/// v1 deliberately avoids heterogeneous n-ary sequencing; meaningful
/// grammar boundaries should be nominalized into domain types early.
data Pair<A, B>(
  first: A,
  second: B
)

export Pair
