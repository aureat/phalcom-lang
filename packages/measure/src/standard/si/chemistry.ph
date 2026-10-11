//! SI amount-of-substance derived units.

from ...unit.linear import LinearUnit
from ..kinds import CatalyticActivity
from .base import mol, s

let kat = LinearUnit<CatalyticActivity>.define(CatalyticActivity(), "kat", mol / s)

export kat
