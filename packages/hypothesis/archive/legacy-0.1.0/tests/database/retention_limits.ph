import Assert from hypothesis
import DatabaseKey from "database/key"
import MemoryDatabase from "database/memory"
import Choice from "choices/choice"
import Example from "choices/example"

example(value) {
  return Example.from(
    choices: const [Choice.integer(value: value, min: 0, max: 100, shrinkTowards: 0, label: None)],
    spans: const [],
    generationSize: 1
  )
}

const key = DatabaseKey.create(
  package: #tests, module: #limits, suite: #DatabaseProperties,
  selector: #bounded, strategyFingerprint: "int", engineFormatVersion: 1
)
const database = MemoryDatabase.new(maxEntries: 3)
for value in [1, 2, 3, 4, 5] { database.save(key, example(value)) }
Assert.equal(const [5, 4, 3], database.fetch(key).map |item| { item.at(0).value })
