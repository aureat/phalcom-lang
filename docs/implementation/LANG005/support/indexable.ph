// LANG005 — proposed index capabilities.
//
// MutableIndexable repeats the read requirement until supertrait semantics are
// explicitly ratified.

trait Indexable {
  type Index
  type Item

  [_ index: Index] -> Item
}

trait MutableIndexable {
  type Index
  type Item

  [_ index: Index] -> Item
  [_ index: Index]=(_ value: Item) -> Unit
}
