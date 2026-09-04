class ExpressionSemanticProbe {
    @class
    intEvaluation(_ monad: StringEitherMonad) {
        let expression: Expression<<X> =>> Either<String, X>, Int> = Expression::IntLiteral(41)
        let result = ExpressionEvaluation.eval(monad, expression)
    }

    @class
    boolEvaluation(_ monad: StringEitherMonad) {
        let expression: Expression<<X> =>> Either<String, X>, Bool> = Expression::BoolLiteral(true)
        let result = ExpressionEvaluation.eval(monad, expression)
    }

    @class
    addEvaluation(_ monad: StringEitherMonad) {
        let expression: Expression<<X> =>> Either<String, X>, Int> = Expression::Add(
            Expression::IntLiteral(20),
            Expression::IntLiteral(22)
        )
        let result = ExpressionEvaluation.eval(monad, expression)
    }

    @class
    ifEvaluation(_ monad: StringEitherMonad) {
        let expression: Expression<<X> =>> Either<String, X>, Int> = Expression::If(
            Expression::BoolLiteral(true),
            Expression::IntLiteral(10),
            Expression::IntLiteral(20)
        )
        let result = ExpressionEvaluation.eval(monad, expression)
    }

    @class
    mapEvaluation(_ monad: StringEitherMonad) {
        let expression: Expression<<X> =>> Either<String, X>, Bool> = Expression::Map(
            Expression::IntLiteral(41),
            |value| { value > 0 }
        )
        let result = ExpressionEvaluation.eval(monad, expression)
    }

    @class
    flatMapEvaluation(_ monad: StringEitherMonad) {
        let expression: Expression<<X> =>> Either<String, X>, Bool> = Expression::FlatMap(
            Expression::IntLiteral(41),
            |value| { Expression::BoolLiteral(value > 0) }
        )
        let result = ExpressionEvaluation.eval(monad, expression)
    }

    @class
    applyEvaluation(_ monad: StringEitherMonad) {
        let function: Expression<<X> =>> Either<String, X>, (Int) -> Bool> = Expression::Pure(|value| {
            value > 0
        })
        let argument: Expression<<X> =>> Either<String, X>, Int> = Expression::IntLiteral(41)
        let expression: Expression<<X> =>> Either<String, X>, Bool> = Expression::Apply(function, argument)
        let result = ExpressionEvaluation.eval(monad, expression)
    }

    @class
    liftEvaluation(_ monad: StringEitherMonad) {
        let effect: Either<String, Bool> = Either::Right(true)
        let expression: Expression<<X> =>> Either<String, X>, Bool> = Expression::Lift(effect)
        let result = ExpressionEvaluation.eval(monad, expression)
    }
}

class ExpressionMonadSemanticProbe {
    @class
    constructorOperations(
        _ expressionMonad: StringEitherExpressionMonad,
        _ source: Expression<<X> =>> Either<String, X>, Int>
    ) {
        let pure = expressionMonad.pure(41)
        let mapped = expressionMonad.map(source, |value| { value > 0 })
        let bound = MonadAlgorithms.bind(
            expressionMonad,
            source,
            |value| { Expression::Pure(value > 0) }
        )
    }

    @class
    traverseConstruction(
        _ expressionMonad: StringEitherExpressionMonad,
        _ values: List<Int>
    ) {
        let expression: Expression<<X> =>> Either<String, X>, List<Int>> = MonadAlgorithms.traverse(
            expressionMonad,
            values,
            |value| {
                Expression::Map(
                    Expression::IntLiteral(value),
                    |item| { item + 10 }
                )
            }
        )
    }
}

class ExpressionIntegrationProbe {
    @class
    traverseAndEvaluate(
        _ expressionMonad: StringEitherExpressionMonad,
        _ evaluationMonad: StringEitherMonad,
        _ values: List<Int>
    ) {
        let expression: Expression<<X> =>> Either<String, X>, List<Int>> = MonadAlgorithms.traverse(
            expressionMonad,
            values,
            |value| {
                Expression::Map(
                    Expression::IntLiteral(value),
                    |item| { item + 10 }
                )
            }
        )
        let result = ExpressionEvaluation.eval(evaluationMonad, expression)
    }
}
