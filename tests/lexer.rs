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

mod operators {
    use crate::lex;

    #[test]
    fn plus() {
        insta::assert_debug_snapshot!(lex("+"));
    }
    #[test]
    fn minus() {
        insta::assert_debug_snapshot!(lex("-"));
    }
    #[test]
    fn star() {
        insta::assert_debug_snapshot!(lex("*"));
    }
    #[test]
    fn slash() {
        insta::assert_debug_snapshot!(lex("/"));
    }
    #[test]
    fn eq() {
        insta::assert_debug_snapshot!(lex("="));
    }
    #[test]
    fn eq_eq() {
        insta::assert_debug_snapshot!(lex("=="));
    }
    #[test]
    fn bang_eq() {
        insta::assert_debug_snapshot!(lex("!="));
    }
    #[test]
    fn lt() {
        insta::assert_debug_snapshot!(lex("<"));
    }
    #[test]
    fn lt_eq() {
        insta::assert_debug_snapshot!(lex("<="));
    }
    #[test]
    fn gt() {
        insta::assert_debug_snapshot!(lex(">"));
    }
    #[test]
    fn gt_eq() {
        insta::assert_debug_snapshot!(lex(">="));
    }
    #[test]
    fn arrow() {
        insta::assert_debug_snapshot!(lex("->"));
    }
    #[test]
    fn arithmetic_expression() {
        insta::assert_debug_snapshot!(lex("a + b * c"));
    }
    #[test]
    fn comparison_expression() {
        insta::assert_debug_snapshot!(lex("a == b and c != d"));
    }
}

mod delimiters {
    use crate::lex;

    #[test]
    fn parens() {
        insta::assert_debug_snapshot!(lex("()"));
    }
    #[test]
    fn brackets() {
        insta::assert_debug_snapshot!(lex("[]"));
    }
    #[test]
    fn curly() {
        insta::assert_debug_snapshot!(lex("{}"));
    }
    #[test]
    fn comma() {
        insta::assert_debug_snapshot!(lex(","));
    }
    #[test]
    fn colon() {
        insta::assert_debug_snapshot!(lex(":"));
    }
    #[test]
    fn dot() {
        insta::assert_debug_snapshot!(lex("."));
    }
    #[test]
    fn semicolon() {
        insta::assert_debug_snapshot!(lex(";"));
    }
}

mod keywords {
    use crate::lex;

    #[test]
    fn and() {
        insta::assert_debug_snapshot!(lex("and"));
    }
    #[test]
    fn or() {
        insta::assert_debug_snapshot!(lex("or"));
    }
    #[test]
    fn not() {
        insta::assert_debug_snapshot!(lex("not"));
    }
    #[test]
    fn not_x() {
        insta::assert_debug_snapshot!(lex("not x"));
    }
    #[test]
    fn similar_idents() {
        insta::assert_debug_snapshot!(lex("andy ore notice"));
    }
}

mod identifiers {
    use crate::lex;

    #[test]
    fn basic() {
        insta::assert_debug_snapshot!(lex("foo"));
    }
    #[test]
    fn underscore_start() {
        insta::assert_debug_snapshot!(lex("_foo"));
    }
    #[test]
    fn with_digits() {
        insta::assert_debug_snapshot!(lex("foo123"));
    }
    #[test]
    fn just_underscore() {
        insta::assert_debug_snapshot!(lex("_"));
    }
}

mod extended_identifiers {
    use crate::lex;

    #[test]
    fn hyphenated() {
        insta::assert_debug_snapshot!(lex("@kebab-name"));
    }
    #[test]
    fn no_hyphen() {
        insta::assert_debug_snapshot!(lex("@foo"));
    }
    #[test]
    fn leading_digit() {
        insta::assert_debug_snapshot!(lex("@123-foo"));
    }
    #[test]
    fn member_access() {
        insta::assert_debug_snapshot!(lex("obj.@kebab-name"));
    }

    mod errors {
        use crate::lex;

        #[test]
        fn empty_at_eof() {
            insta::assert_debug_snapshot!(lex("@"));
        }
        #[test]
        fn empty_before_space() {
            insta::assert_debug_snapshot!(lex("@ foo"));
        }
        #[test]
        fn empty_before_punct() {
            insta::assert_debug_snapshot!(lex("@+"));
        }
    }
}

mod comments {
    use crate::lex;

    #[test]
    fn line_comment() {
        insta::assert_debug_snapshot!(lex("# a comment"));
    }
    #[test]
    fn comment_then_newline() {
        insta::assert_debug_snapshot!(lex("# a comment\nfoo"));
    }
    #[test]
    fn comment_after_code() {
        insta::assert_debug_snapshot!(lex("foo # tail"));
    }
}

mod strings {
    use crate::lex;

    #[test]
    fn basic() {
        insta::assert_debug_snapshot!(lex(r#""hello""#));
    }
    #[test]
    fn empty() {
        insta::assert_debug_snapshot!(lex(r#""""#));
    }
    #[test]
    fn escape_newline() {
        insta::assert_debug_snapshot!(lex(r#""hello\nworld""#));
    }
    #[test]
    fn escape_tab() {
        insta::assert_debug_snapshot!(lex(r#""hello\tworld""#));
    }
    #[test]
    fn escape_backslash() {
        insta::assert_debug_snapshot!(lex(r#""a\\b""#));
    }
    #[test]
    fn escape_quote() {
        insta::assert_debug_snapshot!(lex(r#""a\"b""#));
    }

    mod errors {
        use crate::lex;

        #[test]
        fn raw_newline() {
            insta::assert_debug_snapshot!(lex("\"hello\nworld\""));
        }
        #[test]
        fn unterminated() {
            insta::assert_debug_snapshot!(lex(r#""hello"#));
        }
        #[test]
        fn bad_escape() {
            insta::assert_debug_snapshot!(lex(r#""\x""#));
        }
        #[test]
        fn unterminated_escape() {
            insta::assert_debug_snapshot!(lex("\"hello\\"));
        }
    }
}

mod reserved {
    use crate::lex;

    #[test]
    fn backtick() {
        insta::assert_debug_snapshot!(lex("`"));
    }
    #[test]
    fn dollar() {
        insta::assert_debug_snapshot!(lex("$"));
    }
}
