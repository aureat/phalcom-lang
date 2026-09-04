let expressionEvaluationMonad = StringEitherMonad.new()
let expressionConstructionMonad = StringEitherExpressionMonad.new()

let pureExpression: Expression<<X> =>> Either<String, X>, Int> = Expression::Pure(41)
let pureResult = ExpressionEvaluation.eval(expressionEvaluationMonad, pureExpression)
let expressionPureValue = pureResult.fold(
    left: |error| { -1 },
    right: |value| { value }
)

let addExpression: Expression<<X> =>> Either<String, X>, Int> = Expression::Add(
    Expression::IntLiteral(20),
    Expression::IntLiteral(22)
)
let addResult = ExpressionEvaluation.eval(expressionEvaluationMonad, addExpression)
let expressionAddValue = addResult.fold(
    left: |error| { -1 },
    right: |value| { value }
)

let mapExpression: Expression<<X> =>> Either<String, X>, Bool> = Expression::Map(
    Expression::IntLiteral(41),
    |value| { value > 0 }
)
let mapResult = ExpressionEvaluation.eval(expressionEvaluationMonad, mapExpression)
let expressionMapValue = mapResult.fold(
    left: |error| { false },
    right: |value| { value }
)

let applyFunction: Expression<<X> =>> Either<String, X>, (Int) -> Bool> = Expression::Pure(|value| {
    value > 0
})
let applyArgument: Expression<<X> =>> Either<String, X>, Int> = Expression::IntLiteral(41)
let applyExpression: Expression<<X> =>> Either<String, X>, Bool> = Expression::Apply(applyFunction, applyArgument)
let applyResult = ExpressionEvaluation.eval(expressionEvaluationMonad, applyExpression)
let expressionApplyValue = applyResult.fold(
    left: |error| { false },
    right: |value| { value }
)

let liftSuccess: Either<String, Bool> = Either::Right(true)
let liftExpression: Expression<<X> =>> Either<String, X>, Bool> = Expression::Lift(liftSuccess)
let liftResult = ExpressionEvaluation.eval(expressionEvaluationMonad, liftExpression)
let expressionLiftValue = liftResult.fold(
    left: |error| { false },
    right: |value| { value }
)

let failed: Either<String, Int> = Either::Left("boom")
let failedExpression: Expression<<X> =>> Either<String, X>, Int> = Expression::Lift(failed)
let failedResult = ExpressionEvaluation.eval(expressionEvaluationMonad, failedExpression)
let failureContinuationCalls = 0
let afterFailure = expressionEvaluationMonad.flatMap(failedResult, |value| {
    failureContinuationCalls = failureContinuationCalls + 1
    let continued: Either<String, Bool> = Either::Right(true)
    continued
})
let expressionFailurePreserved = afterFailure.fold(
    left: |error| { error == "boom" },
    right: |value| { false }
)
let expressionFailureShortCircuited = failureContinuationCalls == 0

let expressionList = MonadAlgorithms.traverse(
    expressionConstructionMonad,
    [1, 2, 3],
    |value| {
        Expression::Map(
            Expression::IntLiteral(value),
            |item| { item + 10 }
        )
    }
)
let evaluatedExpressionList = ExpressionEvaluation.eval(expressionEvaluationMonad, expressionList)
let expressionTraverseValue = evaluatedExpressionList.fold(
    left: |error| { false },
    right: |values| {
        values.size == 3 and values[0] == 11 and values[1] == 12 and values[2] == 13
    }
)

let expressionRuntimeAll = expressionPureValue == 41 and expressionAddValue == 42 and expressionMapValue and expressionApplyValue and expressionLiftValue and expressionFailurePreserved and expressionFailureShortCircuited and expressionTraverseValue
