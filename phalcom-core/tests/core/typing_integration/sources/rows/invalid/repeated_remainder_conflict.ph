class RowCorrelationConflictProbe {
    @class
    incompatible() {
        let mut result = RowCalculus.sameRemainder(
            { id: 1, label: "left" },
            { id: 2, count: 3 }
        )
        result
    }
}
