// Regression: the codec validates the same full record signature that it writes,
// including choice metadata and semantic spans.

import { Assert, Choice, DatabaseKey, Example } from "hypothesis"
import databaseModel from "database/database"
import ExampleCodec from "database/codec"
import Span from "choices/span"

const key = DatabaseKey.create(
  package: #tests,
  module: #regression,
  suite: #DatabaseSignature,
  selector: #roundTrips,
  strategyFingerprint: "phase11-signature",
  engineFormatVersion: ExampleCodec.engineFormatVersion
)
const example = Example.from(
  choices: const [
    Choice.integer(value: 4, min: 0, max: 10, shrinkTowards: 0, label: Some.new(#value))
  ],
  spans: const [
    Span.create(id: 0, label: #root, start: 0, end: 1, parent: None, discardable: false)
  ],
  generationSize: 7
)
const record = databaseModel._DatabaseRecord.create(
  example: example,
  failureOrigin: None
)
const encoded = ExampleCodec.encode(key: key, records: const [record])
const decoded = ExampleCodec.decode(payload: encoded, expectedKey: key)
Assert.true(decoded.isOk)
Assert.equal(record.signature, decoded.unwrap.at(0).signature)
