use std::{error::Error, fmt};

use crate::{Delimiter, Keyword, Literal, Operator, SourcePosition, Token, TokenKind};

// ===========================
// 建议阅读顺序（明天学习可直接照这个来）
// ===========================
//
// 第一轮：只建立整体感觉
// 1. 先看 `Program / Stmt / Expr`
//    目标：知道 AST 长什么样
// 2. 再看 `parse_program`
//    目标：知道“整个程序”是怎么一条条读语句的
// 3. 再看 `parse_stmt`
//    目标：知道 parser 怎么根据当前 token 分派语法规则
//
// 第二轮：专门攻表达式
// 4. 看 `parse_expr`
// 5. 看 `parse_binary_expr`
// 6. 看 `parse_unary`
// 7. 看 `parse_primary`
//
// 这四个函数是表达式解析的主干，尤其是 `parse_binary_expr`，
// 它是理解“运算符优先级为何能正确工作”的关键。
//
// 第三轮：看辅助函数
// 8. 看 `current / advance`
// 9. 看 `match_keyword / expect_*`
//
// 这些工具函数虽然短，但它们决定了整个递归下降解析器的节奏。

/// `Program` 是整棵抽象语法树（AST）的根节点。
///
/// 在真实编译器里，源码通常不会直接被“翻译”成目标代码，
/// 而是先被组织成一棵结构化的语法树。
/// 这棵树会把“源码长什么样”转成“程序结构是什么”。
///
/// 例如：
/// `let a = 1 + 2;`
/// 不再只是一些字符，而会被组织成：
/// 1. 一个 `Let` 语句
/// 2. 变量名叫 `a`
/// 3. 右边的值是一个二元表达式 `1 + 2`
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// 一个程序通常由多条语句组成。
    ///
    /// 这里故意使用 `Vec<Stmt>`，因为从教学角度看，
    /// “程序 = 语句列表”是最容易理解、也最常见的第一步抽象。
    pub statements: Vec<Stmt>,
}

/// `Stmt`（statement）表示“语句”。
///
/// 语句的特点通常是：它本身构成一个完整的执行单元。
/// 比如变量定义、`return`、单独写一行的表达式，都可以看成语句。
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// `let` 变量定义语句。
    ///
    /// 例如：
    /// `let x = 1;`
    /// `let mut total = a + b;`
    Let {
        /// 是否带 `mut`，也就是变量是否可变。
        mutable: bool,
        /// 变量名。
        name: String,
        /// 初始化表达式。
        ///
        /// 注意这里不是字符串，而是 `Expr`。
        /// 这意味着在语法层面，我们已经知道它是一个“表达式结构”，
        /// 而不再只是原始源码片段。
        value: Expr,
    },
    /// `return` 语句。
    ///
    /// 之所以是 `Option<Expr>`，是因为：
    /// `return;` 和 `return expr;` 都合法。
    Return(Option<Expr>),
    /// 单独作为一行出现的表达式语句。
    ///
    /// 例如：
    /// `foo + bar;`
    /// 这类语句在很多语言里是允许存在的。
    Expr(Expr),
}

/// `Expr`（expression）表示“表达式”。
///
/// 表达式的核心特点是：它可以计算出一个值。
/// 比如字面量、变量名、`1 + 2`、`-x`、`(a + b)` 都是表达式。
///
/// 学习解析器时，一个很关键的意识是：
/// “表达式”通常比“语句”更适合长成树。
/// 因为表达式内部经常会发生嵌套。
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// 字面量，例如整数、浮点数、布尔值、字符串等。
    Literal(Literal),
    /// 标识符，例如变量名 `answer`。
    Ident(String),
    /// 一元表达式。
    ///
    /// 例如：
    /// `-x`
    /// `!flag`
    ///
    /// 这里用 `Box<Expr>` 是因为 Rust 需要知道枚举的大小。
    /// 如果直接把 `Expr` 再嵌进 `Expr`，类型会无限递归，无法确定大小。
    Unary {
        op: Operator,
        expr: Box<Expr>,
    },
    /// 二元表达式。
    ///
    /// 例如：
    /// `1 + 2`
    /// `a * b`
    /// `x < y`
    ///
    /// 它天然就是一棵小树：
    /// - 左边一个表达式
    /// - 中间一个运算符
    /// - 右边一个表达式
    Binary {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
    },
    /// 括号表达式。
    ///
    /// 从“求值”角度看，`(1 + 2)` 和 `1 + 2` 结果一样；
    /// 但从“语法结构”角度看，括号非常重要，因为它会影响优先级。
    ///
    /// 这里保留 `Grouping`，是为了把这种结构信息清晰地留在 AST 中。
    Grouping(Box<Expr>),
}

