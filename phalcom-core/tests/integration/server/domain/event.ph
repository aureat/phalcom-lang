trait DomainEvent

data OrderPaid {
    orderId: OrderId
    owner: UserId
    total: Money
}

impl DomainEvent for OrderPaid

data DiagnosticEvent {
    label: String
}

impl DomainEvent for DiagnosticEvent
