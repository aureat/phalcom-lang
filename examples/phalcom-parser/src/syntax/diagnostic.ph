from core.source import Span
from lex.token import TokenKind

data DiagnosticKind {
    UnexpectedToken
    ExpectedToken(TokenKind)
    ExpectedExpression
    ExpectedType
    ExpectedPattern
    InvalidRefinementExpression
    UnterminatedString
    UnterminatedCharacter
    InvalidNumber
    AmbiguousSyntax
}


data Diagnostic(
    kind: DiagnosticKind,
    span: Span,
    message: String
)
