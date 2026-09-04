//! Symbolic type and callable syntax AST and parser for Phalcom native metadata.

use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TypeExpr {
    Unknown,
    Never,
    SelfType,
    Named(String),
    Universe(String),
    Parameter(String),
    Applied { origin: Box<TypeExpr>, arguments: Vec<TypeExpr> },
    Union(Vec<TypeExpr>),
    Tuple(Box<ParameterTuple>),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct LabeledParameter {
    pub label: String,
    pub ty: TypeExpr,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct RestParameter {
    pub ty: Option<TypeExpr>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct ParameterTuple {
    pub positional: Vec<TypeExpr>,
    pub labeled: Vec<LabeledParameter>,
    pub rest: Option<RestParameter>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TypeParameter {
    pub name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum GenericConstraintRelation {
    Subtype,
    Equivalent,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct GenericConstraint {
    pub relation: GenericConstraintRelation,
    pub lower: TypeExpr,
    pub upper: TypeExpr,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CallableType {
    pub type_params: Vec<TypeParameter>,
    pub params: ParameterTuple,
    pub return_type: TypeExpr,
    pub constraints: Vec<GenericConstraint>,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum TypeSyntaxError {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("unexpected character: {0:?}")]
    UnexpectedChar(char),
    #[error("expected {expected}, found {found:?}")]
    Expected { expected: &'static str, found: String },
    #[error("invalid type syntax: {0}")]
    Invalid(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Ident(String),
    Where,
    SelfKw,
    Never,
    Unknown,
    Universe,
    Dot,
    Comma,
    Colon,
    Pipe,
    Arrow,
    EqEq,
    Subtype,
    LParen,
    RParen,
    LAngle,
    RAngle,
    Ellipsis,
    Eof,
}

struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    peeked: Option<char>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
            peeked: None,
        }
    }

    fn peek(&mut self) -> Option<char> {
        if self.peeked.is_none() {
            self.peeked = self.chars.next();
        }
        self.peeked
    }

    fn next(&mut self) -> Option<char> {
        if let Some(ch) = self.peeked.take() { Some(ch) } else { self.chars.next() }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Result<Token, TypeSyntaxError> {
        self.skip_whitespace();
        let Some(ch) = self.next() else {
            return Ok(Token::Eof);
        };

        match ch {
            '.' => {
                if self.peek() == Some('.') {
                    self.next();
                    if self.peek() == Some('.') {
                        self.next();
                        Ok(Token::Ellipsis)
                    } else {
                        Err(TypeSyntaxError::Invalid("expected '...' for rest parameter".into()))
                    }
                } else {
                    Ok(Token::Dot)
                }
            }
            ',' => Ok(Token::Comma),
            ':' => Ok(Token::Colon),
            '=' => {
                if self.peek() == Some('=') {
                    self.next();
                    Ok(Token::EqEq)
                } else {
                    Err(TypeSyntaxError::UnexpectedChar('='))
                }
            }
            '|' => Ok(Token::Pipe),
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '<' => {
                if self.peek() == Some(':') {
                    self.next();
                    Ok(Token::Subtype)
                } else {
                    Ok(Token::LAngle)
                }
            }
            '>' => Ok(Token::RAngle),
            '-' => {
                if self.next() == Some('>') {
                    Ok(Token::Arrow)
                } else {
                    Err(TypeSyntaxError::UnexpectedChar('-'))
                }
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut ident = String::new();
                ident.push(c);
                while let Some(peek_ch) = self.peek() {
                    if peek_ch.is_alphanumeric() || peek_ch == '_' {
                        ident.push(self.next().unwrap());
                    } else {
                        break;
                    }
                }
                match ident.as_str() {
                    "where" => Ok(Token::Where),
                    "Self" => Ok(Token::SelfKw),
                    "Never" => Ok(Token::Never),
                    "Unknown" => Ok(Token::Unknown),
                    "universe" => Ok(Token::Universe),
                    _ => Ok(Token::Ident(ident)),
                }
            }
            other => Err(TypeSyntaxError::UnexpectedChar(other)),
        }
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Result<Self, TypeSyntaxError> {
        let mut lexer = Lexer::new(input);
        let current = lexer.next_token()?;
        Ok(Self { lexer, current })
    }

    fn advance(&mut self) -> Result<Token, TypeSyntaxError> {
        let next = self.lexer.next_token()?;
        Ok(std::mem::replace(&mut self.current, next))
    }

    fn peek(&self) -> &Token {
        &self.current
    }

    fn expect(&mut self, expected: Token) -> Result<(), TypeSyntaxError> {
        if self.current == expected {
            self.advance()?;
            Ok(())
        } else {
            Err(TypeSyntaxError::Expected {
                expected: match expected {
                    Token::Ident(_) => "identifier",
                    Token::Dot => "'.'",
                    Token::Comma => "','",
                    Token::Colon => "':'",
                    Token::Pipe => "'|'",
                    Token::Arrow => "'->'",
                    Token::EqEq => "'=='",
                    Token::Subtype => "'<:'",
                    Token::LParen => "'('",
                    Token::RParen => "')'",
                    Token::LAngle => "'<'",
                    Token::RAngle => "'>'",
                    Token::Ellipsis => "'...'",
                    Token::Eof => "end of input",
                    Token::SelfKw => "'Self'",
                    Token::Never => "'Never'",
                    Token::Unknown => "'Unknown'",
                    Token::Universe => "'universe'",
                    Token::Where => "'where'",
                },
                found: format!("{:?}", self.current),
            })
        }
    }

    /// Parses a single TypeExpr.
    pub fn parse_type(&mut self) -> Result<TypeExpr, TypeSyntaxError> {
        self.parse_union()
    }

    fn parse_union(&mut self) -> Result<TypeExpr, TypeSyntaxError> {
        let first = self.parse_primary()?;
        if self.peek() == &Token::Pipe {
            let mut alternatives = vec![first];
            while self.peek() == &Token::Pipe {
                self.advance()?;
                alternatives.push(self.parse_primary()?);
            }
            Ok(TypeExpr::Union(alternatives))
        } else {
            Ok(first)
        }
    }

    fn parse_primary(&mut self) -> Result<TypeExpr, TypeSyntaxError> {
        match self.peek() {
            Token::SelfKw => {
                self.advance()?;
                Ok(TypeExpr::SelfType)
            }
            Token::Never => {
                self.advance()?;
                Ok(TypeExpr::Never)
            }
            Token::Unknown => {
                self.advance()?;
                Ok(TypeExpr::Unknown)
            }
            Token::Universe => {
                self.advance()?;
                self.expect(Token::Dot)?;
                let Token::Ident(name) = self.advance()? else {
                    return Err(TypeSyntaxError::Expected {
                        expected: "identifier after universe.",
                        found: format!("{:?}", self.current),
                    });
                };
                let origin = TypeExpr::Universe(name);
                self.parse_optional_type_args(origin)
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance()?;
                let origin = TypeExpr::Named(name);
                self.parse_optional_type_args(origin)
            }
            Token::LParen => {
                self.advance()?;
                let tuple = self.parse_tuple_body()?;
                self.expect(Token::RParen)?;
                Ok(TypeExpr::Tuple(Box::new(tuple)))
            }
            Token::Eof => Err(TypeSyntaxError::UnexpectedEof),
            other => Err(TypeSyntaxError::Invalid(format!("unexpected token in type expression: {other:?}"))),
        }
    }

    fn parse_optional_type_args(&mut self, origin: TypeExpr) -> Result<TypeExpr, TypeSyntaxError> {
        if self.peek() == &Token::LAngle {
            self.advance()?;
            let mut args = Vec::new();
            while self.peek() != &Token::RAngle && self.peek() != &Token::Eof {
                args.push(self.parse_type()?);
                if self.peek() == &Token::Comma {
                    self.advance()?;
                } else {
                    break;
                }
            }
            self.expect(Token::RAngle)?;
            Ok(TypeExpr::Applied {
                origin: Box::new(origin),
                arguments: args,
            })
        } else {
            Ok(origin)
        }
    }

    fn parse_tuple_body(&mut self) -> Result<ParameterTuple, TypeSyntaxError> {
        let mut tuple = ParameterTuple::default();
        let mut saw_label = false;

        while self.peek() != &Token::RParen && self.peek() != &Token::Eof {
            if self.peek() == &Token::Ellipsis {
                self.advance()?;
                let ty = if self.peek() != &Token::Comma && self.peek() != &Token::RParen {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                tuple.rest = Some(RestParameter { ty });
                if self.peek() == &Token::Comma {
                    self.advance()?;
                }
                break;
            }

            // Check if labeled: Ident followed by Colon
            let is_labeled = if let Token::Ident(_) = self.peek() {
                // We need to look ahead
                // Since our lexer only does 1 token peek, we can parse or check
                true
            } else {
                false
            };

            if is_labeled {
                // Try to see if it's `label: Type` or just a type named `Ident`
                // Let's inspect next token by advancing or clone parser
                let Token::Ident(id) = self.current.clone() else { unreachable!() };
                // Let's clone state to inspect
                let mut lookahead = self.clone_state();
                lookahead.advance()?;
                if lookahead.peek() == &Token::Colon {
                    self.advance()?; // consume ident
                    self.advance()?; // consume colon
                    let ty = self.parse_type()?;
                    tuple.labeled.push(LabeledParameter { label: id, ty });
                    saw_label = true;
                } else {
                    if saw_label {
                        return Err(TypeSyntaxError::Invalid("positional parameter cannot follow labeled parameter".into()));
                    }
                    let ty = self.parse_type()?;
                    tuple.positional.push(ty);
                }
            } else {
                if saw_label {
                    return Err(TypeSyntaxError::Invalid("positional parameter cannot follow labeled parameter".into()));
                }
                let ty = self.parse_type()?;
                tuple.positional.push(ty);
            }

            if self.peek() == &Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }

        Ok(tuple)
    }

    fn clone_state(&self) -> Self {
        Self {
            lexer: Lexer {
                chars: self.lexer.chars.clone(),
                peeked: self.lexer.peeked,
            },
            current: self.current.clone(),
        }
    }

    fn parse_generic_constraints(&mut self) -> Result<Vec<GenericConstraint>, TypeSyntaxError> {
        let mut constraints = Vec::new();
        loop {
            let lower = self.parse_type()?;
            let relation = match self.peek() {
                Token::Subtype => {
                    self.advance()?;
                    GenericConstraintRelation::Subtype
                }
                Token::EqEq => {
                    self.advance()?;
                    GenericConstraintRelation::Equivalent
                }
                other => {
                    return Err(TypeSyntaxError::Expected {
                        expected: "'<:' or '==' in generic constraint",
                        found: format!("{other:?}"),
                    });
                }
            };
            let upper = self.parse_type()?;
            constraints.push(GenericConstraint { relation, lower, upper });
            if self.peek() == &Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }
        Ok(constraints)
    }

    /// Parses a CallableType, e.g.:
    /// `() -> String`
    /// `(Symbol) -> Option<Method>`
    /// `<T>(T) -> Option<T>`
    /// `<T, U>(T, using: U) -> U`
    pub fn parse_callable(&mut self) -> Result<CallableType, TypeSyntaxError> {
        let mut type_params = Vec::new();
        if self.peek() == &Token::LAngle {
            self.advance()?;
            while self.peek() != &Token::RAngle && self.peek() != &Token::Eof {
                let Token::Ident(name) = self.advance()? else {
                    return Err(TypeSyntaxError::Expected {
                        expected: "generic type parameter name",
                        found: format!("{:?}", self.current),
                    });
                };
                type_params.push(TypeParameter { name });
                if self.peek() == &Token::Comma {
                    self.advance()?;
                } else {
                    break;
                }
            }
            self.expect(Token::RAngle)?;
        }

        self.expect(Token::LParen)?;
        let params = self.parse_tuple_body()?;
        self.expect(Token::RParen)?;
        self.expect(Token::Arrow)?;
        let return_type = self.parse_type()?;
        let constraints = if self.peek() == &Token::Where {
            self.advance()?;
            self.parse_generic_constraints()?
        } else {
            Vec::new()
        };

        Ok(CallableType {
            type_params,
            params,
            return_type,
            constraints,
        })
    }

    /// Parses params = [Object, Object, foo: SomeType, ...]
    pub fn parse_param_list(&mut self) -> Result<ParameterTuple, TypeSyntaxError> {
        self.parse_tuple_body()
    }
}

pub fn parse_type_expr(input: &str) -> Result<TypeExpr, TypeSyntaxError> {
    let mut parser = Parser::new(input)?;
    let ty = parser.parse_type()?;
    if parser.peek() != &Token::Eof {
        return Err(TypeSyntaxError::Invalid(format!("trailing tokens: {:?}", parser.peek())));
    }
    Ok(ty)
}

pub fn parse_callable_type(input: &str) -> Result<CallableType, TypeSyntaxError> {
    let mut parser = Parser::new(input)?;
    let callable = parser.parse_callable()?;
    if parser.peek() != &Token::Eof {
        return Err(TypeSyntaxError::Invalid(format!("trailing tokens: {:?}", parser.peek())));
    }
    Ok(callable)
}

impl fmt::Display for TypeExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeExpr::Unknown => write!(f, "Unknown"),
            TypeExpr::Never => write!(f, "Never"),
            TypeExpr::SelfType => write!(f, "Self"),
            TypeExpr::Named(name) => write!(f, "{name}"),
            TypeExpr::Universe(name) => write!(f, "universe.{name}"),
            TypeExpr::Parameter(name) => write!(f, "{name}"),
            TypeExpr::Applied { origin, arguments } => {
                write!(f, "{origin}<")?;
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ">")
            }
            TypeExpr::Union(alts) => {
                for (i, alt) in alts.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{alt}")?;
                }
                Ok(())
            }
            TypeExpr::Tuple(tuple) => write!(f, "{tuple}"),
        }
    }
}

impl fmt::Display for ParameterTuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        let mut first = true;
        for pos in &self.positional {
            if !first {
                write!(f, ", ")?;
            }
            first = false;
            write!(f, "{pos}")?;
        }
        for labeled in &self.labeled {
            if !first {
                write!(f, ", ")?;
            }
            first = false;
            write!(f, "{}: {}", labeled.label, labeled.ty)?;
        }
        if let Some(rest) = &self.rest {
            if !first {
                write!(f, ", ")?;
            }
            if let Some(ty) = &rest.ty {
                write!(f, "...{ty}")?;
            } else {
                write!(f, "...")?;
            }
        }
        write!(f, ")")
    }
}

impl fmt::Display for CallableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.type_params.is_empty() {
            write!(f, "<")?;
            for (i, param) in self.type_params.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", param.name)?;
            }
            write!(f, ">")?;
        }
        write!(f, "{} -> {}", self.params, self.return_type)?;
        if !self.constraints.is_empty() {
            write!(f, " where ")?;
            for (index, constraint) in self.constraints.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                write!(
                    f,
                    "{} {} {}",
                    constraint.lower,
                    match constraint.relation {
                        GenericConstraintRelation::Subtype => "<:",
                        GenericConstraintRelation::Equivalent => "==",
                    },
                    constraint.upper
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_types() {
        assert_eq!(parse_type_expr("Object").unwrap(), TypeExpr::Named("Object".into()));
        assert_eq!(parse_type_expr("Self").unwrap(), TypeExpr::SelfType);
        assert_eq!(parse_type_expr("Never").unwrap(), TypeExpr::Never);
        assert_eq!(parse_type_expr("Unknown").unwrap(), TypeExpr::Unknown);
        assert_eq!(
            parse_type_expr("universe.BoundMethodFamily").unwrap(),
            TypeExpr::Universe("BoundMethodFamily".into())
        );
    }

    #[test]
    fn test_parse_applied_types() {
        assert_eq!(
            parse_type_expr("Option<Method>").unwrap(),
            TypeExpr::Applied {
                origin: Box::new(TypeExpr::Named("Option".into())),
                arguments: vec![TypeExpr::Named("Method".into())],
            }
        );
        assert_eq!(
            parse_type_expr("Result<String, Error>").unwrap(),
            TypeExpr::Applied {
                origin: Box::new(TypeExpr::Named("Result".into())),
                arguments: vec![TypeExpr::Named("String".into()), TypeExpr::Named("Error".into())],
            }
        );
    }

    #[test]
    fn test_parse_union_types() {
        assert_eq!(
            parse_type_expr("A | B").unwrap(),
            TypeExpr::Union(vec![TypeExpr::Named("A".into()), TypeExpr::Named("B".into())])
        );
    }

    #[test]
    fn test_parse_callables() {
        let callable = parse_callable_type("(Symbol) -> Option<Method>").unwrap();
        assert_eq!(callable.type_params.len(), 0);
        assert_eq!(callable.params.positional, vec![TypeExpr::Named("Symbol".into())]);
        assert_eq!(
            callable.return_type,
            TypeExpr::Applied {
                origin: Box::new(TypeExpr::Named("Option".into())),
                arguments: vec![TypeExpr::Named("Method".into())]
            }
        );

        let generic = parse_callable_type("<T>(T) -> Option<T>").unwrap();
        assert_eq!(generic.type_params[0].name, "T");
        assert_eq!(generic.params.positional, vec![TypeExpr::Named("T".into())]);

        let labeled = parse_callable_type("(Object, foo: String) -> Bool").unwrap();
        assert_eq!(labeled.params.positional, vec![TypeExpr::Named("Object".into())]);
        assert_eq!(labeled.params.labeled[0].label, "foo");
        assert_eq!(labeled.params.labeled[0].ty, TypeExpr::Named("String".into()));
        assert_eq!(labeled.return_type, TypeExpr::Named("Bool".into()));
    }

    #[test]
    fn test_parse_callable_generic_constraints() {
        let callable = parse_callable_type("<T>(T) -> T where T <: Object, T == T").unwrap();
        assert_eq!(callable.constraints.len(), 2);
        assert_eq!(callable.constraints[0].relation, GenericConstraintRelation::Subtype);
        assert_eq!(callable.constraints[0].lower, TypeExpr::Named("T".into()));
        assert_eq!(callable.constraints[0].upper, TypeExpr::Named("Object".into()));
        assert_eq!(callable.constraints[1].relation, GenericConstraintRelation::Equivalent);
    }
}
