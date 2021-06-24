/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

#[cfg(test)]
mod tests {
    mod lexer {
        use typesystems::{Lexer, Token, Token::*};
        fn collect(s: &str) -> Vec<Token> {
            Lexer::new(s.chars()).collect()
        }

        #[test]
        fn symbols() {
            assert_eq!(
                collect(",.()[]="),
                vec![Comma, Dot, Lparen, Rparen, Lbracket, Rbracket, Equals]
            );
        }

        #[test]
        fn spacing() {
            assert_eq!(collect(" \t\r \r\t \n\n\n"), vec![Spacing, Newline]);
        }

        #[test]
        fn keywords() {
            assert_eq!(
                collect("let->fn forall in"),
                vec![Let, Arrow, Fn, Spacing, Forall, Spacing, In]
            );
        }

        #[test]
        fn identifiers() {
            assert_eq!(
                collect("_1abc -- ab"),
                vec![
                    Ident("_1abc".to_owned()),
                    Spacing,
                    Comment(" ab".to_owned())
                ]
            );
            assert_eq!(
                collect("abc_def_42_--abc"),
                vec![Ident("abc_def_42_".to_owned()), Comment("abc".to_owned())]
            );
        }

        #[test]
        fn numbers() {
            assert_eq!(
                collect("42 -0.42.42.0"),
                vec![
                    Number("42".to_owned()),
                    Spacing,
                    Number("-0.42".to_owned()),
                    Dot,
                    Number("42.0".to_owned())
                ]
            );
            assert_eq!(
                collect("-42.0e12.-42.0.e2"),
                vec![
                    Number("-42.0e12".to_owned()),
                    Dot,
                    Number("-42.0".to_owned()),
                    Dot,
                    Ident("e2".to_owned())
                ]
            )
        }
    }
}
