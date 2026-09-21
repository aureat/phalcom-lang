from parser.parser import Parser
from syntax.ast import RefinementExpr, RefinementUnaryOperator, RefinementBinaryOperator

// Controlled proposition grammar.
//
// Important: there is deliberately no call syntax in RefinementExpr.
// `self.length` may parse structurally as Member(Self, "length"), but
// semantic analysis decides whether the projection is an admissible field /
// compiler-known structural property rather than arbitrary executable dispatch.

impl Parser {
    parseRefinementPredicate -> RefinementExprId {
        parseRefinementOr()
    }

    parseRefinementOr -> RefinementExprId {
        var left = parseRefinementAnd()

        while atKeyword(Or) {
            advance
            let right = parseRefinementAnd()

            left =
                makeRefinementBinary(
                    left,
                    Or,
                    right
                )
        }

        left
    }

    parseRefinementAnd -> RefinementExprId {
        var left = parseRefinementComparison()

        while atKeyword(And) {
            advance
            let right = parseRefinementComparison()

            left =
                makeRefinementBinary(
                    left,
                    And,
                    right
                )
        }

        left
    }

    parseRefinementComparison -> RefinementExprId {
        var left = parseRefinementAdditive()

        while true {
            match refinementComparison(current.kind) {
                None =>
                    break

                Some(operator) => {
                    advance
                    let right = parseRefinementAdditive()

                    left =
                        makeRefinementBinary(
                            left,
                            operator,
                            right
                        )
                }
            }
        }

        left
    }

    refinementComparison(_ kind: EqualEqual)
        -> Option<RefinementBinaryOperator>
    {
        Some(Equal)
    }

    refinementComparison(_ kind: BangEqual)
        -> Option<RefinementBinaryOperator>
    {
        Some(NotEqual)
    }

    refinementComparison(_ kind: Less)
        -> Option<RefinementBinaryOperator>
    {
        Some(Less)
    }

    refinementComparison(_ kind: LessEqual)
        -> Option<RefinementBinaryOperator>
    {
        Some(LessEqual)
    }

    refinementComparison(_ kind: Greater)
        -> Option<RefinementBinaryOperator>
    {
        Some(Greater)
    }

    refinementComparison(_ kind: GreaterEqual)
        -> Option<RefinementBinaryOperator>
    {
        Some(GreaterEqual)
    }

    refinementComparison(_ kind: TokenKind)
        -> Option<RefinementBinaryOperator>
    {
        None
    }

    parseRefinementAdditive -> RefinementExprId {
        var left = parseRefinementMultiplicative()

        while true {
            match refinementAdditive(current.kind) {
                None =>
                    break

                Some(operator) => {
                    advance
                    let right =
                        parseRefinementMultiplicative()

                    left =
                        makeRefinementBinary(
                            left,
                            operator,
                            right
                        )
                }
            }
        }

        left
    }

    refinementAdditive(_ kind: Plus)
        -> Option<RefinementBinaryOperator>
    {
        Some(Add)
    }

    refinementAdditive(_ kind: Minus)
        -> Option<RefinementBinaryOperator>
    {
        Some(Subtract)
    }

    refinementAdditive(_ kind: TokenKind)
        -> Option<RefinementBinaryOperator>
    {
        None
    }

    parseRefinementMultiplicative -> RefinementExprId {
        var left = parseRefinementUnary()

        while true {
            match refinementMultiplicative(current.kind) {
                None =>
                    break

                Some(operator) => {
                    advance
                    let right =
                        parseRefinementUnary()

                    left =
                        makeRefinementBinary(
                            left,
                            operator,
                            right
                        )
                }
            }
        }

        left
    }

    refinementMultiplicative(_ kind: Star)
        -> Option<RefinementBinaryOperator>
    {
        Some(Multiply)
    }

    refinementMultiplicative(_ kind: Slash)
        -> Option<RefinementBinaryOperator>
    {
        Some(Divide)
    }

    refinementMultiplicative(_ kind: Percent)
        -> Option<RefinementBinaryOperator>
    {
        Some(Remainder)
    }

    refinementMultiplicative(_ kind: TokenKind)
        -> Option<RefinementBinaryOperator>
    {
        None
    }

