// Phase 02 — Settings is an immutable typed value with fluent getter/update
// selector pairs and exact standard defaults.

import Settings from hypothesis
import Phase from hypothesis
import Assert from hypothesis

const standard = Settings.standard

Assert.equal(100, standard.maxExamples)
Assert.equal(1000, standard.maxDiscards)
Assert.equal(1000, standard.maxShrinks)
Assert.equal(10000, standard.maxChoices)
Assert.equal(None, standard.seed)
Assert.equal(None, standard.database)
Assert.equal(50, standard.statefulSteps)
Assert.equal(None, standard.deadline)
Assert.equal(
  const [Phase.Explicit, Phase.Reuse, Phase.Generate, Phase.Shrink],
  standard.phases
)

const examplesChanged = standard.maxExamples(500)
Assert.equal(500, examplesChanged.maxExamples)
Assert.equal(100, standard.maxExamples)
Assert.isFalse(standard === examplesChanged)

const discardsChanged = standard.maxDiscards(2000)
Assert.equal(2000, discardsChanged.maxDiscards)
Assert.equal(1000, standard.maxDiscards)
Assert.isFalse(standard === discardsChanged)

const shrinksChanged = standard.maxShrinks(250)
Assert.equal(250, shrinksChanged.maxShrinks)
Assert.equal(1000, standard.maxShrinks)
Assert.isFalse(standard === shrinksChanged)

const choicesChanged = standard.maxChoices(5000)
Assert.equal(5000, choicesChanged.maxChoices)
Assert.equal(10000, standard.maxChoices)
Assert.isFalse(standard === choicesChanged)

const seedChanged = standard.seed(20260723)
Assert.equal(Some.new(20260723), seedChanged.seed)
Assert.equal(None, standard.seed)
Assert.isFalse(standard === seedChanged)

const database = Map.new()
const databaseChanged = standard.database(database)
Assert.equal(Some.new(database), databaseChanged.database)
Assert.equal(None, standard.database)
Assert.isFalse(standard === databaseChanged)

const phasesChanged = standard.phases(const [Phase.Generate])
Assert.equal(const [Phase.Generate], phasesChanged.phases)
Assert.equal(
  const [Phase.Explicit, Phase.Reuse, Phase.Generate, Phase.Shrink],
  standard.phases
)
Assert.isFalse(standard === phasesChanged)

const statefulChanged = standard.statefulSteps(80)
Assert.equal(80, statefulChanged.statefulSteps)
Assert.equal(50, standard.statefulSteps)
Assert.isFalse(standard === statefulChanged)

const deadlineChanged = standard.deadline(250)
Assert.equal(Some.new(250), deadlineChanged.deadline)
Assert.equal(None, standard.deadline)
Assert.isFalse(standard === deadlineChanged)

Assert.isTrue(|| { standard.maxExamples(0) }.attempt().isErr)
Assert.isTrue(|| { standard.maxDiscards(0) }.attempt().isErr)
Assert.isTrue(|| { standard.maxShrinks(-1) }.attempt().isErr)
Assert.isTrue(|| { standard.maxChoices(0) }.attempt().isErr)
Assert.isTrue(|| { standard.statefulSteps(0) }.attempt().isErr)
Assert.isTrue(|| { standard.phases(const []) }.attempt().isErr)

System.print("PASS core settings")
