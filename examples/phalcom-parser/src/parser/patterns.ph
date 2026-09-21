from parser.parser import Parser
from syntax.ast import Pattern

impl Parser {
    parsePattern -> PatternId {
        parseOrPattern()
    }

    parseOrPattern -> PatternId {
        let first = parseRangePattern()

        if current.kind != Pipe {
            return first
        }

        let alternatives = List<PatternId>.new()
        alternatives.push(first)

        while current.kind == Pipe {
            advance
            alternatives.push(parseRangePattern())
        }

        arena.addPattern(
            Pattern::Or {
                alternatives: alternatives,
                span: Span(
                    patternSpan(alternatives.first).start,
                    patternSpan(alternatives.last).end
                )
            }
        )
    }

    parseRangePattern -> PatternId {
        let startPattern = parsePatternAtom(current.kind)
        parseRangeTail(startPattern, current.kind)
    }

    parseRangeTail(
        _ start: PatternId,
        _ kind: DotDot
    ) -> PatternId {
        advance
        let end = parsePatternAtom(current.kind)

        arena.addPattern(
            Pattern::Range {
                start: start,
                end: end,
                inclusive: false,
                span: Span(
                    patternSpan(start).start,
                    patternSpan(end).end
                )
            }
        )
    }

    parseRangeTail(
        _ start: PatternId,
        _ kind: DotDotEqual
    ) -> PatternId {
        advance
        let end = parsePatternAtom(current.kind)

        arena.addPattern(
            Pattern::Range {
                start: start,
                end: end,
                inclusive: true,
                span: Span(
                    patternSpan(start).start,
                    patternSpan(end).end
                )
            }
        )
    }

    parseRangeTail(
        _ start: PatternId,
        _ kind: TokenKind
    ) -> PatternId {
        start
    }

    parsePatternAtom(_ kind: Identifier) -> PatternId {
        let name = advance

        if tokens.text(name) == "_" {
            return arena.addPattern(
                Pattern::Wildcard {
                    span: name.span
                }
            )
        }

        parseIdentifierPattern(name, current.kind)
    }

    parseIdentifierPattern(
        _ name: Token,
        _ kind: LeftParen
    ) -> PatternId {
        let start = name.span.start
        advance

        let payload = List<PatternId>.new()

        if current.kind != RightParen {
            while true {
                payload.push(parsePattern())

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        let close = expect(RightParen)

        arena.addPattern(
            Pattern::Variant {
                name: name,
                payload: payload,
                span: Span(start, close.span.end)
            }
        )
    }

    parseIdentifierPattern(
        _ name: Token,
        _ kind: TokenKind
    ) -> PatternId {
        arena.addPattern(
            Pattern::Name {
                name: name,
                span: name.span
            }
        )
    }

    parsePatternAtom(_ kind: Integer) -> PatternId {
        let token = advance

        arena.addPattern(
            Pattern::Integer {
                token: token,
                span: token.span
            }
        )
    }

    parsePatternAtom(_ kind: String) -> PatternId {
        let token = advance

        arena.addPattern(
            Pattern::String {
                token: token,
                span: token.span
            }
        )
    }

    parsePatternAtom(_ kind: LeftParen) -> PatternId {
        let open = advance
        let elements = List<PatternId>.new()

        if current.kind != RightParen {
            while true {
                elements.push(parsePattern())

                match advanceIf(Comma) {
                    Some(_) => ()
                    None => break
                }
            }
        }

        let close = expect(RightParen)

        arena.addPattern(
            Pattern::Tuple {
                elements: elements,
                span: Span(open.span.start, close.span.end)
            }
        )
    }

    parsePatternAtom(_ kind: TokenKind) -> PatternId {
        let span = current.span

        report(
            ExpectedPattern,
            span,
            "expected pattern"
        )

        advance

        arena.addPattern(
            Pattern::Error { span: span }
        )
    }
}
