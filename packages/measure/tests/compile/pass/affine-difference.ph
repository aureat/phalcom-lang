from measure.quantity.quantity import Quantity
from measure.exact.rational import Rational
from measure.standard.kinds import TemperatureDifference
from measure.standard.si.temperature import degC

let difference: Quantity<TemperatureDifference, Rational> = degC(20) - degC(10)
