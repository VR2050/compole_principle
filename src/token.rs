/// `SourcePosition` 用来描述源码中的一个位置。
///
/// 目前我们只记录“行号 + 列号”，这已经足够完成大部分教学用途的报错。
/// 更成熟的编译器里，往往还会进一步记录文件名、偏移量、span（起止区间）等信息。
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

/// `Token` 是词法分析器输出的最小语法单元。
///
/// 可以把它理解为“已经被分类过的词”。
/// 比如源码：
/// `let answer = 42;`
/// 在 lexer 看来，大概会变成：
/// - `let` -> 关键字
/// - `answer` -> 标识符
/// - `=` -> 运算符
/// - `42` -> 字面量
/// - `;` -> 分隔符
///
/// parser 后面就是基于这些 token，而不是基于原始字符，继续工作。
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// token 的具体种类。
    pub kind: TokenKind,
    /// 这个 token 在源码中的起始位置。
    pub position: SourcePosition,
}

/// token 的类别。
///
/// 这一层非常重要，因为它决定了解析器“看待源码”的方式。
/// 对 parser 来说，它不关心原始字符是不是 `'l' 'e' 't'`，
/// 它只关心“当前是不是一个 `Keyword(Let)`”。
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword(Keyword),
    Ident(String),
    Literal(Literal),
    Operator(Operator),
    Delimiter(Delimiter),
    Eof,
}

/// 关键字枚举。
///
/// 关键字和标识符最大的区别在于：
/// 关键字是语言保留的，不能被普通变量名随意占用。
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

/// 字面量枚举。
///
/// 字面量是源码里“直接写出来的值”，比如：
/// - `123`
/// - `3.14`
/// - `"hello"`
/// - `true`
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Char(char),
    Float(f64),
    Int(i64),
    Str(String),
    Bool(bool),
}

/// 运算符枚举。
///
/// 这里既包含算术运算符，也包含比较、逻辑、位运算、区间等运算符。
/// 之所以把它们统一建模成一个枚举，是因为 parser 在处理表达式时，
/// 需要频繁根据“当前是不是某种运算符”来做决策。
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

/// 分隔符枚举。
///
/// 分隔符通常不直接参与“运算”，但它们对语法结构非常重要。
/// 比如：
/// - `(` `)` 用来形成分组
/// - `{` `}` 常用于代码块
/// - `;` 表示语句结束
/// - `:` `::` 常在类型或路径语法中出现
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
    /// 根据字符串形式映射到运算符枚举。
    ///
    /// 这个函数目前主要是一个“词法知识表”，
    /// 在后续如果你做调试工具、pretty-printer 或测试辅助时会很方便。
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
    /// 根据字符串形式映射到分隔符枚举。
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
    /// 判断某个标识符文本是否其实是关键字。
    ///
    /// lexer 在读完一段“字母/数字/下划线序列”之后，
    /// 会先得到一个字符串，然后通过这里决定：
    /// 它到底是关键字，还是普通标识符。
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
    /// 通用构造函数。
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Self {
            kind,
            position: SourcePosition::new(line, column),
        }
    }

    /// 构造字面量 token。
    pub fn literal(literal: Literal, line: usize, column: usize) -> Self {
        Self::new(TokenKind::Literal(literal), line, column)
    }

    /// 构造标识符 token。
    pub fn ident(name: String, line: usize, column: usize) -> Self {
        Self::new(TokenKind::Ident(name), line, column)
    }

    /// 构造关键字 token。
    pub fn keyword(keyword: Keyword, line: usize, column: usize) -> Self {
        Self::new(TokenKind::Keyword(keyword), line, column)
    }

    /// 构造分隔符 token。
    pub fn delimiter(delimiter: Delimiter, line: usize, column: usize) -> Self {
        Self::new(TokenKind::Delimiter(delimiter), line, column)
    }

    /// 构造运算符 token。
    pub fn operator(operator: Operator, line: usize, column: usize) -> Self {
        Self::new(TokenKind::Operator(operator), line, column)
    }

    /// 构造 EOF（文件结束）token。
    ///
    /// 这是很多手写解析器里非常有用的技巧：
    /// 给输入流人为补一个“结束标记”，
    /// 这样 parser 可以更统一地处理边界情况。
    pub fn eof(line: usize, column: usize) -> Self {
        Self::new(TokenKind::Eof, line, column)
    }
}

impl TokenKind {
    /// 当前 token 是否为 EOF。
    pub fn is_eof(&self) -> bool {
        matches!(self, TokenKind::Eof)
    }

    /// 当前 token 是否为指定关键字。
    pub fn is_keyword(&self, keyword: Keyword) -> bool {
        matches!(self, TokenKind::Keyword(k) if *k == keyword)
    }

    /// 当前 token 是否为指定运算符。
    pub fn is_operator(&self, operator: Operator) -> bool {
        matches!(self, TokenKind::Operator(op) if *op == operator)
    }

    /// 当前 token 是否为指定分隔符。
    pub fn is_delimiter(&self, delimiter: Delimiter) -> bool {
        matches!(self, TokenKind::Delimiter(d) if *d == delimiter)
    }
}
