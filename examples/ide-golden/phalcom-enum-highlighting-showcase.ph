// Enum / ADT / GADT syntax-highlighting showcase.
// This file intentionally contains competing enum syntaxes for visual comparison.

enum Direction {
    .Up
    .Down
    .Left
    .Right
}

enum Direction {
    case Up, Down, Left, Right
}

enum Direction {
    .North = 0
    .West = 1
    .East = 2
    .South = 3
}

enum Direction {
    case North = 0
    case West => 1
}

enum Option<T> {
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

// Ratified @variant-style ADT syntax.
enum Result<T, E> {
    case Ok(const _ value: T) {
        unwrap -> T {
            value
        }
    }

    case Err(const _ error: E)

    isOk -> Bool {
        match self {
            Ok(value) when value matches (x: _, y: _) => true,
            Err(_) => false,
        }
    }
}

// GADT constructor result types + constructor-local generics.
enum Expr<T> {
    @variant Int(const _ value: Int) -> Expr<Int>

    @variant Bool(const _ value: Bool) -> Expr<Bool>

    @variant Equal<U>(
        const _ left: Expr<U>,
        const _ right: Expr<U>
    ) -> Expr<Bool>

    @variant If<U>(
        const condition: Expr<Bool>,
        const then thenExpr: Expr<U>,
        const else elseExpr: Expr<U>
    ) -> Expr<U>
}
