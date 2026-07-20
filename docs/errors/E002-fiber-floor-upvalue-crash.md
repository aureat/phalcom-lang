# E002 · Fiber-floor failure capture drops the live stack without closing open upvalues

- **Status:** FIXED — `a265684` (2026-07-20). `run_until`'s fiber-floor `Err` arm now closes the originating fiber's live open upvalues (`close_upvalues_from(0)`, against the still-live `VM::stack`, before any switch repoints it) and each intermediate `Call`-mode resumer's parked open upvalues (new fiber-scoped `close_fiber_upvalues_from`, against that resumer's own parked `FiberObject::stack`) before either is cleared. Regression coverage: `tests/lang/concurrency/concurrency_fiber_floor_upvalue_close.ph` (originating fiber) and `concurrency_fiber_floor_upvalue_close_cascade.ph` (intermediate `Call`-mode resumer); both confirmed to panic on the pre-fix tree and pass post-fix.
- **Severity:** blocker — deterministic crash (`index out of bounds` panic)
- **Subsystem:** fibers / upvalue lifecycle
- **Related:** [E001](E001-gc-ensure-temp-root-uaf.md) (same family — value held live across a boundary the scan misses); seam DEC-FIB-A (U-FIBER owns fiber-floor capture)

## Defect

When a fiber fails uncaught, the fiber-floor `Err` arm of `run_until`
(`phalcom-core/src/vm/dispatch.rs` ~L290-338) discards the failed fiber's live
`frames` / `stack` / `open_upvalues` — via `load_live_from`
(`phalcom-core/src/primitive/fiber.rs:55-57`) — **without closing the open
upvalues first**.

A block that captured a fiber local and escaped (e.g. to a module global) is left
with an `Upvalue::Open { fiber, slot }` pointing at the now-empty parked stack.
Calling that block later hits `GetUpvalue` → `heap.fiber(fiber).stack[slot]` at
`dispatch.rs:1062` → `index out of bounds: the len is 0 but the index is 1`.

## Reproduction

Panics under `target/debug/phalcom`:

```phalcom
let leak = { 0 }
let b = Fiber.new {
  let x = 42
  leak = { x }             // block capturing x escapes to a module global
  Fiber.abort(Error.new()) // uncaught failure -> fiber-floor capture, no unwind
}
b.try()                    // b marked Failed; its live stack dropped, upvalue left Open
System.print(leak.call())  // GetUpvalue -> Open{fiber:b, slot} -> b.stack[slot] -> panic
```

> **Repro updated 2026-07-19.** As first recorded this used `var`, which stopped lexing when
> U-BINDINGS removed `Token::Var` (`42aafce`) — the file failed with `Expected one of ";", newline`
> before reaching the crash. With `let` it panics exactly as recorded.

The defect is neither mode- nor API-specific: the same panic reproduces with
`throw` + `try()` in place of `Fiber.abort` + `try()`, and with `call()`.

## Fix direction (NOT implemented / NOT verified)

The fiber-floor `Err` arm must `close_upvalues_from(0)` on the failing fiber's
live mirror before the switch, and close each cascaded resumer's parked
`open_upvalues` before `.clear()`.

This matches the VM's **own** documented unwind invariant at `dispatch.rs:96-103`
("close upvalues first, then truncate … so a closure that escaped the throwing
block still observes its captured locals rather than a use-after-free") — the
failure path simply skips it. Re-derive + full suite + repro before trusting;
commit narrow on `main`.

> **Aggravation noted 2026-07-20:** with the `fiber-pool` feature on, the failed
> fiber's cleared buffers are recycled into new fibers
> (`primitive/fiber.rs:136-140`), so the dangling `Upvalue::Open { fiber, slot }`
> can degrade from a deterministic panic to a **silent stale read** of an
> unrelated fiber's stack slot. The pool is off by default (measured net
> negative), but any future re-enable inherits this.
