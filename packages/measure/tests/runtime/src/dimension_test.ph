from measure.dimension.dimension import Dimension
from .support import Test

let l = Dimension.base("L")
let t = Dimension.base("T")
let m = Dimension.base("M")

Test.expect(
  (l / t * t).equivalent(l),
  "(L/T)*T cancels time"
)

Test.expect(
  (m * l / (t ** 2)).exponent(of: "T") == -2,
  "force-like dimension carries T^-2"
)

Test.expect(
  ((l ** 2) / l).equivalent(l),
  "L^2/L reduces to L"
)

Test.expect(
  (l / l).isDimensionless,
  "L/L is dimensionless"
)

Test.expect(
  (l * m).equivalent(m * l),
  "dimension multiplication is commutative"
)

Test.expect(
  ((l / t) ** -1).equivalent(t / l),
  "negative dimension power reciprocates exponents"
)

Test.pass("dimension")
