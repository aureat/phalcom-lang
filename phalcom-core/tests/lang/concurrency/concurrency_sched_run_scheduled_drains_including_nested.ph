// area: concurrency
// spec: U-SCHED implementation-spec.md §6 (`System.runScheduled`)
// status: PASS
// `System.runScheduled` drains everything queued so far, in order, then
// returns — including a fiber a running scheduled fiber itself schedules
// mid-drain (`System.nextScheduled` is re-read every loop iteration), before
// control returns to `main`.

System.schedule(|| {
  System.print("outer")
  System.schedule(|| { System.print("nested") })
})
System.runScheduled()
System.print("after-run-scheduled")
