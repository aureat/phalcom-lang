// REVISION V1
// Controller observes the shared result after Consumer's branch join.

import app.base.Packet
import app.consumer.Consumer

class Controller {
  @class
  run(_ value: Int, _ useA: Bool) {
    let mut packet = Consumer.choose(value, useA)
    let mut observed = packet.value()
    (observed, packet.class)
  }
}

export Controller
