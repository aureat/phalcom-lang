data CheckoutServices<
    F: Type -> Type,
    Fx: Effect<F>,
    Exec: CommandExecutor<F>,
    Gateway: PaymentGateway<F, Count(3)>
>
where
    Exec::Error == InfrastructureError,
    Gateway::Error == PaymentError
{
    effects: Fx
    executor: Exec
    payment: Gateway
    trace: TraceRecorder
}
