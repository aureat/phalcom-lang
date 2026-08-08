// Immutable reflected rule definitions and complete machine metadata.

import Bundle from "stateful/bundle"
import RuleArgument from "stateful/argument"

@data
@immutable
@sealed
class _RuleKind {
  @variant Initializer
  @variant Normal

  @class
  initializer -> _RuleKind => Initializer.new()

  @class
  normal -> _RuleKind => Normal.new()
}

@data
@immutable
class RuleDefinition {
  const _method: Method
  const _selector: Symbol
  const _kind: _RuleKind
  const _arguments: List<RuleArgument>
  const _targets: List<Bundle<Any>>
  const _whenSelector: Option<Symbol>

  @class
  initializer(
    method: Method,
    selector: Symbol,
    arguments: List<RuleArgument>,
    targets: List<Bundle<Any>>
  ) -> RuleDefinition {
    return RuleDefinition.new(
      method: method,
      selector: selector,
      kind: _RuleKind.initializer,
      arguments: _RuleCopies.list(arguments),
      targets: _RuleCopies.list(targets),
      whenSelector: None
    )
  }

  @class
  normal(
    method: Method,
    selector: Symbol,
    arguments: List<RuleArgument>,
    targets: List<Bundle<Any>>,
    whenSelector: Option<Symbol>
  ) -> RuleDefinition {
    return RuleDefinition.new(
      method: method,
      selector: selector,
      kind: _RuleKind.normal,
      arguments: _RuleCopies.list(arguments),
      targets: _RuleCopies.list(targets),
      whenSelector: whenSelector
    )
  }

  initializer -> Bool {
    return _kind.match(
      initializer: |_| { true },
      normal: |_| { false }
    )
  }

  normal -> Bool => not self.initializer

  fingerprint -> String {
    const argumentParts = List.new()
    for argument in _arguments {
      argumentParts.add(argument.fingerprint)
    }
    const targetParts = List.new()
    for target in _targets {
      targetParts.add(target.fingerprint)
    }
    let whenPart = "always"
    if _whenSelector.isSome || {
      whenPart = _whenSelector.unwrap.toString
    }
    let kindPart = "rule"
    if self.initializer || {
      kindPart = "initialize"
    }
    return kindPart + "(" + _selector.toString + ")" +
      "[" + argumentParts.join(",") + "]" +
      "->[" + targetParts.join(",") + "]" +
      "@when(" + whenPart + ")"
  }
}

@data
@immutable
class StateMachineMetadata {
  const _machineClass: Class
  const _initializers: List<RuleDefinition>
  const _rules: List<RuleDefinition>
  const _invariants: List<Method>
  const _teardown: Option<Method>
  const _fingerprint: String

  @class
  create(
    machineClass: Class,
    initializers: List<RuleDefinition>,
    rules: List<RuleDefinition>,
    invariants: List<Method>,
    teardown: Option<Method>
  ) -> StateMachineMetadata {
    const definitions = List.new()
    for initializer in initializers {
      definitions.add(initializer.fingerprint)
    }
    for rule in rules {
      definitions.add(rule.fingerprint)
    }
    const invariantParts = List.new()
    for invariant in invariants {
      invariantParts.add(invariant.selector.toString)
    }
    let teardownPart = "none"
    if teardown.isSome || {
      teardownPart = teardown.unwrap.selector.toString
    }
    return StateMachineMetadata.new(
      machineClass: machineClass,
      initializers: _RuleCopies.list(initializers),
      rules: _RuleCopies.list(rules),
      invariants: _RuleCopies.list(invariants),
      teardown: teardown,
      fingerprint: "stateful-v1|" + machineClass.name.toString + "|" +
        definitions.join("|") + "|invariants=" + invariantParts.join(",") +
        "|teardown=" + teardownPart
    )
  }
}

class _RuleCopies {
  @class
  list<T>(values: List<T>) -> List<T> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }
}
