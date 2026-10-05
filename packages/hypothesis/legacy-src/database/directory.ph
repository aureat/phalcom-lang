// Bounded process-persistent example database with atomic file replacement.

import DatabaseKey from "database/key"
import databaseModel from "database/database"
import ExampleCodec from "database/codec"
import Example from "choices/example"
import FailureOrigin from "core/failure"
import errors from "core/errors"

protocol _DatabaseFileSystem {
  exists(path: Any) -> Bool
  read(path: Any) -> Result<Bytes, Error>
  createDirectories(path: Any) -> Result<None, Error>
  writeTemporary(path: Any, payload: Bytes) -> Result<Any, Error>
  flush(file: Any) -> Result<None, Error>
  close(file: Any) -> Result<None, Error>
  replaceAtomic(source: Any, destination: Any) -> Result<None, Error>
  quarantine(path: Any) -> Result<None, Error>
  remove(path: Any) -> Result<None, Error>
}


class _DirectoryLockTable {
  @constructor
  new() {
    _active = Set.new()
  }

  withPathLock(path: Any, body: Closure) -> Any {
    const key = path.toString
    if _active.includes(key) {
      throw errors._DatabaseLockUnavailable.new(
        "directory database path is already being modified in this process"
      )
    }
    _active.add(key)
    return || {
      body.call()
    }.ensure || {
      _active.remove(key)
    }
  }
}

const _directoryLocks = _DirectoryLockTable.new()

class DirectoryDatabase {
  @constructor
  new(root: Any) {
    self.init(
      root: root,
      maxEntries: 16,
      maxFileBytes: 1048576,
      files: _SystemDatabaseFileSystem.new()
    )
  }

  @constructor
  @requires(maxEntries > 0)
  @requires(maxFileBytes > 0)
  new(root: Any, maxEntries: Int, maxFileBytes: Int) {
    self.init(
      root: root,
      maxEntries: maxEntries,
      maxFileBytes: maxFileBytes,
      files: _SystemDatabaseFileSystem.new()
    )
  }

  @constructor
  @private
  withFileSystem(
    root: Any,
    maxEntries: Int,
    maxFileBytes: Int,
    files: _DatabaseFileSystem
  ) {
    self.init(
      root: root,
      maxEntries: maxEntries,
      maxFileBytes: maxFileBytes,
      files: files
    )
  }

  @class
  @requires(maxEntries > 0)
  @requires(maxFileBytes > 0)
  withFileSystem(
    root: Any,
    maxEntries: Int,
    maxFileBytes: Int,
    files: _DatabaseFileSystem
  ) -> DirectoryDatabase {
    return DirectoryDatabase.withFileSystem(
      root: root,
      maxEntries: maxEntries,
      maxFileBytes: maxFileBytes,
      files: files
    )
  }

  init(
    root: Any,
    maxEntries: Int,
    maxFileBytes: Int,
    files: _DatabaseFileSystem
  ) -> None {
    _root = root
    _maxEntries = maxEntries
    _maxFileBytes = maxFileBytes
    _files = files
  }

  fetch(key: DatabaseKey) -> List<Example> {
    const examples = List.new()
    for record in self.records(key) {
      examples.add(record.example)
    }
    return examples
  }

  save(key: DatabaseKey, example: Example) -> DirectoryDatabase {
    return self.save(key, example, failureOrigin: None)
  }

  save(
    key: DatabaseKey,
    example: Example,
    failureOrigin: Option<FailureOrigin>
  ) -> DirectoryDatabase {
    // merge-on-write: acquire the shared process-local path lock, then read the
    // latest visible records before constructing the replacement payload.
    const locked = || {
      _directoryLocks.withPathLock(self.path(key)) {
        self.saveMerged(
          key: key,
          example: example,
          failureOrigin: failureOrigin
        )
      }
    }.attempt()
    if locked.isErr and not locked.unwrapErr.isA(errors._DatabaseLockUnavailable) {
      locked.unwrapErr.raise()
    }
    return self
  }

  @private
  saveMerged(
    key: DatabaseKey,
    example: Example,
    failureOrigin: Option<FailureOrigin>
  ) -> None {
    const record = databaseModel._DatabaseRecord.create(
      example: example,
      failureOrigin: failureOrigin
    )
    const records = List.new()
    records.add(record)
    for existing in self.records(key) {
      if existing.signature != record.signature || {
        records.add(existing)
      }
    }
    self.write(key: key, records: records)
  }

