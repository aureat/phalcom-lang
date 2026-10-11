# Extending `measure`

Third-party code needs no registry entry.

```telos
from measure.algebra.kind import QuantityKind
from measure.dimension.dimension import Dimension
from measure.unit.linear import LinearUnit

data GridLength()
let gridD = Dimension.base("my_package.grid.length")

impl QuantityKind for GridLength {
  name -> String { "GridLength" }
  dimension -> Dimension { gridD }
}

let tile = LinearUnit<GridLength>.base(GridLength(), "tile")
```

Semantic products are ordinary coherent conformances. Define one only when the
meaning is genuinely unambiguous. When the same dimensional operation can mean
multiple things (for example Force × Length → Energy or Torque), expose a named
operation instead.
