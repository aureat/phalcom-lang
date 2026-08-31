class MyList {
  map(f) { f(self) }
}
const map_ref = MyList::map::*;
System.print(map_ref.is(Family))