  delete(key: DatabaseKey, example: Example) -> DirectoryDatabase {
    const locked = || {
      _directoryLocks.withPathLock(self.path(key)) {
        self.deleteMerged(key: key, example: example)
      }
    }.attempt()
    if locked.isErr and not locked.unwrapErr.isA(errors._DatabaseLockUnavailable) {
      locked.unwrapErr.raise()
    }
    return self
  }

  @private
  deleteMerged(key: DatabaseKey, example: Example) -> None {
    const records = List.new()
    for existing in self.records(key) {
      if existing.signature != example.signature || {
        records.add(existing)
      }
    }
    if records.size == 0 {
      _files.remove(self.path(key))
      return None
    }
    self.write(key: key, records: records)
  }

  @private
  records(key: DatabaseKey) -> List<databaseModel._DatabaseRecord> {
    const path = self.path(key)
    if not _files.exists(path) {
      return List.new()
    }

    const read = _files.read(path)
    if read.isErr || {
      return List.new()
    }

    const payload = read.unwrap
    if payload.size > _maxFileBytes {
      _files.quarantine(path)
      return List.new()
    }

    const decoded = ExampleCodec.decode(
      payload: payload,
      expectedKey: key
    )
    if decoded.isErr || {
      _files.quarantine(path)
      return List.new()
    }
    return _DirectoryCopies.records(decoded.unwrap)
  }

  @private
  write(
    key: DatabaseKey,
    records: List<databaseModel._DatabaseRecord>
  ) -> None {
    const kept = _DirectoryCopies.records(records)
    while kept.size > _maxEntries {
      kept.removeAt(kept.size - 1)
    }

    let payload = ExampleCodec.encode(key: key, records: kept)
    while payload.size > _maxFileBytes and kept.size > 1 {
      kept.removeAt(kept.size - 1)
      payload = ExampleCodec.encode(key: key, records: kept)
    }
    if payload.size > _maxFileBytes {
      return
    }

    const directory = self._directory
    const destination = self.path(key)
    const temporary = destination + ".tmp-" + Random.system.nextInt.toString

    const created = _files.createDirectories(directory)
    if created.isErr || {
      return
    }

    const opened = _files.writeTemporary(temporary, payload)
    if opened.isErr || {
      _files.remove(temporary)
      return
    }
    const file = opened.unwrap

    const flushed = _files.flush(file)
    if flushed.isErr || {
      _files.close(file)
      _files.remove(temporary)
      return
    }

    const closed = _files.close(file)
    if closed.isErr || {
      _files.remove(temporary)
      return
    }

    const replaced = _files.replaceAtomic(temporary, destination)
    if replaced.isErr || {
      _files.remove(temporary)
    }
  }

  _directory -> String {
    return _root.toString + "/v" + ExampleCodec.schemaVersion.toString
  }

  @private
  path(key: DatabaseKey) -> String {
    return self._directory + "/" + key.fileStem + ".phdb"
  }
}

class _SystemDatabaseFileSystem {
  exists(path: Any) -> Bool { FS.exists(path) }

  read(path: Any) -> Result<Bytes, Error> {
    return || { FS.readBytes(path) }.attempt()
  }

  createDirectories(path: Any) -> Result<None, Error> {
    return || { FS.createDirectories(path); None }.attempt()
  }

  writeTemporary(path: Any, payload: Bytes) -> Result<Any, Error> {
    return || {
      const file = FS.open(path, mode: #writeExclusive)
      file.write(payload)
      return file
    }.attempt()
  }

  flush(file: Any) -> Result<None, Error> {
    return || { file.flush; None }.attempt()
  }

  close(file: Any) -> Result<None, Error> {
    return || { file.close; None }.attempt()
  }

  replaceAtomic(source: Any, destination: Any) -> Result<None, Error> {
    return || { FS.replace(source, with: destination); None }.attempt()
  }

  quarantine(path: Any) -> Result<None, Error> {
    return || {
      const destination = path.toString + ".corrupt-" +
        Random.system.nextInt.toString
      FS.move(path, to: destination)
      return None
    }.attempt()
  }

  remove(path: Any) -> Result<None, Error> {
    return || {
      if FS.exists(path) { FS.remove(path) }
      return None
    }.attempt()
  }
}

class _DirectoryCopies {
  @class
  records(
    values: List<databaseModel._DatabaseRecord>
  ) -> List<databaseModel._DatabaseRecord> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }
}
