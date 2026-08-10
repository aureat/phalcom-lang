// Private stable fingerprints for in-memory search identity. Persistent
// database key/version encoding remains Phase 08 work.

import Example from "choices/example"

class _Fingerprints {
  @class
  example(value: Example) -> String { value.signature }
}
