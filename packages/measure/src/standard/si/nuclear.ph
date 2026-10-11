//! SI radiological named units with nominal dose distinctions.

from ...unit.linear import LinearUnit
from ..kinds import Radioactivity, AbsorbedDose, EquivalentDose
from .base import kg, s
from .geometry import m2
from .mechanics import J

let Bq = LinearUnit<Radioactivity>.define(Radioactivity(), "Bq", s ** -1)
let Gy = LinearUnit<AbsorbedDose>.define(AbsorbedDose(), "Gy", J / kg)
let Sv = LinearUnit<EquivalentDose>.define(EquivalentDose(), "Sv", J / kg)

export Bq, Gy, Sv
