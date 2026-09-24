class CheckoutRequestDecoder {
    value: CheckoutRequest
}

impl Decoder<CheckoutRequest> for CheckoutRequestDecoder {
    type Error = DecodeError

    decode(
        _ bytes: Bytes
    ) -> Result<CheckoutRequest, DecodeError> {
        Ok(value)
    }
}
