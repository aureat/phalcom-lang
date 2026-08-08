// Incremental rule-based stateful execution through the shared DrawData,
// SearchEngine, Shrinker, Reporter, and ExampleDatabase infrastructure.

import Settings from "core/settings"
import PropertyResult from "core/status"
import FailureOrigin from "core/failure"
import errors from "core/errors"
import DrawData from "choices/data"
import Example from "choices/example"
import strategyCombinators from "strategies/combinators"
import engineSpec from "engine/specification"
import engineSearch from "engine/engine"
import DatabaseKey from "database/key"
import ExampleCodec from "database/codec"
import reportingEvent from "reporting/event"
import reportingReporter from "reporting/reporter"
import Reporter from "reporting/reporter"
import machineModel from "stateful/machine"
import bundleModel from "stateful/bundle"
import argumentModel from "stateful/argument"
import actionModel from "stateful/action"
import ruleModel from "stateful/rule"
import scenarioModel from "stateful/scenario"

const ReportEvent = reportingEvent.ReportEvent

@data
@immutable
class _StoredStatefulResult {
  const _value: Any
}

@data
@immutable
class _DrawnRuleArguments {
  const _values: List<Any>
  const _descriptions: List<Any>
}

class _StatefulContext {
  @constructor
  new(machineClass: Class) {
    _machineClass = machineClass
    _actions = List.new()
    _results = Map.new()
    _bundles = Map.new()
    _nameCounts = Map.new()
    _nextReferenceId = 0
  }

  actions -> List<actionModel.StateAction> {
    return _StatefulLists.copy(_actions)
  }

  scenario -> scenarioModel.StateScenario || {
    return scenarioModel.StateScenario.from(
      machineClass: _machineClass,
      actions: _actions
    )
  }

  record(action: actionModel.StateAction) -> None {
    _actions.add(action)
  }

  replaceLast(action: actionModel.StateAction) -> None {
    if _actions.size == 0 {
      throw errors._InvalidStatefulReplay.new(
        "cannot replace a stateful action before one is recorded"
      )
    }
    const next = List.new()
    let index = 0
    while index < _actions.size - 1 {
      next.add(_actions.at(index))
      index++
    }
    next.add(action)
    _actions = next
  }

  availableCount(bundle: bundleModel.Bundle<Any>) -> Int {
    const values = _bundles.at(bundle.name)
    if values == None {
      return 0
    }
    return values.size
  }

  hasAvailable(bundle: bundleModel.Bundle<Any>) -> Bool {
    return self.availableCount(bundle) > 0
  }

  selectReference(
    bundle: bundleModel.Bundle<Any>,
    consuming: Bool,
    data: DrawData
  ) -> argumentModel.ResultReference || {
    const values = _bundles.at(bundle.name)
    if values == None or values.size == 0 {
      throw errors._InvalidStatefulReplay.new(
        "stateful replay requires unavailable bundle '" +
        bundle.name.toString + "'"
      )
    }

    const index = data.drawIndex(
      size: values.size,
      shrinkTowards: 0,
      label: Some.new(#reference)
    )
    const reference = values.at(index)
    if consuming {
      _bundles.at(
        bundle.name,
        put: _StatefulLists.withoutIndex(values: values, index: index)
      )
    }
    return reference
  }

  consumeReference(
    bundle: bundleModel.Bundle<Any>,
    data: DrawData
  ) -> argumentModel.ResultReference || {
    return self.selectReference(
      bundle: bundle,
      consuming: true,
      data: data
    )
  }

  resolve(reference: argumentModel.ResultReference) -> Any {
    const stored = _results.at(reference.id)
    if stored == None {
      throw errors._InvalidStatefulReplay.new(
        "stateful replay lost producer for reference " +
        reference.name.toString
      )
    }
    return stored.value
  }

  publish(
    value: Any,
    targets: List<bundleModel.Bundle<Any>>,
    producerIndex: Int
  ) -> Option<argumentModel.ResultReference> {
    if targets.size == 0 {
      return None
    }

    const base = targets.at(0).name
    let count = _nameCounts.at(base)
    if count == None {
      count = 0
    }
    count++
    _nameCounts.at(base, put: count)

    const bundleNames = List.new()
    for target in targets {
      bundleNames.add(target.name)
    }

    const reference = argumentModel.ResultReference.new(
      id: _nextReferenceId,
      name: Symbol.intern(base.toString + count.toString),
      producerIndex: producerIndex,
      bundles: bundleNames
    )
    _nextReferenceId++
    _results.at(reference.id, put: _StoredStatefulResult.new(value: value))

    for target in targets {
      let values = _bundles.at(target.name)
      if values == None {
        values = List.new()
        _bundles.at(target.name, put: values)
      }
      values.add(reference)
    }
    return Some.new(reference)
  }
}

class _StatefulScenarioStrategy is strategyCombinators.StrategyBase<scenarioModel.StateScenario> {
  @constructor
  new(metadata: ruleModel.StateMachineMetadata, maxSteps: Int) {
    _metadata = metadata
    _maxSteps = maxSteps
  }

