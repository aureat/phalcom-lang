class Counter {
  @class _count = 1
  @class bump() { _count = _count + 1 }
  @class count => _count
  @class count=(value) { _count = value }
}
Counter.bump()
System.print(Counter.count)
Counter.count = 7
System.print(Counter.count)
