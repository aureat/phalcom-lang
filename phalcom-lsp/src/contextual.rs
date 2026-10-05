use phalcom_ast::lexer::Lexer;
use phalcom_ast::token::Token;
use std::collections::BTreeSet;

/// Finds contextual `mut` markers without treating every identifier spelled
/// `mut` as a keyword. The recognized token shapes mirror the parser's
/// `let mut`, mutable-field, mutable-trait-property, and read/write-delegation
/// prefixes.
pub(crate) fn contextual_mut_ranges(text: &str) -> BTreeSet<(usize, usize)> {
    let tokens = Lexer::new(text).filter_map(Result::ok).collect::<Vec<_>>();
    let mut ranges = BTreeSet::new();

    for (index, (start, token, end)) in tokens.iter().enumerate() {
        if !matches!(token, Token::Identifier(name) if name == "mut") {
            continue;
        }
        let previous = index.checked_sub(1).map(|previous| &tokens[previous].1);
        let next = tokens.get(index + 1).map(|next| &next.1);
        let after_next = tokens.get(index + 2).map(|next| &next.1);

        let let_binding = matches!(previous, Some(Token::Let)) && !matches!(next, Some(Token::Equal | Token::Colon));
        let mutable_field = matches!(next, Some(Token::FieldIdentifier(_)))
            && matches!(
                after_next,
                None | Some(Token::Newline | Token::RBrace | Token::Eof | Token::Equal | Token::Colon)
            );
        let mutable_trait_property = matches!(next, Some(Token::Identifier(_))) && matches!(after_next, Some(Token::Colon));
        let read_write_delegation = matches!(next, Some(Token::Identifier(_))) && matches!(after_next, Some(Token::Identifier(word)) if word == "via");

        if let_binding || mutable_field || mutable_trait_property || read_write_delegation {
            ranges.insert((*start, *end));
        }
    }

    ranges
}
