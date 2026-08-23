from ..domain.parcel import /*@navigation.parcel.use*/Parcel
from geo.point import /*@navigation.point.cross_project*/Point
from units.distance import /*@navigation.distance.direct*/Distance

const parcelClass = Parcel
const pointClass = Point
const distanceClass = Distance
const /*@navigation.core.int*/builtinType = Int

export parcelClass, pointClass, distanceClass, builtinType
