/// A generated value type paired with its property predicate.
data PropertySpec<T, S>(
  strategy: S,
  property: (T) -> Bool,
  identity: Symbol
)

export PropertySpec
