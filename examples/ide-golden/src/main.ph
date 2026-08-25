from .domain.parcel import Parcel
from .service.planner import Planner
from geo.point import Point
from units.weight import Weight
from .domain.shipment import Shipment

class Main {
  @class
  main {
    const origin: Point = Point.new(0, y: 0)
    const destination: Point = Point.new(3, y: 4)

    const parcel: Parcel = Parcel.new(
      "PKG-001", 
      destination: destination, 
      weight: Weight.new(12)
    )

    const shipment: Shipment = Planner.plan(parcel, origin: origin)

    const x = #{hello: String}

    System.print("Phalcom IDE Golden")
    System.print("parcel: " + shipment.parcel.id)
    System.print("origin: (" + origin.x.toString + ", " + origin.y.toString + ")")
    System.print("destination: (" + destination.x.toString + ", " + destination.y.toString + ")")
    System.print("distance: " + shipment.route.distance.units.toString)
    System.print("weight: " + shipment.parcel.weight.units.toString)
    System.print("service: " + shipment.serviceName)
    System.print("status: " + shipment.status.name)
  }
}

Main.main
