use devsl::lexer::Lexer;
use devsl::parser::{ParseError, Parser, Stmt};

fn parse(src: &str) -> Result<Vec<Stmt>, ParseError> {
    let tokens = Lexer::new(src).lex().expect("lex failed in parser test");
    Parser::new(tokens).parse()
}

#[test]
fn hello_world() {
    insta::assert_debug_snapshot!(parse(r#"print("Hello, World!");"#));
}

mod exprs {
    use crate::parse;

    #[test]
    fn string_literal() {
        insta::assert_debug_snapshot!(parse(r#""hello""#));
    }
    #[test]
    fn bare_identifier() {
        insta::assert_debug_snapshot!(parse("foo"));
    }
    #[test]
    fn call_no_args() {
        insta::assert_debug_snapshot!(parse("f()"));
    }
    #[test]
    fn call_identifier_arg() {
        insta::assert_debug_snapshot!(parse("print(x)"));
    }
    #[test]
    fn call_string_arg() {
        insta::assert_debug_snapshot!(parse(r#"print("hi")"#));
    }
    #[test]
    fn call_multiple_args() {
        insta::assert_debug_snapshot!(parse("f(a, b, c)"));
    }
    #[test]
    fn call_nested() {
        insta::assert_debug_snapshot!(parse("f(g(x))"));
    }
    #[test]
    fn call_mixed_args() {
        insta::assert_debug_snapshot!(parse(r#"f(x, "y", g(z))"#));
    }
}

mod statements {
    use crate::parse;

    #[test]
    fn empty_input() {
        insta::assert_debug_snapshot!(parse(""));
    }
    #[test]
    fn only_newlines() {
        insta::assert_debug_snapshot!(parse("\n\n\n"));
    }
    #[test]
    fn trailing_semicolon() {
        insta::assert_debug_snapshot!(parse("f();"));
    }
    #[test]
    fn no_trailing_terminator() {
        insta::assert_debug_snapshot!(parse("f()"));
    }
    #[test]
    fn semicolon_separated() {
        insta::assert_debug_snapshot!(parse("a();b()"));
    }
    #[test]
    fn newline_separated() {
        insta::assert_debug_snapshot!(parse("a()\nb()"));
    }
    #[test]
    fn blank_lines_between() {
        insta::assert_debug_snapshot!(parse("a()\n\n\nb()"));
    }
}

mod numbers {
    use crate::parse;

    #[test]
    fn int_literal() {
        insta::assert_debug_snapshot!(parse("42"));
    }
    #[test]
    fn float_literal() {
        insta::assert_debug_snapshot!(parse("3.14"));
    }
    #[test]
    fn call_with_int_arg() {
        insta::assert_debug_snapshot!(parse("print(42)"));
    }
    #[test]
    fn call_with_float_arg() {
        insta::assert_debug_snapshot!(parse("print(3.14)"));
    }
}

mod errors {
    use crate::parse;

    #[test]
    fn unclosed_paren_eof() {
        insta::assert_debug_snapshot!(parse("print("));
    }
    #[test]
    fn trailing_comma_eof() {
        insta::assert_debug_snapshot!(parse("print(x,"));
    }
    #[test]
    fn missing_comma() {
        insta::assert_debug_snapshot!(parse("print(x y)"));
    }
    #[test]
    fn leading_comma() {
        insta::assert_debug_snapshot!(parse("print(,)"));
    }
    #[test]
    fn missing_terminator_between() {
        insta::assert_debug_snapshot!(parse("a() b()"));
    }
    #[test]
    fn unexpected_start() {
        insta::assert_debug_snapshot!(parse(","));
    }
    #[test]
    fn eof_after_terminator() {
        insta::assert_debug_snapshot!(parse(";"));
    }
}
