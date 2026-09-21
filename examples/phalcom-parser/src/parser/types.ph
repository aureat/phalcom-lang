from parser.parser import Parser
from syntax.ast import TypeExpr, TypeArgument, TupleTypeElement

impl Parser {
    parseType -> TypeExprId {
        parseRefinedType()
    }

    parseRefinedType -> TypeExprId {
        let base = parseUnionType()

        if not atKeyword(Where) {
            return base
        }

        advance

        let predicate = parseRefinementPredicate()

        arena.addType(
            TypeExpr::Refined {
                base: base,
                predicate: predicate,
                span: Span(
                    typeSpan(base).start,
                    refinementSpan(predicate).end
                )
            }
        )
    }

    parseUnionType -> TypeExprId {
        var left = parseIntersectionType()

        while current.kind == Pipe {
            advance
            let right = parseIntersectionType()

            left =
                arena.addType(
                    TypeExpr::Union {
                        left: left,
                        right: right,
                        span: Span(
                            typeSpan(left).start,
                            typeSpan(right).end
                        )
                    }
                )
        }

        left
    }

    parseIntersectionType -> TypeExprId {
        var left = parsePostfixType()

        while current.kind == Ampersand {
            advance
            let right = parsePostfixType()

            left =
                arena.addType(
                    TypeExpr::Intersection {
                        left: left,
                        right: right,
                        span: Span(
                            typeSpan(left).start,
                            typeSpan(right).end
                        )
                    }
                )
        }

        left
    }

    parsePostfixType -> TypeExprId {
        var base = parseTypeAtom(current.kind)

        while true {
            match current.kind {
                Less => {
                    let start = typeSpan(base).start
                    let arguments = parseTypeArguments()

                    base =
                        arena.addType(
                            TypeExpr::Generic {
                                base: base,
                                arguments: arguments,
                                span: Span(
                                    start,
                                    previous.span.end
                                )
                            }
                        )
                }

                DoubleColon => {
                    advance
                    let member = expectIdentifier()

                    base =
                        arena.addType(
                            TypeExpr::Associated {
                                base: base,
                                member: member,
                                span: Span(
                                    typeSpan(base).start,
                                    member.span.end
                                )
                            }
                        )
                }

                _ => break
            }
        }

        base
    }

    parseTypeAtom(_ kind: Identifier) -> TypeExprId {
        let name = advance

        arena.addType(
            TypeExpr::Name {
                name: name,
                span: name.span
            }
        )
    }

    parseTypeAtom(_ kind: LeftParen) -> TypeExprId {
        parseTupleOrGroupedType()
    }

    parseTypeAtom(_ kind: Pipe) -> TypeExprId {
        parseCallableType()
    }

    parseTypeAtom(_ kind: TokenKind) -> TypeExprId {
        let span = current.span

        report(
            ExpectedType,
            span,
            "expected type"
        )

        advance

        arena.addType(
            TypeExpr::Error { span: span }
        )
    }

    parseTupleOrGroupedType -> TypeExprId {
        let open = expect(LeftParen)

        if current.kind == RightParen {
            let close = advance

            return arena.addType(
                TypeExpr::Tuple {
                    elements: List<TupleTypeElement>.new(),
                    span: Span(open.span.start, close.span.end)
                }
            )
        }

        let first = parseTupleTypeElement()

        if current.kind != Comma {
            match first {
                Positional(type) => {
                    expect(RightParen)
                    return type
                }

                Named { .. } => ()
            }
        }

        let elements = List<TupleTypeElement>.new()
        elements.push(first)

        while advanceIf(Comma).isSome {
            if current.kind == RightParen {
                break
            }

            elements.push(parseTupleTypeElement())
        }

        let close = expect(RightParen)

        arena.addType(
            TypeExpr::Tuple {
                elements: elements,
                span: Span(open.span.start, close.span.end)
            }
        )
    }

    parseTupleTypeElement -> TupleTypeElement {
        parseTupleTypeElement(current.kind)
    }

    parseTupleTypeElement(_ kind: Identifier)
        -> TupleTypeElement
    {
        if peek(1).kind != Colon {
            return Positional(parseType())
        }

        let name = advance
        advance

        Named {
            name: name,
            type: parseType()
        }
    }

    parseTupleTypeElement(_ kind: TokenKind)
        -> TupleTypeElement
    {
        Positional(parseType())
    }

    parseCallableType -> TypeExprId {
        let start = expect(Pipe).span.start
        let parameters = List<TypeExprId>.new()

        if current.kind != Pipe {
            while true {
                parameters.push(parseType())

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        expect(Pipe)
        expect(Arrow)

        let result = parseType()

        arena.addType(
            TypeExpr::Callable {
                parameters: parameters,
                result: result,
                span: Span(start, typeSpan(result).end)
            }
        )
    }

    parseTypeArguments -> List<TypeArgument> {
        let arguments = List<TypeArgument>.new()
        expect(Less)

        if current.kind != Greater {
            while true {
                arguments.push(
                    parseTypeArgument(current.kind)
                )

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        expect(Greater)

        arguments
    }

    parseTypeArgument(
        _ kind: Integer | Float | String | Character
    ) -> TypeArgument {
        Value(parseExpression())
    }

    parseTypeArgument(_ kind: TokenKind) -> TypeArgument {
        Type(parseType())
    }
}
