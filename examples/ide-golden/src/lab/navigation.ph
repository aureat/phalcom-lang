from ..domain.parcel import /*@navigation.parcel.use*/Parcel
from geo.point import /*@navigation.point.cross_project*/Point
from units.distance import /*@navigation.distance.direct*/Distance

let parcelClass = Parcel
let pointClass = Point
let distanceClass = Distance
let /*@navigation.core.int*/builtinType = Int

export parcelClass, pointClass, distanceClass, builtinType
