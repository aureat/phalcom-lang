data PublishedEvent {
    OrderPaidEvent(OrderPaid)
}

class RecordingEventPublisher {
    events: List<PublishedEvent>
}

impl EventPublisher<TestFx> for RecordingEventPublisher {
    type Error = InfrastructureError

    publish<E: DomainEvent>(
        event: E
    ) -> TestFx<
        Result<
            (),
            InfrastructureError
        >
    > {
        if event is OrderPaid {
            events.append(.OrderPaidEvent(event))
        }

        TestFx {
            value: .Ok(()),
        }
    }
}
