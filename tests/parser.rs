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

mod arithmetic {
    use crate::parse;

    #[test]
    fn add() {
        insta::assert_debug_snapshot!(parse("1 + 2"));
    }
    #[test]
    fn sub() {
        insta::assert_debug_snapshot!(parse("5 - 3"));
    }
    #[test]
    fn mul() {
        insta::assert_debug_snapshot!(parse("4 * 6"));
    }
    #[test]
    fn div() {
        insta::assert_debug_snapshot!(parse("10 / 2"));
    }
    #[test]
    fn precedence_mul_over_add() {
        insta::assert_debug_snapshot!(parse("1 + 2 * 3"));
    }
    #[test]
    fn left_assoc_add() {
        insta::assert_debug_snapshot!(parse("1 + 2 + 3"));
    }
    #[test]
    fn left_assoc_div() {
        insta::assert_debug_snapshot!(parse("8 / 4 / 2"));
    }
    #[test]
    fn parens_grouping() {
        insta::assert_debug_snapshot!(parse("(1 + 2) * 3"));
    }
    #[test]
    fn mixed_with_floats() {
        insta::assert_debug_snapshot!(parse("1.5 + 2"));
    }
    #[test]
    fn call_in_expr() {
        insta::assert_debug_snapshot!(parse("f(1) + 2"));
    }
}

mod bindings {
    use crate::parse;

    #[test]
    fn var_simple() {
        insta::assert_debug_snapshot!(parse("var x = 5"));
    }
    #[test]
    fn var_expression() {
        insta::assert_debug_snapshot!(parse("var x = 2 + 3"));
    }
    #[test]
    fn var_string() {
        insta::assert_debug_snapshot!(parse(r#"var name = "alice""#));
    }
    #[test]
    fn const_simple() {
        insta::assert_debug_snapshot!(parse("const pi = 3.14"));
    }
    #[test]
    fn var_then_use() {
        insta::assert_debug_snapshot!(parse("var x = 1\nprint(x)"));
    }
    #[test]
    fn var_missing_equals() {
        insta::assert_debug_snapshot!(parse("var x 5"));
    }
    #[test]
    fn var_missing_name() {
        insta::assert_debug_snapshot!(parse("var = 5"));
    }
    #[test]
    fn var_missing_value() {
        insta::assert_debug_snapshot!(parse("var x ="));
    }
}

mod bools_and_null {
    use crate::parse;

    #[test]
    fn bool_true() {
        insta::assert_debug_snapshot!(parse("true"));
    }
    #[test]
    fn bool_false() {
        insta::assert_debug_snapshot!(parse("false"));
    }
    #[test]
    fn null_literal() {
        insta::assert_debug_snapshot!(parse("null"));
    }
}

mod comparisons {
    use crate::parse;

    #[test]
    fn lt() {
        insta::assert_debug_snapshot!(parse("1 < 2"));
    }
    #[test]
    fn gt() {
        insta::assert_debug_snapshot!(parse("3 > 1"));
    }
    #[test]
    fn lt_eq() {
        insta::assert_debug_snapshot!(parse("1 <= 1"));
    }
    #[test]
    fn gt_eq() {
        insta::assert_debug_snapshot!(parse("2 >= 2"));
    }
    #[test]
    fn eq() {
        insta::assert_debug_snapshot!(parse("1 == 1"));
    }
    #[test]
    fn not_eq() {
        insta::assert_debug_snapshot!(parse("1 != 2"));
    }
    #[test]
    fn precedence_arith_in_cmp() {
        insta::assert_debug_snapshot!(parse("1 + 2 < 3 * 4"));
    }
    #[test]
    fn non_associative_chain() {
        insta::assert_debug_snapshot!(parse("1 < 2 < 3"));
    }
}

mod logical_binary {
    use crate::parse;

    #[test]
    fn and_basic() {
        insta::assert_debug_snapshot!(parse("true and false"));
    }
    #[test]
    fn or_basic() {
        insta::assert_debug_snapshot!(parse("true or false"));
    }
    #[test]
    fn and_binds_tighter_than_or() {
        insta::assert_debug_snapshot!(parse("true or false and true"));
    }
    #[test]
    fn comparison_in_logical() {
        insta::assert_debug_snapshot!(parse("1 < 2 and 3 == 3"));
    }
}

mod logical_not {
    use crate::parse;

    #[test]
    fn not_bool() {
        insta::assert_debug_snapshot!(parse("not true"));
    }
    #[test]
    fn not_call() {
        insta::assert_debug_snapshot!(parse("not f(x)"));
    }
    #[test]
    fn double_not() {
        insta::assert_debug_snapshot!(parse("not not true"));
    }
    #[test]
    fn not_then_comparison() {
        insta::assert_debug_snapshot!(parse("not 1 < 2"));
    }
    #[test]
    fn not_in_and() {
        insta::assert_debug_snapshot!(parse("not true and false"));
    }
}

mod blocks {
    use crate::parse;

    #[test]
    fn empty_block() {
        insta::assert_debug_snapshot!(parse("{}"));
    }
    #[test]
    fn block_with_one_stmt() {
        insta::assert_debug_snapshot!(parse(r#"{ print("hi") }"#));
    }
    #[test]
    fn block_with_multiple_stmts() {
        insta::assert_debug_snapshot!(parse("{ var x = 1\n print(x) }"));
    }
    #[test]
    fn nested_blocks() {
        insta::assert_debug_snapshot!(parse("{ { var x = 1 } }"));
    }
    #[test]
    fn unclosed_block() {
        insta::assert_debug_snapshot!(parse("{ var x = 1"));
    }
    #[test]
    fn block_with_blank_lines() {
        insta::assert_debug_snapshot!(parse("{\n\nvar x = 1\n\n}"));
    }
}

mod reassign {
    use crate::parse;

    #[test]
    fn reassign_int() {
        insta::assert_debug_snapshot!(parse("x = 5"));
    }
    #[test]
    fn reassign_expression() {
        insta::assert_debug_snapshot!(parse("x = 1 + 2"));
    }
    #[test]
    fn reassign_string() {
        insta::assert_debug_snapshot!(parse(r#"x = "hello""#));
    }
    #[test]
    fn reassign_then_use() {
        insta::assert_debug_snapshot!(parse("x = 5\nprint(x)"));
    }
    #[test]
    fn reassign_missing_value() {
        insta::assert_debug_snapshot!(parse("x ="));
    }
    #[test]
    fn bare_identifier_still_works() {
        insta::assert_debug_snapshot!(parse("x"));
    }
    #[test]
    fn call_still_works() {
        insta::assert_debug_snapshot!(parse("x()"));
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
