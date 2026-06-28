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
