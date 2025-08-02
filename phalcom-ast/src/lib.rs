use crate::ast::Program;
use crate::error::{SyntaxError, SyntaxErrorKind};
use crate::lexer::Lexer;
use crate::token::LexicalError;
use lalrpop_util::{lalrpop_mod, ParseError};

pub mod ast;
pub mod error;
pub mod lexer;
pub mod token;
pub mod util;

lalrpop_mod!(pub parser);

pub type ParserResult<T> = Result<T, SyntaxError>;

pub fn parse_source(source: &str, offset: usize) -> ParserResult<Program> {
    let lexer = Lexer::new(source).map(|token| match token {
        Ok((l, token, r)) => Ok((l + offset, token, r + offset)),
        Err(e) => Err(e),
    });

    let parser = parser::ProgramParser::new();
    let parser_result = parser.parse(lexer);

    parser_result.map_err(|err| match err {
        ParseError::ExtraToken { token: (start, _, end) } => SyntaxError {
            kind: SyntaxErrorKind::ExtraToken {
                token: source[start - offset..end - offset].to_string(),
            },
            range: start..end,
        },
        ParseError::InvalidToken { location } => SyntaxError {
            kind: SyntaxErrorKind::InvalidToken,
            range: location..location,
        },
        ParseError::UnrecognizedEof { location, expected } => SyntaxError {
            kind: SyntaxErrorKind::UnrecognizedEof { expected },
            range: location..location,
        },
        ParseError::UnrecognizedToken {
            token: (start, _, end),
            expected,
        } => SyntaxError {
            kind: SyntaxErrorKind::UnrecognizedToken {
                token: source[start - offset..end - offset].to_string(),
                expected,
            },
            range: start..end,
        },
        ParseError::User { error } => match error {
            LexicalError::InvalidInteger(err) => SyntaxError {
                kind: SyntaxErrorKind::InvalidInteger,
                range: 0..0,
            },
            LexicalError::InvalidFloat(err) => SyntaxError {
                kind: SyntaxErrorKind::InvalidFloat,
                range: 0..0,
            },
            LexicalError::InvalidToken(span) => SyntaxError {
                kind: SyntaxErrorKind::InvalidToken,
                range: span,
            },
            LexicalError::Invalid => SyntaxError {
                kind: SyntaxErrorKind::InvalidToken,
                range: 0..0,
            },
        },
    })
}
