from measure.exact.rational import Rational
let mut value = Rational.of(1, over: 3)
let mut i = 0
while i < 10000 {
  value = value * Rational.of(i + 2, over: i + 1)
  i = i + 1
}
System.print(value)
