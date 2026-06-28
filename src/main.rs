use std::io::{self, Write};
use std::process::ExitCode;

use devsl::eval::{Env, EvalCtx, eval};
use devsl::lexer::Lexer;
use devsl::parser::Parser;

fn main() -> ExitCode {
    let src = r#"
        var x = 2 + 3
        const greeting = "hello"
        print(greeting, string(x))
    "#;

    let tokens = match Lexer::new(src).lex() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("lex error: {e:?}");
            return ExitCode::from(2);
        }
    };

    let stmts = match Parser::new(tokens).parse() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("parse error: {e:?}");
            return ExitCode::from(2);
        }
    };

    let stdout = io::stdout();
    let mut lock = stdout.lock();
    let mut ctx = EvalCtx {
        env: Env::new(),
        out: &mut lock,
    };
    if let Err(e) = eval(&stmts, &mut ctx) {
        let _ = lock.flush();
        eprintln!("eval error: {e:?}");
        return ExitCode::from(1);
    }
    let _ = lock.flush();
    ExitCode::SUCCESS
}
