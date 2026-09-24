// Task 25: typed runtime trace. The trace is application-observable evidence,
// not a dynamic mock call log.
data TraceEvent {
    RequestIdentified(RequestId)
    RouteDecoded(OrderId)
    Authenticated(UserId)
    BodyDecoded
    DraftLoaded(OrderId)
    Submitted(OrderId)
    PaymentAuthorized(OrderId)
    Paid(OrderId)
    Saved(OrderId)
    Published(OrderId)
    Responded
}

class TraceRecorder {
    events: List<TraceEvent>
}

impl TraceRecorder {
    record(
        event: TraceEvent
    ) -> () {
        events.append(event)
    }
}
