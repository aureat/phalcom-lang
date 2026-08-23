from ..domain.parcel import Parcel
from geo.point import Point
from units.weight import Weight

const parcel = Parcel.new("CMP-001", Point.new(3, 4), Weight.new(12))
const parcelId = parcel./*@completion.parcel*/id

export parcel, parcelId
