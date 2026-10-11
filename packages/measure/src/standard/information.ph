//! Decimal and IEC information units plus data-rate units.

from ..exact.rational import Rational
from ..unit.linear import LinearUnit
from ..unit.prefix import Prefix
from .kinds import Information, DataRate
from .prefixes import kilo, mega, giga, tera
from .si.base import s

let bit = LinearUnit<Information>.base(Information(), "bit")
let B = bit.scaled(by: Rational.whole(8), symbol: "B")

let kB = kilo(B)
let MB = mega(B)
let GB = giga(B)
let TB = tera(B)

let kibi = Prefix.define("Ki", scale: Rational.whole(1024))
let mebi = Prefix.define("Mi", scale: Rational.whole(1024 ** 2))
let gibi = Prefix.define("Gi", scale: Rational.whole(1024 ** 3))
let tebi = Prefix.define("Ti", scale: Rational.whole(1024 ** 4))

let KiB = kibi(B)
let MiB = mebi(B)
let GiB = gibi(B)
let TiB = tebi(B)

let bps = LinearUnit<DataRate>.define(DataRate(), "bit/s", bit / s)
let kbps = kilo(bps)
let Mbps = mega(bps)
let Gbps = giga(bps)

export (
  bit, B, kB, MB, GB, TB,
  kibi, mebi, gibi, tebi, KiB, MiB, GiB, TiB,
  bps, kbps, Mbps, Gbps
)
