class Probe {
  construct new(size:) { _size = size }
  size => _size
  iterate(cursor) {
    const next = (cursor == None).ifTrue({ 0 }, ifFalse: { cursor + 1 })
    return (next < self.size).ifTrue({ next }, ifFalse: { None })
  }
}
const p = Probe.new(size: 5)
System.print(p.iterate(None))
