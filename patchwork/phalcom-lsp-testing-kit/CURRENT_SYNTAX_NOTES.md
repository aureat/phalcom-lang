# Current syntax encoded by the fixtures

These choices follow the inspected repository, not speculative future syntax.

- inheritance: `class Child is Parent {}`.
- import: `import "./path" as Name`.
- imported members: ordinary member sends, e.g. `A.User`.
- positional parameter: `method(_ value)`.
- labeled parameter: `move(_ x, to dest)`.
- labeled call: `move(1, to: 2)`.
- setter: `value=(put next)`.
- subscript getter: `[_ index] { ... }`.
- subscript setter: `[_ index]=(put value) { ... }`.
- no-argument closure: `|| { ... }`.
- constructor attribute: `@constructor`.
- class-side attribute: `@class`.
- record fixture uses the parser's current `#{ ... }` spelling.
- selector literal: `#move(_,to)`.
- the parser baseline still contains parenthesized `for (x in xs)` tests.

Current superclass AST stores a single raw identifier, so this kit does not invent an unsupported `class Child is A.Parent` fixture. Cross-module class identity is tested through `A.User` / `B.User`.
