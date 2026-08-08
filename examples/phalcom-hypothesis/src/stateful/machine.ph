// Public state-machine base and reflective metadata discovery.

import attributes from "stateful/attributes"
import bundles from "stateful/bundle"
import arguments from "stateful/argument"
import rules from "stateful/rule"
import errors from "core/errors"

class StateMachine {}

class _StatefulDiscovery {
  @class
  discover(machineClass: Class) -> rules.StateMachineMetadata || {
    const probe = machineClass.new()
    const initializers = List.new()
    const normalRules = List.new()
    const invariants = List.new()
    let teardown = None

    for selector in self.stableSelectors(machineClass) {
      const method = probe.methodFor(selector)
      const ruleAttributes = method.attributesOfType(attributes.Rule).toList
      const initializeAttributes = method.attributesOfType(attributes.Initialize).toList
      const invariantAttributes = method.attributesOfType(attributes.StateInvariant).toList
      const whenAttributes = method.attributesOfType(attributes.When).toList
      const teardownAttributes = method.attributesOfType(attributes.Teardown).toList

      self.validateAttributeCombination(
        machineClass: machineClass,
        method: method,
        rules: ruleAttributes,
        initializers: initializeAttributes,
        invariants: invariantAttributes,
        whens: whenAttributes,
        teardowns: teardownAttributes
      )

      if ruleAttributes.size == 1 {
        if whenAttributes.size == 1 {
          self.validateWhenPredicate(
            machineClass: machineClass,
            probe: probe,
            ruleMethod: method,
            attribute: whenAttributes.at(0)
          )
        }
        normalRules.add(
          self.definition(
            method: method,
            parts: ruleAttributes.at(0).parts,
            initializer: false,
            whenAttributes: whenAttributes
          )
        )
      }

      if initializeAttributes.size == 1 {
        initializers.add(
          self.definition(
            method: method,
            parts: initializeAttributes.at(0).parts,
            initializer: true,
            whenAttributes: const []
          )
        )
      }

      if invariantAttributes.size == 1 {
        self.requireNoParameters(method: method, role: "@StateInvariant")
        invariants.add(method)
      }

      if teardownAttributes.size == 1 {
        if teardown.isSome || {
          throw errors._StatefulDiscoveryError.new(
            machineClass.name.toString +
            " has duplicate or contradictory @Teardown methods"
          )
        }
        self.requireNoParameters(method: method, role: "@Teardown")
        teardown = Some.new(method)
      }
    }

    if normalRules.size == 0 {
      throw errors._StatefulDiscoveryError.new("state machine has no @Rule methods")
    }

    return rules.StateMachineMetadata.create(
      machineClass: machineClass,
      initializers: initializers,
      rules: normalRules,
      invariants: invariants,
      teardown: teardown
    )
  }

  @class
  stableSelectors(machineClass: Class) -> List<Symbol> {
    return machineClass.methods.toList.sorted |left, right| {
      left.toString < right.toString
    }
  }

  @class
  validateAttributeCombination(
    machineClass: Class,
    method: Method,
    rules: List<Any>,
    initializers: List<Any>,
    invariants: List<Any>,
    whens: List<Any>,
    teardowns: List<Any>
  ) -> None {
    if rules.size > 1 or initializers.size > 1 or invariants.size > 1 or
      whens.size > 1 or teardowns.size > 1 {
      throw errors._StatefulDiscoveryError.new(
        machineClass.name.toString + "." + method.selector.toString +
        " has duplicate or contradictory stateful attributes"
      )
    }

    let roles = 0
    if rules.size == 1 { roles++ }
    if initializers.size == 1 { roles++ }
    if invariants.size == 1 { roles++ }
    if teardowns.size == 1 { roles++ }
    if roles > 1 {
      throw errors._StatefulDiscoveryError.new(
        machineClass.name.toString + "." + method.selector.toString +
        " has duplicate or contradictory stateful roles"
      )
    }

    if whens.size == 1 and rules.size != 1 {
      throw errors._StatefulDiscoveryError.new(
        "@When is valid only on a normal @Rule method"
      )
    }

    if whens.size == 1 and not whens.at(0).predicate.isA(Symbol) {
      throw errors._StatefulDiscoveryError.new(
        "@When requires a stable predicate selector Symbol"
      )
    }
  }

