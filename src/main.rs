use compole_principle::parse_source;

fn main() {
    let source = r#"
        // a tiny end-to-end demo
        let mut answer = 1 + 2 * 3;
        return answer;
    "#;

    match parse_source(source) {
        Ok(program) => println!("{program:#?}"),
        Err(err) => eprintln!("frontend error: {err}"),
    }
}
