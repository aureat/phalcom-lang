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
