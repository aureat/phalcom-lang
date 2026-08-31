// area: imports
// spec: U15 plan §4/§6
// status: PASS
// Kernel classes (`Object`, `Number`, `List`, …) are the bootstrap, not a
// module: an imported unit uses `List` with no `import` of it.

import "./lib/kernel_user" as K
System.print(K.total)
