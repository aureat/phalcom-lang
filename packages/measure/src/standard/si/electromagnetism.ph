//! SI electromagnetic named units.

from ...unit.linear import LinearUnit
from ..kinds import (
  ElectricCharge, Voltage, Resistance, Conductance, Capacitance,
  MagneticFlux, MagneticFluxDensity, Inductance
)
from ..prefixes import milli, micro, kilo, mega
from .base import A, s
from .geometry import m2
from .mechanics import W

let C = LinearUnit<ElectricCharge>.define(ElectricCharge(), "C", A * s)
let V = LinearUnit<Voltage>.define(Voltage(), "V", W / A)
let ohm = LinearUnit<Resistance>.define(Resistance(), "Ω", V / A)
let S = LinearUnit<Conductance>.define(Conductance(), "S", A / V)
let F = LinearUnit<Capacitance>.define(Capacitance(), "F", C / V)
let Wb = LinearUnit<MagneticFlux>.define(MagneticFlux(), "Wb", V * s)
let tesla = LinearUnit<MagneticFluxDensity>.define(MagneticFluxDensity(), "T", Wb / m2)
let H = LinearUnit<Inductance>.define(Inductance(), "H", Wb / A)

let mA = milli(A)
let uA = micro(A)
let mV = milli(V)
let kV = kilo(V)
let kohm = kilo(ohm)
let Mohm = mega(ohm)
let uF = micro(F)

export C, V, ohm, S, F, Wb, tesla, H, mA, uA, mV, kV, kohm, Mohm, uF
