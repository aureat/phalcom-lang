data UnitScore private(value: Float)

trait Distance<Input, Value> {
    distance(left: Input, right: Input) -> Value
}

trait Similarity<Input, Value> {
    similarity(left: Input, right: Input) -> Value
}

trait Score<Input, Value> {
    score(left: Input, right: Input) -> Value
}

trait Symmetric<Input, Value> {}

// Distance<Input, Value>
trait Metric<Input, Value> {}

// Distance<Input, UnitScore>
trait NormalizedDistance<Input> {}

// Similarity<Input, UnitScore>
trait NormalizedSimilarity<Input> {}
