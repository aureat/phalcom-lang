// LANG005 — proposed kernel hash capability.
//
// This deliberately does not decide whether every Object is Hashable.
//
// Law:
//   a == b  =>  a.hash == b.hash

trait Hashable {
  hash -> Int
}
