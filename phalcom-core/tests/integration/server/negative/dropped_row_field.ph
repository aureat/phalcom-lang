// Expected failure: decodeJson requires `rawBody: Bytes`, but this projection
// intentionally drops it before the decoder stage.
invalidDroppedRow<D>(
    request: RawRequest,
    using decoder: D
)
where
    D: Decoder<CheckoutRequest>,
    D::Error == DecodeError
{
    let initial = requestContext(request)
    let dropped = {
        method: initial.method,
        path: initial.path,
        headers: initial.headers,
    }

    decodeJson<CheckoutRequest>(
        dropped,
        using: decoder
    )
}
