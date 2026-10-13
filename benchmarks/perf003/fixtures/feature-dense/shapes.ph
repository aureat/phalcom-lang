class ShapeOps {
    @class
    identityRecord<**R>(
        _ value: { label: String, **R }
    ) -> { label: String, **R} {
        value
    }
}

type data Retry {
    Never
    Count
}

data RetryPolicy<R: Retry> {
    attempts: Int
}

const pair = (1, 2)
const details = { label: "perf", attempts: 3 }
let extended = ShapeOps.identityRecord(details)
export ShapeOps, Retry, RetryPolicy, pair, details, extended
