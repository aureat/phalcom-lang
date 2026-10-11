from measure.algebra.kind import QuantityKind, ProductKind
from measure.dimension.dimension import Dimension
from measure.exact.rational import Rational
from measure.standard.kinds import ElapsedTime
from measure.standard.si.base import s
from measure.unit.linear import LinearUnit

data GridLength()
data GridArea()
data GridVelocity()

let gridD = Dimension.base("test.grid.length")

impl QuantityKind for GridLength {
  name -> String { "GridLength" }
  dimension -> Dimension { gridD }
}

impl QuantityKind for GridArea {
  name -> String { "GridArea" }
  dimension -> Dimension { gridD ** 2 }
}

impl QuantityKind for GridVelocity {
  name -> String { "GridVelocity" }
  dimension -> Dimension { gridD / ElapsedTime().dimension }
}

impl ProductKind<GridLength> for GridLength {
  type Output = GridArea
  productKind(_ other: GridLength) -> GridArea { GridArea() }
}

impl ProductKind<ElapsedTime> for GridVelocity {
  type Output = GridLength
  productKind(_ other: ElapsedTime) -> GridLength { GridLength() }
}

let tile = LinearUnit<GridLength>.base(GridLength(), "tile")
let chunk = tile.scaled(by: Rational.whole(32), symbol: "chunk")
let tileps = LinearUnit<GridVelocity>.define(GridVelocity(), "tile/s", tile / s)

export GridLength, GridArea, GridVelocity, tile, chunk, tileps
