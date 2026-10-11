//! Complete SI decimal-prefix constants.

from ..exact.rational import (
  rationalTenth, rationalHundredth, rationalThousandth,
  rationalMillionth, rationalBillionth, rationalTrillionth,
  rationalQuadrillionth, rationalQuintillionth, rationalSextillionth,
  rationalSeptillionth, rationalOctillionth, rationalNonillionth,
  rationalTen, rationalHundred, rationalThousand, rationalMillion,
  rationalBillion, rationalTrillion, rationalQuadrillion, rationalQuintillion,
  rationalSextillion, rationalSeptillion, rationalOctillion, rationalNonillion
)
from ..unit.prefix import Prefix

/// Complete SI decimal-prefix vocabulary.
///
/// These are true static transparent-data values. Their construction requires
/// no arbitrary compile-time method execution; ordinary Prefix behavior is
/// used after materialization at runtime.
const quecto = Prefix("q", rationalNonillionth)
const ronto = Prefix("r", rationalOctillionth)
const yocto = Prefix("y", rationalSeptillionth)
const zepto = Prefix("z", rationalSextillionth)
const atto = Prefix("a", rationalQuintillionth)
const femto = Prefix("f", rationalQuadrillionth)
const pico = Prefix("p", rationalTrillionth)
const nano = Prefix("n", rationalBillionth)
const micro = Prefix("µ", rationalMillionth)
const milli = Prefix("m", rationalThousandth)
const centi = Prefix("c", rationalHundredth)
const deci = Prefix("d", rationalTenth)

const deca = Prefix("da", rationalTen)
const hecto = Prefix("h", rationalHundred)
const kilo = Prefix("k", rationalThousand)
const mega = Prefix("M", rationalMillion)
const giga = Prefix("G", rationalBillion)
const tera = Prefix("T", rationalTrillion)
const peta = Prefix("P", rationalQuadrillion)
const exa = Prefix("E", rationalQuintillion)
const zetta = Prefix("Z", rationalSextillion)
const yotta = Prefix("Y", rationalSeptillion)
const ronna = Prefix("R", rationalOctillion)
const quetta = Prefix("Q", rationalNonillion)

export (
  quecto, ronto, yocto, zepto, atto, femto, pico, nano, micro, milli, centi, deci,
  deca, hecto, kilo, mega, giga, tera, peta, exa, zetta, yotta, ronna, quetta
)
