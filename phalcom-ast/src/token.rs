use logos::{Logos, Span};
use std::num::{ParseFloatError, ParseIntError};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    InvalidFloat(ParseFloatError),
    InvalidToken(Span),

    #[default]
    Invalid,
}

impl From<ParseIntError> for LexicalError {
    fn from(err: ParseIntError) -> Self {
        LexicalError::InvalidInteger(err)
    }
}

impl From<ParseFloatError> for LexicalError {
    fn from(err: ParseFloatError) -> Self {
        LexicalError::InvalidFloat(err)
    }
}

impl LexicalError {
    fn from_lexer(lex: &mut logos::Lexer<Token>) -> Self {
        LexicalError::InvalidToken(lex.span())
    }
}

fn lex_number(lexer: &mut logos::Lexer<Token>) -> Result<f64, ParseFloatError> {
    let slice = lexer.slice();
    slice.parse::<f64>()
}

fn lex_string(lexer: &mut logos::Lexer<Token>) -> String {
    let slice = lexer.slice();
    slice[1..slice.len() - 1].to_string()
}

fn lex_identifier(lexer: &mut logos::Lexer<Token>) -> String {
    let slice = lexer.slice();
    slice.to_string()
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(error = LexicalError)]
#[logos(skip r"[ \t\f]+")]
#[logos(skip r"//[^\n]*")]
pub enum Token {
    #[token("let")]
    Let,
    #[token("fn")]
    Fn,
    #[token("class")]
    Class,
    #[token("return")]
    Return,
    #[token("true")]
    True,
    #[token("+=")]
    PlusEqual,
    #[token("-=")]
    MinusEqual,
    #[token("*=")]
    AsteriskEqual,
    #[token("/=")]
    SlashEqual,
    #[token("%=")]
    PercentEqual,
    #[token("false")]
    False,
    #[token("nil")]
    Nil,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("import")]
    Import,
    #[token("self")]
    SelfKw,
    #[token("super")]
    Super,
    #[token("in")]
    In,
    #[token("as")]
    As,
    #[token("is")]
    Is,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    #[token("static")]
    Static,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", lex_identifier)]
    Identifier(String),

    #[regex(r#""[^"]*""#, lex_string)]
    String(String),

    #[regex(r#"[0-9]+(\.[0-9]+)?"#, lex_number)]
    Number(f64),

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    #[token("=")]
    Equal,
    #[token("==")]
    EqualEqual,
    #[token("!=")]
    BangEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,

    #[token(";")]
    Semicolon,
    #[regex(r"\r?\n")]
    Newline,

    #[token(":")]
    Colon,
    #[token("::")]
    ColonColon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("..")]
    DotDot,
    #[token("...")]
    DotDotDot,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("?")]
    Question,
    #[token("!")]
    Bang,
    #[token("@")]
    At,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Asterisk,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,

    #[end]
    Eof,
    // #[logos(skip r"[ \t\f]+")]
    // #[logos(skip r"//[^\n]*")]
    // #[error(LexicalError)]
}
