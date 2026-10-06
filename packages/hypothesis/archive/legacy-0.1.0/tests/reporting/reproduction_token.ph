import {
  Choice,
  Example,
  Gen,
  Phase,
  Reproduction,
  ReproductionToken,
  Settings
} from "hypothesis"

const example = Example.from(
  choices: const [
    Choice.integer(value: 10, min: 0, max: 100, shrinkTowards: 0, label: Some.new(#value))
  ],
  spans: const [],
  generationSize: 42
)
const settings = Settings.standard
  .maxExamples(500)
  .maxDiscards(2000)
  .maxShrinks(250)
  .maxChoices(5000)
  .seed(20260723)
  .phases(const [Phase.Reuse])
const token = ReproductionToken.create(
  propertyId: #dynamicProperty,
  example: example,
  settings: settings
)
Assert.equal(example, token.example)
Assert.equal(settings, token.settings)
Assert.true(token.text.startsWith("phalcom-hypothesis:v1:"))
const result = Reproduction.replay(
  token: token,
  strategies: const [Gen.int(min: 0, max: 100)],
  target: |value| { Assert.true(value < 10) }
)
Assert.true(result.failed)
