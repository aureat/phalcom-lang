```typescript
(#Some(_), 10)
(#Some(_), 10, nestingDepth: 1)
(#None, ())

SomeConstructor: (T) -> (#Some(_), T)) = |value: T| (#Some(_), value)
NoneConstructor: () -> (#None, ()) = Singleton<None>(|| (#None, ()))
OptionMethodTable = #{
	Some(_): SomeConstructor,
	NoneConstructor: None
}
Option = TypeClass(OptionMethodTable)
// TypeClass is Class ???

type Option<T> =
	#{
		tag: #Some(_),
		payload: (T), // unary tuple
	}
	|
	#{
		tag: #None
		payload: () // the singleton empty tuple
	}

// type alias
type TaggedUnionMember<Tag, Payload> = 
	#{
		tag: Tag,
		payload: Payload
	}
	
type Option.Some(_)<Payload> = TaggedUnionMember<SymbolLiteral<#Some(_)>, Payload>

type Option.None<()> = NoneOfOption = TaggedUnionMember<SymbolLiteral<#None>, ()>
```

```typescript
@dataclass
class Person {
	const _name: String
	const _age: Int
}

#{
	tag: #Person
	payload: {
		name: String,
		age: Int
	},
	methods: #{
		name: |self| {
			self.payload[#name]
		}
		name=(value): |self, value| {
			self.payload[#name] = value
		},
		age: |self| {
			self.payload[#age]
		}
		age=(value): |self, value| {
			self.payload[#age] = value
		}
	},
}

@typeclass
class Option<T> {
	variants {
		Some(_ value: T),
		None(), // produces a new empty object every time — you probably don't want this.
		None; // singleton type
	}
	
	
}


```