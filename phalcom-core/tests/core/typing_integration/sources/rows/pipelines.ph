class RowPipelineProbe {
    @class
    redecompose() {
        let mut input = { name: "Phalcom", age: 8, enabled: true }
        let mut tagged = RowCalculus.tagged(input)
        let mut result = RowCalculus.consumeTagged(tagged)
        result
    }
}
