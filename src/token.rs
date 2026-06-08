#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
}

impl SourcePosition {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword(Keyword),
    Ident(String),
    Literal(Literal),
    Operator(Operator),
    Delimiter(Delimiter),
    Eof,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Keyword {
    Let,
    Const,
    Static,
    Mut,
    Fn,
    Return,
    Struct,
    Enum,
    Trait,
    Type,
    Impl,
    If,
    Else,
    Match,
    For,
    Loop,
    Continue,
    Break,
    In,
    While,
    Mod,
    Pub,
    Crate,
    SelfLower,
    SelfUpper,
    Super,
    Async,
    Await,
    Unsafe,
    Extern,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Char(char),
    Float(f64),
    Int(i64),
    Str(String),
    Bool(bool),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Assign,
    AddAssign,
    SubAssign,
    DivAssign,
    MulAssign,
    ModAssign,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    Shl,
    Shr,
    Inc,
    Dec,
    Arrow,
    Range,
    RangeEq,
    FatArrow,
    Question,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Delimiter {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Scope,
    Semicolon,
    Colon,
}

impl Operator {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "+" => Some(Self::Add),
            "-" => Some(Self::Sub),
            "*" => Some(Self::Mul),
            "/" => Some(Self::Div),
            "%" => Some(Self::Mod),
            "=" => Some(Self::Assign),
            "+=" => Some(Self::AddAssign),
            "-=" => Some(Self::SubAssign),
            "*=" => Some(Self::MulAssign),
            "/=" => Some(Self::DivAssign),
            "%=" => Some(Self::ModAssign),
            "==" => Some(Self::Eq),
            "!=" => Some(Self::Ne),
            "<" => Some(Self::Lt),
            "<=" => Some(Self::Le),
            ">" => Some(Self::Gt),
            ">=" => Some(Self::Ge),
            "&&" => Some(Self::And),
            "||" => Some(Self::Or),
            "!" => Some(Self::Not),
            "&" => Some(Self::BitAnd),
            "|" => Some(Self::BitOr),
            "^" => Some(Self::BitXor),
            "~" => Some(Self::BitNot),
            "<<" => Some(Self::Shl),
            ">>" => Some(Self::Shr),
            "++" => Some(Self::Inc),
            "--" => Some(Self::Dec),
            "->" => Some(Self::Arrow),
            ".." => Some(Self::Range),
            "..=" => Some(Self::RangeEq),
            "=>" => Some(Self::FatArrow),
            "?" => Some(Self::Question),
            _ => None,
        }
    }
}

impl Delimiter {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "(" => Some(Self::LParen),
            ")" => Some(Self::RParen),
            "{" => Some(Self::LBrace),
            "}" => Some(Self::RBrace),
            "[" => Some(Self::LBracket),
            "]" => Some(Self::RBracket),
            "," => Some(Self::Comma),
            "." => Some(Self::Dot),
            "::" => Some(Self::Scope),
            ";" => Some(Self::Semicolon),
            ":" => Some(Self::Colon),
            _ => None,
        }
    }
}

impl Keyword {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "let" => Some(Self::Let),
            "const" => Some(Self::Const),
            "static" => Some(Self::Static),
            "mut" => Some(Self::Mut),
            "fn" => Some(Self::Fn),
            "return" => Some(Self::Return),
            "struct" => Some(Self::Struct),
            "enum" => Some(Self::Enum),
            "trait" => Some(Self::Trait),
            "type" => Some(Self::Type),
            "impl" => Some(Self::Impl),
            "if" => Some(Self::If),
            "else" => Some(Self::Else),
            "match" => Some(Self::Match),
            "for" => Some(Self::For),
            "loop" => Some(Self::Loop),
            "continue" => Some(Self::Continue),
            "break" => Some(Self::Break),
            "in" => Some(Self::In),
            "while" => Some(Self::While),
            "mod" => Some(Self::Mod),
            "pub" => Some(Self::Pub),
            "crate" => Some(Self::Crate),
            "self" => Some(Self::SelfLower),
            "Self" => Some(Self::SelfUpper),
            "super" => Some(Self::Super),
            "async" => Some(Self::Async),
            "await" => Some(Self::Await),
            "unsafe" => Some(Self::Unsafe),
            "extern" => Some(Self::Extern),
            _ => None,
        }
    }
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Self {
            kind,
            position: SourcePosition::new(line, column),
        }
    }

    pub fn literal(literal: Literal, line: usize, column: usize) -> Self {
        Self::new(TokenKind::Literal(literal), line, column)
    }

    pub fn ident(name: String, line: usize, column: usize) -> Self {
        Self::new(TokenKind::Ident(name), line, column)
    }

    pub fn keyword(keyword: Keyword, line: usize, column: usize) -> Self {
        Self::new(TokenKind::Keyword(keyword), line, column)
    }

    pub fn delimiter(delimiter: Delimiter, line: usize, column: usize) -> Self {
        Self::new(TokenKind::Delimiter(delimiter), line, column)
    }

    pub fn operator(operator: Operator, line: usize, column: usize) -> Self {
        Self::new(TokenKind::Operator(operator), line, column)
    }

    pub fn eof(line: usize, column: usize) -> Self {
        Self::new(TokenKind::Eof, line, column)
    }
}

impl TokenKind {
    pub fn is_eof(&self) -> bool {
        matches!(self, TokenKind::Eof)
    }

    pub fn is_keyword(&self, keyword: Keyword) -> bool {
        matches!(self, TokenKind::Keyword(k) if *k == keyword)
    }

    pub fn is_operator(&self, operator: Operator) -> bool {
        matches!(self, TokenKind::Operator(op) if *op == operator)
    }

    pub fn is_delimiter(&self, delimiter: Delimiter) -> bool {
        matches!(self, TokenKind::Delimiter(d) if *d == delimiter)
    }
}
