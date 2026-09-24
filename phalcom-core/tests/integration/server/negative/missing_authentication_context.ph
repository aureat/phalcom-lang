// Expected failure: checkoutHandler requires `user: User`.
invalidMissingAuthentication<
  F: Type -> Type,
  Fx: Effect<F>,
  Exec: CommandExecutor<F>,
  Gateway: PaymentGateway<F, Count(3)>
>(
    context: {
        requestId: RequestId,
        params: { orderId: OrderId },
        body: CheckoutRequest,
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
