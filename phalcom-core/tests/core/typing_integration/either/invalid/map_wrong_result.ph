class Probe {
    @class
    run(_ source: Either<String, Int>) {
        let mut bad: Either<String, String> = source.map(|value| { value > 0 })
    }
}
