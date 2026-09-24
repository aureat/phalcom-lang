// Expected failure: checkoutHandler requires `body: CheckoutRequest`.
invalidMissingBody<
  F: Type -> Type,
  Fx: Effect<F>,
  Exec: CommandExecutor<F>,
  Gateway: PaymentGateway<F, Count(3)>
>(
    context: {
        requestId: RequestId,
        user: User,
        params: { orderId: OrderId },
        ...
    },
    using services: CheckoutServices<F, Fx, Exec, Gateway>
)
where
    Exec::Error == InfrastructureError,
    Gateway::Error == PaymentError
{
    checkoutHandler(context, using: services)
}
