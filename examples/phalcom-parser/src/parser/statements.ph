from parser.parser import Parser
from syntax.ast import Stmt

impl Parser {
    parseStatement -> StmtId {
        parseStatement(current.kind)
    }

    parseStatement(_ kind: Keyword(Let)) -> StmtId {
        parseLetStatement(mutable: false)
    }

    parseStatement(_ kind: Keyword(Var)) -> StmtId {
        parseLetStatement(mutable: true)
    }

    parseStatement(_ kind: Keyword(Return)) -> StmtId {
        parseReturnStatement()
    }

    parseStatement(_ kind: Keyword(Break)) -> StmtId {
        parseBreakStatement()
    }

    parseStatement(_ kind: Keyword(Continue)) -> StmtId {
        let token = advance

        arena.addStmt(
            Stmt::Continue {
                span: token.span
            }
        )
    }

    parseStatement(_ kind: TokenKind) -> StmtId {
        let expression = parseExpression()

        arena.addStmt(
            Stmt::Expr {
                expression: expression,
                span: exprSpan(expression)
            }
        )
    }

    parseLetStatement(mutable: Bool) -> StmtId {
        let start = advance.span.start
        let name = expectIdentifier()

        let type =
            match advanceIf(Colon) {
                Some(_) => Some(parseType())
                None => None
            }

        let value =
            match advanceIf(Equal) {
                Some(_) => Some(parseExpression())
                None => None
            }

        let end =
            match value {
                Some(expr) => exprSpan(expr).end
                None => previous.span.end
            }

        arena.addStmt(
            Stmt::Let {
                mutable: mutable,
                name: name,
                type: type,
                value: value,
                span: Span(start, end)
            }
        )
    }

    parseReturnStatement -> StmtId {
        let start = advance.span.start
        let value =
            if beginsExpression(current.kind) {
                Some(parseExpression())
            } else {
                None
            }

        arena.addStmt(
            Stmt::Return {
                value: value,
                span: Span(
                    start,
                    match value {
                        Some(expr) => exprSpan(expr).end
                        None => previous.span.end
                    }
                )
            }
        )
    }

    parseBreakStatement -> StmtId {
        let start = advance.span.start
        let value =
            if beginsExpression(current.kind) {
                Some(parseExpression())
            } else {
                None
            }

        arena.addStmt(
            Stmt::Break {
                value: value,
                span: Span(
                    start,
                    match value {
                        Some(expr) => exprSpan(expr).end
                        None => previous.span.end
                    }
                )
            }
        )
    }

    parseBlockExpression -> ExprId {
        let open = expect(LeftBrace)
        let statements = List<StmtId>.new()
        var trailing: Option<ExprId> = None

        skipSeparators()

        while current.kind != RightBrace
            and current.kind != Eof
        {
            let statement = parseStatement()

            match arena.stmt(statement) {
                Stmt::Expr { expression, .. }
                    if current.kind == RightBrace =>
                {
                    trailing = Some(expression)
                }

                _ =>
                    statements.push(statement)
            }

            skipSeparators()
        }

        let close = expect(RightBrace)

        arena.addExpr(
            Expr::Block {
                statements: statements,
                trailing: trailing,
                span: Span(open.span.start, close.span.end)
            }
        )
    }
}
