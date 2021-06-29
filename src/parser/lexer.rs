/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::stream::Stream;
use super::token::Token;

pub struct Lexer<'a> {
    stream: Stream<'a, char>,
}

impl<'a> Lexer<'a> {
    pub fn new<I>(chars: I) -> Self
    where
        I: Iterator<Item = char> + 'a,
    {
        Self {
            stream: Stream::new(chars),
        }
    }
}

impl<'a> Lexer<'a> {
    fn fallback(&mut self, len: usize) -> Option<Token> {
        self.stream.undo(len);
        None
    }

    fn squash<P>(&mut self, predicate: P)
    where
        P: Fn(char) -> bool,
    {
        loop {
            if let Some(c) = self.stream.next() {
                if predicate(c) {
                    continue;
                } else {
                    self.stream.undo(1);
                    break;
                }
            }
            break;
        }
    }

    fn take_while<P>(&mut self, predicate: P) -> String
    where
        P: Fn(char) -> bool,
    {
        let mut taken = String::new();
        loop {
            if let Some(c) = self.stream.next() {
                if predicate(c) {
                    taken.push(c);
                    continue;
                } else {
                    self.stream.undo(1);
                    break;
                }
            }
            break;
        }
        taken
    }
}

impl<'a> Lexer<'a> {
    fn symbol(&mut self) -> Option<Token> {
        match self.stream.next().unwrap() {
            ',' => Some(Token::Comma),
            '.' => Some(Token::Dot),
            '(' => Some(Token::Lparen),
            ')' => Some(Token::Rparen),
            '[' => Some(Token::Lbracket),
            ']' => Some(Token::Rbracket),
            '=' => Some(Token::Equals),
            ' ' | '\t' | '\r' => {
                self.squash(|x| matches!(x, ' ' | '\t' | '\r'));
                Some(Token::Spacing)
            }
            '\n' => {
                self.squash(|x| matches!(x, '\n'));
                Some(Token::Newline)
            }
            _ => self.fallback(1),
        }
    }

    fn operator(&mut self) -> Option<Token> {
        match self.stream.next().unwrap() {
            '-' => match self.stream.next() {
                Some('>') => Some(Token::Arrow),
                None => self.fallback(1),
                _ => self.fallback(2),
            },
            _ => self.fallback(1),
        }
    }

    fn kw_or_ident(&mut self) -> Option<Token> {
        match self.stream.next().unwrap() {
            'a'..='z' | 'A'..='Z' | '_' => {
                self.stream.undo(1);
                let ident =
                    self.take_while(|x| matches!(x, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'));
                match ident.as_str() {
                    "fn" => Some(Token::Fn),
                    "forall" => Some(Token::Forall),
                    "in" => Some(Token::In),
                    "let" => Some(Token::Let),
                    _ => Some(Token::Ident(ident)),
                }
            }
            _ => self.fallback(1),
        }
    }

    fn number(&mut self) -> Option<Token> {
        match self.stream.next().unwrap() {
            '-' => match self.stream.next() {
                Some('0'..='9') => {
                    self.stream.undo(1);
                    if let Some(Token::Number(mut number)) = self.number() {
                        number.insert(0, '-');
                        return Some(Token::Number(number));
                    }
                    unreachable!()
                }
                None => self.fallback(1),
                _ => self.fallback(2),
            },
            '0'..='9' => {
                self.stream.undo(1);
                let mut number = self.take_while(|x| matches!(x, '0'..='9'));

                match self.stream.next() {
                    Some('.') => {
                        let fraction = self.take_while(|x| matches!(x, '0'..='9'));
                        if !fraction.is_empty() {
                            number.push('.');
                            number.push_str(fraction.as_str());
                        }
                    }
                    None => (),
                    _ => self.stream.undo(1),
                };

                match self.stream.next() {
                    Some('e') => {
                        let exponent = self.take_while(|x| matches!(x, '0'..='9'));
                        if !exponent.is_empty() {
                            number.push('e');
                            number.push_str(exponent.as_str());
                        }
                    }
                    None => (),
                    _ => self.stream.undo(1),
                };

                Some(Token::Number(number))
            }
            _ => self.fallback(1),
        }
    }

    fn comment(&mut self) -> Option<Token> {
        match self.stream.next().unwrap() {
            '-' => match self.stream.next() {
                Some('-') => Some(Token::Comment(self.take_while(|x| x != '\n'))),
                None => self.fallback(1),
                _ => self.fallback(2),
            },
            _ => self.fallback(1),
        }
    }
}

#[rustfmt::skip]
impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.stream.next().and_then(|c| {
            self.stream.undo(1);
            let token = self.symbol().or_else(
                     || self.operator().or_else(
                     || self.kw_or_ident().or_else(
                     || self.number().or_else(
                     || self.comment().or_else(
                     || panic!("Unexpected character '{}'", c))))));
            self.stream.commit();
            token
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, Token, Token::*};
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