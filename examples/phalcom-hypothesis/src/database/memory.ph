// Deterministic bounded in-memory example database.

import DatabaseKey from "database/key"
import databaseModel from "database/database"
import Example from "choices/example"
import FailureOrigin from "core/failure"

class MemoryDatabase {
  @constructor
  new() {
    self.init(maxEntries: 32)
  }

  @constructor
  @requires(maxEntries > 0)
  new(maxEntries: Int) {
    self.init(maxEntries: maxEntries)
  }

  init(maxEntries: Int) -> None {
    _maxEntries = maxEntries
    _entries = Map.new()
  }

  fetch(key: DatabaseKey) -> List<Example> {
    const bucket = self._bucket(key)
    const examples = List.new()
    for record in bucket {
      examples.add(record.example)
    }
    return examples
  }

  save(key: DatabaseKey, example: Example) -> MemoryDatabase {
    return self.save(
      key,
      example,
      failureOrigin: None
    )
  }

  save(
    key: DatabaseKey,
    example: Example,
    failureOrigin: Option<FailureOrigin>
  ) -> MemoryDatabase {
    const record = databaseModel._DatabaseRecord.create(
      example: example,
      failureOrigin: failureOrigin
    )
    const kept = List.new()
    kept.add(record)
    for existing in self._bucket(key) {
      if existing.signature != record.signature {
        kept.add(existing)
      }
    }
    while kept.size > _maxEntries {
      kept.removeAt(kept.size - 1)
    }
    _entries.at(key.canonical, put: kept)
    return self
  }

  delete(key: DatabaseKey, example: Example) -> MemoryDatabase {
    const kept = List.new()
    const signature = example.signature
    for existing in self._bucket(key) {
      if existing.signature != signature {
        kept.add(existing)
      }
    }
    _entries.at(key.canonical, put: kept)
    return self
  }

  entryCount -> Int {
    let total = 0
    for key in _entries.keys {
      total += _entries.at(key).size
    }
    return total
  }

  _bucket(key: DatabaseKey) -> List<databaseModel._DatabaseRecord> {
    const found = _entries.at(key.canonical)
    if found == None {
      return List.new()
    }
    const copied = List.new()
    for record in found {
      copied.add(record)
    }
    return copied
  }
}
