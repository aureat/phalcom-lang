// Stateful attributes are passive reflected metadata. They never wrap methods,
// alter dispatch, or install implementation into a machine class.

import Strategy from "strategies/strategy"

@On(Method)
class Rule is Attribute {
  @constructor
  new(*parts: Any) {
    _parts = _StatefulAttributeCopies.list(parts)
  }

  parts -> List<Any> { _StatefulAttributeCopies.list(_parts) }
}

@On(Method)
class Initialize is Attribute {
  @constructor
  new(*parts: Any) {
    _parts = _StatefulAttributeCopies.list(parts)
  }

  parts -> List<Any> { _StatefulAttributeCopies.list(_parts) }
}

@On(Method)
class StateInvariant is Attribute {
  @constructor
  new() {}
}

@On(Method)
class When is Attribute {
  @constructor
  new(predicate: Any) {
    _predicate = predicate
  }

  predicate -> Any { _predicate }
}

@On(Method)
class Teardown is Attribute {
  @constructor
  new() {}
}

class _StatefulAttributeCopies {
  @class
  list<T>(values: List<T>) -> List<T> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }
}
