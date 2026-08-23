from ..domain.parcel import Parcel
from ..domain.shipment import Shipment
from ..domain.status import PlannedStatus
from geo.point import Point
from geo.route import Route

class Planner {
  @class
  plan(_ parcel: Parcel, origin: Point) -> Shipment {
    const route = Route.new(origin, destination: parcel.destination)
    Shipment.new(parcel, route: route, status: PlannedStatus.new())
  }
}

export Planner
