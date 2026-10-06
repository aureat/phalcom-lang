from prolog.id import PredicateId
from .term import Term
from .goal import Goal
from .clause import Clause

data Predicate<*Ts> private(
  id: PredicateId,
  name: Symbol,
)

impl<*Ts> Predicate<*Ts> {
  @class
  named(name: Symbol) -> Self {
    Predicate<*Ts>(
      id: PredicateId.fresh,
      name: name
    )
  }

  call(
    *args: Pack<*(Ts::Map<Term>)>
  ) -> Relationship<*Ts> {
    Relationship<> {
      predicate: self,
      args: args
    }
  }
}

data Relationship<*Ts> {
  predicate: Predicate<*Ts>,
  args: Pack<*(Ts::Map<Term>)>,
}

impl<*Ts> Relationship<*Ts> {
  toString -> String { "\(predicateName)(\(args))" }
}

impl<*Ts> Goal for Relationship<*Ts> {
  predicateId -> PredicateId { predicate.id }
  predicateName -> Symbol { predicate.name }
  arity -> Int { args.size }
}

impl<*Ts> Clause for Relationship<*Ts> {}

export (Predicate, Relationship)
