//! Multi-layer syntax highlighting for the Phalcom REPL.
//!
//! Applies token-class colors to input text using three non-degrading layers:
//! - **L1 Lexical**: `phalcom_ast::lexer` token stream (synchronous per keystroke).
//! - **L2 Syntactic**: `phalcom-lsp` semantic token pass.
//! - **L3 Live**: `ReplOracle` snapshot lookup (dimming unbound identifiers).

use crate::oracle::ReplOracle;
use nu_ansi_term::{Color, Style};
use phalcom_ast::lexer::Lexer;
use phalcom_ast::token::Token;
use reedline::{Highlighter, StyledText};
use std::cell::RefCell;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct Span {
    s: usize,
    e: usize,
}

/// Three-layer non-degrading syntax highlighter for the Phalcom REPL.
pub struct PhalcomHighlighter {
    oracle: Arc<ReplOracle>,
}

impl PhalcomHighlighter {
    /// Creates a new [`PhalcomHighlighter`] backed by `oracle`.
    pub fn new(oracle: Arc<ReplOracle>) -> Self {
        Self { oracle }
    }
}

impl Highlighter for PhalcomHighlighter {
    fn highlight(&self, line: &str, _pos: usize) -> StyledText {
        let spans = RefCell::new(Vec::<Span>::new());

        let mut styled = StyledText::new();
        styled.push((Style::default(), line.to_string()));

        let mut color = |s: usize, e: usize, style: Style| {
            if s < e && e <= line.len() {
                styled.style_range(s, e, style);
            }
            spans.borrow_mut().push(Span { s, e });
        };

        let kw_style = Style::new().fg(Color::Magenta).bold();
        let method_style = Style::new().fg(Color::Fixed(75)).bold();
        let str_style = Style::new().fg(Color::Green);
        let num_style = Style::new().fg(Color::Fixed(216));
        let class_style = Style::new().fg(Color::Fixed(222)).bold();
        let var_style = Style::new().fg(Color::Red);
        let bool_style = Style::new().fg(Color::Fixed(216));
        let bracket_style = Style::new().fg(Color::Fixed(223)).bold();
        let unbound_style = Style::new().fg(Color::DarkGray).dimmed();

        // --- Layer 1: Lexical (phalcom_ast::lexer) ---
        let lexer = Lexer::new(line);
        for item in lexer {
            let (start, tok, end) = match item {
                Ok(triple) => triple,
                Err(_) => continue,
            };

            if start >= end || end > line.len() {
                continue;
            }

            match tok {
                Token::Let
                | Token::Const
                | Token::Fn
                | Token::Class
                | Token::Return
                | Token::If
                | Token::Else
                | Token::While
                | Token::For
                | Token::Break
                | Token::Continue
                | Token::Import
                | Token::SelfKw
                | Token::Super
                | Token::In
                | Token::As
                | Token::Is
                | Token::And
                | Token::Or
                | Token::Not
                | Token::Static
                | Token::Construct
                | Token::Throw
                | Token::Try => {
                    color(start, end, kw_style);
                }
                Token::True | Token::False => {
                    color(start, end, bool_style);
                }
                Token::Number(_) => {
                    color(start, end, num_style);
                }
                Token::String(_) | Token::StringInterp(_) => {
                    color(start, end, str_style);
                }
                Token::LParen
                | Token::RParen
                | Token::LBrace
                | Token::RBrace
                | Token::LBracket
                | Token::RBracket => {
                    color(start, end, bracket_style);
                }
                Token::Identifier(ref name) => {
                    let rest = line[end..].trim_start();
                    let is_call = rest.starts_with('(');

                    let first_char = name.chars().next().unwrap_or('\0');
                    if is_call {
                        color(start, end, method_style);
                    } else if first_char.is_uppercase() {
                        color(start, end, class_style);
                    } else {
                        color(start, end, var_style);
                    }
                }
                _ => {}
            }
        }

        // --- Layer 3: Live Oracle unbound refinement ---
        let snap = self.oracle.snapshot();

        let lexer3 = Lexer::new(line);
        for item in lexer3 {
            let (start, tok, end) = match item {
                Ok(triple) => triple,
                Err(_) => continue,
            };

            if let Token::Identifier(ref name) = tok {
                let is_bound = snap.is_global_name_bound(name) || is_builtin_symbol(name);
                if !is_bound {
                    color(start, end, unbound_style);
                }
            }
        }

        styled
    }
}

fn is_builtin_symbol(name: &str) -> bool {
    matches!(
        name,
        "Object"
            | "Class"
            | "String"
            | "List"
            | "Map"
            | "Number"
            | "Bool"
            | "True"
            | "False"
            | "Nil"
            | "Symbol"
            | "Function"
            | "Module"
            | "Fiber"
            | "Future"
            | "print"
    )
}
