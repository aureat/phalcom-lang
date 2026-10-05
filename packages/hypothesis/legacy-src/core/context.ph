// Per-example property context and stack-disciplined installation.
//
// The stack is process-local until the runtime exposes ContextLocal<T>.
// Cleanup is nevertheless structurally guaranteed through ensure.

import errors from "core/errors"

class _PropertyContext {
  @constructor
  new() {
    _notes = List.new()
    _events = Map.new()
  }

  notes -> List<Any> {
    const copied = List.new()
    for note in _notes {
      copied.add(note)
    }
    return copied
  }

  events -> Map<Symbol, Int> {
    const copied = Map.new()
    _events.entries.each |entry| {
      copied.at(entry.key, put: entry.value)
    }
    return copied
  }

  note(value: Any) -> None {
    _notes.add(value)
  }

  event(label: Symbol) -> None {
    let count = _events.at(label)
    if count == None {
      count = 0
    }
    _events.at(label, put: count + 1)
  }
}

class _PropertyContextStack {
  @constructor
  new() {
    _items = List.new()
  }

  push(context: _PropertyContext) -> _PropertyContext {
    _items.add(context)
    return context
  }

  pop() -> _PropertyContext {
    if _items.size == 0 {
      throw errors._PropertyContextUnderflow.new("property context stack underflow")
    }

    const index = _items.size - 1
    const current = _items.at(index)
    _items.removeAt(index)
    return current
  }

  current -> Option<_PropertyContext> {
    if _items.size == 0 {
      return None
    }

    return Some.new(_items.at(_items.size - 1))
  }

  with(context: _PropertyContext, body: Closure) -> Any {
    self.push(context)
    return || {
      body.call()
    }.ensure || {
      self.pop()
    }
  }
}

const _propertyContexts = _PropertyContextStack.new()
