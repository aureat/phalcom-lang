// area: blocks
// spec: blocks.md §5; functions.md §1-2
// status: PASS
// A block passed as a method argument, stashed on an instance field, and
// RETURNED by a later method call — the caller then invokes it itself. The
// block is never called inside the method that received it, only round-
// tripped through the object, proving blocks are ordinary first-class values
// across method-call boundaries.
class Box {
  store(_ block) {
    _block = block
    return self
  }
  fetch() {
    return _block
  }
}
const box = Box.new()
box.store({ 6 * 7 })
const retrieved = box.fetch()
System.print(retrieved.call())
