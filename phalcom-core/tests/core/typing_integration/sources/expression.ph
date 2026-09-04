enum Expression<F: Type -> Type, T> {
    @variant
    Pure<A>(_ value: A) -> Expression<F, A>

    @variant
    IntLiteral(_ value: Int) -> Expression<F, Int>

    @variant
    BoolLiteral(_ value: Bool) -> Expression<F, Bool>

    @variant
    Add(
        _ left: Expression<F, Int>,
        _ right: Expression<F, Int>
    ) -> Expression<F, Int>

    @variant
    If<A>(
        _ condition: Expression<F, Bool>,
        _ yes: Expression<F, A>,
        _ no: Expression<F, A>
    ) -> Expression<F, A>

    @variant
    Map<A, B>(
        _ source: Expression<F, A>,
        _ transform: (A) -> B
    ) -> Expression<F, B>

    @variant
    FlatMap<A, B>(
        _ source: Expression<F, A>,
        _ next: (A) -> Expression<F, B>
    ) -> Expression<F, B>

    @variant
    Apply<A, B>(
        _ function: Expression<F, (A) -> B>,
        _ argument: Expression<F, A>
    ) -> Expression<F, B>

    @variant
    Lift<A>(_ effect: F<A>) -> Expression<F, A>
}

class ExpressionEvaluation {
    @class
    eval<F: Type -> Type, T>(
        _ monad: Monad<F>,
        _ expression: Expression<F, T>
    ) -> F<T> {
        match expression {
            Expression::Pure(value) => monad.pure(value)
            Expression::IntLiteral(value) => monad.pure(value)
            Expression::BoolLiteral(value) => monad.pure(value)
            Expression::Add(left, right) => monad.flatMap(
                ExpressionEvaluation.eval(monad, left),
                |leftValue| {
                    monad.map(
                        ExpressionEvaluation.eval(monad, right),
                        |rightValue| { leftValue + rightValue }
                    )
                }
            )
            Expression::If(condition, yes, no) => monad.flatMap(
                ExpressionEvaluation.eval(monad, condition),
                |conditionValue| {
                    if (conditionValue) {
                        ExpressionEvaluation.eval(monad, yes)
                    } else {
                        ExpressionEvaluation.eval(monad, no)
                    }
                }
            )
            Expression::Map(source, transform) => monad.map(
                ExpressionEvaluation.eval(monad, source),
                transform
            )
            Expression::FlatMap(source, next) => monad.flatMap(
                ExpressionEvaluation.eval(monad, source),
                |value| { ExpressionEvaluation.eval(monad, next.call(value)) }
            )
            Expression::Apply(function, argument) => monad.flatMap(
                ExpressionEvaluation.eval(monad, function),
                |functionValue| {
                    monad.map(
                        ExpressionEvaluation.eval(monad, argument),
                        |argumentValue| { functionValue.call(argumentValue) }
                    )
                }
            )
            Expression::Lift(effect) => effect
        }
    }
}

class ExpressionMonad<F: Type -> Type> is Monad<<X> =>> Expression<F, X>> {
    map<A, B>(
        _ value: Expression<F, A>,
        _ transform: (A) -> B
    ) -> Expression<F, B> {
        Expression::Map(value, transform)
    }

    pure<A>(_ value: A) -> Expression<F, A> {
        Expression::Pure(value)
    }

    map2<A, B, C>(
        _ left: Expression<F, A>,
        _ right: Expression<F, B>,
        _ combine: (A, B) -> C
    ) -> Expression<F, C> {
        Expression::FlatMap(left, |leftValue| {
            Expression::Map(right, |rightValue| {
                combine.call(leftValue, rightValue)
            })
        })
    }

    flatMap<A, B>(
        _ value: Expression<F, A>,
        _ next: (A) -> Expression<F, B>
    ) -> Expression<F, B> {
        Expression::FlatMap(value, next)
    }
}

class StringEitherExpressionMonad is ExpressionMonad<
        <X> =>> Expression<
            <Y> =>> Either<String, Y>, X>
        > {

}
