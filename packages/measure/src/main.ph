from .exact.rational import Rational
from .standard.si.base import h, km
from .standard.si.mechanics import kmph
from .standard.si.temperature import degC, degF, deltaC

let distance = kmph(90) * h(2)
System.print(distance.in(km))

let thermal = degC(20) - degF(50)
System.print(thermal.in(deltaC))

System.print(Rational.of(2, over: 4))
