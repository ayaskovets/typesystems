/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

pub use parser_stream::{Stream, Streamable};

#[derive(PartialEq, Clone, Debug)]
pub enum Token {
    Comma,
    Dot,
    Lparen,
    Rparen,
    Lbracket,
    Rbracket,
    Equals,
    Spacing,
    Newline,
    Arrow,
    Fn,
    Forall,
    In,
    Let,
    Ident(String),
    Number(String),
    Comment(String),
}

impl Streamable<char> for Token {
    #[rustfmt::skip]
    fn from(s: &mut Stream<char>) -> Option<Token> {
           symbol(s).or_else(
        || operator(s).or_else(
        || keyword_or_ident(s).or_else(
        || number(s).or_else(
        || comment(s)))))
    }
}

fn symbol(s: &mut Stream<char>) -> Option<Token> {
    match s.next().unwrap() {
        ',' => Some(Token::Comma),
        '.' => Some(Token::Dot),
        '(' => Some(Token::Lparen),
        ')' => Some(Token::Rparen),
        '[' => Some(Token::Lbracket),
        ']' => Some(Token::Rbracket),
        '=' => Some(Token::Equals),
        ' ' | '\t' | '\r' => {
            s.squash(|x| matches!(x, ' ' | '\t' | '\r'));
            Some(Token::Spacing)
        }
        '\n' => {
            s.squash(|x| matches!(x, '\n'));
            Some(Token::Newline)
        }
        _ => s.fallback(1),
    }
}

fn operator(s: &mut Stream<char>) -> Option<Token> {
    match s.next().unwrap() {
        '-' => match s.next() {
            Some('>') => Some(Token::Arrow),
            None => s.fallback(1),
            _ => s.fallback(2),
        },
        _ => s.fallback(1),
    }
}

fn keyword_or_ident(s: &mut Stream<char>) -> Option<Token> {
    match s.next().unwrap() {
        'a'..='z' | 'A'..='Z' | '_' => {
            s.undo(1);
            let ident: String = s
                .take_while(|x| matches!(x, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'))
                .into_iter()
                .collect();
            match ident.as_str() {
                "fn" => Some(Token::Fn),
                "forall" => Some(Token::Forall),
                "in" => Some(Token::In),
                "let" => Some(Token::Let),
                _ => Some(Token::Ident(ident)),
            }
        }
        _ => s.fallback(1),
    }
}

fn number(s: &mut Stream<char>) -> Option<Token> {
    match s.next().unwrap() {
        '-' => match s.next() {
            Some('0'..='9') => {
                s.undo(1);
                if let Some(Token::Number(mut number)) = number(s) {
                    number.insert(0, '-');
                    return Some(Token::Number(number));
                }
                unreachable!()
            }
            None => s.fallback(1),
            _ => s.fallback(2),
        },
        '0'..='9' => {
            s.undo(1);
            let mut number: String = s
                .take_while(|x| matches!(x, '0'..='9'))
                .into_iter()
                .collect();

            match s.next() {
                Some('.') => {
                    let fraction: String = s
                        .take_while(|x| matches!(x, '0'..='9'))
                        .into_iter()
                        .collect();
                    if !fraction.is_empty() {
                        number.push('.');
                        number.push_str(fraction.as_str());
                    }
                }
                None => (),
                _ => s.undo(1),
            };

            match s.next() {
                Some('e') => {
                    let exponent: String = s
                        .take_while(|x| matches!(x, '0'..='9'))
                        .into_iter()
                        .collect();
                    if !exponent.is_empty() {
                        number.push('e');
                        number.push_str(exponent.as_str());
                    }
                }
                None => (),
                _ => s.undo(1),
            };

            Some(Token::Number(number))
        }
        _ => s.fallback(1),
    }
}

fn comment(s: &mut Stream<char>) -> Option<Token> {
    match s.next().unwrap() {
        '-' => match s.next() {
            Some('-') => Some(Token::Comment(
                s.take_while(|&x| x != '\n').into_iter().collect::<String>(),
            )),
            None => s.fallback(1),
            _ => s.fallback(2),
        },
        _ => s.fallback(1),
    }
}

#[cfg(test)]
mod tests {
    use super::super::Lexer;
    use super::{Token, Token::*};

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
