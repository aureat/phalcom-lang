import Assert from hypothesis
import DatabaseKey from "database/key"
import directory from "database/directory"
import Example from "choices/example"

class OrderedFiles {
  @constructor
  new() { _events = List.new() }

  read(path) -> Result<Bytes, Error> { Err.new(FileNotFoundError.new(path)) }
  exists(path) -> Bool { false }
  createDirectories(path) -> Result<None, Error> { _events.add(#mkdir); return Ok.new(None) }
  writeTemporary(path, payload) -> Result<Any, Error> { _events.add(#writeTemporary); return Ok.new(#file) }
  flush(file) -> Result<None, Error> { _events.add(#flush); return Ok.new(None) }
  close(file) -> Result<None, Error> { _events.add(#close); return Ok.new(None) }
  replaceAtomic(source, destination) -> Result<None, Error> { _events.add(#replaceAtomic); return Ok.new(None) }
  quarantine(path) -> Result<None, Error> { Ok.new(None) }
  remove(path) -> Result<None, Error> { Ok.new(None) }

  events -> List<Symbol> { _events }
}

const files = OrderedFiles.new()
const database = directory.DirectoryDatabase.withFileSystem(
  root: "cache",
  maxEntries: 8,
  maxFileBytes: 65536,
  files: files
)
const key = DatabaseKey.create(
  package: #tests, module: #atomic, suite: #DatabaseProperties,
  selector: #writes, strategyFingerprint: "int", engineFormatVersion: 1
)
database.save(key, Example.empty)
Assert.equal(const [#mkdir, #writeTemporary, #flush, #close, #replaceAtomic], files.events)
