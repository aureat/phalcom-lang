// Discovery rejects invalid predicates and bundle target/result relationships
// before the search engine begins generating examples.
import Assert from hypothesis
import Bundle from hypothesis
import Gen from hypothesis
import Rule from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis
import When from hypothesis

const TypedValues = Bundle<Int>.new(#value, elementType: Int)

class MissingPredicateMachine is StateMachine {
  @When(#doesNotExist)
  @Rule
  step() { None }
}

class PredicateArityMachine is StateMachine {
  available(extra: Int) -> Bool { return true }

  @When(#available)
  @Rule
  step() { None }
}

class PredicateTypeMachine is StateMachine {
  available -> Int => 1

  @When(#available)
  @Rule
  step() { None }
}

class InvalidTargetMachine is StateMachine {
  @Rule(Gen.text, TypedValues.publish)
  create(value: String) -> String { return value }
}

Assert.true({ Stateful.check(MissingPredicateMachine) }.attempt().isErr)
Assert.true({ Stateful.check(PredicateArityMachine) }.attempt().isErr)
Assert.true({ Stateful.check(PredicateTypeMachine) }.attempt().isErr)
Assert.true({ Stateful.check(InvalidTargetMachine) }.attempt().isErr)
