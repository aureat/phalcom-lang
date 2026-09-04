class RowEitherIntegrationProbe {

    @class
    standaloneAnnotate(
        _ record: #{ cached: Bool, name: String, value: Int }
    ) {
        let result = RowCalculus.annotate(record, |value| { value == 0 })
    }

    @class
    mapRecord(
        _ source: Either<String, #{ cached: Bool, name: String, value: Int }>
    ) {
        let mapped = source.map(|record| {
            RowCalculus.annotate(
                record,
                |value| { value == 0 }
            )
        })
    }
}

class RowNestedAdtProbe {
    @class
    preserveNested(
        _ payload: Either<String, Int>
    ) {
        let result = RowCalculus.preserveValue(
            #{ value: payload, cached: true, label: "nested" }
        )
    }
}

class RowMonadIntegrationProbe {
    @class
    bindRecord(
        _ monad: StringEitherMonad,
        _ source: Either<String, #{ cached: Bool, name: String, value: Int }>
    ) {
        let result = MonadAlgorithms.bind(
            monad,
            source,
            |record| {
                let transformed = RowCalculus.annotate(
                    record,
                    |value| { value > 0 }
                )
                let next: Either<String, #{ cached: Bool, mapped: Bool, name: String, value: Int }> =
                    Either::Right(transformed)
                next
            }
        )
    }
}
