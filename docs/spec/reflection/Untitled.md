```ph
class Behavior {
    methods -> MethodDictionary
    ownMethods -> MethodDictionary
}
```

```ph
@readonly
class MethodDictionary {
	owner -> Behavior
	size -> Int

	selectors -> MethodSelectorSet
	values -> MethodSet
	entries -> MethodEntrySet

	[_ selector: Selector] -> Method
	at(_ selector: Selector) -> Option<Method>
	contains(_ selector: Selector) -> Bool

	snapshot -> MethodDictionarySnapshot
	
	iterator -> ???
}
```

```ph
class Selector {
	base -> Symbol
	arity -> Int
	parts -> SelectorParts
}
```

```ph
class SelectorParts {
	size -> Int
	[_ position: Int] -> SelectorPart
	at(_ position: Int) -> Option<SelectorPart>
}
```

```ph
@data
class SelectorPart {
	position -> Int
	label -> Option<Symbol>
}
```

```ph
List #size
Behavior #class
String.class #empty
String #trim(_)

User
class User // it's also User.class
class User.class
User.class#name
```

```ph
const object = Object.new()
object.someMethod(10, 20, debug: true)
object.method(10, 20, :debug, #{^debug, ^dump})
```

```ph
class User {
	const _name: Option<String>
	const _age: Option<Int>
	
	@constructor
	call(_ name: String, _ age: Int) {
		_name = Some(name)
		_age = Some(age)
	}

	@class
	anonymous() { User(None, None) }
	
	name { _name }
	age { _age }
	
	+()
	
	toString {
		"User(\(_name), \(_age))"
	}
	
	toDebugString {
		"""User(
			\(_name.toDebugString),
			\(_age.toDebugString)
		)"""
	}
}

const user = User("Altun", 24)

user::Some()
user::None()

&object.(method...)

&object.(...)

&object[_, _, default]=(_)
```

```ph
User(...)
```

```ph
enum Option<T> {
	type Some(_ value: T)
	type None
}

type UnitCircle {

	private constructor(x: Int, y: Int)

	@constructor 
	of(_ x: Int, _ y: Int) -> Result<UnitCircle, Error> {
		if x**2 + y**2 == 1 
			then UnitCircle(x, y)
			else Error("x**2 and y**2 should add up to 1")
	}
}

class for Option<T> {
	
}

```

```ph
data Point(x: Int, y: Int) {
	constructor(private)
	
	
}

data Config(
	_ name: String, 
	options: #{
		ip: String,
		host: String,
	}
) class {
	
}

data ConfigOptions {
	ip: String,
	host: String
} class {
	
}

```

```ph
type class Trait {
	name -> String
}

class Implementer {
	_name: String
	name -> String = _name
}

class Implementer {
	@get _name: String
}

impl Trait for Implementer {
	
}
```

```ph
trait Counter {
	private mut count: Int
	// translates to:
	// count -> Int
	// count=(_: Int) -> ()
	
	increment { count++ }
}

class CounterImpl with Counter {
	private mut count: Int = _count
	// translates to
	// _count: Int
	// private mut count = _count
	// which in turn means
	// count
}
```