import "../../src/typing" as Typing

class Left<T> {}
class Right<T> {}

const leftT: Typing.TypeParameter = Left.typeParameters.first
const rightT: Typing.TypeParameter = Right.typeParameters.first

assert(leftT.name == rightT.name)
assert(leftT.owner === Left)
assert(rightT.owner === Right)
assert(leftT.equivalentTo(rightT).not)
assert(leftT.hash != rightT.hash)

const leftAgain = Left.typeParameters.first
assert(leftT === leftAgain)
