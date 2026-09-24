// Expected failure: CheckoutServices requires PaymentGateway<F, Count(3)>.
invalidRetryServices(
    effects: TestEffect,
    executor: GoldenCommandExecutor,
    payment: FakePaymentGateway<Count(4)>,
    trace: TraceRecorder
) {
    CheckoutServices<
        TestFx,
        TestEffect,
        GoldenCommandExecutor,
        FakePaymentGateway<Count(4)>
    > {
        effects,
        executor,
        payment,
        trace,
    }
}
