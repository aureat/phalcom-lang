// Phase 10: bare @Given resolves opt-in data and recursive sealed domain models.

import Assert from hypothesis
import Given from hypothesis
import PropertyRunner from hypothesis
import PropertySuite from hypothesis
import Settings from hypothesis
import arbitrary from hypothesis

@arbitrary
@data
@immutable
class Point {
  const _x: Int
  const _y: Int
}

@arbitrary
@data
@sealed
class Tree {
  @variant Leaf(value: Int)
  @variant Branch(left: Tree, right: Tree)
}

class DomainProperties is PropertySuite {
  @Given
  generated(point: Point, tree: Tree) {
    Assert.true(point.isA(Point))
    Assert.true(tree.isA(Tree))
  }
}

const run = PropertyRunner.run(
  const [DomainProperties],
  with: Settings.standard.examples(5)
)
Assert.equal(1, run.passedCount)
