// area: collections
// spec: E.3 boundedness + F.2 outgoing generic *
// status: NEGATIVE

class Sink {
  take(*items) { return items.size }
}

Sink.new().take(*(0..))
