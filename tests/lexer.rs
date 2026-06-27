use devsl::lexer::Lexer;

#[test]
fn hello_world() {
    let mut lex = Lexer::new(r#"print("Hello, World!");"#);
    let tokens = lex.lex().unwrap();
    insta::assert_debug_snapshot!(tokens);
}
