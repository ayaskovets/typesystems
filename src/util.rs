/*
 * Copyright (c) 2021, Andrei Yaskovets
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use tokenstream::Stream;

pub fn identifier(s: &mut Stream<char>) -> String {
    let mut ident = String::new();
    match s.next().unwrap() {
        'a'..='z' | 'A'..='Z' | '_' => {
            s.undo(1);
            ident = s
                .take_while(|x| matches!(x, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'))
                .into_iter()
                .collect();
        }
        _ => s.undo(1),
    }
    ident
}

pub fn number(s: &mut Stream<char>) -> String {
    let mut num = String::new();
    match s.next().unwrap() {
        '0'..='9' => {
            s.undo(1);
            num = s
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
                        num.push('.');
                        num.push_str(fraction.as_str());
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
    num
}
