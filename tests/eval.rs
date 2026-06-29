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

#[test]
fn spec_demo() {
    let src = "
        var x = 2 + 3
        const greeting = \"hello\"
        print(greeting, string(x))
    ";
    insta::assert_debug_snapshot!(run(src));
}

#[test]
fn arithmetic_in_var() {
    insta::assert_debug_snapshot!(run("var n = 1 + 2 * 3\nprint(string(n))"));
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

mod rules {
    use crate::run;

    #[test]
    fn cannot_shadow_print() {
        insta::assert_debug_snapshot!(run("var print = 1"));
    }

    #[test]
    fn cannot_shadow_print_with_const() {
        insta::assert_debug_snapshot!(run("const print = 1"));
    }

    #[test]
    fn cannot_bind_print_result() {
        insta::assert_debug_snapshot!(run("var x = print(\"hi\")"));
    }

    #[test]
    fn unknown_identifier() {
        insta::assert_debug_snapshot!(run("print(undefined_thing)"));
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

mod logic {
    use crate::run;

    #[test]
    fn print_true() {
        insta::assert_debug_snapshot!(run("print(true)"));
    }
    #[test]
    fn print_false() {
        insta::assert_debug_snapshot!(run("print(false)"));
    }
    #[test]
    fn print_null() {
        insta::assert_debug_snapshot!(run("print(null)"));
    }
    #[test]
    fn comparison_in_print() {
        insta::assert_debug_snapshot!(run("print(1 < 2)"));
    }
    #[test]
    fn and_or_precedence() {
        insta::assert_debug_snapshot!(run("print(true or false and false)"));
    }
    #[test]
    fn not_then_and() {
        insta::assert_debug_snapshot!(run("print(not false and true)"));
    }
    #[test]
    fn string_of_bool() {
        insta::assert_debug_snapshot!(run("print(string(true))"));
    }
    #[test]
    fn string_of_null() {
        insta::assert_debug_snapshot!(run("print(string(null))"));
    }
    #[test]
    fn or_short_circuit_with_skipped_call() {
        insta::assert_debug_snapshot!(run(r#"print(true or print("rhs"))"#));
    }
    #[test]
    fn and_short_circuit_catches_skipped_name_error() {
        insta::assert_debug_snapshot!(run("print(false and nope)"));
    }
}

mod scoping {
    use crate::run;

    #[test]
    fn reassign_int() {
        insta::assert_debug_snapshot!(run("var x = 1\nx = 2\nprint(x)"));
    }
    #[test]
    fn reassign_across_types() {
        insta::assert_debug_snapshot!(run("var x = 1\nx = \"hi\"\nprint(x)"));
    }
    #[test]
    fn reassign_unknown_fails() {
        insta::assert_debug_snapshot!(run("nope = 1"));
    }
    #[test]
    fn reassign_const_fails() {
        insta::assert_debug_snapshot!(run("const pi = 3.14\npi = 3.0"));
    }
    #[test]
    fn block_shadows_outer() {
        insta::assert_debug_snapshot!(run("var x = 1\n{ var x = 2\n print(x) }\nprint(x)"));
    }
    #[test]
    fn block_inner_var_does_not_leak() {
        insta::assert_debug_snapshot!(run("{ var x = 1 }\nprint(x)"));
    }
    #[test]
    fn block_reassign_affects_outer() {
        insta::assert_debug_snapshot!(run("var x = 1\n{ x = 2 }\nprint(x)"));
    }
    #[test]
    fn nested_blocks_compose() {
        insta::assert_debug_snapshot!(run(
            "var x = 1\n{ var x = 2\n { var x = 3\n print(x) }\n print(x) }\nprint(x)"
        ));
    }
    #[test]
    fn empty_block_is_noop() {
        insta::assert_debug_snapshot!(run("var x = 1\nif true {}\nprint(x)"));
    }
}

mod conditionals {
    use crate::run;

    #[test]
    fn if_true_prints_then() {
        insta::assert_debug_snapshot!(run("if true { print(\"yes\") }"));
    }

    #[test]
    fn if_false_no_else_skips() {
        insta::assert_debug_snapshot!(run("if false { print(\"no\") }"));
    }

    #[test]
    fn if_false_runs_else() {
        insta::assert_debug_snapshot!(run(r#"if false { print("no") } else { print("yes") }"#));
    }

    #[test]
    fn else_if_picks_middle() {
        insta::assert_debug_snapshot!(run(r#"
            var code = 404
            if code >= 500 { print("server") }
            else if code >= 400 { print("client") }
            else { print("ok") }
            "#));
    }

    #[test]
    fn else_if_falls_through_to_else() {
        insta::assert_debug_snapshot!(run(r#"
            var code = 200
            if code >= 500 { print("server") }
            else if code >= 400 { print("client") }
            else { print("ok") }
            "#));
    }

    #[test]
    fn condition_with_and() {
        insta::assert_debug_snapshot!(run(r#"if 1 < 2 and 3 > 2 { print("yes") }"#));
    }

    #[test]
    fn condition_non_bool_is_type_error() {
        insta::assert_debug_snapshot!(run("if 5 { print(\"x\") }"));
    }

    #[test]
    fn nested_if_inner_runs() {
        insta::assert_debug_snapshot!(run(r#"
            if true {
                if true {
                    print("inner")
                }
            }
            "#));
    }

    #[test]
    fn if_block_scope_does_not_leak() {
        insta::assert_debug_snapshot!(run("if true { var x = 1 }\nprint(x)"));
    }
}
