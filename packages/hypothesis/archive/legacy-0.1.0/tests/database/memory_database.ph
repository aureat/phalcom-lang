import Assert from hypothesis
import DatabaseKey from "database/key"
import MemoryDatabase from "database/memory"
import Choice from "choices/choice"
import Example from "choices/example"

const key = DatabaseKey.create(
  package: #tests,
  module: #memory,
  suite: #MemoryDatabaseProperties,
  selector: #stores(value:),
  strategyFingerprint: "int",
  engineFormatVersion: 1
)

example(value) {
  return Example.from(
    choices: const [
      Choice.integer(
        value: value,
        min: 0,
        max: 100,
        shrinkTowards: 0,
        label: None
      )
    ],
    spans: const [],
    generationSize: 10
  )
}

const database = MemoryDatabase.new(maxEntries: 2)
database.save(key, example(1))
database.save(key, example(2))
database.save(key, example(2))
database.save(key, example(3))

const fetched = database.fetch(key)
Assert.equal(2, fetched.size)
Assert.equal(3, fetched.at(0).at(0).value)
Assert.equal(2, fetched.at(1).at(0).value)

fetched.clear()
Assert.equal(2, database.fetch(key).size)

database.delete(key, example(3))
Assert.equal(1, database.fetch(key).size)
