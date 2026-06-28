use devsl::eval::{Env, EvalCtx, EvalError, eval};
use devsl::lexer::Lexer;
use devsl::parser::Parser;

fn run(src: &str) -> Result<String, EvalError> {
    let tokens = Lexer::new(src).lex().expect("lex failed in eval test");
    let stmts = Parser::new(tokens)
        .parse()
        .expect("parse failed in eval test");
    let mut buf = Vec::<u8>::new();
    let mut ctx = EvalCtx {
        env: Env::new(),
        out: &mut buf,
    };
    eval(&stmts, &mut ctx)?;
    Ok(String::from_utf8(buf).expect("utf8"))
}

#[test]
fn hello_world() {
    insta::assert_debug_snapshot!(run(r#"print("Hello, World!")"#));
}

mod print {
    use crate::run;

    #[test]
    fn single_string() {
        insta::assert_debug_snapshot!(run(r#"print("hi")"#));
    }
    #[test]
    fn multiple_strings_space_separated() {
        insta::assert_debug_snapshot!(run(r#"print("a", "b", "c")"#));
    }
    #[test]
    fn no_args_just_newline() {
        insta::assert_debug_snapshot!(run("print()"));
    }
    #[test]
    fn two_statements() {
        insta::assert_debug_snapshot!(run("print(\"first\")\nprint(\"second\")"));
    }
}

mod int_builtin {
    use crate::run;

    #[test]
    fn int_of_int() {
        insta::assert_debug_snapshot!(run(r#"print(int(42))"#));
    }
    #[test]
    fn int_of_float_truncates_toward_zero() {
        insta::assert_debug_snapshot!(run(r#"print(int(3.7))"#));
    }
    #[test]
    fn int_of_negative_float() {
        insta::assert_debug_snapshot!(run(r#"print(int(0 - 3.7))"#));
    }
    #[test]
    fn int_of_string() {
        insta::assert_debug_snapshot!(run(r#"print(int("123"))"#));
    }
    #[test]
    fn int_of_bad_string() {
        insta::assert_debug_snapshot!(run(r#"print(int("abc"))"#));
    }
    #[test]
    fn int_of_float_overflow() {
        insta::assert_debug_snapshot!(run(r#"print(int(99999999999999999999.0))"#));
    }
}

mod float_builtin {
    use crate::run;

    #[test]
    fn float_of_float() {
        insta::assert_debug_snapshot!(run(r#"print(float(3.14))"#));
    }
    #[test]
    fn float_of_int() {
        insta::assert_debug_snapshot!(run(r#"print(float(7))"#));
    }
    #[test]
    fn float_of_string() {
        insta::assert_debug_snapshot!(run(r#"print(float("2.5"))"#));
    }
    #[test]
    fn float_of_bad_string() {
        insta::assert_debug_snapshot!(run(r#"print(float("nope"))"#));
    }
}

mod number_builtin {
    use crate::run;

    #[test]
    fn number_of_int() {
        insta::assert_debug_snapshot!(run(r#"print(number(42))"#));
    }
    #[test]
    fn number_of_float() {
        insta::assert_debug_snapshot!(run(r#"print(number(3.14))"#));
    }
    #[test]
    fn number_of_int_string() {
        insta::assert_debug_snapshot!(run(r#"print(number("42"))"#));
    }
    #[test]
    fn number_of_float_string() {
        insta::assert_debug_snapshot!(run(r#"print(number("3.14"))"#));
    }
    #[test]
    fn number_of_bad_string() {
        insta::assert_debug_snapshot!(run(r#"print(number("abc"))"#));
    }
}

mod string_builtin {
    use crate::run;

    #[test]
    fn string_of_int() {
        insta::assert_debug_snapshot!(run(r#"print(string(42))"#));
    }
    #[test]
    fn string_of_float() {
        insta::assert_debug_snapshot!(run(r#"print(string(3.14))"#));
    }
    #[test]
    fn string_of_string() {
        insta::assert_debug_snapshot!(run(r#"print(string("hi"))"#));
    }
    #[test]
    fn string_wrong_arity_zero() {
        insta::assert_debug_snapshot!(run(r#"print(string())"#));
    }
    #[test]
    fn string_wrong_arity_two() {
        insta::assert_debug_snapshot!(run(r#"print(string(1, 2))"#));
    }
}
