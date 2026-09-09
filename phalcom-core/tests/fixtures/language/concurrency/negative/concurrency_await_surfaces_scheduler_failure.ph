// area: concurrency
// spec: concurrency.md; E010
// status: NEGATIVE

// Root await keeps its quiescence diagnostic, but includes the unhandled
// scheduler failures produced while this await itself drove the scheduler.
// The top-level form is required: `try` would place the scheduler switch
// beneath the native `block_on` frame and must remain guarded.
const pending = Future.new()
System.schedule || {
  Error.new("settler failed").raise()
}
pending.await
