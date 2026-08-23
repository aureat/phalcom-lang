from ..domain.shipment import Shipment

class Report {
  @class
  render(_ shipment: Shipment) -> String {
    "parcel: " + shipment.parcel.id
  }
}

export Report
