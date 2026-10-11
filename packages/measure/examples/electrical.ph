from measure.standard.si.electromagnetism import A, V, ohm

let resistance = V(12) / A(3)
System.print(resistance.in(ohm))
