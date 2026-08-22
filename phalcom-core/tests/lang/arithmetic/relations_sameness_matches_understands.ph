// area: arithmetic
// spec: ===, matches, understands, ...

// 1. === exact sameness
let s1 = "hello"
let s2 = "hello"
System.print(s1 === s1)
System.print(s1 === s2)
System.print(1 === 1)
System.print(1 === 1.0)
System.print(Some(1) === Some(1))
System.print(None === None)

// 2. candidate matches pattern
System.print("abc" matches "abc")
System.print("abc" matches "def")
System.print(#+(_) matches SelectorPattern(#+...))

// 3. object understands selector
System.print([1, 2] understands #size)
System.print([1, 2] understands #nonExistentMethod)

// 4. ... ellipsis
System.print(...)
System.print(... === Ellipsis.instance)

// 5. Comparison chain: 0 <= x < 10
let x = 5
System.print(0 <= x < 10)
System.print(0 <= x < 3)
