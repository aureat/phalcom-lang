class Probe {
    @class
    run(_ source: Either<String, Int>) {
        let mut bad: Either<Int, Int> = source.mapLeft(|value| { true })
    }
}
