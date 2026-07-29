import Assert from hypothesis
import DatabaseKey from "database/key"
import directory from "database/directory"

class ScriptedFiles {
  @constructor
  new() {
    _quarantined = List.new()
  }

  read(path) -> Result<Bytes, Error> => Ok.new(b"corrupt")
  createDirectories(path) -> Result<None, Error> => Ok.new(None)
  writeTemporary(path, payload) -> Result<Any, Error> => Ok.new(#temporary)
  flush(file) -> Result<None, Error> => Ok.new(None)
  close(file) -> Result<None, Error> => Ok.new(None)
  replaceAtomic(source, destination) -> Result<None, Error> => Ok.new(None)
  quarantine(path) -> Result<None, Error> {
    _quarantined.add(path)
    return Ok.new(None)
  }
  remove(path) -> Result<None, Error> => Ok.new(None)
  exists(path) -> Bool => true

  quarantined -> List<Any> => _quarantined
}

const files = ScriptedFiles.new()
const database = directory.DirectoryDatabase.withFileSystem(
  root: "cache",
  maxEntries: 8,
  maxFileBytes: 65536,
  files: files
)
const key = DatabaseKey.create(
  package: #tests,
  module: #corruption,
  suite: #DatabaseProperties,
  selector: #recovers,
  strategyFingerprint: "int",
  engineFormatVersion: 1
)

Assert.equal(0, database.fetch(key).size)
Assert.equal(1, files.quarantined.size)
