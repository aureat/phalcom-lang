// area: errors
// spec: selectors.md §2
// status: NEGATIVE
// The ASI-hazard adjacency rule: `#` and the name must be directly adjacent
// (no whitespace) to lex as a symbol. A lone `#` followed by whitespace never
// forms a symbol and fails to lex.

const a = # move
