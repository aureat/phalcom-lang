// Opt-in constructor and sealed-hierarchy strategy derivation.
//
// Derivation consumes ordinary reflected annotations and produces ordinary
// Strategy values. Generated values therefore use the same DrawData, replay,
// spans, shrinking, persistence, and reporting path as explicit strategies.

import Strategy from "strategies/strategy"
import DrawData from "choices/data"
import Gen from "strategies/gen"
import base from "strategies/combinators"
import attributes from "strategies/attributes"
import errors from "core/errors"

@data
@immutable
class _DerivedParameter {
  const _name: Symbol
  const _label: Option<Symbol>
  const _type: Any
  const _strategy: Strategy<Any>

  fingerprint -> String {
    let label = "_"
    if _label.isSome || {
      label = _label.unwrap.toString
    }
    return _name.toString + ":" + label + ":" +
      _type.toString + "=" + _strategy.fingerprint
  }
}

class _DerivedStrategy<T> is base.StrategyBase<T> {
  @constructor
  new(inner: Strategy<T>, fingerprint: String) {
    _inner = inner
    _fingerprint = fingerprint
  }

  draw(data: DrawData) -> T => _inner.draw(data)

  fingerprint -> String => _fingerprint
}

class _ConstructorStrategy<T> is base.StrategyBase<T> {
  @constructor
  new(
    targetType: Any,
    constructor: Method,
    parameters: List<_DerivedParameter>,
    fingerprint: String
  ) {
    _targetType = targetType
    _constructor = constructor
    _parameters = _DerivationCopies.list(parameters)
    _fingerprint = fingerprint
  }

  draw(data: DrawData) -> T {
    return data.withSpan(label: #derivedConstructor, discardable: false) {
      const arguments = List.new()
      for parameter in _parameters {
        arguments.add(parameter.strategy.draw(data))
      }
      return _constructor.invokeOn(_targetType, arguments)
    }
  }

  fingerprint -> String => _fingerprint
}

class _Derivation {
  @class
  derive(
    type: Any,
    registry: Any,
    path: List<String>
  ) -> Strategy<Any> {
    if not self.isArbitrary(type) {
      throw self.failure(
        message: "no exact strategy is registered and the type is not marked @arbitrary",
        path: path
      )
    }

    if self.isSealed(type) {
      return self.deriveSealed(type: type, registry: registry, path: path)
    }

    return self.deriveConstructor(
      type: type,
      registry: registry,
      path: path,
      recursiveRoot: None,
      recursiveReplacement: None
    )
  }

  @class
  isArbitrary(type: Any) -> Bool {
    if not type.respondsTo(#attributesOfType) {
      return false
    }
    return type.attributesOfType(attributes.arbitrary).toList.size == 1
  }

  @class
  isSealed(type: Any) -> Bool {
    if type.respondsTo(#variants) {
      return type.variants.toList.size > 0
    }
    if type.respondsTo(#isSealed) {
      return type.isSealed
    }
    return false
  }

  @class
  deriveConstructor(
    type: Any,
    registry: Any,
    path: List<String>,
    recursiveRoot: Option<Any>,
    recursiveReplacement: Option<Strategy<Any>>
  ) -> Strategy<Any> {
    const constructor = self.primaryConstructor(type: type, path: path)
    self.requireSafeConstructor(
      type: type,
      constructor: constructor,
      path: path
    )

    const parameters = List.new()
    const fingerprintParts = List.new()
    for parameter in constructor.parameters || {
      if parameter.respondsTo(#isRest) and parameter.isRest || {
        throw self.failure(
          message: "unsafe automatic derivation for rest parameter '" +
            parameter.name.toString + "'; register a custom strategy",
          path: self.extend(path: path, segment: parameter.name.toString)
        )
      }

      if parameter.type.isNone || {
        throw self.failure(
          message: "constructor parameter '" + parameter.name.toString +
            "' has no reflected type annotation",
          path: self.extend(path: path, segment: parameter.name.toString)
        )
      }

      let label = None
      if parameter.respondsTo(#label) {
        label = parameter.label
      }
      const parameterType = parameter.type.unwrap
      const parameterPath = self.extend(
        path: path,
        segment: constructor.selector.toString + "." + parameter.name.toString
      )
      const resolved = self.resolveParameter(
        type: parameterType,
        registry: registry,
        path: parameterPath,
        recursiveRoot: recursiveRoot,
        recursiveReplacement: recursiveReplacement
      )
      const descriptor = _DerivedParameter.new(
        name: parameter.name,
        label: label,
        type: parameterType,
        strategy: resolved
      )
      parameters.add(descriptor)
      fingerprintParts.add(descriptor.fingerprint)
    }

    const fingerprint = "construct(" + type.name.toString + ":" +
      constructor.selector.toString + "[" + fingerprintParts.join(",") + "])"
    return _ConstructorStrategy.new(
      targetType: type,
      constructor: constructor,
      parameters: parameters,
      fingerprint: fingerprint
    )
  }

