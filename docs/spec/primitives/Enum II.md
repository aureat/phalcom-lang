```ph

enum Direction {
	.Up 
	.Down 
	.Left 
	.Right
}

enum Direction {
	.North = 0
	.West = 1
	.East = 2
	.South = 3
}

```

```ph

enum CompanyKind {
	.Some<T>(_ value: T)
	.None
	
	toString {
		match self {
			Some(value) => "Some(\(value))",
			None => "None",
		}
	}
}
```

```ph
enum Expr<T> {
	case Int(_ value: Int) -> Expr<Int>
	
	case Bool(_ value: Bool) -> Expr<Bool>

	case Add(
		_ left: Expr<Int>,
		_ right: Expr<Int>
	) -> Expr<Int>

	case Equal<U>(
		_ left: Expr<U>,
		_ right: Expr<U>
	) -> Expr<Bool>

	eval -> T {
		...
	}
}

enum Expr<T> =
	| Int(_ value: Int) -> Expr<Int>
	| Bool(_ value: Bool) -> Expr<Bool>
	| Add(
		_ left: Expr<Int>,
		_ right: Expr<Int>,
	) -> Expr<Int>
	| Equal<U>(
		_ left: Expr<U>,
		_ right: Expr<U>,
	)
```

```ph
NominalType Option<T> {

    representation =
          Tagged<#Some(_), (T,)>
        | Tagged<#None, ()>

    constructors = {
        Some(_) :
            <T>(T) -> Option<T>

        None :
            <T> Option<T>
    }

    methods = {
        map<U> :
            (Option<T>, (T) -> U) -> Option<U>

        flatMap<U> :
            (Option<T>, (T) -> Option<U>) -> Option<U>

        isSome :
            Option<T> -> Bool
    }

    traitImplementations = {
        ...
    }
}

```

```ph

(Person::name::)()

Person::name::=(_)(value)

System::"init new(_)"

System.print

enum Result<T, E> {
	isOk -> Bool
	isErr -> Bool

	::Ok(_ value: T) {
		isOk { true }
		isErr { false }
	}
	
	::Err(_ error: T) {
		isOk { false }
		isErr { true }
	}
}

Result::Err(TypeError("type mismatch"))
Result:Ok(())

Result::Err:: // the getter only
Result::Err::* // the whole family

Result::Err
Result::Err*

Result::*call::* // really want this to be the whole family

Result::Ok:: // getter only
```

