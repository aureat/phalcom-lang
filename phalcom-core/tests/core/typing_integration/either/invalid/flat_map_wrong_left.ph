class Probe {
    @class
    run(_ source: Either<String, Int>, _ wrong: Either<Int, Bool>) {
        let mut bad = source.flatMap(|value| {
            wrong
        })
    }
}