  @class
  deriveSealed(
    type: Any,
    registry: Any,
    path: List<String>
  ) -> Strategy<Any> {
    const variants = self.stableVariants(type)
    if variants.size == 0 {
      throw self.failure(
        message: "sealed @arbitrary type has no reflected variants",
        path: path
      )
    }

    const terminalVariants = List.new()
    const recursiveVariants = List.new()
    for variant in variants {
      if self.variantContainsType(variant: variant, root: type, path: path) {
        recursiveVariants.add(variant)
      } else {
        terminalVariants.add(variant)
      }
    }

    if terminalVariants.size == 0 {
      throw self.failure(
        message: "recursive sealed hierarchy has no terminal variant; " +
          "register a custom strategy",
        path: path
      )
    }

    const terminalStrategies = List.new()
    for variant in terminalVariants {
      terminalStrategies.add(
        self.deriveConstructor(
          type: variant,
          registry: registry,
          path: self.extend(path: path, segment: variant.name.toString),
          recursiveRoot: None,
          recursiveReplacement: None
        )
      )
    }

    const terminal = Gen.oneOf(*terminalStrategies)
    const variantNames = List.new()
    for variant in variants {
      variantNames.add(variant.name.toString)
    }

    if recursiveVariants.size == 0 {
      return _DerivedStrategy.new(
        inner: terminal,
        fingerprint: "sealed(" + type.name.toString + ":" +
          variantNames.join(",") + ";" + terminal.fingerprint + ")"
      )
    }

    const recursive = Gen.recursive(
      base: terminal,
      extend: |child| {
        const choices = _DerivationCopies.list(terminalStrategies)
        for variant in recursiveVariants {
          choices.add(
            self.deriveConstructor(
              type: variant,
              registry: registry,
              path: self.extend(path: path, segment: variant.name.toString),
              recursiveRoot: Some.new(type),
              recursiveReplacement: Some.new(child)
            )
          )
        }
        return Gen.oneOf(*choices)
      }
    )
    return _DerivedStrategy.new(
      inner: recursive,
      fingerprint: "recursive-sealed(" + type.name.toString + ":" +
        variantNames.join(",") + ";base=" + terminal.fingerprint + ")"
    )
  }

  @class
  stableVariants(type: Any) -> List<Class> {
    let reflected = const []
    if type.respondsTo(#variants) {
      reflected = type.variants.toList
    } else if type.respondsTo(#subclasses) {
      reflected = type.subclasses.toList
    }
    return reflected.sorted |left, right| {
      left.name.toString < right.name.toString
    }
  }

  @class
  variantContainsType(
    variant: Class,
    root: Any,
    path: List<String>
  ) -> Bool {
    const constructor = self.primaryConstructor(type: variant, path: path)
    for parameter in constructor.parameters || {
      if parameter.type.isSome and self.containsType(parameter.type.unwrap, root: root) {
        return true
      }
    }
    return false
  }

  @class
  containsType(type: Any, root: Any) -> Bool {
    if type == root {
      return true
    }
    if type.respondsTo(#origin) and type.respondsTo(#arguments) {
      for argument in type.arguments || {
        if self.containsType(argument, root: root) {
          return true
        }
      }
    }
    return false
  }

