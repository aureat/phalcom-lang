class Probe {
    @class
    run(_ source: Either<String, Int>) {
        let bad: Either<Int, Int> = source.mapLeft(|value| { true })
    }
}
