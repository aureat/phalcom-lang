import { JsonReporter, ReportEvent } from "hypothesis"

const reporter = JsonReporter.new()
reporter.handle(ReportEvent.propertyStarted(id: #sample))
reporter.handle(ReportEvent.phaseStarted(id: #sample, phase: #generate))
const records = reporter.records
Assert.equal(2, records.size)
Assert.equal(1, records.at(0).at("schemaVersion"))
Assert.equal("property_started", records.at(0).at("type"))
Assert.equal("phase_started", records.at(1).at("type"))
Assert.true(reporter.jsonLines.at(0).startsWith("{\"schemaVersion\":1"))
