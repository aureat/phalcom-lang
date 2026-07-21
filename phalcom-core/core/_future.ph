//! Module-level phaldoc
//! 
//! System object provides methods for interacting with the system.
//! NOTE: THIS FILE IS NOT MEANT TO BE IMPORTED

/// Class-level phaldoc
/// The `System` object provides methods for interacting with the system.
@sealed
class System {
  /// U-STRING write funnel (ADR-0049 amendment): pure `.ph` control flow over
  /// native `write_(_)` and the `toString` message. Additive-only: does not
  /// touch the native `print(_)` pathway (pre-existing divergence between
  /// `Value::to_string` and the `.toString` message is out of scope).
  @native @class
  write(obj) {
    System.writeObject_(obj)
    obj
  }

  @native @class
  writeObject_(obj) {
    const s = obj.toString
    (s is String)
      .ifTrue({ write_(s) }, 
       ifFalse: { write_("invalid toString") })
    obj
  }

  /// U-SCHED: the `.ph`-callable counterpart to `VM::run`'s native
  /// root-drive belt-and-suspenders pump (`vm/dispatch.rs`) — pumps
  /// `System.nextScheduled` to exhaustion, `try()`-resuming each queued
  /// fiber (capture-not-propagate, so one scheduled task's uncaught raise
  /// cannot abort another) — including any fiber a running scheduled fiber
  /// itself schedules mid-drain, since `nextScheduled` is re-read every
  /// iteration. Deliberately does **not** unwrap via `.match(some:none:)`
  /// (which runs its arm through `Block#call`'s native re-entrant
  /// `run_until`, forbidding a fiber switch underneath, ADR-0030 §4): `f.try()`
  /// must run at this method's own top level, not nested inside a block a
  /// native primitive is driving, so the receiver is unwrapped via
  /// `unwrapOr(_)` into a plain local first, and `try()` sent as its own
  /// statement.
  @native @class
  runScheduled() {
    let next = nextScheduled
    while (next.isSome) {
      let f = next.unwrapOr(None)
      f.try()
      next = nextScheduled
    }
  }
}