from .predicate import (
  Predicate,
  PredicateId
)

data Predicate<*Ts> private(
  id: PredicateId,
  name: Symbol,
)

impl<*Ts> Predicate<*Ts> {
  @class
  named(name: Symbol) -> Self<*Ts> {
    Predicate(
      PredicateId.fresh,
      name
    )
  }

  call(*args) -> Relationship<*Ts> {
    Relationship {
      predicate: self,
      args: args
    }
  }
}

export Predicate