    parseRefinementUnary -> RefinementExprId {
        parseRefinementUnary(current.kind)
    }

    parseRefinementUnary(_ kind: Plus) -> RefinementExprId {
        parseRefinementUnaryOperator(Positive)
    }

    parseRefinementUnary(_ kind: Minus) -> RefinementExprId {
        parseRefinementUnaryOperator(Negative)
    }

    parseRefinementUnary(
        _ kind: Bang | Keyword(Not)
    ) -> RefinementExprId {
        parseRefinementUnaryOperator(Not)
    }

    parseRefinementUnary(_ kind: TokenKind) -> RefinementExprId {
        parseRefinementPrimary(current.kind)
    }

    parseRefinementUnaryOperator(
        _ operator: RefinementUnaryOperator
    ) -> RefinementExprId {
        let start = advance.span.start
        let operand = parseRefinementUnary()

        arena.addRefinement(
            RefinementExpr::Unary {
                operator: operator,
                operand: operand,
                span: Span(
                    start,
                    refinementSpan(operand).end
                )
            }
        )
    }

    parseRefinementPrimary(_ kind: Integer)
        -> RefinementExprId
    {
        let token = advance

        parseRefinementMemberTail(
            arena.addRefinement(
                RefinementExpr::Integer {
                    token: token,
                    span: token.span
                }
            )
        )
    }

    parseRefinementPrimary(_ kind: Float)
        -> RefinementExprId
    {
        let token = advance

        parseRefinementMemberTail(
            arena.addRefinement(
                RefinementExpr::Float {
                    token: token,
                    span: token.span
                }
            )
        )
    }

    parseRefinementPrimary(_ kind: Keyword(True))
        -> RefinementExprId
    {
        let token = advance

        parseRefinementMemberTail(
            arena.addRefinement(
                RefinementExpr::Bool {
                    value: true,
                    span: token.span
                }
            )
        )
    }

    parseRefinementPrimary(_ kind: Keyword(False))
        -> RefinementExprId
    {
        let token = advance

        parseRefinementMemberTail(
            arena.addRefinement(
                RefinementExpr::Bool {
                    value: false,
                    span: token.span
                }
            )
        )
    }

    parseRefinementPrimary(_ kind: Keyword(Self))
        -> RefinementExprId
    {
        let token = advance

        parseRefinementMemberTail(
            arena.addRefinement(
                RefinementExpr::Self {
                    span: token.span
                }
            )
        )
    }

    parseRefinementPrimary(_ kind: Identifier)
        -> RefinementExprId
    {
        let name = advance

        parseRefinementMemberTail(
            arena.addRefinement(
                RefinementExpr::Name {
                    name: name,
                    span: name.span
                }
            )
        )
    }

    parseRefinementPrimary(_ kind: LeftParen)
        -> RefinementExprId
    {
        advance
        let nested = parseRefinementPredicate()
        expect(RightParen)
        parseRefinementMemberTail(nested)
    }

    parseRefinementPrimary(_ kind: TokenKind)
        -> RefinementExprId
    {
        let span = current.span

        report(
            InvalidRefinementExpression,
            span,
            "expected controlled refinement operand"
        )

        advance

        arena.addRefinement(
            RefinementExpr::Error { span: span }
        )
    }

    parseRefinementMemberTail(
        _ base: RefinementExprId
    ) -> RefinementExprId {
        var expression = base

        while current.kind == Dot {
            advance

            let member = expectIdentifier()

            expression =
                arena.addRefinement(
                    RefinementExpr::Member {
                        receiver: expression,
                        member: member,
                        span: Span(
                            refinementSpan(expression).start,
                            member.span.end
                        )
                    }
                )
        }

        expression
    }

    makeRefinementBinary(
        _ left: RefinementExprId,
        _ operator: RefinementBinaryOperator,
        _ right: RefinementExprId
    ) -> RefinementExprId {
        arena.addRefinement(
            RefinementExpr::Binary {
                left: left,
                operator: operator,
                right: right,
                span: Span(
                    refinementSpan(left).start,
                    refinementSpan(right).end
                )
            }
        )
    }
}
