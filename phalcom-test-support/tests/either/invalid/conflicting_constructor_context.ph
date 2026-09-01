class Probe {
    @class
    run() {
        let bad: Either<String, Int> = Either::Left(42)
    }
}
