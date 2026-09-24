// Expected failure: params.orderId is UserId rather than OrderId.
invalidNestedOrderId<F, Fx, Exec, Gateway>(
    context: {
        requestId: RequestId,
        user: User,
        params: { orderId: UserId },
        body: CheckoutRequest,
        ...
    },
    using services: CheckoutServices<F, Fx, Exec, Gateway>
)
where
    F: Type -> Type,
    Fx: Effect<F>,
    Exec: CommandExecutor<F>,
    Exec::Error == InfrastructureError,
    Gateway: PaymentGateway<F, Count(3)>,
    Gateway::Error == PaymentError
{
    checkoutHandler(context, using: services)
}
