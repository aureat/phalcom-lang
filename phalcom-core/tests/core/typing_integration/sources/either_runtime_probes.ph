let mut runtimeLeft: Either<String, Int> = Either::Left("boom")
let mut runtimeRight: Either<String, Int> = Either::Right(41)

let mut runtimeLeftIsLeft = runtimeLeft.isLeft
let mut runtimeLeftIsRight = runtimeLeft.isRight
let mut runtimeRightIsLeft = runtimeRight.isLeft
let mut runtimeRightIsRight = runtimeRight.isRight

let mut mappedRight = runtimeRight.map(|value| { value + 1 })
let mut mappedRightValue = mappedRight.fold(
    left: |error| { -1 },
    right: |value| { value }
)

let mut mappedLeft = runtimeLeft.map(|value| { value + 1 })
let mut mappedLeftPreserved = mappedLeft.fold(
    left: |error| { error == "boom" },
    right: |value| { false }
)

let mut mappedLeftSide = runtimeLeft.mapLeft(|error| { error == "boom" })
let mut mappedLeftSideValue = mappedLeftSide.fold(
    left: |value| { value },
    right: |value| { false }
)

let mut bimapRight = runtimeRight.bimap(
    left: |error| { error == "boom" },
    right: |value| { value + 1 }
)
let mut bimapRightValue = bimapRight.fold(
    left: |value| { -1 },
    right: |value| { value }
)

let mut flatMapped = runtimeRight.flatMap(|value| {
    let mut next: Either<String, Bool> = Either::Right(value == 41)
    next
})
let mut flatMappedValue = flatMapped.fold(
    left: |error| { false },
    right: |value| { value }
)

let mut flatMappedLeft = runtimeLeft.flatMap(|value| {
    let mut next: Either<String, Bool> = Either::Right(value == 41)
    next
})
let mut flatMappedLeftPreserved = flatMappedLeft.fold(
    left: |error| { error == "boom" },
    right: |value| { false }
)

let mut swappedLeft = runtimeLeft.swap
let mut swappedLeftValue = swappedLeft.fold(
    left: |value| { false },
    right: |value| { value == "boom" }
)

let mut swappedRight = runtimeRight.swap
let mut swappedRightValue = swappedRight.fold(
    left: |value| { value == 41 },
    right: |value| { false }
)

let mut fallbackValue = runtimeLeft.getOrElse(99)
let mut preservedValue = runtimeRight.getOrElse(99)
let mut recoveredValue = runtimeLeft.recover(|error| 77)
let mut unrecoveredValue = runtimeRight.recover(|error| 77)

let mut replacement: Either<Bool, Int> = Either::Right(100)
let mut orElseLeft = runtimeLeft.orElse(replacement)
let mut orElseLeftValue = orElseLeft.fold(
    left: |value| -1,
    right: |value| value
)

let mut ignoredReplacement: Either<Bool, Int> = Either::Left(false)
let mut orElseRight = runtimeRight.orElse(ignoredReplacement)
let mut orElseRightValue = orElseRight.fold(
    left: |value| -1,
    right: |value| value
)

let mut zipOther: Either<String, Bool> = Either::Right(true)
let mut zipped = runtimeRight.zip(zipOther)
let mut zipValue = zipped.fold(
    left: |error| { -1 },
    right: |pair| {
        let mut (number, flag) = pair
        if flag { number } else { -1 }
    }
)

let mut runtimeAll = runtimeLeftIsLeft and not runtimeLeftIsRight and not runtimeRightIsLeft and runtimeRightIsRight and mappedRightValue == 42 and mappedLeftPreserved and mappedLeftSideValue and bimapRightValue == 42 and flatMappedValue and flatMappedLeftPreserved and swappedLeftValue and swappedRightValue and fallbackValue == 99 and preservedValue == 41 and recoveredValue == 77 and unrecoveredValue == 41 and orElseLeftValue == 100 and orElseRightValue == 41 and zipValue == 41
