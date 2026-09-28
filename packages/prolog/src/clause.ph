// Clause is intentionally a marker for now.
//
// A solver eventually needs to distinguish:
//
//   Fact:
//     Relationship<...>
//
//   Rule:
//     Rule<Head, *Body>
//
// But we should not erase either representation merely to make
// Clause expose one homogeneous runtime structure.
//
trait Clause {}

export Clause