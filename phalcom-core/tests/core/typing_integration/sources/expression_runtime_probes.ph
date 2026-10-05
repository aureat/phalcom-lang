let mut expressionEvaluationMonad = StringEitherMonad.new()
let mut expressionConstructionMonad = StringEitherExpressionMonad.new()

let mut pureExpression: Expression<<X> =>> Either<String, X>, Int> = Expression::Pure(41)
let mut pureResult = ExpressionEvaluation.eval(expressionEvaluationMonad, pureExpression)
let mut expressionPureValue = pureResult.fold(
    left: |error| { -1 },
    right: |value| { value }
)

let mut addExpression: Expression<<X> =>> Either<String, X>, Int> = Expression::Add(
    Expression::IntLiteral(20),
    Expression::IntLiteral(22)
)
let mut addResult = ExpressionEvaluation.eval(expressionEvaluationMonad, addExpression)
let mut expressionAddValue = addResult.fold(
    left: |error| { -1 },
    right: |value| { value }
)

let mut mapExpression: Expression<<X> =>> Either<String, X>, Bool> = Expression::Map(
    Expression::IntLiteral(41),
    |value| { value > 0 }
)
let mut mapResult = ExpressionEvaluation.eval(expressionEvaluationMonad, mapExpression)
let mut expressionMapValue = mapResult.fold(
    left: |error| { false },
    right: |value| { value }
)

let mut applyFunction: Expression<<X> =>> Either<String, X>, (Int) -> Bool> = Expression::Pure(|value| {
    value > 0
})
let mut applyArgument: Expression<<X> =>> Either<String, X>, Int> = Expression::IntLiteral(41)
let mut applyExpression: Expression<<X> =>> Either<String, X>, Bool> = Expression::Apply(applyFunction, applyArgument)
let mut applyResult = ExpressionEvaluation.eval(expressionEvaluationMonad, applyExpression)
let mut expressionApplyValue = applyResult.fold(
    left: |error| { false },
    right: |value| { value }
)

let mut liftSuccess: Either<String, Bool> = Either::Right(true)
let mut liftExpression: Expression<<X> =>> Either<String, X>, Bool> = Expression::Lift(liftSuccess)
let mut liftResult = ExpressionEvaluation.eval(expressionEvaluationMonad, liftExpression)
let mut expressionLiftValue = liftResult.fold(
    left: |error| { false },
    right: |value| { value }
)

let mut failed: Either<String, Int> = Either::Left("boom")
let mut failedExpression: Expression<<X> =>> Either<String, X>, Int> = Expression::Lift(failed)
let mut failedResult = ExpressionEvaluation.eval(expressionEvaluationMonad, failedExpression)
let mut failureContinuationCalls = 0
let mut afterFailure = expressionEvaluationMonad.flatMap(failedResult, |value| {
    failureContinuationCalls = failureContinuationCalls + 1
    let mut continued: Either<String, Bool> = Either::Right(true)
    continued
})
let mut expressionFailurePreserved = afterFailure.fold(
    left: |error| { error == "boom" },
    right: |value| { false }
)
let mut expressionFailureShortCircuited = failureContinuationCalls == 0

let mut expressionList = MonadAlgorithms.traverse(
    expressionConstructionMonad,
    [1, 2, 3],
    |value| {
        Expression::Map(
            Expression::IntLiteral(value),
            |item| { item + 10 }
        )
    }
)
let mut evaluatedExpressionList = ExpressionEvaluation.eval(expressionEvaluationMonad, expressionList)
let mut expressionTraverseValue = evaluatedExpressionList.fold(
    left: |error| { false },
    right: |values| {
        values.size == 3 and values[0] == 11 and values[1] == 12 and values[2] == 13
    }
)

let mut expressionRuntimeAll = expressionPureValue == 41 and expressionAddValue == 42 and expressionMapValue and expressionApplyValue and expressionLiftValue and expressionFailurePreserved and expressionFailureShortCircuited and expressionTraverseValue
