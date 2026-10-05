class MyList {
  map(f) { f(self) }
}
let map_ref = &MyList.map...;
System.print(map_ref.is(Family))
