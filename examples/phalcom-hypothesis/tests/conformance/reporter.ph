// Phase 11: reporter forwarding is synchronous, ordered, and exactly once per child.

import {
  Assert,
  CompositeReporter,
  RecordingReporter,
  ReportEvent,
  Reporter
} from "hypothesis"

const left = RecordingReporter.new()
const right = RecordingReporter.new()
const reporter: Reporter = CompositeReporter.new(const [left, right])
const event = ReportEvent.suiteStarted(total: 3)
reporter.handle(event)
Assert.equal(const [event], left.events)
Assert.equal(const [event], right.events)
