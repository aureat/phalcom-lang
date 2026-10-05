class Ref {
  mut _row
  mut _col
  @constructor
  at(_ row, _ col) { _row = row; _col = col }
  row { _row }
  col { _col }
}
let mut C = Ref
let mut r = C.at(2, 8)
System.print(r.row)
System.print(r.col)
