class RowEitherInvalidProbe {
    @class
    collision(
        _ source: Either<String, { name: String, tag: String }>
    ) {
        let mut result = source.map(|record| {
            RowCalculus.tagged(record)
        })
    }
}
