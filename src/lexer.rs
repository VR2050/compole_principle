use std::{error::Error, fmt, iter::Peekable, str::Chars};

use crate::{Delimiter, Keyword, Literal, Operator, SourcePosition, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub position: SourcePosition,
}

impl LexError {
    fn new(position: SourcePosition, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            position,
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {}, column {}",
            self.message, self.position.line, self.position.column
        )
    }
}

impl Error for LexError {}

#[derive(Debug)]
pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    current: Option<char>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars().peekable();
        let current = chars.next();
        Self {
            chars,
            current,
            line: 1,
            column: 1,
        }
    }

    fn current(&self) -> Option<char> {
        self.current
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn peek_second(&self) -> Option<char> {
        let mut clone = self.chars.clone();
        clone.next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.current;

        if let Some(c) = ch {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        self.current = self.chars.next();
        ch
    }

    fn position(&self) -> SourcePosition {
        SourcePosition::new(self.line, self.column)
    }

    fn skip_ignored(&mut self) {
        loop {
            while matches!(self.current(), Some(c) if c.is_whitespace()) {
                self.bump();
            }

            if self.current() == Some('/') && self.peek() == Some('/') {
                while let Some(c) = self.current() {
                    self.bump();
                    if c == '\n' {
                        break;
                    }
                }
                continue;
            }

            break;
        }
    }

    fn read_identifier(&mut self) -> Token {
        let position = self.position();
        let mut ident = String::new();

        while matches!(self.current(), Some(ch) if ch.is_ascii_alphanumeric() || ch == '_') {
            ident.push(self.bump().unwrap());
        }

        match ident.as_str() {
            "true" => Token::literal(Literal::Bool(true), position.line, position.column),
            "false" => Token::literal(Literal::Bool(false), position.line, position.column),
            _ => match Keyword::from_str(&ident) {
                Some(keyword) => Token::keyword(keyword, position.line, position.column),
                None => Token::ident(ident, position.line, position.column),
            },
        }
    }

    fn read_number(&mut self) -> Result<Token, LexError> {
        let position = self.position();
        let mut number = String::new();
        let mut is_float = false;

        while let Some(c) = self.current() {
            if c.is_ascii_digit() {
                number.push(self.bump().unwrap());
            } else if c == '.' && self.peek().is_some_and(|next| next.is_ascii_digit()) {
                is_float = true;
                number.push(self.bump().unwrap());
            } else {
                break;
            }
        }

        if is_float {
            let value = number
                .parse()
                .map_err(|_| LexError::new(position, format!("invalid float literal `{number}`")))?;
            Ok(Token::literal(Literal::Float(value), position.line, position.column))
        } else {
            let value = number
                .parse()
                .map_err(|_| LexError::new(position, format!("invalid integer literal `{number}`")))?;
            Ok(Token::literal(Literal::Int(value), position.line, position.column))
        }
    }

    fn read_string(&mut self) -> Result<Token, LexError> {
        let position = self.position();
        let mut s = String::new();
        self.bump();

        while let Some(c) = self.current() {
            match c {
                '"' => {
                    self.bump();
                    return Ok(Token::literal(
                        Literal::Str(s),
                        position.line,
                        position.column,
                    ));
                }
                '\\' => {
                    self.bump();
                    let escaped = self
                        .current()
                        .ok_or_else(|| LexError::new(position, "unterminated string literal"))?;
                    let mapped = match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        '\'' => '\'',
                        other => {
                            return Err(LexError::new(
                                self.position(),
                                format!("unsupported escape sequence `\\{other}`"),
                            ));
                        }
                    };
                    s.push(mapped);
                    self.bump();
                }
                _ => {
                    s.push(c);
                    self.bump();
                }
            }
        }

        Err(LexError::new(position, "unterminated string literal"))
    }

    fn read_char(&mut self) -> Result<Token, LexError> {
        let position = self.position();
        self.bump();

        let value = match self.current() {
            Some('\\') => {
                self.bump();
                match self.current() {
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('r') => '\r',
                    Some('\\') => '\\',
                    Some('\'') => '\'',
                    Some(other) => {
                        return Err(LexError::new(
                            self.position(),
                            format!("unsupported escape sequence `\\{other}`"),
                        ));
                    }
                    None => return Err(LexError::new(position, "unterminated char literal")),
                }
            }
            Some(c) => c,
            None => return Err(LexError::new(position, "unterminated char literal")),
        };

        self.bump();
        if self.current() != Some('\'') {
            return Err(LexError::new(position, "char literal must contain exactly one character"));
        }
        self.bump();

        Ok(Token::literal(
            Literal::Char(value),
            position.line,
            position.column,
        ))
    }

    fn read_operator(&mut self) -> Result<Token, LexError> {
        let position = self.position();
        let op = match self.current().unwrap() {
            '+' => {
                self.bump();
                if self.current() == Some('=') {
                    self.bump();
                    Operator::AddAssign
                } else if self.current() == Some('+') {
                    self.bump();
                    Operator::Inc
                } else {
                    Operator::Add
                }
            }
            '-' => {
                self.bump();
                if self.current() == Some('=') {
                    self.bump();
                    Operator::SubAssign
                } else if self.current() == Some('>') {
                    self.bump();
                    Operator::Arrow
                } else if self.current() == Some('-') {
                    self.bump();
                    Operator::Dec
                } else {
                    Operator::Sub
                }
            }
            '*' => {
                self.bump();
                if self.current() == Some('=') {
                    self.bump();
                    Operator::MulAssign
                } else {
                    Operator::Mul
                }
            }
            '/' => {
                self.bump();
                if self.current() == Some('=') {
                    self.bump();
                    Operator::DivAssign
                } else {
                    Operator::Div
                }
            }
            '%' => {
                self.bump();
                if self.current() == Some('=') {
                    self.bump();
                    Operator::ModAssign
                } else {
                    Operator::Mod
                }
            }
            '=' => {
                self.bump();
                if self.current() == Some('=') {
                    self.bump();
                    Operator::Eq
                } else if self.current() == Some('>') {
                    self.bump();
                    Operator::FatArrow
                } else {
                    Operator::Assign
                }
            }
            '!' => {
                self.bump();
                if self.current() == Some('=') {
                    self.bump();
                    Operator::Ne
                } else {
                    Operator::Not
                }
            }
            '<' => {
                self.bump();
                if self.current() == Some('=') {
                    self.bump();
                    Operator::Le
                } else if self.current() == Some('<') {
                    self.bump();
                    Operator::Shl
                } else {
                    Operator::Lt
                }
            }
            '>' => {
                self.bump();
                if self.current() == Some('=') {
                    self.bump();
                    Operator::Ge
                } else if self.current() == Some('>') {
                    self.bump();
                    Operator::Shr
                } else {
                    Operator::Gt
                }
            }
            '&' => {
                self.bump();
                if self.current() == Some('&') {
                    self.bump();
                    Operator::And
                } else {
                    Operator::BitAnd
                }
            }
            '|' => {
                self.bump();
                if self.current() == Some('|') {
                    self.bump();
                    Operator::Or
                } else {
                    Operator::BitOr
                }
            }
            '^' => {
                self.bump();
                Operator::BitXor
            }
            '~' => {
                self.bump();
                Operator::BitNot
            }
            '.' => {
                self.bump();
                if self.current() == Some('.') {
                    self.bump();
                    if self.current() == Some('=') {
                        self.bump();
                        Operator::RangeEq
                    } else {
                        Operator::Range
                    }
                } else {
                    return Err(LexError::new(position, "single `.` should be lexed as delimiter"));
                }
            }
            '?' => {
                self.bump();
                Operator::Question
            }
            _ => return Err(LexError::new(position, "unknown operator")),
        };

        Ok(Token::operator(op, position.line, position.column))
    }

    fn read_delimiter(&mut self) -> Result<Token, LexError> {
        let position = self.position();

        let delimiter = match self.current().unwrap() {
            '(' => Delimiter::LParen,
            ')' => Delimiter::RParen,
            '{' => Delimiter::LBrace,
            '}' => Delimiter::RBrace,
            '[' => Delimiter::LBracket,
            ']' => Delimiter::RBracket,
            ',' => Delimiter::Comma,
            ';' => Delimiter::Semicolon,
            ':' => {
                self.bump();
                if self.current() == Some(':') {
                    self.bump();
                    return Ok(Token::delimiter(Delimiter::Scope, position.line, position.column));
                }
                return Ok(Token::delimiter(Delimiter::Colon, position.line, position.column));
            }
            '.' => {
                self.bump();
                return Ok(Token::delimiter(Delimiter::Dot, position.line, position.column));
            }
            _ => return Err(LexError::new(position, "unknown delimiter")),
        };

        self.bump();
        Ok(Token::delimiter(delimiter, position.line, position.column))
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_ignored();

        match self.current() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => Ok(self.read_identifier()),
            Some(c) if c.is_ascii_digit() => self.read_number(),
            Some('"') => self.read_string(),
            Some('\'') => self.read_char(),
            Some('.')
                if self.peek() == Some('.')
                    || (self.peek() == Some('.') && self.peek_second() == Some('=')) =>
            {
                self.read_operator()
            }
            Some('+')
            | Some('-')
            | Some('*')
            | Some('/')
            | Some('%')
            | Some('=')
            | Some('!')
            | Some('<')
            | Some('>')
            | Some('&')
            | Some('|')
            | Some('^')
            | Some('~')
            | Some('?') => self.read_operator(),
            Some('(')
            | Some(')')
            | Some('{')
            | Some('}')
            | Some('[')
            | Some(']')
            | Some(',')
            | Some(';')
            | Some(':')
            | Some('.') => self.read_delimiter(),
            None => Ok(Token::eof(self.line, self.column)),
            Some(c) => Err(LexError::new(
                self.position(),
                format!("unexpected character `{c}`"),
            )),
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let eof = token.kind.is_eof();
            tokens.push(token);

            if eof {
                break;
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::{Literal, Operator, TokenKind};

    #[test]
    fn lexes_comments_and_range_operator() {
        let tokens = Lexer::new("let a = 1..=3; // range").tokenize().unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::Keyword(_)));
        assert!(matches!(tokens[3].kind, TokenKind::Literal(Literal::Int(1))));
        assert!(matches!(tokens[4].kind, TokenKind::Operator(Operator::RangeEq)));
    }

    #[test]
    fn reports_unterminated_string() {
        let err = Lexer::new("\"hello").tokenize().unwrap_err();
        assert!(err.message.contains("unterminated string"));
    }
}
