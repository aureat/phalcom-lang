// BLOCKED: current Phalcom compilation rejects qualified Choice cases in the integer shrink-pass implementation.
from hypothesis.choices.choice import Choice
import hypothesis.engine.shrink_pass as ShrinkPasses
import hypothesis_tests.support.test as Testing

let shrinkPass = ShrinkPasses.IntegerShrinkPass.new()
let candidates = shrinkPass.candidates(Choice::Integer(37, 0, 100, 0))

Testing.Test.expect(candidates.size == 1, "integer shrink pass must propose the target")
match candidates[0] {
  Choice::Integer(value, min, max, target) => {
    Testing.Test.expect(value == 0 and target == 0, "integer shrink candidate must use its target")
    Testing.Test.expect(min == 0 and max == 100, "integer shrink candidate must preserve its domain")
  }
  _ => { throw Error.new("expected integer shrink candidate") }
}

let alreadyMinimal = shrinkPass.candidates(Choice::Integer(0, 0, 100, 0))
Testing.Test.expect(alreadyMinimal.size == 0, "integer shrink pass must omit an unchanged target")
Testing.Test.pass("engine.integer_shrink_pass")
