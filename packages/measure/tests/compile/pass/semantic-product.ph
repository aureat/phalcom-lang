from measure.quantity.quantity import Quantity
from measure.exact.rational import Rational
from measure.standard.kinds import Length
from measure.standard.si.base import h
from measure.standard.si.mechanics import mps

let distance: Quantity<Length, Rational> =
  mps(100) * h(4)
