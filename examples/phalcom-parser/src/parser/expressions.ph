from core.domains import BindingPower
from parser.parser import Parser
from syntax.ast import Expr, UnaryOperator, BinaryOperator, CallArgument, MatchArm, ClosureParameter

data InfixRule(
    operator: BinaryOperator,
    left: BindingPower,
    right: BindingPower
)


impl Parser {
    parseExpression -> ExprId {
        parseExpressionWithPower(0)
    }

    parseExpressionWithPower(
        _ minimum: BindingPower
    ) -> ExprId {
        var left = parsePrefix(current.kind)

        while true {
            left = parsePostfix(left, current.kind)

            match infixRule(current.kind) {
                None =>
                    break

                Some(rule) if rule.left < minimum =>
                    break

                Some(rule) => {
                    advance

                    let right =
                        parseExpressionWithPower(rule.right)

                    left =
                        arena.addExpr(
                            Expr::Binary {
                                left: left,
                                operator: rule.operator,
                                right: right,
                                span: Span(
                                    exprSpan(left).start,
                                    exprSpan(right).end
                                )
                            }
                        )
                }
            }
        }

        left
    }

    // Prefix family ------------------------------------------------------------

    parsePrefix(_ kind: Integer) -> ExprId {
        let token = advance

        arena.addExpr(
            Expr::Integer {
                token: token,
                span: token.span
            }
        )
    }

    parsePrefix(_ kind: Float) -> ExprId {
        let token = advance

        arena.addExpr(
            Expr::Float {
                token: token,
                span: token.span
            }
        )
    }

    parsePrefix(_ kind: String) -> ExprId {
        let token = advance

        arena.addExpr(
            Expr::String {
                token: token,
                span: token.span
            }
        )
    }

    parsePrefix(_ kind: Character) -> ExprId {
        let token = advance

        arena.addExpr(
            Expr::Character {
                token: token,
                span: token.span
            }
        )
    }

    parsePrefix(_ kind: Keyword(True)) -> ExprId {
        let token = advance

        arena.addExpr(
            Expr::Bool {
                value: true,
                span: token.span
            }
        )
    }

    parsePrefix(_ kind: Keyword(False)) -> ExprId {
        let token = advance

        arena.addExpr(
            Expr::Bool {
                value: false,
                span: token.span
            }
        )
    }

    parsePrefix(_ kind: Identifier) -> ExprId {
        let token = advance

        arena.addExpr(
            Expr::Name {
                name: token,
                span: token.span
            }
        )
    }

    parsePrefix(_ kind: LeftParen) -> ExprId {
        parseParenOrTupleExpression()
    }

    parsePrefix(_ kind: LeftBracket) -> ExprId {
        parseListExpression()
    }

    parsePrefix(_ kind: LeftBrace) -> ExprId {
        parseBlockExpression()
    }

    parsePrefix(_ kind: Keyword(If)) -> ExprId {
        parseIfExpression()
    }

    parsePrefix(_ kind: Keyword(Match)) -> ExprId {
        parseMatchExpression()
    }

    parsePrefix(_ kind: Pipe) -> ExprId {
        parseClosureExpression()
    }

    parsePrefix(
        _ kind: Plus | Minus | Bang | Keyword(Not) | Ampersand
    ) -> ExprId {
        parseUnaryExpression()
    }

    parsePrefix(_ kind: TokenKind) -> ExprId {
        let span = current.span

        report(
            ExpectedExpression,
            span,
            "expected expression"
        )

        advance

        arena.addExpr(
            Expr::Error { span: span }
        )
    }

    // Postfix family -----------------------------------------------------------

    parsePostfix(
        _ receiver: ExprId,
        _ kind: LeftParen
    ) -> ExprId {
        parseCallExpression(receiver)
    }

    parsePostfix(
        _ receiver: ExprId,
        _ kind: Dot
    ) -> ExprId {
        parseMemberExpression(receiver)
    }

    parsePostfix(
        _ receiver: ExprId,
        _ kind: DoubleColon
    ) -> ExprId {
        parseAssociatedExpression(receiver)
    }

    parsePostfix(
        _ receiver: ExprId,
        _ kind: TokenKind
    ) -> ExprId {
        receiver
    }

    // Infix family -------------------------------------------------------------

