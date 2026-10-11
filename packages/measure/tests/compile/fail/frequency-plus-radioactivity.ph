from measure.standard.si.mechanics import Hz
from measure.standard.si.nuclear import Bq

// Same T^-1 dimension, distinct semantic kinds.
let invalid = Hz(1) + Bq(1)
