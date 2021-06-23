/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

#[derive(Debug)] // TODO
pub enum Token {
    // single-char tokens
    Comma,
    Dot,
    Lparen,
    Rparen,
    Lbracket,
    Rbracket,
    Equals,
    Spacing,
    Newline,
    // operators
    Arrow,
    // keywords
    Fn,
    Forall,
    In,
    Let,
    // variable-size tokens
    Ident(String),
    Number(String),
    Comment(String),
}
