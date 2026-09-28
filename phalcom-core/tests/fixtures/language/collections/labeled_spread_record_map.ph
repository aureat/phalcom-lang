// area: collections
// spec: collection spread Part II §§2, 4–5, 13, 15–19
// status: PASS
// Runtime uses a shared labeled-lane helper, while Record and Map retain
// different source-level admission rules.

const tupleSource = (1, left: 2, right: 3)
const recordSource = { recordLeft: 4, recordRight: 5 }
const mapSource = Map { mapLeft: 6, mapRight: 7 }

const recordFromTuple = { head: 0, **tupleSource, tail: 8 }
const recordFromRecord = { **recordSource }

System.print(Map.from(recordFromTuple).toString)
System.print(Map.from(recordFromRecord).toString)
System.print(({ **() }).class == Record)
const gcRecord = { held: [1, 2], **(), after: System.gc }
System.print(Map.from(gcRecord).toString)
System.print(Map.from({ trailing: 1, }).toString)
System.print(Map.from({ **recordSource, }).toString)
System.print(Map.from({
  multiline: 2,
}).toString)
System.print(Map.from({
  **recordSource,
}).toString)
System.print(Map { before: 0, **tupleSource, after: 9 }.toString)
System.print(Map { **recordSource }.toString)
System.print(Map { **mapSource }.toString)
System.print(Map { **() }.toString)

let trace = []
const laterKey = || { trace.append("later-key"); #later }
const laterValue = || { trace.append("later-value"); 9 }
const duplicate = || {
  const ignored = Map { same: 0, **Map { same: 1 }, [laterKey.call()]: laterValue.call() }
}.on(DuplicateKeyError) |e| { e.message }
System.print(duplicate)
System.print(trace)

let recordTrace = []
const recordLaterKey = || { recordTrace.append("record-later-key"); #recordLater }
const recordLaterValue = || { recordTrace.append("record-later-value"); 10 }
const recordDuplicate = || {
  const ignored = { same: 0, **(same: 1,), [recordLaterKey.call()]: recordLaterValue.call() }
}.on(Error) |e| { e.message }
System.print(recordDuplicate)
System.print(recordTrace)
