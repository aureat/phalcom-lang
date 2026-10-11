from measure.standard.si.base import h, km
from measure.standard.si.mechanics import kmph

let distance = kmph(90) * h(2)
System.print(distance.in(km))
