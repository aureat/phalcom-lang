prepareCheckoutContext<
  A: Authenticator,
  D: Decoder<CheckoutRequest>
>(
    request: RawRequest,
    using ids: RequestIdFactory,
    authenticator: A,
    decoder: D
) -> Result<
    {
        requestId: RequestId,
        user: User,
        params: {
            orderId: OrderId
        },
        body: CheckoutRequest,
        method: Method,
        path: String,
        headers: Headers,
        rawBody: Bytes
    },
    CheckoutError
>
where
    A::Error == Unauthorized,
    D::Error == DecodeError
{
    let initial = requestContext(request)
    let identified = attachRequestId(initial, using: ids)

    let routed = match decodeCheckoutRoute(identified) {
        Err(error) => {
            return Err(Route(error))
        }
        Ok(value) => value
    }

    let authenticated = match authenticate(
        routed,
        using: authenticator
    ) {
        Err(_) => {
            return Err(Unauthorized)
        }
        Ok(value) => value
    }

    match decodeJson<CheckoutRequest>(
        authenticated,
        using: decoder
    ) {
        Err(error) => Err(Decode(error))
        Ok(context) => Ok(context)
    }
}

processRequest<
  A: Authenticator,
  D: Decoder<CheckoutRequest>,
  F: Type -> Type,
  Fx: Effect<F>,
  Exec: CommandExecutor<F>,
  Gateway: PaymentGateway<F, Count(3)>
>(
    request: RawRequest,
    using ids: RequestIdFactory,
    authenticator: A,
    decoder: D,
    services: CheckoutServices<
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
    A::Error == Unauthorized,
    D::Error == DecodeError,
    Exec::Error == InfrastructureError,
    Gateway::Error == PaymentError
{
    let initial = requestContext(request)
    let identified = attachRequestId(initial, using: ids)
    services.trace.record(.RequestIdentified(identified.requestId))

    let routed = match decodeCheckoutRoute(identified) {
        Err(error) => {
            return services.effects.pure(
                Err(Route(error))
            )
        }
        Ok(value) => value
    }
    services.trace.record(.RouteDecoded(routed.params.orderId))

    let authenticated = match authenticate(
        routed,
        using: authenticator
    ) {
        Err(_) => {
            return services.effects.pure(
                Err(Unauthorized)
            )
        }
        Ok(value) => value
    }
    services.trace.record(Authenticated(authenticated.user.id))

    let context = match decodeJson<CheckoutRequest>(
        authenticated,
        using: decoder
    ) {
        Err(error) => {
            return services.effects.pure(
                Err(Decode(error))
            )
        }
        Ok(value) => value
    }
    services.trace.record(BodyDecoded)

    checkoutHandler(
        context,
        using: services
    )
}
