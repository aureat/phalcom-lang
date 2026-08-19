from .base import (
  Var,
  BinOp,
)

const x = Var.number(#x)
const y = Var.number(#y)
const a = Var.number(#a)
const b = Var.number(#b)

const expr1 = (x + y) * (a - b) * 5 + (x / a) - (y % b)

// const z = Var.bool(#z)
// const w = Var.bool(#w)
// const expr2 = z + w
// System.print(z.toRepr)

const main = |*args| {
  System.print(expr1.toRepr)
}

main()