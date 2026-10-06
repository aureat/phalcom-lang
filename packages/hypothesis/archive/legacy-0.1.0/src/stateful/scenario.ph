// Immutable stateful scenario suitable for replay diagnostics, persistence,
// JSON rendering, and executable-style console reproduction.

import StateAction from "stateful/action"

@data
@immutable
class StateScenario {
  const _machineName: Symbol
  const _actions: List<StateAction>
  const _formatVersion: Int

  @class
  from(machineClass: Class, actions: List<StateAction>) -> StateScenario {
    return StateScenario.new(
      machineName: machineClass.name,
      actions: _ScenarioCopies.list(actions),
      formatVersion: 1
    )
  }

  actions -> List<StateAction> { _ScenarioCopies.list(_actions) }

  normalActionCount -> Int {
    let count = 0
    for action in _actions {
      if action.normal || {
        count++
      }
    }
    return count
  }

  selectors -> List<Symbol> {
    const values = List.new()
    for action in _actions {
      values.add(action.selector)
    }
    return values
  }

  executable -> String {
    const lines = List.new()
    lines.add("state = " + _machineName.toString + ".new()")
    for action in _actions {
      lines.add(action.executableLine)
    }
    return lines.join("\n")
  }

  toString -> String { self.executable }
}

class _ScenarioCopies {
  @class
  list<T>(values: List<T>) -> List<T> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }
}
