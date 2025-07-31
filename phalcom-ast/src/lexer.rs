use crate::token::{LexicalError, Token};
use logos::{Logos, SpannedIter};

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'input> {
    token_stream: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            token_stream: Token::lexer(input).spanned(),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream.next().map(|(token, span)| Ok((span.start, token?, span.end)))
    }
}

// use crate::token::{LexicalError, Token};
// use logos::{Logos, SpannedIter};
// use std::collections::VecDeque;
//
// pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;
//
// pub struct Lexer<'input> {
//     inner: SpannedIter<'input, Token>,
//     buf: VecDeque<Spanned<Token, usize, LexicalError>>,
//     brace_depth: usize,       // { }
//     paren_brack_depth: usize, // (…) and […]
//     prev: Option<(usize, Token, usize)>,
//     eof_done: bool,
// }
//
// impl<'input> Lexer<'input> {
//     pub fn new(input: &'input str) -> Self {
//         Self {
//             inner: Token::lexer(input).spanned(),
//             buf: VecDeque::new(),
//             brace_depth: 0,
//             paren_brack_depth: 0,
//             prev: None,
//             eof_done: false,
//         }
//     }
//
//     fn can_end_statement(tok: &Token) -> bool {
//         use Token::*;
//         matches!(
//             tok,
//             Identifier(_)
//             | Number(_)
//             | String(_)
//             | True | False | Nil
//             | Break | Continue
//             | Return        // bare `return`
//             | RParen | RBracket | RBrace
//         )
//     }
//
//     fn handle_newline(&mut self, loc: usize) {
//         if self.paren_brack_depth == 0 {
//             // ← **new condition**
//             if let Some((_, ref prev_tok, _)) = self.prev {
//                 if Self::can_end_statement(prev_tok) {
//                     self.buf.push_back(Ok((loc, Token::Semicolon, loc)));
//                 }
//             }
//         }
//     }
//
//     fn push(&mut self, span: (usize, Token, usize)) {
//         use Token::*;
//         match span.1 {
//             LBrace => self.brace_depth += 1,
//             RBrace => self.brace_depth = self.brace_depth.saturating_sub(1),
//             LParen | LBracket => self.paren_brack_depth += 1,
//             RParen | RBracket => self.paren_brack_depth = self.paren_brack_depth.saturating_sub(1),
//             _ => {}
//         }
//         self.prev = Some(span.clone());
//         self.buf.push_back(Ok(span));
//     }
// }
//
// impl<'input> Iterator for Lexer<'input> {
//     type Item = Spanned<Token, usize, LexicalError>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         // anything queued already?
//         if let Some(item) = self.buf.pop_front() {
//             return Some(item);
//         }
//
//         // fetch next raw token from Logos
//         match self.inner.next() {
//             Some((Ok(tok), span)) => {
//                 let (start, end) = (span.start, span.end);
//                 match tok {
//                     Token::Newline => {
//                         self.handle_newline(start);
//                         self.next() // pull again
//                     }
//                     other => {
//                         self.push((start, other, end));
//                         self.buf.pop_front()
//                     }
//                 }
//             }
//             Some((Err(e), _span)) => Some(Err(e)),
//             None => {
//                 // End-of-file: maybe one last implied semicolon.
//                 if !self.eof_done {
//                     self.eof_done = true;
//                     if let Some((loc, ref prev_tok, _)) = self.prev {
//                         if Self::can_end_statement(prev_tok) {
//                             return Some(Ok((loc, Token::Semicolon, loc)));
//                         }
//                     }
//                 }
//                 None
//             }
//         }
//     }
// }
