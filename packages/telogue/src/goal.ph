from .id import PredicateId

trait Goal {
  predicateId -> PredicateId
  predicateName -> Symbol
  arity -> Int
}

export Goal
