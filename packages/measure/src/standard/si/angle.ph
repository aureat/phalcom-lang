//! Dimensionless-but-nominal angle and solid-angle units.

import universe.math as math
from ...unit.approximate import ApproximateUnit
from ...unit.linear import LinearUnit
from ..kinds import Angle, SolidAngle

let rad = LinearUnit<Angle>.base(Angle(), "rad")
let sr = LinearUnit<SolidAngle>.base(SolidAngle(), "sr")

/// Degree has irrational scale π/180 relative to the radian reference scale.
let deg = ApproximateUnit<Angle>.define(
  Angle(),
  "°",
  scale: math.pi / 180.0
)

export rad, sr, deg
