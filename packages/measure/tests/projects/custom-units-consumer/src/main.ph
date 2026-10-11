from custom_units.grid import GridArea, GridLength, chunk, tile, tileps
from measure.exact.rational import Rational
from measure.quantity.quantity import Quantity
from measure.standard.si.base import s

let area: Quantity<GridArea, Rational> = chunk(3) * chunk(2)
let moved: Quantity<GridLength, Rational> = tileps(10) * s(2)

System.print(area.toString)
System.print(moved.in(tile))
