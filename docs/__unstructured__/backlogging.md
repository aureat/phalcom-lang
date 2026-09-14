
## Types

#### Syntax

- `<:` `>:`
- `<~>`
- `as`, `as?`
- `User?` => `Option<User>`

#### Selective Reification

- Separate singleton instance for each specialized type
	- native `@singleton`
- Reflection
- Simple composable type descriptors

### Modules

- `__module__` and other dunder names in repl.
- test whether dunder in normal projects and imported modules, packages work correctly.
- Consider making module paths associated lookup

```ph
from universe::collections::bytes import Bytes

const { Bytes } = universe::collections::bytes::Bytes
```
