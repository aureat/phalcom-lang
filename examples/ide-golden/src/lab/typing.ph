from ..domain.parcel import Parcel
from geo.point import Point
from units.weight import Weight

let /*@hover.int*/inferred: Int = /*@mutation.binding_mismatch*/42
let /*@inlay.local.explicit*/explicit: Int = 42
let /*@inlay.local.inferred*/inferredAgain = 42
let point = /*@hover.point*/Point.new(1, y: 2)
let parcel = Parcel.new("LAB-001", destination: point, weight: Weight.new(3))

export inferred, explicit, inferredAgain, point, parcel
