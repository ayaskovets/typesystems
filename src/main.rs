/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod lexer;

fn main() {
    lexer::test();

    let lex = lexer::Lexer::new("forall a. a -> a".chars());
    for t in lex {
        println!("{:?}", t);
    }
}
