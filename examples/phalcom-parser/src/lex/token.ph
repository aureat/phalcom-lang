from core.domains import Natural
from core.source import SourceText, Span

data Keyword {
    As
    Break
    Class
    Continue
    Data
    Else
    Enum
    False
    For
    If
    Impl
    In
    Is
    Let
    Match
    None
    Not
    Or
    And
    Private
    Return
    Self
    Trait
    True
    Type
    Var
    Where
    While
}


data TokenKind {
    Identifier
    Integer
    Float
    String
    Character

    Newline
    Eof

    Keyword(Keyword)

    LeftParen
    RightParen
    LeftBrace
    RightBrace
    LeftBracket
    RightBracket

    Comma
    Colon
    Semicolon
    Dot
    DoubleColon
    Question
    At

    Arrow
    FatArrow

    Equal
    EqualEqual
    BangEqual

    Less
    LessEqual
    Greater
    GreaterEqual

    Plus
    Minus
    Star
    Slash
    Percent

    Ampersand
    Pipe
    Bang

    PlusEqual
    MinusEqual
    StarEqual
    SlashEqual

    DotDot
    DotDotEqual
}


data Token(
    kind: TokenKind,
    span: Span
)


class TokenBuffer {
    _ source: SourceText
    _ tokens: List<Token>
}

impl TokenBuffer {
    @class
    new(
        _ source: SourceText,
        _ tokens: List<Token>
    ) -> TokenBuffer {
        if tokens.length == 0 {
            tokens.push(
                Token(Eof, Span.empty(at: source.length))
            )
        } else {
            match tokens.last.kind {
                Eof => ()
                _ => tokens.push(
                    Token(Eof, Span.empty(at: source.length))
                )
            }
        }

        TokenBuffer(source, tokens)
    }

    length -> Natural {
        _tokens.length
    }

    source -> SourceText {
        _source
    }

    at(_ index: Natural) -> Token
    where index < self.length
    {
        _tokens[index]
    }

    atOrEof(_ index: Natural) -> Token {
        if index < length {
            _tokens[index]
        } else {
            _tokens[length - 1]
        }
    }

    text(_ token: Token) -> String {
        _source.slice(token.span)
    }
}
