from .parcel import Parcel
from .status import ShipmentStatus
from geo.route import Route

class Shipment {
  _parcel: Parcel
  _route: Route
  _status

  @constructor
  new(_ parcel: Parcel, route: Route, status: ShipmentStatus) {
    _parcel = parcel
    _route = route
    _status = status
  }

  parcel -> Parcel { _parcel }
  route -> Route { _route }
  status -> ShipmentStatus { _status }
  serviceName -> String { "standard" }
}

class ExpressShipment is Shipment {
  expressCode -> String { "EXPRESS" }
  serviceName -> String { "express" }
}

export Shipment, ExpressShipment
