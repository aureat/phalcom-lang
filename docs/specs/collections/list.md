# `List<T>`

`List<T>` is a mutable, dynamically resizable sequence. Its generic argument
describes the source-level element type; it does not imply a numeric conversion
or a separate collection identity for each runtime allocation. `List.new()`
creates a fresh empty list. `List.withCapacity(n)` creates an empty list after
reserving capacity and requires a nonnegative `n`.

## Reading and writing

- `size` reports the number of elements. `first` and `last` return `None` for
  an empty list and `Some(value)` otherwise.
- `at(index)` uses the collection's direct indexed access and fails for an
  invalid index. `get(index)` returns `Option` and accepts negative indices
  relative to the end. `get(index, orElse:)` calls its fallback only when the
  index is absent.
- Bracket read access supports the specified index/default and range forms.
  Range access returns a fresh List containing the selected values. It does not
  create a mutable aliasing slice.
- `append`, `prepend`, `insert`, `remove`, `swap`, `removeAll`, indexed
  assignment, and `replace` mutate the receiver. `append` and the insertion
  operations do not change the identity of the List.
- `iterator` returns a fresh iterator beginning at the first current element.
  The canonical `Iterator`/`Iterable` laws are owned by
  [`../iterator.md`](../iterator.md).

`join` builds a String by calling each element's string conversion, with an
optional separator. `toMap(merging:)` delegates entry construction and duplicate
key handling to the provided merge callable.

## Equality and literals

List equality is structural and order-sensitive. Mutability does not make
ordinary equality identity-based. Its hash behavior follows the accepted
collection hashability rules; mutable Lists cannot be `Map` keys or `Set`
members.

The literal `[a, b, c]` constructs a fresh mutable List, evaluates elements
eagerly from left to right, permits a trailing comma, and is construction syntax
only. It does not add subscript syntax for arbitrary expressions. The empty
literal `[]` is an empty List. See
[`../collection-literals.md`](../collection-literals.md).

Implementations may use a native growable backing store, but capacity and
physical representation are not part of the source value's identity.