    infixRule(_ kind: Keyword(Or)) -> Option<InfixRule> {
        Some(InfixRule(LogicalOr, 10, 11))
    }

    infixRule(_ kind: Keyword(And)) -> Option<InfixRule> {
        Some(InfixRule(LogicalAnd, 20, 21))
    }

    infixRule(_ kind: EqualEqual) -> Option<InfixRule> {
        Some(InfixRule(Equal, 30, 31))
    }

    infixRule(_ kind: BangEqual) -> Option<InfixRule> {
        Some(InfixRule(NotEqual, 30, 31))
    }

    infixRule(_ kind: Less) -> Option<InfixRule> {
        Some(InfixRule(Less, 40, 41))
    }

    infixRule(_ kind: LessEqual) -> Option<InfixRule> {
        Some(InfixRule(LessEqual, 40, 41))
    }

    infixRule(_ kind: Greater) -> Option<InfixRule> {
        Some(InfixRule(Greater, 40, 41))
    }

    infixRule(_ kind: GreaterEqual) -> Option<InfixRule> {
        Some(InfixRule(GreaterEqual, 40, 41))
    }

    infixRule(_ kind: Keyword(Is)) -> Option<InfixRule> {
        Some(InfixRule(Is, 40, 41))
    }

    infixRule(_ kind: Keyword(As)) -> Option<InfixRule> {
        Some(InfixRule(As, 40, 41))
    }

    infixRule(_ kind: Plus) -> Option<InfixRule> {
        Some(InfixRule(Add, 50, 51))
    }

    infixRule(_ kind: Minus) -> Option<InfixRule> {
        Some(InfixRule(Subtract, 50, 51))
    }

    infixRule(_ kind: Star) -> Option<InfixRule> {
        Some(InfixRule(Multiply, 60, 61))
    }

    infixRule(_ kind: Slash) -> Option<InfixRule> {
        Some(InfixRule(Divide, 60, 61))
    }

    infixRule(_ kind: Percent) -> Option<InfixRule> {
        Some(InfixRule(Remainder, 60, 61))
    }

    infixRule(_ kind: TokenKind) -> Option<InfixRule> {
        None
    }

    // Concrete expression forms ----------------------------------------------

    parseParenOrTupleExpression -> ExprId {
        let open = expect(LeftParen)

        if current.kind == RightParen {
            let close = advance

            return arena.addExpr(
                Expr::Tuple {
                    elements: List<ExprId>.new(),
                    span: Span(open.span.start, close.span.end)
                }
            )
        }

        let first = parseExpression()

        if current.kind != Comma {
            expect(RightParen)
            return first
        }

        let elements = List<ExprId>.new()
        elements.push(first)

        while advanceIf(Comma).isSome {
            if current.kind == RightParen {
                break
            }

            elements.push(parseExpression())
        }

        let close = expect(RightParen)

        arena.addExpr(
            Expr::Tuple {
                elements: elements,
                span: Span(open.span.start, close.span.end)
            }
        )
    }

