from .clause import Clause

data Rule<Head, *Body> {
  head: Head,
  body: Tuple<*Body>,
}

impl<Head, *Body> Rule<Head, *Body> {
  size -> Int { body.size }
  isFact -> Bool { size == 0 }
  toString -> String { "Rule(\(head) :- \(body))" }
}

impl<Head, *Body> Clause for Rule<Head, *Body> {}

export Rule
