let runtimeMonad = StringEitherMonad.new()
let runtimeRight: Either<String, Int> = Either::Right(41)
let runtimeLeft: Either<String, Int> = Either::Left("boom")

let monadMappedRight = runtimeMonad.map(runtimeRight, |value| { value + 1 })
let monadMappedRightValue = monadMappedRight.fold(
    left: |error| { -1 },
    right: |value| { value }
)

let mapLeftTransformCalls = 0
let monadMappedLeft = runtimeMonad.map(runtimeLeft, |value| {
    mapLeftTransformCalls = mapLeftTransformCalls + 1
    value + 1
})
let monadMappedLeftPreserved = monadMappedLeft.fold(
    left: |error| { error == "boom" },
    right: |value| { false }
)
let monadMapLeftShortCircuited = mapLeftTransformCalls == 0

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

let map2LeftFailureCalls = 0
let map2Failure: Either<String, Int> = Either::Left("map2-fail")
let monadMap2Failure = runtimeMonad.map2(map2Failure, map2Other, |a, b| {
    map2LeftFailureCalls = map2LeftFailureCalls + 1
    a + b
})
let monadMap2FailurePreserved = monadMap2Failure.fold(
    left: |error| { error == "map2-fail" },
    right: |value| { false }
)
let monadMap2LeftShortCircuited = map2LeftFailureCalls == 0

let map2RightFailureCalls = 0
let map2RightFailure: Either<String, Int> = Either::Left("map2-right-fail")
let monadMap2RightFailure = runtimeMonad.map2(map2Right, map2RightFailure, |a, b| {
    map2RightFailureCalls = map2RightFailureCalls + 1
    a + b
})
let monadMap2RightFailurePreserved = monadMap2RightFailure.fold(
    left: |error| { error == "map2-right-fail" },
    right: |value| { false }
)
let monadMap2RightShortCircuited = map2RightFailureCalls == 0

let flatMapNext: (Int) -> Either<String, Bool> = |value| {
    let next: Either<String, Bool> = Either::Right(value == 41)
    next
}
let monadFlatMap = runtimeMonad.flatMap(runtimeRight, flatMapNext)
let monadFlatMapValue = monadFlatMap.fold(
    left: |error| { false },
    right: |value| { value }
)

let flatMapFailureNextCalls = 0
let flatMapFailureNext: (Int) -> Either<String, Bool> = |value| {
    flatMapFailureNextCalls = flatMapFailureNextCalls + 1
    let next: Either<String, Bool> = Either::Right(value == 41)
    next
}
let monadFlatMapFailure = runtimeMonad.flatMap(runtimeLeft, flatMapFailureNext)
let monadFlatMapFailurePreserved = monadFlatMapFailure.fold(
    left: |error| { error == "boom" },
    right: |value| { false }
)
let monadFlatMapShortCircuited = flatMapFailureNextCalls == 0

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

let traverseFailureTransformCalls = 0
let traverseFailureTransform: (Int) -> Either<String, Int> = |value| {
    traverseFailureTransformCalls = traverseFailureTransformCalls + 1
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
let runtimeTraverseShortCircuited = traverseFailureTransformCalls == 2

let runtimeAll = monadMappedRightValue == 42 and monadMappedLeftPreserved and monadMapLeftShortCircuited and monadPureValue == 7 and monadMap2Value == 3 and monadMap2FailurePreserved and monadMap2LeftShortCircuited and monadMap2RightFailurePreserved and monadMap2RightShortCircuited and monadFlatMapValue and monadFlatMapFailurePreserved and monadFlatMapShortCircuited and runtimeKleisliValue and runtimeTraverseSuccessValue and runtimeTraverseFailurePreserved and runtimeTraverseShortCircuited
