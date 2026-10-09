let mut order = List.new()
let getLeft = || {
  order.append("L")
  2
}
let getRight = || {
  order.append("R")
  let values: List<Int> = [1, 2, 3]
  values
}
let mut res = getLeft.call() in getRight.call()
System.print(order.join(""))
System.print(res)
