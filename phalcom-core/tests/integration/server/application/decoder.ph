trait Decoder<T> {
    type Error: Type

    decode(
        bytes: Bytes
    ) -> Result<T, Self::Error>
}
