let runtimeMonad = StringEitherMonad.new()
let runtimeRight: Either<String, Int> = Either::Right(41)
let runtimeLeft: Either<String, Int> = Either::Left("boom")

let monadMappedRight = runtimeMonad.map(runtimeRight, |value| { value + 1 })
let monadMappedRightValue = monadMappedRight.fold(
    left: |error| { -1 },
    right: |value| { value }
)

let monadMappedLeft = runtimeMonad.map(runtimeLeft, |value| { value + 1 })
let monadMappedLeftPreserved = monadMappedLeft.fold(
    left: |error| { error == "boom" },
    right: |value| { false }
)

let monadPure = runtimeMonad.pure(7)
let monadPureValue = monadPure.fold(
    left: |error| { -1 },
    right: |value| { value }
)

let map2Right: Either<String, Int> = Either::Right(1)
let map2Other: Either<String, Int> = Either::Right(2)
let monadMap2 = runtimeMonad.map2(map2Right, map2Other, |a, b| { a + b })
let monadMap2Value = monadMap2.fold(
    left: |error| { -1 },
    right: |value| { value }
)

let map2Failure: Either<String, Int> = Either::Left("map2-fail")
let monadMap2Failure = runtimeMonad.map2(map2Failure, map2Other, |a, b| { a + b })
let monadMap2FailurePreserved = monadMap2Failure.fold(
    left: |error| { error == "map2-fail" },
    right: |value| { false }
)

let flatMapNext: (Int) -> Either<String, Bool> = |value| {
    let next: Either<String, Bool> = Either::Right(value == 41)
    next
}
let monadFlatMap = runtimeMonad.flatMap(runtimeRight, flatMapNext)
let monadFlatMapValue = monadFlatMap.fold(
    left: |error| { false },
    right: |value| { value }
)
let monadFlatMapFailure = runtimeMonad.flatMap(runtimeLeft, flatMapNext)
let monadFlatMapFailurePreserved = monadFlatMapFailure.fold(
    left: |error| { error == "boom" },
    right: |value| { false }
)

let kleisliFirst: (Int) -> Either<String, Int> = |value| {
    let next: Either<String, Int> = Either::Right(value + 1)
    next
}
let kleisliSecond: (Int) -> Either<String, Bool> = |value| {
    let next: Either<String, Bool> = Either::Right(value == 42)
    next
}
let runtimeKleisli = MonadAlgorithms.kleisli(runtimeMonad, kleisliFirst, kleisliSecond)
let runtimeKleisliValue = runtimeKleisli.call(41).fold(
    left: |error| { false },
    right: |value| { value }
)

let traverseSuccessTransform: (Int) -> Either<String, Int> = |value| {
    let next: Either<String, Int> = Either::Right(value + 10)
    next
}
let runtimeTraverseSuccess = MonadAlgorithms.traverse(runtimeMonad, [1, 2, 3], traverseSuccessTransform)
let runtimeTraverseSuccessValue = runtimeTraverseSuccess.fold(
    left: |error| { false },
    right: |values| {
        values.size == 3 and values[0] == 11 and values[1] == 12 and values[2] == 13
    }
)

let traverseFailureTransform: (Int) -> Either<String, Int> = |value| {
    if (value == 2) {
        let failed: Either<String, Int> = Either::Left("traverse-fail")
        failed
    } else {
        let success: Either<String, Int> = Either::Right(value)
        success
    }
}
let runtimeTraverseFailure = MonadAlgorithms.traverse(runtimeMonad, [1, 2, 3], traverseFailureTransform)
let runtimeTraverseFailurePreserved = runtimeTraverseFailure.fold(
    left: |error| { error == "traverse-fail" },
    right: |values| { false }
)

let runtimeAll = monadMappedRightValue == 42 and monadMappedLeftPreserved and monadPureValue == 7 and monadMap2Value == 3 and monadMap2FailurePreserved and monadFlatMapValue and monadFlatMapFailurePreserved and runtimeKleisliValue and runtimeTraverseSuccessValue and runtimeTraverseFailurePreserved
