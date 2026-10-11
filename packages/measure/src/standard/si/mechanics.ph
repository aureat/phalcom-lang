//! Mechanical units plus named operations for semantically ambiguous products.

from ...algebra.scalar import QuantityScalar
from ...quantity.quantity import Quantity
from ...unit.linear import LinearUnit
from ..kinds import Length, Velocity, Acceleration, Force, Pressure, Energy, Torque, Power, Frequency
from ..prefixes import kilo, mega
from .base import m, km, kg, s, h
from .geometry import m2

let mps = LinearUnit<Velocity>.define(Velocity(), "m/s", m / s)
let kmph = LinearUnit<Velocity>.define(Velocity(), "km/h", km / h)
let mps2 = LinearUnit<Acceleration>.define(Acceleration(), "m/s²", m / (s ** 2))

let N = LinearUnit<Force>.define(Force(), "N", kg * m / (s ** 2))
let kN = kilo(N)
let Pa = LinearUnit<Pressure>.define(Pressure(), "Pa", N / m2)
let kPa = kilo(Pa)
let MPa = mega(Pa)
let J = LinearUnit<Energy>.define(Energy(), "J", N * m)
let kJ = kilo(J)
let MJ = mega(J)
let Nm = LinearUnit<Torque>.define(Torque(), "N·m", N * m)
let W = LinearUnit<Power>.define(Power(), "W", J / s)
let kW = kilo(W)
let MW = mega(W)
let Hz = LinearUnit<Frequency>.define(Frequency(), "Hz", s ** -1)
let kHz = kilo(Hz)
let MHz = mega(Hz)


/// Named mechanics operations for dimensionally valid but semantically
/// ambiguous products.
class Mechanics {
  /// Mechanical work performed by a force over a displacement magnitude.
  @class
  work<N>(
    _ force: Quantity<Force, N>,
    over distance: Quantity<Length, N>
  ) -> Quantity<Energy, N>
    where N |= QuantityScalar
  {
    Quantity.reference(
      Energy(),
      force.magnitude.multiplyScalar(distance.magnitude)
    )
  }

  /// Torque magnitude produced by a force acting at a lever arm.
  @class
  torque<N>(
    _ force: Quantity<Force, N>,
    arm: Quantity<Length, N>
  ) -> Quantity<Torque, N>
    where N |= QuantityScalar
  {
    Quantity.reference(
      Torque(),
      force.magnitude.multiplyScalar(arm.magnitude)
    )
  }
}

export mps, kmph, mps2, N, kN, Pa, kPa, MPa, J, kJ, MJ, Nm, W, kW, MW, Hz, kHz, MHz, Mechanics
