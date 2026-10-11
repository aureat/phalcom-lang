from measure.standard.si.temperature import degC, deltaC

let target = degC(21)
let tolerance = deltaC(1)
let lower = target.retreated(by: tolerance)
System.print(lower.toString)
