class FixedRequestIdFactory {
    value: RequestId
}

impl FixedRequestIdFactory {
    next -> RequestId {
        value
    }
}

impl RequestIdFactory for FixedRequestIdFactory {
    next -> RequestId {
        value
    }
}
