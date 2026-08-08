// Passive property metadata retained on reflected methods.

import Settings from "core/settings"
import Strategy from "strategies/strategy"
import errors from "core/errors"

@data
@immutable
class _GivenOverride {
  const _name: Symbol
  const _strategy: Strategy<Any>
}

@immutable
class GivenArgs {
  const _overrides: List<_GivenOverride>

  @constructor
  new() {
    _overrides = const []
  }

  @constructor
  @private
  from(overrides: List<_GivenOverride>) {
    _overrides = _AttributeCopies.list(overrides)
  }

  for(name: Symbol, use: Strategy<Any>) -> GivenArgs {
    if not use.respondsTo(#draw) {
      throw errors.PropertyDiscoveryError.new(
        "@Given override '" + name.toString + "' is not a Strategy"
      )
    }

    for existing in _overrides {
      if existing.name == name {
        throw errors.PropertyDiscoveryError.new(
          "duplicate override for parameter '" + name.toString + "'"
        )
      }
    }

    const next = _AttributeCopies.list(_overrides)
    next.add(_GivenOverride.new(name: name, strategy: use))
    return GivenArgs.from(overrides: next)
  }

  strategyFor(name: Symbol) -> Option<Strategy<Any>> {
    for override in _overrides {
      if override.name == name {
        return Some.new(override.strategy)
      }
    }
    return None
  }

  names -> List<Symbol> {
    const names = List.new()
    for override in _overrides {
      names.add(override.name)
    }
    return names
  }

  size -> Int => _overrides.size
}

@data
@immutable
@sealed
class GivenMode {
  @variant Inferred
  @variant Explicit(strategies:)
  @variant Overrides(arguments:)

  @class
  inferred -> GivenMode => Inferred.new()

  @class
  explicit(strategies: List<Strategy<Any>>) -> GivenMode {
    return Explicit.new(strategies: _AttributeCopies.list(strategies))
  }

  @class
  overrides(arguments: GivenArgs) -> GivenMode {
    return Overrides.new(arguments: arguments)
  }
}

@On(Method)
class Given is Attribute {
  @constructor
  new(*parts: Any) {
    if parts.size == 0 {
      _mode = GivenMode.inferred
      return
    }

    if parts.size == 1 and parts.at(0).isA(GivenArgs) {
      _mode = GivenMode.overrides(parts.at(0))
      return
    }

    const strategies = List.new()
    for part in parts {
      if not part.respondsTo(#draw) {
        throw errors.PropertyDiscoveryError.new(
          "explicit @Given arguments must all be Strategy values"
        )
      }
      strategies.add(part)
    }
    _mode = GivenMode.explicit(strategies)
  }

  mode -> GivenMode => _mode

  // Compatibility getter for explicit-mode consumers. Inferred and override
  // modes intentionally expose an empty list rather than pretending to have
  // been resolved before method reflection.
  strategies -> List<Strategy<Any>> {
    return _mode.match(
      inferred: |_| { const [] },
      explicit: |value| { _AttributeCopies.list(value.strategies) },
      overrides: |_| { const [] }
    )
  }
}

@On(Method)
class Case is Attribute {
  @constructor
  new(*values: Any) {
    _values = _AttributeCopies.list(values)
  }

  values -> List<Any> => _AttributeCopies.list(_values)
}

@On(Method)
class WithSettings is Attribute {
  @constructor
  new(settings: Settings) {
    _settings = settings
  }

  settings -> Settings => _settings
  config -> Settings => _settings
}

class _AttributeCopies {
  @class
  list<T>(values: List<T>) -> List<T> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }
}