/// 解析错误。
///
/// 和教学代码里常见的 `panic!` 不同，
/// 这里用结构化错误保存“错误信息 + 源码位置”，
/// 这样更接近真实编译器前端的设计。
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

/// `Parser` 接收一串 token，并把它们组织成 AST。
///
/// 可以把它理解为：
/// 1. lexer 负责回答“这里有哪些词”
/// 2. parser 负责回答“这些词组合成了什么语法结构”
pub struct Parser {
    /// 词法分析阶段产出的 token 序列。
    tokens: Vec<Token>,
    /// 当前读取到哪个 token。
    ///
    /// 这是一个非常经典的“游标”设计。
    /// 很多手写递归下降解析器都会这样做。
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseError> {
        Self::new(tokens).parse_program()
    }

    /// 解析整个程序。
    ///
    /// 它的思路很朴素：
    /// 只要还没到 EOF，就不断读取下一条语句。
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while !self.current().kind.is_eof() {
            statements.push(self.parse_stmt()?);
        }

        Ok(Program { statements })
    }

    /// 根据当前 token 的种类，决定应该进入哪一种语句解析函数。
    ///
    /// 这就是递归下降解析器最常见的“分派入口”之一：
    /// 先看当前 token，再决定后续使用哪条语法规则。
    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match &self.current().kind {
            TokenKind::Keyword(Keyword::Let) => self.parse_let_stmt(),
            TokenKind::Keyword(Keyword::Return) => self.parse_return_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    /// 解析 `let` 语句。
    ///
    /// 目前支持的形式是：
    /// `let name = expr;`
    /// `let mut name = expr;`
    fn parse_let_stmt(&mut self) -> Result<Stmt, ParseError> {
        // 当前 token 一定是 `let`，先把它吃掉。
        self.advance();

        // `mut` 是可选的，因此这里不是 `expect_*`，
        // 而是“如果匹配就消费，否则跳过”。
        let mutable = self.match_keyword(Keyword::Mut);
        let name = self.expect_ident("expected identifier after `let`")?;

        // 变量定义里必须出现 `=`。
        self.expect_operator(Operator::Assign)?;
        let value = self.parse_expr()?;
        // 当前这门小语言要求每条语句都以分号结束。
        self.expect_delimiter(Delimiter::Semicolon)?;

        Ok(Stmt::Let {
            mutable,
            name,
            value,
        })
    }

    /// 解析 `return` 语句。
    ///
    /// 这里要特别注意两种情况：
    /// 1. `return;`
    /// 2. `return expr;`
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

    /// 解析表达式语句。
    ///
    /// 这种写法的关键点是：
    /// 先按表达式解析，再要求后面必须跟分号。
    fn parse_expr_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expr()?;
        self.expect_delimiter(Delimiter::Semicolon)?;
        Ok(Stmt::Expr(expr))
    }

    /// 表达式总入口。
    ///
    /// 这里没有直接写死“先解析加法，再解析乘法，再解析一元”这种链式结构，
    /// 而是交给 `parse_binary_expr` 做统一的优先级处理。
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary_expr(1)
    }

    // Pratt-style precedence climbing keeps the teaching surface small
    // while still matching how many production parsers handle expressions.
    //
    // 这里补充一段更偏中文的解释：
    //
    // `parse_binary_expr(min_prec)` 的意思可以理解为：
    // “请帮我解析一个优先级至少为 `min_prec` 的二元表达式”。
    //
    // 为什么它能正确处理 `1 + 2 * 3`？
    // 因为当看到 `+` 以后，右边会以更高的最小优先级继续解析，
    // 于是 `2 * 3` 会先被结合成一个整体，再作为 `+` 的右子树。
    //
    // 最终树形结构就会变成：
    //       (+)
    //      /   \
    //    1      (*)
    //          /   \
    //         2     3
    fn parse_binary_expr(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        // 先解析出“左侧的起点表达式”。
        // 它可能是字面量、标识符、括号表达式，或者一元表达式。
        let mut left = self.parse_unary()?;

        while let TokenKind::Operator(op) = &self.current().kind {
            let op = *op;
            let prec = Self::operator_precedence(&op);

            // 如果当前运算符优先级不够高，就说明它不属于这一层，
            // 应该交给外层调用者去处理。
            if prec < min_prec || prec == 0 {
                break;
            }

            // 消费当前运算符，然后递归解析右侧表达式。
            self.advance();
            let right = self.parse_binary_expr(prec + 1)?;

            // 把“旧 left + 当前运算符 + 新 right”重新折叠成一棵更大的树。
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// 解析一元表达式。
    ///
    /// 一元表达式的优先级通常高于二元表达式，
    /// 所以这里会先于 `parse_primary` 进行处理。
    ///
    /// 例如：
    /// `-1 + 2`
    /// 实际应理解为：
    /// `(-1) + 2`
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

    /// 解析“最基础、不可再拆”的表达式单元。
    ///
    /// 在很多解析器教材里，这一层通常叫 `primary`：
    /// - 字面量
    /// - 标识符
    /// - 括号包起来的表达式
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

    /// 定义运算符优先级。
    ///
    /// 数字越大，优先级越高。
    ///
    /// 例如：
    /// - `*` 比 `+` 优先级高
    /// - `&&` 比 `||` 优先级高
    ///
    /// 这张表是表达式解析正确与否的关键之一。
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

    /// 获取当前 token。
    ///
    /// 这里用了一个比较稳妥的写法：
    /// 即使 `pos` 因为逻辑问题超界，也会被钳制到最后一个 token。
    /// 在本项目里，最后一个 token 应该始终是 EOF。
    fn current(&self) -> &Token {
        let idx = self.pos.min(self.tokens.len().saturating_sub(1));
        &self.tokens[idx]
    }

    /// 向前移动一个 token。
    ///
    /// 遇到 EOF 时不再继续前进，这样可以避免很多边界错误。
    fn advance(&mut self) {
        if !self.current().kind.is_eof() {
            self.pos += 1;
        }
    }

    /// 如果当前 token 是指定关键字，就消费它并返回 `true`；
    /// 否则什么都不做，返回 `false`。
    ///
    /// 这种“match + consume”的小工具在手写解析器里非常常见。
    fn match_keyword(&mut self, keyword: Keyword) -> bool {
        if self.current().kind.is_keyword(keyword) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// 断言当前 token 必须是标识符。
    ///
    /// 如果是，就返回它的名字；
    /// 如果不是，就构造一个带位置的语法错误。
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

    /// 断言当前 token 必须是某个运算符。
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

    /// 断言当前 token 必须是某个分隔符。
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
    // 这个测试非常适合学习“优先级是否正确”。
    //
    // 如果解析器把 `1 + 2 * 3` 错误地解析成 `(1 + 2) * 3`，
    // 那么 AST 的树形结构就会完全不同。
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
    // 这个测试覆盖 `return expr;` 的最基本路径。
    fn parses_return_stmt() {
        let program = parse_source("return 42;").expect("frontend should succeed");
        assert_eq!(
            program.statements,
            vec![Stmt::Return(Some(Expr::Literal(Literal::Int(42))))]
        );
    }

    #[test]
    // 学习编译器前端时，不要只看“成功输入”。
    // 失败路径同样重要，因为真实用户一定会写错代码。
    fn reports_missing_semicolon() {
        let err = parse_source("let value = 1").unwrap_err();
        assert!(format!("{err}").contains("expected delimiter Semicolon"));
    }
}
