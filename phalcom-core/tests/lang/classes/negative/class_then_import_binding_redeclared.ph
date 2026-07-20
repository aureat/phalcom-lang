// U-CLASSCLOSE §8 table, reverse ordering: `class Point` then `import … as
// Point` is reported as binding.redeclared, from the import side — no
// implementation work for this direction, it falls out of the class now
// registering in `global_bindings`. Pins the table's other half.
class Point {}

import "../lib/point_holder" as Point
