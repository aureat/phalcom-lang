let mut order = List.new()
let getLeft = || {
  order.append("L")
  2
}
let getRight = || {
  order.append("R")
  [1, 2, 3]
}
let mut res = getLeft.call() in getRight.call()
System.print(order.join(""))
System.print(res)
