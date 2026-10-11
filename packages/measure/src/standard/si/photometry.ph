//! SI photometric named units.

from ...unit.linear import LinearUnit
from ..kinds import LuminousFlux, Illuminance
from .angle import sr
from .base import cd
from .geometry import m2

let lm = LinearUnit<LuminousFlux>.define(LuminousFlux(), "lm", cd * sr)
let lx = LinearUnit<Illuminance>.define(Illuminance(), "lx", lm / m2)

export lm, lx
