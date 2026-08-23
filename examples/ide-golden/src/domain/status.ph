class ShipmentStatus {
  name -> String { "unknown" }
}

class PlannedStatus is ShipmentStatus {
  name -> String { "planned" }
}

class DeliveredStatus is ShipmentStatus {
  name -> String { "delivered" }
}

export ShipmentStatus, PlannedStatus, DeliveredStatus
