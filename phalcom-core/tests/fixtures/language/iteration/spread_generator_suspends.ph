// area: iteration
// spec: F.2 outgoing packs §30/§34.13; iteration.md; ADR-0030
// status: PASS
// A generic positional `*` is compiler-generated cursor bytecode. The spread
// must be able to park the running Fiber inside an ordinary Iterable method,
// then resume with builder/source/cursor state intact until the final send.

class YieldingSource is Iterable {
  iterate(_ cursor) {
    let next = (cursor == None).ifTrue(|| { 1 }, ifFalse: || { cursor + 1 })
    return (next <= 3).ifTrue(|| {
      Fiber.yield(next * 10)
      next
    }, ifFalse: || { None })
  }

  iteratorValue(_ cursor) { return cursor }
}

class Receiver {
  collect(_ a, _ b, _ c) { return a * 100 + b * 10 + c }
}

const f = Fiber.new || {
  Receiver.new().collect(*YieldingSource.new())
}

System.print(f.call())
System.print(f.call())
System.print(f.call())
System.print(f.call())
