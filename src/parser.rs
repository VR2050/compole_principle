use std::{error::Error, fmt};

use crate::{Delimiter, Keyword, Literal, Operator, SourcePosition, Token, TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        mutable: bool,
        name: String,
        value: Expr,
    },
    Return(Option<Expr>),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Ident(String),
    Unary {
        op: Operator,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub position: SourcePosition,
}

impl ParseError {
    fn new(token: &Token, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            position: token.position,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {}, column {}",
            self.message, self.position.line, self.position.column
        )
    }
}

impl Error for ParseError {}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseError> {
        Self::new(tokens).parse_program()
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while !self.current().kind.is_eof() {
            statements.push(self.parse_stmt()?);
        }

        Ok(Program { statements })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match &self.current().kind {
            TokenKind::Keyword(Keyword::Let) => self.parse_let_stmt(),
            TokenKind::Keyword(Keyword::Return) => self.parse_return_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let mutable = self.match_keyword(Keyword::Mut);
        let name = self.expect_ident("expected identifier after `let`")?;

        self.expect_operator(Operator::Assign)?;
        let value = self.parse_expr()?;
        self.expect_delimiter(Delimiter::Semicolon)?;

        Ok(Stmt::Let {
            mutable,
            name,
            value,
        })
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        if self.current().kind.is_delimiter(Delimiter::Semicolon) {
            self.advance();
            return Ok(Stmt::Return(None));
        }

        let expr = self.parse_expr()?;
        self.expect_delimiter(Delimiter::Semicolon)?;
        Ok(Stmt::Return(Some(expr)))
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expr()?;
        self.expect_delimiter(Delimiter::Semicolon)?;
        Ok(Stmt::Expr(expr))
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary_expr(1)
    }

    // Pratt-style precedence climbing keeps the teaching surface small
    // while still matching how many production parsers handle expressions.
    fn parse_binary_expr(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;

        while let TokenKind::Operator(op) = &self.current().kind {
            let op = *op;
            let prec = Self::operator_precedence(&op);

            if prec < min_prec || prec == 0 {
                break;
            }

            self.advance();
            let right = self.parse_binary_expr(prec + 1)?;

            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match &self.current().kind {
            TokenKind::Operator(Operator::Sub)
            | TokenKind::Operator(Operator::Add)
            | TokenKind::Operator(Operator::Not) => {
                let op = match self.current().kind {
                    TokenKind::Operator(op) => op,
                    _ => unreachable!(),
                };
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match &self.current().kind {
            TokenKind::Literal(literal) => {
                let expr = Expr::Literal(literal.clone());
                self.advance();
                Ok(expr)
            }
            TokenKind::Ident(name) => {
                let expr = Expr::Ident(name.clone());
                self.advance();
                Ok(expr)
            }
            TokenKind::Delimiter(Delimiter::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect_delimiter(Delimiter::RParen)?;
                Ok(Expr::Grouping(Box::new(expr)))
            }
            _ => Err(ParseError::new(self.current(), "expected expression")),
        }
    }

    fn operator_precedence(op: &Operator) -> u8 {
        match op {
            Operator::Or => 1,
            Operator::And => 2,
            Operator::Eq | Operator::Ne => 3,
            Operator::Lt | Operator::Le | Operator::Gt | Operator::Ge => 4,
            Operator::Add | Operator::Sub => 5,
            Operator::Mul | Operator::Div | Operator::Mod => 6,
            _ => 0,
        }
    }

    fn current(&self) -> &Token {
        let idx = self.pos.min(self.tokens.len().saturating_sub(1));
        &self.tokens[idx]
    }

    fn advance(&mut self) {
        if !self.current().kind.is_eof() {
            self.pos += 1;
        }
    }

    fn match_keyword(&mut self, keyword: Keyword) -> bool {
        if self.current().kind.is_keyword(keyword) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_ident(&mut self, message: &str) -> Result<String, ParseError> {
        match &self.current().kind {
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(ParseError::new(self.current(), message)),
        }
    }

    fn expect_operator(&mut self, operator: Operator) -> Result<(), ParseError> {
        if self.current().kind.is_operator(operator) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(
                self.current(),
                format!("expected operator {:?}", operator),
            ))
        }
    }

    fn expect_delimiter(&mut self, delimiter: Delimiter) -> Result<(), ParseError> {
        if self.current().kind.is_delimiter(delimiter) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(
                self.current(),
                format!("expected delimiter {:?}", delimiter),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Expr, Parser, Stmt};
    use crate::{Lexer, Literal, Operator, parse_source};

    #[test]
    fn parses_let_stmt_with_precedence() {
        let tokens = Lexer::new("let mut answer = 1 + 2 * 3;")
            .tokenize()
            .expect("lexer should succeed");

        let program = Parser::parse(tokens).expect("parser should succeed");

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::Let {
                mutable,
                name,
                value,
            } => {
                assert!(*mutable);
                assert_eq!(name, "answer");

                match value {
                    Expr::Binary { left, op, right } => {
                        assert_eq!(op, &Operator::Add);
                        assert_eq!(**left, Expr::Literal(Literal::Int(1)));

                        match &**right {
                            Expr::Binary {
                                left: mul_left,
                                op: mul_op,
                                right: mul_right,
                            } => {
                                assert_eq!(mul_op, &Operator::Mul);
                                assert_eq!(**mul_left, Expr::Literal(Literal::Int(2)));
                                assert_eq!(**mul_right, Expr::Literal(Literal::Int(3)));
                            }
                            other => panic!("expected multiply expr, got {other:?}"),
                        }
                    }
                    other => panic!("expected binary expr, got {other:?}"),
                }
            }
            other => panic!("expected let stmt, got {other:?}"),
        }
    }

    #[test]
    fn parses_return_stmt() {
        let program = parse_source("return 42;").expect("frontend should succeed");
        assert_eq!(
            program.statements,
            vec![Stmt::Return(Some(Expr::Literal(Literal::Int(42))))]
        );
    }

    #[test]
    fn reports_missing_semicolon() {
        let err = parse_source("let value = 1").unwrap_err();
        assert!(format!("{err}").contains("expected delimiter Semicolon"));
    }
}
