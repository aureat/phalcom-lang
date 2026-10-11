from measure.algebra.kind import QuantityKind, ProductKind
from measure.dimension.dimension import Dimension
from measure.exact.rational import Rational
from measure.unit.linear import LinearUnit

data GridLength()
data GridArea()

let gridD = Dimension.base("example.grid.length")

impl QuantityKind for GridLength {
  name -> String { "GridLength" }
  dimension -> Dimension { gridD }
}

impl QuantityKind for GridArea {
  name -> String { "GridArea" }
  dimension -> Dimension { gridD ** 2 }
}

impl ProductKind<GridLength> for GridLength {
  type Output = GridArea
  productKind(_ other: GridLength) -> GridArea { GridArea() }
}

let tile = LinearUnit<GridLength>.base(GridLength(), "tile")
let chunk = tile.scaled(by: Rational.whole(32), symbol: "chunk")

let area = chunk(3) * chunk(2)
System.print(area.toString)
