type data Rounding {
    TowardZero
    TowardPositiveInfinity
    TowardNegativeInfinity
    Nearest
}

type data BoundKind {
    Inclusive
    Exclusive
    Unbounded
}

type data Endian {
    Little
    Big
    Native
}

type data Overflow {
    Wrap
    Saturate
    Trap
}