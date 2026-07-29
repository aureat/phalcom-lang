import Assert from hypothesis
import DatabaseKey from "database/key"

const first = DatabaseKey.create(
  package: #hypothesisTests,
  module: #databaseProperties,
  suite: #CodecProperties,
  selector: #roundTrips(value:),
  strategyFingerprint: "list(int(-10,10))",
  engineFormatVersion: 1
)
const same = DatabaseKey.create(
  package: #hypothesisTests,
  module: #databaseProperties,
  suite: #CodecProperties,
  selector: #roundTrips(value:),
  strategyFingerprint: "list(int(-10,10))",
  engineFormatVersion: 1
)
const changedStrategy = DatabaseKey.create(
  package: #hypothesisTests,
  module: #databaseProperties,
  suite: #CodecProperties,
  selector: #roundTrips(value:),
  strategyFingerprint: "list(int(-100,100))",
  engineFormatVersion: 1
)
const changedVersion = first.with(engineFormatVersion: 2)

Assert.equal(first, same)
Assert.equal(first.canonical, same.canonical)
Assert.equal(first.fileStem, same.fileStem)
Assert.true(first != changedStrategy)
Assert.true(first != changedVersion)
Assert.true(first.canonical.includes("hypothesisTests"))
Assert.true(first.canonical.includes("databaseProperties"))
Assert.true(first.canonical.includes("CodecProperties"))
Assert.true(first.canonical.includes("roundTrips(value:)"))
