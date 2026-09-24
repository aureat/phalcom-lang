// Task 24: one canonical deterministic fixture set for the executable golden.
class GoldenFixtureSet {
    ids: FixedRequestIdFactory
    authenticator: FakeAuthenticator
    decoder: CheckoutRequestDecoder
    repository: FakeOrderRepository
    events: RecordingEventPublisher
    gateway: FakePaymentGateway<Count(3)>
    trace: TraceRecorder
}

impl GoldenFixtureSet {
    class.canonical -> GoldenFixtureSet {
        GoldenFixtureSet {
            ids: FixedRequestIdFactory {
                value: RequestId("req-0001"),
            },
            authenticator: FakeAuthenticator {
                expected: AuthToken("Bearer golden-user"),
                user: FixtureFactory.user,
            },
            decoder: CheckoutRequestDecoder {
                value: FixtureFactory.checkoutRequest,
            },
            repository: FakeOrderRepository {
                draft: .Some(FixtureFactory.draftOrder),
                calls: [],
            },
            events: RecordingEventPublisher {
                events: [],
            },
            gateway: FakePaymentGateway<Count(3)> {
                authorization: PaymentAuthorization {
                    approved: true,
                },
            },
            trace: TraceRecorder {
                events: [],
            },
        }
    }
}
