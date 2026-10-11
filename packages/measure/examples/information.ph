from measure.standard.information import GiB, Mbps
from measure.standard.si.base import s

let payload = GiB(1)
let rate = Mbps(100)
let duration = payload / rate
System.print(duration.in(s))
