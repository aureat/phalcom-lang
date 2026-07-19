// area: collections
// spec: lexical-structure.md §8; ADR-0032 §3.2; U9 spread follow-on
// status: NEGATIVE
// `parse_comma_exprs` reserves a leading-`*` spread slot (kept pattern
// -compatible for the concurrent U14 destructuring scanner). U-COLL ships no
// spread, so a spread element is rejected with a precise diagnostic rather
// than silently mis-parsed — pinning the reserved slot's behaviour.

const l = [*xs, 1]
