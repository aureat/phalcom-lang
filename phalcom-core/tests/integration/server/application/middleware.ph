// Convert the nominal HTTP input into the structural context threaded through
// the middleware pipeline. Row-polymorphic middleware starts after this edge.
requestContext(
    request: RawRequest
) -> {
    method: Method,
    path: String,
    headers: Headers,
    rawBody: Bytes
} {
    {
        method: request.method,
        path: request.path,
        headers: request.headers,
        rawBody: request.rawBody,
    }
}

attachRequestId<**R>(
    context: { **R },
    using ids: RequestIdFactory
) -> { requestId: RequestId, **R } {
    {
        requestId: ids.next,
        **context,
    }
}

decodeCheckoutRoute<**R>(
    context: { path: String, **R }
) -> Result<
    {
        params: { orderId: OrderId },
        path: String,
        **R
    },
    RouteDecodeError
> {
    if context.path != "/orders/ord-123/checkout" {
        return Err(InvalidCheckoutPath)
    }

    Ok({
        params: { orderId: OrderId("ord-123") },
        **context,
    })
}

authorizationToken<**R>(
    context: { headers: Headers, **R }
) -> Result<AuthToken, Unauthorized> {
    match context.headers.authorization {
        None => Err(Unauthorized {})
        Some(value) => Ok(AuthToken(value))
    }
}

authenticate<A: Authenticator, **R>(
    context: {
        headers: Headers,
        **R
    },
    using authenticator: A
) -> Result<
    {
        user: User,
        headers: Headers,
        **R
    },
    A::Error
>
where A::Error == Unauthorized
{
    let token = match authorizationToken(context) {
        Err(_) => {
            // This middleware's concrete golden authenticator uses Unauthorized
            // as its associated error; later tasks can generalize extraction.
            return Err(Unauthorized {})
        }
        Ok(value) => value
    }

    match authenticator.authenticate(token) {
        Err(error) => Err(error)
        Ok(user) => {
            Ok({ user, **context })
        }
    }
}

decodeJson<T, D, **R>(
    context: { rawBody: Bytes, **R },
    using decoder: D
) -> Result<
    {
        body: T,
        rawBody: Bytes,
        **R
    },
    D::Error
>
where D: Decoder<T>
{
    match decoder.decode(context.rawBody) {
        Err(error) => Err(error)
        Ok(body) => {
            Ok({ body, **context })
        }
    }
}
