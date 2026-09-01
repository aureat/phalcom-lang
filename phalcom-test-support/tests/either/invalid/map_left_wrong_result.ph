class Probe {
    @class
    run() {
        let source: Either<String, Int> = Either::Left("error")
        let bad: Either<Int, Int> = source.mapLeft(|value| { true })
    }
}
