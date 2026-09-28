data Predicate<*Ts> private(
  id: PredicateId,
  name: Symbol,
)

impl<*Ts> Predicate<*Ts> {
  Self.named(name: Symbol) -> Self<*Ts> {
    Predicate(
      PredicateId.fresh,
      name,
    )
  }

  call(*args: *Ts) -> Relationship<*Ts> {
    Relationship {
      predicate: self,
      args: args
    }
  }
}

export Predicate
