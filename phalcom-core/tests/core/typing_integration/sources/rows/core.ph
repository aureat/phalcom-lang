class RowCalculus {
    @class
    preserve<R: RecordFields>(
        _ value: { name: String, **R }
    ) -> { name: String, **R } {
        value
    }

    @class
    preserveValue<T, R: RecordFields>(
        _ value: { value: T, **R }
    ) -> { value: T, **R } {
        value
    }

    @class
    annotate<A, B, R: RecordFields>(
        _ value: { value: A, **R },
        _ transform: (A) -> B
    ) -> { value: A, mapped: B, **R } {
        { **value, mapped: transform.call(value.value) }
    }

    @class
    tagged<R: RecordFields>(
        _ value: { name: String, **R }
    ) -> { name: String, tag: String, **R } {
        { **value, tag: "entity" }
    }

    @class
    sameRemainder<R: RecordFields>(
        _ left: { id: Int, **R },
        _ right: { id: Int, **R }
    ) -> { id: Int, **R } {
        left
    }

    @class
    consumeTagged<R: RecordFields>(
        _ value: { tag: String, **R }
    ) -> { tag: String, **R } {
        value
    }

    @class
    make<R: RecordFields>() -> { value: Int, **R } {
        { value: 1 }
    }
}