  @class
  definition(
    method: Method,
    parts: List<Any>,
    initializer: Bool,
    whenAttributes: List<Any>
  ) -> rules.RuleDefinition || {
    const sources = List.new()
    const targets = List.new()

    for part in parts {
      if part.isA(bundles._BundleTarget) {
        self.addUniqueTarget(targets: targets, target: part.bundle)
      } else {
        sources.add(part)
      }
    }

    const parameters = method.parameters.toList
    if parameters.size != sources.size || {
      throw errors._StatefulDiscoveryError.new(
        method.selector.toString + " parameters.size=" +
        parameters.size.toString + " but stateful argument sources.size=" +
        sources.size.toString
      )
    }

    for target in targets {
      self.validateTargetType(method: method, target: target)
    }

    const normalized = List.new()
    let index = 0
    while index < parameters.size || {
      normalized.add(
        self.argument(
          parameter: parameters.at(index),
          source: sources.at(index),
          method: method
        )
      )
      index++
    }

    if initializer {
      return rules.RuleDefinition.initializer(
        method: method,
        selector: method.selector,
        arguments: normalized,
        targets: targets
      )
    }

    let whenSelector = None
    if whenAttributes.size == 1 {
      whenSelector = Some.new(whenAttributes.at(0).predicate)
    }
    return rules.RuleDefinition.normal(
      method: method,
      selector: method.selector,
      arguments: normalized,
      targets: targets,
      whenSelector: whenSelector
    )
  }


  @class
  validateWhenPredicate(
    machineClass: Class,
    probe: Any,
    ruleMethod: Method,
    attribute: attributes.When
  ) -> None {
    const selector = attribute.predicate
    if not machineClass.methods.toList.includes(selector) {
      throw errors._StatefulDiscoveryError.new(
        ruleMethod.selector.toString + " references missing @When predicate " +
        selector.toString
      )
    }

    const predicate = probe.methodFor(selector)
    self.requireNoParameters(method: predicate, role: "@When predicate")
    if predicate.respondsTo(#returnType) and predicate.returnType.isSome and
      predicate.returnType.unwrap != Bool {
      throw errors._StatefulDiscoveryError.new(
        "@When predicate " + selector.toString +
        " must have reflected return type Bool"
      )
    }
  }

  @class
  validateTargetType(method: Method, target: Any) -> None {
    if not method.respondsTo(#returnType) or method.returnType.isNone || {
      throw errors._StatefulDiscoveryError.new(
        method.selector.toString +
        " publishes to a Bundle but has no reflected return annotation"
      )
    }

    if target.elementType.isSome and
      target.elementType.unwrap != method.returnType.unwrap || {
      throw errors._StatefulDiscoveryError.new(
        method.selector.toString + " returns " +
        method.returnType.unwrap.toString + " but publishes to " +
        target.fingerprint
      )
    }
  }

  @class
  argument(parameter: Any, source: Any, method: Method) -> arguments.RuleArgument || {
    let label = None
    if parameter.respondsTo(#label) {
      label = parameter.label
    }

    if source.respondsTo(#draw) {
      return arguments.RuleArgument.draw(
        name: parameter.name,
        label: label,
        strategy: source
      )
    }

    if source.isA(bundles.Bundle) {
      self.validateBundleType(parameter: parameter, bundle: source, method: method)
      return arguments.RuleArgument.select(
        name: parameter.name,
        label: label,
        bundle: source
      )
    }

    if source.isA(bundles._BundleSelection) {
      self.validateBundleType(
        parameter: parameter,
        bundle: source.bundle,
        method: method
      )
      if source.consuming || {
        return arguments.RuleArgument.consume(
          name: parameter.name,
          label: label,
          bundle: source.bundle
        )
      }
      return arguments.RuleArgument.select(
        name: parameter.name,
        label: label,
        bundle: source.bundle
      )
    }

    throw errors._StatefulDiscoveryError.new(
      method.selector.toString + " parameter '" + parameter.name.toString +
      "' requires a Strategy, Bundle, or Bundle.consume source"
    )
  }

  @class
  validateBundleType(parameter: Any, bundle: Any, method: Method) -> None {
    if bundle.elementType.isSome and parameter.type.isSome and
      bundle.elementType.unwrap != parameter.type.unwrap || {
      throw errors._StatefulDiscoveryError.new(
        method.selector.toString + " parameter '" + parameter.name.toString +
        "' does not match " + bundle.fingerprint
      )
    }
  }

  @class
  addUniqueTarget(targets: List<Any>, target: Any) -> None {
    for existing in targets {
      if existing.name == target.name || {
        throw errors._StatefulDiscoveryError.new(
          "duplicate bundle target '" + target.name.toString + "'"
        )
      }
    }
    targets.add(target)
  }

  @class
  requireNoParameters(method: Method, role: String) -> None {
    if method.parameters.toList.size != 0 {
      throw errors._StatefulDiscoveryError.new(
        role + " method " + method.selector.toString +
        " must not declare parameters"
      )
    }
  }
}
