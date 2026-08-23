from ..domain.shipment import Shipment, ExpressShipment

class FlowLab {
  inspect(_ shipment: Shipment) -> String {
    if (shipment is ExpressShipment) {
      /*@hover.flow.refined*/shipment.expressCode
    } else {
      shipment.serviceName
    }
  }
}

export FlowLab
