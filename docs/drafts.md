### anonymous callables

```python
numericTypes.map 

numericTypes.map(pos) |type: Type| -> Type { List<type> }


```

```python
[]
[]=(_)
[_, _, _]
[_, _, _]=(_)
```

```python
numbers
	.map (|n| 2 * n)
	.where (|n| n < 100)
	
numbers
	.map |value| { value > 3 }
	.where |n| { n < 100 }
	
users
	.where (|user| user.active)
	.map (|user| user.name)
	.where (|name| not name.isEmpty)
	.toList

let optionallyAnnotated: (Int, Int) -> Int

const fn = |x, y| x + y
const fn = |x, y| { x + y }
const fn = |x, y| {
	x + y
}
```

```typescript
const removedCount: Int = users
	.removeAll where: |user| { user.expired }
	.any where: |user| { user.expired }
	
{
	+(other): ...,
	-(other): ...,
}

(a, b, c: d, options: {
	
})

(Int, Int, c: String, options: {
	
})
```

```typescript

predicate
	.ifTrue(|| { ... }, 
		ifFalse: || { ... })

predicate
	.ifTrue || { ... } 
		ifFalse: || { ... }

predicate.match(
	true: || ...,
	false: || ...
)

predicate.match
	true: || { ... },
	false: || { ... }

result.match
	ok: |v| {
		...
	}
	err: |e| {
		...
	}
	
result.match(ok: |v| ..., err: |e| ...)
```

```python
import fingerprints.
```