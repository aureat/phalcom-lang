// A rule is:
//
//     Head :- Body0, Body1, ...
//
// Head and every body element should eventually be constrained
// to implement Goal.
//
// We deliberately retain every body's precise concrete type.
//
// Desired future constraint:
//
//     Head implements Goal
//     forall G in Body: G implements Goal
//
// Exact pack-constraint syntax is intentionally not invented here.
//
data Rule<Head, *Body> {
  head: Head,
  body: Tuple<*Body>,
}

impl<Head, *Body> Rule<Head, *Body> {
  bodySize -> Int {
    body.size
  }

  toString -> String {
    "\(head) :- \(body)"
  }
}

impl<Head, *Body> Clause for Rule<Head, *Body> {}


export Rule