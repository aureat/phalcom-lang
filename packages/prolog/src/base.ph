// A KnowledgeBase is a heterogeneous immutable collection of clauses.
//
// Desired future constraint:
//
//     forall C in Clauses:
//       C implements Clause
//
// We should retain:
//
//   KnowledgeBase<
//     Relationship<Person, Person>,
//     Relationship<Person, Person>,
//     Rule<...>,
//     Rule<...>
//   >
//
// rather than immediately erasing this to:
//
//   List<Clause>
//
data KnowledgeBase<*Clauses> private(
  clauses: Tuple<*Clauses>,
)

impl<*Clauses> KnowledgeBase<*Clauses> {
  Self.of(
    *clauses: *Clauses
  ) -> Self<*Clauses> {
    Self(
      clauses,
    )
  }

  size -> Int {
    clauses.size
  }

  isEmpty -> Bool {
    size == 0
  }

  toString -> String {
    "KnowledgeBase(\(clauses))"
  }
}

export KnowledgeBase