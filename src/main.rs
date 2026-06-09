use compole_principle::parse_source;

/// 这个 `main` 不是项目的重点逻辑，
/// 它更像一个“把前端流程串起来的最小示例”。
///
/// 如果你明天学习时想验证自己的理解，
/// 可以修改这里的源码字符串，然后重新运行程序，
/// 看输出的 AST 是否符合你的预期。
fn main() {
    let source = r#"
        // a tiny end-to-end demo
        let mut answer = 1 + 2 * 3;
        return answer;
    "#;

    // `parse_source` 内部会先进行词法分析，再进行语法分析。
    match parse_source(source) {
        // 成功时打印 AST，方便直接观察语法树长什么样。
        Ok(program) => println!("{program:#?}"),
        // 失败时打印带位置信息的错误。
        Err(err) => eprintln!("frontend error: {err}"),
    }
}
