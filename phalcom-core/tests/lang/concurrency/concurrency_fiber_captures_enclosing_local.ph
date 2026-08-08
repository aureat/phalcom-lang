// C-FIB-6: a fiber body that closes over a still-live local of an enclosing,
// currently-parked frame must read and write that local through the *owning*
// fiber's stack, not whichever fiber is running (ADR-0030; upvalue.rs
// `Open { fiber, slot }`). Regression for the cross-fiber open-upvalue panic:
// `gen`'s frame is parked while `f` is resumed, so `x` is reached across fibers.
class Gen {
  @class
  run() {
    let x = 7
    const f = Fiber.new {
      Fiber.yield(x)   // read x across the fiber boundary (gen's frame parked)
      x = 99           // write x back into gen's parked stack
    }
    System.print(f.call())   // -> 7 (the yielded read)
    f.call()                 // resume: runs `x = 99`
    return x                 // gen observes the cross-fiber write
  }
}
System.print(Gen.run())      // -> 99
