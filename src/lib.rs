pub mod lexer;
pub mod parser;
pub mod token;

pub use lexer::{LexError, Lexer};
pub use parser::{Expr, ParseError, Parser, Program, Stmt};
pub use token::{Delimiter, Keyword, Literal, Operator, SourcePosition, Token, TokenKind};

#[derive(Debug)]
pub enum FrontendError {
    Lex(LexError),
    Parse(ParseError),
}

impl std::fmt::Display for FrontendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lex(err) => write!(f, "{err}"),
            Self::Parse(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for FrontendError {}

impl From<LexError> for FrontendError {
    fn from(value: LexError) -> Self {
        Self::Lex(value)
    }
}

impl From<ParseError> for FrontendError {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

pub fn parse_source(source: &str) -> Result<Program, FrontendError> {
    let tokens = Lexer::new(source).tokenize()?;
    Ok(Parser::parse(tokens)?)
}
