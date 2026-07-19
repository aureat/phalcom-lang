// area: collections
// spec: lexical-structure.md §6 (brace disambiguation); ADR-0032 §3.1
// status: PASS (anti-regression harness for the U4 block interaction)
// Four of the five §6 brace rows stay blocks after the map discriminator was
// added (`{ IDENT : }` ⇒ map is the only new branch). The map row itself is
// pinned separately in `negative/map_literal_pending.ph`.
//   { x, y => … }  two-parameter block
//   { x => … }     one-parameter block
//   { }            empty block
//   { expr }       zero-parameter block

const two = { x, y => x + y }
System.print(two.call(3, 4))
const one = { x => x }
System.print(one.call(5))
const empty = {}
System.print(empty.class)
const zero = { 1 + 1 }
System.print(zero.call())
