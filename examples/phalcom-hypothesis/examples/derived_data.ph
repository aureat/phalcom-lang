// Bare @Given generation for an immutable data model.

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

class PointProperties is PropertySuite {
  @Given
  translationRoundTrips(point: Point, dx: Int, dy: Int) {
    const moved = Point.new(x: point.x + dx, y: point.y + dy)
    const restored = Point.new(x: moved.x - dx, y: moved.y - dy)
    Assert.equal(point, restored)
  }
}

PropertyRunner.run(
  const [PointProperties],
  with: Settings.standard.examples(100)
)
