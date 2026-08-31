// area: imports
// spec: U15 plan §1/§6; object-model.md §4; DEC-U15 A+A (relative file-path
// resolution + whole-module binding)
// status: PASS
// The canonical `import` example: `lib/answer.ph` defines a top-level
// `answer`; reading `B.answer` reaches it as an ordinary send on the
// imported `Module` — "everything top-level is a member" (Draft 0.1, no
// explicit `export`).

import "./lib/answer" as B
System.print(B.answer)
