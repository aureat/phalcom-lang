// area: concurrency
// spec: concurrency.md; ADR-0030 §3
// status: NEGATIVE

// A parent that is waiting for a child is not manually resumable. The child
// must not be able to steal the parent's active control-transfer boundary.
let parent = None
let child = None

parent = Fiber.new || {
  child = Fiber.new || {
    parent.call()
  }
  child.call()
}

parent.call()
