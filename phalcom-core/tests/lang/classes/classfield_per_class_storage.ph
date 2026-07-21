class Base {
  @class _count = 0
  @class bump() { _count = _count + 1 }
  @class count => _count
}
class Derived extends Base {}
Base.bump()
Base.bump()
System.print(Base.count)
System.print(Derived.count)
