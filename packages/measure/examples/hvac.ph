from measure.standard.si.base import min
from measure.standard.si.temperature import degC, deltaC, deltaCPerMin

let start = degC(18)
let target = degC(22)
let rise = target - start
let rate = deltaCPerMin(2)
let required = rise / rate

System.print(required.in(min))
