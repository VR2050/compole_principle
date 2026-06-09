//! 这个 crate 目前实现的是一个“小型编译器前端”的核心骨架。
//!
//! 你可以把它理解为一条流水线：
//! 1. `lexer` 把源码字符串切分成 token
//! 2. `parser` 把 token 组织成 AST
//! 3. `parse_source` 把这两步串起来，提供统一入口
//!
//! 如果你明天要按“学习顺序”阅读，建议这样看：
//! 1. 先看 `token.rs`
//!    目标：理解 token 是什么、AST 后面会消费什么输入
//! 2. 再看 `lexer.rs`
//!    目标：理解“字符流”是怎么变成“token 流”的
//! 3. 然后看 `parser.rs`
//!    目标：理解“token 流”是怎么变成“抽象语法树”的
//! 4. 最后看 `main.rs`
//!    目标：把整条调用链串起来

pub mod lexer;
pub mod parser;
pub mod token;

/// 对外重导出 lexer，方便使用者直接从 crate 根路径访问。
pub use lexer::{LexError, Lexer};
/// 对外重导出 parser/AST 相关类型。
pub use parser::{Expr, ParseError, Parser, Program, Stmt};
/// 对外重导出词法与语法共享的基础类型。
pub use token::{Delimiter, Keyword, Literal, Operator, SourcePosition, Token, TokenKind};

/// `FrontendError` 是整个前端阶段的统一错误类型。
///
/// 这样做的好处是：调用方不需要分别处理 lexer 和 parser 的错误来源，
/// 只需要面对一个统一的“前端失败”概念即可。
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

/// 最适合教学和调用的统一入口。
///
/// 它把“词法分析 + 语法分析”这两个阶段串起来，
/// 因此你只需要提供源码字符串，就能直接拿到 AST。
///
/// 这也是很多真实编译器/解释器 API 的常见设计方式：
/// 对外暴露一个高层入口，对内再拆分多个阶段。
pub fn parse_source(source: &str) -> Result<Program, FrontendError> {
    let tokens = Lexer::new(source).tokenize()?;
    Ok(Parser::parse(tokens)?)
}