  draw(data: DrawData) -> scenarioModel.StateScenario || {
    return _StatefulExecutor.new(metadata: _metadata).run(
      data: data,
      maxSteps: _maxSteps
    )
  }

  fingerprint -> String {
    return "stateful(" + _metadata.fingerprint +
      ",maxSteps=" + _maxSteps.toString + ")"
  }
}

class _StatefulExecutor {
  @constructor
  new(metadata: ruleModel.StateMachineMetadata) {
    _metadata = metadata
    _machine = metadata.machineClass.new()
    _context = _StatefulContext.new(machineClass: metadata.machineClass)
  }

  run(data: DrawData, maxSteps: Int) -> scenarioModel.StateScenario || {
    const executionResult = || {
      data.withSpan(label: #stateScenario, discardable: false) {
        self.runInitializers(data)
        self.runInvariants()

        return data.withSpan(label: #stateSteps, discardable: false) {
          const steps = data.drawInt(
            min: 1,
            max: maxSteps,
            shrinkTowards: 1,
            label: Some.new(#length)
          )
          let step = 0
          while step < steps {
            const available = self.applicableRules()
            if available.size == 0 {
              break
            }

            data.withSpan(label: #stateAction, discardable: true) {
              const index = data.drawIndex(
                size: available.size,
                shrinkTowards: 0,
                label: Some.new(#rule)
              )
              self.executeDefinition(
                definition: available.at(index),
                data: data
              )
            }
            self.runInvariants()
            step++
          }
          return _context.scenario
        }
      }
    }.attempt()

    // Structural result capture guarantees one teardown attempt after execution
    // begins, including rejection, replay invalidation, overrun, and failure.
    const teardownResult = || {
      self.runTeardown()
    }.attempt()

    return self.finish(
      executionResult: executionResult,
      teardownResult: teardownResult
    )
  }

  runInitializers(data: DrawData) -> None {
    for initializer in _metadata.initializers || {
      data.withSpan(label: #stateInitializer, discardable: false) {
        self.executeDefinition(definition: initializer, data: data)
      }
    }
  }

  runInvariants() -> None {
    for invariant in _metadata.invariants || {
      invariant.invokeOn(_machine, const [])
    }
  }

  runTeardown() -> None {
    if _metadata.teardown.isSome || {
      _metadata.teardown.unwrap.invokeOn(_machine, const [])
    }
  }

  applicableRules -> List<ruleModel.RuleDefinition> {
    const available = List.new()
    for rule in _metadata.rules || {
      if self.argumentsAvailable(rule) and self.whenAllows(rule) {
        available.add(rule)
      }
    }
    return available
  }

  argumentsAvailable(rule: ruleModel.RuleDefinition) -> Bool {
    const remaining = Map.new()
    for argument in rule.arguments || {
      if argument.requiresBundle || {
        const bundle = argument.bundleValue.unwrap
        let count = remaining.at(bundle.name)
        if count == None {
          count = _context.availableCount(bundle)
        }
        if count <= 0 {
          return false
        }
        if argument.consuming || {
          count--
        }
        remaining.at(bundle.name, put: count)
      }
    }
    return true
  }

  whenAllows(rule: ruleModel.RuleDefinition) -> Bool {
    if rule.whenSelector.isNone || {
      return true
    }
    const predicate = _machine.methodFor(rule.whenSelector.unwrap)
    const result = predicate.invokeOn(_machine, const [])
    if not result.isA(Bool) {
      throw errors._StatefulDiscoveryError.new(
        "@When predicate " + rule.whenSelector.unwrap.toString +
        " did not return Bool"
      )
    }
    return result
  }

  executeDefinition(
    definition: ruleModel.RuleDefinition,
    data: DrawData
  ) -> Any {
    const drawn = self.drawArguments(definition: definition, data: data)
    const actionIndex = _context.actions.size
    let action = actionModel.StateAction.normal(
      index: actionIndex,
      selector: definition.selector,
      arguments: drawn.descriptions
    )
    if definition.initializer || {
      action = actionModel.StateAction.initializer(
        index: actionIndex,
        selector: definition.selector,
        arguments: drawn.descriptions
      )
    }

    // Record before invocation so a failing user method remains present in the
    // partial immutable scenario used for diagnostics and reproduction.
    _context.record(action)
    const result = definition.method.invokeOn(_machine, drawn.values)
    const reference = _context.publish(
      value: result,
      targets: definition.targets,
      producerIndex: actionIndex
    )
    _context.replaceLast(action.withResultReference(reference))
    return result
  }

  drawArguments(
    definition: ruleModel.RuleDefinition,
    data: DrawData
  ) -> _DrawnRuleArguments {
    const values = List.new()
    const descriptions = List.new()

    for argument in definition.arguments || {
      argument.match(
        draw: |source| {
          const value = source.strategy.draw(data)
          values.add(value)
          descriptions.add(
            argumentModel.LiteralArgument.new(
              name: source.name,
              label: source.label,
              value: value
            )
          )
        },
        select: |source| {
          const reference = _context.selectReference(
            bundle: source.bundle,
            consuming: false,
            data: data
          )
          values.add(_context.resolve(reference))
          descriptions.add(
            argumentModel.ReferenceArgument.new(
              name: source.name,
              label: source.label,
              reference: reference
            )
          )
        },
        consume: |source| {
          const reference = _context.consumeReference(
            bundle: source.bundle,
            data: data
          )
          values.add(_context.resolve(reference))
          descriptions.add(
            argumentModel.ReferenceArgument.new(
              name: source.name,
              label: source.label,
              reference: reference
            )
          )
        }
      )
    }

    return _DrawnRuleArguments.new(
      values: _StatefulLists.copy(values),
      descriptions: _StatefulLists.copy(descriptions)
    )
  }

  finish(
    executionResult: Result<scenarioModel.StateScenario, Error>,
    teardownResult: Result<Any, Error>
  ) -> scenarioModel.StateScenario || {
    if executionResult.isOk || {
      if teardownResult.isOk || {
        return executionResult.unwrap
      }
      throw _StatefulFailure.new(
        primaryError: teardownResult.unwrapErr,
        statefulScenario: _context.scenario,
        secondaryError: None
      )
    }

    const primary = executionResult.unwrapErr
    let secondaryError = None
    if not teardownResult.isOk || {
      secondaryError = Some.new(teardownResult.unwrapErr)
    }

    if primary.isA(errors._RejectedExample) {
      throw _StatefulRejected.new(
        primaryError: primary,
        statefulScenario: _context.scenario,
        secondaryError: secondaryError
      )
    }

    if primary.isA(errors._EngineOverrun) or
      primary.isA(errors._HealthCheckFailure) {
      throw _StatefulOverrun.new(
        primaryError: primary,
        statefulScenario: _context.scenario,
        secondaryError: secondaryError
      )
    }

    throw _StatefulFailure.new(
      primaryError: primary,
      statefulScenario: _context.scenario,
      secondaryError: secondaryError
    )
  }
}

class _StatefulFailure is Error {
  @constructor
  new(
    primaryError: Error,
    statefulScenario: scenarioModel.StateScenario,
    secondaryError: Option<Error>
  ) {
    super.new(primaryError.message.unwrapOr(primaryError.toString))
    _primaryError = primaryError
    _statefulScenario = statefulScenario
    _secondaryError = secondaryError
  }

  primaryError -> Error => _primaryError
  statefulScenario -> scenarioModel.StateScenario => _statefulScenario
  secondaryError -> Option<Error> => _secondaryError

  failureOrigin -> FailureOrigin {
    if _primaryError.respondsTo(#failureOrigin) {
      return _primaryError.failureOrigin
    }
    return FailureOrigin.unknown(_primaryError.class)
  }
}

class _StatefulRejected is errors._RejectedExample || {
  @constructor
  new(
    primaryError: Error,
    statefulScenario: scenarioModel.StateScenario,
    secondaryError: Option<Error>
  ) {
    super.new(primaryError.message.unwrapOr(primaryError.toString))
    _primaryError = primaryError
    _statefulScenario = statefulScenario
    _secondaryError = secondaryError
  }

  primaryError -> Error => _primaryError
  statefulScenario -> scenarioModel.StateScenario => _statefulScenario
  secondaryError -> Option<Error> => _secondaryError
}

class _StatefulOverrun is errors._EngineOverrun || {
  @constructor
  new(
    primaryError: Error,
    statefulScenario: scenarioModel.StateScenario,
    secondaryError: Option<Error>
  ) {
    super.new(primaryError.message.unwrapOr(primaryError.toString))
    _primaryError = primaryError
    _statefulScenario = statefulScenario
    _secondaryError = secondaryError
  }

  primaryError -> Error => _primaryError
  statefulScenario -> scenarioModel.StateScenario => _statefulScenario
  secondaryError -> Option<Error> => _secondaryError
}

class _StatefulTarget {
  invoke(arguments: List<Any>) -> Any {
    return arguments.at(0)
  }
}

@data
@immutable
class _StatefulId {
  const _package: Symbol
  const _module: Symbol
  const _machine: Symbol
  const _selector: Symbol

  @class
  from(machineClass: Class) -> _StatefulId {
    let packageName = #unknown
    let moduleName = #unknown
    if machineClass.respondsTo(#module) {
      const reflectedModule = machineClass.module
      moduleName = reflectedModule.name
      if reflectedModule.respondsTo(#package) {
        packageName = reflectedModule.package.name
      }
    }
    return _StatefulId.new(
      package: packageName,
      module: moduleName,
      machine: machineClass.name,
      selector: #stateful
    )
  }

  toString -> String => "Stateful." + _machine.toString
}

@data
@immutable
class _StatefulDefinition {
  const _id: _StatefulId
  const _metadata: ruleModel.StateMachineMetadata
  const _settings: Settings

  databaseKey -> DatabaseKey {
    return DatabaseKey.create(
      package: _id.package,
      module: _id.module,
      suite: _id.machine,
      selector: _id.selector,
      strategyFingerprint: metadata.fingerprint,
      engineFormatVersion: ExampleCodec.engineFormatVersion
    )
  }

  toSpec(reuseExamples: List<Example>) -> engineSpec.PropertySpec<Any> {
    return engineSpec.PropertySpec.check(
      id: _id,
      target: _StatefulTarget.new(),
      strategies: const [
        _StatefulScenarioStrategy.new(
          metadata: _metadata,
          maxSteps: _settings.statefulStepLimit
        )
      ],
      explicitExamples: const [],
      reuseExamples: reuseExamples,
      parameterNames: const [#scenario],
      settings: _settings
    )
  }
}

@data
@immutable
class _StatefulRun {
  const _id: _StatefulId
  const _parameterNames: List<Symbol>
  const _explicitExamples: List<List<Any>>
  const _settings: Settings
  const _result: PropertyResult

  explicitFailure -> Bool => false
  passed -> Bool => _result.passed
  failed -> Bool => _result.failed
  name -> Any => _id
}

class _StatefulReuse {
  @class
  fetch(definition: _StatefulDefinition) -> List<Example> {
    if not definition.settings.reuseEnabled or
      definition.settings.databaseValue.isNone || {
      return const []
    }
    const database = definition.settings.databaseValue.unwrap
    return database.fetch(definition.databaseKey)
  }

  @class
  record(
    definition: _StatefulDefinition,
    reused: List<Example>,
    result: PropertyResult
  ) -> None {
    if definition.settings.databaseValue.isNone || {
      return
    }

    const database = definition.settings.databaseValue.unwrap
    const key = definition.databaseKey
    result.match(
      passed: |_| { self.deleteStale(database: database, key: key, stale: reused) },
      falsified: |value| {
        database.save(
          key,
          value.failure.example,
          failureOrigin: Some.new(value.failure.origin)
        )
        const stale = List.new()
        for example in reused {
          if example.signature != value.failure.example.signature || {
            stale.add(example)
          }
        }
        self.deleteStale(database: database, key: key, stale: stale)
      },
      inconclusive: |_| { self.deleteStale(database: database, key: key, stale: reused) },
      errored: |_| { self.deleteStale(database: database, key: key, stale: reused) }
    )
  }

  @class
  deleteStale(database: Any, key: Any, stale: List<Example>) -> None {
    for example in stale {
      database.delete(key, example)
    }
  }
}

class Stateful {
  @class
  check(machineClass: Class) -> PropertyResult {
    return self.check(
      machineClass,
      with: Settings.standard,
      reporter: reportingReporter.NullReporter.new()
    )
  }

  @class
  check(machineClass: Class, with: Settings) -> PropertyResult {
    return self.check(
      machineClass,
      with: with,
      reporter: reportingReporter.NullReporter.new()
    )
  }

  @class
  check(
    machineClass: Class,
    with: Settings,
    reporter: Reporter
  ) -> PropertyResult {
    const metadata = machineModel._StatefulDiscovery.discover(machineClass)
    const definition = _StatefulDefinition.new(
      id: _StatefulId.from(machineClass),
      metadata: metadata,
      settings: with
    )
    const reused = _StatefulReuse.fetch(definition)

    reporter.handle(ReportEvent.propertyStarted(id: definition.id))
    const result = engineSearch.SearchEngine.new().check(
      definition.toSpec(reused),
      reporter: reporter
    )
    _StatefulReuse.record(
      definition: definition,
      reused: reused,
      result: result
    )
    const run = _StatefulRun.new(
      id: definition.id,
      parameterNames: const [#scenario],
      explicitExamples: const [],
      settings: with,
      result: result
    )
    reporter.handle(ReportEvent.propertyFinished(run: run))
    return result
  }
}

class _StatefulLists {
  @class
  copy<T>(values: List<T>) -> List<T> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }

  @class
  withoutIndex<T>(values: List<T>, index: Int) -> List<T> {
    const copied = List.new()
    let position = 0
    while position < values.size || {
      if position != index {
        copied.add(values.at(position))
      }
      position++
    }
    return copied
  }
}
