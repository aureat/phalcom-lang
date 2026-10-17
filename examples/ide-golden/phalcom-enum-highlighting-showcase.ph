// Closed-sum / GADT syntax-highlighting showcase.
// This file intentionally contains competing data spellings for visual comparison.

data Direction {
    .Up
    .Down
    .Left
    .Right
}

data Direction {
    case Up, Down, Left, Right
}

data Direction {
    .North = 0
    .West = 1
    .East = 2
    .South = 3
}

data Direction {
    case North = 0
    case West => 1
}

data Option<T> {
    .Some<T>(_ value: T)
    .None

    toString {
        match self {
            Some(value) when value matches (x: _, y: _) => "Some(\(value))",
            Some((x: _, y: _)) => "pair",
            Option.Some(tuple) when tuple matches (x: _, y: _) => "tuple",
            None => "None",
        }
    }
}

// Bare-case ADT syntax.
data Result<T, E> {
    Ok(const _ value: T) {
        unwrap -> T {
            value
        }
    }

    Err(const _ error: E)

    isOk -> Bool {
        match self {
            Ok(value) when value matches (x: _, y: _) => true,
            Err(_) => false,
        }
    }
}

// GADT constructor result types + constructor-local generics.
data Expr<T> {
    Int(const _ value: Int) -> Expr<Int>

    Bool(const _ value: Bool) -> Expr<Bool>

    Equal<U>(
        const _ left: Expr<U>,
        const _ right: Expr<U>
    ) -> Expr<Bool>

    If<U>(
        const condition: Expr<Bool>,
        const then thenExpr: Expr<U>,
        const else elseExpr: Expr<U>
    ) -> Expr<U>
}
