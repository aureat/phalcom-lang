from measure.standard.si.base import km
from measure.standard.si.mechanics import J

// Expected compile failure: Energy unit cannot express Length quantity.
let invalid = km(1).in(J)
