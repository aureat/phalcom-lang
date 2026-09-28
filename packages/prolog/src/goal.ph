// Goal is deliberately behavioral.
//
// Relationship retains its exact generic signature:
//
//   Relationship<Person, Person>
//
// while Goal provides the common surface required by rules,
// queries, reflection and eventually the solver.
//
trait Goal {
  predicateId -> PredicateId
  predicateName -> Symbol
  arity -> Int
}

export Goal