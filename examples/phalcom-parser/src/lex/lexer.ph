from core.domains import SourceOffset
from core.source import SourceText, Span
from lex.token import Keyword, Token, TokenBuffer, TokenKind
from syntax.diagnostic import Diagnostic, DiagnosticKind

class Lexer {
    source: SourceText
    var cursor: SourceOffset
    var tokens: List<Token>
    var diagnostics: List<Diagnostic>
}

impl Lexer {
    @class
    lex(_ source: SourceText) -> (TokenBuffer, List<Diagnostic>) {
        let lexer = Lexer(
            source: source,
            cursor: 0,
            tokens: List<Token>.new(),
            diagnostics: List<Diagnostic>.new()
        )

        lexer.run()

        (
            TokenBuffer.new(source, lexer.tokens),
            lexer.diagnostics
        )
    }

    run {
        while cursor < source.length {
            scanToken()
        }

        tokens.push(
            Token(
                Eof,
                Span.empty(at: source.length)
            )
        )
    }

    currentChar -> Option<Char> {
        source.charAt(cursor)
    }

    advanceChar -> Option<Char> {
        match currentChar {
            None => None
            Some(char) => {
                cursor += 1
                Some(char)
            }
        }
    }

    advanceIfChar(_ expected: Char) -> Bool {
        match currentChar {
            Some(char) if char == expected => {
                cursor += 1
                true
            }
            _ => false
        }
    }

    scanToken {
        let start = cursor

        match advanceChar() {
            None => ()

            Some(' ' | '\t' | '\r') => ()
            Some('\n') => emit(Newline, from: start)

            Some('(') => emit(LeftParen, from: start)
            Some(')') => emit(RightParen, from: start)
            Some('{') => emit(LeftBrace, from: start)
            Some('}') => emit(RightBrace, from: start)
            Some('[') => emit(LeftBracket, from: start)
            Some(']') => emit(RightBracket, from: start)
            Some(',') => emit(Comma, from: start)
            Some(';') => emit(Semicolon, from: start)
            Some('@') => emit(At, from: start)
            Some('?') => emit(Question, from: start)

            Some(':') if advanceIfChar(':') =>
                emit(DoubleColon, from: start)

            Some(':') =>
                emit(Colon, from: start)

            Some('.') if advanceIfChar('.') and advanceIfChar('=') =>
                emit(DotDotEqual, from: start)

            Some('.') if advanceIfChar('.') =>
                emit(DotDot, from: start)

            Some('.') =>
                emit(Dot, from: start)

            Some('+') if advanceIfChar('=') =>
                emit(PlusEqual, from: start)

            Some('+') =>
                emit(Plus, from: start)

            Some('-') if advanceIfChar('>') =>
                emit(Arrow, from: start)

            Some('-') if advanceIfChar('=') =>
                emit(MinusEqual, from: start)

            Some('-') =>
                emit(Minus, from: start)

            Some('*') if advanceIfChar('=') =>
                emit(StarEqual, from: start)

            Some('*') =>
                emit(Star, from: start)

            Some('/') if advanceIfChar('/') =>
                skipLineComment()

            Some('/') if advanceIfChar('=') =>
                emit(SlashEqual, from: start)

            Some('/') =>
                emit(Slash, from: start)

            Some('%') => emit(Percent, from: start)

            Some('=') if advanceIfChar('=') =>
                emit(EqualEqual, from: start)

            Some('=') if advanceIfChar('>') =>
                emit(FatArrow, from: start)

            Some('=') =>
                emit(Equal, from: start)

            Some('!') if advanceIfChar('=') =>
                emit(BangEqual, from: start)

            Some('!') =>
                emit(Bang, from: start)

            Some('<') if advanceIfChar('=') =>
                emit(LessEqual, from: start)

            Some('<') =>
                emit(Less, from: start)

            Some('>') if advanceIfChar('=') =>
                emit(GreaterEqual, from: start)

            Some('>') =>
                emit(Greater, from: start)

            Some('&') => emit(Ampersand, from: start)
            Some('|') => emit(Pipe, from: start)

            Some('"') => scanString(from: start)
            Some('\'') => scanCharacter(from: start)

            Some(c) if c.isAsciiDigit =>
                scanNumber(from: start)

            Some(c) if c.isIdentifierStart =>
                scanIdentifier(from: start)

            Some(_) =>
                diagnostics.push(
                    Diagnostic(
                        UnexpectedToken,
                        Span(start, cursor),
                        "unexpected character"
                    )
                )
        }
    }

