class RowCalculusProbe {
    @class
    preserveRemainder() {
        let mut result = RowCalculus.preserve({ name: "Phalcom", stable: true, version: 1 })
        result
    }

    @class
    preserveValueAndRow() {
        let mut result = RowCalculus.preserveValue({ value: 42, cached: true, label: "answer" })
        result
    }

    @class
    annotateHigherOrder() {
        let mut result = RowCalculus.annotate(
            { value: 42, cached: true, name: "answer" },
            |value| { value == 42 }
        )
        result
    }

    @class
    preserveEmptyRemainder() {
        let mut result = RowCalculus.preserve({ name: "Phalcom" })
        result
    }

    @class
    expectedResultSelectsRow() {
        let mut result: { value: Int, label: String } = RowCalculus.make()
        result
    }
}
