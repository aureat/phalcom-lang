// LANG005 — proposed future kernel trait surface.
//
// This is migration input for C6. Current production Iterable/for behavior
// remains the cursor-based implementation described by docs/spec/current/iteration.md.

// Target architecture after C6 generic evidence and production migration.
trait Iterator {
  type Item

  next -> Option<Self::Item>

  iterator { self }
}

trait Iterable {
  type Item
  type Iterator

  iterator -> Self::Iterator
}

// C6 must supply the ordinary evidence relationships:
//   Self::Iterator conforms to Iterator
//   Self::Iterator::Item = Self::Item
//
// C6 must also establish the general Iterator ⇒ Iterable relationship. These
// relationships are intentionally not represented as provisional trait-bound
// syntax in this support artifact.

// Target laws:
// - a reusable source's iterator getter creates a fresh traversal;
// - an iterator's iterator getter returns the same traversal object;
// - next returns Option<Item> directly;
// - None is stable exhaustion;
// - ordinary errors propagate and are not converted to None;
// - no public cursor, hasNext, reset, or clone requirement remains.
