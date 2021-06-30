/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod stream;
pub use stream::*;

mod tokenizer;
pub use tokenizer::*;

mod util;
pub use util::*;

pub type Lexer<'a, Token> = Tokenizer<'a, char, Token>;
pub type Parser<'a, Token, Expr> = Tokenizer<'a, Token, Expr>;