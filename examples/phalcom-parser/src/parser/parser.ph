from core.domains import Natural, DeclId
from lex.token import Token, TokenBuffer, TokenKind, Keyword
from syntax.arena import AstArena
from syntax.ast import Decl
from syntax.diagnostic import Diagnostic, DiagnosticKind

data ParseResult(
    declarations: List<DeclId>,
    arena: AstArena,
    diagnostics: List<Diagnostic>
)


class Parser {
    tokens: TokenBuffer
    arena: AstArena

    var cursor: Natural
    var diagnostics: List<Diagnostic>
}

impl Parser {
    @class
    parse(_ tokens: TokenBuffer) -> ParseResult {
        let parser = Parser(
            tokens: tokens,
            arena: AstArena.new(),
            cursor: 0,
            diagnostics: List<Diagnostic>.new()
        )

        let declarations = parser.parseModule()

        ParseResult(
            declarations,
            parser.arena,
            parser.diagnostics
        )
    }

    parseModule -> List<DeclId> {
        let declarations = List<DeclId>.new()

        skipSeparators()

        while current.kind != Eof {
            declarations.push(parseDeclaration())
            skipSeparators()
        }

        declarations
    }

    current -> Token {
        tokens.atOrEof(cursor)
    }

    previous -> Token {
        if cursor == 0 {
            current
        } else {
            tokens.atOrEof(cursor - 1)
        }
    }

    peek(_ distance: Natural) -> Token {
        tokens.atOrEof(cursor + distance)
    }

    at(_ kind: TokenKind) -> Bool {
        current.kind == kind
    }

    atKeyword(_ keyword: Keyword) -> Bool {
        current.kind == Keyword(keyword)
    }

    advance -> Token {
        let token = current

        if token.kind != Eof {
            cursor += 1
        }

        token
    }

    advanceIf(_ kind: TokenKind) -> Option<Token> {
        if current.kind == kind {
            Some(advance)
        } else {
            None
        }
    }

    advanceIfKeyword(_ keyword: Keyword) -> Option<Token> {
        if current.kind == Keyword(keyword) {
            Some(advance)
        } else {
            None
        }
    }

    expect(_ kind: TokenKind) -> Token {
        if current.kind == kind {
            return advance
        }

        report(
            ExpectedToken(kind),
            current.span,
            "expected token"
        )

        Token(
            kind,
            Span.empty(at: current.span.start)
        )
    }

    expectIdentifier -> Token {
        match current.kind {
            Identifier => advance
            _ => {
                report(
                    UnexpectedToken,
                    current.span,
                    "expected identifier"
                )

                Token(
                    Identifier,
                    Span.empty(at: current.span.start)
                )
            }
        }
    }

    skipSeparators {
        while match current.kind {
            Newline | Semicolon => true
            _ => false
        } {
            advance
        }
    }

    report(
        _ kind: DiagnosticKind,
        _ span: Span,
        _ message: String
    ) {
        diagnostics.push(
            Diagnostic(kind, span, message)
        )
    }
}
