// Phase 11: database adapters preserve copy isolation, deduplicate, delete, and bound retention.

import { Assert, Choice, DatabaseKey, Example, ExampleDatabase, MemoryDatabase } from "hypothesis"

const key = DatabaseKey.create(
  package: #tests,
  module: #conformance,
  suite: #DatabaseAdapter,
  selector: #stores,
  strategyFingerprint: "phase11",
  engineFormatVersion: 1
)
example(value: Int) -> Example {
  return Example.from(
    choices: const [
      Choice.integer(value: value, min: 0, max: 10, shrinkTowards: 0, label: None)
    ],
    spans: const [],
    generationSize: 1
  )
}

const database: ExampleDatabase = MemoryDatabase.new(maxEntries: 2)
database.save(key, example(1), failureOrigin: None)
database.save(key, example(2), failureOrigin: None)
database.save(key, example(2), failureOrigin: None)
const fetched = database.fetch(key)
Assert.equal(2, fetched.size)
fetched.clear()
Assert.equal(2, database.fetch(key).size)
database.delete(key, example(2))
Assert.equal(1, database.fetch(key).size)
