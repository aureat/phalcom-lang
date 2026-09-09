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


```