data CheckoutServices<F, Fx, Exec, Gateway>
where
    F: Type -> Type,
    Fx: Effect<F>,
    Exec: CommandExecutor<F>,
    Exec::Error == InfrastructureError,
    Gateway: PaymentGateway<F, Count(3)>,
    Gateway::Error == PaymentError
{
    effects: Fx
    executor: Exec
    payment: Gateway
    trace: TraceRecorder
}
