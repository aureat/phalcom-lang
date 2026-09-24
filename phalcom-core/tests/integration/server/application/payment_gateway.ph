trait PaymentGateway<F, R>
where
    F: Type -> Type,
    R: Retry
{
    type Error: Type

    authorize(
        request: PaymentRequest
    ) -> F<
        Result<
            PaymentAuthorization,
            Self::Error
        >
    >
}
