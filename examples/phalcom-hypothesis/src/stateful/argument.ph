// Immutable rule-source metadata and immutable action arguments.

import Strategy from "strategies/strategy"
import Bundle from "stateful/bundle"

@data
@immutable
@sealed
class RuleArgument {
  @variant Draw(name:, label:, strategy:)
  @variant Select(name:, label:, bundle:)
  @variant Consume(name:, label:, bundle:)

  @class
  draw(
    name: Symbol,
    label: Option<Symbol>,
    strategy: Strategy<Any>
  ) -> RuleArgument {
    return Draw.new(name: name, label: label, strategy: strategy)
  }

  @class
  select(
    name: Symbol,
    label: Option<Symbol>,
    bundle: Bundle<Any>
  ) -> RuleArgument {
    return Select.new(name: name, label: label, bundle: bundle)
  }

  @class
  consume(
    name: Symbol,
    label: Option<Symbol>,
    bundle: Bundle<Any>
  ) -> RuleArgument {
    return Consume.new(name: name, label: label, bundle: bundle)
  }

  name -> Symbol {
    return self.match(
      draw: |value| { value.name },
      select: |value| { value.name },
      consume: |value| { value.name }
    )
  }

  label -> Option<Symbol> {
    return self.match(
      draw: |value| { value.label },
      select: |value| { value.label },
      consume: |value| { value.label }
    )
  }

  requiresBundle -> Bool {
    return self.match(
      draw: |_| { false },
      select: |_| { true },
      consume: |_| { true }
    )
  }

  bundleValue -> Option<Bundle<Any>> {
    return self.match(
      draw: |_| { None },
      select: |value| { Some.new(value.bundle) },
      consume: |value| { Some.new(value.bundle) }
    )
  }

  consuming -> Bool {
    return self.match(
      draw: |_| { false },
      select: |_| { false },
      consume: |_| { true }
    )
  }

  fingerprint -> String {
    return self.match(
      draw: |value| {
        "draw(" + value.name.toString + ":" + value.strategy.fingerprint + ")"
      },
      select: |value| {
        "select(" + value.name.toString + ":" + value.bundle.fingerprint + ")"
      },
      consume: |value| {
        "consume(" + value.name.toString + ":" + value.bundle.fingerprint + ")"
      }
    )
  }
}

@data
@immutable
class ResultReference {
  const _id: Int
  const _name: Symbol
  const _producerIndex: Int
  const _bundles: List<Symbol>

  executable -> String => _name.toString
}

@data
@immutable
class LiteralArgument {
  const _name: Symbol
  const _label: Option<Symbol>
  const _value: Any

  executable -> String {
    const rendered = _value.toString
    if _label.isSome {
      return _label.unwrap.toString + ": " + rendered
    }
    return rendered
  }
}

@data
@immutable
class ReferenceArgument {
  const _name: Symbol
  const _label: Option<Symbol>
  const _reference: ResultReference

  executable -> String {
    const rendered = _reference.executable
    if _label.isSome {
      return _label.unwrap.toString + ": " + rendered
    }
    return rendered
  }
}
