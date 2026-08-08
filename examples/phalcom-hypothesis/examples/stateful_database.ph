// Model-based testing of a key/value store with typed bundles and references.

import Assert from hypothesis
import Bundle from hypothesis
import Gen from hypothesis
import Initialize from hypothesis
import Rule from hypothesis
import StateInvariant from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis
import Teardown from hypothesis
import When from hypothesis
import Settings from hypothesis

const Keys = Bundle<Bytes>.new(#key)
const Values = Bundle<Bytes>.new(#value)

class MemoryStore {
  @constructor
  new() { _entries = Map.new() }

  save(key: Bytes, value: Bytes) { _entries.at(key, put: value) }
  fetch(key: Bytes) -> Any => _entries.at(key)
  delete(key: Bytes) { _entries.remove(key) }
  size -> Int => _entries.size
  close() { None }
}

class DatabaseMachine is StateMachine {
  @constructor
  new() {
    _store = MemoryStore.new()
    _model = Map.new()
  }

  @Initialize
  startEmpty() {
    Assert.equal(0, _store.size)
  }

  @Rule(Gen.bytes, Keys.publish)
  createKey(value: Bytes) -> Bytes {
    return value
  }

  @Rule(Gen.bytes, Values.publish)
  createValue(value: Bytes) -> Bytes {
    return value
  }

  @Rule(Keys, Values)
  save(key: Bytes, value: Bytes) {
    _store.save(key: key, value: value)
    _model.at(key, put: value)
  }

  hasSavedValues -> Bool => _model.size > 0

  @When(#hasSavedValues)
  @Rule(Keys)
  valuesAgree(key: Bytes) {
    if _model.at(key) != None {
      Assert.equal(_model.at(key), _store.fetch(key))
    }
  }

  @When(#hasSavedValues)
  @Rule(Keys.consume)
  delete(key: Bytes) {
    _store.delete(key)
    _model.remove(key)
  }

  @StateInvariant
  sizesAgree() {
    Assert.equal(_model.size, _store.size)
  }

  @Teardown
  close() {
    _store.close()
  }
}

const result = Stateful.check(
  DatabaseMachine,
  with: Settings.standard
    .examples(100)
    .statefulSteps(50)
    .seed(20260723)
)

if result.failed || {
  System.print(result.error.statefulScenario.executable)
}
