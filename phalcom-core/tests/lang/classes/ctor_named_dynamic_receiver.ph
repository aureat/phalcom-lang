class Ref {
  @constructor
  at(row, col) { _row = row; _col = col }
  row => _row
  col => _col
}
let C = Ref
let r = C.at(2, 8)
System.print(r.row)
System.print(r.col)
