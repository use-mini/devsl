use devsl::lexer::{LexError, Lexer, Token};

fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    Lexer::new(src).lex()
}

#[test]
fn hello_world() {
    insta::assert_debug_snapshot!(lex(r#"print("Hello, World!");"#));
}

mod numbers {
    use crate::lex;

    #[test]
    fn int_basic() {
        insta::assert_debug_snapshot!(lex("42"));
    }
    #[test]
    fn int_zero() {
        insta::assert_debug_snapshot!(lex("0"));
    }
    #[test]
    fn int_underscores() {
        insta::assert_debug_snapshot!(lex("1_000_000"));
    }
    #[test]
    fn int_overflow() {
        insta::assert_debug_snapshot!(lex("99999999999999999999"));
    }

    #[test]
    fn float_basic() {
        insta::assert_debug_snapshot!(lex("3.14"));
    }
    #[test]
    fn float_leading_dot() {
        insta::assert_debug_snapshot!(lex(".5"));
    }
    #[test]
    fn float_trailing_dot() {
        insta::assert_debug_snapshot!(lex("1."));
    }
    #[test]
    fn float_exponent() {
        insta::assert_debug_snapshot!(lex("1e10"));
    }
    #[test]
    fn float_exponent_caps() {
        insta::assert_debug_snapshot!(lex("1E10"));
    }
    #[test]
    fn float_exp_neg() {
        insta::assert_debug_snapshot!(lex("2.5e-3"));
    }
    #[test]
    fn float_exp_pos() {
        insta::assert_debug_snapshot!(lex("2.5e+3"));
    }
    #[test]
    fn float_with_underscore() {
        insta::assert_debug_snapshot!(lex("1_000.5"));
    }
    #[test]
    fn float_dot_exp() {
        insta::assert_debug_snapshot!(lex(".5e2"));
    }

    #[test]
    fn multi_sequence() {
        insta::assert_debug_snapshot!(lex("1 2 3"));
    }
    #[test]
    fn multi_mixed() {
        insta::assert_debug_snapshot!(lex("42 3.14 1e10"));
    }

    mod errors {
        use crate::lex;

        #[test]
        fn multiple_dots() {
            insta::assert_debug_snapshot!(lex("1.2.3"));
        }
        #[test]
        fn double_underscore() {
            insta::assert_debug_snapshot!(lex("1__2"));
        }
        #[test]
        fn underscore_before_dot() {
            insta::assert_debug_snapshot!(lex("1_.5"));
        }
        #[test]
        fn underscore_at_eof() {
            insta::assert_debug_snapshot!(lex("1_"));
        }
        #[test]
        fn trailing_alpha() {
            insta::assert_debug_snapshot!(lex("42abc"));
        }
        #[test]
        fn exponent_eof() {
            insta::assert_debug_snapshot!(lex("1e"));
        }
        #[test]
        fn exponent_sign_eof() {
            insta::assert_debug_snapshot!(lex("1e+"));
        }
        #[test]
        fn exponent_alpha() {
            insta::assert_debug_snapshot!(lex("1e+abc"));
        }
    }
}
