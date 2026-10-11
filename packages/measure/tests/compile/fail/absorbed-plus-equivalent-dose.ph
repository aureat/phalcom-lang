from measure.standard.si.nuclear import Gy, Sv

// Same structural dimension, distinct semantic kinds.
let invalid = Gy(1) + Sv(1)
