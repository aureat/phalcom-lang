// area: imports/negative
// spec: U15 plan §5/§6
// status: NEGATIVE
// A relative import that resolves to no existing `.ph` file raises a clean
// error naming the attempted path, never a panic.

import "./does_not_exist" as X
