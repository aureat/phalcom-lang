// Rule return values are published into typed bundle descriptors.
import Assert from hypothesis
import Bundle from hypothesis
import Gen from hypothesis
import Rule from hypothesis
import Settings from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis

const PublishedKeys = Bundle<Bytes>.new(#key)

class PublishingMachine is StateMachine {
  @Rule(Gen.bytes, PublishedKeys.publish)
  create(value: Bytes) -> Bytes { return value }

  @Rule(PublishedKeys)
  observe(value: Bytes) { Assert.true(value.isA(Bytes)) }
}

Stateful.check(PublishingMachine, with: Settings.standard.statefulSteps(10))