    parseListExpression -> ExprId {
        let open = expect(LeftBracket)
        let elements = List<ExprId>.new()

        if current.kind != RightBracket {
            while true {
                elements.push(parseExpression())

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        let close = expect(RightBracket)

        arena.addExpr(
            Expr::List {
                elements: elements,
                span: Span(open.span.start, close.span.end)
            }
        )
    }

    parseUnaryExpression -> ExprId {
        let token = advance
        let operator = unaryOperator(token.kind)
        let operand = parseExpressionWithPower(90)

        arena.addExpr(
            Expr::Unary {
                operator: operator,
                operand: operand,
                span: Span(
                    token.span.start,
                    exprSpan(operand).end
                )
            }
        )
    }

    unaryOperator(_ kind: Plus) -> UnaryOperator { Positive }
    unaryOperator(_ kind: Minus) -> UnaryOperator { Negative }
    unaryOperator(_ kind: Bang | Keyword(Not)) -> UnaryOperator { LogicalNot }
    unaryOperator(_ kind: Ampersand) -> UnaryOperator { Reference }

    parseCallExpression(_ callee: ExprId) -> ExprId {
        let arguments = List<CallArgument>.new()
        expect(LeftParen)

        if current.kind != RightParen {
            while true {
                let label =
                    if current.kind == Identifier
                        and peek(1).kind == Colon
                    {
                        let label = advance
                        advance
                        Some(label)
                    } else {
                        None
                    }

                arguments.push(
                    CallArgument(
                        label,
                        parseExpression()
                    )
                )

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        let close = expect(RightParen)

        arena.addExpr(
            Expr::Call {
                callee: callee,
                arguments: arguments,
                span: Span(
                    exprSpan(callee).start,
                    close.span.end
                )
            }
        )
    }

    parseMemberExpression(_ receiver: ExprId) -> ExprId {
        expect(Dot)
        let member = expectIdentifier()

        arena.addExpr(
            Expr::Member {
                receiver: receiver,
                member: member,
                span: Span(
                    exprSpan(receiver).start,
                    member.span.end
                )
            }
        )
    }

    parseAssociatedExpression(_ receiver: ExprId) -> ExprId {
        expect(DoubleColon)
        let member = expectIdentifier()

        arena.addExpr(
            Expr::Associated {
                receiver: receiver,
                member: member,
                span: Span(
                    exprSpan(receiver).start,
                    member.span.end
                )
            }
        )
    }

    parseIfExpression -> ExprId {
        let start = expect(Keyword(If)).span.start
        let condition = parseExpression()
        let thenBranch = parseBlockExpression()

        let elseBranch =
            match advanceIfKeyword(Else) {
                None => None
                Some(_) if atKeyword(If) =>
                    Some(parseIfExpression())
                Some(_) =>
                    Some(parseBlockExpression())
            }

        arena.addExpr(
            Expr::If {
                condition: condition,
                thenBranch: thenBranch,
                elseBranch: elseBranch,
                span: Span(
                    start,
                    match elseBranch {
                        Some(branch) => exprSpan(branch).end
                        None => exprSpan(thenBranch).end
                    }
                )
            }
        )
    }

    parseMatchExpression -> ExprId {
        let start = expect(Keyword(Match)).span.start
        let subject = parseExpression()

        expect(LeftBrace)

        let arms = List<MatchArm>.new()
        skipSeparators()

        while current.kind != RightBrace
            and current.kind != Eof
        {
            let armStart = current.span.start
            let pattern = parsePattern()

            let guard =
                match advanceIfKeyword(If) {
                    Some(_) => Some(parseExpression())
                    None => None
                }

            expect(FatArrow)

            let body = parseExpression()

            arms.push(
                MatchArm(
                    pattern,
                    guard,
                    body,
                    Span(armStart, exprSpan(body).end)
                )
            )

            advanceIf(Comma)
            skipSeparators()
        }

        let close = expect(RightBrace)

        arena.addExpr(
            Expr::Match {
                subject: subject,
                arms: arms,
                span: Span(start, close.span.end)
            }
        )
    }

    parseClosureExpression -> ExprId {
        let start = expect(Pipe).span.start
        let parameters = List<ClosureParameter>.new()

        if current.kind != Pipe {
            while true {
                let name = expectIdentifier()

                let type =
                    match advanceIf(Colon) {
                        Some(_) => Some(parseType())
                        None => None
                    }

                parameters.push(
                    ClosureParameter(name, type)
                )

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        expect(Pipe)

        let body =
            if current.kind == LeftBrace {
                parseBlockExpression()
            } else {
                parseExpression()
            }

        arena.addExpr(
            Expr::Closure {
                parameters: parameters,
                body: body,
                span: Span(start, exprSpan(body).end)
            }
        )
    }

    // Expression-start family -------------------------------------------------

    beginsExpression(_ kind: Identifier) -> Bool { true }
    beginsExpression(_ kind: Integer) -> Bool { true }
    beginsExpression(_ kind: Float) -> Bool { true }
    beginsExpression(_ kind: String) -> Bool { true }
    beginsExpression(_ kind: Character) -> Bool { true }
    beginsExpression(_ kind: LeftParen) -> Bool { true }
    beginsExpression(_ kind: LeftBracket) -> Bool { true }
    beginsExpression(_ kind: LeftBrace) -> Bool { true }
    beginsExpression(_ kind: Pipe) -> Bool { true }
    beginsExpression(_ kind: Plus | Minus | Bang | Ampersand) -> Bool { true }
    beginsExpression(_ kind: Keyword(True | False | If | Match | Not)) -> Bool { true }
    beginsExpression(_ kind: TokenKind) -> Bool { false }
}
