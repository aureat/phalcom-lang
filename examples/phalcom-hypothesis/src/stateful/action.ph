// Immutable executed state-machine actions.

import arguments from "stateful/argument"

@data
@immutable
@sealed
class _StateActionKind {
  @variant Initializer
  @variant Normal

  @class
  initializer -> _StateActionKind => Initializer.new()

  @class
  normal -> _StateActionKind => Normal.new()
}

@data
@immutable
class StateAction {
  const _index: Int
  const _kind: _StateActionKind
  const _selector: Symbol
  const _arguments: List<Any>
  const _resultReference: Option<arguments.ResultReference>

  @class
  initializer(
    index: Int,
    selector: Symbol,
    arguments: List<Any>
  ) -> StateAction {
    return StateAction.new(
      index: index,
      kind: _StateActionKind.initializer,
      selector: selector,
      arguments: _StateActionCopies.list(arguments),
      resultReference: None
    )
  }

  @class
  normal(
    index: Int,
    selector: Symbol,
    arguments: List<Any>
  ) -> StateAction {
    return StateAction.new(
      index: index,
      kind: _StateActionKind.normal,
      selector: selector,
      arguments: _StateActionCopies.list(arguments),
      resultReference: None
    )
  }

  initializer -> Bool {
    return _kind.match(
      initializer: |_| { true },
      normal: |_| { false }
    )
  }

  normal -> Bool => not self.initializer

  withResultReference(
    value: Option<arguments.ResultReference>
  ) -> StateAction {
    return StateAction.new(
      index: _index,
      kind: _kind,
      selector: _selector,
      arguments: _StateActionCopies.list(_arguments),
      resultReference: value
    )
  }

  executableLine -> String {
    const rendered = List.new()
    for argument in _arguments {
      rendered.add(argument.executable)
    }
    const call = "state." + _selector.toString + "(" + rendered.join(", ") + ")"
    if _resultReference.isSome {
      return _resultReference.unwrap.executable + " = " + call
    }
    return call
  }

  toString -> String => self.executableLine
}

class _StateActionCopies {
  @class
  list<T>(values: List<T>) -> List<T> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }
}
