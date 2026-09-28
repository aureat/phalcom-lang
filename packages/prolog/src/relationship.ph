// A Relationship is an application of a Predicate.
//
// For:
//
//   Predicate<Person, Int>
//
// args has type:
//
//   (Term<Person>, Term<Int>)
//
// Therefore the relationship retains both:
//
//   1. predicate identity
//   2. exact logical argument domains
//
data Relationship<*Ts> {
  predicate: Predicate<*Ts>,
  args: (*(Ts::Map<Term>))
}

impl<*Ts> Relationship<*Ts> {
  toString -> String {
    "\(predicateName)(\(args))"
  }
}

impl Goal for Relationship<*Ts> {
  predicateId -> PredicateId {
    predicate.id
  }

  predicateName -> Symbol {
    predicate.name
  }

  arity -> Int {
    args.size
  }
}

// Every relationship can be used as a fact
impl<*Ts> Clause for Relationship<*Ts> {}

export Relationship