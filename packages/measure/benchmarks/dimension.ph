from measure.dimension.dimension import Dimension
let a = Dimension.base("a")
let b = Dimension.base("b")
let c = Dimension.base("c")
let d = Dimension.base("d")
let mut value = a
let mut i = 0
while i < 10000 {
  value = ((value * b) / b) * c / c * d / d
  i = i + 1
}
System.print(value)
