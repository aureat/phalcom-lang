class RowPipelineProbe {
    @class
    redecompose() {
        let input = #{ name: "Phalcom", age: 8, enabled: true }
        let tagged = RowCalculus.tagged(input)
        let result = RowCalculus.consumeTagged(tagged)
        result
    }
}
