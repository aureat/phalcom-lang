from universe.concurrency.fiber import System
from universe.errors.error import Error

/// Observability and execution trace sink.
class Tracer {

  /// Returns a standard output tracer.
  @class
  stdout -> Tracer { Tracer.new() }

  /// Records callable entry with argument representation.
  enter(_ name: Object, _ args: Object) -> Unit {
    System.print("-> " + name.toString + " " + args.toString)
  }

  /// Records callable normal exit with result and elapsed time.
  exit(_ name: Object, _ result: Dynamic, _ elapsed: Object) -> Unit {
    System.print("<- " + name.toString + " = " + result.toString)
  }

  /// Records callable abnormal exit with raised error.
  threw(_ name: Object, _ err: Error) -> Unit {
    System.print("!! " + name.toString + " threw " + err.toString)
  }
}

export Tracer
