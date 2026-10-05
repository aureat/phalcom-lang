data KnowledgeBase<T> private(
  clauses: Dynamic
)

impl<T> KnowledgeBase<T> {
  @class
  of(
    clauses: T
  ) -> Self<T> {
    KnowledgeBase<T>(clauses: clauses)
  }

  size -> Int { clauses.size }
  isEmpty -> Bool { size == 0 }
  toString -> String { "KnowledgeBase(\(clauses))" }
}

export KnowledgeBase
