@!documentation("Canonical Iterator and Iterable trait laws.")

// Transition module for LANG005.C6.P4. The legacy cursor classes remain in
// `iterable.ph` until P5 retires their runtime superclass protocol. These
// traits are source-authored semantic contracts and do not alter that cursor
// implementation or the production collection hierarchy.
trait Iterator {
  type Item

  next -> Option<Self::Item>

  iterator { self }
}

trait Iterable
  where
    Self::Iterator |= Iterator,
    <Self::Iterator as Iterator>::Item == Self::Item
{
  type Item
  type Iterator

  iterator -> Self::Iterator
}

// The canonical Iterator => Iterable theorem is an ordinary guarded
// generic-target conformance. It intentionally remains in this transition
// module while the legacy public Iterable runtime class is retired by P5.
impl<I> Iterable for I
  where I |= Iterator
{
  type Item = <I as Iterator>::Item
  type Iterator = I

  iterator { self }
}

export Iterator, Iterable