  @class
  resolveParameter(
    type: Any,
    registry: Any,
    path: List<String>,
    recursiveRoot: Option<Any>,
    recursiveReplacement: Option<Strategy<Any>>
  ) -> Strategy<Any> {
    if recursiveRoot.isSome and type == recursiveRoot.unwrap || {
      return recursiveReplacement.unwrap
    }

    if recursiveRoot.isSome and type.respondsTo(#origin) and
      type.respondsTo(#arguments) and self.containsType(type, root: recursiveRoot.unwrap) {
      return self.resolveRecursiveApplied(
        type: type,
        registry: registry,
        path: path,
        recursiveRoot: recursiveRoot.unwrap,
        recursiveReplacement: recursiveReplacement.unwrap
      )
    }

    return registry.resolve(type: type, path: path)
  }

  @class
  resolveRecursiveApplied(
    type: Any,
    registry: Any,
    path: List<String>,
    recursiveRoot: Any,
    recursiveReplacement: Strategy<Any>
  ) -> Strategy<Any> {
    const origin = type.origin
    const arguments = type.arguments

    if origin == Option {
      self.requireArity(type: type, arguments: arguments, expected: 1, path: path)
      return Gen.option(
        self.resolveParameter(
          type: arguments.at(0),
          registry: registry,
          path: self.extend(path: path, segment: "Option value"),
          recursiveRoot: Some.new(recursiveRoot),
          recursiveReplacement: Some.new(recursiveReplacement)
        )
      )
    }

    if origin == List {
      self.requireArity(type: type, arguments: arguments, expected: 1, path: path)
      return Gen.list(
        of: self.resolveParameter(
          type: arguments.at(0),
          registry: registry,
          path: self.extend(path: path, segment: "List element"),
          recursiveRoot: Some.new(recursiveRoot),
          recursiveReplacement: Some.new(recursiveReplacement)
        )
      )
    }

    if origin == Set {
      self.requireArity(type: type, arguments: arguments, expected: 1, path: path)
      return Gen.set(
        of: self.resolveParameter(
          type: arguments.at(0),
          registry: registry,
          path: self.extend(path: path, segment: "Set element"),
          recursiveRoot: Some.new(recursiveRoot),
          recursiveReplacement: Some.new(recursiveReplacement)
        )
      )
    }

    if origin == Tuple {
      const strategies = List.new()
      let index = 0
      for argument in arguments {
        strategies.add(
          self.resolveParameter(
            type: argument,
            registry: registry,
            path: self.extend(
              path: path,
              segment: "Tuple argument " + index.toString
            ),
            recursiveRoot: Some.new(recursiveRoot),
            recursiveReplacement: Some.new(recursiveReplacement)
          )
        )
        index++
      }
      return Gen.tuple(*strategies)
    }

    if origin == Map {
      self.requireArity(type: type, arguments: arguments, expected: 2, path: path)
      return Gen.map(
        keys: self.resolveParameter(
          type: arguments.at(0),
          registry: registry,
          path: self.extend(path: path, segment: "Map key"),
          recursiveRoot: Some.new(recursiveRoot),
          recursiveReplacement: Some.new(recursiveReplacement)
        ),
        values: self.resolveParameter(
          type: arguments.at(1),
          registry: registry,
          path: self.extend(path: path, segment: "Map value"),
          recursiveRoot: Some.new(recursiveRoot),
          recursiveReplacement: Some.new(recursiveReplacement)
        )
      )
    }

    if origin == Result {
      self.requireArity(type: type, arguments: arguments, expected: 2, path: path)
      return Gen.result(
        ok: self.resolveParameter(
          type: arguments.at(0),
          registry: registry,
          path: self.extend(path: path, segment: "Result ok"),
          recursiveRoot: Some.new(recursiveRoot),
          recursiveReplacement: Some.new(recursiveReplacement)
        ),
        error: self.resolveParameter(
          type: arguments.at(1),
          registry: registry,
          path: self.extend(path: path, segment: "Result error"),
          recursiveRoot: Some.new(recursiveRoot),
          recursiveReplacement: Some.new(recursiveReplacement)
        )
      )
    }

    throw self.failure(
      message: "recursive field uses unsupported applied type " + type.toString,
      path: path
    )
  }

  @class
  primaryConstructor(type: Any, path: List<String>) -> Method {
    const constructors = self.constructors(type)
    if constructors.size == 0 {
      throw self.failure(
        message: "no reflected @constructor is available for automatic derivation",
        path: path
      )
    }
    if constructors.size > 1 {
      throw self.failure(
        message: "unsafe automatic derivation with multiple constructors; " +
          "register a custom strategy",
        path: path
      )
    }
    return constructors.at(0)
  }

  @class
  constructors(type: Any) -> List<Method> {
    if type.respondsTo(#constructors) {
      return type.constructors.toList.sorted |left, right| {
        left.selector.toString < right.selector.toString
      }
    }

    const reflected = List.new()
    if type.respondsTo(#methods) and type.respondsTo(#methodFor) {
      for selector in type.methods || {
        const method = type.methodFor(selector)
        if method.respondsTo(#isConstructor) and method.isConstructor || {
          reflected.add(method)
        }
      }
    }
    return reflected.sorted |left, right| {
      left.selector.toString < right.selector.toString
    }
  }

  @class
  requireSafeConstructor(
    type: Any,
    constructor: Method,
    path: List<String>
  ) -> None {
    if self.hasPreconditions(constructor) {
      throw self.failure(
        message: "constrained constructor " + type.name.toString + "." +
          constructor.selector.toString + " is unsafe for automatic derivation; " +
          "register a custom strategy with @strategy(" + type.name.toString + ")",
        path: path
      )
    }
  }

  @class
  hasPreconditions(constructor: Method) -> Bool {
    if constructor.respondsTo(#preconditions) and
      constructor.preconditions.toList.size > 0 {
      return true
    }
    if constructor.respondsTo(#contracts) and
      constructor.contracts.respondsTo(#requires) and
      constructor.contracts.requires.toList.size > 0 {
      return true
    }
    if constructor.respondsTo(#attributes) {
      for attribute in constructor.attributes || {
        const name = attribute.class.name.toString
        if name == "requires" or name == "Requires" {
          return true
        }
      }
    }
    return false
  }

  @class
  requireArity(
    type: Any,
    arguments: List<Any>,
    expected: Int,
    path: List<String>
  ) -> None {
    if arguments.size != expected {
      throw self.failure(
        message: type.toString + " expected " + expected.toString +
          " type arguments, received " + arguments.size.toString,
        path: path
      )
    }
  }

  @class
  extend(path: List<String>, segment: String) -> List<String> {
    const copied = _DerivationCopies.list(path)
    copied.add(segment)
    return copied
  }

  @class
  failure(message: String, path: List<String>) -> errors.StrategyResolutionError || {
    return errors.StrategyResolutionError.new(
      message + "; resolution path: " + path.join(" -> ")
    )
  }
}

class _DerivationCopies {
  @class
  list<T>(values: List<T>) -> List<T> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }
}
