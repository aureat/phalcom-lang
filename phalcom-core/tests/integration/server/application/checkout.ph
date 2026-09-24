data CheckoutView {
    orderId: OrderId
    state: String
    total: Money
}

checkoutHandler<F, Fx, Exec, Gateway>(
    context: {
        requestId: RequestId,
        user: User,
        params: {
            orderId: OrderId
        },
        body: CheckoutRequest,
        ...
    },
    using services: CheckoutServices<
        F,
        Fx,
        Exec,
        Gateway
    >
) -> F<
    Result<
        Response<Ok, CheckoutView>,
        CheckoutError
    >
>
where
    F: Type -> Type,
    Fx: Effect<F>,
    Exec: CommandExecutor<F>,
    Exec::Error == InfrastructureError,
    Gateway: PaymentGateway<F, Count(3)>,
    Gateway::Error == PaymentError
{
    services.effects.flatMap(
        services.executor.execute(
            .LoadDraft(context.params.orderId)
        )
    ) |loadedResult| {
        match loadedResult {
            .Err(error) => {
                services.effects.pure(
                    .Err(.Infrastructure(error))
                )
            }

            .Ok(.None) => {
                services.effects.pure(
                    .Err(.OrderNotFound)
                )
            }

            .Ok(.Some(draft)) => {
                expectType<Order<Draft>>(draft)
                services.trace.record(.DraftLoaded(draft.id))

                checkoutLoaded(
                    draft,
                    context.user,
                    context.body,
                    using: services
                )
            }
        }
    }
}

checkoutLoaded<F, Fx, Exec, Gateway>(
    draft: Order<Draft>,
    user: User,
    request: CheckoutRequest,
    using services: CheckoutServices<
        F,
        Fx,
        Exec,
        Gateway
    >
) -> F<
    Result<
        Response<Ok, CheckoutView>,
        CheckoutError
    >
>
where
    F: Type -> Type,
    Fx: Effect<F>,
    Exec: CommandExecutor<F>,
    Exec::Error == InfrastructureError,
    Gateway: PaymentGateway<F, Count(3)>,
    Gateway::Error == PaymentError
{
    if draft.owner != user.id {
        return services.effects.pure(
            .Err(.Forbidden)
        )
    }

    match submit(draft) {
        .Err(error) => {
            services.effects.pure(
                .Err(.Submit(error))
            )
        }

        .Ok(submitted) => {
            expectType<Order<Submitted>>(submitted)
            services.trace.record(.Submitted(submitted.id))

            authorizePayment(
                submitted,
                request,
                using: services
            )
        }
    }
}

authorizePayment<
  F: Type -> Type,
  Fx: Effect<F>,
  Exec: CommandExecutor<F>,
  Gateway: PaymentGateway<F, Count(3)>
>(
    order: Order<Submitted>,
    request: CheckoutRequest,
    using services: CheckoutServices<F, Fx, Exec, Gateway>
) -> F<
    Result<
        Response<Ok, CheckoutView>,
        CheckoutError
    >
>
where
    Exec::Error == InfrastructureError,
    Gateway::Error == PaymentError
{
    let paymentRequest = PaymentRequest {
        orderId: order.id,
        amount: order.total,
        token: request.paymentToken,
    }

    services.effects.flatMap(
        services.payment.authorize(paymentRequest)
    ) |authorizationResult| {
        match authorizationResult {
            Err(error) => {
                services.effects.pure(
                    Err(.Payment(error))
                )
            }

            Ok(authorization) => {
                services.trace.record(PaymentAuthorized(order.id))
                completePayment(order, authorization, using: services)
            }
        }
    }
}

completePayment<F, Fx, Exec, Gateway>(
    order: Order<Submitted>,
    authorization: PaymentAuthorization,
    using services: CheckoutServices<
        F,
        Fx,
        Exec,
        Gateway
    >
) -> F<
    Result<
        Response<Ok, CheckoutView>,
        CheckoutError
    >
>
where
    F: Type -> Type,
    Fx: Effect<F>,
    Exec: CommandExecutor<F>,
    Exec::Error == InfrastructureError,
    Gateway: PaymentGateway<F, Count(3)>,
    Gateway::Error == PaymentError
{
    match pay(order, authorization) {
        Err(error) => {
            services.effects.pure(Err(Payment(error)))
        }

        Ok(paid) => {
            expectType<Order<Paid>>(paid)
            services.trace.record(Paid(paid.id))

            completePayment(paid, using: services)
        }
    }
}

// Task 20 + Task 21 composition: persistence is followed by publication.
persistPaid<F, Fx, Exec, Gateway>(
    paid: Order<Paid>,
    using services: CheckoutServices<
        F,
        Fx,
        Exec,
        Gateway
    >
) -> F<
    Result<
        Response<Ok, CheckoutView>,
        CheckoutError
    >
>
where
    F: Type -> Type,
    Fx: Effect<F>,
    Exec: CommandExecutor<F>,
    Exec::Error == InfrastructureError,
    Gateway: PaymentGateway<F, Count(3)>,
    Gateway::Error == PaymentError
{
    services.effects.flatMap(
        services.executor.execute(
            .SavePaid(paid)
        )
    ) |savedResult| {
        match savedResult {
            Err(error) => {
                services.effects.pure(
                    Err(Infrastructure(error))
                )
            }

            Ok(saved) => {
                expectType<Order<Paid>>(saved)
                services.trace.record(Saved(saved.id))

                publishPaid(saved, using: services)
            }
        }
    }
}

publishPaid<
  F: Type -> Type,
  Fx: Effect<F>,
  Exec: CommandExecutor<F>,
  Gateway: PaymentGateway<F, Count(3)>,
>(
    paid: Order<Paid>,
    using services: CheckoutServices<F, Fx, Exec, Gateway>
) -> F<
    Result<
        Response<Ok, CheckoutView>,
        CheckoutError
    >
>
where
    Exec::Error == InfrastructureError,
    Gateway::Error == PaymentError
{
    let event = OrderPaid {
        orderId: paid.id,
        owner: paid.owner,
        total: paid.total,
    }

    services.effects.map(
        services.executor.execute(
            PublishPaid(event)
        )
    ) |publishResult| {
        match publishResult {
            Err(error) => {
                Err(Infrastructure(error))
            }

            Ok(()) => {
                services.trace.record(Published(paid.id))

                let response = checkoutResponse(paid)
                expectType<Response<Ok, CheckoutView>>(response)

                services.trace.record(Responded)
                Ok(response)
            }
        }
    }
}

checkoutResponse<
    F: Type -> Type,
    Fx: Effect<F>,
    Exec: CommandExecutor<F>,
    Gateway: PaymentGateway<F, Count(3)>,
>(
    paid: Order<Paid>
) -> Response<Ok, CheckoutView> {
    Response<Ok, CheckoutView> {
        body: CheckoutView {
            orderId: paid.id,
            state: "paid",
            total: paid.total,
        },
        headers: Headers {
            authorization: None,
        },
    }
}
