/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod stream;
use stream::Stream;

mod token;
use token::Token;

use std::str::Chars;

pub struct Lexer<'a> {
    stream: Stream<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            stream: Stream::new(chars),
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

    fn startswith(&mut self, c: char) -> Token {
        match c {
            // single-char tokens
            ',' => Token::Comma,
            '.' => Token::Dot,
            '(' => Token::Lparen,
            ')' => Token::Rparen,
            '[' => Token::Lbracket,
            ']' => Token::Rbracket,
            '=' => Token::Equals,
            ' ' | '\t' | '\r' => {
                self.take_while(|x| matches!(x, ' ' | '\t' | '\r'));
                Token::Spacing
            }
            '\n' => {
                self.take_while(|x| matches!(x, '\n'));
                Token::Newline
            }
            // operators / etc
            '-' => {
                todo!()
            }
            // keywords / identifiers
            'a'..='z' | 'A'..='Z' | '_' => {
                self.stream.undo(1);
                let ident =
                    self.take_while(|x| matches!(x, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'));
                match ident.as_str() {
                    "fn" => Token::Fn,
                    "forall" => Token::Forall,
                    "in" => Token::In,
                    "let" => Token::Let,
                    _ => Token::Ident(ident),
                }
            }
            // numbers
            '0'..='9' => {
                self.stream.undo(1);
                let mut number = self.take_while(|x| matches!(x, '0'..='9'));

                if let Some('.') = self.stream.next() {
                    let fraction = self.take_while(|x| matches!(x, '0'..='9'));
                    if !fraction.is_empty() {
                        number.push('.');
                        number.push_str(fraction.as_str());
                    } else {
                        self.stream.undo(1);
                    }
                } else {
                    self.stream.undo(1);
                }

                if let Some('e') = self.stream.next() {
                    let exponent = self.take_while(|x| matches!(x, '0'..='9'));
                    if !exponent.is_empty() {
                        number.push('e');
                        number.push_str(exponent.as_str());
                    }
                } else {
                    self.stream.undo(1);
                }

                Token::Number(number)
            }
            _ => panic!("Unexpected character '{}'", c),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.stream.next().and_then(|c| {
            let token = Some(self.startswith(c));
            self.stream.commit();
            token
        })
    }
}

pub fn test() {
    let cs = "abcdefgh";
    let mut s = Stream::new(cs.chars());
    println!("{}", s.next().unwrap());
    println!("{}", s.next().unwrap());
    s.undo(2);
    println!("r: {}", s.next().unwrap());
    println!("{}", s.next().unwrap());
    s.commit();
    s.undo(1);
    println!("c: {}", s.next().unwrap());
    s.undo(1);
    println!("r: {}", s.next().unwrap());
    println!("{}", s.next().unwrap());
    println!("{}", s.next().unwrap());
    s.commit();
    println!("c1: {}", s.next().unwrap());
    println!("{}", s.next().unwrap());
    println!("{}", s.next().unwrap());
    s.undo(3);
    println!("r: {}", s.next().unwrap());
}
