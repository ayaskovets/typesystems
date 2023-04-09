pub use tokenstream::{Parser, Stream, Tokenizer};
pub use typesystems::*;

use std::str::FromStr;

fn test() -> Result<(), String> {
    struct Testing {
        env: Env<Type>,
        gen: Gen<Type>,
    }

    impl Testing {
        pub fn new() -> Self {
            Testing {
                env: Env::new(),
                gen: Gen::new(),
            }
        }

        pub fn define(&mut self, name: &str, ty: &str) -> Result<(), String> {
            self.env.insert(name, Type::from_str(ty, &mut self.gen)?);
            Ok(())
        }

        pub fn deduce(&mut self, code: &str) -> Result<Type, String> {
            Ok(infer(&Term::from_str(code)?, &mut self.env, &mut self.gen)?)
        }
    }

    let mut testing = Testing::new();

    testing.define("head", "forall[a] list[a] -> a")?;
    testing.define("tail", "forall[a] list[a] -> list[a]")?;
    testing.define("nil", "forall[a] list[a]")?;
    testing.define("cons", "forall[a] (a, list[a]) -> list[a]")?;
    testing.define("cons_curry", "forall[a] a -> list[a] -> list[a]")?;
    testing.define("map", "forall[a, b] (a -> b, list[a]) -> list[b]")?;
    testing.define("map_curry", "forall[a, b] (a -> b) -> list[a] -> list[b]")?;
    testing.define("one", "int")?;
    testing.define("zero", "int")?;
    testing.define("succ", "int -> int")?;
    testing.define("plus", "(int, int) -> int")?;
    testing.define("eq", "forall[a] (a, a) -> bool")?;
    testing.define("eq_curry", "forall[a] a -> a -> bool")?;
    testing.define("not", "bool -> bool")?;
    testing.define("true", "bool")?;
    testing.define("false", "bool")?;
    testing.define("pair", "forall[a, b] (a, b) -> pair[a, b]")?;
    testing.define("pair_curry", "forall[a, b] a -> b -> pair[a, b]")?;
    testing.define("first", "forall[a, b] pair[a, b] -> a")?;
    testing.define("second", "forall[a, b] pair[a, b] -> b")?;
    testing.define("id", "forall[a] a -> a")?;
    testing.define("const", "forall[a, b] a -> b -> a")?;
    testing.define("apply", "forall[a, b] (a -> b, a) -> b")?;
    testing.define("apply_curry", "forall[a, b] (a -> b) -> a -> b")?;
    testing.define("choose", "forall[a] (a, a) -> a")?;
    testing.define("choose_curry", "forall[a] a -> a -> a")?;

    let mut testcase = |code: &str, result: &str| -> Result<(), String> {
        match testing.deduce(code) {
            Ok(ty) => {
                let t_signature = Type::from_str(result, &mut testing.gen)?;
                assert_eq!(ty.to_string(), t_signature.to_string());
            }
            Err(error) => {
                assert_eq!(error, result)
            }
        }
        Ok(())
    };

    testcase(r"id", r"forall[a] a -> a")?;
    testcase(r"one", r"int")?;
    testcase(r"x", r"Undefined variable 'x'")?;
    testcase(r"let x = x in x", r"Undefined variable 'x'")?;
    testcase(r"let x = id in x", r"forall[a] a -> a")?;
    testcase(r"let x = \y -> y in x", r"forall[a] a -> a")?;
    testcase(r"\x -> x", r"forall[a] a -> a")?;
    testcase(r"\x -> x", r"forall[int] int -> int")?;
    testcase(r"pair", r"forall[a, b] (a, b) -> pair[a, b]")?;
    testcase(r"pair", r"forall[z, x] (x, z) -> pair[x, z]")?;
    testcase(r"\x -> let y = \z -> z in y", r"forall[a, b] a -> b -> b")?;
    testcase(r"let f = \x -> x in let id = \y -> y in eq(f, id)", r"bool")?;
    testcase(
        r"let f = \x -> x in let id = \y -> y in eq_curry(f)(id)",
        r"bool",
    )?;
    testcase(r"let f = \x -> x in eq(f, succ)", r"bool")?;
    testcase(r"let f = \x -> x in eq_curry(f)(succ)", r"bool")?;
    testcase(r"\x -> x(false)", r"forall[a] (bool -> a) -> a")?;
    testcase(
        r"let f = \x -> x in pair(f(one), f(true))",
        r"pair[int, bool]",
    )?;
    testcase(r"let f = id in pair(f(one), f(true))", r"pair[int, bool]")?;
    testcase(
        r"\f -> pair(f(one), f(true))",
        r"Cannot unify int with bool",
    )?;
    testcase(
        r"let f = \x y -> let a = eq(x, y) in eq(x, y) in f",
        r"forall[a] (a, a) -> bool",
    )?;
    testcase(
        r"let f = \x y -> let a = eq_curry(x)(y) in eq_curry(x)(y) in f",
        r"forall[a] (a, a) -> bool",
    )?;
    testcase(r"id(id)", r"forall[a] a -> a")?;
    testcase(r"choose(\x y -> x, \x y -> y)", r"forall[a] (a, a) -> a")?;
    testcase(
        r"choose_curry(\x y -> x)(\x y -> y)",
        r"forall[a] (a, a) -> a",
    )?;
    testcase(
        r"let x = id in let y = let z = x(id) in z in y",
        r"forall[a] a -> a",
    )?;
    testcase(r"cons(id, nil)", r"forall[a] list[a -> a]")?;
    testcase(r"cons_curry(id)(nil)", r"forall[a] list[a -> a]")?;
    testcase(
        r"let lst1 = cons(id, nil) in let lst2 = cons(succ, lst1) in lst2",
        r"list[int -> int]",
    )?;
    testcase(
        r"cons_curry(id)(cons_curry(succ)(cons_curry(id)(nil)))",
        r"list[int -> int]",
    )?;
    testcase(r"plus(one, true)", "Cannot unify int with bool")?;
    testcase(r"plus(one)", "Incorrect number of arguments. Must be 1")?;
    testcase(r"\x -> let y = x in y", r"forall[a] a -> a")?;
    testcase(
        r"\x -> let y = let z = x(\x -> x) in z in y",
        r"forall[a, b] ((a -> a) -> b) -> b",
    )?;
    testcase(
        r"\x -> \y -> let x = x(y) in x(y)",
        r"forall[a, b] (a -> a -> b) -> a -> b",
    )?;
    testcase(r"\x -> let y = \z -> x in y", r"forall[a, b] a -> b -> a")?;
    testcase(
        r"\x -> \y -> let x = x(y) in \x -> y(x)",
        r"forall[a, b, c] ((a -> b) -> c) -> (a -> b) -> a -> b",
    )?;
    testcase(
        r"\x -> let y = \z -> z in y(y)",
        r"forall[a, b] a -> b -> b",
    )?;
    testcase(r"one(id)", "Invalid type of function: int")?;
    testcase(
        r"let const = \x -> \y -> x in const",
        r"forall[a, b] a -> b -> a",
    )?;
    testcase(
        r"let apply = \f x -> f(x) in apply",
        r"forall[a, b] (a -> b, a) -> b",
    )?;
    testcase(
        r"let apply_curry = \f -> \x -> f(x) in apply_curry",
        r"forall[a, b] (a -> b) -> a -> b",
    )?;
    testcase(
        r"\x -> let y = \z -> x(z) in y",
        r"forall[a, b] (a -> b) -> a -> b",
    )?;
    testcase(
        r"\f -> let x = \g y -> let _ = g(y) in eq(f, g) in x",
        r"forall[a, b] (a -> b) -> (a -> b, a) -> bool",
    )?;
    testcase(r"\x -> let y = x in y(y)", "Infinite type")?;
    testcase(r"\x -> x(x)", "Infinite type")?;

    Ok(())
}

fn main() {
    match test() {
        Ok(_) => {}
        Err(error) => println!("{}", error),
    };
}
