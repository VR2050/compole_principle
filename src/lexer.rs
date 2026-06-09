use std::{error::Error, fmt, iter::Peekable, str::Chars};

use crate::{Delimiter, Keyword, Literal, Operator, SourcePosition, Token};

/// 词法分析阶段的错误类型。
///
/// 如果说 parser 处理的是“语法不合法”，
/// 那么 lexer 处理的通常是“连 token 都切不出来”的情况。
///
/// 比如：
/// - 字符串没有闭合
/// - 字符字面量非法
/// - 出现了未知字符
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

/// `Lexer` 负责把源码字符串切分成 token 序列。
///
/// 这是编译器前端的第一步。
/// 从学习角度看，你可以把它理解为：
/// “先给原始字符做分类，后面的 parser 才更容易工作”。
///
/// 例如源码：
/// `let x = 1 + 2;`
/// lexer 会大致把它变成：
/// `Keyword(Let), Ident("x"), Operator(Assign), Literal(Int(1)), ...`
#[derive(Debug)]
pub struct Lexer<'a> {
    /// 剩余字符流，使用 `Peekable` 是因为词法分析经常需要“偷看下一个字符”。
    chars: Peekable<Chars<'a>>,
    /// 当前正在处理的字符。
    current: Option<char>,
    /// 当前所在行号。
    line: usize,
    /// 当前所在列号。
    column: usize,
}

impl<'a> Lexer<'a> {
    /// 从源码字符串创建 lexer。
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

    /// 读取当前字符，但不前进。
    fn current(&self) -> Option<char> {
        self.current
    }

    /// 偷看下一个字符，但不消耗它。
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// 再往后偷看一个字符。
    ///
    /// 这在处理 `..=` 这种多字符 token 时很有用。
    fn peek_second(&self) -> Option<char> {
        let mut clone = self.chars.clone();
        clone.next()
    }

    /// 向前消费一个字符，并同步更新行列号。
    ///
    /// 这是 lexer 最核心的基础操作之一。
    /// 几乎所有“读取某种 token”的函数，底层都会不断调用它。
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

    /// 获取当前位置。
    fn position(&self) -> SourcePosition {
        SourcePosition::new(self.line, self.column)
    }

    /// 跳过无意义输入。
    ///
    /// 对 parser 来说，空白和注释通常都不参与语法结构，
    /// 因此最常见的做法就是在 lexer 阶段直接吃掉它们。
    ///
    /// 目前支持：
    /// - 空格、换行、制表符等空白
    /// - `//` 单行注释
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

    /// 读取标识符或关键字。
    ///
    /// 这类 token 的共同特征是：
    /// 以字母/下划线开头，后面跟字母、数字或下划线。
    ///
    /// 读完之后还要进一步判断：
    /// - 是普通变量名？
    /// - 还是语言保留关键字？
    /// - 又或者是 `true/false` 这样的布尔字面量？
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

    /// 读取数字字面量。
    ///
    /// 当前支持：
    /// - 整数字面量
    /// - 浮点数字面量
    ///
    /// 这里的判断逻辑比较直接：
    /// 如果中间出现一个 `.`，并且后面还是数字，就按浮点数处理。
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

    /// 读取字符串字面量。
    ///
    /// 这一类逻辑通常是 lexer 里最容易写出边界 bug 的地方之一，
    /// 因为要处理：
    /// - 正常结束 `"..."`
    /// - 转义字符 `\n`、`\"`
    /// - 一直到文件结束都没闭合
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

    /// 读取字符字面量。
    ///
    /// 与字符串不同，字符字面量必须且只能包含一个字符，
    /// 所以这里会做更严格的长度/闭合校验。
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

    /// 读取运算符。
    ///
    /// 这是 lexer 里另一个非常典型的“最长匹配”场景：
    /// 例如看到 `.` 时，要继续看它是不是 `..` 或 `..=`；
    /// 看到 `<` 时，要继续看它是不是 `<=` 或 `<<`。
    ///
    /// 工业级 lexer 往往也会遵循类似思路，只是实现会更系统化。
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

    /// 读取分隔符。
    ///
    /// 分隔符和运算符的区别并不在“长得像不像符号”，
    /// 而在于它在语言设计里的职责不同。
    ///
    /// 比如：
    /// - `+` 会参与计算
    /// - `;` 更像是语句边界标记
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

    /// 读取下一个 token。
    ///
    /// 推荐你明天学习 lexer 时重点看这个函数，
    /// 因为它体现了词法分析器最关键的“总分派”思路：
    /// 先观察当前字符，再决定调用哪个专门的读取函数。
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

    /// 把整个输入一次性切分成 token 列表。
    ///
    /// 这是最适合教学和调试的高层接口。
    /// 它会不断调用 `next_token()`，直到遇到 EOF。
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
    // 这个测试覆盖了两个学习点：
    // 1. lexer 能否跳过单行注释
    // 2. lexer 能否正确识别 `..=` 这样的多字符运算符
    fn lexes_comments_and_range_operator() {
        let tokens = Lexer::new("let a = 1..=3; // range").tokenize().unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::Keyword(_)));
        assert!(matches!(tokens[3].kind, TokenKind::Literal(Literal::Int(1))));
        assert!(matches!(tokens[4].kind, TokenKind::Operator(Operator::RangeEq)));
    }

    #[test]
    // 失败测试和成功测试一样重要。
    // 真实编译器前端最有价值的能力之一，就是“把错误定位并讲清楚”。
    fn reports_unterminated_string() {
        let err = Lexer::new("\"hello").tokenize().unwrap_err();
        assert!(err.message.contains("unterminated string"));
    }
}
