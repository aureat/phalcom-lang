import Assert from hypothesis
import DatabaseKey from "database/key"
import directory from "database/directory"
import Example from "choices/example"

const root = ".tmp/phalcom-hypothesis-process-reuse-" + Random.system.nextInt.toString
const key = DatabaseKey.create(
  package: #tests, module: #processReuse, suite: #DatabaseProperties,
  selector: #survives, strategyFingerprint: "int", engineFormatVersion: 1
)

{
  directory.DirectoryDatabase.new(root: root).save(key, Example.empty)
  const nextProcess = directory.DirectoryDatabase.new(root: root)
  Assert.equal(1, nextProcess.fetch(key).size)
}.ensure {
  if FS.exists(root) { FS.removeTree(root) }
}
