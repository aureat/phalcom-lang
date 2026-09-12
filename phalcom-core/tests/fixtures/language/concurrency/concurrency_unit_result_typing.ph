// area: concurrency
// spec: 01-core-type-lattice-and-unit.md §5; concurrency.md §2
class UnitResultProbe {
  @class
  method(_ param: Int) -> Result<(), Error> {
    Ok(())
  }

  @class
  same(_ result: Result<Unit, Error>) -> Result<(), Error> {
    result
  }
}
const result: Result<Unit, Error> = UnitResultProbe.method(1)
System.print(UnitResultProbe.same(result).isOk)
const action = Future.async || { () }
System.runScheduled()
System.print(action.await)
