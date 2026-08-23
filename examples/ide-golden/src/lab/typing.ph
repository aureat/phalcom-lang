from ..domain.parcel import Parcel
from geo.point import Point
from units.weight import Weight

const /*@hover.int*/inferred: Int = /*@mutation.binding_mismatch*/42
const /*@inlay.local.explicit*/explicit: Int = 42
const /*@inlay.local.inferred*/inferredAgain = 42
const point = /*@hover.point*/Point.new(1, 2)
const parcel = Parcel.new("LAB-001", point, Weight.new(3))

export inferred, explicit, inferredAgain, point, parcel
