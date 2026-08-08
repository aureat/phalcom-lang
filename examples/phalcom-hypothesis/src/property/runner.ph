// Reflective suite runner over the authoritative search engine with typed
// reporting events and named counterexample arguments.

import Settings from "core/settings"
import PropertyResult from "core/status"
import StrategyRegistry from "strategies/registry"
import Example from "choices/example"
import discovery from "property/discovery"
import engineSearch from "engine/engine"
import ReportEvent from "reporting/event"
import Reporter from "reporting/reporter"
import reportingReporter from "reporting/reporter"

@data
@immutable
class PropertyRun {
  const _id: discovery.PropertyId
  const _parameterNames: List<Symbol>
  const _explicitExamples: List<List<Any>>
  const _settings: Settings
  const _result: PropertyResult

  namedArguments -> Map<Symbol, Any> {
    const named = Map.new()
    const arguments = _result.args
    let index = 0
    while index < _parameterNames.size and index < arguments.size {
      named.at(_parameterNames.at(index), put: arguments.at(index))
      index++
    }
    return named
  }

  explicitFailure -> Bool {
    return _result.match(
      passed: |_| { false },
      falsified: |value| {
        if value.failure.example != Example.empty {
          return false
        }
        return _RunnerLists.includes(
          values: _explicitExamples,
          target: value.failure.arguments
        )
      },
      inconclusive: |_| { false },
      errored: |_| { false }
    )
  }

  passed -> Bool => _result.passed
  failed -> Bool => _result.failed
  name -> Any => _id
}

@data
@immutable
class PropertySuiteResult {
  const _runs: List<PropertyRun>

  results -> List<PropertyResult> {
    const values = List.new()
    for run in _runs {
      values.add(run.result)
    }
    return values
  }

  passedCount -> Int {
    let count = 0
    for run in _runs {
      if run.passed {
        count++
      }
    }
    return count
  }

  failedCount -> Int => _runs.size - self.passedCount
  passed -> Bool => self.failedCount == 0

  summaryLines -> List<String> {
    const lines = List.new()
    for run in _runs {
      run.result.match(
        passed: |_| { lines.add("PASS " + run.id.toString) },
        falsified: |_| { lines.add("FAIL " + run.id.toString) },
        inconclusive: |_| { lines.add("INCONCLUSIVE " + run.id.toString) },
        errored: |_| { lines.add("ERROR " + run.id.toString) }
      )
    }
    lines.add("")
    lines.add(
      self.passedCount.toString + " passed, " +
      self.failedCount.toString + " failed"
    )
    return lines
  }
}

class PropertyRunner {
  @class
  run(
    suiteClasses: List<Class>,
    with: Settings
  ) -> PropertySuiteResult {
    return self.run(
      suiteClasses,
      with: with,
      registry: StrategyRegistry.standard,
      reporter: reportingReporter.NullReporter.new()
    )
  }

  @class
  run(
    suiteClasses: List<Class>,
    with: Settings,
    reporter: Reporter
  ) -> PropertySuiteResult {
    return self.run(
      suiteClasses,
      with: with,
      registry: StrategyRegistry.standard,
      reporter: reporter
    )
  }

  @class
  run(
    suiteClasses: List<Class>,
    with: Settings,
    registry: StrategyRegistry
  ) -> PropertySuiteResult {
    return self.run(
      suiteClasses,
      with: with,
      registry: registry,
      reporter: reportingReporter.NullReporter.new()
    )
  }

  @class
  run(
    suiteClasses: List<Class>,
    with: Settings,
    registry: StrategyRegistry,
    reporter: Reporter
  ) -> PropertySuiteResult {
    const runs = List.new()
    const checkedReporter = reportingReporter._CheckedReporter.new(reporter)
    checkedReporter.handle(ReportEvent.suiteStarted(total: suiteClasses.size))

    for suiteClass in suiteClasses {
      const receiver = suiteClass.new()
      const definitions = discovery.PropertyDiscovery.discover(
        suiteClass: suiteClass,
        receiver: receiver,
        defaults: with,
        registry: registry
      )

      for definition in definitions {
        checkedReporter.handle(ReportEvent.propertyStarted(id: definition.id))
        const reuseExamples = _PropertyReuse.fetch(definition: definition)
        const result = engineSearch.SearchEngine.new().check(
          definition.toSpec(reuseExamples),
          reporter: checkedReporter
        )
        _PropertyReuse.record(
          definition: definition,
          reused: reuseExamples,
          result: result
        )
        const run = PropertyRun.new(
          id: definition.id,
          parameterNames: _RunnerLists.copy(definition.parameterNames),
          explicitExamples: _RunnerLists.nested(definition.explicitExamples),
          settings: definition.settings,
          result: result
        )
        runs.add(run)
        checkedReporter.handle(ReportEvent.propertyFinished(run: run))
      }
    }

    const suite = PropertySuiteResult.new(runs: runs)
    checkedReporter.handle(ReportEvent.suiteFinished(result: suite))
    return suite
  }
}

class _PropertyReuse {
  @class
  fetch(definition: discovery.PropertyDefinition) -> List<Example> {
    const settings = definition.settings
    if not settings.reuseEnabled or settings.databaseValue.isNone {
      return const []
    }
    return settings.databaseValue.unwrap.fetch(definition.databaseKey)
  }

  @class
  record(
    definition: discovery.PropertyDefinition,
    reused: List<Example>,
    result: PropertyResult
  ) -> None {
    const settings = definition.settings
    if settings.databaseValue.isNone {
      return
    }

    const database = settings.databaseValue.unwrap
    const key = definition.databaseKey
    result.match(
      passed: |_| { self.deleteStale(database: database, key: key, stale: reused) },
      falsified: |value| {
        const accepted = value.failure.example
        if accepted == Example.empty and _RunnerLists.includes(
          values: definition.explicitExamples,
          target: value.failure.arguments
        ) {
          return None
        }
        database.save(
          key,
          accepted,
          failureOrigin: Some.new(value.failure.origin)
        )
        const stale = List.new()
        for example in reused {
          if example.signature != accepted.signature {
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

class _RunnerLists {
  @class
  copy<T>(values: List<T>) -> List<T> {
    const copied = List.new()
    for value in values {
      copied.add(value)
    }
    return copied
  }

  @class
  nested(values: List<List<Any>>) -> List<List<Any>> {
    const copied = List.new()
    for value in values {
      copied.add(self.copy(value))
    }
    return copied
  }

  @class
  includes(values: List<List<Any>>, target: List<Any>) -> Bool {
    for value in values {
      if value == target {
        return true
      }
    }
    return false
  }
}
