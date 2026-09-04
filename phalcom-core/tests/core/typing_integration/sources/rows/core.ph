class RowCalculus {
    @class
    preserve<R: RecordRow>(
        _ value: #{ name: String, | R }
    ) -> #{ name: String, | R } {
        value
    }

    @class
    preserveValue<T, R: RecordRow>(
        _ value: #{ value: T, | R }
    ) -> #{ value: T, | R } {
        value
    }

    @class
    annotate<A, B, R: RecordRow>(
        _ value: #{ value: A, | R },
        _ transform: (A) -> B
    ) -> #{ value: A, mapped: B, | R } {
        #{ **value, mapped: transform.call(value.value) }
    }

    @class
    tagged<R: RecordRow>(
        _ value: #{ name: String, | R }
    ) -> #{ name: String, tag: String, | R } {
        #{ **value, tag: "entity" }
    }

    @class
    sameRemainder<R: RecordRow>(
        _ left: #{ id: Int, | R },
        _ right: #{ id: Int, | R }
    ) -> #{ id: Int, | R } {
        left
    }

    @class
    consumeTagged<R: RecordRow>(
        _ value: #{ tag: String, | R }
    ) -> #{ tag: String, | R } {
        value
    }

    @class
    make<R: RecordRow>() -> #{ value: Int, | R } {
        #{ value: 1 }
    }
}
