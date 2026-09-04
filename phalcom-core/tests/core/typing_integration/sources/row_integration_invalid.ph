class RowEitherInvalidProbe {
    @class
    collision(
        _ source: Either<String, #{ name: String, tag: String }>
    ) {
        let result = source.map(|record| {
            RowCalculus.tagged(record)
        })
    }
}
