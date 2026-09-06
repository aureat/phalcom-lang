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

```