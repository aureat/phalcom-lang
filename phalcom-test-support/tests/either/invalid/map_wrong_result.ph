class Probe {
    @class
    run() {
        let source: Either<String, Int> = Either::Right(1)
        let bad: Either<String, String> = source.map(|value| { value > 0 })
    }
}
