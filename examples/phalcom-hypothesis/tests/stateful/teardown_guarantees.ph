// Teardown is structurally guaranteed exactly once for every created machine
// instance on pass, user failure, strategy rejection, replay invalidation, and
// an execution-time internal error.
import Assert from hypothesis
import Bundle from hypothesis
import Gen from hypothesis
import Rule from hypothesis
import Teardown from hypothesis
import When from hypothesis
import Settings from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis

const TeardownRecords = List.new()
const RejectingInt = Gen.int.filter(
  { _ => false },
  maxAttempts: 1
)
const DependencyValues = Bundle<Int>.new(#dependency)

class TeardownProbe is StateMachine {
  @constructor
  new() {
    _teardownRecord = List.new()
    _teardownRecord.add(0)
    TeardownRecords.add(_teardownRecord)
  }

  recordTeardown() {
    _teardownRecord.at(0, put: _teardownRecord.at(0) + 1)
  }
}

class PassingTeardownMachine is TeardownProbe {
  @Rule
  pass() { None }

  @Teardown
  close() { self.recordTeardown() }
}

class FailingTeardownMachine is TeardownProbe {
  @Rule
  fail() { Assert.fail("failure") }

  @Teardown
  close() { self.recordTeardown() }
}

class RejectingTeardownMachine is TeardownProbe {
  @Rule(RejectingInt)
  reject(value: Int) { None }

  @Teardown
  close() { self.recordTeardown() }
}

class ReplayInvalidationTeardownMachine is TeardownProbe {
  @Rule(Gen.int, DependencyValues.publish)
  produce(value: Int) -> Int { return value }

  @Rule(DependencyValues)
  depend(value: Int) { Assert.fail("dependent failure") }

  @Teardown
  close() { self.recordTeardown() }
}

class InternalErrorTeardownMachine is TeardownProbe {
  // The reflected annotation passes discovery; the dynamic value is invalid.
  invalidPredicate -> Bool => 1

  @When(#invalidPredicate)
  @Rule
  step() { None }

  @Teardown
  close() { self.recordTeardown() }
}

const small = Settings.standard
  .examples(3)
  .statefulSteps(5)
  .seed(20260723)

Stateful.check(PassingTeardownMachine, with: small)
Stateful.check(FailingTeardownMachine, with: small)
Stateful.check(RejectingTeardownMachine, with: small)
Stateful.check(ReplayInvalidationTeardownMachine, with: small)
Stateful.check(InternalErrorTeardownMachine, with: small)

for record in TeardownRecords {
  Assert.equal(1, record.at(0))
}
