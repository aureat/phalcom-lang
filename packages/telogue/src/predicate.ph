from prolog.id import PredicateId
from prolog.term import Term
from prolog.goal import Goal
from prolog.clause import Clause

data Predicate<*Ts> private(
  id: PredicateId,
  name: Symbol,
)

data Relationship<*Ts> {
  predicate: Predicate<*Ts>,
  args: Pack<*(Ts::Map<Term>)>,
}

impl<*Ts> Predicate<*Ts> {
  @class
  named(name: Symbol) -> Self {
    Predicate<*Ts>(
      PredicateId.fresh,
      name,
    )
  }

  call(
    *args: *(Ts::Map<Term>)
  ) -> Relationship<*Ts> {
    Relationship<*Ts> {
      predicate: self,
      args: args,
    }
  }
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
