import Assert from hypothesis
import DatabaseKey from "database/key"
import databaseModel from "database/database"
import ExampleCodec from "database/codec"
import Choice from "choices/choice"
import Span from "choices/span"
import Example from "choices/example"
import FailureOrigin from "core/failure"
import PropertyAssertionError from "property/assertion"

const key = DatabaseKey.create(
  package: #tests,
  module: #codec,
  suite: #CodecProperties,
  selector: #roundTrips(value:),
  strategyFingerprint: "tuple(int,bool,index,bytes)",
  engineFormatVersion: ExampleCodec.engineFormatVersion
)
const example = Example.from(
  choices: const [
    Choice.integer(value: -4, min: -10, max: 10, shrinkTowards: 0, label: Some.new(#number)),
    Choice.boolean(value: true, shrinkTowards: false, label: None),
    Choice.index(value: 2, size: 5, shrinkTowards: 0, label: Some.new(#branch)),
    Choice.bytes(value: b"abc", minSize: 0, maxSize: 8, shrinkTowards: b"", label: None)
  ],
  spans: const [
    Span.create(id: 0, label: #root, start: 0, end: 4, parent: None, discardable: false),
    Span.create(id: 1, label: #child, start: 2, end: 4, parent: Some.new(0), discardable: true)
  ],
  generationSize: 17
)
const origin = FailureOrigin.new(
  errorType: PropertyAssertionError,
  module: #codecProperties,
  selector: #roundTrips(value:),
  line: 42,
  column: 7,
  label: Some.new(#roundTrip)
)
const record = databaseModel._DatabaseRecord.create(
  example: example,
  failureOrigin: Some.new(origin)
)

const payload = ExampleCodec.encode(key: key, records: const [record])
const decoded = ExampleCodec.decode(payload: payload, expectedKey: key).unwrap
Assert.equal(1, decoded.size)
Assert.equal(example, decoded.at(0).example)
Assert.equal(origin, decoded.at(0).failureOrigin.unwrap)
