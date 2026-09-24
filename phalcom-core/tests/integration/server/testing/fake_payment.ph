class FakePaymentGateway<R: Retry> {
    authorization: PaymentAuthorization
}

impl<R: Retry> PaymentGateway<TestFx, R>
    for FakePaymentGateway<R>
{
    type Error = PaymentError

    authorize( _ request: PaymentRequest)
     -> TestFx<Result<PaymentAuthorization, PaymentError>> {
        TestFx { value: Ok(authorization) }
    }
}
