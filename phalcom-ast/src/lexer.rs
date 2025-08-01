use crate::token::{LexicalError, Token};
use logos::{Logos, SpannedIter};

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'input> {
    token_stream: SpannedIter<'input, Token>,
    last_span_end: usize,
    last_was_newline: bool,
    injected_newline: bool,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            token_stream: Token::lexer(input).spanned(),
            last_span_end: 0,
            last_was_newline: true,
            injected_newline: false,
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((tok_res, span)) = self.token_stream.next() {
            let item = tok_res.map(|tok| (span.start, tok, span.end));

            self.last_span_end = span.end;
            self.last_was_newline = matches!(item, Ok((_, Token::Newline, _)));
            return Some(item);
        }

        if !self.injected_newline && !self.last_was_newline {
            self.injected_newline = true;
            let pos = self.last_span_end; // 0-length span at file end
            return Some(Ok((pos, Token::Newline, pos)));
        }

        None
    }
}

// use crate::token::{LexicalError, Token};
// use logos::{Logos, SpannedIter};
//
// pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;
//
// pub struct Lexer<'input> {
//     token_stream: SpannedIter<'input, Token>,
// }
//
// impl<'input> Lexer<'input> {
//     pub fn new(input: &'input str) -> Self {
//         Self {
//             token_stream: Token::lexer(input).spanned(),
//         }
//     }
// }
//
// impl<'input> Iterator for Lexer<'input> {
//     type Item = Spanned<Token, usize, LexicalError>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         self.token_stream.next().map(|(token, span)| Ok((span.start, token?, span.end)))
//     }
// }
