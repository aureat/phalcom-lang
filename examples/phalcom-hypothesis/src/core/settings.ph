// Immutable execution settings.
//
// Getter and update selectors intentionally share names. Type annotations are
// reflective metadata; contracts guard the small set of numeric invariants.

import Phase from "core/phase"
import ExampleDatabase from "database/database"
import ChoiceProviderFactory from "choices/provider"
import providers from "choices/provider"

@data
@immutable
class Settings {
  const _maxExamples: Int
  const _maxDiscards: Int
  const _maxShrinks: Int
  const _maxChoices: Int
  const _seed: Option<Int>
  const _database: Option<ExampleDatabase>
  const _phases: List<Phase>
  const _statefulSteps: Int
  const _deadline: Option<Any>
  const _choiceProviderFactory: Option<ChoiceProviderFactory>

  @class
  standard -> Settings {
    return Settings.new(
      maxExamples: 100,
      maxDiscards: 1000,
      maxShrinks: 1000,
      maxChoices: 10000,
      seed: None,
      database: None,
      phases: const [
        Phase.Explicit,
        Phase.Reuse,
        Phase.Generate,
        Phase.Shrink
      ],
      statefulSteps: 50,
      deadline: None,
      choiceProviderFactory: None
    )
  }

  @requires(value > 0)
  maxExamples(value: Int) -> Settings {
    return self.with(maxExamples: value)
  }

  @requires(value > 0)
  maxDiscards(value: Int) -> Settings {
    return self.with(maxDiscards: value)
  }

  @requires(value >= 0)
  maxShrinks(value: Int) -> Settings {
    return self.with(maxShrinks: value)
  }

  @requires(value > 0)
  maxChoices(value: Int) -> Settings {
    return self.with(maxChoices: value)
  }

  seed(value: Int) -> Settings {
    return self.with(seed: Some.new(value))
  }

  withoutSeed -> Settings {
    return self.with(seed: None)
  }

  database(value: ExampleDatabase) -> Settings {
    return self.with(database: Some.new(value))
  }

  withoutDatabase -> Settings {
    return self.with(database: None)
  }

  @requires(value.size > 0)
  phases(value: List<Phase>) -> Settings {
    return self.with(phases: value)
  }

  @requires(value > 0)
  statefulSteps(value: Int) -> Settings {
    return self.with(statefulSteps: value)
  }

  deadline(value: Any) -> Settings {
    if value == None {
      return self.with(deadline: None)
    }

    return self.with(deadline: Some.new(value))
  }

  choiceProvider(factory: ChoiceProviderFactory) -> Settings {
    return self.with(choiceProviderFactory: Some.new(factory))
  }

  withoutChoiceProvider -> Settings {
    return self.with(choiceProviderFactory: None)
  }

  choiceProviderFactoryValue -> Option<ChoiceProviderFactory> {
    return _choiceProviderFactory
  }

  resolvedChoiceProviderFactory -> ChoiceProviderFactory {
    if _choiceProviderFactory.isSome {
      return _choiceProviderFactory.unwrap
    }
    return providers.SystemRandomProviderFactory.new(seed: self.resolvedSeed)
  }

  // Compatibility selectors retained while the Phase 01 engine is replaced.
  examples(value: Int) -> Settings => self.maxExamples(value)
  discardLimit(value: Int) -> Settings => self.maxDiscards(value)
  shrinkLimit(value: Int) -> Settings => self.maxShrinks(value)
  choiceLimit(value: Int) -> Settings => self.maxChoices(value)
  withDatabase(value: ExampleDatabase) -> Settings => self.database(value)

  seedValue -> Option<Int> => _seed
  databaseValue -> Option<ExampleDatabase> => _database
  statefulStepLimit -> Int => _statefulSteps

  reuseEnabled -> Bool => _phases.includes(Phase.Reuse)
  generationEnabled -> Bool => _phases.includes(Phase.Generate)
  shrinkingEnabled -> Bool => _phases.includes(Phase.Shrink)

  reuse(flag: Bool) -> Settings {
    return self._phase(Phase.Reuse, enabled: flag)
  }

  generation(flag: Bool) -> Settings {
    return self._phase(Phase.Generate, enabled: flag)
  }

  shrinking(flag: Bool) -> Settings {
    return self._phase(Phase.Shrink, enabled: flag)
  }

  resolvedSeed -> Int => _seed.unwrapOr(Random.system.nextInt)

  _phase(target: Phase, enabled: Bool) -> Settings {
    const next = List.new()
    for existing in _phases {
      if existing != target {
        next.add(existing)
      }
    }

    if enabled {
      next.add(target)
    }

    return self.phases(next)
  }
}
