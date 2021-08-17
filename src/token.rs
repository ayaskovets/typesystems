/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use tokenstream::{Stream, Streamable};

fn identifier(s: &mut Stream<char>) -> String {
    let mut ident = String::new();
    if let Some(c) = s.next() {
        match c {
            'a'..='z' | 'A'..='Z' | '_' => {
                s.undo(1);
                ident = s
                    .take(|x| matches!(x, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'))
                    .into_iter()
                    .collect();
            }
            _ => s.undo(1),
        }
    };
    ident
}

fn number(s: &mut Stream<char>) -> String {
    let mut num = String::new();
    if let Some(c) = s.next() {
        match c {
            '0'..='9' => {
                s.undo(1);
                num = s.take(|x| matches!(x, '0'..='9')).into_iter().collect();

                match s.next() {
                    Some('.') => {
                        let fraction: String =
                            s.take(|x| matches!(x, '0'..='9')).into_iter().collect();
                        if !fraction.is_empty() {
                            num.push('.');
                            num.push_str(fraction.as_str());
                        }
                    }
                    None => (),
                    _ => s.undo(1),
                };

                match s.next() {
                    Some('e') => {
                        let exponent: String =
                            s.take(|x| matches!(x, '0'..='9')).into_iter().collect();
                        if !exponent.is_empty() {
                            num.push('e');
                            num.push_str(exponent.as_str());
                        }
                    }
                    None => (),
                    _ => s.undo(1),
                };
            }
            _ => s.undo(1),
        }
    };
    num
}

#[derive(PartialEq, Clone, Debug)]
pub enum Token {
    Comma,
    Colon,
    Semicolon,
    Dot,
    Lparen,
    Rparen,
    Lbracket,
    Rbracket,
    Lbrace,
    Rbrace,
    Plus,
    Star,
    Slash,
    Percent,
    Equals,
    GT,
    LT,
    GE,
    LE,
    EQ,
    NE,
    Spacing,
    Newline,
    Arrow,
    Backslash,
    Forall,
    In,
    Let,
    If,
    Then,
    Else,
    Not,
    And,
    Or,
    True,
    False,
    Ident(String),
    Number(String),
    Comment(String),
}

fn keyword(s: &str) -> Option<Token> {
    match s {
        "forall" => Some(Token::Forall),
        "in" => Some(Token::In),
        "let" => Some(Token::Let),
        "if" => Some(Token::If),
        "then" => Some(Token::Then),
        "else" => Some(Token::Else),
        "not" => Some(Token::Not),
        "and" => Some(Token::And),
        "or" => Some(Token::Or),
        "true" => Some(Token::True),
        "false" => Some(Token::False),
        _ => None,
    }
}

// fixed lexer token tree, a little messy but fast
impl Streamable<char> for Token {
    fn from(s: &mut Stream<char>) -> Option<Token> {
        s.next().and_then(|c| match c {
            ',' => Some(Token::Comma),
            ':' => Some(Token::Colon),
            ';' => Some(Token::Semicolon),
            '.' => Some(Token::Dot),
            '(' => Some(Token::Lparen),
            ')' => Some(Token::Rparen),
            '[' => Some(Token::Lbracket),
            ']' => Some(Token::Rbracket),
            '{' => Some(Token::Lbrace),
            '}' => Some(Token::Rbrace),
            '+' => Some(Token::Plus),
            '*' => Some(Token::Star),
            '/' => Some(Token::Slash),
            '%' => Some(Token::Percent),
            '\\' => Some(Token::Backslash),
            c1 @ ('<' | '>' | '=' | '!') => {
                let c1_token = match c1 {
                    '<' => Some(Token::LT),
                    '>' => Some(Token::GT),
                    '=' => Some(Token::Equals),
                    _ => None,
                };
                match s.next() {
                    None => c1_token,
                    Some('=') => match c1 {
                        '<' => Some(Token::LE),
                        '>' => Some(Token::GE),
                        '=' => Some(Token::EQ),
                        '!' => Some(Token::NE),
                        _ => unreachable!("All the cases were handled"),
                    },
                    Some(_) => {
                        s.undo(1);
                        c1_token
                    }
                }
            }
            ' ' | '\t' | '\r' => {
                s.skip(|x| matches!(x, ' ' | '\t' | '\r'));
                Some(Token::Spacing)
            }
            '\n' => {
                s.skip(|x| matches!(x, '\n'));
                Some(Token::Newline)
            }
            '-' => match s.next() {
                None => {
                    s.undo(1);
                    None
                }
                Some('>') => Some(Token::Arrow),
                Some('-') => Some(Token::Comment(
                    s.take(|x| !matches!(x, '\n')).into_iter().collect(),
                )),
                Some('0'..='9') => {
                    s.undo(1);
                    Some(Token::Number("-".to_owned() + &number(s)))
                }
                Some(_) => {
                    s.undo(2);
                    None
                }
            },
            'a'..='z' | 'A'..='Z' | '_' => {
                s.undo(1);
                let ident = identifier(s);
                keyword(ident.as_str()).or_else(|| Some(Token::Ident(ident)))
            }
            '0'..='9' => {
                s.undo(1);
                Some(Token::Number(number(s)))
            }
            _ => {
                s.undo(1);
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Token, Token::*};
    use tokenstream::Tokenizer;

    fn collect(s: &str) -> Vec<Token> {
        Tokenizer::new(s.chars()).collect()
    }

    #[test]
    fn operators() {
        assert_eq!(
            collect("<=>%.+*!===:;"),
            vec![LE, GT, Percent, Dot, Plus, Star, NE, EQ, Colon, Semicolon]
        );
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
            collect(r"let->\ forall in"),
            vec![Let, Arrow, Backslash, Spacing, Forall, Spacing, In]
        );
        assert_eq!(
            collect("if a or b"),
            vec![
                If,
                Spacing,
                Ident("a".to_owned()),
                Spacing,
                Or,
                Spacing,
                Ident("b".to_owned())
            ]
        );
        assert_eq!(
            collect("then c else and"),
            vec![
                Then,
                Spacing,
                Ident("c".to_owned()),
                Spacing,
                Else,
                Spacing,
                And
            ]
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
