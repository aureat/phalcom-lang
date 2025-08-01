use crate::ast::Program;
use crate::error::{PhalcomError, SyntaxError};
use crate::lexer::Lexer;
use lalrpop_util::{lalrpop_mod, ParseError};
use std::ops::Range;

pub mod ast;
pub mod error;
pub mod lexer;
pub mod token;
mod util;

lalrpop_mod!(pub parser);

pub type ParserResult<T> = Result<T, Vec<(PhalcomError, Range<usize>)>>;

pub fn parse(source: &str, offset: usize) -> ParserResult<Program> {
    let lexer = Lexer::new(source).map(|token| match token {
        Ok((l, token, r)) => Ok((l + offset, token, r + offset)),
        Err(e) => Err(e),
    });

    let parser = parser::ProgramParser::new();
    let mut errors: Vec<(PhalcomError, Range<usize>)> = Vec::new();

    let mut parser_errors = Vec::new();
    let program = match parser.parse(lexer) {
        Ok(program) => program,
        Err(err) => {
            parser_errors.push(err);
            Program::default()
        }
    };

    errors.extend(parser_errors.into_iter().map(|err| match err {
        ParseError::ExtraToken { token: (start, _, end) } => (
            PhalcomError::SyntaxError(SyntaxError::ExtraToken {
                token: source[start - offset..end - offset].to_string(),
            }),
            start..end,
        ),
        ParseError::InvalidToken { location } => (PhalcomError::SyntaxError(SyntaxError::InvalidToken), location..location),
        ParseError::UnrecognizedEof { location, expected } => (PhalcomError::SyntaxError(SyntaxError::UnrecognizedEof { expected }), location..location),
        ParseError::UnrecognizedToken {
            token: (start, _, end),
            expected,
        } => (
            PhalcomError::SyntaxError(SyntaxError::UnrecognizedToken {
                token: source[start - offset..end - offset].to_string(),
                expected,
            }),
            start..end,
        ),
        ParseError::User { error: _error } => (PhalcomError::SyntaxError(SyntaxError::Other), 0..0),
    }));

    if errors.is_empty() { Ok(program) } else { Err(errors) }
}