    skipLineComment {
        while cursor < source.length {
            match currentChar {
                Some('\n') | None => return
                _ => cursor += 1
            }
        }
    }

    scanString(from start: SourceOffset) {
        while cursor < source.length {
            match advanceChar() {
                Some('"') => {
                    emit(String, from: start)
                    return
                }

                Some('\\') if cursor < source.length =>
                    cursor += 1

                None =>
                    break

                _ =>
                    ()
            }
        }

        diagnostics.push(
            Diagnostic(
                UnterminatedString,
                Span(start, cursor),
                "unterminated string literal"
            )
        )

        emit(String, from: start)
    }

    scanCharacter(from start: SourceOffset) {
        if cursor < source.length {
            advanceChar()

            if advanceIfChar('\'') {
                emit(Character, from: start)
                return
            }
        }

        diagnostics.push(
            Diagnostic(
                UnterminatedCharacter,
                Span(start, cursor),
                "unterminated character literal"
            )
        )

        emit(Character, from: start)
    }

    scanNumber(from start: SourceOffset) {
        while match currentChar {
            Some(c) if c.isAsciiDigit => true
            _ => false
        } {
            cursor += 1
        }

        if currentChar == Some('.') {
            match source.charAt(cursor + 1) {
                Some(c) if c.isAsciiDigit => {
                    cursor += 1

                    while match currentChar {
                        Some(d) if d.isAsciiDigit => true
                        _ => false
                    } {
                        cursor += 1
                    }

                    emit(Float, from: start)
                    return
                }

                _ => ()
            }
        }

        emit(Integer, from: start)
    }

    scanIdentifier(from start: SourceOffset) {
        while match currentChar {
            Some(c) if c.isIdentifierContinue => true
            _ => false
        } {
            cursor += 1
        }

        let span = Span(start, cursor)
        let text = source.slice(span)

        match keyword(text) {
            Some(keyword) =>
                tokens.push(Token(Keyword(keyword), span))

            None =>
                tokens.push(Token(Identifier, span))
        }
    }

    keyword(_ text: "as") -> Option<Keyword> { Some(As) }
    keyword(_ text: "break") -> Option<Keyword> { Some(Break) }
    keyword(_ text: "class") -> Option<Keyword> { Some(Class) }
    keyword(_ text: "continue") -> Option<Keyword> { Some(Continue) }
    keyword(_ text: "data") -> Option<Keyword> { Some(Data) }
    keyword(_ text: "else") -> Option<Keyword> { Some(Else) }
    keyword(_ text: "enum") -> Option<Keyword> { Some(Enum) }
    keyword(_ text: "false") -> Option<Keyword> { Some(False) }
    keyword(_ text: "for") -> Option<Keyword> { Some(For) }
    keyword(_ text: "if") -> Option<Keyword> { Some(If) }
    keyword(_ text: "impl") -> Option<Keyword> { Some(Impl) }
    keyword(_ text: "in") -> Option<Keyword> { Some(In) }
    keyword(_ text: "is") -> Option<Keyword> { Some(Is) }
    keyword(_ text: "let") -> Option<Keyword> { Some(Let) }
    keyword(_ text: "match") -> Option<Keyword> { Some(Match) }
    keyword(_ text: "none") -> Option<Keyword> { Some(None) }
    keyword(_ text: "not") -> Option<Keyword> { Some(Not) }
    keyword(_ text: "or") -> Option<Keyword> { Some(Or) }
    keyword(_ text: "and") -> Option<Keyword> { Some(And) }
    keyword(_ text: "private") -> Option<Keyword> { Some(Private) }
    keyword(_ text: "return") -> Option<Keyword> { Some(Return) }
    keyword(_ text: "self") -> Option<Keyword> { Some(Self) }
    keyword(_ text: "trait") -> Option<Keyword> { Some(Trait) }
    keyword(_ text: "true") -> Option<Keyword> { Some(True) }
    keyword(_ text: "type") -> Option<Keyword> { Some(Type) }
    keyword(_ text: "var") -> Option<Keyword> { Some(Var) }
    keyword(_ text: "where") -> Option<Keyword> { Some(Where) }
    keyword(_ text: "while") -> Option<Keyword> { Some(While) }
    keyword(_ text: String) -> Option<Keyword> { None }

    emit(_ kind: TokenKind, from start: SourceOffset) {
        tokens.push(
            Token(
                kind,
                Span(start, cursor)
            )
        )
    }
}
