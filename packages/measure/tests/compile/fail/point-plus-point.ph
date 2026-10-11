from measure.standard.si.temperature import degC

let a = degC(10)
let b = degC(20)

// Expected compile failure: affine points do not define point addition.
let invalid = a + b
