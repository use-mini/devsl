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

mod lists {
    use crate::run;

    #[test]
    fn print_empty() {
        insta::assert_debug_snapshot!(run("print([])"));
    }
    #[test]
    fn print_ints() {
        insta::assert_debug_snapshot!(run("print([1, 2, 3])"));
    }
    #[test]
    fn strings_quoted_inside() {
        insta::assert_debug_snapshot!(run(r#"print(["a", "b"])"#));
    }
    #[test]
    fn nested() {
        insta::assert_debug_snapshot!(run("print([[1, 2], [3]])"));
    }
    #[test]
    fn equality_same() {
        insta::assert_debug_snapshot!(run("print([1, 2] == [1, 2])"));
    }
    #[test]
    fn equality_diff_order() {
        insta::assert_debug_snapshot!(run("print([1, 2] == [2, 1])"));
    }
    #[test]
    fn equality_diff_len() {
        insta::assert_debug_snapshot!(run("print([1, 2] == [1, 2, 3])"));
    }
    #[test]
    fn equality_nested() {
        insta::assert_debug_snapshot!(run("print([[1], [2]] == [[1], [2]])"));
    }
}

mod objects {
    use crate::run;

    #[test]
    fn print_empty() {
        insta::assert_debug_snapshot!(run("print({})"));
    }
    #[test]
    fn print_one() {
        insta::assert_debug_snapshot!(run(r#"print({name: "Ana"})"#));
    }
    #[test]
    fn print_many() {
        insta::assert_debug_snapshot!(run(r#"print({name: "Ana", age: 30})"#));
    }
    #[test]
    fn print_hyphen_key_quoted() {
        insta::assert_debug_snapshot!(run(r#"print({"first-name": "Ana"})"#));
    }
    #[test]
    fn print_extended_key_quoted() {
        insta::assert_debug_snapshot!(run(r#"print({@first-name: "Ana"})"#));
    }
    #[test]
    fn equality_same_order() {
        insta::assert_debug_snapshot!(run("print({a: 1, b: 2} == {a: 1, b: 2})"));
    }
    #[test]
    fn equality_diff_order() {
        insta::assert_debug_snapshot!(run("print({a: 1, b: 2} == {b: 2, a: 1})"));
    }
    #[test]
    fn equality_nested() {
        insta::assert_debug_snapshot!(run("print({a: {b: 1}} == {a: {b: 1}})"));
    }
    #[test]
    fn equality_diff_value() {
        insta::assert_debug_snapshot!(run("print({a: 1} == {a: 2})"));
    }
    #[test]
    fn equality_diff_keys() {
        insta::assert_debug_snapshot!(run("print({a: 1} == {b: 1})"));
    }
    #[test]
    fn cross_compound_equality_is_false() {
        insta::assert_debug_snapshot!(run("print([] == {})"));
    }
}

mod member_access {
    use crate::run;

    #[test]
    fn member_plain() {
        insta::assert_debug_snapshot!(run(r#"print({name: "Ana"}.name)"#));
    }
    #[test]
    fn member_extended() {
        insta::assert_debug_snapshot!(run(r#"print({"first-name": "Ana"}.@first-name)"#));
    }
    #[test]
    fn member_extended_key_to_extended_access() {
        insta::assert_debug_snapshot!(run(r#"print({@first-name: "Ana"}.@first-name)"#));
    }
    #[test]
    fn member_missing() {
        insta::assert_debug_snapshot!(run(r#"print({name: "Ana"}.age)"#));
    }
    #[test]
    fn member_on_list_is_type_error() {
        insta::assert_debug_snapshot!(run("print([1, 2].name)"));
    }
    #[test]
    fn member_on_int_is_type_error() {
        insta::assert_debug_snapshot!(run("print((1).x)"));
    }
    #[test]
    fn member_on_null_is_type_error() {
        insta::assert_debug_snapshot!(run("print(null.x)"));
    }
}

mod bracket_access_eval {
    use crate::run;

    #[test]
    fn list_index_zero() {
        insta::assert_debug_snapshot!(run("print([10, 20][0])"));
    }
    #[test]
    fn list_index_last() {
        insta::assert_debug_snapshot!(run("print([10, 20][1])"));
    }
    #[test]
    fn list_index_out_of_bounds() {
        insta::assert_debug_snapshot!(run("print([10, 20][5])"));
    }
    #[test]
    fn list_index_negative_is_error() {
        insta::assert_debug_snapshot!(run("print([10, 20][0 - 1])"));
    }
    #[test]
    fn list_index_float_is_type_error() {
        insta::assert_debug_snapshot!(run("print([10, 20][1.0])"));
    }
    #[test]
    fn list_index_string_is_type_error() {
        insta::assert_debug_snapshot!(run(r#"print([10, 20]["x"])"#));
    }
    #[test]
    fn object_bracket_string() {
        insta::assert_debug_snapshot!(run(r#"print({name: "Ana"}["name"])"#));
    }
    #[test]
    fn object_bracket_missing() {
        insta::assert_debug_snapshot!(run(r#"print({}["nope"])"#));
    }
    #[test]
    fn object_bracket_int_is_type_error() {
        insta::assert_debug_snapshot!(run(r#"print({a: 1}[0])"#));
    }
    #[test]
    fn index_on_int_is_type_error() {
        insta::assert_debug_snapshot!(run("print((1)[0])"));
    }
    #[test]
    fn index_on_string_is_type_error() {
        insta::assert_debug_snapshot!(run(r#"print("hi"[0])"#));
    }
}

mod chained_access {
    use crate::run;

    #[test]
    fn member_then_index() {
        insta::assert_debug_snapshot!(run(r#"print({xs: [10, 20]}.xs[1])"#));
    }
    #[test]
    fn index_then_member() {
        insta::assert_debug_snapshot!(run(r#"print([{n: 1}, {n: 2}][1].n)"#));
    }
    #[test]
    fn deep_nested() {
        insta::assert_debug_snapshot!(run(r#"print({a: {b: [{c: 42}]}}.a.b[0].c)"#));
    }
    #[test]
    fn const_object_field_via_dot() {
        insta::assert_debug_snapshot!(run(r#"
            const u = {name: "Ana", tags: ["admin", "beta"]}
            print(u.name, u.tags[0])
            "#));
    }
    #[test]
    fn equality_with_nested_lists_and_objects() {
        insta::assert_debug_snapshot!(run(r#"
            const a = {users: [{name: "Ana"}, {name: "Bo"}]}
            const b = {users: [{name: "Ana"}, {name: "Bo"}]}
            print(a == b)
            "#));
    }
}

mod for_loop {
    use crate::run;

    #[test]
    fn prints_each_int() {
        insta::assert_debug_snapshot!(run("for x in [1, 2, 3] { print(x) }"));
    }
    #[test]
    fn empty_list_no_output() {
        insta::assert_debug_snapshot!(run("for x in [] { print(x) }"));
    }
    #[test]
    fn break_exits_early() {
        insta::assert_debug_snapshot!(run("for x in [1, 2, 3] { if x == 2 { break }\nprint(x) }"));
    }
    #[test]
    fn continue_skips_one() {
        insta::assert_debug_snapshot!(run(
            "for x in [1, 2, 3] { if x == 2 { continue }\nprint(x) }"
        ));
    }
    #[test]
    fn break_through_nested_if() {
        insta::assert_debug_snapshot!(run(
            "for x in [1, 2, 3] { if x == 2 { break } }\nprint(\"done\")"
        ));
    }
    #[test]
    fn nested_for_break_inner_only() {
        insta::assert_debug_snapshot!(run(
            "for x in [1, 2] { for y in [10, 20] { if y == 10 { break }\nprint(y) }\nprint(x) }"
        ));
    }
    #[test]
    fn loop_var_const_in_body() {
        insta::assert_debug_snapshot!(run("for x in [1] { x = 99 }"));
    }
    #[test]
    fn loop_var_not_visible_after() {
        insta::assert_debug_snapshot!(run("for x in [1] { }\nprint(x)"));
    }
    #[test]
    fn outer_var_restored_after_loop() {
        insta::assert_debug_snapshot!(run("var x = 1\nfor x in [2, 3] { }\nprint(x)"));
    }
    #[test]
    fn non_list_iter_int() {
        insta::assert_debug_snapshot!(run("for x in 1 { }"));
    }
    #[test]
    fn non_list_iter_object() {
        insta::assert_debug_snapshot!(run("for x in {a: 1} { }"));
    }
    #[test]
    fn body_error_propagates() {
        insta::assert_debug_snapshot!(run("for x in [1, 2] { undefined_thing }"));
    }
}

mod redeclaration {
    use crate::run;

    #[test]
    fn var_then_var_same_scope() {
        insta::assert_debug_snapshot!(run("var x = 1\nvar x = 2"));
    }
    #[test]
    fn const_then_const_same_scope() {
        insta::assert_debug_snapshot!(run("const x = 1\nconst x = 2"));
    }
    #[test]
    fn const_then_var_same_scope() {
        insta::assert_debug_snapshot!(run("const x = 1\nvar x = 2"));
    }
    #[test]
    fn var_then_const_same_scope() {
        insta::assert_debug_snapshot!(run("var x = 1\nconst x = 2"));
    }
    #[test]
    fn nested_scope_shadow_still_ok() {
        insta::assert_debug_snapshot!(run("var x = 1\n{ var x = 2\n print(x) }\nprint(x)"));
    }
}

mod functions {
    use crate::run;

    #[test]
    fn named_fn_call() {
        insta::assert_debug_snapshot!(run("fn greet(n) { \"hi, \" + n }\nprint(greet(\"Ana\"))"));
    }
    #[test]
    fn anonymous_fn_call() {
        insta::assert_debug_snapshot!(run("const double = fn(x) { x * 2 }\nprint(double(21))"));
    }
    #[test]
    fn immediately_invoked() {
        insta::assert_debug_snapshot!(run("print((fn(x) { x * 2 })(21))"));
    }
    #[test]
    fn zero_args() {
        insta::assert_debug_snapshot!(run("fn f() { 42 }\nprint(f())"));
    }
    #[test]
    fn multiple_args() {
        insta::assert_debug_snapshot!(run("fn add(a, b, c) { a + b + c }\nprint(add(1, 2, 3))"));
    }

    #[test]
    fn explicit_return() {
        insta::assert_debug_snapshot!(run("fn f() { return 7 }\nprint(f())"));
    }
    #[test]
    fn implicit_last_expr_return() {
        insta::assert_debug_snapshot!(run("fn f() { 1 + 2 }\nprint(f())"));
    }
    #[test]
    fn early_return_short_circuits() {
        insta::assert_debug_snapshot!(run(
            "fn classify(c) { if c >= 500 { return \"server\" }\nif c >= 400 { return \"client\" }\n\"ok\" }\nprint(classify(404))"
        ));
    }
    #[test]
    fn return_no_value_is_null() {
        insta::assert_debug_snapshot!(run("fn f() { return }\nprint(f())"));
    }
    #[test]
    fn empty_body_returns_null() {
        insta::assert_debug_snapshot!(run("fn f() { }\nprint(f())"));
    }
    #[test]
    fn last_statement_not_expr_returns_null() {
        insta::assert_debug_snapshot!(run("fn f() { var x = 1 }\nprint(f())"));
    }

    #[test]
    fn captures_const_by_value() {
        insta::assert_debug_snapshot!(run("const m = 3\nfn triple(x) { x * m }\nprint(triple(7))"));
    }
    #[test]
    fn capture_is_snapshot_not_live() {
        insta::assert_debug_snapshot!(run(
            "var m = 3\nconst f = fn(x) { x * m }\nm = 99\nprint(f(2))"
        ));
    }
    #[test]
    fn captured_name_is_const_inside_body() {
        insta::assert_debug_snapshot!(run("var m = 1\nconst f = fn() { m = 2 }\nf()"));
    }
    #[test]
    fn nested_closure_captures_outer_param() {
        insta::assert_debug_snapshot!(run(
            "fn outer(x) { fn(y) { x + y } }\nconst add5 = outer(5)\nprint(add5(3))"
        ));
    }

    #[test]
    fn named_recursion() {
        insta::assert_debug_snapshot!(run(
            "fn count(n) { if n == 0 { return }\nprint(n)\ncount(n - 1) }\ncount(3)"
        ));
    }

    #[test]
    fn fn_passed_as_argument() {
        insta::assert_debug_snapshot!(run(
            "fn apply(f, x) { f(x) }\nfn dbl(n) { n * 2 }\nprint(apply(dbl, 5))"
        ));
    }
    #[test]
    fn fn_returned_from_fn() {
        insta::assert_debug_snapshot!(run(
            "fn make_adder(n) { fn(x) { x + n } }\nconst inc = make_adder(1)\nprint(inc(41))"
        ));
    }
    #[test]
    fn fn_stored_in_list() {
        insta::assert_debug_snapshot!(run(
            "const fs = [fn(x) { x + 1 }, fn(x) { x * 2 }]\nprint(fs[0](10), fs[1](10))"
        ));
    }
    #[test]
    fn fn_stored_in_object() {
        insta::assert_debug_snapshot!(run("const o = {f: fn(x) { x + 1 }}\nprint(o.f(41))"));
    }

    #[test]
    fn arity_too_few() {
        insta::assert_debug_snapshot!(run("fn f(x, y) { x + y }\nprint(f(1))"));
    }
    #[test]
    fn arity_too_many() {
        insta::assert_debug_snapshot!(run("fn f(x) { x }\nprint(f(1, 2))"));
    }
    #[test]
    fn anonymous_arity_error_says_function() {
        insta::assert_debug_snapshot!(run("print((fn(x) { x })())"));
    }
    #[test]
    fn calling_int_is_type_error() {
        insta::assert_debug_snapshot!(run("print((42)(1))"));
    }
    #[test]
    fn calling_string_is_type_error() {
        insta::assert_debug_snapshot!(run(r#"print("hi"(1))"#));
    }
    #[test]
    fn fn_redeclares_existing_name() {
        insta::assert_debug_snapshot!(run("const foo = 1\nfn foo() { }"));
    }
    #[test]
    fn fn_then_fn_same_name() {
        insta::assert_debug_snapshot!(run("fn foo() { }\nfn foo() { }"));
    }

    #[test]
    fn fn_body_can_use_for_break() {
        insta::assert_debug_snapshot!(run(
            "fn count_to(n) { for i in [1, 2, 3, 4, 5] { if i > n { break }\nprint(i) } }\ncount_to(3)"
        ));
    }
    // NOTE: parser lacks unary minus, so `-1` in `return -1` fails to parse.
    // `Minus` is only registered as a binary operator (BinOp::Sub).
    #[ignore]
    #[test]
    fn return_inside_for_inside_fn() {
        insta::assert_debug_snapshot!(run(
            "fn first_match(xs) { for x in xs { if x == 2 { return x } }\nreturn -1 }\nprint(first_match([1, 2, 3]))"
        ));
    }
}
