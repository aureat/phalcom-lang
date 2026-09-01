## GCS001

Generic identity solves parameter from 

```ph
class Probe {
	@class
	identity<T>(_ value: T) -> T { ... }
}

let a = Probe.identity(10) // a: Int
let b = Probe.identity(true) // b: Bool

assert()
```
